use serde::{Serialize, Deserialize};
use vm_core::ir::{StateHash, OrishaId};
use crate::anomaly::{Anomaly, AnomalySeverity, AnomalyClassification};
use uuid::Uuid;

/// ⚔️ Enforcement Action: deterministic response to anomaly
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnforcementAction {
    #[serde(rename = "observe")]
    Observe { log_level: LogLevel, retain_evidence: bool },
    
    #[serde(rename = "flag_for_review")]
    FlagForReview { reviewers: Vec<OrishaId>, timeout_ms: u64 },
    
    #[serde(rename = "quarantine_state")]
    QuarantineState { 
        scope: QuarantineScope, 
        duration_ms: Option<u64>,
        release_conditions: Vec<ReleaseCondition>
    },
    
    #[serde(rename = "rollback_transition")]
    RollbackTransition { 
        to_state: StateHash, 
        preserve_audit_trail: bool,
        compensation_required: bool
    },
    
    #[serde(rename = "punish_agent")]
    PunishAgent {
        agent_id: String,
        penalty: PenaltyType,
        duration_ms: u64,
        appeal_window_ms: Option<u64>
    },
    
    #[serde(rename = "emergency_halt")]
    EmergencyHalt {
        scope: HaltScope,
        require_quorum: Vec<OrishaId>,
        auto_resume_condition: Option<String>
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug, Info, Warn, Error, Critical
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuarantineScope {
    #[serde(rename = "path")]
    Path { path: String },
    #[serde(rename = "subtree")]
    Subtree { root_path: String },
    #[serde(rename = "agent")]
    Agent { agent_id: String },
    #[serde(rename = "branch")]
    Branch { branch_id: String },
    #[serde(rename = "global")]
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReleaseCondition {
    #[serde(rename = "time_elapsed")]
    TimeElapsed { min_duration_ms: u64 },
    #[serde(rename = "orisha_approval")]
    OrishaApproval { required: Vec<OrishaId> },
    #[serde(rename = "anomaly_resolved")]
    AnomalyResolved { verification_trace: Uuid },
    #[serde(rename = "manual_override")]
    ManualOverride { authorized_signers: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PenaltyType {
    #[serde(rename = "capability_revocation")]
    CapabilityRevocation { ops: Vec<String> },
    #[serde(rename = "reputation_slash")]
    ReputationSlash { delta: String }, // changed from f64 to String for Ord
    #[serde(rename = "economic_bond")]
    EconomicBond { amount: i64, unlock_condition: String },
    #[serde(rename = "execution_throttle")]
    ExecutionThrottle { max_ops_per_minute: u64 },
    #[serde(rename = "temporary_suspension")]
    TemporarySuspension { resume_after_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum HaltScope {
    #[serde(rename = "agent")]
    Agent(String),
    #[serde(rename = "branch")]
    Branch(String),
    #[serde(rename = "shard")]
    Shard(String),
    #[serde(rename = "global")]
    Global,
}

/// 🪜 Action Ladder: maps severity + classification → deterministic action
pub struct ActionLadder {
    pub policy: EnforcementPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementPolicy {
    pub default_severity_threshold: AnomalySeverity,
    pub escalation_rules: Vec<EscalationRule>,
    pub orisha_weights: std::collections::HashMap<OrishaId, u8>,
    pub auto_quarantine_enabled: bool,
    pub rollback_authority_threshold: f64,  // weighted quorum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub match_classification: AnomalyClassificationMatcher,
    pub min_severity: AnomalySeverity,
    pub action: EnforcementAction,
    pub requires_consensus: Vec<OrishaId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyClassificationMatcher {
    Any,
    Exact(AnomalyClassification),
    Pattern { r#type: String, field_pattern: Option<String> },
    OrishaSpecific { orisha: OrishaId, violation_type: String },
}

impl ActionLadder {
    pub fn new(policy: EnforcementPolicy) -> Self {
        Self { policy }
    }
    
    /// 🎯 Determine enforcement action for anomaly
    pub fn determine_action(&self, anomaly: &Anomaly) -> EnforcementDecision {
        // 1. Check explicit escalation rules
        for rule in &self.policy.escalation_rules {
            if rule.matches(anomaly) {
                return EnforcementDecision {
                    action: rule.action.clone(),
                    confidence: 1.0,
                    rationale: format!("Rule match: {:?}", rule.match_classification),
                    requires_consensus: rule.requires_consensus.clone(),
                };
            }
        }
        
        // 2. Default severity-based escalation
        let action = match (&anomaly.severity, &anomaly.classification) {
            (AnomalySeverity::Observational, _) => EnforcementAction::Observe {
                log_level: LogLevel::Debug,
                retain_evidence: true,
            },
            (AnomalySeverity::Warning, _) => EnforcementAction::FlagForReview {
                reviewers: vec!["ọbàtálá".into()],
                timeout_ms: 300_000,  // 5 minutes
            },
            (AnomalySeverity::Critical, AnomalyClassification::ConcurrencyConflict { .. }) => {
                EnforcementAction::QuarantineState {
                    scope: QuarantineScope::Branch { branch_id: "unknown".into() },  // resolved later
                    duration_ms: Some(600_000),
                    release_conditions: vec![ReleaseCondition::OrishaApproval {
                        required: vec!["yemọja".into(), "èṣù".into()],
                    }],
                }
            }
            (AnomalySeverity::Critical, AnomalyClassification::EconomicAnomaly { .. }) => {
                EnforcementAction::RollbackTransition {
                    to_state: anomaly.evidence.state_before,
                    preserve_audit_trail: true,
                    compensation_required: true,
                }
            }
            (AnomalySeverity::Catastrophic, _) => EnforcementAction::EmergencyHalt {
                scope: HaltScope::Global,
                require_quorum: vec!["èṣù".into(), "ọbàtálá".into(), "ṣàngó".into()],
                auto_resume_condition: Some("all_validators_reconciled".into()),
            },
            _ => EnforcementAction::FlagForReview {
                reviewers: vec!["èṣù".into()],
                timeout_ms: 60_000,
            },
        };
        
        EnforcementDecision {
            action,
            confidence: anomaly.confidence,
            rationale: "Default severity escalation".into(),
            requires_consensus: vec![],
        }
    }
}

impl EscalationRule {
    pub fn matches(&self, anomaly: &Anomaly) -> bool {
        if anomaly.severity < self.min_severity {
            return false;
        }
        self.match_classification.matches(&anomaly.classification)
    }
}

impl AnomalyClassificationMatcher {
    pub fn matches(&self, classification: &AnomalyClassification) -> bool {
        match (self, classification) {
            (AnomalyClassificationMatcher::Any, _) => true,
            (AnomalyClassificationMatcher::Exact(a), b) => a == b,
            (AnomalyClassificationMatcher::Pattern { r#type, field_pattern }, b) => {
                let b_val = serde_json::to_value(b).unwrap();
                let b_type = b_val["type"].as_str().unwrap();
                if r#type != b_type { return false; }
                if let Some(pattern) = field_pattern {
                    // Simple glob matching on field values
                    let b_str = serde_json::to_string(b).unwrap();
                    return b_str.contains(pattern);
                }
                true
            }
            (AnomalyClassificationMatcher::OrishaSpecific { orisha, violation_type }, b) => {
                // Match if classification involves this Orisha and violation type
                let b_val = serde_json::to_value(b).unwrap();
                b_val.as_object()
                    .map(|obj| obj.get("orisha").and_then(|v| v.as_str()) == Some(orisha) &&
                           obj.get("violation_type").and_then(|v| v.as_str()) == Some(violation_type))
                    .unwrap_or(false)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnforcementDecision {
    pub action: EnforcementAction,
    pub confidence: f64,
    pub rationale: String,
    pub requires_consensus: Vec<OrishaId>,
}
