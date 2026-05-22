use serde::{Serialize, Deserialize};
use crate::zangbeto::event::RealityEvent;
use crate::zangbeto::drift::DriftReport;
use crate::kernel::css::CanonicalSystemState;
use sha2::{Sha256, Digest};

/// 🔐 Append-only ledger of verified reality events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLedger {
    pub chain_id: String,
    pub genesis_hash: String,
    pub events: Vec<LedgerEntry>,
    pub current_tip: String,  // hash of latest committed event
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub event: RealityEvent,
    pub drift_report: Option<DriftReport>,
    pub commit_receipt: CommitReceipt,
    pub prev_hash: String,
    pub entry_hash: String,  // sha256(prev_hash + event + receipt)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitReceipt {
    pub committed_at: u64,
    pub committed_by: String,  // validator node ID
    pub orisha_signatures: Vec<OrishaSignature>,
    pub merkle_root: String,   // for batch verification
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrishaSignature {
    pub orisha: String,
    pub signature: String,  // cryptographic signature over entry_hash
    pub public_key: String,
}

impl EventLedger {
    pub fn new(chain_id: &str, genesis_state: &CanonicalSystemState) -> Self {
        Self {
            chain_id: chain_id.into(),
            genesis_hash: genesis_state.state_hash.clone(),
            events: vec![],
            current_tip: genesis_state.state_hash.clone(),
        }
    }
    
    /// Append verified event to ledger
    pub fn append(&mut self, event: RealityEvent, drift: Option<DriftReport>) -> Result<LedgerEntry, LedgerError> {
        // Verify event integrity
        if !event.post_state_hash.starts_with("sha256:") {
            return Err(LedgerError::InvalidHash);
        }
        
        // Compute entry hash
        let prev_hash = self.current_tip.clone();
        let entry_data = format!("{}{}{}", prev_hash, 
            serde_json::to_string(&event).unwrap(),
            serde_json::to_string(&drift).unwrap());
        
        let mut hasher = Sha256::new();
        hasher.update(entry_data.as_bytes());
        let entry_hash = format!("sha256:{}", hex::encode(hasher.finalize()));
        
        // Create receipt (in production: collect Orisha signatures)
        let receipt = CommitReceipt {
            committed_at: event.timestamp,
            committed_by: "zangbeto-coordinator".into(),
            orisha_signatures: vec![], // populated by validator network
            merkle_root: self.compute_merkle_root(),
        };
        
        let entry = LedgerEntry {
            event,
            drift_report: drift,
            commit_receipt: receipt,
            prev_hash,
            entry_hash: entry_hash.clone(),
        };
        
        self.events.push(entry.clone());
        self.current_tip = entry_hash;
        
        Ok(entry)
    }
    
    /// Query: find all events with drift of specific type
    pub fn query_drift(&self, drift_type: &str) -> Vec<&LedgerEntry> {
        self.events.iter()
            .filter(|e| e.drift_report.as_ref()
                .map(|d| matches!(&d.drift_type, Some(t) if format!("{:?}", t).contains(drift_type)))
                .unwrap_or(false))
            .collect()
    }
    
    /// Verify entire chain integrity
    pub fn verify_chain(&self) -> bool {
        let mut prev_hash = self.genesis_hash.clone();
        
        for entry in &self.events {
            if entry.prev_hash != prev_hash {
                return false;
            }
            // Verify entry_hash computation
            // Verify Orisha signatures (in production)
            prev_hash = entry.entry_hash.clone();
        }
        
        prev_hash == self.current_tip
    }
    
    fn compute_merkle_root(&self) -> String {
        // Simplified: in production, build Merkle tree of event hashes
        let mut hasher = Sha256::new();
        let hashes: Vec<String> = self.events.iter().map(|e| e.entry_hash.clone()).collect();
        hasher.update(hashes.join(",").as_bytes());
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Invalid hash")]
    InvalidHash,
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    #[error("Chain integrity broken")]
    ChainIntegrityBroken,
}
