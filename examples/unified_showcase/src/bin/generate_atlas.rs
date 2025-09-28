// Atlas generator utility for AstraWeave unified showcase
// This tool generates texture atlases from individual textures based on configuration

#[path = "../texture_manager.rs"]
mod texture_manager;

use anyhow::Result;
use std::path::Path;
use texture_manager::TextureManager;

fn main() -> Result<()> {
    println!("AstraWeave Unified Showcase - Texture Atlas Generator");
    println!("===================================================");

    // Load the atlas configuration
    let config_path = Path::new("assets/textures/atlas_config.toml");

    println!(
        "Loading texture atlas configuration from: {}",
        config_path.display()
    );
    let mut texture_manager = TextureManager::new(config_path)?;

    // Base path for texture loading
    let base_path = Path::new("assets");

    // Preload all textures
    println!("Preloading textures...");
    texture_manager.preload_all_textures(base_path)?;

    // Generate and save the atlases
    println!("Generating texture atlases...");
    let output_path = Path::new("assets/textures");
    texture_manager.save_atlases(output_path)?;

    println!("All texture atlases generated successfully!");
    println!("Location: {}", output_path.display());
    println!("Remember to update your shader bindings to use the new atlas textures.");

    Ok(())
}
