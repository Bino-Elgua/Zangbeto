use serde::{Serialize, Deserialize};
use serde_cbor::ser::Serializer;
use sha2::{Sha256, Digest};

/// 📜 CanonicalEncode: deterministic binary serialization trait
pub trait CanonicalEncode {
    fn encode_canonical(&self) -> Vec<u8>;
    fn hash_canonical(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.encode_canonical());
        hasher.finalize().into()
    }
}

/// 🧬 CanonicalStateIR: semantic opcodes with deterministic encoding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", content = "args", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CanonicalStateIR {
    // Core state mutations
    AddTask { 
        intent: String, 
        priority: u8, 
        metadata: CanonicalValue 
    },
    UpdateMemory { 
        scope: MemoryScope, 
        key: String, 
        value: CanonicalValue,
        version_hint: Option<u64>
    },
    ModifyEconomy { 
        field: EconomyField, 
        delta: i64,
        authorization_proof: Option<AuthProof>
    },
    SetConstraint { 
        name: ConstraintName, 
        value: CanonicalValue,
        expiry: Option<u64>
    },
    
    // Orisha semantic ops
    BlessPath { 
        path: String, 
        by: OrishaId, 
        reason: String,
        signature: Option<Signature>
    },
    SealData { 
        path: String, 
        algorithm: SealAlgorithm, 
        by: OrishaId,
        key_ref: Option<KeyId>
    },
    PredictBranch { 
        horizon: u64, 
        confidence_threshold: f64,
        model_version: String
    },
    SyncTimeline { 
        target_timestamp: u64, 
        tolerance_ms: u64,
        sync_source: String
    },
    
    // Cross-cutting
    LogAnomaly { 
        severity: AnomalySeverity, 
        description: String, 
        context: CanonicalValue 
    },
    GenerateRepairDelta { 
        target_path: String, 
        strategy: RepairStrategy,
        rollback_point: Option<StateHash>
    },
}

/// 🧱 CanonicalValue: JSON-like but with deterministic encoding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CanonicalValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<CanonicalValue>),
    Object(std::collections::BTreeMap<String, CanonicalValue>), // BTreeMap for key order
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryScope { Public, Private, Ephemeral }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EconomyField { Balance, Reputation, Stake }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintName { PrivacyMode, Sabbath, MaxDiffSize, RateLimit }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SealAlgorithm { AES256GCM, ChaCha20Poly1305, WalrusSeal }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnomalySeverity { Low, Medium, High, Critical }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairStrategy { Rollback, Patch, Compensate, Quarantine }

pub type StateHash = [u8; 32];
pub type OrishaId = String;
pub type KeyId = String;
pub type Signature = Vec<u8>;
pub type AuthProof = Vec<u8>;

impl CanonicalEncode for CanonicalStateIR {
    fn encode_canonical(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut serializer = Serializer::new(&mut buf);
        self.serialize(&mut serializer).unwrap();
        buf
    }
}

impl CanonicalStateIR {
    pub fn target_path(&self) -> Option<String> {
        match self {
            CanonicalStateIR::UpdateMemory { key, scope, .. } => {
                Some(format!("/memory/{:?}/{}", scope, key))
            }
            CanonicalStateIR::BlessPath { path, .. } => Some(path.clone()),
            CanonicalStateIR::SealData { path, .. } => Some(path.clone()),
            _ => None,
        }
    }
}

impl CanonicalEncode for CanonicalValue {
    fn encode_canonical(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut serializer = Serializer::new(&mut buf);
        self.serialize(&mut serializer).unwrap();
        buf
    }
}
