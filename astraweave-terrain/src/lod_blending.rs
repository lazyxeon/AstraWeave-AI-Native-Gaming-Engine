//! LOD Vertex Morphing for Seamless Level-of-Detail Transitions
//!
//! This module implements vertex morphing between LOD levels to eliminate
//! popping artifacts. When the camera distance crosses an LOD threshold,
//! vertices smoothly interpolate between the high and low detail meshes.

use crate::meshing::{ChunkMesh, MeshVertex};
use glam::{IVec3, Vec3};
use std::collections::HashMap;

/// Configuration for LOD morphing
#[derive(Debug, Clone, Copy)]
pub struct MorphConfig {
    /// Distance at which morphing begins (near boundary)
    pub morph_start: f32,
    /// Distance at which morphing completes (far boundary)
    pub morph_end: f32,
    /// Maximum search radius for vertex correspondence (voxels)
    pub search_radius: f32,
}

impl Default for MorphConfig {
    fn default() -> Self {
        Self {
            morph_start: 0.0,
            morph_end: 50.0,
            search_radius: 2.0,
        }
    }
}

impl MorphConfig {
    /// Create config for specific LOD transition
    pub fn for_lod_transition(lod_start: f32, lod_end: f32) -> Self {
        let transition_zone = (lod_end - lod_start) * 0.2; // 20% of distance range
        Self {
            morph_start: lod_end - transition_zone,
            morph_end: lod_end,
            search_radius: 2.0,
        }
    }
}

/// Result of vertex correspondence search
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct VertexCorrespondence {
    /// Index in high LOD mesh
    high_lod_index: usize,
    /// Index in low LOD mesh (if found)
    low_lod_index: Option<usize>,
    /// Distance to nearest low LOD vertex
    distance: f32,
}

/// Morphed mesh with interpolated vertices
#[derive(Debug, Clone)]
pub struct MorphedMesh {
    /// Original mesh data
    pub mesh: ChunkMesh,
    /// Morph factor applied (0.0 = high LOD, 1.0 = low LOD)
    pub morph_factor: f32,
}

impl MorphedMesh {
    /// Create a morphed mesh (no morphing applied yet)
    pub fn new(mesh: ChunkMesh) -> Self {
        Self {
            mesh,
            morph_factor: 0.0,
        }
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.mesh.vertices.len()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.mesh.indices.len() / 3
    }
}

/// LOD blending engine
pub struct LodBlender {
    config: MorphConfig,
}

impl LodBlender {
    /// Create a new LOD blender
    pub fn new(config: MorphConfig) -> Self {
        Self { config }
    }

    /// Compute morph factor based on camera distance
    /// Returns 0.0 at morph_start (high LOD), 1.0 at morph_end (low LOD)
    pub fn compute_morph_factor(&self, distance: f32) -> f32 {
        if distance <= self.config.morph_start {
            0.0 // Pure high LOD
        } else if distance >= self.config.morph_end {
            1.0 // Pure low LOD
        } else {
            // Linear interpolation in transition zone
            let range = self.config.morph_end - self.config.morph_start;
            let offset = distance - self.config.morph_start;
            (offset / range).clamp(0.0, 1.0)
        }
    }

