//! Structure generation system for placing buildings, ruins, and other constructions
//! 
//! This module provides procedural placement of structures within terrain chunks
//! based on biome type, terrain features, and generation rules.

use anyhow::Result;
use glam::Vec3;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use crate::{BiomeType, TerrainChunk};

/// Types of structures that can be generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructureType {
    // Residential structures
    Cottage,
    Farmhouse,
    Villa,
    Cabin,
    
    // Commercial/Community
    Tavern,
    Blacksmith,
    Market,
    Temple,
    
    // Defensive structures
    Watchtower,
    Fort,
    Wall,
    Gate,
    
    // Ancient/Ruins
    AncientRuin,
    StoneCircle,
    Obelisk,
    Tomb,
    
    // Natural formations
    Cave,
    RockFormation,
    CrystalFormation,
    
    // Infrastructure
    Bridge,
    Well,
    Windmill,
    Lighthouse,
}

impl StructureType {
    /// Get appropriate structure types for a biome
    pub fn for_biome(biome: BiomeType) -> Vec<Self> {
        use StructureType::*;
        
        match biome {
            BiomeType::Grassland => vec![
                Cottage, Farmhouse, Villa, Tavern, Blacksmith, Market, Temple,
                Watchtower, Well, Windmill, AncientRuin
            ],
            BiomeType::Desert => vec![
                Villa, Market, Temple, Watchtower, Fort, AncientRuin, 
                Obelisk, Tomb, RockFormation, Well
            ],
            BiomeType::Forest => vec![
                Cottage, Cabin, Temple, Watchtower, AncientRuin, 
                StoneCircle, Cave, RockFormation
            ],
            BiomeType::Mountain => vec![
                Cabin, Fort, Watchtower, Temple, Cave, AncientRuin,
                CrystalFormation, RockFormation, Bridge
            ],
            BiomeType::Tundra => vec![
                Cabin, Fort, Watchtower, Cave, AncientRuin, 
                RockFormation, CrystalFormation
            ],
            BiomeType::Swamp => vec![
                Cabin, Temple, AncientRuin, StoneCircle, Cave, Bridge
            ],
            BiomeType::Beach => vec![
                Cottage, Tavern, Lighthouse, Temple, Cave, RockFormation
            ],
            BiomeType::River => vec![
                Cottage, Farmhouse, Tavern, Blacksmith, Bridge, Well, Windmill
            ],
        }
    }

    /// Get the typical size of this structure type
    pub fn typical_size(&self) -> f32 {
        use StructureType::*;
        
        match self {
            Cottage | Cabin => 8.0,
            Farmhouse | Villa => 12.0,
            Tavern | Blacksmith | Market => 10.0,
            Temple => 15.0,
            Watchtower => 6.0,
            Fort => 20.0,
            Wall | Gate => 5.0,
            AncientRuin => 15.0,
            StoneCircle => 12.0,
            Obelisk => 3.0,
            Tomb => 8.0,
            Cave => 6.0,
            RockFormation => 4.0,
            CrystalFormation => 5.0,
            Bridge => 15.0,
            Well => 2.0,
            Windmill => 8.0,
            Lighthouse => 6.0,
        }
    }

    /// Get the rarity of this structure (lower = more common)
    pub fn rarity(&self) -> f32 {
        use StructureType::*;
        
        match self {
            // Common structures
            Cottage | Farmhouse | RockFormation => 0.8,
            
            // Uncommon structures
            Cabin | Tavern | Blacksmith | Well | Windmill => 0.6,
            
            // Rare structures
            Villa | Market | Temple | Watchtower | Cave => 0.4,
            
            // Very rare structures
            Fort | AncientRuin | StoneCircle | Bridge | Lighthouse => 0.2,
            
            // Extremely rare structures
            Wall | Gate | Obelisk | Tomb | CrystalFormation => 0.1,
        }
    }

    /// Check if this structure can be placed on the given terrain slope
    pub fn can_place_on_slope(&self, slope: f32) -> bool {
        use StructureType::*;
        
        let max_slope = match self {
            // Must be on flat ground
            Farmhouse | Market | Temple | Fort => 0.1,
            
            // Can handle gentle slopes
            Cottage | Villa | Tavern | Blacksmith | Well | Windmill => 0.2,
            
            // Can handle moderate slopes
            Cabin | Watchtower | AncientRuin | StoneCircle => 0.4,
            
            // Can handle steep slopes
            Cave | RockFormation | CrystalFormation | Lighthouse => 0.8,
            
            // Very flexible placement
            Wall | Gate | Obelisk | Tomb | Bridge => 1.0,
        };
        
        slope <= max_slope
    }

