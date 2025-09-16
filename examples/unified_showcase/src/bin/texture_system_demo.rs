// Texture system demonstration for AstraWeave unified showcase
// This script shows how to use the texture system components to load, manage, and render textures

mod texture_manager;
mod texture_utils;
mod material;

use anyhow::Result;
use std::{path::Path, fs};
use material::{Material, MaterialLibrary};
use texture_manager::TextureManager;
use texture_utils::{generate_placeholder_texture, find_normal_map, resolve_texture_path};

fn main() -> Result<()> {
    println!("AstraWeave Unified Showcase - Texture System Demo");
    println!("=================================================");
    
    // 1. Ensure texture directories exist
    println!("\nChecking texture directories...");
    let base_path = Path::new("assets/textures");
    
    let texture_directories = [
        base_path.join("terrain"),
        base_path.join("structures"), 
        base_path.join("characters"),
        base_path.join("skybox"),
        base_path.join("effects")
    ];
    
    for dir in &texture_directories {
        if !dir.exists() {
            println!("Creating directory: {}", dir.display());
            fs::create_dir_all(dir)?;
        } else {
            println!("Directory exists: {}", dir.display());
        }
    }
    
    // 2. Load texture atlas configuration
    println!("\nLoading texture atlas configuration...");
    let config_path = base_path.join("atlas_config.toml");
    
    if !config_path.exists() {
        println!("WARNING: Atlas configuration not found at {}", config_path.display());
        println!("Using texture directories directly instead of atlas");
    } else {
        println!("Atlas configuration found: {}", config_path.display());
        let mut texture_manager = TextureManager::new(&config_path)?;
        
        // 3. Generate placeholder textures for missing entries
        println!("\nChecking for missing textures and generating placeholders...");
        
        for (texture_id, entry) in texture_manager.get_all_textures() {
            let texture_path = Path::new("assets").join(&entry.diffuse_path);
            
            if !texture_path.exists() {
                println!("Missing texture: {} ({})", texture_id, texture_path.display());
                
                // Generate appropriate placeholder based on texture type
                let (size, color) = if texture_id.starts_with("terrain") {
                    (256, [150, 150, 150, 255])  // Gray for terrain
                } else if texture_id.starts_with("trees") {
                    (256, [100, 150, 100, 255])  // Green for trees
                } else if texture_id.starts_with("houses") {
                    (256, [150, 100, 80, 255])   // Brown for houses
                } else if texture_id.starts_with("characters") {
                    (256, [200, 150, 150, 255])  // Skin tone for characters
                } else if texture_id.starts_with("effects") {
                    (256, [100, 150, 200, 128])  // Semi-transparent blue for effects
                } else {
                    (256, [200, 200, 200, 255])  // Default white
                };
                
                // Create parent directory if needed
                if let Some(parent) = texture_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                // Generate and save placeholder
                texture_utils::save_placeholder_texture(&texture_path, size, color)?;
                
                // Also generate a placeholder normal map if needed
                if let Some(normal_path) = &entry.normal_path {
                    let normal_full_path = Path::new("assets").join(normal_path);
                    if !normal_full_path.exists() {
                        texture_utils::save_placeholder_texture(&normal_full_path, size, [128, 128, 255, 255])?;
                    }
                }
            }
        }
        
        // 4. Generate texture atlases
        println!("\nGenerating texture atlases...");
        texture_manager.save_atlases(base_path)?;
    }
    
    // 5. Test texture resolution functionality
    println!("\nTesting texture resolution...");
    let test_textures = [
        "grass.png",
        "dirt.ktx2",  // Should convert to PNG
        "nonexistent.png",  // Should fail
        "terrain/grass.png",  // Should work with subdirectory
        "textures/terrain/stone.png"  // Should work with full path
    ];
    
    for texture_name in &test_textures {
        match resolve_texture_path(Path::new("assets"), texture_name) {
            Some(path) => println!("Resolved '{}' to '{}'", texture_name, path.display()),
            None => println!("Could not resolve texture: {}", texture_name)
        }
    }
    
    // 6. Create material library with environment materials
    println!("\nCreating material library...");
    let mut material_library = MaterialLibrary::new();
    
    println!("\nCreating grassland materials...");
    material_library.create_environment_materials("grassland");
    
    println!("\nCreating desert materials...");
    material_library.create_environment_materials("desert");
    
    println!("\nCreating forest materials...");
    material_library.create_environment_materials("forest");
    
    // 7. Save material library as JSON
    println!("\nSaving material library...");
    match material_library.save_to_json() {
        Ok(json) => {
            let output_path = base_path.join("materials.json");
            fs::write(&output_path, json)?;
            println!("Material library saved to: {}", output_path.display());
        },
        Err(e) => println!("Error saving material library: {}", e)
    }
    
    println!("\nTexture system demo completed successfully!");
    println!("You can now use these textures in your unified_showcase application.");
    
    Ok(())
}