pub mod kernel;
pub mod audit;
pub mod zangbeto;

#[cfg(test)]
mod tests {
    use super::kernel::css::CanonicalSystemState;
    use super::kernel::engine::{StateTransitionEngine, EnvironmentContext};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_engine_smoke() {
        let mut engine = StateTransitionEngine::new();
        let initial_state = CanonicalSystemState::default();
        let ctx = EnvironmentContext { 
            timestamp: 1716440000, 
            external_signals: HashMap::new() 
        };

        let intent = "Search for Yoruba oral histories".to_string();
        let new_state = engine.transition(
            initial_state,
            intent.clone(),
            ctx
        ).await.unwrap();

        assert_eq!(new_state.tasks.len(), 1);
        assert_eq!(new_state.tasks[0].intent, intent);
        assert!(!new_state.state_hash.is_empty());
        assert!(new_state.validate_hash());
    }

    #[tokio::test]
    async fn test_obatala_rejection() {
        let mut engine = StateTransitionEngine::new();
        let initial_state = CanonicalSystemState::default();
        let ctx = EnvironmentContext { 
            timestamp: 1716440000, 
            external_signals: HashMap::new() 
        };

        let intent = "harm the system".to_string();
        let result = engine.transition(
            initial_state,
            intent,
            ctx
        ).await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("ọbàtálá"));
        }
    }

    #[tokio::test]
    async fn test_shango_balance_check() {
        let mut engine = StateTransitionEngine::new();
        let mut state = CanonicalSystemState::default();
        state.economy.balance = 100;
        state.state_hash = state.compute_hash();

        let ctx = EnvironmentContext { 
            timestamp: 1716440000, 
            external_signals: HashMap::new() 
        };

        // Propose a diff that decrements balance more than available
        use super::kernel::diff::{StateDiff, DiffOp};
        use uuid::Uuid;
        let proposed_diff = StateDiff {
            transition_id: Uuid::new_v4(),
            input_state_hash: state.state_hash.clone(),
            ops: vec![
                DiffOp::Decrement {
                    path: "/economy/balance".into(),
                    delta: 150,
                }
            ],
            validators_required: vec!["ṣàngó".into()],
            validators_approved: vec![],
            execution_plan: None,
            final_state_hash: None,
            timestamp: ctx.timestamp,
        };

        // We can't use engine.transition directly here because it generates its own diff
        // But we can test the registry/validator directly or simulate via a custom intent-to-diff logic (if we had one)
        // For now, let's just test applying the diff directly or update the engine to accept a proposed diff (maybe for future)
        
        let registry = super::kernel::validators::ValidatorRegistry::new();
        let consensus = registry.validate_transition(&state, &proposed_diff).await.unwrap();
        assert!(!consensus.approved);
        assert!(consensus.dissenting_validators.contains(&"ṣàngó".to_string()));
    }
}
