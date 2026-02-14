//! Texture streaming and residency system with priority-based loading
//!
//! Provides LRU caching, priority queuing, and distance-based residency management.

use crate::texture::{Texture, TextureUsage};
use glam::Vec3;
use log::{debug, error, warn};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Unique identifier for textures in the asset system
pub type AssetId = String;

/// GPU texture handle wrapping the actual resource
#[derive(Debug, Clone)]
pub struct TextureHandle {
    pub id: AssetId,
    pub texture: Arc<Texture>,
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub memory_bytes: usize,
}

/// State of an asset in the streaming system
#[derive(Debug)]
enum AssetState {
    Loading,
    Resident(TextureHandle),
    Failed(#[allow(dead_code)] String),
}

/// Texture load request with priority
#[derive(Debug, Clone)]
struct LoadRequest {
    id: AssetId,
    priority: u32,
    distance: f32, // Distance from camera
}

impl Eq for LoadRequest {}

impl PartialEq for LoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.id == other.id
    }
}

impl PartialOrd for LoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then closer distance
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Reverse distance comparison (closer is higher priority)
                other
                    .distance
                    .partial_cmp(&self.distance)
                    .unwrap_or(Ordering::Equal)
            }
            other => other,
        }
    }
}

type LoadResult = Result<(AssetId, Texture), (AssetId, String)>;

/// Texture streaming manager with LRU eviction and priority-based loading
pub struct TextureStreamingManager {
    /// Map of asset states (Resident, Loading, Failed)
    assets: HashMap<AssetId, AssetState>,
    /// LRU queue for eviction (front = oldest)
    lru_queue: VecDeque<AssetId>,
    /// Priority queue for pending load requests
    load_queue: BinaryHeap<LoadRequest>,
    /// Set of IDs currently in load_queue, for O(1) dedup
    pending_ids: HashSet<AssetId>,
    /// Maximum GPU memory budget in bytes
    max_memory_bytes: usize,
    /// Current GPU memory usage in bytes
    current_memory_bytes: usize,
    /// Camera position for distance-based residency
    camera_position: Vec3,

    /// Channel for receiving async load results
    result_rx: mpsc::Receiver<LoadResult>,
    /// Sender to clone for async tasks
    result_tx: mpsc::Sender<LoadResult>,
}

impl TextureStreamingManager {
    /// Create a new texture streaming manager
    ///
    /// # Arguments
    /// * `max_memory_mb` - Maximum GPU memory budget in megabytes
    pub fn new(max_memory_mb: usize) -> Self {
        let (tx, rx) = mpsc::channel(32); // Buffer size of 32 results
        Self {
            assets: HashMap::new(),
            lru_queue: VecDeque::new(),
            load_queue: BinaryHeap::new(),
            pending_ids: HashSet::new(),
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory_bytes: 0,
            camera_position: Vec3::ZERO,
            result_rx: rx,
            result_tx: tx,
        }
    }

    /// Request a texture with given priority
    ///
    /// Returns immediately if texture is already loaded, otherwise queues for async loading.
    ///
    /// # Arguments
    /// * `id` - Asset ID of the texture (assumed to be file path)
    /// * `priority` - Load priority (higher = more urgent)
    /// * `distance` - Distance from camera (for tie-breaking)
    ///
    /// # Returns
    /// * `Some(TextureHandle)` if texture is resident in GPU memory
    /// * `None` if texture needs to be loaded (queued)
    pub fn request_texture(
        &mut self,
        id: AssetId,
        priority: u32,
        distance: f32,
    ) -> Option<TextureHandle> {
        // Check if resident
        if let Some(AssetState::Resident(handle)) = self.assets.get(&id) {
            // Update LRU (move to back)
            if let Some(pos) = self.lru_queue.iter().position(|x| x == &id) {
                self.lru_queue.remove(pos);
                self.lru_queue.push_back(id.clone());
            }
            return Some(handle.clone());
        }

        // Check if already loading or failed — don't re-queue
        if self.assets.contains_key(&id) {
            return None;
        }

        // Check if already pending in the load queue — don't double-queue
        if self.pending_ids.contains(&id) {
            return None;
        }

        // Queue for load — do NOT insert into assets map yet.
        // The Loading state will be set by process_next_load when it actually
        // picks this request off the queue and spawns the async task.
        self.pending_ids.insert(id.clone());
        self.load_queue.push(LoadRequest {
            id,
            priority,
            distance,
        });

        None
    }

