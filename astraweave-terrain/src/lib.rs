//! AstraWeave Terrain Generation Module
//! 
//! This module provides procedural terrain generation using noise functions,
//! heightmaps, and biome classification for the AstraWeave engine.

pub mod chunk;
pub mod heightmap;
pub mod noise_gen;
pub mod biome;
pub mod climate;
pub mod erosion;
pub mod scatter;

pub use chunk::{TerrainChunk, ChunkId, ChunkManager};
pub use heightmap::{Heightmap, HeightmapConfig};
pub use noise_gen::{NoiseConfig, TerrainNoise};
pub use biome::{Biome, BiomeType, BiomeConfig};
pub use climate::{ClimateMap, ClimateConfig};
pub use scatter::{VegetationScatter, VegetationInstance, ScatterConfig, ScatterResult};

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Configuration for the world generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Random seed for generation
    pub seed: u64,
    /// Size of terrain chunks in world units
    pub chunk_size: f32,
    /// Resolution of heightmaps (vertices per chunk edge)
    pub heightmap_resolution: u32,
    /// Noise configuration for terrain generation
    pub noise: NoiseConfig,
    /// Climate configuration for biome assignment
    pub climate: ClimateConfig,
    /// Available biome configurations
    pub biomes: Vec<BiomeConfig>,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            chunk_size: 256.0,
            heightmap_resolution: 128,
            noise: NoiseConfig::default(),
            climate: ClimateConfig::default(),
            biomes: vec![
                BiomeConfig::grassland(),
                BiomeConfig::desert(),
                BiomeConfig::forest(),
                BiomeConfig::mountain(),
            ],
        }
    }
}

/// Main world generator that coordinates terrain, climate, and biome generation
#[derive(Debug)]
pub struct WorldGenerator {
    config: WorldConfig,
    noise: TerrainNoise,
    climate: ClimateMap,
    chunk_manager: ChunkManager,
}

impl WorldGenerator {
    /// Create a new world generator with the given configuration
    pub fn new(config: WorldConfig) -> Self {
        let noise = TerrainNoise::new(&config.noise, config.seed);
        let climate = ClimateMap::new(&config.climate, config.seed + 1);
        let chunk_manager = ChunkManager::new(config.chunk_size, config.heightmap_resolution);
        
        Self {
            config,
            noise,
            climate,
            chunk_manager,
        }
    }

    /// Generate a terrain chunk at the given world position with vegetation and resources
    pub fn generate_chunk_with_scatter(&mut self, chunk_id: ChunkId) -> anyhow::Result<(TerrainChunk, ScatterResult)> {
        // Generate the basic terrain chunk
        let chunk = self.generate_chunk(chunk_id)?;
        
        // Generate scatter for the chunk
        let scatter_result = self.scatter_chunk_content(&chunk)?;
        
        Ok((chunk, scatter_result))
    }

    /// Generate scatter content (vegetation and resources) for an existing chunk
    pub fn scatter_chunk_content(&self, chunk: &TerrainChunk) -> anyhow::Result<ScatterResult> {
        let mut result = ScatterResult::new(chunk.id());
        
        // Create scatter system
        let scatter_config = ScatterConfig::default();
        let scatter = VegetationScatter::new(scatter_config);
        
        // Sample the biome at the chunk center to determine configuration
        let chunk_center = chunk.id().to_center_pos(self.config.chunk_size);
        let center_biome = chunk.get_biome_at_world_pos(chunk_center, self.config.chunk_size)
            .unwrap_or(BiomeType::Grassland);
        
        // Find the biome configuration
        let biome_config = self.config.biomes
            .iter()
            .find(|b| b.biome_type == center_biome)
            .unwrap_or(&self.config.biomes[0]);
        
        // Generate vegetation
        result.vegetation = scatter.scatter_vegetation(
            chunk,
            self.config.chunk_size,
            biome_config,
            self.config.seed + chunk.id().x as u64 * 1000 + chunk.id().z as u64,
        )?;
        
        // Generate resources
        result.resources = scatter.scatter_resources(
            chunk,
            self.config.chunk_size,
            biome_config,
            self.config.seed + chunk.id().x as u64 * 2000 + chunk.id().z as u64,
        )?;
        
        Ok(result)
    }
    pub fn generate_chunk(&mut self, chunk_id: ChunkId) -> anyhow::Result<TerrainChunk> {
        // Generate heightmap for this chunk
        let heightmap = self.noise.generate_heightmap(
            chunk_id, 
            self.config.chunk_size, 
            self.config.heightmap_resolution
        )?;

        // Generate climate data for biome assignment
        let climate_data = self.climate.sample_chunk(
            chunk_id,
            self.config.chunk_size,
            self.config.heightmap_resolution
        )?;

        // Assign biomes based on height and climate
        let biome_map = self.assign_biomes(&heightmap, &climate_data)?;

        // Create the terrain chunk
        let mut chunk = TerrainChunk::new(chunk_id, heightmap, biome_map);

        // Apply erosion if enabled
        if self.config.noise.erosion_enabled {
            chunk.apply_erosion(self.config.noise.erosion_strength)?;
        }

        self.chunk_manager.add_chunk(chunk.clone());
        Ok(chunk)
    }

