//! Texture streaming and residency system with priority-based loading
//!
//! Provides LRU caching, priority queuing, and distance-based residency management.

use glam::Vec3;
use log::{debug, warn};
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;

/// Unique identifier for textures in the asset system
pub type AssetId = String;

/// GPU texture handle (placeholder - would be wgpu::Texture in real implementation)
#[derive(Debug, Clone)]
pub struct TextureHandle {
    pub id: AssetId,
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub memory_bytes: usize,
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
                other.distance.partial_cmp(&self.distance).unwrap_or(Ordering::Equal)
            }
            other => other,
        }
    }
}

/// Texture streaming manager with LRU eviction and priority-based loading
pub struct TextureStreamingManager {
    /// LRU cache of loaded textures
    loaded_textures: HashMap<AssetId, TextureHandle>,
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
}

impl TextureStreamingManager {
    /// Create a new texture streaming manager
    ///
    /// # Arguments
    /// * `max_memory_mb` - Maximum GPU memory budget in megabytes
    pub fn new(max_memory_mb: usize) -> Self {
        Self {
            loaded_textures: HashMap::new(),
            lru_queue: VecDeque::new(),
            load_queue: BinaryHeap::new(),
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory_bytes: 0,
            camera_position: Vec3::ZERO,
        }
    }

    /// Request a texture with given priority
    ///
    /// Returns immediately if texture is already loaded, otherwise queues for async loading.
    ///
    /// # Arguments
    /// * `id` - Asset ID of the texture
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
        // Check if already loaded
        if self.loaded_textures.contains_key(&id) {
            // Touch for LRU
            self.touch_texture(&id);
            return self.loaded_textures.get(&id).cloned();
        }

        // Queue for loading
        self.load_queue.push(LoadRequest {
            id,
            priority,
            distance,
        });