    /// Get minimum distance to other structures of the same type
    pub fn min_spacing(&self) -> f32 {
        use StructureType::*;
        
        match self {
            // Large spacing for major structures
            Fort | Temple | Market => 100.0,
            
            // Medium spacing for important structures
            Villa | Tavern | Blacksmith | Lighthouse | Bridge => 50.0,
            
            // Small spacing for common structures
            Cottage | Farmhouse | Cabin | Watchtower => 30.0,
            
            // Very small spacing for minor structures
            Well | RockFormation => 20.0,
            
            // Minimal spacing for natural features
            Cave | CrystalFormation | AncientRuin | StoneCircle | Obelisk | Tomb => 15.0,
            
            // No spacing restrictions
            Wall | Gate | Windmill => 10.0,
        }
    }
}

/// Configuration for structure generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureConfig {
    /// Overall density of structures (0.0 = none, 1.0 = maximum)
    pub density: f32,
    /// Minimum distance from chunk edges for structure placement
    pub edge_buffer: f32,
    /// Height variation tolerance for structure placement
    pub height_tolerance: f32,
    /// Whether to generate ancient/mystical structures
    pub include_ancient: bool,
    /// Whether to generate defensive structures
    pub include_defensive: bool,
    /// Seed for structure generation
    pub seed: u64,
}

impl Default for StructureConfig {
    fn default() -> Self {
        Self {
            density: 0.3,
            edge_buffer: 20.0,
            height_tolerance: 2.0,
            include_ancient: true,
            include_defensive: true,
            seed: 0,
        }
    }
}

/// A placed structure instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureInstance {
    /// Type of structure
    pub structure_type: StructureType,
    /// Position in world coordinates
    pub position: Vec3,
    /// Rotation around Y axis in radians
    pub rotation: f32,
    /// Scale multiplier (1.0 = normal size)
    pub scale: f32,
    /// Path to the 3D model file
    pub model_path: String,
    /// Optional texture variant
    pub texture_variant: Option<String>,
}

/// Result of structure generation for a chunk
#[derive(Debug, Clone, Default)]
pub struct StructureResult {
    /// List of structures in this chunk
    pub structures: Vec<StructureInstance>,
    /// Total count by type for statistics
    pub counts_by_type: std::collections::HashMap<StructureType, u32>,
}

impl StructureResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a structure to the result
    pub fn add_structure(&mut self, structure: StructureInstance) {
        *self.counts_by_type.entry(structure.structure_type).or_insert(0) += 1;
        self.structures.push(structure);
    }

    /// Get total structure count
    pub fn total_count(&self) -> usize {
        self.structures.len()
    }
}

/// Structure generation system
#[derive(Debug)]
pub struct StructureGenerator {
    config: StructureConfig,
    rng: StdRng,
}

impl StructureGenerator {
    /// Create a new structure generator
    pub fn new(config: StructureConfig) -> Self {
        let rng = StdRng::seed_from_u64(config.seed);
        Self { config, rng }
    }

    /// Generate structures for a terrain chunk
    pub fn generate_structures(
        &mut self,
        chunk: &TerrainChunk,
        chunk_size: f32,
        biome_type: BiomeType,
    ) -> Result<StructureResult> {
        let mut result = StructureResult::new();
        
        if self.config.density <= 0.0 {
            return Ok(result);
        }

        // Get appropriate structure types for this biome
        let mut available_structures = StructureType::for_biome(biome_type);
        
        // Filter structures based on config
        if !self.config.include_ancient {
            available_structures.retain(|s| !matches!(s, 
                StructureType::AncientRuin | StructureType::StoneCircle | 
                StructureType::Obelisk | StructureType::Tomb
            ));
        }
        
        if !self.config.include_defensive {
            available_structures.retain(|s| !matches!(s,
                StructureType::Fort | StructureType::Watchtower |
                StructureType::Wall | StructureType::Gate
            ));
        }

        if available_structures.is_empty() {
            return Ok(result);
        }

        // Calculate number of structures to attempt based on density
        let max_structures = (chunk_size * chunk_size / 2000.0 * self.config.density) as u32;
        let structure_attempts = self.rng.random_range(0..=max_structures.max(1));

        for _ in 0..structure_attempts {
            if let Some(structure) = self.try_place_structure(
                chunk,
                chunk_size,
                &available_structures,
                &result,
            )? {
                result.add_structure(structure);
            }
        }

        Ok(result)
    }

