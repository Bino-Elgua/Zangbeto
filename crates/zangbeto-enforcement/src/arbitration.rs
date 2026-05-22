use policy_runtime::{OrishaPolicy, PolicyResult, ValidationContext, CanonicalState};
use vm_core::ir::{CanonicalStateIR, OrishaId};
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// ⚖️ ArbitrationEngine: resolves multi-Orisha policy conflicts
pub struct ArbitrationEngine {
    pub policies: BTreeMap<OrishaId, Box<dyn OrishaPolicy>>,
    pub arbitration_graph: ArbitrationGraph,
    pub consensus_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrationGraph {
    /// Directed graph: orisha A → orisha B means A defers to B on conflict
    pub defer_edges: BTreeMap<OrishaId, Vec<OrishaId>>,
    /// Conflict resolution strategies per orisha pair
    pub resolution_strategies: BTreeMap<(OrishaId, OrishaId), ResolutionStrategy>,
    /// Fallback: when no strategy matches
    pub default_strategy: ResolutionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    #[serde(rename = "weighted_vote")]
    WeightedVote { weights: BTreeMap<OrishaId, u8> },
    #[serde(rename = "lexical_order")]
    LexicalOrder,  // Deterministic: alphabetically first wins
    #[serde(rename = "hierarchical")]
    Hierarchical { superior: OrishaId },
    #[serde(rename = "consensus_required")]
    ConsensusRequired { quorum: Vec<OrishaId> },
    #[serde(rename = "random_deterministic")]
    RandomDeterministic { seed_source: String },  // Hash-based pseudo-random
}

#[derive(Debug, Clone)]
pub struct ArbitrationRequest {
    pub request_id: Uuid,
    pub state: CanonicalState,
    pub ops: Vec<CanonicalStateIR>,
    pub context: ValidationContext,
    pub conflicting_policies: Vec<PolicyConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConflict {
    pub orisha_a: OrishaId,
    pub orisha_b: OrishaId,
    pub conflict_type: ConflictType,
    pub a_result: PolicyResult,
    pub b_result: PolicyResult,
    pub evidence: ConflictEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    #[serde(rename = "approve_vs_reject")]
    ApproveVsReject,
    #[serde(rename = "conditional_mismatch")]
    ConditionalMismatch { a_requires: OrishaId, b_requires: OrishaId },
    #[serde(rename = "severity_disagreement")]
    SeverityDisagreement { a_severity: u8, b_severity: u8 },
    #[serde(rename = "resource_conflict")]
    ResourceConflict { resource: String, a_claim: i64, b_claim: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEvidence {
    pub state_hash: [u8; 32],
    pub op_hashes: Vec<[u8; 32]>,
    pub policy_rule_ids: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum ArbitrationDecision {
    Approve { winning_orisha: OrishaId, rationale: String },
    Reject { winning_orisha: OrishaId, rationale: String },
    Conditional { blessing_required: OrishaId, rationale: String },
    DeferToHuman { reason: String, escalation_path: Vec<String> },
}

impl ArbitrationEngine {
    pub fn new(consensus_threshold: f64, graph: ArbitrationGraph) -> Self {
        Self {
            policies: BTreeMap::new(),
            arbitration_graph: graph,
            consensus_threshold,
        }
    }
    
    pub fn register_policy(&mut self, orisha: OrishaId, policy: Box<dyn OrishaPolicy>) {
        self.policies.insert(orisha, policy);
    }
    
    /// ⚡ Resolve conflict between Orisha policies
    pub async fn arbitrate(&self, request: ArbitrationRequest) -> Result<ArbitrationDecision, ArbitrationError> {
        if request.conflicting_policies.is_empty() {
            return Err(ArbitrationError::NoConflictToResolve);
        }
        
        let mut decisions = Vec::new();
        
        for conflict in &request.conflicting_policies {
            let strategy = self.arbitration_graph.resolution_strategies
                .get(&(conflict.orisha_a.clone(), conflict.orisha_b.clone()))
                .or_else(|| self.arbitration_graph.resolution_strategies
                    .get(&(conflict.orisha_b.clone(), conflict.orisha_a.clone())))
                .unwrap_or(&self.arbitration_graph.default_strategy);
            
            let decision = match strategy {
                ResolutionStrategy::WeightedVote { weights } => {
                    self.resolve_weighted_vote(conflict, weights)
                }
                ResolutionStrategy::LexicalOrder => {
                    self.resolve_lexical(conflict)
                }
                ResolutionStrategy::Hierarchical { superior } => {
                    self.resolve_hierarchical(conflict, superior)
                }
                ResolutionStrategy::ConsensusRequired { quorum } => {
                    self.resolve_consensus(conflict, quorum, &request).await?
                }
                ResolutionStrategy::RandomDeterministic { seed_source } => {
                    self.resolve_deterministic_random(conflict, seed_source, &request)
                }
            };
            
            decisions.push(decision);
        }
        
        self.aggregate_decisions(decisions)
    }
    
    fn resolve_weighted_vote(&self, conflict: &PolicyConflict, weights: &BTreeMap<OrishaId, u8>) -> ArbitrationDecision {
        let weight_a = weights.get(&conflict.orisha_a).copied().unwrap_or(5);
        let weight_b = weights.get(&conflict.orisha_b).copied().unwrap_or(5);
        
        if weight_a > weight_b {
            self.policy_result_to_decision(&conflict.a_result, &conflict.orisha_a)
        } else if weight_b > weight_a {
            self.policy_result_to_decision(&conflict.b_result, &conflict.orisha_b)
        } else {
            self.resolve_lexical(conflict)
        }
    }
    
    fn resolve_lexical(&self, conflict: &PolicyConflict) -> ArbitrationDecision {
        if conflict.orisha_a <= conflict.orisha_b {
            self.policy_result_to_decision(&conflict.a_result, &conflict.orisha_a)
        } else {
            self.policy_result_to_decision(&conflict.b_result, &conflict.orisha_b)
        }
    }
    
    fn resolve_hierarchical(&self, conflict: &PolicyConflict, superior: &OrishaId) -> ArbitrationDecision {
        if &conflict.orisha_a == superior {
            self.policy_result_to_decision(&conflict.a_result, &conflict.orisha_a)
        } else if &conflict.orisha_b == superior {
            self.policy_result_to_decision(&conflict.b_result, &conflict.orisha_b)
        } else {
            ArbitrationDecision::DeferToHuman {
                reason: format!("Conflict between {} and {}; {} not involved", 
                    conflict.orisha_a, conflict.orisha_b, superior),
                escalation_path: vec![superior.clone()],
            }
        }
    }
    
    async fn resolve_consensus(
        &self, 
        _conflict: &PolicyConflict, 
        quorum: &[OrishaId],
        request: &ArbitrationRequest,
    ) -> Result<ArbitrationDecision, ArbitrationError> {
        let mut approvals = 0u8;
        let mut rejections = 0u8;
        
        for orisha in quorum {
            if let Some(policy) = self.policies.get(orisha) {
                let result = policy.validate(&request.state, &request.ops, &request.context).await;
                match result {
                    PolicyResult::Approved { .. } => approvals += 1,
                    PolicyResult::Rejected { .. } => rejections += 1,
                    PolicyResult::Conditional { .. } => {}
                }
            }
        }
        
        let total = approvals + rejections;
        if total == 0 {
            return Ok(ArbitrationDecision::DeferToHuman {
                reason: "No quorum members could evaluate".into(),
                escalation_path: quorum.to_vec(),
            });
        }
        
        if approvals as f64 / total as f64 >= self.consensus_threshold {
            Ok(ArbitrationDecision::Approve {
                winning_orisha: "consensus".into(),
                rationale: format!("{}/{} quorum approved", approvals, total),
            })
        } else {
            Ok(ArbitrationDecision::Reject {
                winning_orisha: "consensus".into(),
                rationale: format!("{}/{} quorum rejected", rejections, total),
            })
        }
    }
    
    fn resolve_deterministic_random(
        &self, 
        conflict: &PolicyConflict, 
        seed_source: &str,
        request: &ArbitrationRequest,
    ) -> ArbitrationDecision {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(seed_source.as_bytes());
        hasher.update(request.request_id.as_bytes());
        hasher.update(conflict.orisha_a.as_bytes());
        hasher.update(conflict.orisha_b.as_bytes());
        let hash = hasher.finalize();
        
        if hash[0] % 2 == 0 {
            self.policy_result_to_decision(&conflict.a_result, &conflict.orisha_a)
        } else {
            self.policy_result_to_decision(&conflict.b_result, &conflict.orisha_b)
        }
    }
    
    fn policy_result_to_decision(&self, result: &PolicyResult, orisha: &OrishaId) -> ArbitrationDecision {
        match result {
            PolicyResult::Approved { reasons } => ArbitrationDecision::Approve {
                winning_orisha: orisha.clone(),
                rationale: reasons.join("; "),
            },
            PolicyResult::Rejected { reasons, .. } => ArbitrationDecision::Reject {
                winning_orisha: orisha.clone(),
                rationale: reasons.join("; "),
            },
            PolicyResult::Conditional { requires_blessing, reason } => ArbitrationDecision::Conditional {
                blessing_required: requires_blessing.clone(),
                rationale: format!("{}: {}", orisha, reason),
            },
        }
    }
    
    fn aggregate_decisions(
        &self,
        decisions: Vec<ArbitrationDecision>,
    ) -> Result<ArbitrationDecision, ArbitrationError> {
        if decisions.iter().any(|d| matches!(d, ArbitrationDecision::Reject { .. })) {
            let reject = decisions.iter().find(|d| matches!(d, ArbitrationDecision::Reject { .. })).unwrap();
            return Ok(reject.clone());
        }
        
        if let Some(cond) = decisions.iter().find(|d| matches!(d, ArbitrationDecision::Conditional { .. })) {
            return Ok(cond.clone());
        }
        
        if let Some(defer) = decisions.iter().find(|d| matches!(d, ArbitrationDecision::DeferToHuman { .. })) {
            return Ok(defer.clone());
        }
        
        if let Some(approve) = decisions.iter().find(|d| matches!(d, ArbitrationDecision::Approve { .. })) {
            return Ok(approve.clone());
        }
        
        Ok(ArbitrationDecision::Reject {
            winning_orisha: "arbitration_engine".into(),
            rationale: "No decisive outcome from arbitration".into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArbitrationError {
    #[error("No conflict to resolve")]
    NoConflictToResolve,
    #[error("Policy evaluation failed")]
    PolicyEvaluationFailed,
    #[error("Consensus timeout")]
    ConsensusTimeout,
}
