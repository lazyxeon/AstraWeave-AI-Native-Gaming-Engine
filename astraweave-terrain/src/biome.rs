//! Biome definitions and classification system

use astraweave_gameplay::types::ResourceKind;
use astraweave_gameplay::BiomeRule;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Types of biomes available in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Grassland,
    Desert,
    Forest,
    Mountain,
    Tundra,
    Swamp,
    Beach,
    River,
}

impl BiomeType {
    /// Get a string representation of the biome type
    pub fn as_str(&self) -> &'static str {
        match self {
            BiomeType::Grassland => "grassland",
            BiomeType::Desert => "desert",
            BiomeType::Forest => "forest",
            BiomeType::Mountain => "mountain",
            BiomeType::Tundra => "tundra",
            BiomeType::Swamp => "swamp",
            BiomeType::Beach => "beach",
            BiomeType::River => "river",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "grassland" => Some(BiomeType::Grassland),
            "desert" => Some(BiomeType::Desert),
            "forest" => Some(BiomeType::Forest),
            "mountain" => Some(BiomeType::Mountain),
            "tundra" => Some(BiomeType::Tundra),
            "swamp" => Some(BiomeType::Swamp),
            "beach" => Some(BiomeType::Beach),
            "river" => Some(BiomeType::River),
            _ => None,
        }
    }

    /// Get all available biome types
    pub fn all() -> &'static [BiomeType] {
        &[
            BiomeType::Grassland,
            BiomeType::Desert,
            BiomeType::Forest,
            BiomeType::Mountain,
            BiomeType::Tundra,
            BiomeType::Swamp,
            BiomeType::Beach,
            BiomeType::River,
        ]
    }
}

/// Environmental conditions for biome classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConditions {
    /// Height range (min, max) in world units
    pub height_range: (f32, f32),
    /// Temperature range (0.0 = frozen, 1.0 = hot)
    pub temperature_range: (f32, f32),
    /// Moisture range (0.0 = dry, 1.0 = wet)
    pub moisture_range: (f32, f32),
    /// Slope tolerance (max slope in degrees)
    pub max_slope: f32,
}

impl Default for BiomeConditions {
    fn default() -> Self {
        Self {
            height_range: (0.0, 1000.0),
            temperature_range: (0.0, 1.0),
            moisture_range: (0.0, 1.0),
            max_slope: 90.0,
        }
    }
}

/// Sky and weather parameters for a biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSky {
    /// Horizon color (RGB)
    pub horizon_color: Vec3,
    /// Zenith color (RGB)
    pub zenith_color: Vec3,
    /// Sun color (RGB)
    pub sun_color: Vec3,
    /// Fog density (0.0 = no fog, 1.0 = thick fog)
    pub fog_density: f32,
    /// Fog color (RGB)
    pub fog_color: Vec3,
    /// Cloud coverage (0.0 = clear, 1.0 = overcast)
    pub cloud_coverage: f32,
    /// Precipitation type
    pub precipitation: PrecipitationType,
}

impl Default for BiomeSky {
    fn default() -> Self {
        Self {
            horizon_color: Vec3::new(0.5, 0.7, 0.9),
            zenith_color: Vec3::new(0.2, 0.4, 0.8),
            sun_color: Vec3::new(1.0, 0.9, 0.7),
            fog_density: 0.0,
            fog_color: Vec3::new(0.8, 0.8, 0.9),
            cloud_coverage: 0.3,
            precipitation: PrecipitationType::None,
        }
    }
}

/// Types of precipitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrecipitationType {
    None,
    Rain,
    Snow,
    Fog,
    Sandstorm,
}

/// Vegetation parameters for a biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeVegetation {
    /// Base vegetation density (objects per unit area)
    pub density: f32,
    /// Available vegetation types with weights
    pub vegetation_types: Vec<VegetationType>,
    /// Size variation for vegetation (min, max scale multipliers)
    pub size_variation: (f32, f32),
    /// Rotation randomization
    pub random_rotation: bool,
}

