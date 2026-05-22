use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use vm_core::ir::{CanonicalValue, OrishaId};

/// 🧵 CRDT Memory: conflict-free replicated state with semantic merge
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrdtMemory {
    /// Public hive: LWW-Register with vector clock
    pub public_hive: LwwRegisterMap,
    /// Private seal: encrypted OR-Set with capability gating
    pub private_seal: Option<OrSet>,
    /// Ephemeral: G-Counter for transient metrics
    pub ephemeral: GCounter,
    /// Vector clock for causality tracking
    pub vector_clock: VectorClock,
}

/// 📊 LWW-Register Map: last-writer-wins with deterministic tie-breaking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LwwRegisterMap {
    pub entries: BTreeMap<String, LwwRegister>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwRegister {
    pub value: CanonicalValue,
    pub timestamp: u64,
    pub node_id: String,  // for tie-breaking: lexicographic order
}

impl LwwRegister {
    pub fn merge(self, other: Self) -> Self {
        if self.timestamp > other.timestamp {
            self
        } else if other.timestamp > self.timestamp {
            other
        } else {
            // Tie: lexicographic node_id comparison (deterministic)
            if self.node_id <= other.node_id { self } else { other }
        }
    }
}

/// 🔐 OR-Set: observed-remove set for private data with capability gating
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrSet {
    pub elements: BTreeMap<String, ElementState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementState {
    pub added_by: BTreeMap<String, u64>,  // node_id -> timestamp
    pub removed_by: BTreeMap<String, u64>,
    pub encryption_ref: Option<String>,  // walrus:// or ipfs://
}

impl OrSet {
    pub fn add(&mut self, element: String, node_id: String, timestamp: u64) {
        self.elements
            .entry(element)
            .or_insert_with(|| ElementState {
                added_by: BTreeMap::new(),
                removed_by: BTreeMap::new(),
                encryption_ref: None,
            })
            .added_by
            .insert(node_id, timestamp);
    }
    
    pub fn remove(&mut self, element: String, node_id: String, timestamp: u64) {
        if let Some(state) = self.elements.get_mut(&element) {
            state.removed_by.insert(node_id, timestamp);
        }
    }
    
    pub fn contains(&self, element: &str) -> bool {
        if let Some(state) = self.elements.get(element) {
            // Element exists if added and not removed by any node
            state.added_by.keys().any(|n| !state.removed_by.contains_key(n))
        } else {
            false
        }
    }
    
    /// 🧭 Merge two OR-Sets: union of adds, union of removes
    pub fn merge(mut self, other: OrSet) -> Self {
        for (elem, other_state) in other.elements {
            let entry = self.elements.entry(elem).or_insert_with(|| ElementState {
                added_by: BTreeMap::new(),
                removed_by: BTreeMap::new(),
                encryption_ref: None,
            });
            entry.added_by.extend(other_state.added_by);
            entry.removed_by.extend(other_state.removed_by);
            if entry.encryption_ref.is_none() {
                entry.encryption_ref = other_state.encryption_ref;
            }
        }
        self
    }
}

/// 🔢 G-Counter: grow-only counter for ephemeral metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GCounter {
    pub counts: BTreeMap<String, u64>,  // node_id -> count
}

impl GCounter {
    pub fn increment(&mut self, node_id: String, delta: u64) {
        *self.counts.entry(node_id).or_insert(0) += delta;
    }
    
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }
    
    pub fn merge(mut self, other: GCounter) -> Self {
        for (node, count) in other.counts {
            *self.counts.entry(node).or_insert(0) = 
                (*self.counts.get(&node).unwrap_or(&0)).max(count);
        }
        self
    }
}

/// ⏱️ Vector Clock: causality tracking for merge ordering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorClock {
    pub clocks: BTreeMap<String, u64>,
}

impl VectorClock {
    pub fn tick(&mut self, node_id: String) {
        *self.clocks.entry(node_id).or_insert(0) += 1;
    }
    
    pub fn merge(mut self, other: VectorClock) -> Self {
        for (node, time) in other.clocks {
            *self.clocks.entry(node).or_insert(0) = 
                (*self.clocks.get(&node).unwrap_or(&0)).max(time);
        }
        self
    }
    
