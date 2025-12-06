//! Background chunk loader with priority-based streaming
//!
//! This module implements asynchronous terrain chunk loading with:
//! - Priority-based queue (distance from camera)
//! - Memory budget enforcement
//! - Prefetch strategy (load ahead in camera direction)
//! - Background task pool (tokio)

use crate::{ChunkId, TerrainChunk, WorldGenerator};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Priority for chunk loading (higher = more urgent)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkPriority {
    /// Distance from camera (lower = higher priority)
    pub distance: f32,
    /// Whether chunk is in camera frustum
    pub in_frustum: bool,
    /// Request timestamp (for tie-breaking)
    pub timestamp: u64,
}

impl Eq for ChunkPriority {}

impl Ord for ChunkPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // Frustum chunks have highest priority
        if self.in_frustum != other.in_frustum {
            return self.in_frustum.cmp(&other.in_frustum);
        }

        // Then sort by distance (lower distance = higher priority, so reverse)
        match other.distance.partial_cmp(&self.distance) {
            Some(ord) if ord != Ordering::Equal => ord,
            _ => self.timestamp.cmp(&other.timestamp),
        }
    }
}

impl PartialOrd for ChunkPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A chunk load request with priority
#[derive(Debug, Clone)]
struct ChunkLoadRequest {
    chunk_id: ChunkId,
    priority: ChunkPriority,
}

impl Eq for ChunkLoadRequest {}

impl PartialEq for ChunkLoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.chunk_id == other.chunk_id
    }
}

impl Ord for ChunkLoadRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for ChunkLoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Configuration for background chunk loader
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum number of chunks to keep loaded
    pub max_loaded_chunks: usize,

    /// View distance in chunks
    pub view_distance: u32,

    /// Prefetch distance ahead in camera direction (chunks)
    pub prefetch_distance: u32,

    /// Maximum concurrent load tasks (increased from 4 to 8 for better parallelism)
    pub max_concurrent_loads: usize,

    /// Chunk size in world units
    pub chunk_size: f32,

    /// Frame time threshold for adaptive throttling (ms)
    /// When frame time exceeds this, reduce concurrent loads
    pub adaptive_throttle_threshold_ms: f32,

    /// Reduced concurrent loads when throttling
    pub throttled_concurrent_loads: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_loaded_chunks: 256,
            view_distance: 8,
            prefetch_distance: 4,
            max_concurrent_loads: 8, // Increased from 4
            chunk_size: 256.0,
            adaptive_throttle_threshold_ms: 10.0, // Throttle if frame >10ms
            throttled_concurrent_loads: 2,        // Reduce to 2 when throttling
        }
    }
}

/// Background chunk loader status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderStatus {
    Idle,
    Loading,
    Unloading,
}

/// Statistics for streaming performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamingStats {
    pub loaded_chunk_count: usize,
    pub pending_load_count: usize,
    pub active_load_count: usize,
    pub memory_usage_mb: f32,
    pub chunks_loaded_this_frame: usize,
    pub chunks_unloaded_this_frame: usize,
}

/// Background chunk loader with priority queue
pub struct BackgroundChunkLoader {
    config: StreamingConfig,

    /// World generator (shared, behind RwLock for async access)
    world_gen: Arc<RwLock<WorldGenerator>>,

    /// Loaded chunks (chunk_id -> chunk data)
    loaded_chunks: Arc<RwLock<HashMap<ChunkId, TerrainChunk>>>,

    /// Priority queue for pending loads
    load_queue: Arc<RwLock<BinaryHeap<ChunkLoadRequest>>>,

    /// Set of chunks currently being loaded
    loading: Arc<RwLock<HashSet<ChunkId>>>,

    /// Channel for completed chunks
    completed_tx: mpsc::UnboundedSender<(ChunkId, TerrainChunk)>,
    completed_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<(ChunkId, TerrainChunk)>>>,

    /// Monotonic timestamp for request ordering
    next_timestamp: Arc<RwLock<u64>>,

    /// Current camera position (for priority calculation)
    camera_position: Arc<RwLock<Vec3>>,

    /// Previous camera position (for velocity calculation - Phase 3)
    prev_camera_position: Arc<RwLock<Vec3>>,

    /// Current camera direction (for prefetch)
    camera_direction: Arc<RwLock<Vec3>>,

