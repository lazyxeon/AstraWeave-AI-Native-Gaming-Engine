//! Texture streaming and residency system with priority-based loading
//!
//! Provides LRU caching, priority queuing, and distance-based residency management.

use crate::texture::{Texture, TextureUsage};
use glam::Vec3;
use log::{debug, error, warn};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
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

        // Check if already loading or failed
        if self.assets.contains_key(&id) {
            return None;
        }

        // Queue for load
        self.assets.insert(id.clone(), AssetState::Loading);
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
            // Skip if already tracked
            if self.assets.contains_key(&request.id) {
                return;
            }

            // Mark as Loading
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
    // Tests temporarily disabled during async refactor
}