impl Default for BiomeVegetation {
    fn default() -> Self {
        Self {
            density: 0.1,
            vegetation_types: vec![VegetationType {
                name: "grass".to_string(),
                weight: 1.0,
                model_path: "assets/models/grass.glb".to_string(),
                scale_range: (0.8, 1.2),
                slope_tolerance: 45.0,
            }],
            size_variation: (0.8, 1.5),
            random_rotation: true,
        }
    }
}

/// A single vegetation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegetationType {
    /// Name of the vegetation type
    pub name: String,
    /// Weight for random selection
    pub weight: f32,
    /// Path to the 3D model
    pub model_path: String,
    /// Scale range (min, max)
    pub scale_range: (f32, f32),
    /// Maximum slope this vegetation can grow on (degrees)
    pub slope_tolerance: f32,
}

/// Complete biome configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConfig {
    /// The biome type
    pub biome_type: BiomeType,
    /// Human-readable name
    pub name: String,
    /// Description of the biome
    pub description: String,
    /// Environmental conditions for this biome
    pub conditions: BiomeConditions,
    /// Sky and weather parameters
    pub sky: BiomeSky,
    /// Vegetation configuration
    pub vegetation: BiomeVegetation,
    /// Resource spawning (reusing existing astraweave-gameplay system)
    pub resource_weights: Vec<(ResourceKind, f32)>,
    /// Base resource amounts
    pub base_resource_amount: (u32, u32),
    /// Respawn timing for resources
    pub resource_respawn: (f32, f32),
    /// Ground textures for rendering
    pub ground_textures: Vec<String>,
    /// Priority for biome selection (higher wins in conflicts)
    pub priority: i32,
}

impl BiomeConfig {
    /// Create a default grassland biome
    pub fn grassland() -> Self {
        Self {
            biome_type: BiomeType::Grassland,
            name: "Temperate Grassland".to_string(),
            description: "Rolling hills covered in grass with scattered trees".to_string(),
            conditions: BiomeConditions {
                height_range: (0.0, 50.0),
                temperature_range: (0.3, 0.8),
                moisture_range: (0.4, 0.8),
                max_slope: 30.0,
            },
            sky: BiomeSky {
                horizon_color: Vec3::new(0.6, 0.8, 0.9),
                zenith_color: Vec3::new(0.3, 0.5, 0.9),
                sun_color: Vec3::new(1.0, 0.95, 0.8),
                fog_density: 0.05,
                fog_color: Vec3::new(0.9, 0.95, 1.0),
                cloud_coverage: 0.4,
                precipitation: PrecipitationType::Rain,
            },
            vegetation: BiomeVegetation {
                density: 0.8,
                vegetation_types: vec![
                    VegetationType {
                        name: "grass_cluster".to_string(),
                        weight: 3.0,
                        model_path: "assets/models/grass_cluster.glb".to_string(),
                        scale_range: (0.8, 1.2),
                        slope_tolerance: 45.0,
                    },
                    VegetationType {
                        name: "oak_tree".to_string(),
                        weight: 0.5,
                        model_path: "assets/models/oak_tree.glb".to_string(),
                        scale_range: (0.9, 1.4),
                        slope_tolerance: 25.0,
                    },
                    VegetationType {
                        name: "wildflowers".to_string(),
                        weight: 1.0,
                        model_path: "assets/models/wildflowers.glb".to_string(),
                        scale_range: (0.7, 1.1),
                        slope_tolerance: 35.0,
                    },
                ],
                size_variation: (0.8, 1.5),
                random_rotation: true,
            },
            resource_weights: vec![
                (ResourceKind::Fiber, 3.0),
                (ResourceKind::Wood, 2.0),
                (ResourceKind::Essence, 1.0),
            ],
            base_resource_amount: (3, 8),
            resource_respawn: (30.0, 120.0),
            ground_textures: vec![
                "assets/textures/grass_diffuse.png".to_string(),
                "assets/textures/dirt_diffuse.png".to_string(),
            ],
            priority: 1,
        }
    }

