use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_RECENT_FILES: usize = 10;
const RECENT_FILES_PATH: &str = ".recent_files.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFilesManager {
    files: Vec<PathBuf>,
    #[serde(skip, default = "default_storage_path")]
    storage_path: PathBuf,
}

fn default_storage_path() -> PathBuf {
    PathBuf::from(RECENT_FILES_PATH)
}

impl Default for RecentFilesManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecentFilesManager {
    pub fn new() -> Self {
        Self { 
            files: Vec::new(),
            storage_path: default_storage_path(),
        }
    }
    
    pub fn with_storage_path(path: PathBuf) -> Self {
        Self {
            files: Vec::new(),
            storage_path: path,
        }
    }

    pub fn set_storage_path(&mut self, path: PathBuf) {
        self.storage_path = path;
    }

    pub fn load() -> Self {
        match fs::read_to_string(RECENT_FILES_PATH) {
            Ok(contents) => match serde_json::from_str::<Self>(&contents) {
                Ok(mut manager) => {
                    tracing::info!("Loaded {} recent files", manager.files.len());
                    // Ensure path is set correctly after deserialize
                    manager.storage_path = default_storage_path();
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
        let json =
            serde_json::to_string_pretty(&self).context("Failed to serialize recent files")?;
        fs::write(&self.storage_path, json).with_context(|| format!("Failed to write recent files to {:?}", self.storage_path))?;
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
    use tempfile::NamedTempFile;

    fn test_manager() -> (RecentFilesManager, tempfile::TempPath) {
        let file = NamedTempFile::new().unwrap();
        let path = file.into_temp_path();
        // Keep path alive but close file so we can write to it? 
        // Or just generate a random path in temp dir.
        // NamedTempFile deletes on drop. useful.
        
        let manager = RecentFilesManager::with_storage_path(path.to_path_buf());
        (manager, path)
    }
    
    #[test]
    fn test_recent_files_add() {
        let (mut manager, _path) = test_manager();
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
        let (mut manager, _path) = test_manager();
        let path = PathBuf::from("test.ron");

        manager.add_file(path.clone());
        manager.add_file(path.clone());

        assert_eq!(manager.get_files().len(), 1);
        assert_eq!(manager.get_files()[0], path);
    }

    #[test]
    fn test_recent_files_max_limit() {
        let (mut manager, _path) = test_manager();

        for i in 0..15 {
            manager.add_file(PathBuf::from(format!("test{}.ron", i)));
        }

        assert_eq!(manager.get_files().len(), MAX_RECENT_FILES);
        // Should be test14 ... test5
        assert_eq!(manager.get_files()[0], PathBuf::from("test14.ron"));
    }

    #[test]
    fn test_recent_files_clear() {
        let (mut manager, _path) = test_manager();
        manager.add_file(PathBuf::from("test.ron"));
        manager.clear();

        assert_eq!(manager.get_files().len(), 0);
    }
    
    #[test]
    fn test_persistence() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        // Allow file to be deleted/closed so we can write
        drop(file);
        
        {
            let mut manager = RecentFilesManager::with_storage_path(path.clone());
            manager.add_file(PathBuf::from("persist.ron"));
        } // drops manager, file should be written
        
        assert!(path.exists());
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("persist.ron"));
        
        fs::remove_file(path).unwrap();
    }
    
    #[test]
    fn test_remove_missing_files() {
        let (mut manager, _path) = test_manager();
        
        // create a real file
        let real_file = NamedTempFile::new().unwrap();
        let real_path = real_file.path().to_path_buf();
        
        let fake_path = PathBuf::from("non_existent_file_xyz.ron");
        
        manager.add_file(real_path.clone());
        manager.add_file(fake_path.clone());
        
        assert_eq!(manager.get_files().len(), 2);
        
        manager.remove_missing_files();
        
        assert_eq!(manager.get_files().len(), 1);
        assert_eq!(manager.get_files()[0], real_path);
    }
}
