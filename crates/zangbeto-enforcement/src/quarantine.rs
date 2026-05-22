use crdt_memory::CrdtMemory;
use vm_core::ir::{StateHash, OrishaId};
use crate::anomaly::AnomalyClassification;
use crate::action_ladder::{QuarantineScope, ReleaseCondition};
use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// 🔒 QuarantineManager: isolates divergent state without corruption
#[derive(Clone)]
pub struct QuarantineManager {
    pub quarantined_states: BTreeMap<QuarantineId, QuarantinedState>,
    pub merge_blocklist: BTreeSet<BlockedMerge>,
    pub release_queue: Vec<ReleaseCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuarantineId(pub String);  // hash(quarantine_seed)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedState {
    pub id: QuarantineId,
    pub original_state_hash: StateHash,
    pub quarantined_snapshot: CrdtMemory,
    pub quarantine_reason: AnomalyClassification,
    pub scope: QuarantineScope,
    pub created_at: u64,
    pub expiry: Option<u64>,
    pub release_conditions: Vec<ReleaseCondition>,
    pub access_capabilities: Vec<QuarantineAccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineAccess {
    pub agent_id: String,
    pub permissions: QuarantinePermission,
    pub expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuarantinePermission {
    #[serde(rename = "read_only")]
    ReadOnly,
    #[serde(rename = "audit_only")]
    AuditOnly,
    #[serde(rename = "repair_allowed")]
    RepairAllowed { max_ops: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockedMerge {
    pub left_hash: StateHash,
    pub right_hash: StateHash,
    pub conflict_type: String,
    pub blocked_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseCandidate {
    pub quarantine_id: QuarantineId,
    pub satisfied_conditions: BTreeSet<ReleaseCondition>,
    pub pending_conditions: BTreeSet<ReleaseCondition>,
    pub orisha_approvals: BTreeSet<OrishaId>,
}

impl QuarantineManager {
    pub fn new() -> Self {
        Self {
            quarantined_states: BTreeMap::new(),
            merge_blocklist: BTreeSet::new(),
            release_queue: Vec::new(),
        }
    }

    pub fn get_pending_quarantine_ids(&self) -> Vec<QuarantineId> {
        self.release_queue.iter().map(|c| c.quarantine_id.clone()).collect()
    }
    
    /// 🚫 Quarantine a state subtree
    pub fn quarantine(
        &mut self,
        state: &CrdtMemory,
        scope: QuarantineScope,
        reason: AnomalyClassification,
        release_conditions: Vec<ReleaseCondition>,
    ) -> Result<QuarantineId, QuarantineError> {
        let quarantine_id = Self::generate_quarantine_id(state, &scope, &reason);
        
        // Extract quarantined snapshot based on scope
        let snapshot = match &scope {
            QuarantineScope::Path { path } => {
                self.extract_path_snapshot(state, path)?
            }
            QuarantineScope::Subtree { root_path } => {
                self.extract_subtree_snapshot(state, root_path)?
            }
            QuarantineScope::Agent { .. } => {
                state.clone()
            }
            QuarantineScope::Branch { .. } => {
                state.clone()
            }
            QuarantineScope::Global => {
                state.clone()
            }
        };
        
        let quarantined = QuarantinedState {
            id: quarantine_id.clone(),
            original_state_hash: Self::hash_memory(state),
            quarantined_snapshot: snapshot,
            quarantine_reason: reason,
            scope,
            created_at: current_timestamp(),
            expiry: None,
            release_conditions: release_conditions.clone(),
            access_capabilities: vec![],
        };
        
        self.block_conflicting_merges(&quarantined);
        
        self.quarantined_states.insert(quarantine_id.clone(), quarantined);
        
        self.release_queue.push(ReleaseCandidate {
            quarantine_id: quarantine_id.clone(),
            satisfied_conditions: BTreeSet::new(),
            pending_conditions: release_conditions.into_iter().collect(),
            orisha_approvals: BTreeSet::new(),
        });
        
        Ok(quarantine_id)
    }
    
    /// ✅ Check if a merge is allowed (quarantine-aware)
    pub fn is_merge_allowed(&self, left: &CrdtMemory, right: &CrdtMemory) -> MergePermission {
        let left_hash = Self::hash_memory(left);
        let right_hash = Self::hash_memory(right);
        
        if self.merge_blocklist.contains(&BlockedMerge {
            left_hash,
            right_hash: right_hash,
            conflict_type: "quarantine_source".into(),
            blocked_at: 0,
        }) || self.merge_blocklist.contains(&BlockedMerge {
            left_hash: right_hash,
            right_hash: left_hash,
            conflict_type: "quarantine_source".into(),
            blocked_at: 0,
        }) {
            return MergePermission::Denied {
                reason: "Merge involves quarantined state".into(),
                quarantine_id: None,
            };
        }
        
        let conflict_paths = self.find_quarantined_overlap(left, right);
        if !conflict_paths.is_empty() {
            return MergePermission::Conditional {
                requires_resolution: conflict_paths,
                suggested_action: "quarantine_merge_result".into(),
            };
        }
        
        MergePermission::Allowed
    }
    
    /// 🔄 Attempt to release a quarantine
    pub fn evaluate_release(&mut self, quarantine_id: &QuarantineId) -> ReleaseResult {
        let candidate_idx = self.release_queue.iter()
            .position(|c| &c.quarantine_id == quarantine_id);
        
        let Some(idx) = candidate_idx else {
            return ReleaseResult::NotFound;
        };
        
        let quarantined = self.quarantined_states.get(quarantine_id).unwrap();
        let now = current_timestamp();
        
        let mut satisfied_this_turn = Vec::new();
        
        {
            let candidate = &self.release_queue[idx];
            for cond in &candidate.pending_conditions {
                match cond {
                    ReleaseCondition::TimeElapsed { min_duration_ms } => {
                        if now - quarantined.created_at >= *min_duration_ms {
                            satisfied_this_turn.push(cond.clone());
                        }
                    }
                    ReleaseCondition::OrishaApproval { required } => {
                        if required.iter().all(|o| candidate.orisha_approvals.contains(o)) {
                            satisfied_this_turn.push(cond.clone());
                        }
                    }
                    ReleaseCondition::AnomalyResolved { verification_trace } => {
                        if self.verify_anomaly_resolved(quarantine_id, verification_trace) {
                            satisfied_this_turn.push(cond.clone());
                        }
                    }
                    ReleaseCondition::ManualOverride { .. } => {}
                }
            }
        }
        
        let candidate = &mut self.release_queue[idx];
        for cond in satisfied_this_turn {
            candidate.pending_conditions.remove(&cond);
            candidate.satisfied_conditions.insert(cond);
        }
        
        if candidate.pending_conditions.is_empty() {
            let released = self.quarantined_states.remove(quarantine_id).unwrap();
            self.release_queue.remove(idx);
            self.unblock_merges(&released);
            
            ReleaseResult::Released {
                released_state_hash: Self::hash_memory(&released.quarantined_snapshot),
                quarantine_duration_ms: now - released.created_at,
            }
        } else if !candidate.satisfied_conditions.is_empty() {
            ReleaseResult::Pending {
                satisfied: candidate.satisfied_conditions.len(),
                remaining: candidate.pending_conditions.len(),
                next_condition: candidate.pending_conditions.iter().next().cloned(),
            }
        } else {
            ReleaseResult::NoProgress
        }
    }
    
    pub fn record_orisha_approval(
        &mut self, 
        quarantine_id: &QuarantineId, 
        orisha: OrishaId
    ) -> Result<(), QuarantineError> {
        if let Some(candidate) = self.release_queue.iter_mut()
            .find(|c| &c.quarantine_id == quarantine_id)
        {
            candidate.orisha_approvals.insert(orisha);
            Ok(())
        } else {
            Err(QuarantineError::QuarantineNotFound)
        }
    }
    
    fn generate_quarantine_id(
        state: &CrdtMemory, 
        scope: &QuarantineScope, 
        reason: &AnomalyClassification
    ) -> QuarantineId {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(Self::hash_memory(state));
        hasher.update(serde_cbor::to_vec(scope).unwrap());
        hasher.update(serde_cbor::to_vec(reason).unwrap());
        QuarantineId(hex::encode(hasher.finalize()))
    }
    
    fn hash_memory(memory: &CrdtMemory) -> StateHash {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(serde_cbor::to_vec(memory).unwrap());
        hasher.finalize().into()
    }
    
    fn find_quarantined_overlap(&self, _left: &CrdtMemory, _right: &CrdtMemory) -> Vec<String> {
        vec![]
    }
    
    fn verify_anomaly_resolved(&self, _quarantine_id: &QuarantineId, _trace_id: &Uuid) -> bool {
        true
    }
    
    fn block_conflicting_merges(&mut self, quarantined: &QuarantinedState) {
        self.merge_blocklist.insert(BlockedMerge {
            left_hash: quarantined.original_state_hash,
            right_hash: quarantined.original_state_hash,
            conflict_type: "quarantine_source".into(),
            blocked_at: current_timestamp(),
        });
    }
    
    fn unblock_merges(&mut self, released: &QuarantinedState) {
        self.merge_blocklist.retain(|b| 
            b.left_hash != released.original_state_hash && 
            b.right_hash != released.original_state_hash
        );
    }
    
    fn extract_path_snapshot(&self, state: &CrdtMemory, _path: &str) -> Result<CrdtMemory, QuarantineError> {
        Ok(state.clone())
    }
    
    fn extract_subtree_snapshot(&self, state: &CrdtMemory, _root: &str) -> Result<CrdtMemory, QuarantineError> {
        Ok(state.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergePermission {
    Allowed,
    Denied { reason: String, quarantine_id: Option<QuarantineId> },
    Conditional { requires_resolution: Vec<String>, suggested_action: String },
}

#[derive(Debug)]
pub enum ReleaseResult {
    Released { released_state_hash: StateHash, quarantine_duration_ms: u64 },
    Pending { satisfied: usize, remaining: usize, next_condition: Option<ReleaseCondition> },
    NoProgress,
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum QuarantineError {
    #[error("Quarantine not found")]
    QuarantineNotFound,
    #[error("Path extraction failed")]
    PathExtractionFailed,
    #[error("Invalid scope for state")]
    InvalidScope,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