    /// Create a default desert biome
    pub fn desert() -> Self {
        Self {
            biome_type: BiomeType::Desert,
            name: "Arid Desert".to_string(),
            description: "Sandy dunes with sparse vegetation and extreme temperatures".to_string(),
            conditions: BiomeConditions {
                height_range: (0.0, 30.0),
                temperature_range: (0.7, 1.0),
                moisture_range: (0.0, 0.3),
                max_slope: 20.0,
            },
            sky: BiomeSky {
                horizon_color: Vec3::new(0.9, 0.7, 0.5),
                zenith_color: Vec3::new(0.6, 0.8, 0.9),
                sun_color: Vec3::new(1.0, 0.9, 0.6),
                fog_density: 0.02,
                fog_color: Vec3::new(0.9, 0.8, 0.6),
                cloud_coverage: 0.1,
                precipitation: PrecipitationType::Sandstorm,
            },
            vegetation: BiomeVegetation {
                density: 0.1,
                vegetation_types: vec![
                    VegetationType {
                        name: "cactus".to_string(),
                        weight: 2.0,
                        model_path: "assets/models/cactus.glb".to_string(),
                        scale_range: (0.7, 1.8),
                        slope_tolerance: 15.0,
                    },
                    VegetationType {
                        name: "desert_shrub".to_string(),
                        weight: 1.5,
                        model_path: "assets/models/desert_shrub.glb".to_string(),
                        scale_range: (0.6, 1.2),
                        slope_tolerance: 25.0,
                    },
                ],
                size_variation: (0.5, 2.0),
                random_rotation: true,
            },
            resource_weights: vec![
                (ResourceKind::Crystal, 2.0),
                (ResourceKind::Ore, 1.5),
                (ResourceKind::Fiber, 0.5),
            ],
            base_resource_amount: (1, 4),
            resource_respawn: (60.0, 300.0),
            ground_textures: vec![
                "assets/textures/sand_diffuse.png".to_string(),
                "assets/textures/sandstone_diffuse.png".to_string(),
            ],
            priority: 2,
        }
    }

    /// Create a default forest biome
    pub fn forest() -> Self {
        Self {
            biome_type: BiomeType::Forest,
            name: "Dense Forest".to_string(),
            description: "Thick woodland with towering trees and rich undergrowth".to_string(),
            conditions: BiomeConditions {
                height_range: (10.0, 80.0),
                temperature_range: (0.4, 0.9),
                moisture_range: (0.6, 1.0),
                max_slope: 40.0,
            },
            sky: BiomeSky {
                horizon_color: Vec3::new(0.4, 0.6, 0.4),
                zenith_color: Vec3::new(0.2, 0.4, 0.6),
                sun_color: Vec3::new(0.9, 1.0, 0.8),
                fog_density: 0.15,
                fog_color: Vec3::new(0.7, 0.8, 0.7),
                cloud_coverage: 0.6,
                precipitation: PrecipitationType::Rain,
            },
            vegetation: BiomeVegetation {
                density: 1.5,
                vegetation_types: vec![
                    VegetationType {
                        name: "pine_tree".to_string(),
                        weight: 3.0,
                        model_path: "assets/models/pine_tree.glb".to_string(),
                        scale_range: (1.2, 2.0),
                        slope_tolerance: 35.0,
                    },
                    VegetationType {
                        name: "birch_tree".to_string(),
                        weight: 2.0,
                        model_path: "assets/models/birch_tree.glb".to_string(),
                        scale_range: (1.0, 1.6),
                        slope_tolerance: 30.0,
                    },
                    VegetationType {
                        name: "fern".to_string(),
                        weight: 4.0,
                        model_path: "assets/models/fern.glb".to_string(),
                        scale_range: (0.8, 1.4),
                        slope_tolerance: 50.0,
                    },
                    VegetationType {
                        name: "mushroom".to_string(),
                        weight: 1.0,
                        model_path: "assets/models/mushroom.glb".to_string(),
                        scale_range: (0.5, 1.0),
                        slope_tolerance: 60.0,
                    },
                ],
                size_variation: (0.7, 1.8),
                random_rotation: true,
            },
            resource_weights: vec![
                (ResourceKind::Wood, 4.0),
                (ResourceKind::Fiber, 2.0),
                (ResourceKind::Essence, 2.5),
            ],
            base_resource_amount: (4, 12),
            resource_respawn: (15.0, 60.0),
            ground_textures: vec![
                "assets/textures/forest_floor_diffuse.png".to_string(),
                "assets/textures/moss_diffuse.png".to_string(),
                "assets/textures/bark_diffuse.png".to_string(),
            ],
            priority: 3,
        }
    }

