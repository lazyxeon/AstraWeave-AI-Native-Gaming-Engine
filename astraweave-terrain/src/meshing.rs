//! Isosurface Generation using Dual Contouring
//!
//! This module implements the Dual Contouring algorithm to convert voxel data
//! into smooth polygon meshes. Dual Contouring is preferred over Marching Cubes
//! because it:
//! - Preserves sharp features better
//! - Produces fewer artifacts
//! - Generates more uniform triangles
//! - Handles hermite data (density + gradient)

use crate::voxel_data::{ChunkCoord, Voxel, VoxelChunk, CHUNK_SIZE};
use glam::{IVec3, Vec3};
use std::collections::HashMap;

/// Vertex data for mesh generation
#[derive(Debug, Clone, Copy)]
pub struct MeshVertex {
    /// Position in world space
    pub position: Vec3,
    /// Normal vector
    pub normal: Vec3,
    /// Material ID
    pub material: u16,
}

/// Generated mesh data from voxel chunk
#[derive(Debug, Clone)]
pub struct ChunkMesh {
    /// Chunk coordinate
    pub coord: ChunkCoord,
    /// Vertex data
    pub vertices: Vec<MeshVertex>,
    /// Index buffer (triangles)
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    /// Create an empty mesh
    pub fn empty(coord: ChunkCoord) -> Self {
        Self {
            coord,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Check if mesh is empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.vertices.len() * std::mem::size_of::<MeshVertex>()
            + self.indices.len() * std::mem::size_of::<u32>()
    }
}

/// Edge key for vertex deduplication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeKey {
    min: IVec3,
    max: IVec3,
}

impl EdgeKey {
    fn new(p1: IVec3, p2: IVec3) -> Self {
        if p1.x < p2.x || (p1.x == p2.x && p1.y < p2.y) || (p1.x == p2.x && p1.y == p2.y && p1.z < p2.z) {
            Self { min: p1, max: p2 }
        } else {
            Self { min: p2, max: p1 }
        }
    }
}

/// Dual Contouring mesh generator
pub struct DualContouring {
    /// Vertex cache for deduplication
    vertex_cache: HashMap<IVec3, u32>,
    /// Edge intersection cache
    edge_cache: HashMap<EdgeKey, Vec3>,
}

impl DualContouring {
    /// Create a new Dual Contouring generator
    pub fn new() -> Self {
        Self {
            vertex_cache: HashMap::new(),
            edge_cache: HashMap::new(),
        }
    }

    /// Generate mesh from voxel chunk
    pub fn generate_mesh(&mut self, chunk: &VoxelChunk) -> ChunkMesh {
        self.vertex_cache.clear();
        self.edge_cache.clear();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Process each cell in the chunk
        for z in 0..CHUNK_SIZE - 1 {
            for y in 0..CHUNK_SIZE - 1 {
                for x in 0..CHUNK_SIZE - 1 {
                    let cell_pos = IVec3::new(x, y, z);
                    self.process_cell(chunk, cell_pos, &mut vertices, &mut indices);
                }
            }
        }

        ChunkMesh {
            coord: chunk.coord(),
            vertices,
            indices,
        }
    }

    /// Process a single cell (8 voxels forming a cube)
    fn process_cell(
        &mut self,
        chunk: &VoxelChunk,
        cell_pos: IVec3,
        vertices: &mut Vec<MeshVertex>,
        indices: &mut Vec<u32>,
    ) {
        // Get the 8 corner voxels
        let corners = self.get_cell_corners(chunk, cell_pos);

        // Calculate cell configuration (which corners are solid)
        let mut config = 0u8;
        for (i, corner) in corners.iter().enumerate() {
            if corner.map_or(false, |v| v.is_solid()) {
                config |= 1 << i;
            }
        }

        // Skip if all corners are the same (no surface)
        if config == 0 || config == 255 {
            return;
        }

        // Find surface crossing edges and compute vertex position
        let vertex_pos = self.compute_vertex_position(chunk, cell_pos, &corners);
        let vertex_normal = self.compute_vertex_normal(chunk, cell_pos);
        let material = corners
            .iter()
            .flatten()
            .find(|v| v.is_solid())
            .map(|v| v.material)
            .unwrap_or(0);

        // Add vertex to cache
        let vertex_index = vertices.len() as u32;
        self.vertex_cache.insert(cell_pos, vertex_index);

        let world_pos = chunk.coord().to_world_pos() + vertex_pos;
        vertices.push(MeshVertex {
            position: world_pos,
            normal: vertex_normal,
            material,
        });

        // Generate triangles for this cell
        self.generate_cell_triangles(cell_pos, config, indices);
    }