    /// Camera velocity (for prefetch prediction - Phase 3)
    camera_velocity: Arc<RwLock<Vec3>>,

    /// Last frame time (for adaptive throttling - Phase 2 optimization)
    last_frame_time_ms: Arc<RwLock<f32>>,

    /// Smoothed frame time (exponential moving average for hysteresis)
    smoothed_frame_time_ms: Arc<RwLock<f32>>,
}

impl BackgroundChunkLoader {
    /// Create a new background chunk loader
    pub fn new(config: StreamingConfig, world_gen: Arc<RwLock<WorldGenerator>>) -> Self {
        let (completed_tx, completed_rx) = mpsc::unbounded_channel();

        Self {
            config,
            world_gen,
            loaded_chunks: Arc::new(RwLock::new(HashMap::new())),
            load_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            loading: Arc::new(RwLock::new(HashSet::new())),
            completed_tx,
            completed_rx: Arc::new(tokio::sync::Mutex::new(completed_rx)),
            next_timestamp: Arc::new(RwLock::new(0)),
            camera_position: Arc::new(RwLock::new(Vec3::ZERO)),
            prev_camera_position: Arc::new(RwLock::new(Vec3::ZERO)), // Phase 3: velocity tracking
            camera_direction: Arc::new(RwLock::new(Vec3::X)),
            camera_velocity: Arc::new(RwLock::new(Vec3::ZERO)), // Phase 3: prefetch prediction
            last_frame_time_ms: Arc::new(RwLock::new(0.0)),     // Phase 2: adaptive throttling
            smoothed_frame_time_ms: Arc::new(RwLock::new(0.0)), // Phase 2: hysteresis
        }
    }

    /// Set last frame time for adaptive throttling (Phase 2 optimization)
    pub async fn set_frame_time(&self, frame_time_ms: f32) {
        *self.last_frame_time_ms.write().await = frame_time_ms;

        // Update smoothed frame time (exponential moving average with alpha=0.1)
        let mut smoothed = self.smoothed_frame_time_ms.write().await;
        *smoothed = 0.9 * *smoothed + 0.1 * frame_time_ms;
    }

    /// Get current concurrent load limit (adaptive based on frame time)
    async fn get_adaptive_concurrent_limit(&self) -> usize {
        // DISABLED: Adaptive throttling made performance worse
        // Always use max concurrency for now
        self.config.max_concurrent_loads

        /* ORIGINAL ADAPTIVE LOGIC (disabled):
        let smoothed_time = *self.smoothed_frame_time_ms.read().await;
        let loaded_count = self.loaded_chunks.read().await.len();

        if smoothed_time > self.config.adaptive_throttle_threshold_ms && loaded_count > 50 {
            self.config.throttled_concurrent_loads  // Reduce to 2
        } else {
            self.config.max_concurrent_loads  // Use full 8
        }
        */
    }

    /// Update camera position and direction (Phase 3: calculates velocity for prefetch)
    pub async fn update_camera(&self, position: Vec3, direction: Vec3) {
        // Calculate velocity from position change
        let prev_pos = *self.camera_position.read().await;
        let velocity = position - prev_pos;

        // Update positions
        *self.prev_camera_position.write().await = prev_pos;
        *self.camera_position.write().await = position;
        *self.camera_direction.write().await = direction.normalize();
        *self.camera_velocity.write().await = velocity;
    }

    /// Get predicted camera position (Phase 3: prefetch prediction)
    /// Predicts where camera will be in `seconds_ahead` based on current velocity
    pub async fn get_predicted_position(&self, seconds_ahead: f32) -> Vec3 {
        let current_pos = *self.camera_position.read().await;
        let mut velocity = *self.camera_velocity.read().await;

        // If velocity is near zero (cold start), use camera direction * assumed speed
        // This allows prefetch to work even before velocity is calculated
        if velocity.length() < 0.1 {
            let direction = *self.camera_direction.read().await;
            velocity = direction * 10.0; // Assume 10 m/s forward movement
        }

        // Clamp velocity to detect teleports (if >100 m/s, likely teleported, don't predict)
        let velocity_magnitude = velocity.length();
        const MAX_REALISTIC_VELOCITY: f32 = 100.0; // 100 m/s = 360 km/h

        if velocity_magnitude > MAX_REALISTIC_VELOCITY {
            // Teleport detected - don't prefetch, just return current position
            current_pos
        } else {
            // Normal movement - predict ahead
            current_pos + velocity * seconds_ahead
        }
    }

