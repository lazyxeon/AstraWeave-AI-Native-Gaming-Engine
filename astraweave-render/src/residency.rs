use astraweave_asset::{AssetDatabase, AssetKind};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

/// Tracks residency of assets in GPU memory for streaming.
/// Evicts least-recently-used assets when memory pressure is high.
pub struct ResidencyManager {
    db: Arc<Mutex<AssetDatabase>>,
    loaded_assets: HashMap<String, ResidencyInfo>, // GUID -> info
    lru_queue: VecDeque<String>,                   // GUIDs in LRU order
    max_memory_mb: usize,
    current_memory_mb: usize,
    hot_reload_rx: watch::Receiver<()>,
}

#[derive(Debug, Clone)]
pub struct ResidencyInfo {
    pub kind: AssetKind,
    pub memory_mb: usize,
    pub last_used: std::time::Instant,
    pub gpu_handle: Option<String>, // Placeholder for actual GPU handle
}

impl ResidencyManager {
    pub fn new(db: Arc<Mutex<AssetDatabase>>, max_memory_mb: usize) -> Self {
        let (_tx, rx) = watch::channel(());
        Self {
            db,
            loaded_assets: HashMap::new(),
            lru_queue: VecDeque::new(),
            max_memory_mb,
            current_memory_mb: 0,
            hot_reload_rx: rx,
        }
    }

    pub fn with_hot_reload(
        db: Arc<Mutex<AssetDatabase>>,
        max_memory_mb: usize,
        rx: watch::Receiver<()>,
    ) -> Self {
        Self {
            db,
            loaded_assets: HashMap::new(),
            lru_queue: VecDeque::new(),
            max_memory_mb,
            current_memory_mb: 0,
            hot_reload_rx: rx,
        }
    }

    /// Load an asset into residency if not already loaded.
    pub fn load_asset(&mut self, guid: &str) -> Result<(), anyhow::Error> {
        if self.loaded_assets.contains_key(guid) {
            // Already loaded, update LRU
            self.touch_asset(guid);
            return Ok(());
        }

        // Get asset metadata
        let (meta, memory_mb) = {
            let db = self
                .db
                .lock()
                .map_err(|e| anyhow::anyhow!("Residency DB lock poisoned: {}", e))?;
            if let Some(meta) = db.get_asset(guid) {
                let memory_mb = (meta.size_bytes / (1024 * 1024)) as usize + 1;
                (meta.clone(), memory_mb)
            } else {
                return Err(anyhow::anyhow!("Asset {} not found in database", guid));
            }
        }; // db lock dropped here

        // Evict if necessary
        while self.current_memory_mb + memory_mb > self.max_memory_mb {
            self.evict_lru()?;
        }

        // Load asset (placeholder: in real impl, upload to GPU)
        let info = ResidencyInfo {
            kind: meta.kind,
            memory_mb,
            last_used: std::time::Instant::now(),
            gpu_handle: Some(format!("gpu_{}", guid)),
        };

        self.loaded_assets.insert(guid.to_string(), info);
        self.lru_queue.push_back(guid.to_string());
        self.current_memory_mb += memory_mb;

        Ok(())
    }

    /// Mark asset as recently used.
    pub fn touch_asset(&mut self, guid: &str) {
        if let Some(info) = self.loaded_assets.get_mut(guid) {
            info.last_used = std::time::Instant::now();
            // Move to back of LRU queue
            if let Some(pos) = self.lru_queue.iter().position(|g| g == guid) {
                self.lru_queue.remove(pos);
                self.lru_queue.push_back(guid.to_string());
            }
        }
    }

    /// Evict the least recently used asset.
    pub fn evict_lru(&mut self) -> Result<(), anyhow::Error> {
        if let Some(guid) = self.lru_queue.pop_front() {
            if let Some(info) = self.loaded_assets.remove(&guid) {
                self.current_memory_mb = self.current_memory_mb.saturating_sub(info.memory_mb);
                // Placeholder: unload from GPU
                log::debug!("Evicted asset {}", guid);
            }
        }
        Ok(())
    }

    /// Get loaded assets.
    pub fn get_loaded_assets(&self) -> Vec<String> {
        self.loaded_assets.keys().cloned().collect()
    }

