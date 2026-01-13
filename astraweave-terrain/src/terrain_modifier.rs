//! TerrainModifier - Batched Voxel Updates with Time-Slicing
//!
//! This module implements a terrain modification system that processes voxel
//! operations in a time-sliced manner to maintain frame budget compliance.
//! It uses a two-pass approach:
//! 1. DATA PASS: Apply pending voxel operations (fast)
//! 2. MESH PASS: Re-mesh modified chunks (potentially slow, limited per frame)
//!
//! # Frame Budget
//! The modifier respects a configurable time budget per frame (default 2ms)
//! and will defer remaining work to subsequent frames.
//!
//! # Example
//! ```ignore
//! let mut modifier = TerrainModifier::new(config);
//! modifier.queue_operation(VoxelOp::set(pos, voxel));
//! modifier.tick(delta_time); // Processes ops within time budget
//! ```

use crate::{ChunkCoord, Voxel, VoxelGrid};
use glam::{IVec3, Vec3};
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

/// Configuration for the TerrainModifier
#[derive(Debug, Clone)]
pub struct TerrainModifierConfig {
    /// Maximum time budget for data pass per frame (microseconds)
    pub data_pass_budget_us: u64,
    /// Maximum time budget for mesh pass per frame (microseconds)
    pub mesh_pass_budget_us: u64,
    /// Maximum voxel operations to process per frame
    pub max_ops_per_frame: usize,
    /// Maximum chunk remeshes per frame
    pub max_remeshes_per_frame: usize,
    /// Whether to prioritize chunks near the camera
    pub prioritize_near_camera: bool,
}

impl Default for TerrainModifierConfig {
    fn default() -> Self {
        Self {
            data_pass_budget_us: 1000, // 1ms for data pass
            mesh_pass_budget_us: 2000, // 2ms for mesh pass
            max_ops_per_frame: 1000,
            max_remeshes_per_frame: 4,
            prioritize_near_camera: true,
        }
    }
}

/// A single voxel operation
#[derive(Debug, Clone)]
pub struct VoxelOp {
    /// World position of the voxel
    pub position: IVec3,
    /// The operation type
    pub op_type: VoxelOpType,
    /// Priority (higher = process first)
    pub priority: u8,
    /// Request ID for tracking
    pub request_id: String,
}

/// Types of voxel operations
#[derive(Debug, Clone)]
pub enum VoxelOpType {
    /// Set a voxel to a specific value
    Set(Voxel),
    /// Add density to a voxel
    AddDensity(f32),
    /// Subtract density from a voxel
    SubtractDensity(f32),
    /// Set material only (preserve density)
    SetMaterial(u16),
    /// Blend with existing voxel
    Blend { voxel: Voxel, factor: f32 },
}

impl VoxelOp {
    /// Create a set operation
    pub fn set(position: IVec3, voxel: Voxel, request_id: String) -> Self {
        Self {
            position,
            op_type: VoxelOpType::Set(voxel),
            priority: 128,
            request_id,
        }
    }

    /// Create an add density operation
    pub fn add_density(position: IVec3, density: f32, request_id: String) -> Self {
        Self {
            position,
            op_type: VoxelOpType::AddDensity(density),
            priority: 128,
            request_id,
        }
    }

    /// Create a subtract density operation
    pub fn subtract_density(position: IVec3, density: f32, request_id: String) -> Self {
        Self {
            position,
            op_type: VoxelOpType::SubtractDensity(density),
            priority: 128,
            request_id,
        }
    }

