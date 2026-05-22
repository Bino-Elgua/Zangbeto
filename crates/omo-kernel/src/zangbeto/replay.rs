use crate::zangbeto::event::{RealityEvent, ExpectedState};
use crate::kernel::css::CanonicalSystemState;
use std::collections::HashMap;

/// 🧪 Replay reality from CSS₀ → CSSₙ to detect first divergence
pub struct RealityReplayEngine {
    event_log: Vec<RealityEvent>,
    state_snapshots: HashMap<String, CanonicalSystemState>, // hash → state
}

impl RealityReplayEngine {
    pub fn new() -> Self {
        Self {
            event_log: vec![],
            state_snapshots: HashMap::new(),
        }
    }
    
    /// Add event to replay log
    pub fn record_event(&mut self, event: RealityEvent) {
        // Store pre-state snapshot if not present
        if !self.state_snapshots.contains_key(&event.pre_state_hash) {
            // In production: fetch from persistent store
        }
        self.event_log.push(event);
    }
    
    /// 🎯 Find first point of divergence from expected trajectory
    pub fn find_first_divergence(
        &self,
        initial_state: CanonicalSystemState,
        expected_trajectory: &[ExpectedState],
    ) -> Option<DivergencePoint> {
        let mut current_state = initial_state;
        
        for (idx, (event, expected)) in self.event_log.iter()
            .zip(expected_trajectory.iter())
            .enumerate()
        {
            // Replay diff deterministically
            let mut replayed = current_state.clone();
            if replayed.apply_diff(&event.diff).is_err() {
                return Some(DivergencePoint {
                    event_index: idx,
                    event_id: event.event_id,
                    failure_type: "diff_application_failed".into(),
                    state_before: current_state.state_hash.clone(),
                    state_after_replay: "ERROR".into(),
                    expected_state_hash: expected.predicted_css.state_hash.clone(),
                });
            }
            
            // Compare replayed vs expected
            if replayed.state_hash != expected.predicted_css.state_hash {
                return Some(DivergencePoint {
                    event_index: idx,
                    event_id: event.event_id,
                    failure_type: "state_mismatch".into(),
                    state_before: current_state.state_hash.clone(),
                    state_after_replay: replayed.state_hash.clone(),
                    expected_state_hash: expected.predicted_css.state_hash.clone(),
                });
            }
            
            current_state = replayed;
        }
        
        None // No divergence found
    }
    
    /// 🔁 Rewind to any historical state by hash
    pub fn rewind_to(&self, target_hash: &str) -> Option<CanonicalSystemState> {
        self.state_snapshots.get(target_hash).cloned()
            .or_else(|| {
                // Reconstruct by replaying from genesis
                self.reconstruct_state(target_hash)
            })
    }
    
    fn reconstruct_state(&self, _target_hash: &str) -> Option<CanonicalSystemState> {
        // Binary search + replay for efficient reconstruction
        // Implementation omitted for brevity
        None
    }
}

#[derive(Debug, Clone)]
pub struct DivergencePoint {
    pub event_index: usize,
    pub event_id: uuid::Uuid,
    pub failure_type: String,
    pub state_before: String,
    pub state_after_replay: String,
    pub expected_state_hash: String,
}
