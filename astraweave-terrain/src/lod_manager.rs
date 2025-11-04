//! LOD (Level of Detail) manager with hysteresis to prevent popping
//!
//! This module implements:
//! - 4 LOD levels (L0: full detail, L1: half, L2: quarter, L3: skybox)
//! - Hysteresis curve (10% margin to prevent flickering)
//! - Blend zones for smooth transitions
//! - Distance-based LOD selection

use crate::{meshing::ChunkMesh, ChunkId};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// LOD level for terrain chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LodLevel {
    /// L0: Full detail (all vertices)
    Full = 0,

    /// L1: Half resolution (every 2nd vertex)
    Half = 1,

    /// L2: Quarter resolution (every 4th vertex)
    Quarter = 2,

    /// L3: Skybox/impostor (minimal geometry)
    Skybox = 3,
}

impl LodLevel {
    /// Get vertex skip factor for this LOD level
    pub fn skip_factor(self) -> usize {
        match self {
            LodLevel::Full => 1,
            LodLevel::Half => 2,
            LodLevel::Quarter => 4,
            LodLevel::Skybox => 16,
        }
    }

    /// Get next lower detail level (or None if already lowest)
    pub fn lower(self) -> Option<LodLevel> {
        match self {
            LodLevel::Full => Some(LodLevel::Half),
            LodLevel::Half => Some(LodLevel::Quarter),
            LodLevel::Quarter => Some(LodLevel::Skybox),
            LodLevel::Skybox => None,
        }
    }

    /// Get next higher detail level (or None if already highest)
    pub fn higher(self) -> Option<LodLevel> {
        match self {
            LodLevel::Skybox => Some(LodLevel::Quarter),
            LodLevel::Quarter => Some(LodLevel::Half),
            LodLevel::Half => Some(LodLevel::Full),
            LodLevel::Full => None,
        }
    }
}

/// LOD transition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodConfig {
    /// Distance thresholds for each LOD level (in world units)
    /// [L0->L1, L1->L2, L2->L3]
    pub distance_thresholds: [f32; 3],

    /// Hysteresis margin (0.0-1.0, typically 0.1 for 10%)
    pub hysteresis_margin: f32,

    /// Blend zone size (world units)
    pub blend_zone_size: f32,

    /// Enable blend zones (cross-fade between LODs)
    pub enable_blending: bool,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            // L0->L1 at 256m, L1->L2 at 512m, L2->L3 at 1024m
            distance_thresholds: [256.0, 512.0, 1024.0],
            hysteresis_margin: 0.1, // 10% margin
            blend_zone_size: 32.0,  // 32m blend zone
            enable_blending: true,
        }
    }
}

impl LodConfig {
    /// Get distance threshold for transitioning from `from` to `to` LOD
    pub fn get_threshold(&self, from: LodLevel, to: LodLevel, increasing_detail: bool) -> f32 {
        let base_threshold = match (from, to) {
            (LodLevel::Full, LodLevel::Half) | (LodLevel::Half, LodLevel::Full) => {
                self.distance_thresholds[0]
            }
            (LodLevel::Half, LodLevel::Quarter) | (LodLevel::Quarter, LodLevel::Half) => {
                self.distance_thresholds[1]
            }
            (LodLevel::Quarter, LodLevel::Skybox) | (LodLevel::Skybox, LodLevel::Quarter) => {
                self.distance_thresholds[2]
            }
            _ => return f32::MAX, // Invalid transition
        };

        // Apply hysteresis
        if increasing_detail {
            // Moving to higher detail: subtract margin (trigger sooner)
            base_threshold * (1.0 - self.hysteresis_margin)
        } else {
            // Moving to lower detail: add margin (trigger later)
            base_threshold * (1.0 + self.hysteresis_margin)
        }
    }
}

/// LOD state for a single chunk
#[derive(Debug, Clone)]
pub struct ChunkLodState {
    /// Current LOD level
    pub current_lod: LodLevel,

    /// Target LOD level (for blending)
    pub target_lod: Option<LodLevel>,

