use serde::{Serialize, Deserialize};
use json_patch::{patch, PatchOperation, AddOperation};
use crate::kernel::css::{CanonicalSystemState, StateError};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffOp {
    // Standard JSON Patch
    Add { path: String, value: serde_json::Value },
    Remove { path: String },
    Replace { path: String, value: serde_json::Value },
    
    // Move-style semantic ops
    Increment { path: String, delta: i64 },
    Decrement { path: String, delta: i64 },
    Transfer { from: String, to: String, amount: u64 },
    
    // Orisha-specific
    Bless { path: String, by: String },   // Ọbàtálá approval
    Curse { path: String, by: String },   // Èṣù rejection marker
    Seal { path: String, algorithm: String }, // Yemọja encryption hint
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub transition_id: Uuid,
    pub input_state_hash: String,
    pub ops: Vec<DiffOp>,
    pub validators_required: Vec<String>, // ["èṣù", "ọbàtálá", "ṣàngó"]
    pub validators_approved: Vec<String>,
    pub execution_plan: Option<ExecutionPlan>,
    pub final_state_hash: Option<String>,
    pub timestamp: u64,
}

impl StateDiff {
    pub fn apply_to(&self, state: &mut CanonicalSystemState) -> Result<(), StateError> {
        if !state.state_hash.is_empty() && state.state_hash != self.input_state_hash {
            return Err(StateError::HashMismatch);
        }

        let mut state_val = serde_json::to_value(&state).map_err(|e| StateError::InvalidDiff(e.to_string()))?;

        for op in &self.ops {
            match op {
                DiffOp::Add { path, value } => {
                    let operations = vec![PatchOperation::Add(AddOperation {
                        path: path.parse().map_err(|e: jsonptr::ParseError| StateError::InvalidDiff(e.to_string()))?,
                        value: value.clone(),
                    })];
                    patch(&mut state_val, &operations).map_err(|e| StateError::InvalidDiff(e.to_string()))?;
                }
                DiffOp::Increment { path, delta } => {
                    if let Some(current) = state_val.pointer_mut(path).and_then(|v| v.as_i64()) {
                        let new_val = current + delta;
                        *state_val.pointer_mut(path).unwrap() = serde_json::Value::from(new_val);
                    }
                }
                DiffOp::Bless { path: _, by: _ } => {
                    // Mark path as ethically validated
                }
                // ... handle other ops
                _ => {}
            }
        }

        *state = serde_json::from_value(state_val).map_err(|e| StateError::InvalidDiff(e.to_string()))?;
        state.version += 1;
        state.state_hash = state.compute_hash();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub tool_calls: Vec<ToolCall>,
    pub timeout_ms: u64,
    pub rollback_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub orisha_layer: String, // "Ògún", "Yemọja", etc.
}