    /// Set priority (higher values processed first)
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Statistics for terrain modification
#[derive(Debug, Clone, Default)]
pub struct ModifierStats {
    /// Operations processed this frame
    pub ops_processed: usize,
    /// Operations remaining in queue
    pub ops_pending: usize,
    /// Chunks remeshed this frame
    pub chunks_remeshed: usize,
    /// Chunks pending remesh
    pub chunks_pending_remesh: usize,
    /// Time spent on data pass (microseconds)
    pub data_pass_time_us: u64,
    /// Time spent on mesh pass (microseconds)
    pub mesh_pass_time_us: u64,
    /// Whether work was deferred to next frame
    pub work_deferred: bool,
}

/// Batched terrain modifier with time-slicing
pub struct TerrainModifier {
    config: TerrainModifierConfig,
    /// Pending voxel operations
    op_queue: VecDeque<VoxelOp>,
    /// Chunks that need remeshing (after voxel data updated)
    dirty_chunks: HashSet<ChunkCoord>,
    /// Priority queue for chunk remeshing (coord, distance from camera)
    remesh_priority: Vec<(ChunkCoord, f32)>,
    /// Current camera position for prioritization
    camera_pos: Vec3,
    /// Completed request IDs (for notification)
    completed_requests: Vec<String>,
    /// Statistics from last tick
    last_stats: ModifierStats,
    /// NavMesh regions that need invalidation
    navmesh_dirty_regions: Vec<NavMeshRegion>,
}

/// A region to invalidate in the NavMesh
#[derive(Debug, Clone)]
pub struct NavMeshRegion {
    /// Minimum corner of the region (world coordinates)
    pub min: Vec3,
    /// Maximum corner of the region (world coordinates)
    pub max: Vec3,
}

impl NavMeshRegion {
    /// Create a new region
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create a region from a chunk coordinate
    pub fn from_chunk(coord: ChunkCoord) -> Self {
        let min = coord.to_world_pos();
        let size = crate::CHUNK_SIZE as f32;
        let max = min + Vec3::splat(size);
        Self { min, max }
    }

    /// Check if this region overlaps with another
    pub fn overlaps(&self, other: &NavMeshRegion) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Merge with another region (returns bounding box)
    pub fn merge(&self, other: &NavMeshRegion) -> NavMeshRegion {
        NavMeshRegion {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }
}

impl TerrainModifier {
    /// Create a new terrain modifier
    pub fn new(config: TerrainModifierConfig) -> Self {
        Self {
            config,
            op_queue: VecDeque::new(),
            dirty_chunks: HashSet::new(),
            remesh_priority: Vec::new(),
            camera_pos: Vec3::ZERO,
            completed_requests: Vec::new(),
            last_stats: ModifierStats::default(),
            navmesh_dirty_regions: Vec::new(),
        }
    }

    /// Queue a voxel operation for processing
    pub fn queue_operation(&mut self, op: VoxelOp) {
        // Insert based on priority (higher priority first)
        let insert_pos = self
            .op_queue
            .iter()
            .position(|existing| existing.priority < op.priority)
            .unwrap_or(self.op_queue.len());
        self.op_queue.insert(insert_pos, op);
    }

    /// Queue multiple voxel operations
    pub fn queue_operations(&mut self, ops: Vec<VoxelOp>) {
        for op in ops {
            self.queue_operation(op);
        }
    }

    /// Update camera position for chunk prioritization
    pub fn update_camera(&mut self, camera_pos: Vec3) {
        self.camera_pos = camera_pos;
    }

    /// Process pending operations within time budget
    ///
    /// This is the main tick function, called once per frame.
    /// Returns statistics about work done.
    pub fn tick(&mut self, voxel_grid: &mut VoxelGrid) -> ModifierStats {
        let mut stats = ModifierStats {
            ops_pending: self.op_queue.len(),
            chunks_pending_remesh: self.dirty_chunks.len(),
            ..Default::default()
        };

        // ===========================
        // DATA PASS: Apply voxel operations
        // ===========================
        let data_start = Instant::now();
        let data_budget = Duration::from_micros(self.config.data_pass_budget_us);
        let mut ops_this_frame = 0;

        while !self.op_queue.is_empty()
            && ops_this_frame < self.config.max_ops_per_frame
            && data_start.elapsed() < data_budget
        {
            if let Some(op) = self.op_queue.pop_front() {
                self.apply_operation(voxel_grid, &op);
                ops_this_frame += 1;

                // Track request completion
                if !self.completed_requests.contains(&op.request_id) {
                    self.completed_requests.push(op.request_id.clone());
                }
            }
        }

        stats.ops_processed = ops_this_frame;
        stats.data_pass_time_us = data_start.elapsed().as_micros() as u64;

        // ===========================
        // MESH PASS: Remesh dirty chunks
        // ===========================
        let mesh_start = Instant::now();
        let mesh_budget = Duration::from_micros(self.config.mesh_pass_budget_us);

        // Build priority list for remeshing
        self.update_remesh_priority();

        let mut remeshes = 0;
        while !self.remesh_priority.is_empty()
            && remeshes < self.config.max_remeshes_per_frame
            && mesh_start.elapsed() < mesh_budget
        {
            if let Some((coord, _)) = self.remesh_priority.pop() {
                self.dirty_chunks.remove(&coord);

                // Mark chunk for remeshing in the voxel grid
                if let Some(chunk) = voxel_grid.get_chunk_mut(coord) {
                    // The chunk's dirty flag is already set from apply_operation
                    // Here we would trigger the actual mesh generation
                    // For now, we just mark it clean after "remeshing"
                    chunk.mark_clean();
                    remeshes += 1;

                    // Add NavMesh dirty region
                    self.navmesh_dirty_regions
                        .push(NavMeshRegion::from_chunk(coord));
                }
            }
        }

        stats.chunks_remeshed = remeshes;
        stats.mesh_pass_time_us = mesh_start.elapsed().as_micros() as u64;
        stats.ops_pending = self.op_queue.len();
        stats.chunks_pending_remesh = self.dirty_chunks.len();
        stats.work_deferred = !self.op_queue.is_empty() || !self.dirty_chunks.is_empty();

        self.last_stats = stats.clone();
        stats
    }