    /// Request chunks to be loaded around the camera (Phase 3: includes prefetch)
    pub async fn request_chunks_around_camera(&self) {
        let camera_pos = *self.camera_position.read().await;
        let camera_dir = *self.camera_direction.read().await;

        // Phase 3: Predict position 2 seconds ahead for prefetching
        let predicted_pos = self.get_predicted_position(2.0).await;

        // Get chunks in view distance around CURRENT position
        let _center_chunk = ChunkId::from_world_pos(camera_pos, self.config.chunk_size);
        let view_chunks = ChunkId::get_chunks_in_radius(
            camera_pos,
            self.config.view_distance,
            self.config.chunk_size,
        );

        // Phase 3: Get chunks around PREDICTED position (velocity-based prefetch)
        let predicted_chunks = ChunkId::get_chunks_in_radius(
            predicted_pos,
            self.config.prefetch_distance,
            self.config.chunk_size,
        );

        // Original prefetch (direction-based, for compatibility)
        let prefetch_offset =
            camera_dir * (self.config.prefetch_distance as f32 * self.config.chunk_size);
        let prefetch_center = camera_pos + prefetch_offset;
        let direction_prefetch_chunks = ChunkId::get_chunks_in_radius(
            prefetch_center,
            self.config.prefetch_distance,
            self.config.chunk_size,
        );

        // Combine all chunk sets (view + predicted + direction prefetch)
        let mut all_chunks: HashSet<ChunkId> = view_chunks.into_iter().collect();
        all_chunks.extend(predicted_chunks); // Phase 3: velocity-based prefetch
        all_chunks.extend(direction_prefetch_chunks); // Original direction-based

        // Filter out already loaded and loading chunks
        let loaded = self.loaded_chunks.read().await;
        let loading = self.loading.read().await;

        let to_load: Vec<ChunkId> = all_chunks
            .into_iter()
            .filter(|id| !loaded.contains_key(id) && !loading.contains(id))
            .collect();

        drop(loaded);
        drop(loading);

        // Queue new load requests with priority
        let mut queue = self.load_queue.write().await;
        let mut timestamp = self.next_timestamp.write().await;

        for chunk_id in to_load {
            let chunk_center = chunk_id.to_center_pos(self.config.chunk_size);
            let distance = (chunk_center - camera_pos).length();

            // Simple frustum check (approximate with forward dot product)
            let to_chunk = (chunk_center - camera_pos).normalize();
            let in_frustum = to_chunk.dot(camera_dir) > -0.3; // ~107 degree FOV

            let priority = ChunkPriority {
                distance,
                in_frustum,
                timestamp: *timestamp,
            };

            *timestamp += 1;

            queue.push(ChunkLoadRequest { chunk_id, priority });
        }
    }

    /// Process load queue and start background tasks
    pub async fn process_load_queue(&self) {
        let mut queue = self.load_queue.write().await;
        let mut loading = self.loading.write().await;

        // Adaptive throttling (Phase 2 optimization)
        let max_concurrent = self.get_adaptive_concurrent_limit().await;

        // Start up to max_concurrent_loads tasks (adaptive based on frame time)
        let available_slots = max_concurrent.saturating_sub(loading.len());

        for _ in 0..available_slots {
            if let Some(request) = queue.pop() {
                let chunk_id = request.chunk_id;
                loading.insert(chunk_id);

                // Spawn background load task
                let world_gen = Arc::clone(&self.world_gen);
                let completed_tx = self.completed_tx.clone();
                let loading_set = Arc::clone(&self.loading);

                tokio::spawn(async move {
                    // Generate chunk (lock-free - uses read lock only)
                    let chunk_result = {
                        let gen = world_gen.read().await; // Read lock (shared access)
                        gen.generate_chunk(chunk_id) // Pure function, no mutation
                    };

                    // Send completed chunk (only if successful)
                    if let Ok(chunk) = chunk_result {
                        let _ = completed_tx.send((chunk_id, chunk));
                    }

                    // Remove from loading set
                    loading_set.write().await.remove(&chunk_id);
                });
            } else {
                break;
            }
        }
    }

