// examples/unified_showcase/src/texture_synth.rs
use image::{ImageBuffer, Rgba};
use std::{fs, path::Path};

/// Public entry: ensure default textures exist; regen if `force` is true.
pub fn ensure_textures(out_dir: &str, seed: u32, force: bool) -> anyhow::Result<()> {
    fs::create_dir_all(out_dir)?;
    synth_if_missing(
        out_dir,
        "grass.png",
        seed.wrapping_add(101),
        force,
        synth_grass,
    )?;
    synth_mra_if_missing(out_dir, "grass_mra.png", 0.2, 0.7, 0.0, force)?;
    synth_if_missing(
        out_dir,
        "dirt.png",
        seed.wrapping_add(202),
        force,
        synth_dirt,
    )?;
    synth_mra_if_missing(out_dir, "dirt_mra.png", 0.9, 0.85, 0.0, force)?;
    synth_if_missing(
        out_dir,
        "sand.png",
        seed.wrapping_add(303),
        force,
        synth_sand,
    )?;
    synth_mra_if_missing(out_dir, "sand_mra.png", 0.8, 0.6, 0.0, force)?;
    synth_if_missing(
        out_dir,
        "stone.png",
        seed.wrapping_add(404),
        force,
        synth_stone,
    )?;
    synth_mra_if_missing(out_dir, "stone_mra.png", 0.6, 0.9, 0.05, force)?;
    // Add enhanced forest floor texture for deeper biome variety
    synth_if_missing(
        out_dir,
        "forest_floor.png",
        seed.wrapping_add(505),
        force,
        synth_forest_floor,
    )?;
    synth_mra_if_missing(out_dir, "forest_floor_mra.png", 0.9, 0.8, 0.0, force)?;
    Ok(())
}

fn synth_if_missing<F: Fn(u32, u32, u32) -> ImageBuffer<Rgba<u8>, Vec<u8>>>(
    out_dir: &str,
    name: &str,
    seed: u32,
    force: bool,
    f: F,
) -> anyhow::Result<()> {
    let path = Path::new(out_dir).join(name);
    if force || !path.exists() {
        // Enhanced texture resolution for better 3D biome quality
        let img = f(2048, 2048, seed);
        img.save(&path)?;
        // Enhanced normal map generation with improved quality
        if name.ends_with("grass.png") || name.ends_with("dirt.png") || name.ends_with("sand.png") 
           || name.ends_with("stone.png") || name.ends_with("forest_floor.png")
        {
            let npath = Path::new(out_dir).join(name.replace(".png", "_n.png"));
            // Different strength values for different texture types
            let normal_strength = match name {
                n if n.ends_with("grass.png") => 1.8,         // Subtle for grass
                n if n.ends_with("dirt.png") => 2.2,          // Medium for dirt
                n if n.ends_with("sand.png") => 1.5,          // Gentle for sand
                n if n.ends_with("stone.png") => 3.0,         // Strong for stone
                n if n.ends_with("forest_floor.png") => 2.5,  // Rich detail for forest floor
                _ => 2.5,
            };
            let normal = height_to_normal(&img, normal_strength);
            normal.save(&npath)?;
        }
    }
    Ok(())
}

fn synth_mra_if_missing(
    out_dir: &str,
    name: &str,
    ao: f32,
    roughness: f32,
    metallic: f32,
    force: bool,
) -> anyhow::Result<()> {
    let path = Path::new(out_dir).join(name);
    if force || !path.exists() {
        let w = 4u32; // tiny; GPU will sample fine
        let h = 4u32;
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(w, h);
        let to_u8 = |v: f32| ((v.clamp(0.0, 1.0)) * 255.0) as u8;
        let px = Rgba([to_u8(ao), to_u8(roughness), to_u8(metallic), 255]);
        for y in 0..h { for x in 0..w { img.put_pixel(x, y, px); } }
        img.save(&path)?;
    }
    Ok(())
}

