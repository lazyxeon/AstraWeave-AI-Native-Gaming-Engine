use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_RECENT_FILES: usize = 10;
const RECENT_FILES_PATH: &str = ".recent_files.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFilesManager {
    files: Vec<PathBuf>,
}

impl Default for RecentFilesManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecentFilesManager {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn load() -> Self {
        match fs::read_to_string(RECENT_FILES_PATH) {
            Ok(contents) => match serde_json::from_str::<Self>(&contents) {
                Ok(manager) => {
                    tracing::info!("Loaded {} recent files", manager.files.len());
                    manager
                }
                Err(e) => {
                    tracing::warn!("Failed to parse recent files: {}. Resetting.", e);
                    Self::new()
                }
            },
            Err(_) => Self::new(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)
            .context("Failed to serialize recent files")?;
        fs::write(RECENT_FILES_PATH, json)
            .context("Failed to write recent files to disk")?;
        Ok(())
    }

    pub fn add_file(&mut self, path: PathBuf) {
        if let Some(pos) = self.files.iter().position(|p| p == &path) {
            self.files.remove(pos);
        }

        self.files.insert(0, path);

        if self.files.len() > MAX_RECENT_FILES {
            self.files.truncate(MAX_RECENT_FILES);
        }

        if let Err(e) = self.save() {
            tracing::error!("Failed to save recent files: {}", e);
        }
    }

    pub fn get_files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn clear(&mut self) {
        self.files.clear();
        if let Err(e) = self.save() {
            tracing::error!("Failed to clear recent files: {}", e);
        }
    }

    pub fn remove_missing_files(&mut self) {
        let initial_count = self.files.len();
        self.files.retain(|path| path.exists());
        if self.files.len() != initial_count {
            if let Err(e) = self.save() {
                tracing::error!("Failed to save recent files after removing missing: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_files_add() {
        let mut manager = RecentFilesManager::new();
        let path1 = PathBuf::from("test1.ron");
        let path2 = PathBuf::from("test2.ron");

        manager.add_file(path1.clone());
        assert_eq!(manager.get_files().len(), 1);
        assert_eq!(manager.get_files()[0], path1);

        manager.add_file(path2.clone());
        assert_eq!(manager.get_files().len(), 2);
        assert_eq!(manager.get_files()[0], path2);
        assert_eq!(manager.get_files()[1], path1);
    }

    #[test]
    fn test_recent_files_duplicate() {
        let mut manager = RecentFilesManager::new();
        let path = PathBuf::from("test.ron");

        manager.add_file(path.clone());
        manager.add_file(path.clone());

        assert_eq!(manager.get_files().len(), 1);
        assert_eq!(manager.get_files()[0], path);
    }

    #[test]
    fn test_recent_files_max_limit() {
        let mut manager = RecentFilesManager::new();

        for i in 0..15 {
            manager.add_file(PathBuf::from(format!("test{}.ron", i)));
        }

        assert_eq!(manager.get_files().len(), MAX_RECENT_FILES);
    }

    #[test]
    fn test_recent_files_clear() {
        let mut manager = RecentFilesManager::new();
        manager.add_file(PathBuf::from("test.ron"));
        manager.clear();

        assert_eq!(manager.get_files().len(), 0);
    }
}
