use std::path::Path;
#[path = "../texture_generator.rs"]
mod texture_generator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AstraWeave Unified Showcase - Texture Generator");
    println!("==============================================");

    let output_path = Path::new("assets/textures");
    println!("Generating textures to: {}", output_path.display());

    // Generate all textures with 512x512 resolution
    texture_generator::generate_all_textures(output_path, 512)?;

    println!("All textures generated successfully!");
    println!("Remember to update your texture bindings in the main.rs file.");

    Ok(())
}
