use crate::{anomaly::*, action_ladder::*, quarantine::*, arbitration::*};
use replay_engine::ReplayEngine;
use vm_core::ir::{StateHash};
use policy_runtime::{WasmpolicyHost, CanonicalState, ValidationContext};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// 🕯 Zàngbétò Daemon: continuous enforcement loop
pub struct ZangbetoDaemon {
    pub config: DaemonConfig,
    pub replay_engine: Arc<ReplayEngine>,
    pub policy_host: Arc<WasmpolicyHost>,
    pub action_ladder: ActionLadder,
    pub quarantine_mgr: QuarantineManager,
    pub arbitration_engine: ArbitrationEngine,
    pub event_tx: mpsc::Sender<EnforcementEvent>,
    pub event_rx: Option<mpsc::Receiver<EnforcementEvent>>,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub anomaly_check_interval_ms: u64,
    pub quarantine_eval_interval_ms: u64,
    pub max_pending_anomalies: usize,
    pub auto_enforce: bool,
    pub require_human_approval_for: Vec<EnforcementAction>,
}

#[derive(Debug, Clone)]
pub enum EnforcementEvent {
    AnomalyDetected(Anomaly),
    ActionExecuted(EnforcementReceipt),
    QuarantineReleased(QuarantineId),
    ArbitrationCompleted(ArbitrationDecision),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementReceipt {
    pub receipt_id: Uuid,
    pub anomaly_id: Uuid,
    pub action_taken: EnforcementAction,
    pub execution_timestamp: u64,
    pub state_before: StateHash,
    pub state_after: StateHash,
    pub orisha_signatures: Vec<OrishaSignature>,
    pub merkle_proof: [u8; 32],
}

impl ZangbetoDaemon {
    pub fn new(
        config: DaemonConfig,
        replay_engine: Arc<ReplayEngine>,
        policy_host: Arc<WasmpolicyHost>,
        action_ladder: ActionLadder,
        quarantine_mgr: QuarantineManager,
        arbitration_engine: ArbitrationEngine,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            config,
            replay_engine,
            policy_host,
            action_ladder,
            quarantine_mgr,
            arbitration_engine,
            event_tx: tx,
            event_rx: Some(rx),
            running: false,
        }
    }
    
    /// 🚀 Start the enforcement daemon
    pub async fn start(&mut self) -> Result<(), DaemonError> {
        if self.running {
            return Err(DaemonError::AlreadyRunning);
        }
        self.running = true;
        
        let mut rx = self.event_rx.take().unwrap();
        
        // Main event processing loop
        while let Some(event) = rx.recv().await {
            match event {
                EnforcementEvent::AnomalyDetected(anomaly) => {
                    self.handle_anomaly(anomaly).await?;
                }
                EnforcementEvent::QuarantineReleased(_qid) => {
                    // Reintegrate released state...
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    async fn handle_anomaly(&mut self, anomaly: Anomaly) -> Result<(), DaemonError> {
        let decision = self.action_ladder.determine_action(&anomaly);
        
        if !decision.requires_consensus.is_empty() {
            let request = ArbitrationRequest {
                request_id: Uuid::new_v4(),
                state: CanonicalState::default(),
                ops: vec![],
                context: ValidationContext {
                    agent_token: crypto_kernel::CapabilityToken::dummy(),
                    key_registry: crypto_kernel::KeyRegistry::dummy(),
                    timestamp: current_timestamp(),
                    metadata: serde_json::Value::Null,
                },
                conflicting_policies: vec![],
            };
            
            let _arbitration_result = self.arbitration_engine.arbitrate(request).await
                .map_err(|_| DaemonError::ArbitrationFailed)?;
        }
        
        if self.config.auto_enforce && !self.config.require_human_approval_for.contains(&decision.action) {
            let receipt = self.execute_enforcement_action(&decision.action, &anomaly).await?;
            let _ = self.event_tx.send(EnforcementEvent::ActionExecuted(receipt)).await;
        } else {
            let _ = self.event_tx.send(EnforcementEvent::AnomalyDetected(anomaly)).await;
        }
        
        Ok(())
    }
    
    async fn execute_enforcement_action(
        &mut self,
        action: &EnforcementAction,
        anomaly: &Anomaly,
    ) -> Result<EnforcementReceipt, DaemonError> {
        match action {
            EnforcementAction::Observe { .. } => {
                Ok(self.generate_receipt(anomaly, action.clone(), anomaly.evidence.state_before, anomaly.evidence.state_after))
            }
            EnforcementAction::QuarantineState { scope, release_conditions, .. } => {
                let _qid = self.quarantine_mgr.quarantine(
                    &anomaly.evidence.state_snapshot(),
                    scope.clone(),
                    anomaly.classification.clone(),
                    release_conditions.clone(),
                ).map_err(|_| DaemonError::QuarantineFailed)?;
                
                Ok(self.generate_receipt(anomaly, action.clone(), anomaly.evidence.state_before, anomaly.evidence.state_after))
            }
            EnforcementAction::RollbackTransition { to_state, .. } => {
                Ok(self.generate_receipt(anomaly, action.clone(), anomaly.evidence.state_after, *to_state))
            }
            _ => {
                Ok(self.generate_receipt(anomaly, action.clone(), anomaly.evidence.state_before, anomaly.evidence.state_after))
            }
        }
    }
    
    fn generate_receipt(
        &self,
        anomaly: &Anomaly,
        action: EnforcementAction,
        before: StateHash,
        after: StateHash,
    ) -> EnforcementReceipt {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(anomaly.anomaly_id.as_bytes());
        hasher.update(serde_cbor::to_vec(&action).unwrap());
        hasher.update(before);
        hasher.update(after);
        
        EnforcementReceipt {
            receipt_id: Uuid::new_v4(),
            anomaly_id: anomaly.anomaly_id,
            action_taken: action,
            execution_timestamp: current_timestamp(),
            state_before: before,
            state_after: after,
            orisha_signatures: vec![],
            merkle_proof: hasher.finalize().into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("Daemon already running")]
    AlreadyRunning,
    #[error("Arbitration failed")]
    ArbitrationFailed,
    #[error("Quarantine operation failed")]
    QuarantineFailed,
    #[error("Enforcement execution failed")]
    ExecutionFailed,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