    /// Create a default mountain biome
    pub fn mountain() -> Self {
        Self {
            biome_type: BiomeType::Mountain,
            name: "Rocky Mountains".to_string(),
            description: "High altitude peaks with rocky terrain and sparse vegetation".to_string(),
            conditions: BiomeConditions {
                height_range: (60.0, 200.0),
                temperature_range: (0.0, 0.5),
                moisture_range: (0.2, 0.7),
                max_slope: 70.0,
            },
            sky: BiomeSky {
                horizon_color: Vec3::new(0.7, 0.8, 0.9),
                zenith_color: Vec3::new(0.4, 0.6, 0.9),
                sun_color: Vec3::new(1.0, 0.95, 0.9),
                fog_density: 0.3,
                fog_color: Vec3::new(0.9, 0.9, 1.0),
                cloud_coverage: 0.7,
                precipitation: PrecipitationType::Snow,
            },
            vegetation: BiomeVegetation {
                density: 0.3,
                vegetation_types: vec![
                    VegetationType {
                        name: "alpine_tree".to_string(),
                        weight: 1.0,
                        model_path: "assets/models/alpine_tree.glb".to_string(),
                        scale_range: (0.8, 1.2),
                        slope_tolerance: 45.0,
                    },
                    VegetationType {
                        name: "mountain_grass".to_string(),
                        weight: 2.0,
                        model_path: "assets/models/mountain_grass.glb".to_string(),
                        scale_range: (0.6, 1.0),
                        slope_tolerance: 60.0,
                    },
                    VegetationType {
                        name: "boulder".to_string(),
                        weight: 1.5,
                        model_path: "assets/models/boulder.glb".to_string(),
                        scale_range: (0.8, 2.5),
                        slope_tolerance: 80.0,
                    },
                ],
                size_variation: (0.6, 2.0),
                random_rotation: true,
            },
            resource_weights: vec![
                (ResourceKind::Ore, 3.0),
                (ResourceKind::Crystal, 2.0),
                (ResourceKind::Wood, 0.5),
            ],
            base_resource_amount: (2, 6),
            resource_respawn: (45.0, 180.0),
            ground_textures: vec![
                "assets/textures/rock_diffuse.png".to_string(),
                "assets/textures/snow_diffuse.png".to_string(),
                "assets/textures/gravel_diffuse.png".to_string(),
            ],
            priority: 4,
        }
    }

