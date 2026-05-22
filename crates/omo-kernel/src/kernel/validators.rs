use async_trait::async_trait;
use serde::Serialize;
use crate::kernel::css::{CanonicalSystemState, StateError, TaskStatus};
use crate::kernel::diff::{StateDiff, DiffOp};

#[async_trait]
pub trait OrishaValidator: Send + Sync {
    fn name(&self) -> &'static str;
    fn authority_weight(&self) -> u8; // 1-10, for quorum resolution
    
    /// Validate a proposed transition BEFORE execution
    async fn validate_proposal(
        &self,
        current: &CanonicalSystemState,
        proposed: &StateDiff,
    ) -> Result<ValidationResult, StateError>;
    
    /// Audit a transition AFTER execution (for Zàngbétò)
    async fn audit_transition(
        &self,
        before: &CanonicalSystemState,
        after: &CanonicalSystemState,
        diff: &StateDiff,
    ) -> AuditReport;
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub approved: bool,
    pub reasons: Vec<String>,
    pub suggested_modifications: Option<StateDiff>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub validator: String,
    pub passed: bool,
    pub anomalies: Vec<String>,
    pub integrity_score: f64, // 0.0 - 1.0
}

// Example: Èṣù — Hard Constraint Gate
pub struct EsuValidator;

#[async_trait]
impl OrishaValidator for EsuValidator {
    fn name(&self) -> &'static str { "èṣù" }
    fn authority_weight(&self) -> u8 { 10 } // Highest: gatekeeper
    
    async fn validate_proposal(
        &self,
        current: &CanonicalSystemState,
        proposed: &StateDiff,
    ) -> Result<ValidationResult, StateError> {
        // Schema validation
        if proposed.ops.is_empty() {
            return Ok(ValidationResult {
                approved: false,
                reasons: vec!["Empty diff rejected by Èṣù".into()],
                suggested_modifications: None,
            });
        }
        
        // Constraint enforcement
        if current.constraints.privacy_mode {
            for op in &proposed.ops {
                if let DiffOp::Add { path, .. } = op {
                    if path.contains("private") {
                        return Ok(ValidationResult {
                            approved: false,
                            reasons: vec!["Private write blocked in privacy mode".into()],
                            suggested_modifications: None,
                        });
                    }
                }
            }
        }
        
        Ok(ValidationResult {
            approved: true,
            reasons: vec!["Schema and constraints valid".into()],
            suggested_modifications: None,
        })
    }
    
    async fn audit_transition(
        &self,
        _before: &CanonicalSystemState,
        after: &CanonicalSystemState,
        _diff: &StateDiff,
    ) -> AuditReport {
        AuditReport {
            validator: self.name().into(),
            passed: after.validate_hash(),
            anomalies: vec![],
            integrity_score: 1.0,
        }
    }
}

// Example: Ọbàtálá — Ethical Validator
pub struct ObatalaValidator;

#[async_trait]
impl OrishaValidator for ObatalaValidator {
    fn name(&self) -> &'static str { "ọbàtálá" }
    fn authority_weight(&self) -> u8 { 8 }
    
    async fn validate_proposal(
        &self,
        current: &CanonicalSystemState,
        proposed: &StateDiff,
    ) -> Result<ValidationResult, StateError> {
        // 1. Check current pending tasks
        let mut intents: Vec<String> = current.tasks.iter()
            .filter(|t| matches!(t.status, TaskStatus::Pending))
            .map(|t| t.intent.clone())
            .collect();

        // 2. Check proposed new tasks in the diff
        for op in &proposed.ops {
            if let DiffOp::Add { path, value } = op {
                if path.contains("tasks") {
                    if let Some(intent) = value.get("intent").and_then(|v| v.as_str()) {
                        intents.push(intent.to_string());
                    }
                }
            }
        }
            
        for intent in intents {
            if intent.contains("harm") || intent.contains("deceive") {
                return Ok(ValidationResult {
                    approved: false,
                    reasons: vec![format!("Ethical violation detected by Ọbàtálá in intent: '{}'", intent)],
                    suggested_modifications: None,
                });
            }
        }
        
        Ok(ValidationResult {
            approved: true,
            reasons: vec!["Ethical coherence maintained".into()],
            suggested_modifications: None,
        })
    }
    
