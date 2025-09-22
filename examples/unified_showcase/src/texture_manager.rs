// TextureManager module for AstraWeave unified_showcase
// This module handles texture loading, atlas management, and texture configuration

use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageBuffer, RgbaImage};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

/// Configuration for a texture atlas
#[derive(Debug, Deserialize)]
pub struct TextureAtlasConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub dimensions: [u32; 2],
}

/// A rectangle in UV space for texture atlas coordinates
#[derive(Debug, Clone, Copy)]
pub struct UvRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A texture entry in the atlas with metadata
#[derive(Debug)]
pub struct TextureEntry {
    pub diffuse_path: String,
    pub normal_path: Option<String>,
    pub roughness: f32,
    pub metallic: f32,
    pub uv_rect: UvRect,
    pub alpha_mask: bool,
}

/// Manager for loading and organizing textures
pub struct TextureManager {
    atlas_config: TextureAtlasConfig,
    textures: HashMap<String, TextureEntry>,
    loaded_images: HashMap<String, DynamicImage>,
}

impl TextureManager {
    /// Create a new texture manager from a configuration file
    pub fn new(config_path: &Path) -> Result<Self> {
        println!("Initializing texture manager from: {}", config_path.display());
        let config_str = fs::read_to_string(config_path)?;
        let config: serde_json::Value = toml::from_str(&config_str)?;
        
        let atlas_config: TextureAtlasConfig = toml::from_str(&config_str)?;
        
        println!("Loaded texture atlas config: {}", atlas_config.name);
        println!("Atlas dimensions: {}x{}", atlas_config.dimensions[0], atlas_config.dimensions[1]);
        
        // Parse texture entries from the config
        let mut textures = HashMap::new();
        let mut loaded_images = HashMap::new();
        
        // Process each section in the config that isn't "atlas"
        for (section_key, section_value) in config.as_object().unwrap() {
            if section_key == "atlas" {
                continue; // Skip the atlas section itself
            }
            
            // Process each texture in this section
            if let Some(section_obj) = section_value.as_object() {
                for (texture_key, texture_value) in section_obj {
                    if let Some(texture_obj) = texture_value.as_object() {
                        let texture_id = format!("{}.{}", section_key, texture_key);
                        
                        // Extract texture properties
                        let diffuse_path = texture_obj.get("diffuse")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        let normal_path = texture_obj.get("normal")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let roughness = texture_obj.get("roughness")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.5) as f32;
                            
                        let metallic = texture_obj.get("metallic")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        
                        let alpha_mask = texture_obj.get("alpha_mask")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        // Extract UV coordinates
                        let uv_rect = if let Some(uv_array) = texture_obj.get("uv_rect").and_then(|v| v.as_array()) {
                            if uv_array.len() >= 4 {
                                UvRect {
                                    x: uv_array[0].as_f64().unwrap_or(0.0) as f32 / atlas_config.dimensions[0] as f32,
                                    y: uv_array[1].as_f64().unwrap_or(0.0) as f32 / atlas_config.dimensions[1] as f32,
                                    width: uv_array[2].as_f64().unwrap_or(0.0) as f32 / atlas_config.dimensions[0] as f32,
                                    height: uv_array[3].as_f64().unwrap_or(0.0) as f32 / atlas_config.dimensions[1] as f32,
                                }
                            } else {
                                UvRect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 }
                            }
                        } else {
                            UvRect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 }
                        };
                        
                        // Create and store the texture entry
                        textures.insert(texture_id, TextureEntry {
                            diffuse_path,
                            normal_path,
                            roughness,
                            metallic,
                            uv_rect,
                            alpha_mask,
                        });
                    }
                }
            }
        }
        
        println!("Loaded {} texture entries from config", textures.len());
        
        Ok(Self {
            atlas_config,
            textures,
            loaded_images,
        })
    }
    
    /// Get all texture entries
    pub fn get_all_textures(&self) -> &HashMap<String, TextureEntry> {
        &self.textures
    }
    
    /// Load all textures from their paths
    pub fn preload_all_textures(&mut self, base_path: &Path) -> Result<()> {
        // Collect keys first to avoid borrowing self immutably and mutably at the same time
        let keys: Vec<String> = self.textures.keys().cloned().collect();
        for texture_id in keys.iter() {
            self.load_texture(texture_id, base_path)?;
        }
        
        println!("Preloaded {} textures", self.loaded_images.len());
        Ok(())
    }
    
    /// Load a specific texture by ID
    pub fn load_texture(&mut self, texture_id: &str, base_path: &Path) -> Result<()> {
        if self.loaded_images.contains_key(texture_id) {
            return Ok(());
        }
        
        if let Some(entry) = self.textures.get(texture_id) {
            let diffuse_path = base_path.join(&entry.diffuse_path);
            
            if diffuse_path.exists() {
                let image = image::open(&diffuse_path)?;
                self.loaded_images.insert(texture_id.to_string(), image);
                println!("Loaded texture: {}", texture_id);
                Ok(())
            } else {
                println!("Warning: Texture file not found: {}", diffuse_path.display());
                Err(anyhow::anyhow!("Texture file not found: {}", diffuse_path.display()))
            }
        } else {
            Err(anyhow::anyhow!("Texture ID not found in configuration: {}", texture_id))
        }
    }
    
    /// Get a loaded image by ID
    pub fn get_image(&self, texture_id: &str) -> Option<&DynamicImage> {
        self.loaded_images.get(texture_id)
    }
    
    /// Create a texture atlas from all loaded textures
    pub fn create_atlas(&self) -> Result<RgbaImage> {
        let (width, height) = (self.atlas_config.dimensions[0], self.atlas_config.dimensions[1]);
        let mut atlas = ImageBuffer::new(width, height);
        
        // Fill with transparent black
        for pixel in atlas.pixels_mut() {
            *pixel = image::Rgba([0, 0, 0, 0]);
        }
        
        // Place each texture in its specified position
        for (texture_id, entry) in &self.textures {
            if let Some(image) = self.loaded_images.get(texture_id) {
                // Calculate pixel coordinates from normalized UV coordinates
                let x = (entry.uv_rect.x * width as f32) as u32;
                let y = (entry.uv_rect.y * height as f32) as u32;
                let w = (entry.uv_rect.width * width as f32) as u32;
                let h = (entry.uv_rect.height * height as f32) as u32;
                
                // Resize the texture to fit the allocated space in the atlas
                let resized = image.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
                
                // Copy the resized texture into the atlas at the specified position
                for (i, j, pixel) in resized.to_rgba8().enumerate_pixels() {
                    if i < w && j < h && x + i < width && y + j < height {
                        atlas.put_pixel(x + i, y + j, *pixel);
                    }
                }
            }
        }
        
        Ok(atlas)
    }
    
    /// Create a normal map atlas from all loaded normal textures
    pub fn create_normal_atlas(&self) -> Result<RgbaImage> {
        let (width, height) = (self.atlas_config.dimensions[0], self.atlas_config.dimensions[1]);
        let mut atlas = ImageBuffer::new(width, height);
        
        // Fill with default normal (pointing up)
        for pixel in atlas.pixels_mut() {
            *pixel = image::Rgba([128, 128, 255, 255]);
        }
        
        // Place each normal texture in its specified position
        for (texture_id, entry) in &self.textures {
            if let Some(normal_path) = &entry.normal_path {
                let normal_full_path = Path::new("assets").join(normal_path);
                
                if normal_full_path.exists() {
                    match image::open(&normal_full_path) {
                        Ok(normal_image) => {
                            // Calculate pixel coordinates from normalized UV coordinates
                            let x = (entry.uv_rect.x * width as f32) as u32;
                            let y = (entry.uv_rect.y * height as f32) as u32;
                            let w = (entry.uv_rect.width * width as f32) as u32;
                            let h = (entry.uv_rect.height * height as f32) as u32;
                            
                            // Resize the texture to fit the allocated space in the atlas
                            let resized = normal_image.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
                            
                            // Copy the resized texture into the atlas at the specified position
                            for (i, j, pixel) in resized.to_rgba8().enumerate_pixels() {
                                if i < w && j < h && x + i < width && y + j < height {
                                    atlas.put_pixel(x + i, y + j, *pixel);
                                }
                            }
                        },
                        Err(e) => println!("Error loading normal map {}: {}", normal_path, e),
                    }
                }
            }
        }
        
        Ok(atlas)
    }
    
    /// Get the UV coordinates for a specific texture ID
    pub fn get_uv_rect(&self, texture_id: &str) -> Option<UvRect> {
        self.textures.get(texture_id).map(|entry| entry.uv_rect)
    }
    
    /// Generate and save texture atlases to files
    pub fn save_atlases(&self, output_dir: &Path) -> Result<()> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir)?;
        
        // Create and save diffuse atlas
        let diffuse_atlas = self.create_atlas()?;
        let diffuse_path = output_dir.join("atlas_diffuse.png");
        diffuse_atlas.save(&diffuse_path)?;
        println!("Saved diffuse atlas to: {}", diffuse_path.display());
        
        // Create and save normal atlas
        let normal_atlas = self.create_normal_atlas()?;
        let normal_path = output_dir.join("atlas_normal.png");
        normal_atlas.save(&normal_path)?;
        println!("Saved normal atlas to: {}", normal_path.display());
        
        Ok(())
    }
}