    /// Attempt to place a single structure
    fn try_place_structure(
        &mut self,
        chunk: &TerrainChunk,
        chunk_size: f32,
        available_structures: &[StructureType],
        existing_result: &StructureResult,
    ) -> Result<Option<StructureInstance>> {
        const MAX_ATTEMPTS: u32 = 50;
        
        for _ in 0..MAX_ATTEMPTS {
            // Choose a random structure type based on rarity
            let structure_type = self.choose_structure_type(available_structures)?;
            
            // Generate random position within chunk bounds
            let x = self.rng.random_range(self.config.edge_buffer..chunk_size - self.config.edge_buffer);
            let z = self.rng.random_range(self.config.edge_buffer..chunk_size - self.config.edge_buffer);
            
            // Convert to world position
            let chunk_origin = chunk.id().to_world_pos(chunk_size);
            let world_x = chunk_origin.x + x;
            let world_z = chunk_origin.z + z;
            
            // Get terrain height at this position
            let local_x = (x / chunk_size * (chunk.heightmap().resolution() - 1) as f32) as u32;
            let local_z = (z / chunk_size * (chunk.heightmap().resolution() - 1) as f32) as u32;
            
            if local_x >= chunk.heightmap().resolution() || local_z >= chunk.heightmap().resolution() {
                continue;
            }
            
            let height = chunk.heightmap().get_height(local_x, local_z);
            let position = Vec3::new(world_x, height, world_z);
            
            // Check terrain suitability
            if !self.is_suitable_location(chunk, local_x, local_z, structure_type)? {
                continue;
            }
            
            // Check spacing requirements
            if !self.check_spacing(position, structure_type, existing_result) {
                continue;
            }
            
            // Generate structure properties
            let rotation = self.rng.random_range(0.0..std::f32::consts::TAU);
            let scale = self.rng.random_range(0.8..1.2);
            let model_path = self.get_model_path(structure_type);
            let texture_variant = self.get_texture_variant(structure_type);
            
            return Ok(Some(StructureInstance {
                structure_type,
                position,
                rotation,
                scale,
                model_path,
                texture_variant,
            }));
        }
        
        Ok(None)
    }

    /// Choose a structure type based on rarity weights
    fn choose_structure_type(&mut self, available: &[StructureType]) -> Result<StructureType> {
        if available.is_empty() {
            anyhow::bail!("No available structure types");
        }

        // Calculate total weight
        let total_weight: f32 = available.iter().map(|s| s.rarity()).sum();
        let mut target = self.rng.random_range(0.0..total_weight);
        
        for &structure_type in available {
            target -= structure_type.rarity();
            if target <= 0.0 {
                return Ok(structure_type);
            }
        }
        
        // Fallback to last structure
        Ok(available[available.len() - 1])
    }

    /// Check if a location is suitable for placing a structure
    fn is_suitable_location(
        &self,
        chunk: &TerrainChunk,
        local_x: u32,
        local_z: u32,
        structure_type: StructureType,
    ) -> Result<bool> {
        let heightmap = chunk.heightmap();
        let resolution = heightmap.resolution();
        
        // Check slope in a small area around the position
        let check_radius = 2u32;
        let mut height_samples = Vec::new();
        
        for dx in 0..=check_radius * 2 {
            for dz in 0..=check_radius * 2 {
                let x = local_x.saturating_sub(check_radius).saturating_add(dx).min(resolution - 1);
                let z = local_z.saturating_sub(check_radius).saturating_add(dz).min(resolution - 1);
                height_samples.push(heightmap.get_height(x, z));
            }
        }
        
        let min_height = height_samples.iter().copied().fold(f32::INFINITY, f32::min);
        let max_height = height_samples.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let height_variation = max_height - min_height;
        
        // Check if height variation is within tolerance
        if height_variation > self.config.height_tolerance {
            return Ok(false);
        }
        
        // Calculate slope
        let slope = height_variation / (check_radius as f32 * 2.0);
        
        // Check if structure can be placed on this slope
        Ok(structure_type.can_place_on_slope(slope))
    }

