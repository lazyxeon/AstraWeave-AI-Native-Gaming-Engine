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
                println!("Evicted asset {}", guid);
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
    fn test_residency_load_and_evict() {
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
    }
}