    /// Apply a single voxel operation
    fn apply_operation(&mut self, voxel_grid: &mut VoxelGrid, op: &VoxelOp) {
        let chunk_coord = ChunkCoord::from_world_pos(op.position.as_vec3());

        // Ensure chunk exists
        let chunk = voxel_grid.get_or_create_chunk(chunk_coord);

        // Calculate local position within chunk
        let chunk_origin = chunk_coord.to_world_pos();
        let local_pos = op.position
            - IVec3::new(
                chunk_origin.x as i32,
                chunk_origin.y as i32,
                chunk_origin.z as i32,
            );

        // Apply the operation
        match &op.op_type {
            VoxelOpType::Set(voxel) => {
                chunk.set_voxel(local_pos, *voxel);
            }
            VoxelOpType::AddDensity(amount) => {
                if let Some(existing) = chunk.get_voxel(local_pos) {
                    let new_density = (existing.density + amount).clamp(0.0, 1.0);
                    chunk.set_voxel(local_pos, Voxel::new(new_density, existing.material));
                }
            }
            VoxelOpType::SubtractDensity(amount) => {
                if let Some(existing) = chunk.get_voxel(local_pos) {
                    let new_density = (existing.density - amount).clamp(0.0, 1.0);
                    chunk.set_voxel(local_pos, Voxel::new(new_density, existing.material));
                }
            }
            VoxelOpType::SetMaterial(material) => {
                if let Some(existing) = chunk.get_voxel(local_pos) {
                    chunk.set_voxel(local_pos, Voxel::new(existing.density, *material));
                }
            }
            VoxelOpType::Blend { voxel, factor } => {
                if let Some(existing) = chunk.get_voxel(local_pos) {
                    let new_density = existing.density * (1.0 - factor) + voxel.density * factor;
                    // Use target voxel's material if factor > 0.5
                    let new_material = if *factor > 0.5 {
                        voxel.material
                    } else {
                        existing.material
                    };
                    chunk.set_voxel(
                        local_pos,
                        Voxel::new(new_density.clamp(0.0, 1.0), new_material),
                    );
                }
            }
        }

        // Mark chunk as dirty for remeshing
        self.dirty_chunks.insert(chunk_coord);
    }

