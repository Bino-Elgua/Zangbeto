use serde::{Serialize, Deserialize};
use vm_core::ir::{StateHash, OrishaId, CanonicalValue};
use replay_engine::ExecutionTrace;
use crdt_memory::ConflictSet;
use uuid::Uuid;
use crate::action_ladder::EnforcementAction;

/// 🧿 Anomaly: a detected deviation from expected reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_id: Uuid,
    pub detection_timestamp: u64,
    pub source: AnomalySource,
    pub severity: AnomalySeverity,
    pub classification: AnomalyClassification,
    pub evidence: AnomalyEvidence,
    pub affected_paths: Vec<String>,
    pub recommended_action: EnforcementAction,
    pub confidence: f64,  // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySource {
    #[serde(rename = "replay_mismatch")]
    ReplayMismatch { trace_id: Uuid },
    #[serde(rename = "crdt_conflict")]
    CrdtConflict { merge_id: Uuid },
    #[serde(rename = "policy_violation")]
    PolicyViolation { orisha: OrishaId, rule_id: String },
    #[serde(rename = "economic_invariant")]
    EconomicInvariant { invariant: String, observed: CanonicalValue },
    #[serde(rename = "temporal_drift")]
    TemporalDrift { expected_window: (u64, u64), actual: u64 },
    #[serde(rename = "capability_breach")]
    CapabilityBreach { token_id: Uuid, attempted_op: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnomalySeverity {
    #[serde(rename = "observational")]
    Observational,  // Log only
    #[serde(rename = "warning")]
    Warning,        // Flag for review
    #[serde(rename = "critical")]
    Critical,       // Immediate quarantine
    #[serde(rename = "catastrophic")]
    Catastrophic,   // Rollback + punish
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "details")]
pub enum AnomalyClassification {
    #[serde(rename = "schema_drift")]
    SchemaDrift { expected_schema: String, observed_schema: String },
    #[serde(rename = "ethical_violation")]
    EthicalViolation { principle: String, context: CanonicalValue },
    #[serde(rename = "economic_anomaly")]
    EconomicAnomaly { metric: String, deviation: f64 },
    #[serde(rename = "concurrency_conflict")]
    ConcurrencyConflict { nodes: Vec<String>, conflict_type: String },
    #[serde(rename = "temporal_inconsistency")]
    TemporalInconsistency { clock_skew_ms: i64 },
    #[serde(rename = "capability_escape")]
    CapabilityEscape { granted_ops: Vec<String>, attempted: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyEvidence {
    pub trace_snapshot: Option<ExecutionTrace>,
    pub state_before: StateHash,
    pub state_after: StateHash,
    pub crdt_conflict: Option<ConflictSet>,
    pub policy_reports: Vec<PolicyViolationReport>,
    pub cryptographic_proof: Option<EnforcementProof>,
}

impl AnomalyEvidence {
    pub fn state_snapshot(&self) -> crdt_memory::CrdtMemory {
        // In production: extract from trace or state store
        crdt_memory::CrdtMemory::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationReport {
    pub orisha: OrishaId,
    pub rule_id: String,
    pub violation_details: String,
    pub severity_weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementProof {
    pub anomaly_hash: [u8; 32],
    pub validator_signatures: Vec<OrishaSignature>,
    pub merkle_root: [u8; 32],  // of evidence tree
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrishaSignature {
    pub orisha: OrishaId,
    pub signature: Vec<u8>,
    pub public_key_hash: [u8; 32],
}