    /// Process the next load request from the queue
    ///
    /// # Arguments
    /// * `device` - WGPU device for loading
    /// * `queue` - WGPU queue for loading
    pub fn process_next_load(&mut self, device: &Arc<wgpu::Device>, queue: &Arc<wgpu::Queue>) {
        // 1. Process completed loads
        while let Ok(result) = self.result_rx.try_recv() {
            match result {
                Ok((id, texture)) => {
                    let width = texture.texture.size().width;
                    let height = texture.texture.size().height;
                    let mip_levels = texture.texture.mip_level_count();
                    // Approximate memory: width * height * 4 bytes * 1.33 for mips
                    let memory_bytes =
                        (width * height * 4) as usize + ((width * height * 4) as usize / 3);

                    // Check budget before committing
                    while self.current_memory_bytes + memory_bytes > self.max_memory_bytes {
                        if !self.evict_lru() {
                            warn!("Memory budget full, forcing eviction for {}", id);
                            // Fix: Do not insert if budget is exceeded and eviction fails
                            return;
                        }
                    }

                    let handle = TextureHandle {
                        id: id.clone(),
                        texture: Arc::new(texture),
                        width,
                        height,
                        mip_levels,
                        memory_bytes,
                    };

                    self.current_memory_bytes += memory_bytes;
                    self.assets.insert(id.clone(), AssetState::Resident(handle));
                    self.lru_queue.push_back(id.clone());
                    debug!(
                        "Texture loaded: {} ({} MB)",
                        id,
                        memory_bytes as f32 / 1024.0 / 1024.0
                    );
                }
                Err((id, err)) => {
                    error!("Texture load failed for {}: {}", id, err);
                    self.assets.insert(id, AssetState::Failed(err));
                }
            }
        }

        // 2. Start new loads from queue
        // Pop one per call to throttle
        if let Some(request) = self.load_queue.pop() {
            // Remove from pending set since we're processing it now
            self.pending_ids.remove(&request.id);

            // Skip if already resident, loading, or failed
            if self.assets.contains_key(&request.id) {
                return;
            }

            // Now mark as Loading — this is the single insertion point for Loading state
            self.assets.insert(request.id.clone(), AssetState::Loading);

            // Spawn async task
            let device = device.clone();
            let queue = queue.clone();
            let tx = self.result_tx.clone();
            let id = request.id.clone();
            let path = request.id.clone(); // Assuming ID is path

            tokio::task::spawn(async move {
                let result =
                    Texture::load_texture_async(&device, &queue, &path, TextureUsage::Albedo).await;

                match result {
                    Ok(texture) => {
                        let _ = tx.send(Ok((id, texture))).await;
                    }
                    Err(e) => {
                        let _ = tx.send(Err((id, e.to_string()))).await;
                    }
                }
            });
        }
    }

    /// Evict the least recently used texture
    ///
    /// # Returns
    /// * `true` if a texture was evicted
    /// * `false` if no textures to evict
    pub fn evict_lru(&mut self) -> bool {
        if let Some(id) = self.lru_queue.pop_front() {
            if let Some(AssetState::Resident(handle)) = self.assets.remove(&id) {
                self.current_memory_bytes = self
                    .current_memory_bytes
                    .saturating_sub(handle.memory_bytes);
                debug!(
                    "Evicted texture {} ({:.2}MB)",
                    id,
                    handle.memory_bytes as f32 / (1024.0 * 1024.0)
                );
                return true;
            }
        }
        false
    }

    /// Mark a texture as recently used (moves to end of LRU queue)
    #[allow(dead_code)]
    fn touch_texture(&mut self, id: &AssetId) {
        if let Some(pos) = self.lru_queue.iter().position(|x| x == id) {
            self.lru_queue.remove(pos);
            self.lru_queue.push_back(id.clone());
        }
    }

    /// Update residency based on camera position
    pub fn update_residency(&mut self, camera_pos: Vec3) {
        self.camera_position = camera_pos;
        // Future: Implement distance-based pre-loading/eviction
    }