    /// Get the 8 corner voxels of a cell
    fn get_cell_corners(&self, chunk: &VoxelChunk, cell_pos: IVec3) -> [Option<Voxel>; 8] {
        [
            chunk.get_voxel(cell_pos + IVec3::new(0, 0, 0)),
            chunk.get_voxel(cell_pos + IVec3::new(1, 0, 0)),
            chunk.get_voxel(cell_pos + IVec3::new(1, 1, 0)),
            chunk.get_voxel(cell_pos + IVec3::new(0, 1, 0)),
            chunk.get_voxel(cell_pos + IVec3::new(0, 0, 1)),
            chunk.get_voxel(cell_pos + IVec3::new(1, 0, 1)),
            chunk.get_voxel(cell_pos + IVec3::new(1, 1, 1)),
            chunk.get_voxel(cell_pos + IVec3::new(0, 1, 1)),
        ]
    }

    /// Compute vertex position using QEF (Quadratic Error Function) minimization
    /// Simplified version: use average of edge intersections
    fn compute_vertex_position(&mut self, chunk: &VoxelChunk, cell_pos: IVec3, corners: &[Option<Voxel>; 8]) -> Vec3 {
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        // Check all 12 edges of the cube
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), // Bottom face
            (4, 5), (5, 6), (6, 7), (7, 4), // Top face
            (0, 4), (1, 5), (2, 6), (3, 7), // Vertical edges
        ];

        for (i, j) in edges.iter() {
            if let (Some(v1), Some(v2)) = (corners[*i], corners[*j]) {
                // Check if edge crosses surface (one solid, one empty)
                if v1.is_solid() != v2.is_solid() {
                    let p1 = self.corner_offset(*i);
                    let p2 = self.corner_offset(*j);
                    
                    // Linear interpolation based on density
                    let t = if (v1.density - v2.density).abs() > 0.001 {
                        (0.5 - v1.density) / (v2.density - v1.density)
                    } else {
                        0.5
                    };
                    let t = t.clamp(0.0, 1.0);
                    
                    let intersection = p1 + (p2 - p1) * t;
                    sum += intersection;
                    count += 1;
                }
            }
        }

        if count > 0 {
            cell_pos.as_vec3() + sum / count as f32
        } else {
            // Fallback to cell center
            cell_pos.as_vec3() + Vec3::splat(0.5)
        }
    }

    /// Get offset for corner index (0-7)
    fn corner_offset(&self, index: usize) -> Vec3 {
        let x = if index & 1 != 0 { 1.0 } else { 0.0 };
        let y = if index & 2 != 0 { 1.0 } else { 0.0 };
        let z = if index & 4 != 0 { 1.0 } else { 0.0 };
        Vec3::new(x, y, z)
    }

    /// Compute vertex normal using central differences
    fn compute_vertex_normal(&self, chunk: &VoxelChunk, cell_pos: IVec3) -> Vec3 {
        let epsilon = 1.0;
        
        let dx = self.sample_density(chunk, cell_pos + IVec3::new(1, 0, 0))
            - self.sample_density(chunk, cell_pos - IVec3::new(1, 0, 0));
        let dy = self.sample_density(chunk, cell_pos + IVec3::new(0, 1, 0))
            - self.sample_density(chunk, cell_pos - IVec3::new(0, 1, 0));
        let dz = self.sample_density(chunk, cell_pos + IVec3::new(0, 0, 1))
            - self.sample_density(chunk, cell_pos - IVec3::new(0, 0, 1));

        let gradient = Vec3::new(dx, dy, dz);
        if gradient.length_squared() > 0.001 {
            gradient.normalize()
        } else {
            Vec3::Y // Default up
        }
    }

    /// Sample density at position
    fn sample_density(&self, chunk: &VoxelChunk, pos: IVec3) -> f32 {
        chunk.get_voxel(pos).map(|v| v.density).unwrap_or(0.0)
    }

    /// Generate triangles for a cell based on configuration
    fn generate_cell_triangles(&self, cell_pos: IVec3, config: u8, indices: &mut Vec<u32>) {
        // Simplified triangle generation based on configuration
        // In a full implementation, this would use edge tables like Marching Cubes
        // For now, we generate a simple quad for demonstration
        
        if let Some(&vertex_idx) = self.vertex_cache.get(&cell_pos) {
            // Check neighboring cells and create triangles
            // This is a simplified version - full DC would use edge-based triangulation
            
            // Generate triangles connecting to neighboring vertices
            let neighbors = [
                cell_pos + IVec3::new(1, 0, 0),
                cell_pos + IVec3::new(0, 1, 0),
                cell_pos + IVec3::new(0, 0, 1),
            ];

            for i in 0..neighbors.len() {
                let n1 = neighbors[i];
                let n2 = neighbors[(i + 1) % neighbors.len()];

                if let (Some(&idx1), Some(&idx2)) = (self.vertex_cache.get(&n1), self.vertex_cache.get(&n2)) {
                    // Create triangle
                    indices.push(vertex_idx);
                    indices.push(idx1);
                    indices.push(idx2);
                }
            }
        }
    }
}

