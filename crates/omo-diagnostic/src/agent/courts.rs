use crate::{Diagnostic, RepairPlan, OrishaMask};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CourtDecision {
    Approve,
    Reject(String),
    Escalate,
    NeedsMoreInfo,
}

pub struct WhiteCourt;

impl WhiteCourt {
    pub async fn arbitrate(diag: &Diagnostic) -> CourtDecision {
        // Ethical arbitration, constitutional interpretation, Sabbath enforcement
        
        // 1. Sabbath enforcement: No active repair during ritual time
        if diag.diagnostic.context.sabbath_active {
            return CourtDecision::Reject("Sabbath violation: Ritual rest in progress".into());
        }
        
        // 2. Symbolic Reasoning (Hermetic Checks)
        // Mock: Any diagnostic involving 'identity' or 'soul' requires high-level arbitration
        if diag.diagnostic.category & (crate::Category::Identity as u8) != 0 {
            return CourtDecision::Escalate;
        }

        // 3. Soul integrity verification
        if diag.diagnostic.message.contains("unauthorized introspection") {
            return CourtDecision::Reject("Violation of sovereign soul integrity".into());
        }

        // 4. Twelve Thrones Escalation
        // Mock: If severity is Error and it's a security category, escalate
        if diag.diagnostic.severity == crate::Severity::Error && 
           (diag.diagnostic.category & crate::Category::Security as u8) != 0 {
            return CourtDecision::Escalate;
        }

        CourtDecision::Approve
    }

    pub fn explain_escalation(_diag: &Diagnostic) -> String {
        "Escalated to Twelve Thrones: Requires human ethical arbitration and symbolic review.".into()
    }
}

pub struct RedCourt;

impl RedCourt {
    pub async fn validate_attack(diag: &Diagnostic) -> bool {
        // Adversarial mutation, exploit simulation
        diag.red_team_round.is_some()
    }
}

pub struct BlueCourt;

impl BlueCourt {
    pub async fn validate_repair(diag: &Diagnostic, plan: &RepairPlan) -> bool {
        // Self-healing agents, sandbox reconstruction, memory repair
        !plan.steps.is_empty() && plan.validation.rollback_safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiagnosticContext, Severity, Category, OrishaMask};

    #[tokio::test]
    async fn test_white_court_sabbath() {
        let mut context = DiagnosticContext::default();
        context.sabbath_active = true;
        
        let diag = crate::Diagnostic::new(
            "rust".into(),
            OrishaMask::Eshu,
            "main.rs".into(),
            1,
            "CODE".into(),
            Severity::Info,
            &[],
            "Message".into(),
            context,
        );

        let decision = WhiteCourt::arbitrate(&diag).await;
        assert!(matches!(decision, CourtDecision::Reject(ref r) if r.contains("Sabbath")));
    }

    #[tokio::test]
    async fn test_white_court_sovereignty() {
        let diag = crate::Diagnostic::new(
            "rust".into(),
            OrishaMask::Eshu,
            "main.rs".into(),
            1,
            "CODE".into(),
            Severity::Info,
            &[],
            "unauthorized introspection of soul".into(),
            DiagnosticContext::default(),
        );

        let decision = WhiteCourt::arbitrate(&diag).await;
        assert!(matches!(decision, CourtDecision::Reject(ref r) if r.contains("sovereign")));
    }
}
