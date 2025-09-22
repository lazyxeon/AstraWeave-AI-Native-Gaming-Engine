//! Vegetation and object scatter system

use crate::{Biome, BiomeConfig, TerrainChunk, ChunkId};
use astraweave_gameplay::{ResourceNode, spawn_resources};
use glam::Vec3;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

/// A placed vegetation instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegetationInstance {
    /// World position
    pub position: Vec3,
    /// Rotation in radians around Y axis
    pub rotation: f32,
    /// Scale multiplier
    pub scale: f32,
    /// Vegetation type name
    pub vegetation_type: String,
    /// Model path for rendering
    pub model_path: String,
}

/// A scatter pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScatterConfig {
    /// Use Poisson disk sampling for natural distribution
    pub use_poisson_disk: bool,
    /// Minimum distance between objects (for Poisson disk)
    pub min_distance: f32,
    /// Maximum slope allowed (degrees)
    pub max_slope: f32,
    /// Height range filter (min, max)
    pub height_filter: Option<(f32, f32)>,
    /// Random seed offset for this scatter type
    pub seed_offset: u64,
}

impl Default for ScatterConfig {
    fn default() -> Self {
        Self {
            use_poisson_disk: true,
            min_distance: 2.0,
            max_slope: 45.0,
            height_filter: None,
            seed_offset: 0,
        }
    }
}

/// Vegetation scatter system that places objects based on biome rules
pub struct VegetationScatter {
    config: ScatterConfig,
}

impl VegetationScatter {
    /// Create a new vegetation scatter system
    pub fn new(config: ScatterConfig) -> Self {
        Self { config }
    }

    /// Generate vegetation instances for a terrain chunk
    pub fn scatter_vegetation(
        &self,
        chunk: &TerrainChunk,
        chunk_size: f32,
        biome_config: &BiomeConfig,
        seed: u64,
    ) -> anyhow::Result<Vec<VegetationInstance>> {
        let mut instances = Vec::new();
        let _chunk_origin = chunk.id().to_world_pos(chunk_size);
        
        if biome_config.vegetation.vegetation_types.is_empty() {
            return Ok(instances);
        }

        // Calculate approximate number of vegetation instances
        let chunk_area = chunk_size * chunk_size;
        let target_count = (chunk_area * biome_config.vegetation.density) as usize;

        if target_count == 0 {
            return Ok(instances);
        }

        let mut rng = rand::rngs::StdRng::seed_from_u64(seed + self.config.seed_offset);

        if self.config.use_poisson_disk {
            instances = self.generate_poisson_disk_scatter(
                chunk,
                chunk_size,
                biome_config,
                &mut rng,
                target_count,
            )?;
        } else {
            instances = self.generate_random_scatter(
                chunk,
                chunk_size,
                biome_config,
                &mut rng,
                target_count,
            )?;
        }

        Ok(instances)
    }

    /// Generate scatter using Poisson disk sampling for natural distribution
    fn generate_poisson_disk_scatter(
        &self,
        chunk: &TerrainChunk,
        chunk_size: f32,
        biome_config: &BiomeConfig,
        rng: &mut rand::rngs::StdRng,
        target_count: usize,
    ) -> anyhow::Result<Vec<VegetationInstance>> {
        let mut instances = Vec::new();
        let chunk_origin = chunk.id().to_world_pos(chunk_size);
        let min_dist = self.config.min_distance;
        
        // Simple Poisson disk sampling using rejection method
        let max_attempts = target_count * 30; // Safety limit
        let mut attempts = 0;

        while instances.len() < target_count && attempts < max_attempts {
            attempts += 1;

            // Generate random position
            let local_x = rng.random::<f32>() * chunk_size;
            let local_z = rng.random::<f32>() * chunk_size;
            let mut world_pos = Vec3::new(
                chunk_origin.x + local_x,
                0.0,
                chunk_origin.z + local_z,
            );

            // Get height and biome at this position
            if let Some(height) = chunk.get_height_at_world_pos(world_pos, chunk_size) {
                world_pos.y = height;

                // Check height filter
                if let Some((min_height, max_height)) = self.config.height_filter {
                    if height < min_height || height > max_height {
                        continue;
                    }
                }

                // Check slope (simplified using nearby height samples)
                let slope = self.estimate_slope(chunk, world_pos, chunk_size);
                if slope > self.config.max_slope {
                    continue;
                }

                // Check minimum distance to existing instances
                let too_close = instances.iter().any(|instance: &VegetationInstance| {
                    let distance = (instance.position - world_pos).length();
                    distance < min_dist
                });

                if too_close {
                    continue;
                }

                // Select vegetation type
                if let Some(vegetation_instance) = self.create_vegetation_instance(
                    world_pos,
                    biome_config,
                    rng,
                    slope,
                )? {
                    instances.push(vegetation_instance);
                }
            }
        }

        Ok(instances)
    }

