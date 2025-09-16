// PBR material system for AstraWeave unified showcase

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// PBR material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub albedo: [f32; 3],          // Base color RGB
    pub roughness: f32,            // Surface roughness [0-1]
    pub metallic: f32,             // Metalness factor [0-1]
    pub emissive: [f32; 3],        // Emission color RGB
    pub emissive_strength: f32,    // Emission strength
    pub normal_strength: f32,      // Normal map intensity
    pub alpha_cutoff: f32,         // Alpha cutoff for masked materials
    pub alpha_mode: AlphaMode,     // Transparency handling
    pub double_sided: bool,        // Render both sides of triangles
    
    // Texture references - relative paths or IDs
    pub albedo_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub roughness_texture: Option<String>,
    pub metallic_texture: Option<String>,
    pub emissive_texture: Option<String>,
    pub ao_texture: Option<String>,  // Ambient occlusion
}

// Alpha blending/masking mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaMode {
    Opaque,
    Masked,
    Blend,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            albedo: [1.0, 1.0, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            emissive: [0.0, 0.0, 0.0],
            emissive_strength: 0.0,
            normal_strength: 1.0,
            alpha_cutoff: 0.5,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
            albedo_texture: None,
            normal_texture: None,
            roughness_texture: None,
            metallic_texture: None,
            emissive_texture: None,
            ao_texture: None,
        }
    }
}

// Material library for managing multiple materials
pub struct MaterialLibrary {
    materials: HashMap<String, Material>,
}

impl MaterialLibrary {
    // Create a new empty material library
    pub fn new() -> Self {
        let mut library = Self {
            materials: HashMap::new(),
        };
        
        // Add default material
        library.add_material(Material::default());
        
        library
    }
    
    // Add a material to the library
    pub fn add_material(&mut self, material: Material) {
        self.materials.insert(material.name.clone(), material);
    }
    
    // Get a material by name, or default if not found
    pub fn get_material(&self, name: &str) -> &Material {
        self.materials.get(name).unwrap_or_else(|| self.materials.get("default").unwrap())
    }
    
    // Load materials from a JSON configuration file
    pub fn load_from_json(&mut self, json_str: &str) -> Result<(), serde_json::Error> {
        let materials: Vec<Material> = serde_json::from_str(json_str)?;
        
        for material in materials {
            self.add_material(material);
        }
        
        Ok(())
    }
    
    // Save materials to a JSON configuration file
    pub fn save_to_json(&self) -> Result<String, serde_json::Error> {
        let materials: Vec<Material> = self.materials.values().cloned().collect();
        serde_json::to_string_pretty(&materials)
    }
    