    /// Blend factor (0.0 = current, 1.0 = target)
    pub blend_factor: f32,

    /// Distance from camera
    pub distance: f32,
}

/// Cached LOD meshes for a chunk (Phase 2 optimization)
#[derive(Debug, Clone)]
pub struct ChunkLodCache {
    /// Full detail mesh (L0)
    pub l0_mesh: Option<Arc<ChunkMesh>>,

    /// Half resolution mesh (L1)
    pub l1_mesh: Option<Arc<ChunkMesh>>,

    /// Quarter resolution mesh (L2)
    pub l2_mesh: Option<Arc<ChunkMesh>>,

    /// Skybox/impostor mesh (L3) - typically not cached (minimal geometry)
    pub l3_mesh: Option<Arc<ChunkMesh>>,
}

impl ChunkLodCache {
    /// Create an empty cache
    pub fn new() -> Self {
        Self {
            l0_mesh: None,
            l1_mesh: None,
            l2_mesh: None,
            l3_mesh: None,
        }
    }

    /// Get mesh for specific LOD level
    pub fn get_mesh(&self, lod: LodLevel) -> Option<Arc<ChunkMesh>> {
        match lod {
            LodLevel::Full => self.l0_mesh.clone(),
            LodLevel::Half => self.l1_mesh.clone(),
            LodLevel::Quarter => self.l2_mesh.clone(),
            LodLevel::Skybox => self.l3_mesh.clone(),
        }
    }

    /// Store mesh for specific LOD level
    pub fn set_mesh(&mut self, lod: LodLevel, mesh: Arc<ChunkMesh>) {
        match lod {
            LodLevel::Full => self.l0_mesh = Some(mesh),
            LodLevel::Half => self.l1_mesh = Some(mesh),
            LodLevel::Quarter => self.l2_mesh = Some(mesh),
            LodLevel::Skybox => self.l3_mesh = Some(mesh),
        }
    }

    /// Check if mesh exists for LOD level
    pub fn has_mesh(&self, lod: LodLevel) -> bool {
        self.get_mesh(lod).is_some()
    }

    /// Get total memory usage of cached meshes
    pub fn memory_usage(&self) -> usize {
        let mut total = 0;
        if let Some(m) = &self.l0_mesh {
            total += m.memory_usage();
        }
        if let Some(m) = &self.l1_mesh {
            total += m.memory_usage();
        }
        if let Some(m) = &self.l2_mesh {
            total += m.memory_usage();
        }
        if let Some(m) = &self.l3_mesh {
            total += m.memory_usage();
        }
        total
    }
}

/// LOD manager with hysteresis and mesh caching
pub struct LodManager {
    config: LodConfig,

    /// LOD state per chunk
    chunk_states: HashMap<ChunkId, ChunkLodState>,

    /// Mesh cache per chunk (Phase 2 optimization - eliminates regeneration)
    mesh_cache: HashMap<ChunkId, ChunkLodCache>,

    /// Chunk size (for distance calculations)
    chunk_size: f32,

    /// Cache statistics
    cache_hits: usize,
    cache_misses: usize,
}