    /// Check spacing requirements against existing structures
    fn check_spacing(
        &self,
        position: Vec3,
        structure_type: StructureType,
        existing: &StructureResult,
    ) -> bool {
        let min_spacing = structure_type.min_spacing();
        
        for existing_structure in &existing.structures {
            let distance = (position - existing_structure.position).length();
            
            // Check against same type
            if existing_structure.structure_type == structure_type && distance < min_spacing {
                return false;
            }
            
            // Check against any large structures
            let existing_size = existing_structure.structure_type.typical_size();
            let our_size = structure_type.typical_size();
            let required_distance = (existing_size + our_size) * 0.5;
            
            if distance < required_distance {
                return false;
            }
        }
        
        true
    }

    /// Get the model path for a structure type
    fn get_model_path(&self, structure_type: StructureType) -> String {
        use StructureType::*;
        
        let base_path = "assets/models/structures/";
        let filename = match structure_type {
            Cottage => "cottage.glb",
            Farmhouse => "farmhouse.glb",
            Villa => "villa.glb",
            Cabin => "cabin.glb",
            Tavern => "tavern.glb",
            Blacksmith => "blacksmith.glb",
            Market => "market.glb",
            Temple => "temple.glb",
            Watchtower => "watchtower.glb",
            Fort => "fort.glb",
            Wall => "wall.glb",
            Gate => "gate.glb",
            AncientRuin => "ancient_ruin.glb",
            StoneCircle => "stone_circle.glb",
            Obelisk => "obelisk.glb",
            Tomb => "tomb.glb",
            Cave => "cave.glb",
            RockFormation => "rock_formation.glb",
            CrystalFormation => "crystal_formation.glb",
            Bridge => "bridge.glb",
            Well => "well.glb",
            Windmill => "windmill.glb",
            Lighthouse => "lighthouse.glb",
        };
        
        format!("{}{}", base_path, filename)
    }

    /// Get a texture variant for variety
    fn get_texture_variant(&mut self, structure_type: StructureType) -> Option<String> {
        use StructureType::*;
        
        // Some structures have multiple texture variants
        let variants = match structure_type {
            Cottage | Farmhouse | Cabin => vec!["wood", "stone", "mixed"],
            Villa | Temple => vec!["marble", "sandstone", "brick"],
            Tavern | Blacksmith => vec!["wood", "stone"],
            RockFormation => vec!["granite", "limestone", "basalt"],
            CrystalFormation => vec!["quartz", "amethyst", "emerald"],
            _ => return None,
        };
        
        if variants.is_empty() {
            None
        } else {
            let index = self.rng.random_range(0..variants.len());
            Some(variants[index].to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorldConfig, WorldGenerator, ChunkId};

    #[test]
    fn test_structure_type_for_biome() {
        let grassland_structures = StructureType::for_biome(BiomeType::Grassland);
        assert!(!grassland_structures.is_empty());
        assert!(grassland_structures.contains(&StructureType::Cottage));
        
        let desert_structures = StructureType::for_biome(BiomeType::Desert);
        assert!(desert_structures.contains(&StructureType::Obelisk));
    }

    #[test]
    fn test_structure_properties() {
        assert!(StructureType::Cottage.typical_size() > 0.0);
        assert!(StructureType::Fort.typical_size() > StructureType::Cottage.typical_size());
        
        assert!(StructureType::Cottage.rarity() > StructureType::Fort.rarity());
    }

    #[test]
    fn test_structure_generation() -> Result<()> {
        let config = StructureConfig {
            density: 0.5,
            ..Default::default()
        };
        
        let mut generator = StructureGenerator::new(config);
        
        // Create a test chunk
        let world_config = WorldConfig::default();
        let mut world_gen = WorldGenerator::new(world_config);
        let chunk = world_gen.generate_chunk(ChunkId::new(0, 0))?;
        
        let _result = generator.generate_structures(&chunk, 256.0, BiomeType::Grassland)?;
        
        // Should generate some structures (could be 0 due to random placement failures)
        // No need to assert >= 0 since total_count() returns usize which is always >= 0
        
        Ok(())
    }

    #[test]
    fn test_spacing_requirements() {
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(10.0, 0.0, 0.0); // 10 units away
        
        let mut result = StructureResult::new();
        result.add_structure(StructureInstance {
            structure_type: StructureType::Cottage,
            position: pos1,
            rotation: 0.0,
            scale: 1.0,
            model_path: "test.glb".to_string(),
            texture_variant: None,
        });
        
        let config = StructureConfig::default();
        let generator = StructureGenerator::new(config);
        
        // Should not allow another cottage too close (min spacing is 30.0)
        assert!(!generator.check_spacing(pos2, StructureType::Cottage, &result));
        
        // Should allow a small structure like a well
        assert!(generator.check_spacing(pos2, StructureType::Well, &result));
    }
}