    /// Update the priority list for chunk remeshing
    fn update_remesh_priority(&mut self) {
        self.remesh_priority.clear();

        for &coord in &self.dirty_chunks {
            let chunk_center = coord.to_world_pos() + Vec3::splat(crate::CHUNK_SIZE as f32 / 2.0);
            let distance = (chunk_center - self.camera_pos).length();
            self.remesh_priority.push((coord, distance));
        }

        // Sort by distance (ascending - closer chunks first)
        if self.config.prioritize_near_camera {
            self.remesh_priority
                .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    /// Get statistics from the last tick
    pub fn stats(&self) -> &ModifierStats {
        &self.last_stats
    }

    /// Get pending operation count
    pub fn pending_ops(&self) -> usize {
        self.op_queue.len()
    }

    /// Get pending remesh count
    pub fn pending_remeshes(&self) -> usize {
        self.dirty_chunks.len()
    }

    /// Check if there is pending work
    pub fn has_pending_work(&self) -> bool {
        !self.op_queue.is_empty() || !self.dirty_chunks.is_empty()
    }

    /// Drain completed request IDs
    pub fn drain_completed_requests(&mut self) -> Vec<String> {
        std::mem::take(&mut self.completed_requests)
    }

    /// Take NavMesh dirty regions (for NavMesh system to process)
    pub fn take_navmesh_dirty_regions(&mut self) -> Vec<NavMeshRegion> {
        std::mem::take(&mut self.navmesh_dirty_regions)
    }

    /// Get current NavMesh dirty regions without removing them
    pub fn navmesh_dirty_regions(&self) -> &[NavMeshRegion] {
        &self.navmesh_dirty_regions
    }

    /// Clear all pending operations
    pub fn clear(&mut self) {
        self.op_queue.clear();
        self.dirty_chunks.clear();
        self.remesh_priority.clear();
        self.completed_requests.clear();
        self.navmesh_dirty_regions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_modifier() -> TerrainModifier {
        TerrainModifier::new(TerrainModifierConfig::default())
    }

    #[test]
    fn test_config_default() {
        let config = TerrainModifierConfig::default();
        assert_eq!(config.data_pass_budget_us, 1000);
        assert_eq!(config.mesh_pass_budget_us, 2000);
        assert_eq!(config.max_ops_per_frame, 1000);
        assert_eq!(config.max_remeshes_per_frame, 4);
        assert!(config.prioritize_near_camera);
    }

    #[test]
    fn test_voxel_op_creation() {
        let op = VoxelOp::set(
            IVec3::new(10, 20, 30),
            Voxel::new(0.8, 1),
            "test-001".to_string(),
        );

        assert_eq!(op.position, IVec3::new(10, 20, 30));
        assert_eq!(op.request_id, "test-001");
        assert_eq!(op.priority, 128); // Default priority
    }

    #[test]
    fn test_voxel_op_with_priority() {
        let op = VoxelOp::add_density(IVec3::ZERO, 0.5, "test".to_string()).with_priority(255);

        assert_eq!(op.priority, 255);
    }

    #[test]
    fn test_queue_operations_priority() {
        let mut modifier = create_test_modifier();

        // Queue operations with different priorities
        modifier.queue_operation(
            VoxelOp::set(IVec3::ZERO, Voxel::new(0.5, 0), "low".to_string()).with_priority(50),
        );
        modifier.queue_operation(
            VoxelOp::set(IVec3::ONE, Voxel::new(0.5, 0), "high".to_string()).with_priority(200),
        );
        modifier.queue_operation(
            VoxelOp::set(IVec3::NEG_ONE, Voxel::new(0.5, 0), "medium".to_string())
                .with_priority(100),
        );

        // High priority should be first
        assert_eq!(modifier.op_queue[0].request_id, "high");
        assert_eq!(modifier.op_queue[1].request_id, "medium");
        assert_eq!(modifier.op_queue[2].request_id, "low");
    }

    #[test]
    fn test_pending_work() {
        let mut modifier = create_test_modifier();
        assert!(!modifier.has_pending_work());

        modifier.queue_operation(VoxelOp::set(
            IVec3::ZERO,
            Voxel::new(0.5, 0),
            "test".to_string(),
        ));
        assert!(modifier.has_pending_work());
    }

    #[test]
    fn test_clear() {
        let mut modifier = create_test_modifier();

        modifier.queue_operation(VoxelOp::set(
            IVec3::ZERO,
            Voxel::new(0.5, 0),
            "test".to_string(),
        ));
        modifier.dirty_chunks.insert(ChunkCoord::new(0, 0, 0));

        modifier.clear();

        assert_eq!(modifier.pending_ops(), 0);
        assert_eq!(modifier.pending_remeshes(), 0);
        assert!(!modifier.has_pending_work());
    }

    #[test]
    fn test_navmesh_region_from_chunk() {
        let region = NavMeshRegion::from_chunk(ChunkCoord::new(0, 0, 0));
        assert_eq!(region.min, Vec3::ZERO);
    }

    #[test]
    fn test_navmesh_region_overlaps() {
        let region1 = NavMeshRegion::new(Vec3::ZERO, Vec3::splat(10.0));
        let region2 = NavMeshRegion::new(Vec3::splat(5.0), Vec3::splat(15.0));
        let region3 = NavMeshRegion::new(Vec3::splat(20.0), Vec3::splat(30.0));

        assert!(region1.overlaps(&region2));
        assert!(region2.overlaps(&region1));
        assert!(!region1.overlaps(&region3));
    }

    #[test]
    fn test_navmesh_region_merge() {
        let region1 = NavMeshRegion::new(Vec3::ZERO, Vec3::splat(10.0));
        let region2 = NavMeshRegion::new(Vec3::splat(5.0), Vec3::splat(15.0));

        let merged = region1.merge(&region2);
        assert_eq!(merged.min, Vec3::ZERO);
        assert_eq!(merged.max, Vec3::splat(15.0));
    }

    #[test]
    fn test_modifier_stats_default() {
        let stats = ModifierStats::default();
        assert_eq!(stats.ops_processed, 0);
        assert_eq!(stats.ops_pending, 0);
        assert_eq!(stats.chunks_remeshed, 0);
        assert!(!stats.work_deferred);
    }

    #[test]
    fn test_drain_completed_requests() {
        let mut modifier = create_test_modifier();
        modifier.completed_requests.push("req1".to_string());
        modifier.completed_requests.push("req2".to_string());

        let drained = modifier.drain_completed_requests();
        assert_eq!(drained.len(), 2);
        assert!(modifier.completed_requests.is_empty());
    }

    #[test]
    fn test_update_camera() {
        let mut modifier = create_test_modifier();
        modifier.update_camera(Vec3::new(100.0, 50.0, 100.0));
        assert_eq!(modifier.camera_pos, Vec3::new(100.0, 50.0, 100.0));
    }

    #[test]
    fn test_voxel_op_types() {
        // Test all VoxelOpType variants can be created
        let _ = VoxelOpType::Set(Voxel::new(0.5, 1));
        let _ = VoxelOpType::AddDensity(0.1);
        let _ = VoxelOpType::SubtractDensity(0.1);
        let _ = VoxelOpType::SetMaterial(5);
        let _ = VoxelOpType::Blend {
            voxel: Voxel::new(0.8, 2),
            factor: 0.5,
        };
    }

    #[test]
    fn test_voxel_op_subtract_density() {
        let op = VoxelOp::subtract_density(IVec3::new(5, 5, 5), 0.3, "sub-test".to_string());
        assert_eq!(op.position, IVec3::new(5, 5, 5));
        matches!(op.op_type, VoxelOpType::SubtractDensity(0.3));
    }

    #[test]
    fn test_config_clone() {
        let config = TerrainModifierConfig::default();
        let cloned = config.clone();
        assert_eq!(config.data_pass_budget_us, cloned.data_pass_budget_us);
        assert_eq!(config.mesh_pass_budget_us, cloned.mesh_pass_budget_us);
        assert_eq!(config.max_ops_per_frame, cloned.max_ops_per_frame);
    }

    #[test]
    fn test_modifier_stats_clone() {
        let stats = ModifierStats {
            ops_processed: 10,
            ops_pending: 5,
            chunks_remeshed: 2,
            chunks_pending_remesh: 3,
            data_pass_time_us: 500,
            mesh_pass_time_us: 1000,
            work_deferred: true,
        };
        let cloned = stats.clone();
        assert_eq!(stats.ops_processed, cloned.ops_processed);
        assert_eq!(stats.work_deferred, cloned.work_deferred);
    }

    #[test]
    fn test_voxel_op_clone() {
        let op = VoxelOp::set(IVec3::new(1, 2, 3), Voxel::new(0.5, 1), "clone-test".to_string());
        let cloned = op.clone();
        assert_eq!(op.position, cloned.position);
        assert_eq!(op.priority, cloned.priority);
        assert_eq!(op.request_id, cloned.request_id);
    }

    #[test]
    fn test_navmesh_region_new() {
        let region = NavMeshRegion::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        assert_eq!(region.min, Vec3::ZERO);
        assert_eq!(region.max, Vec3::splat(10.0));
    }

    #[test]
    fn test_navmesh_region_clone() {
        let region = NavMeshRegion::new(Vec3::ZERO, Vec3::splat(5.0));
        let cloned = region.clone();
        assert_eq!(region.min, cloned.min);
        assert_eq!(region.max, cloned.max);
    }

    #[test]
    fn test_queue_multiple_operations() {
        let mut modifier = create_test_modifier();
        let ops = vec![
            VoxelOp::set(IVec3::new(1, 0, 0), Voxel::new(0.5, 0), "op1".to_string()),
            VoxelOp::set(IVec3::new(2, 0, 0), Voxel::new(0.5, 0), "op2".to_string()),
            VoxelOp::set(IVec3::new(3, 0, 0), Voxel::new(0.5, 0), "op3".to_string()),
        ];
        modifier.queue_operations(ops);
        assert_eq!(modifier.pending_ops(), 3);
    }

    #[test]
    fn test_navmesh_dirty_regions_getter() {
        let mut modifier = create_test_modifier();
        modifier.navmesh_dirty_regions.push(NavMeshRegion::new(Vec3::ZERO, Vec3::ONE));
        
        let regions = modifier.navmesh_dirty_regions();
        assert_eq!(regions.len(), 1);
        
        // Getter doesn't remove regions
        assert_eq!(modifier.navmesh_dirty_regions.len(), 1);
    }

    #[test]
    fn test_take_navmesh_dirty_regions() {
        let mut modifier = create_test_modifier();
        modifier.navmesh_dirty_regions.push(NavMeshRegion::new(Vec3::ZERO, Vec3::ONE));
        modifier.navmesh_dirty_regions.push(NavMeshRegion::new(Vec3::ONE, Vec3::splat(2.0)));
        
        let taken = modifier.take_navmesh_dirty_regions();
        assert_eq!(taken.len(), 2);
        assert!(modifier.navmesh_dirty_regions.is_empty());
    }

    #[test]
    fn test_stats_getter() {
        let modifier = create_test_modifier();
        let stats = modifier.stats();
        assert_eq!(stats.ops_processed, 0);
    }

    #[test]
    fn test_pending_ops() {
        let mut modifier = create_test_modifier();
        assert_eq!(modifier.pending_ops(), 0);
        
        modifier.queue_operation(VoxelOp::set(IVec3::ZERO, Voxel::new(0.5, 0), "test".to_string()));
        assert_eq!(modifier.pending_ops(), 1);
    }

    #[test]
    fn test_pending_remeshes() {
        let mut modifier = create_test_modifier();
        assert_eq!(modifier.pending_remeshes(), 0);
        
        modifier.dirty_chunks.insert(ChunkCoord::new(0, 0, 0));
        assert_eq!(modifier.pending_remeshes(), 1);
    }

    #[test]
    fn test_navmesh_region_no_overlap_z_axis() {
        let region1 = NavMeshRegion::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        let region2 = NavMeshRegion::new(Vec3::new(0.0, 0.0, 20.0), Vec3::new(10.0, 10.0, 30.0));
        
        assert!(!region1.overlaps(&region2));
    }

    #[test]
    fn test_navmesh_region_edge_overlap() {
        let region1 = NavMeshRegion::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        let region2 = NavMeshRegion::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(20.0, 10.0, 10.0));
        
        // Edge-touching should still count as overlap
        assert!(region1.overlaps(&region2));
    }

    #[test]
    fn test_modifier_new_with_custom_config() {
        let config = TerrainModifierConfig {
            data_pass_budget_us: 500,
            mesh_pass_budget_us: 1500,
            max_ops_per_frame: 500,
            max_remeshes_per_frame: 2,
            prioritize_near_camera: false,
        };
        let modifier = TerrainModifier::new(config.clone());
        assert_eq!(modifier.config.data_pass_budget_us, 500);
        assert!(!modifier.config.prioritize_near_camera);
    }
}