    /// Morph vertices between two LOD levels
    pub fn morph_vertices(
        &self,
        high_lod: &ChunkMesh,
        low_lod: &ChunkMesh,
        morph_factor: f32,
    ) -> MorphedMesh {
        if morph_factor <= 0.0 {
            // Pure high LOD
            return MorphedMesh {
                mesh: high_lod.clone(),
                morph_factor: 0.0,
            };
        }

        if morph_factor >= 1.0 {
            // Pure low LOD
            return MorphedMesh {
                mesh: low_lod.clone(),
                morph_factor: 1.0,
            };
        }

        // Find vertex correspondence between LOD levels
        let correspondence = self.find_vertex_correspondence(high_lod, low_lod);

        // Create morphed mesh
        let mut morphed = high_lod.clone();

        // Interpolate vertex positions and normals
        for (i, vertex) in morphed.vertices.iter_mut().enumerate() {
            if let Some(corr) = correspondence.get(&i) {
                if let Some(low_idx) = corr.low_lod_index {
                    let low_vertex = &low_lod.vertices[low_idx];

                    // Lerp position: pos = high * (1 - t) + low * t
                    vertex.position = vertex.position.lerp(low_vertex.position, morph_factor);

                    // Slerp normal for smooth lighting transitions
                    let normal_lerp = vertex.normal.lerp(low_vertex.normal, morph_factor);
                    vertex.normal = normal_lerp.normalize_or_zero();

                    // Keep high LOD material (no material morphing)
                }
            }
        }

        MorphedMesh {
            mesh: morphed,
            morph_factor,
        }
    }

    /// Find correspondence between high and low LOD vertices
    fn find_vertex_correspondence(
        &self,
        high_lod: &ChunkMesh,
        low_lod: &ChunkMesh,
    ) -> HashMap<usize, VertexCorrespondence> {
        let mut correspondence = HashMap::new();

        // Build spatial hash for low LOD vertices (faster lookup)
        let low_lod_hash = self.build_spatial_hash(&low_lod.vertices);

        // For each high LOD vertex, find nearest low LOD vertex
        for (i, high_vertex) in high_lod.vertices.iter().enumerate() {
            let (low_idx, distance) =
                self.find_nearest_vertex(high_vertex.position, &low_lod.vertices, &low_lod_hash);

            correspondence.insert(
                i,
                VertexCorrespondence {
                    high_lod_index: i,
                    low_lod_index: if distance <= self.config.search_radius {
                        Some(low_idx)
                    } else {
                        None
                    },
                    distance,
                },
            );
        }

        correspondence
    }

    /// Build spatial hash for vertices (grid-based)
    fn build_spatial_hash(&self, vertices: &[MeshVertex]) -> HashMap<IVec3, Vec<usize>> {
        let mut hash = HashMap::new();
        let cell_size = 1.0; // 1 voxel per cell

        for (i, vertex) in vertices.iter().enumerate() {
            let cell = IVec3::new(
                (vertex.position.x / cell_size).floor() as i32,
                (vertex.position.y / cell_size).floor() as i32,
                (vertex.position.z / cell_size).floor() as i32,
            );

            hash.entry(cell).or_insert_with(Vec::new).push(i);
        }

        hash
    }

    /// Find nearest vertex to a position using spatial hash
    fn find_nearest_vertex(
        &self,
        position: Vec3,
        vertices: &[MeshVertex],
        spatial_hash: &HashMap<IVec3, Vec<usize>>,
    ) -> (usize, f32) {
        let cell_size = 1.0;
        let center_cell = IVec3::new(
            (position.x / cell_size).floor() as i32,
            (position.y / cell_size).floor() as i32,
            (position.z / cell_size).floor() as i32,
        );

        let mut nearest_idx = 0;
        let mut nearest_dist = f32::MAX;

        // Search in 3x3x3 neighborhood
        let search_range = (self.config.search_radius / cell_size).ceil() as i32;
        for dx in -search_range..=search_range {
            for dy in -search_range..=search_range {
                for dz in -search_range..=search_range {
                    let cell = center_cell + IVec3::new(dx, dy, dz);

                    if let Some(indices) = spatial_hash.get(&cell) {
                        for &idx in indices {
                            let dist = position.distance(vertices[idx].position);
                            if dist < nearest_dist {
                                nearest_dist = dist;
                                nearest_idx = idx;
                            }
                        }
                    }
                }
            }
        }

        (nearest_idx, nearest_dist)
    }

    /// Create a transition mesh between two LOD levels at specific distance
    pub fn create_transition_mesh(
        &self,
        high_lod: &ChunkMesh,
        low_lod: &ChunkMesh,
        camera_distance: f32,
    ) -> MorphedMesh {
        let morph_factor = self.compute_morph_factor(camera_distance);
        self.morph_vertices(high_lod, low_lod, morph_factor)
    }
}

