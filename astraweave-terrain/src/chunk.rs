//! Terrain chunk management and streaming

use crate::{BiomeType, Heightmap};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a terrain chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkId {
    pub x: i32,
    pub z: i32,
}

impl ChunkId {
    /// Create a new chunk ID
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Convert world position to chunk ID
    pub fn from_world_pos(world_pos: Vec3, chunk_size: f32) -> Self {
        let chunk_x = (world_pos.x / chunk_size).floor() as i32;
        let chunk_z = (world_pos.z / chunk_size).floor() as i32;
        Self::new(chunk_x, chunk_z)
    }

    /// Get the world position of the chunk's origin (bottom-left corner)
    pub fn to_world_pos(self, chunk_size: f32) -> Vec3 {
        Vec3::new(self.x as f32 * chunk_size, 0.0, self.z as f32 * chunk_size)
    }

    /// Get the center world position of the chunk
    pub fn to_center_pos(self, chunk_size: f32) -> Vec3 {
        let origin = self.to_world_pos(chunk_size);
        origin + Vec3::new(chunk_size * 0.5, 0.0, chunk_size * 0.5)
    }

    /// Get all chunk IDs within a given radius
    pub fn get_chunks_in_radius(center: Vec3, radius: u32, chunk_size: f32) -> Vec<ChunkId> {
        let center_chunk = ChunkId::from_world_pos(center, chunk_size);
        let mut chunks = Vec::new();

        let radius = radius as i32;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                chunks.push(ChunkId::new(center_chunk.x + dx, center_chunk.z + dz));
            }
        }

        chunks
    }

    /// Calculate distance to another chunk (in chunk units)
    pub fn distance_to(self, other: ChunkId) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }
}

/// A single terrain chunk containing heightmap and biome data
#[derive(Debug, Clone)]
pub struct TerrainChunk {
    id: ChunkId,
    heightmap: Heightmap,
    biome_map: Vec<BiomeType>,
    mesh_dirty: bool,
}

impl TerrainChunk {
    /// Create a new terrain chunk
    pub fn new(id: ChunkId, heightmap: Heightmap, biome_map: Vec<BiomeType>) -> Self {
        Self {
            id,
            heightmap,
            biome_map,
            mesh_dirty: true,
        }
    }

    /// Get the chunk ID
    pub fn id(&self) -> ChunkId {
        self.id
    }

    /// Get the heightmap
    pub fn heightmap(&self) -> &Heightmap {
        &self.heightmap
    }

    /// Get the biome map
    pub fn biome_map(&self) -> &[BiomeType] {
        &self.biome_map
    }

    /// Check if the mesh needs to be regenerated
    pub fn is_mesh_dirty(&self) -> bool {
        self.mesh_dirty
    }

    /// Mark the mesh as clean (after regeneration)
    pub fn mark_mesh_clean(&mut self) {
        self.mesh_dirty = false;
    }

    /// Apply hydraulic erosion to the chunk
    pub fn apply_erosion(&mut self, strength: f32) -> anyhow::Result<()> {
        self.heightmap.apply_hydraulic_erosion(strength)?;
        self.mesh_dirty = true;
        Ok(())
    }

    /// Get the height at a world position within this chunk
    pub fn get_height_at_world_pos(&self, world_pos: Vec3, chunk_size: f32) -> Option<f32> {
        let chunk_origin = self.id.to_world_pos(chunk_size);
        let local_pos = world_pos - chunk_origin;

        // Check if position is within chunk bounds
        if local_pos.x < 0.0
            || local_pos.x >= chunk_size
            || local_pos.z < 0.0
            || local_pos.z >= chunk_size
        {
            return None;
        }

        // Convert to heightmap coordinates
        let resolution = self.heightmap.resolution() as f32;
        let u = (local_pos.x / chunk_size) * (resolution - 1.0);
        let v = (local_pos.z / chunk_size) * (resolution - 1.0);

        Some(self.heightmap.sample_bilinear(u, v))
    }

    /// Get the biome at a world position within this chunk
    pub fn get_biome_at_world_pos(&self, world_pos: Vec3, chunk_size: f32) -> Option<BiomeType> {
        let chunk_origin = self.id.to_world_pos(chunk_size);
        let local_pos = world_pos - chunk_origin;

        // Check if position is within chunk bounds
        if local_pos.x < 0.0
            || local_pos.x >= chunk_size
            || local_pos.z < 0.0
            || local_pos.z >= chunk_size
        {
            return None;
        }

        // Convert to biome map coordinates
        let resolution = self.heightmap.resolution() as f32;
        let u = (local_pos.x / chunk_size) * (resolution - 1.0);
        let v = (local_pos.z / chunk_size) * (resolution - 1.0);

        let x = u.round() as usize;
        let z = v.round() as usize;
        let index = z as usize * self.heightmap.resolution() as usize + x as usize;

        self.biome_map.get(index).copied()
    }
}

