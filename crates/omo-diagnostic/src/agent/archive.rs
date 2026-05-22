use std::fs;
use std::path::{Path, PathBuf};
use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};
use crate::Diagnostic;
use tracing::{info, error};

#[derive(Serialize, Deserialize, Debug)]
pub struct SabbathSeal {
    pub week_ending: String,
    pub total_diagnostics: u64,
    pub fixed_count: u64,
    pub disputed_count: u64,
    pub fingerprint_hash: String,
}

pub struct Archive {
    pub root: PathBuf,
}

impl Archive {
    pub fn new(root: &Path) -> Self {
        let archive_path = root.join("ops/archive");
        let _ = fs::create_dir_all(&archive_path);
        Self { root: archive_path }
    }

    pub fn store_receipt(&self, diag: &Diagnostic) -> Result<(), std::io::Error> {
        let filename = format!("{}_{}.json", diag.diagnostic.code, diag.trace_id);
        let path = self.root.join(filename);
        let json = serde_json::to_string_pretty(diag).unwrap();
        fs::write(path, json)?;
        Ok(())
    }

    pub fn seal_week(&self) -> Result<SabbathSeal, std::io::Error> {
        let entries = fs::read_dir(&self.root)?;
        let mut total = 0;
        let mut fixed = 0;
        
        for entry in entries {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                total += 1;
                // In a real implementation, we'd parse the status
            }
        }

        let seal = SabbathSeal {
            week_ending: Utc::now().to_rfc3339(),
            total_diagnostics: total,
            fixed_count: fixed,
            disputed_count: 0,
            fingerprint_hash: "mock_fingerprint".into(),
        };

        let seal_json = serde_json::to_string_pretty(&seal).unwrap();
        fs::write(self.root.join("sabbath_seal.json"), seal_json)?;
        
        info!("🕯️  Sabbath Seal created: {} total diagnostics recorded.", total);
        Ok(seal)
    }
}
