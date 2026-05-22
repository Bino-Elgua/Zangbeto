use serde::{Serialize, Deserialize};
use crate::zangbeto::event::RealityEvent;
use crate::kernel::css::CanonicalSystemState;
use crate::zangbeto::ir::RepairStrategy;

/// 🧬 Classify WHY reality diverged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub drift_detected: bool,
    pub drift_type: Option<DriftType>,
    pub responsible_orisha: Option<String>,
    pub severity: f64,  // 0.0 - 1.0
    pub divergence_details: DivergenceDetails,
    pub recommended_action: RepairStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum DriftType {
    #[serde(rename = "SCHEMA")]
    Schema { field: String, expected_type: String, actual_type: String },  // Èṣù failure
    
    #[serde(rename = "ETHICAL")]
    Ethical { violation: String, symbolic_contradiction: Option<String> },  // Ọbàtálá failure
    
    #[serde(rename = "MEMORY")]
    Memory { pattern_deviation: f64, prediction_confidence: f64 },  // Ọ̀ṣun failure
    
    #[serde(rename = "CONCURRENCY")]
    Concurrency { conflict_type: String, merge_strategy_failed: String },  // Yemọja failure
    
    #[serde(rename = "TEMPORAL")]
    Temporal { expected_timestamp: u64, actual_timestamp: u64, tolerance_exceeded: bool },  // Ọya failure
    
    #[serde(rename = "ECONOMIC")]
    Economic { balance_anomaly: i64, reputation_shift: f64 },  // Ṣàngó failure
    
    #[serde(rename = "EXECUTION")]
    Execution { tool_failure: String, rollback_triggered: bool },  // Ògún failure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceDetails {
    pub expected_hash: String,
    pub actual_hash: String,
    pub field_paths_diverged: Vec<String>,
    pub semantic_diff: serde_json::Value,  // human-readable explanation
}

pub struct DriftClassifier;

impl DriftClassifier {
    /// 🧠 Analyze event for drift: CSSₙ₊₁_actual vs CSSₙ₊₁_expected
    pub fn classify(event: &RealityEvent) -> DriftReport {
        let Some(expected) = &event.expected_state else {
            return DriftReport {
                drift_detected: false,
                drift_type: None,
                responsible_orisha: None,
                severity: 0.0,
                divergence_details: DivergenceDetails {
                    expected_hash: event.pre_state_hash.clone(),
                    actual_hash: event.post_state_hash.clone(),
                    field_paths_diverged: vec![],
                    semantic_diff: serde_json::json!({"note": "No expected state provided"}),
                },
                recommended_action: RepairStrategy::Patch,
            };
        };
        
        // Compute field-level divergence
        let divergence = Self::compute_divergence(
            &expected.predicted_css,
            &Self::reconstruct_state_from_event(event),
        );
        
        if divergence.field_paths_diverged.is_empty() {
            return DriftReport {
                drift_detected: false,
                drift_type: None,
                responsible_orisha: None,
                severity: 0.0,
                divergence_details: divergence,
                recommended_action: RepairStrategy::Patch,
            };
        }
        
        // Classify drift type by field path patterns
        let (drift_type, orisha) = Self::map_drift_to_orisha(&divergence.field_paths_diverged, event);
        
        // Compute severity (weighted by field importance + validator dissent)
        let severity = Self::compute_severity(&divergence, event, expected.confidence);
        
        // Recommend repair strategy
        let action = Self::recommend_repair(&drift_type, severity);
        
        DriftReport {
            drift_detected: true,
            drift_type: Some(drift_type),
            responsible_orisha: Some(orisha),
            severity,
            divergence_details: divergence,
            recommended_action: action,
        }
    }
    
    fn map_drift_to_orisha(field_paths: &[String], event: &RealityEvent) -> (DriftType, String) {
        for path in field_paths {
            if path.contains("identity") || path.contains("constraints") {
                return (DriftType::Schema {
                    field: path.clone(),
                    expected_type: "object".into(),
                    actual_type: "unknown".into(),
                }, "èṣù".into());
            }
            if path.contains("tasks") && event.validators.iter().any(|v| 
                v.orisha == "ọbàtálá" && !v.passed
            ) {
                return (DriftType::Ethical {
                    violation: "Task intent contradicts ethical constraints".into(),
                    symbolic_contradiction: None,
                }, "ọbàtálá".into());
            }
            if path.contains("memory") {
                return (DriftType::Memory {
                    pattern_deviation: 0.85,  // Placeholder: compute from Ọ̀ṣun model
                    prediction_confidence: event.expected_state.as_ref().map(|e| e.confidence).unwrap_or(0.0),
                }, "ọ̀ṣun".into());
            }
            if path.contains("economy") {
                return (DriftType::Economic {
                    balance_anomaly: 0,  // Placeholder: compute delta
                    reputation_shift: 0.0,
                }, "ṣàngó".into());
            }
            if path.contains("timestamp") || path.contains("version") {
                return (DriftType::Temporal {
                    expected_timestamp: 0,  // Placeholder
                    actual_timestamp: event.timestamp,
                    tolerance_exceeded: true,
                }, "ọya".into());
            }
        }
        
        // Default: execution failure
        (DriftType::Execution {
            tool_failure: "Unknown tool error".into(),
            rollback_triggered: false,
        }, "ògún".into())
    }
    
    fn compute_divergence(
        expected: &CanonicalSystemState,
        actual: &CanonicalSystemState,
    ) -> DivergenceDetails {
        // Simplified: in production, use deep diff library with semantic awareness
        let mut paths = vec![];
        
        if expected.identity.wallet != actual.identity.wallet {
            paths.push("/identity/wallet".into());
        }
        if expected.economy.balance != actual.economy.balance {
            paths.push("/economy/balance".into());
        }
        // ... compare all fields
        
        DivergenceDetails {
            expected_hash: expected.state_hash.clone(),
            actual_hash: actual.state_hash.clone(),
            field_paths_diverged: paths.clone(),
            semantic_diff: serde_json::json!({
                "note": "Field-level divergence detected",
                "diverged_fields": paths
            }),
        }
    }
    
    fn compute_severity(
        divergence: &DivergenceDetails,
        event: &RealityEvent,
        prediction_confidence: f64,
    ) -> f64 {
        // Weight by: number of diverged fields + validator dissent + confidence gap
        let field_weight = (divergence.field_paths_diverged.len() as f64 * 0.1).min(0.5);
        let validator_weight = event.validators.iter()
            .filter(|v| !v.passed)
            .map(|v| v.authority_weight as f64 * 0.05)
            .sum::<f64>().min(0.3);
        let confidence_gap = (1.0 - prediction_confidence) * 0.2;
        
        (field_weight + validator_weight + confidence_gap).min(1.0)
    }
    
    fn recommend_repair(drift_type: &DriftType, severity: f64) -> RepairStrategy {
        match (drift_type, severity) {
            (DriftType::Schema { .. }, s) if s > 0.7 => RepairStrategy::Rollback,
            (DriftType::Ethical { .. }, _) => RepairStrategy::Quarantine,
            (DriftType::Economic { .. }, s) if s > 0.5 => RepairStrategy::Compensate,
            _ => RepairStrategy::Patch,
        }
    }
    
    fn reconstruct_state_from_event(_event: &RealityEvent) -> CanonicalSystemState {
        // In production: replay diff on pre_state with constraint-aware application
        // For now: stub (we don't have snapshot storage yet)
        // We'll return a default state for now to avoid unimplemented!
        CanonicalSystemState::default()
    }
}