    /// Check for hot-reload notifications and invalidate affected assets.
    pub fn check_hot_reload(&mut self) {
        if self.hot_reload_rx.has_changed().unwrap_or(false) {
            // Mark hot-reload notification as seen
            let _ = self.hot_reload_rx.borrow_and_update();
            // Clear all loaded assets on hot-reload signal
            self.loaded_assets.clear();
            self.lru_queue.clear();
            self.current_memory_mb = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_asset::AssetMetadata;

    #[test]
    fn test_residency_load_and_evict() -> Result<(), Box<dyn std::error::Error>> {
        let db = Arc::new(Mutex::new(AssetDatabase::new()));
        let mut rm = ResidencyManager::new(db.clone(), 10); // 10 MB limit

        // Mock asset
        let guid = "test_guid".to_string();
        {
            let mut db = db
                .lock()
                .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?;
            db.assets.insert(
                guid.clone(),
                AssetMetadata {
                    guid: guid.clone(),
                    path: "test".to_string(),
                    kind: AssetKind::Texture,
                    hash: "hash".to_string(),
                    dependencies: vec![],
                    last_modified: 0,
                    size_bytes: 5 * 1024 * 1024, // 5 MB
                },
            );
        }

        // Load asset
        rm.load_asset(&guid).unwrap();
        assert!(rm.loaded_assets.contains_key(&guid));
        assert_eq!(rm.current_memory_mb, 6); // 5MB rounds up to 6MB (size/MB + 1)

        // Touch
        rm.touch_asset(&guid);

        // Load another to trigger eviction
        let guid2 = "test_guid2".to_string();
        {
            let mut db = db
                .lock()
                .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?;
            db.assets.insert(
                guid2.clone(),
                AssetMetadata {
                    guid: guid2.clone(),
                    path: "test2".to_string(),
                    kind: AssetKind::Texture,
                    hash: "hash2".to_string(),
                    dependencies: vec![],
                    last_modified: 0,
                    size_bytes: 6 * 1024 * 1024, // 6 MB
                },
            );
        }

        rm.load_asset(&guid2).unwrap();
        // Should have evicted guid
        assert!(!rm.loaded_assets.contains_key(&guid));
        assert!(rm.loaded_assets.contains_key(&guid2));
        assert_eq!(rm.current_memory_mb, 7); // 6MB rounds up to 7MB (size/MB + 1)
        Ok(())
    }

    // --- Mutation-resistant tests ---

    fn make_test_db() -> Arc<Mutex<AssetDatabase>> {
        Arc::new(Mutex::new(AssetDatabase::new()))
    }

    fn insert_asset(db: &Arc<Mutex<AssetDatabase>>, guid: &str, size_mb: u64) {
        let mut db = db.lock().unwrap();
        db.assets.insert(
            guid.to_string(),
            AssetMetadata {
                guid: guid.to_string(),
                path: format!("assets/{guid}"),
                kind: AssetKind::Texture,
                hash: format!("hash_{guid}"),
                dependencies: vec![],
                last_modified: 0,
                size_bytes: size_mb * 1024 * 1024,
            },
        );
    }

    #[test]
    fn new_manager_starts_empty() {
        let db = make_test_db();
        let rm = ResidencyManager::new(db, 100);
        assert!(rm.loaded_assets.is_empty());
        assert_eq!(rm.current_memory_mb, 0);
        assert!(rm.get_loaded_assets().is_empty());
    }

    #[test]
    fn load_asset_not_found_returns_error() {
        let db = make_test_db();
        let mut rm = ResidencyManager::new(db, 100);
        let result = rm.load_asset("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn load_asset_succeeds_and_appears_in_loaded() {
        let db = make_test_db();
        insert_asset(&db, "tex_a", 2);
        let mut rm = ResidencyManager::new(db, 100);
        rm.load_asset("tex_a").unwrap();
        assert!(rm.loaded_assets.contains_key("tex_a"));
        assert!(rm.get_loaded_assets().contains(&"tex_a".to_string()));
        assert!(rm.current_memory_mb > 0);
    }

    #[test]
    fn duplicate_load_does_not_increase_memory() {
        let db = make_test_db();
        insert_asset(&db, "tex_a", 2);
        let mut rm = ResidencyManager::new(db, 100);
        rm.load_asset("tex_a").unwrap();
        let mem_after_first = rm.current_memory_mb;
        rm.load_asset("tex_a").unwrap(); // duplicate
        assert_eq!(
            rm.current_memory_mb, mem_after_first,
            "duplicate load should not increase memory"
        );
    }

    #[test]
    fn evict_lru_removes_oldest_asset() {
        let db = make_test_db();
        insert_asset(&db, "a", 1);
        insert_asset(&db, "b", 1);
        let mut rm = ResidencyManager::new(db, 100);
        rm.load_asset("a").unwrap();
        rm.load_asset("b").unwrap();
        assert_eq!(rm.get_loaded_assets().len(), 2);
        rm.evict_lru().unwrap();
        // "a" was loaded first, should be evicted
        assert!(!rm.loaded_assets.contains_key("a"));
        assert!(rm.loaded_assets.contains_key("b"));
    }

    #[test]
    fn touch_moves_to_back_of_lru() {
        let db = make_test_db();
        insert_asset(&db, "a", 1);
        insert_asset(&db, "b", 1);
        let mut rm = ResidencyManager::new(db, 100);
        rm.load_asset("a").unwrap();
        rm.load_asset("b").unwrap();
        rm.touch_asset("a"); // move "a" to back
        rm.evict_lru().unwrap();
        // Now "b" should be evicted (it's the LRU)
        assert!(
            rm.loaded_assets.contains_key("a"),
            "touched 'a' should survive"
        );
        assert!(!rm.loaded_assets.contains_key("b"), "'b' should be evicted");
    }

    #[test]
    fn memory_pressure_triggers_eviction() {
        let db = make_test_db();
        insert_asset(&db, "big", 8); // will be ~9 MB
        insert_asset(&db, "new", 5); // will be ~6 MB
        let mut rm = ResidencyManager::new(db, 12); // 12 MB limit
        rm.load_asset("big").unwrap();
        assert!(rm.loaded_assets.contains_key("big"));
        rm.load_asset("new").unwrap();
        // big should have been evicted to make room
        assert!(
            !rm.loaded_assets.contains_key("big"),
            "big should be evicted"
        );
        assert!(rm.loaded_assets.contains_key("new"));
    }

    #[test]
    fn gpu_handle_is_set_on_load() {
        let db = make_test_db();
        insert_asset(&db, "tex", 1);
        let mut rm = ResidencyManager::new(db, 100);
        rm.load_asset("tex").unwrap();
        let info = &rm.loaded_assets["tex"];
        assert_eq!(info.gpu_handle, Some("gpu_tex".to_string()));
    }
}