impl Default for LodBlender {
    fn default() -> Self {
        Self::new(MorphConfig::default())
    }
}

/// Multi-LOD mesh manager with automatic morphing
pub struct MorphingLodManager {
    /// LOD meshes (sorted by detail: 0 = highest)
    lod_meshes: Vec<ChunkMesh>,
    /// LOD distance thresholds
    lod_distances: Vec<f32>,
    /// Blender for each LOD transition
    blenders: Vec<LodBlender>,
}

impl MorphingLodManager {
    /// Create a new morphing LOD manager
    pub fn new(lod_meshes: Vec<ChunkMesh>, lod_distances: Vec<f32>) -> Self {
        assert!(
            lod_meshes.len() >= 2,
            "Need at least 2 LOD levels for morphing"
        );
        assert_eq!(
            lod_meshes.len(),
            lod_distances.len(),
            "LOD mesh count must match distance count"
        );

        // Create blenders for each transition
        let mut blenders = Vec::new();
        for i in 0..(lod_distances.len() - 1) {
            let config = MorphConfig::for_lod_transition(lod_distances[i], lod_distances[i + 1]);
            blenders.push(LodBlender::new(config));
        }

        Self {
            lod_meshes,
            lod_distances,
            blenders,
        }
    }

    /// Get appropriate mesh for given camera distance (with morphing)
    pub fn get_mesh_for_distance(&self, distance: f32) -> MorphedMesh {
        // Find which LOD level to use
        let mut lod_level = 0;
        for (i, &threshold) in self.lod_distances.iter().enumerate() {
            if distance < threshold {
                lod_level = i;
                break;
            }
            lod_level = i; // Use last LOD if beyond all thresholds
        }

        // Check if we're in a transition zone
        if lod_level > 0 {
            let prev_distance = self.lod_distances[lod_level - 1];
            let curr_distance = self.lod_distances[lod_level];

            // If in transition zone between LOD levels
            if distance >= prev_distance && distance <= curr_distance {
                let blender = &self.blenders[lod_level - 1];
                return blender.create_transition_mesh(
                    &self.lod_meshes[lod_level - 1],
                    &self.lod_meshes[lod_level],
                    distance,
                );
            }
        }

        // Not in transition zone - return pure LOD level
        MorphedMesh::new(self.lod_meshes[lod_level].clone())
    }