    /// Score how well this biome fits the given environmental conditions
    pub fn score_conditions(&self, height: f32, temperature: f32, moisture: f32) -> f32 {
        let mut score = 0.0;

        // Height score
        if height >= self.conditions.height_range.0 && height <= self.conditions.height_range.1 {
            score += 1.0;
        } else {
            let height_distance = if height < self.conditions.height_range.0 {
                self.conditions.height_range.0 - height
            } else {
                height - self.conditions.height_range.1
            };
            score -= height_distance * 0.01; // Penalty for being outside range
        }

        // Temperature score
        if temperature >= self.conditions.temperature_range.0
            && temperature <= self.conditions.temperature_range.1
        {
            score += 1.0;
        } else {
            let temp_distance = if temperature < self.conditions.temperature_range.0 {
                self.conditions.temperature_range.0 - temperature
            } else {
                temperature - self.conditions.temperature_range.1
            };
            score -= temp_distance * 2.0; // Higher penalty for temperature mismatch
        }

        // Moisture score
        if moisture >= self.conditions.moisture_range.0
            && moisture <= self.conditions.moisture_range.1
        {
            score += 1.0;
        } else {
            let moisture_distance = if moisture < self.conditions.moisture_range.0 {
                self.conditions.moisture_range.0 - moisture
            } else {
                moisture - self.conditions.moisture_range.1
            };
            score -= moisture_distance * 1.5; // Moderate penalty for moisture mismatch
        }

        // Add priority bonus
        score += self.priority as f32 * 0.1;

        score
    }

    /// Check if a slope is suitable for this biome
    pub fn is_slope_suitable(&self, slope_degrees: f32) -> bool {
        slope_degrees <= self.conditions.max_slope
    }
}

/// Runtime biome data for a specific location
#[derive(Debug, Clone)]
pub struct Biome {
    pub biome_type: BiomeType,
    pub config: BiomeConfig,
    pub local_conditions: BiomeConditions,
}

impl Biome {
    /// Create a new biome instance
    pub fn new(biome_type: BiomeType, config: BiomeConfig) -> Self {
        let local_conditions = config.conditions.clone();
        Self {
            biome_type,
            config,
            local_conditions,
        }
    }

    /// Get the vegetation density at this location
    pub fn vegetation_density(&self) -> f32 {
        self.config.vegetation.density
    }

    /// Get resource spawning weights for integration with existing system
    pub fn resource_weights(&self) -> &[(ResourceKind, f32)] {
        &self.config.resource_weights
    }

    /// Convert to the existing BiomeRule format for compatibility
    pub fn to_biome_rule(&self) -> BiomeRule {
        BiomeRule {
            name: self.config.name.clone(),
            weights: self.config.resource_weights.clone(),
            base_amount: self.config.base_resource_amount,
            respawn: self.config.resource_respawn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_type_string_conversion() {
        assert_eq!(BiomeType::Grassland.as_str(), "grassland");
        assert_eq!(BiomeType::from_str("desert"), Some(BiomeType::Desert));
        assert_eq!(BiomeType::from_str("invalid"), None);
    }

    #[test]
    fn test_biome_config_creation() {
        let grassland = BiomeConfig::grassland();
        assert_eq!(grassland.biome_type, BiomeType::Grassland);
        assert!(!grassland.resource_weights.is_empty());
    }

    #[test]
    fn test_biome_scoring() {
        let grassland = BiomeConfig::grassland();

        // Perfect conditions
        let score1 = grassland.score_conditions(25.0, 0.5, 0.6);

        // Poor conditions
        let score2 = grassland.score_conditions(100.0, 0.1, 0.1);

        assert!(score1 > score2);
    }

    #[test]
    fn test_slope_suitability() {
        let mountain = BiomeConfig::mountain();
        assert!(mountain.is_slope_suitable(45.0));
        assert!(!mountain.is_slope_suitable(80.0));
    }

    #[test]
    fn test_biome_rule_conversion() {
        let forest_config = BiomeConfig::forest();
        let biome = Biome::new(BiomeType::Forest, forest_config);
        let biome_rule = biome.to_biome_rule();

        assert_eq!(biome_rule.name, "Dense Forest");
        assert!(!biome_rule.weights.is_empty());
    }

    #[test]
    fn test_default_biomes() {
        let biomes = vec![
            BiomeConfig::grassland(),
            BiomeConfig::desert(),
            BiomeConfig::forest(),
            BiomeConfig::mountain(),
        ];

        for biome in biomes {
            assert!(!biome.name.is_empty());
            assert!(!biome.description.is_empty());
            assert!(!biome.resource_weights.is_empty());
            assert!(!biome.ground_textures.is_empty());
        }
    }
}