/// Manages loading, unloading, and caching of terrain chunks
#[derive(Debug)]
pub struct ChunkManager {
    chunks: HashMap<ChunkId, TerrainChunk>,
    chunk_size: f32,
    #[allow(dead_code)]
    heightmap_resolution: u32, // currently unused
    max_loaded_chunks: usize,
}

impl ChunkManager {
    /// Create a new chunk manager
    pub fn new(chunk_size: f32, heightmap_resolution: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size,
            heightmap_resolution,
            max_loaded_chunks: 256, // Limit memory usage
        }
    }

    /// Add a chunk to the manager
    pub fn add_chunk(&mut self, chunk: TerrainChunk) {
        // If we're at capacity, remove the oldest chunk
        if self.chunks.len() >= self.max_loaded_chunks {
            // Simple LRU: remove a random chunk (in production, use proper LRU)
            if let Some(&chunk_id) = self.chunks.keys().next() {
                self.chunks.remove(&chunk_id);
            }
        }

        self.chunks.insert(chunk.id(), chunk);
    }

    /// Get a chunk by ID
    pub fn get_chunk(&self, chunk_id: ChunkId) -> Option<&TerrainChunk> {
        self.chunks.get(&chunk_id)
    }

    /// Get a mutable chunk by ID
    pub fn get_chunk_mut(&mut self, chunk_id: ChunkId) -> Option<&mut TerrainChunk> {
        self.chunks.get_mut(&chunk_id)
    }

    /// Check if a chunk is loaded
    pub fn has_chunk(&self, chunk_id: ChunkId) -> bool {
        self.chunks.contains_key(&chunk_id)
    }

    /// Get all chunk IDs within a radius of a center position
    pub fn get_chunks_in_radius(&self, center: Vec3, radius: u32) -> Vec<ChunkId> {
        ChunkId::get_chunks_in_radius(center, radius, self.chunk_size)
    }

    /// Unload chunks that are too far from the center
    pub fn unload_distant_chunks(&mut self, center: Vec3, max_radius: u32) {
        let center_chunk = ChunkId::from_world_pos(center, self.chunk_size);
        let max_distance = max_radius as f32;

        let to_remove: Vec<ChunkId> = self
            .chunks
            .keys()
            .filter(|&&chunk_id| chunk_id.distance_to(center_chunk) > max_distance)
            .copied()
            .collect();

        for chunk_id in to_remove {
            self.chunks.remove(&chunk_id);
        }
    }

    /// Get the height at a world position by finding the appropriate chunk
    pub fn get_height_at_world_pos(&self, world_pos: Vec3) -> Option<f32> {
        let chunk_id = ChunkId::from_world_pos(world_pos, self.chunk_size);
        self.get_chunk(chunk_id)?
            .get_height_at_world_pos(world_pos, self.chunk_size)
    }

    /// Get the biome at a world position by finding the appropriate chunk
    pub fn get_biome_at_world_pos(&self, world_pos: Vec3) -> Option<BiomeType> {
        let chunk_id = ChunkId::from_world_pos(world_pos, self.chunk_size);
        self.get_chunk(chunk_id)?
            .get_biome_at_world_pos(world_pos, self.chunk_size)
    }

    /// Get all loaded chunk IDs
    pub fn loaded_chunks(&self) -> Vec<ChunkId> {
        self.chunks.keys().copied().collect()
    }

    /// Get the total number of loaded chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Set the maximum number of loaded chunks
    pub fn set_max_loaded_chunks(&mut self, max_chunks: usize) {
        self.max_loaded_chunks = max_chunks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HeightmapConfig;

    #[test]
    fn test_chunk_id_conversion() {
        let world_pos = Vec3::new(100.0, 0.0, 200.0);
        let chunk_size = 256.0;

        let chunk_id = ChunkId::from_world_pos(world_pos, chunk_size);
        let back_to_world = chunk_id.to_world_pos(chunk_size);

        assert_eq!(chunk_id, ChunkId::new(0, 0));
        assert_eq!(back_to_world, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_chunk_radius() {
        let center = Vec3::new(128.0, 0.0, 128.0);
        let chunks = ChunkId::get_chunks_in_radius(center, 1, 256.0);

        assert_eq!(chunks.len(), 9); // 3x3 grid
        assert!(chunks.contains(&ChunkId::new(0, 0)));
        assert!(chunks.contains(&ChunkId::new(-1, -1)));
        assert!(chunks.contains(&ChunkId::new(1, 1)));
    }

    #[test]
    fn test_chunk_manager() {
        let mut manager = ChunkManager::new(256.0, 64);

        let chunk_id = ChunkId::new(0, 0);
        let heightmap = Heightmap::new(HeightmapConfig::default()).unwrap();
        let biome_map = vec![BiomeType::Grassland; 64 * 64];
        let chunk = TerrainChunk::new(chunk_id, heightmap, biome_map);

        manager.add_chunk(chunk);

        assert!(manager.has_chunk(chunk_id));
        assert_eq!(manager.chunk_count(), 1);
    }
}