    /// Generate scatter using simple random placement
    fn generate_random_scatter(
        &self,
        chunk: &TerrainChunk,
        chunk_size: f32,
        biome_config: &BiomeConfig,
        rng: &mut rand::rngs::StdRng,
        target_count: usize,
    ) -> anyhow::Result<Vec<VegetationInstance>> {
        let mut instances = Vec::new();
        let chunk_origin = chunk.id().to_world_pos(chunk_size);

        for _ in 0..target_count {
            // Generate random position
            let local_x = rng.random::<f32>() * chunk_size;
            let local_z = rng.random::<f32>() * chunk_size;
            let mut world_pos = Vec3::new(
                chunk_origin.x + local_x,
                0.0,
                chunk_origin.z + local_z,
            );

            // Get height and biome at this position
            if let Some(height) = chunk.get_height_at_world_pos(world_pos, chunk_size) {
                world_pos.y = height;

                // Check height filter
                if let Some((min_height, max_height)) = self.config.height_filter {
                    if height < min_height || height > max_height {
                        continue;
                    }
                }

                // Check slope
                let slope = self.estimate_slope(chunk, world_pos, chunk_size);
                if slope > self.config.max_slope {
                    continue;
                }

                // Create vegetation instance
                if let Some(vegetation_instance) = self.create_vegetation_instance(
                    world_pos,
                    biome_config,
                    rng,
                    slope,
                )? {
                    instances.push(vegetation_instance);
                }
            }
        }

        Ok(instances)
    }

    /// Estimate slope at a position using nearby height samples
    fn estimate_slope(
        &self,
        chunk: &TerrainChunk,
        world_pos: Vec3,
        chunk_size: f32,
    ) -> f32 {
        let offset = 1.0; // Sample distance
        
        let height_center = world_pos.y;
        let height_x = chunk.get_height_at_world_pos(
            world_pos + Vec3::new(offset, 0.0, 0.0),
            chunk_size,
        ).unwrap_or(height_center);
        let height_z = chunk.get_height_at_world_pos(
            world_pos + Vec3::new(0.0, 0.0, offset),
            chunk_size,
        ).unwrap_or(height_center);

        let dx = height_x - height_center;
        let dz = height_z - height_center;
        let slope_radians = (dx * dx + dz * dz).sqrt().atan2(offset);
        
        slope_radians.to_degrees()
    }

    /// Create a vegetation instance with appropriate type and scaling
    fn create_vegetation_instance(
        &self,
        position: Vec3,
        biome_config: &BiomeConfig,
        rng: &mut rand::rngs::StdRng,
        slope: f32,
    ) -> anyhow::Result<Option<VegetationInstance>> {
        // Filter vegetation types by slope tolerance
        let suitable_types: Vec<_> = biome_config.vegetation.vegetation_types
            .iter()
            .filter(|veg_type| slope <= veg_type.slope_tolerance)
            .collect();

        if suitable_types.is_empty() {
            return Ok(None);
        }

        // Weighted random selection
        let total_weight: f32 = suitable_types.iter().map(|vt| vt.weight).sum();
        if total_weight <= 0.0 {
            return Ok(None);
        }

        let random_value = rng.random::<f32>() * total_weight;
        let mut accumulated_weight = 0.0;
        let mut selected_type = suitable_types[0];

        for veg_type in &suitable_types {
            accumulated_weight += veg_type.weight;
            if random_value <= accumulated_weight {
                selected_type = veg_type;
                break;
            }
        }

        // Generate scale
        let scale = if biome_config.vegetation.random_rotation {
            rng.random_range(selected_type.scale_range.0..=selected_type.scale_range.1)
        } else {
            (selected_type.scale_range.0 + selected_type.scale_range.1) * 0.5
        };

        // Generate rotation
        let rotation = if biome_config.vegetation.random_rotation {
            rng.random::<f32>() * std::f32::consts::TAU
        } else {
            0.0
        };

        Ok(Some(VegetationInstance {
            position,
            rotation,
            scale,
            vegetation_type: selected_type.name.clone(),
            model_path: selected_type.model_path.clone(),
        }))
    }