        None
    }

    /// Process the next load request from the priority queue
    ///
    /// This should be called periodically to process pending loads.
    /// In a real implementation, this would trigger async texture loading.
    ///
    /// # Arguments
    /// * `texture_data` - Callback to load texture data (width, height, bytes)
    pub fn process_next_load<F>(&mut self, mut texture_data: F) -> Option<AssetId>
    where
        F: FnMut(&AssetId) -> Option<(u32, u32, usize)>,
    {
        // Iterative loop to process requests without recursion
        while let Some(request) = self.load_queue.pop() {
            // Skip if already loaded (might have been loaded while in queue)
            if self.loaded_textures.contains_key(&request.id) {
                continue;
            }

            // Load texture data
            let (width, height, memory_bytes) = match texture_data(&request.id) {
                Some(data) => data,
                None => continue,
            };

            // Evict LRU textures until we have space
            while self.current_memory_bytes + memory_bytes > self.max_memory_bytes {
                if !self.evict_lru() {
                    // No more textures to evict and still not enough space
                    warn!(
                        "Cannot load texture {} ({}MB): not enough memory",
                        request.id,
                        memory_bytes / (1024 * 1024)
                    );
                    return None;
                }
            }

            // Create texture handle (placeholder)
            let handle = TextureHandle {
                id: request.id.clone(),
                width,
                height,
                mip_levels: calculate_mip_levels(width, height),
                memory_bytes,
            };

            // Add to cache
            self.loaded_textures.insert(request.id.clone(), handle);
            self.lru_queue.push_back(request.id.clone());
            self.current_memory_bytes += memory_bytes;

            debug!(
                "Loaded texture {} ({}x{}, {} mips, {:.2}MB) - {:.1}% memory used",
                request.id,
                width,
                height,
                calculate_mip_levels(width, height),
                memory_bytes as f32 / (1024.0 * 1024.0),
            (self.current_memory_bytes as f32 / self.max_memory_bytes as f32) * 100.0
        );

            return Some(request.id);
        }
        
        // Queue is empty or all requests were skipped
        None
    }

    /// Evict the least recently used texture
    ///
    /// # Returns
    /// * `true` if a texture was evicted
    /// * `false` if no textures to evict
    pub fn evict_lru(&mut self) -> bool {
        if let Some(id) = self.lru_queue.pop_front() {
            if let Some(handle) = self.loaded_textures.remove(&id) {
                self.current_memory_bytes = self.current_memory_bytes.saturating_sub(handle.memory_bytes);
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
    fn touch_texture(&mut self, id: &AssetId) {
        if let Some(pos) = self.lru_queue.iter().position(|x| x == id) {
            self.lru_queue.remove(pos);
            self.lru_queue.push_back(id.clone());
        }
    }

    /// Update residency based on camera position
    ///
    /// This can be used to adjust priorities or pre-load textures based on proximity.
    pub fn update_residency(&mut self, camera_pos: Vec3) {
        self.camera_position = camera_pos;
        // In a full implementation, this would:
        // 1. Calculate distances to all texture users
        // 2. Adjust load priorities
        // 3. Pre-emptively evict far textures
        // 4. Queue nearby textures for loading
    }

    /// Get current memory usage statistics
    pub fn get_stats(&self) -> TextureStreamingStats {
        TextureStreamingStats {
            loaded_count: self.loaded_textures.len(),
            pending_count: self.load_queue.len(),
            memory_used_bytes: self.current_memory_bytes,
            memory_budget_bytes: self.max_memory_bytes,
            memory_used_percent: (self.current_memory_bytes as f32 / self.max_memory_bytes as f32) * 100.0,
        }
    }

    /// Check if a texture is resident
    pub fn is_resident(&self, id: &AssetId) -> bool {
        self.loaded_textures.contains_key(id)
    }

    /// Clear all loaded textures
    pub fn clear(&mut self) {
        self.loaded_textures.clear();
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

/// Calculate number of mip levels for a texture
fn calculate_mip_levels(width: u32, height: u32) -> u32 {
    let max_dimension = width.max(height) as f32;
    (max_dimension.log2().floor() as u32 + 1).min(12) // Cap at 12 mip levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_request_and_load() {
        let mut mgr = TextureStreamingManager::new(100); // 100 MB budget

        // Request texture (not loaded)
        let handle = mgr.request_texture("tex1".to_string(), 10, 5.0);
        assert!(handle.is_none());

        // Process load
        let loaded = mgr.process_next_load(|id| {
            if id == "tex1" {
                Some((1024, 1024, 4 * 1024 * 1024)) // 4MB texture
            } else {
                None
            }
        });
        assert_eq!(loaded, Some("tex1".to_string()));

        // Request again (should be loaded)
        let handle = mgr.request_texture("tex1".to_string(), 10, 5.0);
        assert!(handle.is_some());

        let stats = mgr.get_stats();
        assert_eq!(stats.loaded_count, 1);
        assert_eq!(stats.memory_used_bytes, 4 * 1024 * 1024);
    }

    #[test]
    fn test_lru_eviction() {
        let mut mgr = TextureStreamingManager::new(10); // 10 MB budget

        // Load first texture (8MB)
        mgr.request_texture("tex1".to_string(), 10, 5.0);
        mgr.process_next_load(|_| Some((2048, 2048, 8 * 1024 * 1024)));

        // Load second texture (8MB) - should evict first
        mgr.request_texture("tex2".to_string(), 10, 3.0);
        mgr.process_next_load(|_| Some((2048, 2048, 8 * 1024 * 1024)));

        let stats = mgr.get_stats();
        assert_eq!(stats.loaded_count, 1);
        assert!(mgr.is_resident(&"tex2".to_string()));
        assert!(!mgr.is_resident(&"tex1".to_string()));
    }

    #[test]
    fn test_priority_ordering() {
        let mut mgr = TextureStreamingManager::new(100);

        // Queue multiple requests with different priorities
        mgr.request_texture("tex_low".to_string(), 5, 10.0);
        mgr.request_texture("tex_high".to_string(), 20, 10.0);
        mgr.request_texture("tex_med".to_string(), 10, 10.0);

        // High priority should load first
        let loaded = mgr.process_next_load(|_| Some((512, 512, 1024 * 1024)));
        assert_eq!(loaded, Some("tex_high".to_string()));

        // Medium priority next
        let loaded = mgr.process_next_load(|_| Some((512, 512, 1024 * 1024)));
        assert_eq!(loaded, Some("tex_med".to_string()));

        // Low priority last
        let loaded = mgr.process_next_load(|_| Some((512, 512, 1024 * 1024)));
        assert_eq!(loaded, Some("tex_low".to_string()));
    }

    #[test]
    fn test_distance_tie_breaking() {
        let mut mgr = TextureStreamingManager::new(100);

        // Queue requests with same priority but different distances
        mgr.request_texture("tex_far".to_string(), 10, 100.0);
        mgr.request_texture("tex_near".to_string(), 10, 5.0);

        // Closer texture should load first
        let loaded = mgr.process_next_load(|_| Some((512, 512, 1024 * 1024)));
        assert_eq!(loaded, Some("tex_near".to_string()));

        let loaded = mgr.process_next_load(|_| Some((512, 512, 1024 * 1024)));
        assert_eq!(loaded, Some("tex_far".to_string()));
    }
}
