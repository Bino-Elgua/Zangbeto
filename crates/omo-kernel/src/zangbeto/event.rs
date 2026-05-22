use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::kernel::css::CanonicalSystemState;
use crate::kernel::diff::StateDiff;
use crate::zangbeto::drift::DriftReport;
pub use crate::zangbeto::ledger::CommitReceipt as FinalityReceipt;
pub use crate::zangbeto::ledger::CommitReceipt;
use crate::zangbeto::ir::CanonicalStateIR;

/// 🧿 The atomic unit of truth: everything is an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityEvent {
    pub event_id: Uuid,
    pub pre_state_hash: String,          // sha256:CSSₙ
    pub post_state_hash: String,         // sha256:CSSₙ₊₁
    pub fingerprint: TransitionFingerprint,
    pub diff: StateDiff,
    pub validators: Vec<ValidatorReport>,
    pub expected_state: Option<ExpectedState>,  // Ọ̀ṣun prediction
    pub drift_analysis: Option<DriftReport>,    // Zàngbétò classification
    pub repair_delta: Option<StateDiff>,        // Ògún healing patch
    pub finality_receipt: Option<FinalityReceipt>, // Ṣàngó anchor
    pub timestamp: u64,
    pub metadata: serde_json::Value,
}

/// 🧬 Transition Fingerprint: hashes behavioral intent, not just data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionFingerprint {
    pub intent_hash: String,              // sha256(normalized_intent)
    pub validator_set_hash: String,       // sha256(sorted_validator_names)
    pub op_signature: String,             // sha256(CanonicalStateIR ops)
    pub environment_hash: String,         // sha256(external_signals snapshot)
}

impl TransitionFingerprint {
    pub fn compute(
        intent: &str,
        validators: &[&str],
        ir_ops: &[CanonicalStateIR],
        env_signals: &serde_json::Value,
    ) -> Self {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        
        // Intent hash (normalized: lowercase, trimmed, whitespace-collapsed)
        let normalized_intent = intent.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        hasher.update(normalized_intent.as_bytes());
        let intent_hash = format!("sha256:{}", hex::encode(hasher.finalize_reset()));
        
        // Validator set hash (sorted for determinism)
        let mut sorted_validators = validators.to_vec();
        sorted_validators.sort();
        hasher.update(sorted_validators.join(",").as_bytes());
        let validator_set_hash = format!("sha256:{}", hex::encode(hasher.finalize_reset()));
        
        // Op signature (semantic IR, not raw JSON)
        let ir_serialized = serde_json::to_string(ir_ops).unwrap();
        hasher.update(ir_serialized.as_bytes());
        let op_signature = format!("sha256:{}", hex::encode(hasher.finalize_reset()));
        
        // Environment hash
        hasher.update(serde_json::to_string(env_signals).unwrap().as_bytes());
        let environment_hash = format!("sha256:{}", hex::encode(hasher.finalize()));
        
        Self {
            intent_hash,
            validator_set_hash,
            op_signature,
            environment_hash,
        }
    }
}

/// 🧠 Expected State: prophetic model from Ọ̀ṣun (Julia) or rule engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedState {
    pub predicted_css: CanonicalSystemState,
    pub confidence: f64,  // 0.0 - 1.0
    pub source: String,   // "ọ̀ṣun:julia", "rule_engine", "llm:v3"
    pub prediction_metadata: serde_json::Value,
}

/// 📊 Validator Report: standardized output from any Orisha layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorReport {
    pub orisha: String,
    pub passed: bool,
    pub authority_weight: u8,
    pub reasons: Vec<String>,
    pub anomalies_detected: Vec<String>,
    pub integrity_score: f64,  // 0.0 - 1.0
    pub execution_time_ms: u64,
}