    /// Get current memory usage statistics
    pub fn get_stats(&self) -> TextureStreamingStats {
        let loaded_count = self
            .assets
            .values()
            .filter(|s| matches!(s, AssetState::Resident(_)))
            .count();
        let pending_count = self.load_queue.len()
            + self
                .assets
                .values()
                .filter(|s| matches!(s, AssetState::Loading))
                .count();

        TextureStreamingStats {
            loaded_count,
            pending_count,
            memory_used_bytes: self.current_memory_bytes,
            memory_budget_bytes: self.max_memory_bytes,
            memory_used_percent: (self.current_memory_bytes as f32 / self.max_memory_bytes as f32)
                * 100.0,
        }
    }

    /// Check if a texture is resident
    pub fn is_resident(&self, id: &AssetId) -> bool {
        matches!(self.assets.get(id), Some(AssetState::Resident(_)))
    }

    /// Clear all loaded textures
    pub fn clear(&mut self) {
        self.assets.clear();
        self.lru_queue.clear();
        self.load_queue.clear();
        self.pending_ids.clear();
        self.current_memory_bytes = 0;
    }
}

/// Texture streaming statistics
#[derive(Debug, Clone)]
pub struct TextureStreamingStats {
    pub loaded_count: usize,
    pub pending_count: usize,
    pub memory_used_bytes: usize,
    pub memory_budget_bytes: usize,
    pub memory_used_percent: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: check if an ID is in the pending queue (load_queue via pending_ids set)
    fn is_pending(mgr: &TextureStreamingManager, id: &str) -> bool {
        mgr.pending_ids.contains(id)
    }

