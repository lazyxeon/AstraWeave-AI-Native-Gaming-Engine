// Texture conversion utility for AstraWeave unified showcase
// This tool converts textures between different formats (PNG, KTX2, etc.)

use anyhow::Result;
use std::fs;
use std::path::Path;

// Utility to find normal maps based on texture naming convention
pub fn find_normal_map(albedo_path: &Path) -> Option<std::path::PathBuf> {
    // Look for _n suffix
    let file_stem = albedo_path.file_stem()?.to_str()?;
    let ext = albedo_path.extension()?.to_str()?;

    // First, try _n suffix (most common)
    let normal_path = albedo_path.with_file_name(format!("{}_n.{}", file_stem, ext));
    if normal_path.exists() {
        return Some(normal_path);
    }

    // Try name_n suffix
    let normal_path = albedo_path.with_file_name(format!("{}_n.{}", file_stem, ext));
    if normal_path.exists() {
        return Some(normal_path);
    }

    // Try name-normal suffix
    let normal_path = albedo_path.with_file_name(format!("{}-normal.{}", file_stem, ext));
    if normal_path.exists() {
        return Some(normal_path);
    }

    // Try without file extension change
    let name_without_ext = file_stem.to_string();
    if name_without_ext.ends_with(".png") || name_without_ext.ends_with(".ktx2") {
        let base_name = name_without_ext.rsplit_once('.').unwrap().0;
        let normal_path = albedo_path.with_file_name(format!("{}_n.{}", base_name, ext));
        if normal_path.exists() {
            return Some(normal_path);
        }
    }

    None
}

// Convert from KTX2 reference to PNG if needed
pub fn convert_texture_reference(texture_path: &str) -> String {
    if texture_path.ends_with(".ktx2") {
        texture_path.replace(".ktx2", ".png")
    } else {
        texture_path.to_string()
    }
}

// Check if a texture exists, try alternative formats if not
pub fn resolve_texture_path(base_path: &Path, texture_name: &str) -> Option<std::path::PathBuf> {
    let full_path = base_path.join(texture_name);

    if full_path.exists() {
        return Some(full_path);
    }

    // Try different extensions
    let extensions = [".png", ".ktx2", ".jpg", ".jpeg"];

    // Extract name without extension
    let file_stem = Path::new(texture_name).file_stem()?.to_str()?;

    for ext in extensions.iter() {
        let alternative_path = base_path.join(format!("{}{}", file_stem, ext));
        if alternative_path.exists() {
            return Some(alternative_path);
        }
    }

    // Try looking in texture/subfolders based on texture type
    let subfolders = ["terrain", "structures", "characters", "effects", "skybox"];

    for folder in subfolders.iter() {
        let subfolder_path = base_path.join(folder).join(texture_name);
        if subfolder_path.exists() {
            return Some(subfolder_path);
        }

        // Try different extensions in this subfolder too
        for ext in extensions.iter() {
            let alternative_path = base_path.join(folder).join(format!("{}{}", file_stem, ext));
            if alternative_path.exists() {
                return Some(alternative_path);
            }
        }
    }

    None
}

// Generate a simple placeholder texture when a texture is missing
pub fn generate_placeholder_texture(size: u32, color: [u8; 4]) -> Result<image::RgbaImage> {
    let mut img = image::RgbaImage::new(size, size);

    // Fill with base color
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba(color);
    }

    // Add a grid pattern for visibility
    let grid_size = size / 8;
    for y in 0..size {
        for x in 0..size {
            if (x / grid_size + y / grid_size) % 2 == 0 {
                let darker = [
                    (color[0] as f32 * 0.8) as u8,
                    (color[1] as f32 * 0.8) as u8,
                    (color[2] as f32 * 0.8) as u8,
                    color[3],
                ];
                img.put_pixel(x, y, image::Rgba(darker));
            }
        }
    }

    // Add an 'X' pattern
    for i in 0..size {
        let diag_pos = i;
        let inverse_pos = size - 1 - i;

        if diag_pos < size && inverse_pos < size {
            // Make the X lines 3 pixels thick
            for offset in -1..=1 {
                let x1 = (diag_pos as i32 + offset).clamp(0, size as i32 - 1) as u32;
                let x2 = (inverse_pos as i32 + offset).clamp(0, size as i32 - 1) as u32;

                img.put_pixel(x1, diag_pos, image::Rgba([0, 0, 0, 255]));
                img.put_pixel(x2, diag_pos, image::Rgba([0, 0, 0, 255]));
            }
        }
    }

    Ok(img)
}

// Save a placeholder texture when the original is not found
pub fn save_placeholder_texture(path: &Path, size: u32, color: [u8; 4]) -> Result<()> {
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let placeholder = generate_placeholder_texture(size, color)?;
    placeholder.save(path)?;

    println!("Created placeholder texture at: {}", path.display());
    Ok(())
}
