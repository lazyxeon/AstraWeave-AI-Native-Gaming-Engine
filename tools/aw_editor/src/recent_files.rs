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

    /// Returns the number of recent files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns true if there are no recent files.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the most recently opened file.
    pub fn most_recent(&self) -> Option<&PathBuf> {
        self.files.first()
    }

    /// Returns the capacity (maximum number of files).
    pub fn capacity(&self) -> usize {
        MAX_RECENT_FILES
    }

    /// Returns true if at capacity.
    pub fn is_at_capacity(&self) -> bool {
        self.files.len() >= MAX_RECENT_FILES
    }

    /// Returns the number of files that still exist.
    pub fn existing_count(&self) -> usize {
        self.files.iter().filter(|p| p.exists()).count()
    }

    /// Returns the number of missing files.
    pub fn missing_count(&self) -> usize {
        self.files.iter().filter(|p| !p.exists()).count()
    }

    /// Returns true if there are missing files.
    pub fn has_missing_files(&self) -> bool {
        self.missing_count() > 0
    }

    /// Returns files matching a pattern (case-insensitive filename contains).
    pub fn find_by_pattern(&self, pattern: &str) -> Vec<&PathBuf> {
        let pattern_lower = pattern.to_lowercase();
        self.files
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_lowercase().contains(&pattern_lower))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Returns files with a specific extension.
    pub fn filter_by_extension(&self, ext: &str) -> Vec<&PathBuf> {
        let ext_lower = ext.to_lowercase();
        self.files
            .iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase() == ext_lower)
                    .unwrap_or(false)
            })
            .collect()
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

    // ====================================================================
    // RecentFilesManager New Methods Tests
    // ====================================================================

    #[test]
    fn test_recent_files_len_is_empty() {
        let (manager, _path) = test_manager();
        assert_eq!(manager.len(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_recent_files_most_recent() {
        let (mut manager, _path) = test_manager();
        assert!(manager.most_recent().is_none());

        manager.add_file(PathBuf::from("first.ron"));
        manager.add_file(PathBuf::from("second.ron"));

        assert_eq!(manager.most_recent(), Some(&PathBuf::from("second.ron")));
    }

    #[test]
    fn test_recent_files_capacity() {
        let (manager, _path) = test_manager();
        assert_eq!(manager.capacity(), MAX_RECENT_FILES);
    }

    #[test]
    fn test_recent_files_is_at_capacity() {
        let (mut manager, _path) = test_manager();
        assert!(!manager.is_at_capacity());

        for i in 0..MAX_RECENT_FILES {
            manager.add_file(PathBuf::from(format!("file{}.ron", i)));
        }

        assert!(manager.is_at_capacity());
    }

    #[test]
    fn test_recent_files_existing_missing_count() {
        let (mut manager, _path) = test_manager();

        let real_file = NamedTempFile::new().unwrap();
        let real_path = real_file.path().to_path_buf();

        manager.add_file(real_path);
        manager.add_file(PathBuf::from("nonexistent.ron"));

        assert_eq!(manager.existing_count(), 1);
        assert_eq!(manager.missing_count(), 1);
        assert!(manager.has_missing_files());
    }

    #[test]
    fn test_recent_files_find_by_pattern() {
        let (mut manager, _path) = test_manager();

        manager.add_file(PathBuf::from("level1.ron"));
        manager.add_file(PathBuf::from("level2.ron"));
        manager.add_file(PathBuf::from("settings.ron"));

        let results = manager.find_by_pattern("level");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_recent_files_filter_by_extension() {
        let (mut manager, _path) = test_manager();

        manager.add_file(PathBuf::from("scene.ron"));
        manager.add_file(PathBuf::from("config.json"));
        manager.add_file(PathBuf::from("data.ron"));

        let rons = manager.filter_by_extension("ron");
        assert_eq!(rons.len(), 2);

        let jsons = manager.filter_by_extension("json");
        assert_eq!(jsons.len(), 1);
    }
}