    /// Collect completed chunks
    pub async fn collect_completed_chunks(&self) -> usize {
        let mut rx = self.completed_rx.lock().await;
        let mut loaded = self.loaded_chunks.write().await;
        let mut count = 0;

        // Drain all completed chunks
        while let Ok((chunk_id, chunk)) = rx.try_recv() {
            loaded.insert(chunk_id, chunk);
            count += 1;
        }

        count
    }

    /// Get streaming statistics
    pub async fn get_stats(&self) -> StreamingStats {
        let loaded = self.loaded_chunks.read().await;
        let queue = self.load_queue.read().await;
        let loading = self.loading.read().await;

        // Rough memory estimate (chunk data + overhead)
        let bytes_per_chunk = 128 * 128 * 4; // Heightmap (f32)
        let memory_usage_mb = (loaded.len() * bytes_per_chunk) as f32 / (1024.0 * 1024.0);

        StreamingStats {
            loaded_chunk_count: loaded.len(),
            pending_load_count: queue.len(),
            active_load_count: loading.len(),
            memory_usage_mb,
            chunks_loaded_this_frame: 0,   // Updated by caller
            chunks_unloaded_this_frame: 0, // Updated by caller
        }
    }

    /// Get a loaded chunk by ID
    pub async fn get_chunk(&self, chunk_id: ChunkId) -> Option<TerrainChunk> {
        self.loaded_chunks.read().await.get(&chunk_id).cloned()
    }

    /// Check if a chunk is loaded
    pub async fn is_loaded(&self, chunk_id: ChunkId) -> bool {
        self.loaded_chunks.read().await.contains_key(&chunk_id)
    }

    /// Get all loaded chunk IDs
    pub async fn get_loaded_chunk_ids(&self) -> Vec<ChunkId> {
        self.loaded_chunks.read().await.keys().copied().collect()
    }

    /// Check if a chunk is currently being loaded
    pub async fn is_loading(&self, chunk_id: ChunkId) -> bool {
        self.loading.read().await.contains(&chunk_id)
    }

    /// Unload distant chunks with explicit camera position
    pub async fn unload_distant_chunks(&self, camera_pos: Vec3) -> usize {
        let mut loaded = self.loaded_chunks.write().await;

        // If under budget, nothing to unload
        if loaded.len() <= self.config.max_loaded_chunks {
            return 0;
        }

        // Calculate distances and sort
        let mut chunks_with_distance: Vec<(ChunkId, f32)> = loaded
            .keys()
            .map(|&id| {
                let center = id.to_center_pos(self.config.chunk_size);
                let distance = (center - camera_pos).length();
                (id, distance)
            })
            .collect();

        chunks_with_distance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        // Unload furthest chunks until under budget
        let to_unload = loaded.len().saturating_sub(self.config.max_loaded_chunks);
        let mut unloaded = 0;

        for (chunk_id, _) in chunks_with_distance.iter().take(to_unload) {
            loaded.remove(chunk_id);
            unloaded += 1;
        }

        unloaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_priority_ordering() {
        let p1 = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 1,
        };

        let p2 = ChunkPriority {
            distance: 5.0,
            in_frustum: true,
            timestamp: 2,
        };

        // Closer chunk should have higher priority
        assert!(p2 > p1);
    }

    #[test]
    fn test_frustum_priority_higher() {
        let p1 = ChunkPriority {
            distance: 10.0,
            in_frustum: false,
            timestamp: 1,
        };

        let p2 = ChunkPriority {
            distance: 15.0,
            in_frustum: true,
            timestamp: 2,
        };

        // In-frustum chunk should have higher priority even if further
        assert!(p2 > p1);
    }

