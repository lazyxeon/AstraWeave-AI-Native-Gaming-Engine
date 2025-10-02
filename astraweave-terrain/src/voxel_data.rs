//! Voxel Data Structures for Hybrid Voxel/Polygon Terrain
//!
//! This module implements a Sparse Voxel Octree (SVO) for efficient storage
//! and manipulation of voxel terrain data. The voxel system supports:
//! - Dynamic terrain deformation and destruction
//! - Material assignment per voxel
//! - Efficient sparse storage
//! - Integration with World Partition for streaming

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Size of a voxel chunk in each dimension (32x32x32 voxels)
pub const CHUNK_SIZE: i32 = 32;

/// Maximum depth of the octree (allows for fine detail)
pub const MAX_OCTREE_DEPTH: u32 = 5;

/// Voxel density value (0.0 = empty, 1.0 = solid)
/// Values between 0 and 1 represent the isosurface
pub type Density = f32;

/// Material ID for voxel rendering
pub type MaterialId = u16;

/// 3D coordinate for a voxel chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    /// Create a new chunk coordinate
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Convert world position to chunk coordinate
    pub fn from_world_pos(pos: Vec3) -> Self {
        Self {
            x: (pos.x / CHUNK_SIZE as f32).floor() as i32,
            y: (pos.y / CHUNK_SIZE as f32).floor() as i32,
            z: (pos.z / CHUNK_SIZE as f32).floor() as i32,
        }
    }

    /// Get the world position of the chunk's origin (min corner)
    pub fn to_world_pos(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE as f32,
            self.y as f32 * CHUNK_SIZE as f32,
            self.z as f32 * CHUNK_SIZE as f32,
        )
    }

    /// Get neighboring chunk coordinates
    pub fn neighbors(&self) -> [ChunkCoord; 6] {
        [
            ChunkCoord::new(self.x + 1, self.y, self.z),
            ChunkCoord::new(self.x - 1, self.y, self.z),
            ChunkCoord::new(self.x, self.y + 1, self.z),
            ChunkCoord::new(self.x, self.y - 1, self.z),
            ChunkCoord::new(self.x, self.y, self.z + 1),
            ChunkCoord::new(self.x, self.y, self.z - 1),
        ]
    }
}

/// A single voxel with density and material information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Voxel {
    /// Density value (0.0 = empty, 1.0 = solid)
    pub density: Density,
    /// Material ID for rendering
    pub material: MaterialId,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            density: 0.0,
            material: 0,
        }
    }
}

impl Voxel {
    /// Create a new voxel with given density and material
    pub fn new(density: Density, material: MaterialId) -> Self {
        Self { density, material }
    }

    /// Check if voxel is solid (density > 0.5)
    pub fn is_solid(&self) -> bool {
        self.density > 0.5
    }

    /// Check if voxel is empty (density < 0.01)
    pub fn is_empty(&self) -> bool {
        self.density < 0.01
    }
}

/// Octree node for sparse voxel storage
#[derive(Debug, Clone, Serialize, Deserialize)]
enum OctreeNode {
    /// Leaf node containing a single voxel value
    Leaf(Voxel),
    /// Internal node with 8 children (may be None for empty space)
    Internal(Box<[Option<OctreeNode>; 8]>),
}

impl OctreeNode {
    /// Create a new leaf node
    fn leaf(voxel: Voxel) -> Self {
        OctreeNode::Leaf(voxel)
    }

    /// Create a new internal node with all children set to None
    fn internal() -> Self {
        OctreeNode::Internal(Box::new([None, None, None, None, None, None, None, None]))
    }

    /// Get child index for given local position within node bounds
    fn child_index(local_pos: IVec3, size: i32) -> usize {
        let half = size / 2;
        let x = if local_pos.x >= half { 1 } else { 0 };
        let y = if local_pos.y >= half { 1 } else { 0 };
        let z = if local_pos.z >= half { 1 } else { 0 };
        x | (y << 1) | (z << 2)
    }

    /// Get the voxel at a specific position within this node
    fn get_voxel(&self, local_pos: IVec3, size: i32, depth: u32) -> Option<Voxel> {
        match self {
            OctreeNode::Leaf(voxel) => Some(*voxel),
            OctreeNode::Internal(children) => {
                if depth >= MAX_OCTREE_DEPTH {
                    return None;
                }
                let idx = Self::child_index(local_pos, size);
                let half = size / 2;
                let child_pos = IVec3::new(
                    local_pos.x % half,
                    local_pos.y % half,
                    local_pos.z % half,
                );
                children[idx]
                    .as_ref()
                    .and_then(|child| child.get_voxel(child_pos, half, depth + 1))
            }
        }
    }