    // Create predefined materials for different environment types
    pub fn create_environment_materials(&mut self, environment: &str) {
        match environment {
            "grassland" => {
                // Terrain materials
                self.add_material(Material {
                    name: "grass".to_string(),
                    albedo: [0.3, 0.5, 0.2],
                    roughness: 0.8,
                    albedo_texture: Some("terrain/grass.png".to_string()),
                    normal_texture: Some("terrain/grass_n.png".to_string()),
                    ..Default::default()
                });
                
                self.add_material(Material {
                    name: "dirt".to_string(),
                    albedo: [0.6, 0.4, 0.3],
                    roughness: 0.9,
                    albedo_texture: Some("terrain/dirt.png".to_string()),
                    normal_texture: Some("terrain/dirt_n.png".to_string()),
                    ..Default::default()
                });
                
                // Vegetation materials
                self.add_material(Material {
                    name: "tree_bark".to_string(),
                    albedo: [0.4, 0.3, 0.2],
                    roughness: 0.9,
                    albedo_texture: Some("structures/tree_bark.png".to_string()),
                    normal_texture: Some("structures/tree_bark_n.png".to_string()),
                    ..Default::default()
                });
                
                self.add_material(Material {
                    name: "tree_leaves".to_string(),
                    albedo: [0.2, 0.4, 0.1],
                    roughness: 0.8,
                    alpha_mode: AlphaMode::Masked,
                    alpha_cutoff: 0.5,
                    double_sided: true,
                    albedo_texture: Some("structures/leaves_oak.png".to_string()),
                    normal_texture: Some("structures/leaves_oak_n.png".to_string()),
                    ..Default::default()
                });
                
                // Structure materials
                self.add_material(Material {
                    name: "wood_wall".to_string(),
                    albedo: [0.7, 0.5, 0.3],
                    roughness: 0.7,
                    albedo_texture: Some("structures/wood_wall.png".to_string()),
                    normal_texture: Some("structures/wood_wall_n.png".to_string()),
                    ..Default::default()
                });
                
                self.add_material(Material {
                    name: "thatch_roof".to_string(),
                    albedo: [0.7, 0.6, 0.3],
                    roughness: 0.9,
                    albedo_texture: Some("structures/thatch_roof.png".to_string()),
                    normal_texture: Some("structures/thatch_roof_n.png".to_string()),
                    ..Default::default()
                });
            },
            "desert" => {
                // Terrain materials
                self.add_material(Material {
                    name: "sand".to_string(),
                    albedo: [0.9, 0.8, 0.6],
                    roughness: 0.7,
                    albedo_texture: Some("terrain/sand.png".to_string()),
                    normal_texture: Some("terrain/sand_n.png".to_string()),
                    ..Default::default()
                });
                
                self.add_material(Material {
                    name: "stone".to_string(),
                    albedo: [0.7, 0.65, 0.5],
                    roughness: 0.9,
                    albedo_texture: Some("terrain/stone.png".to_string()),
                    normal_texture: Some("terrain/stone_n.png".to_string()),
                    ..Default::default()
                });
                
                // Vegetation materials
                self.add_material(Material {
                    name: "cactus".to_string(),
                    albedo: [0.2, 0.5, 0.2],
                    roughness: 0.8,
                    albedo_texture: Some("structures/cactus.png".to_string()),
                    normal_texture: Some("structures/cactus_n.png".to_string()),
                    ..Default::default()
                });
                
                // Structure materials
                self.add_material(Material {
                    name: "adobe_wall".to_string(),
                    albedo: [0.8, 0.7, 0.5],
                    roughness: 0.8,
                    albedo_texture: Some("structures/adobe_wall.png".to_string()),
                    normal_texture: Some("structures/adobe_wall_n.png".to_string()),
                    ..Default::default()
                });
            },
            "forest" => {
                // Terrain materials
                self.add_material(Material {
                    name: "forest_floor".to_string(),
                    albedo: [0.3, 0.25, 0.2],
                    roughness: 0.9,
                    albedo_texture: Some("terrain/forest_floor.png".to_string()),
                    normal_texture: Some("terrain/forest_floor_n.png".to_string()),
                    ..Default::default()
                });
                
                // Vegetation materials
                self.add_material(Material {
                    name: "pine_tree".to_string(),
                    albedo: [0.3, 0.25, 0.2],
                    roughness: 0.9,
                    albedo_texture: Some("structures/tree_bark.png".to_string()),
                    normal_texture: Some("structures/tree_bark_n.png".to_string()),
                    ..Default::default()
                });
                
                self.add_material(Material {
                    name: "pine_needles".to_string(),
                    albedo: [0.2, 0.35, 0.2],
                    roughness: 0.7,
                    alpha_mode: AlphaMode::Masked,
                    alpha_cutoff: 0.5,
                    double_sided: true,
                    albedo_texture: Some("structures/leaves_pine.png".to_string()),
                    normal_texture: Some("structures/leaves_pine_n.png".to_string()),
                    ..Default::default()
                });
                
                // Structure materials
                self.add_material(Material {
                    name: "log_cabin".to_string(),
                    albedo: [0.5, 0.35, 0.2],
                    roughness: 0.8,
                    albedo_texture: Some("structures/wood_wall.png".to_string()),
                    normal_texture: Some("structures/wood_wall_n.png".to_string()),
                    ..Default::default()
                });
            },
            _ => {
                // Generic materials for unknown environment types
                self.add_material(Material {
                    name: "terrain".to_string(),
                    albedo: [0.5, 0.5, 0.5],
                    roughness: 0.8,
                    ..Default::default()
                });
                
                self.add_material(Material {
                    name: "structure".to_string(),
                    albedo: [0.6, 0.6, 0.6],
                    roughness: 0.7,
                    ..Default::default()
                });
            }
        }
    }
}

// Convert material to GPU-compatible uniform buffer
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialGpu {
    pub albedo: [f32; 4],           // RGB + padding
    pub emissive: [f32; 4],         // RGB + emissive strength
    pub roughness_metallic: [f32; 4], // Roughness, metallic, normal strength, alpha cutoff
    pub flags: u32,                  // Bitflags for material properties
    pub _padding: [u32; 3],          // Padding for 16-byte alignment
}

impl From<&Material> for MaterialGpu {
    fn from(material: &Material) -> Self {
        let mut flags = 0;
        
        // Set alpha mode flags
        flags |= match material.alpha_mode {
            AlphaMode::Opaque => 0,
            AlphaMode::Masked => 1,
            AlphaMode::Blend => 2,
        };
        
        // Set double-sided flag
        if material.double_sided {
            flags |= 4;
        }
        
        // Set texture usage flags
        if material.albedo_texture.is_some() { flags |= 1 << 4; }
        if material.normal_texture.is_some() { flags |= 1 << 5; }
        if material.roughness_texture.is_some() { flags |= 1 << 6; }
        if material.metallic_texture.is_some() { flags |= 1 << 7; }
        if material.emissive_texture.is_some() { flags |= 1 << 8; }
        if material.ao_texture.is_some() { flags |= 1 << 9; }
        
        Self {
            albedo: [material.albedo[0], material.albedo[1], material.albedo[2], 1.0],
            emissive: [
                material.emissive[0],
                material.emissive[1],
                material.emissive[2],
                material.emissive_strength,
            ],
            roughness_metallic: [
                material.roughness,
                material.metallic,
                material.normal_strength,
                material.alpha_cutoff,
            ],
            flags,
            _padding: [0, 0, 0],
        }
    }
}