    #[tokio::test]
    async fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.max_loaded_chunks, 256);
        assert_eq!(config.view_distance, 8);
        assert_eq!(config.prefetch_distance, 4);
        assert_eq!(config.max_concurrent_loads, 8);
        assert_eq!(config.chunk_size, 256.0);
        assert_eq!(config.adaptive_throttle_threshold_ms, 10.0);
        assert_eq!(config.throttled_concurrent_loads, 2);
    }

    #[test]
    fn test_chunk_priority_equal_distance_timestamp() {
        let p1 = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 1,
        };

        let p2 = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 2,
        };

        // Same distance and frustum, later timestamp has higher priority
        assert!(p2 > p1);
    }

    #[test]
    fn test_chunk_priority_eq_trait() {
        let p1 = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 1,
        };

        let p2 = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 1,
        };

        assert_eq!(p1, p2);
    }

    #[test]
    fn test_chunk_priority_partial_ord() {
        let p1 = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 1,
        };

        let p2 = ChunkPriority {
            distance: 5.0,
            in_frustum: true,
            timestamp: 2,
        };

        assert!(p1.partial_cmp(&p2).is_some());
    }

    #[test]
    fn test_chunk_load_request_eq() {
        let r1 = ChunkLoadRequest {
            chunk_id: ChunkId::new(0, 0),
            priority: ChunkPriority {
                distance: 10.0,
                in_frustum: true,
                timestamp: 1,
            },
        };

        let r2 = ChunkLoadRequest {
            chunk_id: ChunkId::new(0, 0),
            priority: ChunkPriority {
                distance: 5.0, // Different priority
                in_frustum: false,
                timestamp: 2,
            },
        };

        // Same chunk ID means equal, regardless of priority
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_chunk_load_request_ord() {
        let r1 = ChunkLoadRequest {
            chunk_id: ChunkId::new(0, 0),
            priority: ChunkPriority {
                distance: 10.0,
                in_frustum: true,
                timestamp: 1,
            },
        };

        let r2 = ChunkLoadRequest {
            chunk_id: ChunkId::new(1, 1),
            priority: ChunkPriority {
                distance: 5.0,
                in_frustum: true,
                timestamp: 2,
            },
        };

        // r2 should be greater (higher priority) due to lower distance
        assert!(r2 > r1);
    }

    #[test]
    fn test_loader_status_eq() {
        assert_eq!(LoaderStatus::Idle, LoaderStatus::Idle);
        assert_eq!(LoaderStatus::Loading, LoaderStatus::Loading);
        assert_eq!(LoaderStatus::Unloading, LoaderStatus::Unloading);
        assert_ne!(LoaderStatus::Idle, LoaderStatus::Loading);
    }

    #[test]
    fn test_streaming_stats_default() {
        let stats = StreamingStats::default();
        assert_eq!(stats.loaded_chunk_count, 0);
        assert_eq!(stats.pending_load_count, 0);
        assert_eq!(stats.active_load_count, 0);
        assert_eq!(stats.memory_usage_mb, 0.0);
        assert_eq!(stats.chunks_loaded_this_frame, 0);
        assert_eq!(stats.chunks_unloaded_this_frame, 0);
    }

    #[test]
    fn test_streaming_stats_serialization() {
        let stats = StreamingStats {
            loaded_chunk_count: 10,
            pending_load_count: 5,
            active_load_count: 2,
            memory_usage_mb: 64.5,
            chunks_loaded_this_frame: 3,
            chunks_unloaded_this_frame: 1,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: StreamingStats = serde_json::from_str(&json).unwrap();
        
        assert_eq!(stats.loaded_chunk_count, deserialized.loaded_chunk_count);
        assert_eq!(stats.pending_load_count, deserialized.pending_load_count);
        assert_eq!(stats.active_load_count, deserialized.active_load_count);
        assert!((stats.memory_usage_mb - deserialized.memory_usage_mb).abs() < 0.001);
        assert_eq!(stats.chunks_loaded_this_frame, deserialized.chunks_loaded_this_frame);
        assert_eq!(stats.chunks_unloaded_this_frame, deserialized.chunks_unloaded_this_frame);
    }

    #[test]
    fn test_chunk_priority_cmp_nan_distance() {
        // Test edge case with NaN distance (should fallback to timestamp)
        let p1 = ChunkPriority {
            distance: f32::NAN,
            in_frustum: true,
            timestamp: 1,
        };

        let p2 = ChunkPriority {
            distance: f32::NAN,
            in_frustum: true,
            timestamp: 2,
        };

        // Should still compare by timestamp when distances are NaN
        assert!(p2 > p1);
    }

    #[test]
    fn test_streaming_config_custom() {
        let config = StreamingConfig {
            max_loaded_chunks: 512,
            view_distance: 16,
            prefetch_distance: 8,
            max_concurrent_loads: 16,
            chunk_size: 128.0,
            adaptive_throttle_threshold_ms: 5.0,
            throttled_concurrent_loads: 4,
        };

        assert_eq!(config.max_loaded_chunks, 512);
        assert_eq!(config.view_distance, 16);
        assert_eq!(config.prefetch_distance, 8);
        assert_eq!(config.max_concurrent_loads, 16);
        assert_eq!(config.chunk_size, 128.0);
        assert_eq!(config.adaptive_throttle_threshold_ms, 5.0);
        assert_eq!(config.throttled_concurrent_loads, 4);
    }

    #[test]
    fn test_chunk_load_request_partial_cmp() {
        let r1 = ChunkLoadRequest {
            chunk_id: ChunkId::new(0, 0),
            priority: ChunkPriority {
                distance: 10.0,
                in_frustum: true,
                timestamp: 1,
            },
        };

        let r2 = ChunkLoadRequest {
            chunk_id: ChunkId::new(1, 1),
            priority: ChunkPriority {
                distance: 5.0,
                in_frustum: true,
                timestamp: 2,
            },
        };

        assert!(r1.partial_cmp(&r2).is_some());
        assert_eq!(r1.partial_cmp(&r2), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_chunk_priority_distance_only_comparison() {
        // Both in frustum, different distances
        let near = ChunkPriority {
            distance: 5.0,
            in_frustum: true,
            timestamp: 1,
        };

        let far = ChunkPriority {
            distance: 20.0,
            in_frustum: true,
            timestamp: 1,
        };

        // Near should have higher priority (lower distance)
        assert!(near > far);
    }

    #[test]
    fn test_chunk_priority_frustum_beats_distance() {
        // Test that frustum flag always beats distance
        let near_out_of_frustum = ChunkPriority {
            distance: 1.0,
            in_frustum: false,
            timestamp: 1,
        };

        let far_in_frustum = ChunkPriority {
            distance: 100.0,
            in_frustum: true,
            timestamp: 1,
        };

        // In-frustum should win even with larger distance
        assert!(far_in_frustum > near_out_of_frustum);
    }

    #[test]
    fn test_chunk_priority_timestamp_tiebreaker() {
        // Same distance and frustum, different timestamps
        let earlier = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 100,
        };

        let later = ChunkPriority {
            distance: 10.0,
            in_frustum: true,
            timestamp: 200,
        };

        // Later timestamp should have higher priority
        assert!(later > earlier);
    }

    #[test]
    fn test_chunk_load_request_different_chunks_same_priority() {
        let r1 = ChunkLoadRequest {
            chunk_id: ChunkId::new(0, 0),
            priority: ChunkPriority {
                distance: 10.0,
                in_frustum: true,
                timestamp: 1,
            },
        };

        let r2 = ChunkLoadRequest {
            chunk_id: ChunkId::new(5, 5),
            priority: ChunkPriority {
                distance: 10.0,
                in_frustum: true,
                timestamp: 1,
            },
        };

        // Different chunks with same priority should not be equal
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_loader_status_all_variants() {
        let statuses = [LoaderStatus::Idle, LoaderStatus::Loading, LoaderStatus::Unloading];
        
        // All variants should be distinguishable
        for (i, s1) in statuses.iter().enumerate() {
            for (j, s2) in statuses.iter().enumerate() {
                if i == j {
                    assert_eq!(s1, s2);
                } else {
                    assert_ne!(s1, s2);
                }
            }
        }
    }

    #[test]
    fn test_streaming_stats_clone() {
        let stats = StreamingStats {
            loaded_chunk_count: 100,
            pending_load_count: 50,
            active_load_count: 8,
            memory_usage_mb: 128.0,
            chunks_loaded_this_frame: 4,
            chunks_unloaded_this_frame: 2,
        };

        let cloned = stats.clone();
        assert_eq!(stats.loaded_chunk_count, cloned.loaded_chunk_count);
        assert_eq!(stats.pending_load_count, cloned.pending_load_count);
        assert_eq!(stats.active_load_count, cloned.active_load_count);
        assert_eq!(stats.memory_usage_mb, cloned.memory_usage_mb);
    }

    #[test]
    fn test_streaming_config_clone() {
        let config = StreamingConfig {
            max_loaded_chunks: 1024,
            view_distance: 12,
            prefetch_distance: 6,
            max_concurrent_loads: 4,
            chunk_size: 64.0,
            adaptive_throttle_threshold_ms: 8.0,
            throttled_concurrent_loads: 1,
        };

        let cloned = config.clone();
        assert_eq!(config.max_loaded_chunks, cloned.max_loaded_chunks);
        assert_eq!(config.view_distance, cloned.view_distance);
        assert_eq!(config.chunk_size, cloned.chunk_size);
    }

    // Async tests for BackgroundChunkLoader
    mod async_tests {
        use super::*;
        use crate::{WorldConfig, WorldGenerator};

        fn create_test_loader() -> BackgroundChunkLoader {
            let config = StreamingConfig::default();
            let world_config = WorldConfig::default();
            let world_gen = Arc::new(RwLock::new(WorldGenerator::new(world_config)));
            BackgroundChunkLoader::new(config, world_gen)
        }

        #[tokio::test]
        async fn test_background_loader_creation() {
            let loader = create_test_loader();
            
            // Verify initial state
            let stats = loader.get_stats().await;
            assert_eq!(stats.loaded_chunk_count, 0);
            assert_eq!(stats.pending_load_count, 0);
            assert_eq!(stats.active_load_count, 0);
        }

        #[tokio::test]
        async fn test_set_frame_time() {
            let loader = create_test_loader();
            
            // Set frame time
            loader.set_frame_time(16.67).await;
            
            // Set another frame time to test smoothing
            loader.set_frame_time(20.0).await;
            loader.set_frame_time(15.0).await;
            
            // Just verify no panic - internal state is private
        }

        #[tokio::test]
        async fn test_update_camera() {
            let loader = create_test_loader();
            
            // Update camera position and direction
            let pos = Vec3::new(100.0, 50.0, 100.0);
            let dir = Vec3::new(1.0, 0.0, 0.0);
            
            loader.update_camera(pos, dir).await;
            
            // Move camera to test velocity calculation
            let new_pos = Vec3::new(110.0, 50.0, 100.0);
            loader.update_camera(new_pos, dir).await;
        }

        #[tokio::test]
        async fn test_get_predicted_position_stationary() {
            let loader = create_test_loader();
            
            // Set initial camera position
            let pos = Vec3::new(0.0, 0.0, 0.0);
            let dir = Vec3::new(1.0, 0.0, 0.0);
            loader.update_camera(pos, dir).await;
            
            // Get predicted position (cold start - should use direction)
            let predicted = loader.get_predicted_position(1.0).await;
            
            // With zero velocity, should use direction * assumed speed (10.0)
            // So predicted = pos + direction * 10.0 * 1.0
            assert!(predicted.x > pos.x, "Predicted position should be ahead in camera direction");
        }

        #[tokio::test]
        async fn test_get_predicted_position_moving() {
            let loader = create_test_loader();
            
            // Set initial position
            let pos1 = Vec3::new(0.0, 0.0, 0.0);
            let dir = Vec3::new(1.0, 0.0, 0.0);
            loader.update_camera(pos1, dir).await;
            
            // Move camera (creates velocity)
            let pos2 = Vec3::new(10.0, 0.0, 0.0);
            loader.update_camera(pos2, dir).await;
            
            // Predict 1 second ahead
            let predicted = loader.get_predicted_position(1.0).await;
            
            // Should be ahead of current position
            assert!(predicted.x > pos2.x);
        }

        #[tokio::test]
        async fn test_get_predicted_position_teleport_detection() {
            let loader = create_test_loader();
            
            // Set initial position
            let pos1 = Vec3::new(0.0, 0.0, 0.0);
            let dir = Vec3::new(1.0, 0.0, 0.0);
            loader.update_camera(pos1, dir).await;
            
            // Teleport (>100 m/s threshold)
            let pos2 = Vec3::new(1000.0, 0.0, 0.0);
            loader.update_camera(pos2, dir).await;
            
            // Should return current position (no prediction on teleport)
            let predicted = loader.get_predicted_position(1.0).await;
            assert_eq!(predicted, pos2);
        }

        #[tokio::test]
        async fn test_is_loaded_empty() {
            let loader = create_test_loader();
            
            // Nothing should be loaded initially
            let chunk_id = ChunkId::new(0, 0);
            assert!(!loader.is_loaded(chunk_id).await);
        }

        #[tokio::test]
        async fn test_is_loading_empty() {
            let loader = create_test_loader();
            
            // Nothing should be loading initially
            let chunk_id = ChunkId::new(0, 0);
            assert!(!loader.is_loading(chunk_id).await);
        }

        #[tokio::test]
        async fn test_get_chunk_empty() {
            let loader = create_test_loader();
            
            // Should return None for non-existent chunk
            let chunk_id = ChunkId::new(0, 0);
            assert!(loader.get_chunk(chunk_id).await.is_none());
        }

        #[tokio::test]
        async fn test_get_loaded_chunk_ids_empty() {
            let loader = create_test_loader();
            
            // Should return empty vec initially
            let ids = loader.get_loaded_chunk_ids().await;
            assert!(ids.is_empty());
        }

        #[tokio::test]
        async fn test_collect_completed_chunks_empty() {
            let loader = create_test_loader();
            
            // Should return 0 when no chunks completed
            let count = loader.collect_completed_chunks().await;
            assert_eq!(count, 0);
        }

        #[tokio::test]
        async fn test_unload_distant_chunks_under_budget() {
            let loader = create_test_loader();
            
            // With no chunks loaded, unload should do nothing
            let camera_pos = Vec3::ZERO;
            let unloaded = loader.unload_distant_chunks(camera_pos).await;
            assert_eq!(unloaded, 0);
        }

        #[tokio::test]
        async fn test_get_stats_initial() {
            let loader = create_test_loader();
            
            let stats = loader.get_stats().await;
            
            assert_eq!(stats.loaded_chunk_count, 0);
            assert_eq!(stats.pending_load_count, 0);
            assert_eq!(stats.active_load_count, 0);
            assert_eq!(stats.memory_usage_mb, 0.0);
        }

        #[tokio::test]
        async fn test_get_adaptive_concurrent_limit() {
            let loader = create_test_loader();
            
            // Should return max_concurrent_loads (adaptive throttling disabled)
            let limit = loader.get_adaptive_concurrent_limit().await;
            assert_eq!(limit, loader.config.max_concurrent_loads);
        }

        #[tokio::test]
        async fn test_request_chunks_around_camera() {
            let loader = create_test_loader();
            
            // Set camera position
            let pos = Vec3::new(0.0, 0.0, 0.0);
            let dir = Vec3::new(1.0, 0.0, 0.0);
            loader.update_camera(pos, dir).await;
            
            // Request chunks - this should populate the load queue
            loader.request_chunks_around_camera().await;
            
            // Verify queue has entries
            let stats = loader.get_stats().await;
            assert!(stats.pending_load_count > 0, "Should have pending chunks to load");
        }

        #[tokio::test]
        async fn test_process_load_queue_empty() {
            let loader = create_test_loader();
            
            // Process empty queue should not panic
            loader.process_load_queue().await;
            
            let stats = loader.get_stats().await;
            assert_eq!(stats.active_load_count, 0);
        }

        #[tokio::test]
        async fn test_full_loading_cycle() {
            let loader = create_test_loader();
            
            // Set camera position
            let pos = Vec3::new(0.0, 0.0, 0.0);
            let dir = Vec3::new(1.0, 0.0, 0.0);
            loader.update_camera(pos, dir).await;
            
            // Request chunks
            loader.request_chunks_around_camera().await;
            
            // Process load queue
            loader.process_load_queue().await;
            
            // Give spawned tasks time to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Collect completed chunks
            let completed = loader.collect_completed_chunks().await;
            
            // Should have loaded some chunks
            let stats = loader.get_stats().await;
            assert!(completed > 0 || stats.loaded_chunk_count > 0 || stats.active_load_count > 0,
                "Should have made progress loading chunks");
        }

        #[tokio::test]
        async fn test_frame_time_smoothing() {
            let loader = create_test_loader();
            
            // Set multiple frame times to test exponential moving average
            for frame_time in [16.67, 20.0, 15.0, 18.0, 12.0] {
                loader.set_frame_time(frame_time).await;
            }
            
            // The smoothed value should be somewhere between min and max
            // (Internal state is private, but we verify no panics)
        }

        #[tokio::test]
        async fn test_camera_velocity_calculation() {
            let loader = create_test_loader();
            
            // Set sequence of positions to test velocity tracking
            let positions = [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(3.0, 0.0, 0.0),
            ];
            
            let dir = Vec3::new(1.0, 0.0, 0.0);
            
            for pos in positions {
                loader.update_camera(pos, dir).await;
            }
            
            // Predicted position should account for velocity
            let predicted = loader.get_predicted_position(1.0).await;
            assert!(predicted.x > 3.0, "Should predict ahead based on velocity");
        }
    }
}