    /// Test-only helper to inject a fake Resident entry without requiring GPU.
    /// Uses pollster + wgpu's built-in test adapter to create a minimal device.
    fn inject_resident(mgr: &mut TextureStreamingManager, id: &str, memory_bytes: usize) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        // If no adapter available (e.g., headless CI), skip by silently returning.
        let Ok(adapter) = adapter else { return };
        let (device, _queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test_dummy"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let tex = Texture {
            texture,
            view,
            sampler,
        };
        let handle = TextureHandle {
            id: id.into(),
            texture: Arc::new(tex),
            width: 1,
            height: 1,
            mip_levels: 1,
            memory_bytes,
        };
        mgr.assets.insert(id.into(), AssetState::Resident(handle));
        mgr.lru_queue.push_back(id.into());
        mgr.current_memory_bytes += memory_bytes;
    }

    // ——— Core state machine tests (no GPU required) ———

    #[test]
    fn request_queues_without_premature_loading_state() {
        // Regression test for the state machine deadlock:
        // request_texture must NOT insert Loading into self.assets, only into the queue.
        // process_next_load is the sole owner of the Loading→spawn transition.
        let mut mgr = TextureStreamingManager::new(64);

        // First request should return None and add to pending queue
        let result = mgr.request_texture("tex_a".into(), 10, 5.0);
        assert!(
            result.is_none(),
            "new texture should not be immediately resident"
        );

        // The texture should be pending in the queue, but NOT in the assets map
        assert!(is_pending(&mgr, "tex_a"), "should be in pending set");
        assert!(
            !mgr.assets.contains_key("tex_a"),
            "should NOT be in assets map yet — this was the deadlock bug"
        );

        // Verify the load queue has one entry
        assert_eq!(mgr.load_queue.len(), 1, "queue should have one request");
    }

    #[test]
    fn duplicate_request_does_not_double_queue() {
        let mut mgr = TextureStreamingManager::new(64);

        // Request the same texture twice
        mgr.request_texture("tex_dup".into(), 10, 5.0);
        mgr.request_texture("tex_dup".into(), 20, 3.0);

        // Should only have one entry in load queue — the second call sees pending_ids
        assert_eq!(
            mgr.load_queue.len(),
            1,
            "duplicate request should not create second queue entry"
        );
        assert!(is_pending(&mgr, "tex_dup"));
    }

    #[test]
    fn failed_texture_is_not_requeued() {
        let mut mgr = TextureStreamingManager::new(64);

        // Manually insert a Failed texture
        mgr.assets
            .insert("tex_fail".into(), AssetState::Failed("disk error".into()));

        // Requesting a failed texture should return None and NOT add to queue
        let result = mgr.request_texture("tex_fail".into(), 10, 5.0);
        assert!(result.is_none());
        assert_eq!(
            mgr.load_queue.len(),
            0,
            "failed texture should not be re-queued"
        );
    }

    #[test]
    fn loading_texture_blocks_requeue_but_not_via_assets_map() {
        // After process_next_load runs and sets Loading in assets map,
        // a subsequent request_texture call should return None and NOT add a duplicate.
        let mut mgr = TextureStreamingManager::new(64);

        // Simulate process_next_load having picked up "tex_x" and set it as Loading
        mgr.assets.insert("tex_x".into(), AssetState::Loading);

        // Now request the same texture
        let result = mgr.request_texture("tex_x".into(), 10, 1.0);
        assert!(result.is_none());
        assert!(
            !is_pending(&mgr, "tex_x"),
            "should not be added to pending queue"
        );
        assert_eq!(mgr.load_queue.len(), 0, "should not be added to load queue");
    }

    #[test]
    fn stats_counts_pending_correctly() {
        let mut mgr = TextureStreamingManager::new(64);

        // Add some pending requests
        mgr.request_texture("a".into(), 1, 10.0);
        mgr.request_texture("b".into(), 2, 5.0);

        let stats = mgr.get_stats();
        assert_eq!(stats.loaded_count, 0);
        // pending_count = queue len + loading count in assets
        assert_eq!(stats.pending_count, 2);
    }

    #[test]
    fn clear_resets_all_state() {
        let mut mgr = TextureStreamingManager::new(64);

        mgr.request_texture("x".into(), 1, 1.0);
        // Also add a Loading entry directly
        mgr.assets.insert("y".into(), AssetState::Loading);

        mgr.clear();

        assert!(mgr.assets.is_empty());
        assert!(mgr.lru_queue.is_empty());
        assert!(mgr.load_queue.is_empty());
        assert!(mgr.pending_ids.is_empty());
        assert_eq!(mgr.current_memory_bytes, 0);
    }

    // ——— Tests requiring GPU adapter (resident textures) ———

    #[test]
    fn resident_texture_is_returned_and_lru_updated() {
        let mut mgr = TextureStreamingManager::new(64);

        inject_resident(&mut mgr, "tex_resident", 4);

        // If inject_resident silently skipped (no adapter), skip this test
        if !mgr.is_resident(&"tex_resident".to_string()) {
            eprintln!("Skipping: no GPU adapter available");
            return;
        }

        // Requesting a resident texture should return the handle
        let result = mgr.request_texture("tex_resident".into(), 5, 1.0);
        assert!(result.is_some(), "resident texture should be returned");
        assert_eq!(result.unwrap().id, "tex_resident");

        // LRU should have it at the back
        assert_eq!(mgr.lru_queue.back().unwrap(), "tex_resident");
    }

    #[test]
    fn evict_lru_removes_oldest() {
        let mut mgr = TextureStreamingManager::new(64);

        inject_resident(&mut mgr, "oldest", 100);
        inject_resident(&mut mgr, "newest", 100);

        // If no adapter, skip
        if !mgr.is_resident(&"oldest".to_string()) {
            eprintln!("Skipping: no GPU adapter available");
            return;
        }

        assert_eq!(mgr.current_memory_bytes, 200);
        assert!(mgr.evict_lru());
        assert!(
            !mgr.is_resident(&"oldest".to_string()),
            "oldest should be evicted"
        );
        assert!(
            mgr.is_resident(&"newest".to_string()),
            "newest should remain"
        );
        assert_eq!(mgr.current_memory_bytes, 100);
    }

    #[test]
    fn stats_counts_loaded_with_resident() {
        let mut mgr = TextureStreamingManager::new(64);

        // Add some pending requests
        mgr.request_texture("a".into(), 1, 10.0);

        inject_resident(&mut mgr, "c", 4);

        if !mgr.is_resident(&String::from("c")) {
            eprintln!("Skipping: no GPU adapter available");
            return;
        }

        let stats = mgr.get_stats();
        assert_eq!(stats.loaded_count, 1);
        assert!(stats.pending_count >= 1); // at least "a" is pending
    }
}