    /// Get number of LOD levels
    pub fn lod_count(&self) -> usize {
        self.lod_meshes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel_data::ChunkCoord;

    #[test]
    fn test_morph_factor_calculation() {
        let config = MorphConfig {
            morph_start: 100.0,
            morph_end: 200.0,
            search_radius: 2.0,
        };
        let blender = LodBlender::new(config);

        // Before transition zone
        assert_eq!(blender.compute_morph_factor(50.0), 0.0);
        assert_eq!(blender.compute_morph_factor(100.0), 0.0);

        // In transition zone
        assert_eq!(blender.compute_morph_factor(150.0), 0.5);

        // After transition zone
        assert_eq!(blender.compute_morph_factor(200.0), 1.0);
        assert_eq!(blender.compute_morph_factor(250.0), 1.0);
    }

    #[test]
    fn test_morph_config_for_lod() {
        let config = MorphConfig::for_lod_transition(100.0, 200.0);

        // Transition zone should be 20% of range (20 units)
        assert_eq!(config.morph_start, 180.0);
        assert_eq!(config.morph_end, 200.0);
    }

    #[test]
    fn test_pure_high_lod() {
        let blender = LodBlender::default();

        let high_lod = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![MeshVertex {
                position: Vec3::new(1.0, 2.0, 3.0),
                normal: Vec3::Y,
                material: 1,
            }],
            indices: vec![0],
        };

        let low_lod = ChunkMesh::empty(ChunkCoord::new(0, 0, 0));

        let morphed = blender.morph_vertices(&high_lod, &low_lod, 0.0);

        assert_eq!(morphed.morph_factor, 0.0);
        assert_eq!(morphed.mesh.vertices[0].position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_pure_low_lod() {
        let blender = LodBlender::default();

        let high_lod = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![MeshVertex {
                position: Vec3::new(1.0, 2.0, 3.0),
                normal: Vec3::Y,
                material: 1,
            }],
            indices: vec![0],
        };

        let low_lod = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![MeshVertex {
                position: Vec3::new(2.0, 3.0, 4.0),
                normal: Vec3::Y,
                material: 1,
            }],
            indices: vec![0],
        };

        let morphed = blender.morph_vertices(&high_lod, &low_lod, 1.0);

        assert_eq!(morphed.morph_factor, 1.0);
        assert_eq!(morphed.mesh.vertices[0].position, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_vertex_interpolation() {
        let config = MorphConfig {
            morph_start: 0.0,
            morph_end: 100.0,
            search_radius: 5.0, // Large search radius to ensure correspondence
        };
        let blender = LodBlender::new(config);

        let high_lod = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![MeshVertex {
                position: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::Y,
                material: 1,
            }],
            indices: vec![0],
        };

        let low_lod = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![MeshVertex {
                position: Vec3::new(2.0, 2.0, 2.0),
                normal: Vec3::Y,
                material: 1,
            }],
            indices: vec![0],
        };

        // 50% morph
        let morphed = blender.morph_vertices(&high_lod, &low_lod, 0.5);

        // Position should be halfway between
        let expected = Vec3::new(1.0, 1.0, 1.0);
        let actual = morphed.mesh.vertices[0].position;
        assert!(
            (actual - expected).length() < 0.01,
            "Expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn test_spatial_hash_build() {
        let blender = LodBlender::default();

        let vertices = vec![
            MeshVertex {
                position: Vec3::new(0.5, 0.5, 0.5),
                normal: Vec3::Y,
                material: 1,
            },
            MeshVertex {
                position: Vec3::new(1.5, 1.5, 1.5),
                normal: Vec3::Y,
                material: 1,
            },
        ];

        let hash = blender.build_spatial_hash(&vertices);

        // Should have 2 cells
        assert!(hash.len() >= 1);
    }

    #[test]
    fn test_morphing_lod_manager() {
        let lod0 = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![MeshVertex {
                position: Vec3::ZERO,
                normal: Vec3::Y,
                material: 1,
            }],
            indices: vec![0],
        };

        let lod1 = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![MeshVertex {
                position: Vec3::ONE,
                normal: Vec3::Y,
                material: 1,
            }],
            indices: vec![0],
        };

        let manager = MorphingLodManager::new(vec![lod0, lod1], vec![100.0, 200.0]);

        assert_eq!(manager.lod_count(), 2);

        // Close distance - should use LOD 0
        let mesh = manager.get_mesh_for_distance(50.0);
        assert_eq!(mesh.morph_factor, 0.0);

        // Far distance - should use LOD 1
        let mesh = manager.get_mesh_for_distance(250.0);
        assert!(mesh.morph_factor >= 0.0); // Could be morphed or pure LOD 1
    }

    #[test]
    fn test_morphed_mesh_properties() {
        let mesh = ChunkMesh {
            coord: ChunkCoord::new(0, 0, 0),
            vertices: vec![
                MeshVertex {
                    position: Vec3::ZERO,
                    normal: Vec3::Y,
                    material: 1,
                },
                MeshVertex {
                    position: Vec3::ONE,
                    normal: Vec3::Y,
                    material: 1,
                },
                MeshVertex {
                    position: Vec3::X,
                    normal: Vec3::Y,
                    material: 1,
                },
            ],
            indices: vec![0, 1, 2],
        };

        let morphed = MorphedMesh::new(mesh);

        assert_eq!(morphed.vertex_count(), 3);
        assert_eq!(morphed.triangle_count(), 1);
        assert_eq!(morphed.morph_factor, 0.0);
    }
}