    /// 🧭 Compare: returns HappensBefore, Concurrent, or HappensAfter
    pub fn compare(&self, other: &VectorClock) -> Causality {
        let mut self_greater = false;
        let mut other_greater = false;
        
        let all_nodes: std::collections::BTreeSet<_> = 
            self.clocks.keys().chain(other.clocks.keys()).collect();
        
        for node in all_nodes {
            let a = self.clocks.get(node).copied().unwrap_or(0);
            let b = other.clocks.get(node).copied().unwrap_or(0);
            if a > b { self_greater = true; }
            if b > a { other_greater = true; }
        }
        
        match (self_greater, other_greater) {
            (true, false) => Causality::HappensBefore,
            (false, true) => Causality::HappensAfter,
            (false, false) => Causality::Equal,
            (true, true) => Causality::Concurrent,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Causality {
    HappensBefore,
    HappensAfter,
    Equal,
    Concurrent,
}

/// 🧭 Merge Algebra: explicit operators for state reconciliation
pub trait MergeAlgebra: Sized {
    /// Join: least upper bound (merge)
    fn join(self, other: Self) -> Self;
    
    /// Meet: greatest lower bound (intersection)
    fn meet(self, other: Self) -> Option<Self>;
    
    /// Conflict set: extract divergent elements
    fn conflict_set(&self, other: &Self) -> ConflictSet;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConflictSet {
    pub divergent_paths: Vec<String>,
    pub resolution_hints: Vec<ResolutionHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionHint {
    pub path: String,
    pub strategy: MergeStrategy,
    pub orisha_recommendation: Option<OrishaId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    Lww,          // Last-writer-wins
    Semantic,     // Domain-specific merge function
    OrishaArb,    // Delegate to Orisha policy
    Manual,       // Require human review
}

impl MergeAlgebra for CrdtMemory {
    fn join(self, other: Self) -> Self {
        Self {
            public_hive: self.public_hive.join(other.public_hive),
            private_seal: match (self.private_seal, other.private_seal) {
                (Some(a), Some(b)) => Some(a.merge(b)),
                (a, b) => a.or(b),
            },
            ephemeral: self.ephemeral.merge(other.ephemeral),
            vector_clock: self.vector_clock.merge(other.vector_clock),
        }
    }
    
    fn meet(self, other: Self) -> Option<Self> {
        // Intersection: only elements present in both
        let public_meet = self.public_hive.meet(other.public_hive)?;
        Some(Self {
            public_hive: public_meet,
            private_seal: None,  // Private data requires capability match
            ephemeral: GCounter::default(),  // Ephemeral doesn't intersect
            vector_clock: self.vector_clock,  // Keep causality
        })
    }
    
    fn conflict_set(&self, other: &Self) -> ConflictSet {
        let mut conflicts = ConflictSet::default();
        
        // Compare public hive entries
        let all_keys: std::collections::BTreeSet<_> = 
            self.public_hive.entries.keys()
                .chain(other.public_hive.entries.keys())
                .cloned()
                .collect();
        
        for key in all_keys {
            match (self.public_hive.entries.get(&key), other.public_hive.entries.get(&key)) {
                (Some(a), Some(b)) if a.value != b.value => {
                    conflicts.divergent_paths.push(format!("/public/{}", key));
                    conflicts.resolution_hints.push(ResolutionHint {
                        path: format!("/public/{}", key),
                        strategy: MergeStrategy::Lww,
                        orisha_recommendation: Some("èṣù".into()),
                    });
                }
                (Some(_), None) | (None, Some(_)) => {
                    conflicts.divergent_paths.push(format!("/public/{}", key));
                    conflicts.resolution_hints.push(ResolutionHint {
                        path: format!("/public/{}", key),
                        strategy: MergeStrategy::Semantic,
                        orisha_recommendation: Some("yemọja".into()),
                    });
                }
                _ => {}
            }
        }
        
        conflicts
    }
}

impl MergeAlgebra for LwwRegisterMap {
    fn join(mut self, other: Self) -> Self {
        for (key, other_reg) in other.entries {
            if let Some(self_reg) = self.entries.get_mut(&key) {
                *self_reg = self_reg.clone().merge(other_reg);
            } else {
                self.entries.insert(key, other_reg);
            }
        }
        self
    }
    
    fn meet(self, other: Self) -> Option<Self> {
        let mut result = BTreeMap::new();
        for (key, self_reg) in &self.entries {
            if let Some(other_reg) = other.entries.get(key) {
                if self_reg.value == other_reg.value {
                    result.insert(key.clone(), self_reg.clone());
                }
            }
        }
        if result.is_empty() && !self.entries.is_empty() && !other.entries.is_empty() {
            None
        } else {
            Some(Self { entries: result })
        }
    }
    
    fn conflict_set(&self, _other: &Self) -> ConflictSet {
        ConflictSet::default()  // Simplified
    }
}