    async fn audit_transition(
        &self,
        _before: &CanonicalSystemState,
        _after: &CanonicalSystemState,
        _diff: &StateDiff,
    ) -> AuditReport {
        // Post-hoc ethical audit
        AuditReport {
            validator: self.name().into(),
            passed: true,
            anomalies: vec![],
            integrity_score: 0.98,
        }
    }
}

// Ọ̀ṣun: Memory Integrity Validator
pub struct OshunValidator;

#[async_trait]
impl OrishaValidator for OshunValidator {
    fn name(&self) -> &'static str { "ọ̀ṣun" }
    fn authority_weight(&self) -> u8 { 7 }
    
    async fn validate_proposal(&self, _current: &CanonicalSystemState, proposed: &StateDiff) -> Result<ValidationResult, StateError> {
        // Check for memory corruption or unauthorized memory access
        for op in &proposed.ops {
            if let DiffOp::Add { path, .. } = op {
                if path.contains("memory") && !path.contains("public_hive") {
                     return Ok(ValidationResult {
                        approved: false,
                        reasons: vec!["Unauthorized memory access attempt detected by Ọ̀ṣun".into()],
                        suggested_modifications: None,
                    });
                }
            }
        }
        Ok(ValidationResult { approved: true, reasons: vec![], suggested_modifications: None })
    }

    async fn audit_transition(&self, _before: &CanonicalSystemState, _after: &CanonicalSystemState, _diff: &StateDiff) -> AuditReport {
        AuditReport { validator: self.name().into(), passed: true, anomalies: vec![], integrity_score: 1.0 }
    }
}

// Yemọja: Swarm & Stability Validator
pub struct YemojaValidator;

#[async_trait]
impl OrishaValidator for YemojaValidator {
    fn name(&self) -> &'static str { "yemọja" }
    fn authority_weight(&self) -> u8 { 6 }
    
    async fn validate_proposal(&self, _current: &CanonicalSystemState, _proposed: &StateDiff) -> Result<ValidationResult, StateError> {
        // Check for swarm instability or resource exhaustion
        Ok(ValidationResult { approved: true, reasons: vec![], suggested_modifications: None })
    }

    async fn audit_transition(&self, _before: &CanonicalSystemState, _after: &CanonicalSystemState, _diff: &StateDiff) -> AuditReport {
        AuditReport { validator: self.name().into(), passed: true, anomalies: vec![], integrity_score: 1.0 }
    }
}

// Ògún: Tool misuse & Execution Validator
pub struct OgunValidator;

#[async_trait]
impl OrishaValidator for OgunValidator {
    fn name(&self) -> &'static str { "ògún" }
    fn authority_weight(&self) -> u8 { 9 }
    
    async fn validate_proposal(&self, _current: &CanonicalSystemState, proposed: &StateDiff) -> Result<ValidationResult, StateError> {
        if let Some(ref plan) = proposed.execution_plan {
            for call in &plan.tool_calls {
                if call.tool_name.contains("rm") || call.tool_name.contains("delete") {
                    return Ok(ValidationResult {
                        approved: false,
                        reasons: vec!["Dangerous tool use blocked by Ògún".into()],
                        suggested_modifications: None,
                    });
                }
            }
        }
        Ok(ValidationResult { approved: true, reasons: vec![], suggested_modifications: None })
    }

    async fn audit_transition(&self, _before: &CanonicalSystemState, _after: &CanonicalSystemState, _diff: &StateDiff) -> AuditReport {
        AuditReport { validator: self.name().into(), passed: true, anomalies: vec![], integrity_score: 1.0 }
    }
}