    /// Set the voxel at a specific position within this node
    fn set_voxel(&mut self, local_pos: IVec3, size: i32, depth: u32, voxel: Voxel) {
        if depth >= MAX_OCTREE_DEPTH {
            // At max depth, convert to leaf
            *self = OctreeNode::Leaf(voxel);
            return;
        }

        match self {
            OctreeNode::Leaf(_) => {
                // Convert leaf to internal node
                *self = OctreeNode::internal();
                self.set_voxel(local_pos, size, depth, voxel);
            }
            OctreeNode::Internal(children) => {
                let idx = Self::child_index(local_pos, size);
                let half = size / 2;
                let child_pos = IVec3::new(
                    local_pos.x % half,
                    local_pos.y % half,
                    local_pos.z % half,
                );

                if children[idx].is_none() {
                    children[idx] = Some(OctreeNode::leaf(Voxel::default()));
                }

                if let Some(child) = &mut children[idx] {
                    child.set_voxel(child_pos, half, depth + 1, voxel);
                }
            }
        }
    }
}

/// A chunk of voxel data using Sparse Voxel Octree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelChunk {
    /// Chunk coordinate in world space
    coord: ChunkCoord,
    /// Root of the octree
    root: Option<OctreeNode>,
    /// Flag indicating if chunk has been modified
    dirty: bool,
}

impl VoxelChunk {
    /// Create a new empty voxel chunk
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            root: None,
            dirty: false,
        }
    }

    /// Get the chunk coordinate
    pub fn coord(&self) -> ChunkCoord {
        self.coord
    }

    /// Check if chunk is dirty (needs remeshing)
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark chunk as clean (after meshing)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Get voxel at local position (0..CHUNK_SIZE)
    pub fn get_voxel(&self, local_pos: IVec3) -> Option<Voxel> {
        if !self.is_valid_local_pos(local_pos) {
            return None;
        }
        self.root
            .as_ref()
            .and_then(|root| root.get_voxel(local_pos, CHUNK_SIZE, 0))
    }

    /// Set voxel at local position (0..CHUNK_SIZE)
    pub fn set_voxel(&mut self, local_pos: IVec3, voxel: Voxel) {
        if !self.is_valid_local_pos(local_pos) {
            return;
        }

        if self.root.is_none() {
            self.root = Some(OctreeNode::leaf(Voxel::default()));
        }

        if let Some(root) = &mut self.root {
            root.set_voxel(local_pos, CHUNK_SIZE, 0, voxel);
            self.dirty = true;
        }
    }

    /// Check if local position is within chunk bounds
    fn is_valid_local_pos(&self, pos: IVec3) -> bool {
        pos.x >= 0 && pos.x < CHUNK_SIZE && pos.y >= 0 && pos.y < CHUNK_SIZE && pos.z >= 0 && pos.z < CHUNK_SIZE
    }

    /// Get voxel at world position
    pub fn get_voxel_world(&self, world_pos: Vec3) -> Option<Voxel> {
        let local_pos = self.world_to_local(world_pos);
        self.get_voxel(local_pos)
    }

    /// Set voxel at world position
    pub fn set_voxel_world(&mut self, world_pos: Vec3, voxel: Voxel) {
        let local_pos = self.world_to_local(world_pos);
        self.set_voxel(local_pos, voxel);
    }

    /// Convert world position to local chunk position
    fn world_to_local(&self, world_pos: Vec3) -> IVec3 {
        let chunk_origin = self.coord.to_world_pos();
        let local = world_pos - chunk_origin;
        IVec3::new(
            local.x.floor() as i32,
            local.y.floor() as i32,
            local.z.floor() as i32,
        )
    }

    /// Check if chunk is empty (no solid voxels)
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.estimate_tree_size()
    }

    /// Estimate octree memory usage
    fn estimate_tree_size(&self) -> usize {
        fn node_size(node: &OctreeNode) -> usize {
            match node {
                OctreeNode::Leaf(_) => std::mem::size_of::<Voxel>(),
                OctreeNode::Internal(children) => {
                    let mut size = std::mem::size_of::<[Option<OctreeNode>; 8]>();
                    for child in children.iter().flatten() {
                        size += node_size(child);
                    }
                    size
                }
            }
        }
        self.root.as_ref().map_or(0, node_size)
    }
}

/// Grid of voxel chunks with HashMap-based storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelGrid {
    /// Chunks stored by coordinate
    chunks: HashMap<ChunkCoord, VoxelChunk>,
    /// List of dirty chunks that need remeshing
    dirty_chunks: Vec<ChunkCoord>,
}