// -------- Simple fractal noise ----------
fn hash(mut x: u32) -> u32 {
    x ^= x >> 17;
    x = x.wrapping_mul(0xed5ad4bb);
    x ^= x >> 11;
    x = x.wrapping_mul(0xac4c1b51);
    x ^= x >> 15;
    x = x.wrapping_mul(0x31848bab);
    x ^= x >> 14;
    x
}

fn noise2d(x: i32, y: i32, seed: u32) -> f32 {
    let h = hash(x as u32 ^ (y as u32).rotate_left(16) ^ seed);
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn fbm(x: f32, y: f32, seed: u32, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    let (mut f, mut a, mut sum, mut norm) = (1.0, 0.5, 0.0, 0.0);
    for i in 0..octaves {
        let n = smooth_noise(x * f, y * f, seed.wrapping_add(i as u32));
        sum += a * n;
        norm += a;
        f *= lacunarity;
        a *= gain;
    }
    sum / norm.max(1e-6)
}

fn smooth_noise(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let xf = x - x0 as f32;
    let yf = y - y0 as f32;

    let n00 = noise2d(x0, y0, seed);
    let n10 = noise2d(x0 + 1, y0, seed);
    let n01 = noise2d(x0, y0 + 1, seed);
    let n11 = noise2d(x0 + 1, y0 + 1, seed);

    let sx = xf * xf * (3.0 - 2.0 * xf);
    let sy = yf * yf * (3.0 - 2.0 * yf);
    let ix0 = n00 * (1.0 - sx) + n10 * sx;
    let ix1 = n01 * (1.0 - sx) + n11 * sx;
    ix0 * (1.0 - sy) + ix1 * sy
}

// -------- Materials ----------
fn synth_grass(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 32.0;
            let v = y as f32 / h as f32 * 32.0;
            
            // Enhanced grass pattern with multiple layers
            let base = fbm(u, v, seed, 6, 2.05, 0.52);
            let clump = fbm(u * 0.35, v * 0.35, seed ^ 0x55aa, 4, 2.0, 0.6);
            let fine_detail = fbm(u * 8.0, v * 8.0, seed ^ 0x1337, 3, 2.0, 0.4);
            let dirt_patches = fbm(u * 0.8, v * 0.8, seed ^ 0xcafe, 5, 2.0, 0.5);
            
            // Height variation with patches of dirt showing through
            let height = (0.55 + 0.35 * base + 0.30 * clump + 0.1 * fine_detail).clamp(0.0, 1.0);
            let dirt_factor = (dirt_patches > 0.3) as i32 as f32 * 0.3;
            
            // Enhanced grass coloration with seasonal variation
            let grass_green = 90.0 + 120.0 * height;
            let grass_yellow = 70.0 + 40.0 * (1.0 - height);
            let dirt_brown = 65.0 + 45.0 * height;
            
            let (r, g, b) = if dirt_factor > 0.0 {
                // Dirt patches mixed with grass
                let mix = 0.7;
                (
                    ((grass_green * 0.35 + grass_yellow * 0.25) * mix + dirt_brown * (1.0 - mix)) as u8,
                    (grass_green * mix + dirt_brown * 0.6 * (1.0 - mix)) as u8,
                    (grass_green * 0.2 * mix + dirt_brown * 0.4 * (1.0 - mix)) as u8,
                )
            } else {
                // Pure grass areas
                (
                    (grass_green * 0.35 + grass_yellow * 0.25) as u8,
                    grass_green as u8,
                    (grass_green * 0.2) as u8,
                )
            };
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_dirt(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 16.0;
            let v = y as f32 / h as f32 * 16.0;
            let grains = fbm(u * 1.4, v * 1.2, seed ^ 0xdead00, 6, 2.0, 0.5);
            let pebbles = fbm(u * 0.35, v * 0.35, seed ^ 0xbeef11, 3, 2.0, 0.55);
            let height = (0.5 + 0.4 * grains + 0.25 * pebbles).clamp(0.0, 1.0);
            let r = (60.0 + 110.0 * height) as u8;
            let g = (45.0 + 65.0 * height) as u8;
            let b = (35.0 + 45.0 * height) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_sand(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 20.0;
            let v = y as f32 / h as f32 * 20.0;
            
            // Enhanced sand patterns with multiple detail levels
            let fine_grains = fbm(u * 4.0, v * 4.0, seed ^ 0xc0ff33, 7, 2.0, 0.5);
            let dune_patterns = fbm(u * 0.3, v * 0.3, seed ^ 0xfade01, 4, 2.0, 0.6);
            let wind_ripples = fbm(u * 12.0, v * 2.0, seed ^ 0x7ead33, 5, 2.0, 0.4);
            let large_formations = fbm(u * 0.1, v * 0.1, seed ^ 0xbeac44, 3, 2.0, 0.7);
            let micro_detail = fbm(u * 16.0, v * 16.0, seed ^ 0x5a9d17, 4, 2.0, 0.3);
            
            // Combine patterns for realistic sand height variation
            let height = (0.5 + 0.25 * fine_grains + 0.2 * dune_patterns + 0.15 * wind_ripples 
                         + 0.1 * large_formations + 0.08 * micro_detail).clamp(0.0, 1.0);
            
            // Enhanced sand coloration with mineral variation
            let base_r = 210.0 + 35.0 * height;
            let base_g = 185.0 + 45.0 * height;
            let base_b = 135.0 + 30.0 * height;
            
            // Add mineral deposits and color variation
            let mineral_factor = fbm(u * 0.7, v * 0.7, seed ^ 0x9abe7a, 3, 2.0, 0.6);
            let iron_tint = if mineral_factor > 0.6 { 0.15 } else { 0.0 };
            
            let r = (base_r + iron_tint * 20.0).min(255.0) as u8;
            let g = (base_g - iron_tint * 10.0).max(0.0) as u8;
            let b = (base_b - iron_tint * 25.0).max(0.0) as u8;
            
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_stone(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 10.0;
            let v = y as f32 / h as f32 * 10.0;
            
            // Enhanced stone patterns with geological realism
            let veins = fbm(u * 1.6, v * 1.3, seed ^ 0x7777, 7, 2.1, 0.5);
            let base = fbm(u, v, seed ^ 0x1111, 6, 2.0, 0.55);
            let cracks = fbm(u * 3.2, v * 3.2, seed ^ 0xc7ac4, 5, 2.0, 0.4);
            let weathering = fbm(u * 0.5, v * 0.5, seed ^ 0xaea754, 4, 2.0, 0.6);
            let fine_texture = fbm(u * 8.0, v * 8.0, seed ^ 0xf19e7, 4, 2.0, 0.3);
            
            // Combine for realistic stone height variation
            let height = (0.55 + 0.3 * base + 0.2 * veins - 0.1 * cracks 
                         + 0.1 * weathering + 0.05 * fine_texture).clamp(0.0, 1.0);
            
            // Enhanced stone coloration with mineral variation
            let base_gray = 130.0 + 80.0 * height;
            let vein_brightness = if veins > 0.5 { 25.0 } else { 0.0 };
            let crack_darkness = if cracks > 0.7 { -20.0 } else { 0.0 };
            
            // Color variation for different stone types
            let stone_type = fbm(u * 0.2, v * 0.2, seed ^ 0x79e34, 2, 2.0, 0.5);
            let (r_tint, g_tint, b_tint) = if stone_type > 0.3 {
                (10.0, 5.0, -5.0) // Slightly warmer stone
            } else if stone_type < -0.3 {
                (-5.0, 0.0, 10.0) // Slightly cooler stone
            } else {
                (0.0, 0.0, 0.0) // Neutral gray stone
            };
            
            let r = (base_gray + vein_brightness + crack_darkness + r_tint).clamp(0.0, 255.0) as u8;
            let g = (base_gray + vein_brightness + crack_darkness + g_tint).clamp(0.0, 255.0) as u8;
            let b = (base_gray + vein_brightness + crack_darkness + b_tint).clamp(0.0, 255.0) as u8;
            
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

// -------- Height → Normal (Sobel) ----------
fn height_to_normal(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    strength: f32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();
    let mut out = ImageBuffer::new(w, h);
    let h_sample = |x: i32, y: i32| -> f32 {
        let xi = ((x % w as i32) + w as i32) % w as i32;
        let yi = ((y % h as i32) + h as i32) % h as i32;
        let p = img.get_pixel(xi as u32, yi as u32);
        // luminance as height
        (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) / 255.0
    };
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = (h_sample(x + 1, y) - h_sample(x - 1, y)) * strength;
            let dy = (h_sample(x, y + 1) - h_sample(x, y - 1)) * strength;
            // normal from gradient (Tangent X -> +x, Tangent Y -> +y, Up -> +z)
            let mut nx = -dx;
            let mut ny = -dy;
            let mut nz = 1.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            nx /= len;
            ny /= len;
            nz /= len;
            let r = ((nx * 0.5 + 0.5) * 255.0) as u8;
            let g = ((ny * 0.5 + 0.5) * 255.0) as u8;
            let b = ((nz * 0.5 + 0.5) * 255.0) as u8;
            out.put_pixel(x as u32, y as u32, Rgba([r, g, b, 255]));
        }
    }
    out
}

/// Enhanced forest floor texture with leaves, moss, and organic material
fn synth_forest_floor(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 24.0;
            let v = y as f32 / h as f32 * 24.0;
            
            // Multi-layered forest floor composition
            let leaf_litter = fbm(u * 2.5, v * 2.5, seed ^ 0x1eaf7, 6, 2.0, 0.5);
            let moss_patches = fbm(u * 1.2, v * 1.2, seed ^ 0x90557, 5, 2.0, 0.6);
            let soil_base = fbm(u * 0.8, v * 0.8, seed ^ 0x5011, 4, 2.0, 0.55);
            let twigs_debris = fbm(u * 6.0, v * 6.0, seed ^ 0x791a7, 4, 2.0, 0.4);
            let decomposition = fbm(u * 0.3, v * 0.3, seed ^ 0xdeca7, 3, 2.0, 0.7);
            
            // Height variation from organic matter
            let height = (0.5 + 0.25 * leaf_litter + 0.2 * moss_patches + 0.15 * soil_base
                         + 0.1 * twigs_debris + 0.1 * decomposition).clamp(0.0, 1.0);
            
            // Color composition based on forest floor materials
            let moss_factor = (moss_patches > 0.2) as i32 as f32;
            let decay_factor = (decomposition > 0.4) as i32 as f32;
            
            // Base brown soil colors
            let soil_r = 85.0 + 40.0 * height;
            let soil_g = 65.0 + 35.0 * height;
            let soil_b = 35.0 + 25.0 * height;
            
            // Moss green tinting
            let moss_r = 45.0 + 30.0 * height;
            let moss_g = 95.0 + 60.0 * height;
            let moss_b = 35.0 + 20.0 * height;
            
            // Decayed organic matter (darker)
            let decay_r = 35.0 + 20.0 * height;
            let decay_g = 30.0 + 15.0 * height;
            let decay_b = 20.0 + 10.0 * height;
            
            // Blend colors based on composition
            let (r, g, b) = if moss_factor > 0.0 && decay_factor > 0.0 {
                // Mixed moss and decay
                (
                    (soil_r * 0.4 + moss_r * 0.3 + decay_r * 0.3) as u8,
                    (soil_g * 0.4 + moss_g * 0.3 + decay_g * 0.3) as u8,
                    (soil_b * 0.4 + moss_b * 0.3 + decay_b * 0.3) as u8,
                )
            } else if moss_factor > 0.0 {
                // Mossy areas
                (
                    (soil_r * 0.6 + moss_r * 0.4) as u8,
                    (soil_g * 0.6 + moss_g * 0.4) as u8,
                    (soil_b * 0.6 + moss_b * 0.4) as u8,
                )
            } else if decay_factor > 0.0 {
                // Decayed organic matter
                (
                    (soil_r * 0.7 + decay_r * 0.3) as u8,
                    (soil_g * 0.7 + decay_g * 0.3) as u8,
                    (soil_b * 0.7 + decay_b * 0.3) as u8,
                )
            } else {
                // Regular forest soil
                (soil_r as u8, soil_g as u8, soil_b as u8)
            };
            
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}