impl LodManager {
    /// Create a new LOD manager
    pub fn new(config: LodConfig, chunk_size: f32) -> Self {
        Self {
            config,
            chunk_states: HashMap::new(),
            mesh_cache: HashMap::new(),
            chunk_size,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Get mesh from cache (Phase 2 optimization)
    pub fn get_cached_mesh(&mut self, chunk_id: ChunkId, lod: LodLevel) -> Option<Arc<ChunkMesh>> {
        if let Some(cache) = self.mesh_cache.get(&chunk_id) {
            if let Some(mesh) = cache.get_mesh(lod) {
                self.cache_hits += 1;
                return Some(mesh);
            }
        }
        self.cache_misses += 1;
        None
    }

    /// Store mesh in cache (Phase 2 optimization)
    pub fn cache_mesh(&mut self, chunk_id: ChunkId, lod: LodLevel, mesh: Arc<ChunkMesh>) {
        let cache = self
            .mesh_cache
            .entry(chunk_id)
            .or_insert_with(ChunkLodCache::new);
        cache.set_mesh(lod, mesh);
    }

    /// Get cache hit rate (for diagnostics)
    pub fn cache_hit_rate(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f32 / total as f32
        }
    }

    /// Get total cache memory usage
    pub fn cache_memory_usage(&self) -> usize {
        self.mesh_cache.values().map(|c| c.memory_usage()).sum()
    }

    /// Evict cache for chunks beyond distance threshold
    pub fn evict_distant_cache(&mut self, camera_pos: Vec3, max_distance: f32) -> usize {
        let mut evicted = 0;
        self.mesh_cache.retain(|chunk_id, _| {
            let chunk_center = chunk_id.to_center_pos(self.chunk_size);
            let distance = (chunk_center - camera_pos).length();
            if distance > max_distance {
                evicted += 1;
                false
            } else {
                true
            }
        });
        evicted
    }

    /// Update LOD for a chunk based on camera position
    pub fn update_chunk_lod(&mut self, chunk_id: ChunkId, camera_pos: Vec3) -> bool {
        let chunk_center = chunk_id.to_center_pos(self.chunk_size);
        let distance = (chunk_center - camera_pos).length();

        // Get or create chunk state
        let state = self
            .chunk_states
            .entry(chunk_id)
            .or_insert_with(|| ChunkLodState {
                current_lod: LodLevel::Full,
                target_lod: None,
                blend_factor: 0.0,
                distance,
            });

        state.distance = distance;

        // Determine target LOD based on distance
        let target_lod = if distance < self.config.distance_thresholds[0] {
            LodLevel::Full
        } else if distance < self.config.distance_thresholds[1] {
            LodLevel::Half
        } else if distance < self.config.distance_thresholds[2] {
            LodLevel::Quarter
        } else {
            LodLevel::Skybox
        };

        // Check if LOD should change (with hysteresis)
        if target_lod != state.current_lod {
            let increasing_detail = (target_lod as u8) < (state.current_lod as u8);
            let threshold =
                self.config
                    .get_threshold(state.current_lod, target_lod, increasing_detail);

            let should_transition = if increasing_detail {
                distance < threshold
            } else {
                distance > threshold
            };

            if should_transition {
                if self.config.enable_blending {
                    // Start blend transition
                    state.target_lod = Some(target_lod);
                    state.blend_factor = 0.0;
                } else {
                    // Instant transition
                    state.current_lod = target_lod;
                    state.target_lod = None;
                    state.blend_factor = 0.0;
                }
                return true; // LOD changed
            }
        }

        // Update blend factor if transitioning
        if let Some(target) = state.target_lod {
            if self.config.enable_blending {
                // Advance blend factor (lerp towards target)
                state.blend_factor += 0.1; // Adjust blend speed as needed

                if state.blend_factor >= 1.0 {
                    // Transition complete
                    state.current_lod = target;
                    state.target_lod = None;
                    state.blend_factor = 0.0;
                    return true; // LOD changed
                }
            } else {
                // Instant transition
                state.current_lod = target;
                state.target_lod = None;
                state.blend_factor = 0.0;
                return true;
            }
        }

        false // No LOD change
    }

    /// Update all loaded chunks
    pub fn update_all_chunks(&mut self, chunk_ids: &[ChunkId], camera_pos: Vec3) -> usize {
        let mut changed_count = 0;

        for &chunk_id in chunk_ids {
            if self.update_chunk_lod(chunk_id, camera_pos) {
                changed_count += 1;
            }
        }

        // Remove states for unloaded chunks
        self.chunk_states.retain(|id, _| chunk_ids.contains(id));

        changed_count
    }

    /// Get LOD state for a chunk
    pub fn get_chunk_state(&self, chunk_id: ChunkId) -> Option<&ChunkLodState> {
        self.chunk_states.get(&chunk_id)
    }

    /// Get current LOD level for a chunk
    pub fn get_chunk_lod(&self, chunk_id: ChunkId) -> Option<LodLevel> {
        self.chunk_states.get(&chunk_id).map(|s| s.current_lod)
    }

    /// Check if chunk is transitioning between LODs
    pub fn is_transitioning(&self, chunk_id: ChunkId) -> bool {
        self.chunk_states
            .get(&chunk_id)
            .map(|s| s.target_lod.is_some())
            .unwrap_or(false)
    }

    /// Get blend factor for a transitioning chunk
    pub fn get_blend_factor(&self, chunk_id: ChunkId) -> f32 {
        self.chunk_states
            .get(&chunk_id)
            .map(|s| s.blend_factor)
            .unwrap_or(0.0)
    }

    /// Get statistics
    pub fn get_stats(&self) -> LodStats {
        let mut stats = LodStats::default();

        for state in self.chunk_states.values() {
            match state.current_lod {
                LodLevel::Full => stats.full_count += 1,
                LodLevel::Half => stats.half_count += 1,
                LodLevel::Quarter => stats.quarter_count += 1,
                LodLevel::Skybox => stats.skybox_count += 1,
            }

            if state.target_lod.is_some() {
                stats.transitioning_count += 1;
            }
        }

        stats.total_chunks = self.chunk_states.len();
        stats
    }
}

/// LOD statistics
#[derive(Debug, Clone, Default)]
pub struct LodStats {
    pub total_chunks: usize,
    pub full_count: usize,
    pub half_count: usize,
    pub quarter_count: usize,
    pub skybox_count: usize,
    pub transitioning_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lod_level_skip_factors() {
        assert_eq!(LodLevel::Full.skip_factor(), 1);
        assert_eq!(LodLevel::Half.skip_factor(), 2);
        assert_eq!(LodLevel::Quarter.skip_factor(), 4);
        assert_eq!(LodLevel::Skybox.skip_factor(), 16);
    }