impl Default for DualContouring {
    fn default() -> Self {
        Self::new()
    }
}

/// Async mesh generator for background processing
pub struct AsyncMeshGenerator {
    generator: DualContouring,
}

impl AsyncMeshGenerator {
    /// Create a new async mesh generator
    pub fn new() -> Self {
        Self {
            generator: DualContouring::new(),
        }
    }

    /// Generate mesh asynchronously
    pub async fn generate_mesh_async(&mut self, chunk: VoxelChunk) -> ChunkMesh {
        // In a real implementation, this would use tokio::spawn
        // For now, we just call the sync version
        self.generator.generate_mesh(&chunk)
    }

    /// Generate multiple meshes in parallel
    pub async fn generate_meshes_parallel(&mut self, chunks: Vec<VoxelChunk>) -> Vec<ChunkMesh> {
        // Use rayon for parallel processing
        use rayon::prelude::*;
        
        chunks
            .into_par_iter()
            .map(|chunk| {
                let mut gen = DualContouring::new();
                gen.generate_mesh(&chunk)
            })
            .collect()
    }
}

impl Default for AsyncMeshGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// LOD (Level of Detail) configuration
#[derive(Debug, Clone, Copy)]
pub struct LodConfig {
    /// Distance thresholds for each LOD level
    pub distances: [f32; 4],
    /// Simplification factors for each LOD level
    pub simplification: [f32; 4],
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            distances: [100.0, 250.0, 500.0, 1000.0],
            simplification: [1.0, 0.5, 0.25, 0.125],
        }
    }
}

/// LOD mesh generator
pub struct LodMeshGenerator {
    config: LodConfig,
    generators: Vec<DualContouring>,
}

impl LodMeshGenerator {
    /// Create a new LOD mesh generator
    pub fn new(config: LodConfig) -> Self {
        Self {
            config,
            generators: vec![DualContouring::new(); 4],
        }
    }

    /// Generate mesh with appropriate LOD based on distance
    pub fn generate_mesh_lod(&mut self, chunk: &VoxelChunk, distance: f32) -> ChunkMesh {
        let lod_level = self.select_lod_level(distance);
        self.generators[lod_level].generate_mesh(chunk)
    }

    /// Select LOD level based on distance
    fn select_lod_level(&self, distance: f32) -> usize {
        for (i, &threshold) in self.config.distances.iter().enumerate() {
            if distance < threshold {
                return i;
            }
        }
        3 // Furthest LOD
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel_data::Voxel;

    #[test]
    fn test_dual_contouring_empty_chunk() {
        let coord = ChunkCoord::new(0, 0, 0);
        let chunk = VoxelChunk::new(coord);
        
        let mut dc = DualContouring::new();
        let mesh = dc.generate_mesh(&chunk);
        
        assert!(mesh.is_empty());
    }

    #[test]
    fn test_dual_contouring_single_voxel() {
        let coord = ChunkCoord::new(0, 0, 0);
        let mut chunk = VoxelChunk::new(coord);
        
        // Set a single solid voxel
        chunk.set_voxel(IVec3::new(5, 5, 5), Voxel::new(1.0, 1));
        
        let mut dc = DualContouring::new();
        let mesh = dc.generate_mesh(&chunk);
        
        // Should generate some vertices
        assert!(!mesh.vertices.is_empty());
    }

    #[test]
    fn test_mesh_vertex_creation() {
        let vertex = MeshVertex {
            position: Vec3::new(1.0, 2.0, 3.0),
            normal: Vec3::Y,
            material: 5,
        };
        
        assert_eq!(vertex.position.x, 1.0);
        assert_eq!(vertex.normal, Vec3::Y);
        assert_eq!(vertex.material, 5);
    }

    #[test]
    fn test_lod_selection() {
        let config = LodConfig::default();
        let mut lod_gen = LodMeshGenerator::new(config);
        
        assert_eq!(lod_gen.select_lod_level(50.0), 0);
        assert_eq!(lod_gen.select_lod_level(200.0), 1);
        assert_eq!(lod_gen.select_lod_level(400.0), 2);
        assert_eq!(lod_gen.select_lod_level(1500.0), 3);
    }

    #[test]
    fn test_edge_key_ordering() {
        let p1 = IVec3::new(0, 0, 0);
        let p2 = IVec3::new(1, 0, 0);
        
        let key1 = EdgeKey::new(p1, p2);
        let key2 = EdgeKey::new(p2, p1);
        
        assert_eq!(key1, key2);
    }
}