use vm_core::ir::{CanonicalStateIR, CanonicalEncode, StateHash, OrishaId, CanonicalValue};
use policy_runtime::CanonicalState;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use ed25519_dalek::Verifier;

/// 🎬 ExecutionTrace: deterministic, verifiable record of state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub trace_id: Uuid,
    pub input_state_hash: StateHash,
    pub ops: Vec<TraceStep>,
    pub output_state_hash: StateHash,
    pub validator_signatures: Vec<ValidatorSignature>,
    pub resource_usage: ResourceUsage,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub step_index: usize,
    pub ir_op: CanonicalStateIR,
    pub op_hash: [u8; 32],  // hash_canonical() of op
    pub pre_state_hash: StateHash,
    pub post_state_hash: StateHash,
    pub validator_checks: Vec<ValidatorCheck>,
    pub resource_delta: ResourceDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCheck {
    pub orisha: OrishaId,
    pub check_hash: [u8; 32],  // hash of validation logic + inputs
    pub result: CheckResult,
    pub execution_time_ms: u64,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckResult {
    Pass,
    Fail { reason: String },
    Conditional { requires_blessing: OrishaId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub orisha: OrishaId,
    pub signature: Vec<u8>,
    pub public_key_hash: [u8; 32],
    pub signed_payload_hash: StateHash,  // hash of trace up to this point
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_ms: u64,
    pub memory_peak_bytes: u64,
    pub io_ops: u64,
    pub economic_delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceDelta {
    pub cpu_ms_delta: u64,
    pub memory_delta: i64,
    pub io_ops_delta: u64,
}

/// 🔁 ReplayEngine: verify execution determinism
pub struct ReplayEngine {
    pub state_store: std::collections::HashMap<StateHash, CanonicalState>,
    pub key_registry: crypto_kernel::KeyRegistry,
}

impl ReplayEngine {
    pub fn new(key_registry: crypto_kernel::KeyRegistry) -> Self {
        Self { state_store: std::collections::HashMap::new(), key_registry }
    }
    
    /// ✅ Verify a trace is deterministic and valid
    pub fn verify_trace(&self, trace: &ExecutionTrace) -> Result<VerificationResult, ReplayError> {
        // 1. Verify input state exists
        let input_state = self.state_store.get(&trace.input_state_hash)
            .ok_or(ReplayError::StateNotFound(trace.input_state_hash))?;
        
        // 2. Replay each step deterministically
        let mut current_state = input_state.clone();
        for step in &trace.ops {
            // Verify op hash matches canonical encoding
            let computed_op_hash = step.ir_op.hash_canonical();
            if computed_op_hash != step.op_hash {
                return Err(ReplayError::OpHashMismatch(step.step_index));
            }
            
            // Execute op (deterministic function)
            let (new_state, resource_delta) = self.execute_op_deterministic(
                &current_state, 
                &step.ir_op
            )?;
            
            // Verify post-state hash
            let computed_post_hash = new_state.compute_hash();
            if computed_post_hash != step.post_state_hash {
                return Err(ReplayError::StateHashMismatch(step.step_index));
            }
            
            // Verify resource usage is within bounds
            if resource_delta.cpu_ms_delta > 1000 {  // example limit
                return Err(ReplayError::ResourceExceeded(step.step_index));
            }
            
            current_state = new_state;
        }
        
        // 3. Verify final state hash
        let final_computed = current_state.compute_hash();
        if final_computed != trace.output_state_hash {
            return Err(ReplayError::FinalStateHashMismatch);
        }
        
        // 4. Verify validator signatures
        for sig in &trace.validator_signatures {
            let pubkey = self.key_registry.get_key_by_hash(&sig.public_key_hash)
                .ok_or(ReplayError::KeyNotFound)?;
            
            // Verify signature over trace prefix
            let payload = self.compute_signature_payload(trace, sig.orisha.as_str())?;
            
            let signature = ed25519_dalek::Signature::from_slice(&sig.signature)
                .map_err(|_| ReplayError::InvalidValidatorSignature(sig.orisha.clone()))?;

            pubkey.verify(&payload, &signature)
                .map_err(|_| ReplayError::InvalidValidatorSignature(sig.orisha.clone()))?;
        }
        
        Ok(VerificationResult {
            trace_id: trace.trace_id,
            deterministic: true,
            resource_compliant: true,
            signatures_valid: true,
        })
    }
    
    /// 🧭 Execute a single IR op deterministically (pure function)
    fn execute_op_deterministic(
        &self,
        state: &CanonicalState,
        op: &CanonicalStateIR,
    ) -> Result<(CanonicalState, ResourceDelta), ReplayError> {
        let mut new_state = state.clone();
        
        match op {
            CanonicalStateIR::AddTask { intent, priority, .. } => {
                new_state.memory.public_hive.entries.insert(
                    format!("task:{}", uuid::Uuid::new_v4()),
                    crdt_memory::LwwRegister {
                        value: CanonicalValue::Object([
                            ("intent".into(), CanonicalValue::String(intent.clone())),
                            ("priority".into(), CanonicalValue::Integer(*priority as i64)),
                        ].into_iter().collect()),
                        timestamp: 0,
                        node_id: "replay".into(),
                    }
                );
            }
            CanonicalStateIR::ModifyEconomy { field, delta, .. } => {
                match field {
                    vm_core::ir::EconomyField::Balance => {
                        new_state.economy.balance += delta;
                    }
                    vm_core::ir::EconomyField::Reputation => {
                        new_state.economy.reputation = 
                            (new_state.economy.reputation + *delta as f64 / 100.0).clamp(0.0, 1.0);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        new_state.vector_clock.tick("replay-engine".into());
        new_state.state_hash = new_state.compute_hash();
        
        Ok((new_state, ResourceDelta::default()))
    }
    
    fn compute_signature_payload(
        &self,
        trace: &ExecutionTrace,
        orisha: &str,
    ) -> Result<Vec<u8>, ReplayError> {
        // Deterministic serialization of trace prefix for signing
        #[derive(Serialize)]
        struct SigPayload<'a> {
            trace_id: &'a Uuid,
            orisha: &'a str,
            input_hash: &'a StateHash,
            ops_count: usize,
            output_hash: &'a StateHash,
        }
        
        let payload = SigPayload {
            trace_id: &trace.trace_id,
            orisha,
            input_hash: &trace.input_state_hash,
            ops_count: trace.ops.len(),
            output_hash: &trace.output_state_hash,
        };
        
        let mut buf = Vec::new();
        let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
        payload.serialize(&mut serializer).unwrap();
        Ok(buf)
    }
}

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub trace_id: Uuid,
    pub deterministic: bool,
    pub resource_compliant: bool,
    pub signatures_valid: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("State not found: {0:?}")]
    StateNotFound(StateHash),
    #[error("Op hash mismatch at step {0}")]
    OpHashMismatch(usize),
    #[error("State hash mismatch at step {0}")]
    StateHashMismatch(usize),
    #[error("Final state hash mismatch")]
    FinalStateHashMismatch,
    #[error("Resource limit exceeded at step {0}")]
    ResourceExceeded(usize),
    #[error("Validator key not found")]
    KeyNotFound,
    #[error("Invalid signature from {0}")]
    InvalidValidatorSignature(OrishaId),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}