    /// Generate resource nodes using existing spawn_resources function
    pub fn scatter_resources(
        &self,
        chunk: &TerrainChunk,
        chunk_size: f32,
        biome_config: &BiomeConfig,
        seed: u64,
    ) -> anyhow::Result<Vec<ResourceNode>> {
        let chunk_origin = chunk.id().to_world_pos(chunk_size);
        let area_min = chunk_origin;
        let area_max = chunk_origin + Vec3::new(chunk_size, 0.0, chunk_size);
        
        // Calculate resource count based on biome density
        let chunk_area = chunk_size * chunk_size;
        let base_count = (chunk_area * biome_config.vegetation.density * 0.1) as usize; // 10% of vegetation density
        let count = base_count.max(1).min(20); // Reasonable limits

        // Convert to BiomeRule for compatibility
        let biome = Biome::new(biome_config.biome_type, biome_config.clone());
        let biome_rule = biome.to_biome_rule();

        let resources = spawn_resources(
            seed + 1000, // Different seed offset for resources
            area_min,
            area_max,
            count,
            &biome_rule,
            None, // No weave consequence for now
        );

        Ok(resources)
    }
}

/// Combined scatter result containing vegetation, resources, and structures
#[derive(Debug, Clone)]
pub struct ScatterResult {
    pub vegetation: Vec<VegetationInstance>,
    pub resources: Vec<ResourceNode>,
    pub structures: Vec<crate::structures::StructureInstance>,
    pub chunk_id: ChunkId,
}

impl ScatterResult {
    /// Create a new scatter result
    pub fn new(chunk_id: ChunkId) -> Self {
        Self {
            vegetation: Vec::new(),
            resources: Vec::new(),
            structures: Vec::new(),
            chunk_id,
        }
    }

    /// Get total number of scattered objects
    pub fn total_count(&self) -> usize {
        self.vegetation.len() + self.resources.len()
    }

    /// Check if the scatter result is empty
    pub fn is_empty(&self) -> bool {
        self.vegetation.is_empty() && self.resources.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Heightmap, HeightmapConfig, BiomeConfig, BiomeType};

    #[test]
    fn test_vegetation_scatter_creation() {
        let config = ScatterConfig::default();
        let scatter = VegetationScatter::new(config);
        assert_eq!(scatter.config.min_distance, 2.0);
    }

    #[test]
    fn test_scatter_generation() -> anyhow::Result<()> {
        let scatter = VegetationScatter::new(ScatterConfig::default());
        
        // Create a simple test chunk
        let chunk_id = ChunkId::new(0, 0);
        let heightmap_config = HeightmapConfig { resolution: 32, ..Default::default() };
        let heightmap = Heightmap::new(heightmap_config)?;
        let biome_map = vec![BiomeType::Grassland; 32 * 32];
        let chunk = TerrainChunk::new(chunk_id, heightmap, biome_map);
        
        let biome_config = BiomeConfig::grassland();
        let vegetation = scatter.scatter_vegetation(&chunk, 256.0, &biome_config, 12345)?;
        
        // Should generate some vegetation for grassland
        assert!(!vegetation.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_resource_scattering() -> anyhow::Result<()> {
        let scatter = VegetationScatter::new(ScatterConfig::default());
        
        // Create a test chunk
        let chunk_id = ChunkId::new(0, 0);
        let heightmap_config = HeightmapConfig { resolution: 32, ..Default::default() };
        let heightmap = Heightmap::new(heightmap_config)?;
        let biome_map = vec![BiomeType::Forest; 32 * 32];
        let chunk = TerrainChunk::new(chunk_id, heightmap, biome_map);
        
        let biome_config = BiomeConfig::forest();
        let resources = scatter.scatter_resources(&chunk, 256.0, &biome_config, 12345)?;
        
        // Forest should have resources
        assert!(!resources.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_slope_filtering() {
        let scatter = VegetationScatter::new(ScatterConfig {
            max_slope: 30.0,
            ..Default::default()
        });
        
        // Create chunk with varying heights
        let chunk_id = ChunkId::new(0, 0);
        let heightmap_config = HeightmapConfig { resolution: 16, ..Default::default() };
        let mut heightmap = Heightmap::new(heightmap_config).unwrap();
        
        // Create a steep slope
        for x in 0..16 {
            for z in 0..16 {
                heightmap.set_height(x, z, x as f32 * 10.0); // Very steep
            }
        }
        
        let biome_map = vec![BiomeType::Mountain; 16 * 16];
        let chunk = TerrainChunk::new(chunk_id, heightmap, biome_map);
        
        let test_pos = Vec3::new(64.0, 50.0, 64.0);
        let slope = scatter.estimate_slope(&chunk, test_pos, 256.0);
        
        // Should detect steep slope
        assert!(slope > 30.0);
    }

    #[test]
    fn test_scatter_result() {
        let mut result = ScatterResult::new(ChunkId::new(0, 0));
        assert!(result.is_empty());
        assert_eq!(result.total_count(), 0);
        
        result.vegetation.push(VegetationInstance {
            position: Vec3::ZERO,
            rotation: 0.0,
            scale: 1.0,
            vegetation_type: "test".to_string(),
            model_path: "test.glb".to_string(),
        });
        
        assert!(!result.is_empty());
        assert_eq!(result.total_count(), 1);
    }
}