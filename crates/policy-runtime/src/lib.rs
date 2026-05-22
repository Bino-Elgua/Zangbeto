use wasmtime::{Engine, Store, Module, Linker, Config};
use vm_core::ir::{CanonicalStateIR, OrishaId, StateHash};
use crypto_kernel::{CapabilityToken, KeyRegistry};
use serde::{Serialize, Deserialize};
use crdt_memory::{CrdtMemory, VectorClock};

/// 🧿 OrishaPolicy: WASM-hosted validator module interface
#[async_trait::async_trait]
pub trait OrishaPolicy: Send + Sync {
    fn name(&self) -> &'static str;
    fn authority_weight(&self) -> u8;
    
    /// Validate a proposed IR sequence against policy rules
    async fn validate(
        &self,
        state: &CanonicalState,
        ops: &[CanonicalStateIR],
        context: &ValidationContext,
    ) -> PolicyResult;
    
    /// Audit a completed transition for compliance
    async fn audit(
        &self,
        before: &CanonicalState,
        after: &CanonicalState,
        ops: &[CanonicalStateIR],
    ) -> AuditReport;
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub agent_token: CapabilityToken,
    pub key_registry: KeyRegistry,
    pub timestamp: u64,
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub enum PolicyResult {
    Approved { reasons: Vec<String> },
    Rejected { reasons: Vec<String>, suggestion: Option<String> },
    Conditional { requires_blessing: OrishaId, reason: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub orisha: String,
    pub passed: bool,
    pub anomalies: Vec<String>,
    pub integrity_score: f64,
}

/// 🧱 CanonicalState: full state snapshot for validation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalState {
    pub identity: Identity,
    pub memory: CrdtMemory,
    pub economy: EconomyState,
    pub constraints: ConstraintSet,
    pub vector_clock: VectorClock,
    pub state_hash: StateHash,
}

impl CanonicalState {
    pub fn compute_hash(&self) -> StateHash {
        // Simplified hash: in reality, CBOR encode and SHA256
        [0; 32]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Identity {
    pub agent_id: String,
    pub wallet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EconomyState {
    pub balance: i64,
    pub reputation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConstraintSet {
    pub privacy_mode: bool,
    pub sabbath: bool,
}

/// 🧵 WASM Policy Host: loads and executes Orisha modules
pub struct WasmpolicyHost {
    pub engine: Engine,
    pub linker: Linker<PolicyContext>,
    pub policies: std::collections::HashMap<OrishaId, LoadedPolicy>,
}

pub struct PolicyContext {
    pub state: CanonicalState,
    pub ops: Vec<CanonicalStateIR>,
    pub context: ValidationContext,
    pub result: Option<PolicyResult>,
}

pub struct LoadedPolicy {
    pub instance: wasmtime::Instance,
    pub memory: wasmtime::Memory,
    pub validate_fn: wasmtime::TypedFunc<(u32, u32), i32>,  // (ptr, len) -> result_code
}

impl WasmpolicyHost {
    pub fn new() -> Result<Self, PolicyError> {
        let mut config = Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.wasm_reference_types(true);
        
        let engine = Engine::new(&config)?;
        let linker = Linker::new(&engine);
        
        Ok(Self {
            engine,
            linker,
            policies: std::collections::HashMap::new(),
        })
    }
    
    /// 📦 Load a compiled WASM policy module
    pub fn load_policy(&mut self, orisha_id: OrishaId, wasm_bytes: &[u8]) -> Result<(), PolicyError> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        let mut store = Store::new(&self.engine, PolicyContext {
            state: CanonicalState::default(),
            ops: vec![],
            context: ValidationContext {
                agent_token: CapabilityToken::dummy(),
                key_registry: KeyRegistry::dummy(),
                timestamp: 0,
                metadata: serde_json::Value::Null,
            },
            result: None,
        });
        
        let instance = self.linker.instantiate(&mut store, &module)?;
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or(PolicyError::MissingMemory)?;
        let validate_fn = instance.get_typed_func::<(u32, u32), i32>(&mut store, "validate")?;
        
        self.policies.insert(orisha_id, LoadedPolicy {
            instance,
            memory,
            validate_fn,
        });
        
        Ok(())
    }
    
    /// ⚡ Execute policy validation
    pub async fn validate_with_policy(
        &self,
        orisha_id: &OrishaId,
        _state: &CanonicalState,
        _ops: &[CanonicalStateIR],
        _context: &ValidationContext,
    ) -> Result<PolicyResult, PolicyError> {
        let _policy = self.policies.get(orisha_id)
            .ok_or(PolicyError::PolicyNotFound(orisha_id.clone()))?;
        
        // In production: copy to WASM memory, call validate_fn, read result
        Ok(PolicyResult::Approved { reasons: vec![format!("{} policy passed", orisha_id)] })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("WASM compilation failed: {0}")]
    WasmError(#[from] wasmtime::Error),
    #[error("Policy not found: {0}")]
    PolicyNotFound(OrishaId),
    #[error("Missing memory in WASM module")]
    MissingMemory,
    #[error("Missing validate function in WASM module")]
    MissingValidateFn,
    #[error("Serialization failed")]
    SerializationFailed,
}