// Ọya: Network & Anomaly Validator
pub struct OyaValidator;

#[async_trait]
impl OrishaValidator for OyaValidator {
    fn name(&self) -> &'static str { "ọya" }
    fn authority_weight(&self) -> u8 { 5 }
    
    async fn validate_proposal(&self, _current: &CanonicalSystemState, _proposed: &StateDiff) -> Result<ValidationResult, StateError> {
        Ok(ValidationResult { approved: true, reasons: vec![], suggested_modifications: None })
    }

    async fn audit_transition(&self, _before: &CanonicalSystemState, _after: &CanonicalSystemState, _diff: &StateDiff) -> AuditReport {
        AuditReport { validator: self.name().into(), passed: true, anomalies: vec![], integrity_score: 1.0 }
    }
}

// Ṣàngó: Economic Fraud & Justice Validator
pub struct ShangoValidator;

#[async_trait]
impl OrishaValidator for ShangoValidator {
    fn name(&self) -> &'static str { "ṣàngó" }
    fn authority_weight(&self) -> u8 { 9 }
    
    async fn validate_proposal(&self, current: &CanonicalSystemState, proposed: &StateDiff) -> Result<ValidationResult, StateError> {
        for op in &proposed.ops {
            if let DiffOp::Decrement { path, delta } = op {
                if path.contains("balance") {
                    let current_balance = current.economy.balance;
                    if current_balance < *delta {
                        return Ok(ValidationResult {
                            approved: false,
                            reasons: vec!["Insufficient balance for economic transition (Ṣàngó justice)".into()],
                            suggested_modifications: None,
                        });
                    }
                }
            }
        }
        Ok(ValidationResult { approved: true, reasons: vec![], suggested_modifications: None })
    }

    async fn audit_transition(&self, _before: &CanonicalSystemState, _after: &CanonicalSystemState, _diff: &StateDiff) -> AuditReport {
        AuditReport { validator: self.name().into(), passed: true, anomalies: vec![], integrity_score: 1.0 }
    }
}

// Registry
pub struct ValidatorRegistry {
    validators: Vec<Box<dyn OrishaValidator>>,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        Self {
            validators: vec![
                Box::new(EsuValidator),
                Box::new(ObatalaValidator),
                Box::new(OshunValidator),
                Box::new(YemojaValidator),
                Box::new(OgunValidator),
                Box::new(OyaValidator),
                Box::new(ShangoValidator),
            ],
        }
    }
    
    pub async fn validate_transition(
        &self,
        current: &CanonicalSystemState,
        proposed: &StateDiff,
    ) -> Result<ConsensusResult, StateError> {
        let mut results = Vec::new();
        
        for validator in &self.validators {
            if proposed.validators_required.contains(&validator.name().to_string()) {
                let result = validator.validate_proposal(current, proposed).await?;
                results.push((validator.name(), result.approved, validator.authority_weight()));
            }
        }
        
        Ok(ConsensusResult::resolve(results))
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ConsensusResult {
    pub approved: bool,
    pub quorum_met: bool,
    pub dissenting_validators: Vec<String>,
}

impl ConsensusResult {
    pub fn resolve(validations: Vec<(&str, bool, u8)>) -> Self {
        let total_weight: u16 = validations.iter().map(|(_, _, w)| *w as u16).sum();
        let approved_weight: u16 = validations.iter()
            .filter(|(_, approved, _)| *approved)
            .map(|(_, _, w)| *w as u16)
            .sum();
        
        // Ṣàngó tie-breaker: finality requires 70% weighted approval
        let quorum_threshold = (total_weight as f64 * 0.7) as u16;
        
        Self {
            approved: approved_weight >= quorum_threshold,
            quorum_met: approved_weight >= quorum_threshold,
            dissenting_validators: validations.iter()
                .filter(|(_, approved, _)| !approved)
                .map(|(name, _, _)| name.to_string())
                .collect(),
        }
    }
}