    /// Get an existing chunk if it's loaded
    pub fn get_chunk(&self, chunk_id: ChunkId) -> Option<&TerrainChunk> {
        self.chunk_manager.get_chunk(chunk_id)
    }

    /// Stream chunks around a center position, loading/unloading as needed
    pub fn stream_chunks(&mut self, center: Vec3, radius: u32) -> anyhow::Result<Vec<ChunkId>> {
        let chunks_to_load = self.chunk_manager.get_chunks_in_radius(center, radius);
        let mut loaded = Vec::new();

        for chunk_id in chunks_to_load {
            if !self.chunk_manager.has_chunk(chunk_id) {
                self.generate_chunk(chunk_id)?;
                loaded.push(chunk_id);
            }
        }

        // Unload chunks that are too far away
        let unload_radius = radius + 2; // Keep a buffer
        self.chunk_manager.unload_distant_chunks(center, unload_radius);

        Ok(loaded)
    }

    /// Assign biomes to heightmap points based on climate data
    fn assign_biomes(
        &self, 
        heightmap: &Heightmap, 
        climate_data: &[(f32, f32)] // (temperature, moisture) pairs
    ) -> anyhow::Result<Vec<BiomeType>> {
        let mut biome_map = Vec::with_capacity(climate_data.len());

        for (i, &(temperature, moisture)) in climate_data.iter().enumerate() {
            let height = heightmap.get_height_at_index(i);
            let biome = self.find_best_biome(height, temperature, moisture);
            biome_map.push(biome);
        }

        Ok(biome_map)
    }

    /// Find the best biome for given environmental conditions
    fn find_best_biome(&self, height: f32, temperature: f32, moisture: f32) -> BiomeType {
        let mut best_biome = BiomeType::Grassland;
        let mut best_score = f32::NEG_INFINITY;

        for biome_config in &self.config.biomes {
            let score = biome_config.score_conditions(height, temperature, moisture);
            if score > best_score {
                best_score = score;
                best_biome = biome_config.biome_type;
            }
        }

        best_biome
    }

    /// Get the world configuration
    pub fn config(&self) -> &WorldConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_generator_creation() {
        let config = WorldConfig::default();
        let generator = WorldGenerator::new(config);
        assert_eq!(generator.config.seed, 12345);
    }

    #[test]
    fn test_chunk_generation() -> anyhow::Result<()> {
        let config = WorldConfig::default();
        let mut generator = WorldGenerator::new(config);
        
        let chunk_id = ChunkId::new(0, 0);
        let chunk = generator.generate_chunk(chunk_id)?;
        
        assert_eq!(chunk.id(), chunk_id);
        assert!(chunk.heightmap().max_height() >= chunk.heightmap().min_height());
        
        Ok(())
    }

    #[test]
    fn test_chunk_streaming() -> anyhow::Result<()> {
        let config = WorldConfig::default();
        let mut generator = WorldGenerator::new(config);
        
        let center = Vec3::new(128.0, 0.0, 128.0);
        let loaded_chunks = generator.stream_chunks(center, 2)?;
        
        assert!(!loaded_chunks.is_empty());
        
        Ok(())
    }
}