    #[test]
    fn lod_level_transitions() {
        assert_eq!(LodLevel::Full.lower(), Some(LodLevel::Half));
        assert_eq!(LodLevel::Half.lower(), Some(LodLevel::Quarter));
        assert_eq!(LodLevel::Quarter.lower(), Some(LodLevel::Skybox));
        assert_eq!(LodLevel::Skybox.lower(), None);

        assert_eq!(LodLevel::Skybox.higher(), Some(LodLevel::Quarter));
        assert_eq!(LodLevel::Quarter.higher(), Some(LodLevel::Half));
        assert_eq!(LodLevel::Half.higher(), Some(LodLevel::Full));
        assert_eq!(LodLevel::Full.higher(), None);
    }

    #[test]
    fn hysteresis_margins() {
        let config = LodConfig::default();

        // Increasing detail (moving closer)
        let threshold_in = config.get_threshold(LodLevel::Half, LodLevel::Full, true);
        assert!(threshold_in < config.distance_thresholds[0]);

        // Decreasing detail (moving away)
        let threshold_out = config.get_threshold(LodLevel::Full, LodLevel::Half, false);
        assert!(threshold_out > config.distance_thresholds[0]);

        // Hysteresis gap
        assert!(threshold_out > threshold_in);
    }

    #[test]
    fn lod_manager_basic() {
        let config = LodConfig {
            distance_thresholds: [256.0, 512.0, 1024.0],
            hysteresis_margin: 0.1,
            blend_zone_size: 32.0,
            enable_blending: false, // Disable blending for simpler test
        };
        let mut manager = LodManager::new(config, 256.0);
        let chunk_id = ChunkId::new(0, 0);

        // Start at chunk center (distance = 0) -> Full LOD
        let chunk_center = chunk_id.to_center_pos(256.0);
        manager.update_chunk_lod(chunk_id, chunk_center);
        assert_eq!(manager.get_chunk_lod(chunk_id), Some(LodLevel::Full));

        // Move to exact threshold distance + a bit more (300m from center)
        let far_pos = chunk_center + Vec3::new(300.0, 0.0, 0.0);
        manager.update_chunk_lod(chunk_id, far_pos);
        // Distance is 300, threshold for downgrade is 256 * 1.1 = 281.6
        assert_eq!(manager.get_chunk_lod(chunk_id), Some(LodLevel::Half));
    }
}
