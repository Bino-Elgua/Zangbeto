use std::path::{Path, PathBuf};
use tokio::process::Command;
use tempfile::TempDir;
use tracing::{info, error};

pub struct Sandbox {
    pub root: TempDir,
    pub original_dir: PathBuf,
}

impl Sandbox {
    pub async fn new(original_dir: &Path) -> Result<Self, std::io::Error> {
        let temp = tempfile::tempdir()?;
        info!("🛠  Created sandbox at: {:?}", temp.path());
        
        // Copy relevant files (minimal for demo: just the shrine directory)
        // In prod, this would be a full git clone or selective copy
        let _ = Command::new("cp")
            .arg("-r")
            .arg(original_dir.join("shrine"))
            .arg(temp.path())
            .spawn()?
            .wait()
            .await?;

        Ok(Self {
            root: temp,
            original_dir: original_dir.to_path_buf(),
        })
    }

    pub async fn run_command(&self, cmd: &str, args: &[&str]) -> Result<bool, std::io::Error> {
        let status = Command::new(cmd)
            .args(args)
            .current_dir(self.root.path())
            .spawn()?
            .wait()
            .await?;
        
        Ok(status.success())
    }

    pub async fn apply_patch(&self, patch: &str) -> Result<bool, std::io::Error> {
        let patch_path = self.root.path().join("repair.patch");
        tokio::fs::write(&patch_path, patch).await?;

        let status = Command::new("patch")
            .arg("-p1")
            .arg("-i")
            .arg("repair.patch")
            .current_dir(self.root.path())
            .spawn()?
            .wait()
            .await?;

        Ok(status.success())
    }

    pub fn path(&self) -> &Path {
        self.root.path()
    }
}