impl VoxelGrid {
    /// Create a new empty voxel grid
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            dirty_chunks: Vec::new(),
        }
    }

    /// Get a chunk at the given coordinate
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&VoxelChunk> {
        self.chunks.get(&coord)
    }

    /// Get a mutable chunk at the given coordinate
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut VoxelChunk> {
        self.chunks.get_mut(&coord)
    }

    /// Get or create a chunk at the given coordinate
    pub fn get_or_create_chunk(&mut self, coord: ChunkCoord) -> &mut VoxelChunk {
        self.chunks
            .entry(coord)
            .or_insert_with(|| VoxelChunk::new(coord))
    }

    /// Set voxel at world position
    pub fn set_voxel(&mut self, world_pos: Vec3, voxel: Voxel) {
        let coord = ChunkCoord::from_world_pos(world_pos);
        let chunk = self.get_or_create_chunk(coord);
        chunk.set_voxel_world(world_pos, voxel);
        
        if chunk.is_dirty() && !self.dirty_chunks.contains(&coord) {
            self.dirty_chunks.push(coord);
        }
    }

    /// Get voxel at world position
    pub fn get_voxel(&self, world_pos: Vec3) -> Option<Voxel> {
        let coord = ChunkCoord::from_world_pos(world_pos);
        self.chunks
            .get(&coord)
            .and_then(|chunk| chunk.get_voxel_world(world_pos))
    }

    /// Get list of dirty chunks that need remeshing
    pub fn dirty_chunks(&self) -> &[ChunkCoord] {
        &self.dirty_chunks
    }

    /// Mark a chunk as clean (after meshing)
    pub fn mark_chunk_clean(&mut self, coord: ChunkCoord) {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.mark_clean();
        }
        self.dirty_chunks.retain(|&c| c != coord);
    }

    /// Remove a chunk from the grid
    pub fn remove_chunk(&mut self, coord: ChunkCoord) -> Option<VoxelChunk> {
        self.dirty_chunks.retain(|&c| c != coord);
        self.chunks.remove(&coord)
    }

    /// Get total number of chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get total memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.chunks.values().map(|c| c.memory_usage()).sum()
    }

    /// Clear all chunks
    pub fn clear(&mut self) {
        self.chunks.clear();
        self.dirty_chunks.clear();
    }

    /// Get all chunk coordinates
    pub fn chunk_coords(&self) -> Vec<ChunkCoord> {
        self.chunks.keys().copied().collect()
    }
}

impl Default for VoxelGrid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_coord_conversion() {
        let pos = Vec3::new(50.0, 100.0, -30.0);
        let coord = ChunkCoord::from_world_pos(pos);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 3);
        assert_eq!(coord.z, -1);

        let world_pos = coord.to_world_pos();
        assert_eq!(world_pos.x, 32.0);
        assert_eq!(world_pos.y, 96.0);
        assert_eq!(world_pos.z, -32.0);
    }

    #[test]
    fn test_voxel_chunk_basic() {
        let coord = ChunkCoord::new(0, 0, 0);
        let mut chunk = VoxelChunk::new(coord);

        assert!(chunk.is_empty());
        assert!(!chunk.is_dirty());

        let voxel = Voxel::new(1.0, 1);
        chunk.set_voxel(IVec3::new(5, 10, 15), voxel);

        assert!(!chunk.is_empty());
        assert!(chunk.is_dirty());

        let retrieved = chunk.get_voxel(IVec3::new(5, 10, 15));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().density, 1.0);
        assert_eq!(retrieved.unwrap().material, 1);
    }

    #[test]
    fn test_voxel_grid() {
        let mut grid = VoxelGrid::new();

        let pos1 = Vec3::new(10.0, 20.0, 30.0);
        let pos2 = Vec3::new(50.0, 60.0, 70.0);

        grid.set_voxel(pos1, Voxel::new(1.0, 1));
        grid.set_voxel(pos2, Voxel::new(0.8, 2));

        assert_eq!(grid.chunk_count(), 2);
        assert_eq!(grid.dirty_chunks().len(), 2);

        let retrieved1 = grid.get_voxel(pos1);
        assert!(retrieved1.is_some());
        assert_eq!(retrieved1.unwrap().density, 1.0);

        let retrieved2 = grid.get_voxel(pos2);
        assert!(retrieved2.is_some());
        assert_eq!(retrieved2.unwrap().material, 2);
    }

    #[test]
    fn test_voxel_is_solid() {
        let solid = Voxel::new(1.0, 0);
        let empty = Voxel::new(0.0, 0);
        let partial = Voxel::new(0.5, 0);

        assert!(solid.is_solid());
        assert!(!empty.is_solid());
        assert!(!partial.is_solid());

        assert!(!solid.is_empty());
        assert!(empty.is_empty());
        assert!(!partial.is_empty());
    }

    #[test]
    fn test_chunk_neighbors() {
        let coord = ChunkCoord::new(5, 10, 15);
        let neighbors = coord.neighbors();

        assert_eq!(neighbors[0], ChunkCoord::new(6, 10, 15));
        assert_eq!(neighbors[1], ChunkCoord::new(4, 10, 15));
        assert_eq!(neighbors[2], ChunkCoord::new(5, 11, 15));
        assert_eq!(neighbors[3], ChunkCoord::new(5, 9, 15));
        assert_eq!(neighbors[4], ChunkCoord::new(5, 10, 16));
        assert_eq!(neighbors[5], ChunkCoord::new(5, 10, 14));
    }

    #[test]
    fn test_dirty_chunk_tracking() {
        let mut grid = VoxelGrid::new();
        let coord = ChunkCoord::new(0, 0, 0);

        grid.set_voxel(Vec3::new(5.0, 5.0, 5.0), Voxel::new(1.0, 0));
        assert_eq!(grid.dirty_chunks().len(), 1);

        grid.mark_chunk_clean(coord);
        assert_eq!(grid.dirty_chunks().len(), 0);
    }
}