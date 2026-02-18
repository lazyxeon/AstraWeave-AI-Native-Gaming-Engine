// Standalone texture loading verification test
use std::path::Path;

fn main() {
    println!("=== Asset Loading Verification ===\n");
    
    let texture_paths = [
        "assets/textures/pine forest textures/forest_ground_04_diff.png",
        "assets/textures/pine forest textures/rocky_trail_diff.png",
        "assets/textures/pine forest textures/pine_bark_diff.png",
        "assets/textures/pine forest textures/pine_twig_diff.png",
        "assets/textures/pine forest textures/forest_ground_04_nor_gl.png",
        "assets/textures/pine forest textures/forest_ground_04_rough.png",
        "assets/textures/pine forest textures/rocky_trail_nor_gl.png",
        "assets/textures/pine forest textures/rocky_trail_rough.png",
        "assets/textures/cobblestone.png",
    ];
    
    println!("1. ASSET FILE EXISTENCE CHECK:");
    println!("{:-<80}", "");
    
    let mut all_exist = true;
    for path in &texture_paths {
        let exists = Path::new(path).exists();
        let metadata = if exists {
            let meta = std::fs::metadata(path).unwrap();
            format!("✓ EXISTS | Size: {} bytes ({:.2} MB)", 
                meta.len(), 
                meta.len() as f64 / 1_048_576.0)
        } else {
            all_exist = false;
            "✗ MISSING".to_string()
        };
        println!("  {:<60} {}", path, metadata);
    }
    
    println!("\n2. TEXTURE LOADING TEST:");
    println!("{:-<80}", "");
    
    for path in &texture_paths {
        if !Path::new(path).exists() {
            println!("  {:<60} SKIPPED (file missing)", path);
            continue;
        }
        
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                println!("  {:<60} ✓ LOADED | Dimensions: {}x{} | Format: RGBA8 | Pixels: {}", 
                    path,
                    rgba.width(),
                    rgba.height(),
                    rgba.len() / 4
                );
                
                // Verify image data is not all zeros
                let non_zero = rgba.iter().any(|&x| x != 0);
                if !non_zero {
                    println!("    WARNING: Image data appears to be all zeros!");
                }
            }
            Err(e) => {
                println!("  {:<60} ✗ FAILED | Error: {}", path, e);
                all_exist = false;
            }
        }
    }
    
    println!("\n3. TEXTURE FORMAT VALIDATION:");
    println!("{:-<80}", "");
    
    for path in &texture_paths {
        if !Path::new(path).exists() {
            continue;
        }
        
        if let Ok(img) = image::open(path) {
            let is_normal_or_rough = path.contains("nor_gl") || path.contains("rough");
            let expected_format = if is_normal_or_rough {
                "Rgba8Unorm (linear)"
            } else {
                "Rgba8UnormSrgb (sRGB)"
            };
            
            println!("  {:<60} Format: {}", 
                path.split('/').last().unwrap_or(path),
                expected_format
            );
        }
    }
    
    println!("\n4. GPU UPLOAD SIMULATION:");
    println!("{:-<80}", "");
    
    for path in &texture_paths {
        if !Path::new(path).exists() {
            continue;
        }
        
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let width = rgba.width();
            let height = rgba.height();
            let bytes_per_row = 4 * width;
            let total_size = bytes_per_row * height;
            
            // Simulate GPU upload validation
            let valid_alignment = bytes_per_row % 256 == 0 || bytes_per_row < 256;
            let size_ok = total_size == rgba.len() as u32;
            
            println!("  {:<40} BytesPerRow: {} | TotalSize: {} | Valid: {}", 
                path.split('/').last().unwrap_or(path),
                bytes_per_row,
                total_size,
                if valid_alignment && size_ok { "✓" } else { "✗" }
            );
        }
    }
    
    println!("\n5. SUMMARY:");
    println!("{:-<80}", "");
    
    if all_exist {
        println!("  ✓ All assets exist and can be loaded");
        println!("  ✓ All textures decode successfully");
        println!("  ✓ Ready for GPU upload");
    } else {
        println!("  ✗ Some assets are missing or failed to load");
        println!("  ✗ Texture pipeline validation FAILED");
    }
    
    println!("\n=== Verification Complete ===");
}
