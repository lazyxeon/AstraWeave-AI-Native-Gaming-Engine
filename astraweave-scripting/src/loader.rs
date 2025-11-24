use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct ScriptAsset {
    pub path: PathBuf,
    pub source: String,
    pub hash: String,
}

pub struct ScriptLoader;

impl ScriptLoader {
    pub async fn load(path: impl AsRef<Path>) -> Result<ScriptAsset> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .await
            .context(format!("Failed to read script file: {}", path.display()))?;
        
        // Compute hash for change detection
        let hash = compute_hash(&source);

        Ok(ScriptAsset {
            path: path.to_path_buf(),
            source,
            hash,
        })
    }
}

fn compute_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
