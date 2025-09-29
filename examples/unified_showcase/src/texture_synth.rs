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
    synth_if_missing(
        out_dir,
        "tree_bark.png",
        seed.wrapping_add(606),
        force,
        synth_tree_bark,
    )?;
    synth_mra_if_missing(out_dir, "tree_bark_mra.png", 0.85, 0.65, 0.12, force)?;
    synth_if_missing(
        out_dir,
        "tree_leaves.png",
        seed.wrapping_add(707),
        force,
        synth_tree_leaves,
    )?;
    synth_mra_if_missing(out_dir, "tree_leaves_mra.png", 0.55, 0.35, 0.02, force)?;
    synth_if_missing(
        out_dir,
        "rock_lichen.png",
        seed.wrapping_add(808),
        force,
        synth_rock_lichen,
    )?;
    synth_mra_if_missing(out_dir, "rock_lichen_mra.png", 0.75, 0.6, 0.08, force)?;
    synth_if_missing(
        out_dir,
        "rock_slate.png",
        seed.wrapping_add(909),
        force,
        synth_rock_slate,
    )?;
    synth_mra_if_missing(out_dir, "rock_slate_mra.png", 0.65, 0.85, 0.12, force)?;
    synth_if_missing(
        out_dir,
        "plaster.png",
        seed.wrapping_add(1010),
        force,
        synth_plaster,
    )?;
    synth_mra_if_missing(out_dir, "plaster_mra.png", 0.95, 0.55, 0.02, force)?;
    synth_if_missing(
        out_dir,
        "roof_tile.png",
        seed.wrapping_add(1111),
        force,
        synth_roof_tile,
    )?;
    synth_mra_if_missing(out_dir, "roof_tile_mra.png", 0.7, 0.45, 0.18, force)?;
    synth_if_missing(
        out_dir,
        "cloth.png",
        seed.wrapping_add(1212),
        force,
        synth_cloth,
    )?;
    synth_mra_if_missing(out_dir, "cloth_mra.png", 0.6, 0.5, 0.05, force)?;
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
        // Enhanced texture resolution for photorealistic 3D biome quality
        let img = f(4096, 4096, seed);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        img.save(&path)?;
        // Enhanced normal map generation with improved quality
        if name.ends_with("grass.png")
            || name.ends_with("dirt.png")
            || name.ends_with("sand.png")
            || name.ends_with("stone.png")
            || name.ends_with("forest_floor.png")
            || name.ends_with("tree_bark.png")
            || name.ends_with("tree_leaves.png")
            || name.ends_with("rock_lichen.png")
            || name.ends_with("rock_slate.png")
            || name.ends_with("plaster.png")
            || name.ends_with("roof_tile.png")
            || name.ends_with("cloth.png")
        {
            let npath = Path::new(out_dir).join(name.replace(".png", "_n.png"));
            // Different strength values for different texture types
            let normal_strength = match name {
                n if n.ends_with("grass.png") => 1.8,        // Subtle for grass
                n if n.ends_with("dirt.png") => 2.2,         // Medium for dirt
                n if n.ends_with("sand.png") => 1.5,         // Gentle for sand
                n if n.ends_with("stone.png") => 3.0,        // Strong for stone
                n if n.ends_with("forest_floor.png") => 2.5, // Rich detail for forest floor
                n if n.ends_with("tree_bark.png") => 3.4,    // Deep grooves for bark
                n if n.ends_with("tree_leaves.png") => 1.6,  // Softer leaf structure
                n if n.ends_with("rock_lichen.png") => 2.8,  // Patchy lichen height
                n if n.ends_with("rock_slate.png") => 3.2,   // Layered slate relief
                n if n.ends_with("plaster.png") => 1.2,      // Subtle plaster bumps
                n if n.ends_with("roof_tile.png") => 2.4,    // Tile bevels
                n if n.ends_with("cloth.png") => 1.4,        // Fabric weave
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
        for y in 0..h {
            for x in 0..w {
                img.put_pixel(x, y, px);
            }
        }
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
            let u = x as f32 / w as f32 * 64.0; // Higher frequency for more detail
            let v = y as f32 / h as f32 * 64.0;

            // Multi-layered grass pattern with enhanced realism
            let base = fbm(u, v, seed, 8, 2.1, 0.45); // More octaves for detail
            let clump = fbm(u * 0.25, v * 0.25, seed ^ 0x55aa, 6, 2.0, 0.6);
            let fine_detail = fbm(u * 12.0, v * 12.0, seed ^ 0x1337, 5, 2.0, 0.3);
            let dirt_patches = fbm(u * 1.2, v * 1.2, seed ^ 0xcafe, 7, 2.0, 0.4);
            let micro_variation = fbm(u * 32.0, v * 32.0, seed ^ 0xfeed, 4, 2.0, 0.2);
            let seasonal_variation = fbm(u * 0.5, v * 0.5, seed ^ 0xdead, 3, 2.0, 0.5);

            // Height variation with patches of dirt showing through
            let height = (0.6
                + 0.25 * base
                + 0.2 * clump
                + 0.08 * fine_detail
                + 0.05 * micro_variation
                + 0.02 * seasonal_variation)
                .clamp(0.0, 1.0);
            let dirt_factor = (dirt_patches > 0.4) as i32 as f32 * 0.25;

            // Enhanced grass coloration with seasonal and health variation
            let grass_green = 110.0 + 120.0 * height + seasonal_variation * 30.0;
            let grass_yellow = 80.0 + 50.0 * (1.0 - height) - seasonal_variation * 20.0;
            let dirt_brown_r = 75.0 + 35.0 * height;
            let dirt_brown_g = 55.0 + 25.0 * height;
            let dirt_brown_b = 35.0 + 15.0 * height;

            let (r, g, b) = if dirt_factor > 0.0 {
                // Dirt patches mixed with grass - more realistic blending
                let mix = dirt_factor;
                let grass_r = grass_green * 0.3 + grass_yellow * 0.2;
                let grass_g = grass_green;
                let grass_b = grass_green * 0.15;

                (
                    ((grass_r * (1.0 - mix) + dirt_brown_r * mix) as u8).min(255),
                    ((grass_g * (1.0 - mix) + dirt_brown_g * mix) as u8).min(255),
                    ((grass_b * (1.0 - mix) + dirt_brown_b * mix) as u8).min(255),
                )
            } else {
                // Pure grass areas with subtle color variation
                (
                    (grass_green * 0.3 + grass_yellow * 0.2) as u8,
                    grass_green as u8,
                    (grass_green * 0.15) as u8,
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
            let u = x as f32 / w as f32 * 32.0; // Higher frequency for more detail
            let v = y as f32 / h as f32 * 32.0;

            // Enhanced sand patterns with multiple detail levels and realistic dune formation
            let fine_grains = fbm(u * 6.0, v * 6.0, seed ^ 0xc0ff33, 9, 2.0, 0.45);
            let dune_patterns = fbm(u * 0.4, v * 0.4, seed ^ 0xfade01, 6, 2.0, 0.55);
            let wind_ripples = fbm(u * 16.0, v * 3.0, seed ^ 0x7ead33, 7, 2.0, 0.35);
            let large_formations = fbm(u * 0.15, v * 0.15, seed ^ 0xbeac44, 4, 2.0, 0.65);
            let micro_detail = fbm(u * 24.0, v * 24.0, seed ^ 0x5a9d17, 6, 2.0, 0.25);
            let mineral_deposits = fbm(u * 1.2, v * 1.2, seed ^ 0xfeed, 5, 2.0, 0.4);
            let erosion_patterns = fbm(u * 8.0, v * 8.0, seed ^ 0xdead, 4, 2.0, 0.3);

            // Combine patterns for realistic sand height variation
            let height = (0.55
                + 0.18 * fine_grains
                + 0.15 * dune_patterns
                + 0.12 * wind_ripples
                + 0.08 * large_formations
                + 0.06 * micro_detail
                + 0.04 * erosion_patterns)
                .clamp(0.0, 1.0);

            // Enhanced sand coloration with mineral variation and realistic color gradients
            let base_r = 220.0 + 30.0 * height;
            let base_g = 195.0 + 40.0 * height;
            let base_b = 140.0 + 25.0 * height;

            // Mineral deposits and color variation
            let mineral_factor = mineral_deposits;
            let iron_tint = if mineral_factor > 0.6 { 0.2 } else { 0.0 };
            let clay_tint = if mineral_factor > 0.4 && mineral_factor < 0.6 {
                0.15
            } else {
                0.0
            };

            // Wind erosion effects
            let erosion_factor = erosion_patterns;
            let wind_darkening = if erosion_factor > 0.7 { -15.0 } else { 0.0 };

            // Dune shadow effects
            let dune_shadow = dune_patterns * 10.0;

            let r = (base_r + iron_tint * 30.0 + clay_tint * 15.0 + wind_darkening + dune_shadow)
                .clamp(0.0, 255.0) as u8;
            let g = (base_g - iron_tint * 15.0
                + clay_tint * 20.0
                + wind_darkening * 0.8
                + dune_shadow * 0.6)
                .clamp(0.0, 255.0) as u8;
            let b = (base_b - iron_tint * 35.0
                + clay_tint * 10.0
                + wind_darkening * 0.6
                + dune_shadow * 0.4)
                .clamp(0.0, 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_stone(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 16.0; // Higher frequency for more detail
            let v = y as f32 / h as f32 * 16.0;

            // Enhanced stone patterns with geological realism and multiple rock types
            let veins = fbm(u * 2.2, v * 2.0, seed ^ 0x7777, 9, 2.1, 0.45);
            let base = fbm(u, v, seed ^ 0x1111, 8, 2.0, 0.5);
            let cracks = fbm(u * 4.0, v * 4.0, seed ^ 0xc7ac4, 6, 2.0, 0.35);
            let weathering = fbm(u * 0.6, v * 0.6, seed ^ 0xaea754, 5, 2.0, 0.55);
            let fine_texture = fbm(u * 10.0, v * 10.0, seed ^ 0xf19e7, 5, 2.0, 0.25);
            let crystal_inclusions = fbm(u * 6.0, v * 6.0, seed ^ 0xdead, 4, 2.0, 0.3);
            let sedimentary_layers = fbm(u * 0.3, v * 0.8, seed ^ 0xbeef, 3, 2.0, 0.6);

            // Combine for realistic stone height variation with more complexity
            let height = (0.6 + 0.25 * base + 0.15 * veins - 0.08 * cracks
                + 0.08 * weathering
                + 0.04 * fine_texture
                + 0.03 * crystal_inclusions
                + 0.02 * sedimentary_layers)
                .clamp(0.0, 1.0);

            // Enhanced stone coloration with mineral variation and rock types
            let base_gray = 140.0 + 70.0 * height;
            let vein_brightness = if veins > 0.5 { 30.0 } else { 0.0 };
            let crack_darkness = if cracks > 0.6 { -25.0 } else { 0.0 };
            let weathering_effect = weathering * 15.0;

            // Color variation for different stone types based on position
            let stone_type_u = fbm(u * 0.1, v * 0.1, seed ^ 0x79e34, 2, 2.0, 0.5);
            let stone_type_v = fbm(u * 0.08, v * 0.12, seed ^ 0xfeed, 2, 2.0, 0.5);

            let (r_tint, g_tint, b_tint) = if stone_type_u > 0.6 {
                // Granite-like with quartz veins
                (15.0, 10.0, 5.0)
            } else if stone_type_u < 0.2 {
                // Basalt-like, darker
                (-10.0, -5.0, 5.0)
            } else if stone_type_v > 0.7 {
                // Limestone-like, slightly yellowish
                (8.0, 5.0, -5.0)
            } else if stone_type_v < 0.3 {
                // Sandstone-like, reddish
                (12.0, -2.0, -8.0)
            } else {
                // Standard gray stone
                (0.0, 0.0, 0.0)
            };

            // Crystal inclusions add sparkle
            let crystal_effect = if crystal_inclusions > 0.7 { 20.0 } else { 0.0 };

            let r = (base_gray
                + vein_brightness
                + crack_darkness
                + weathering_effect
                + r_tint
                + crystal_effect)
                .clamp(0.0, 255.0) as u8;
            let g = (base_gray
                + vein_brightness
                + crack_darkness
                + weathering_effect
                + g_tint
                + crystal_effect * 0.8)
                .clamp(0.0, 255.0) as u8;
            let b = (base_gray
                + vein_brightness
                + crack_darkness
                + weathering_effect
                + b_tint
                + crystal_effect * 0.6)
                .clamp(0.0, 255.0) as u8;

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
            let u = x as f32 / w as f32 * 36.0; // Higher frequency for more detail
            let v = y as f32 / h as f32 * 36.0;

            // Multi-layered forest floor composition with enhanced realism
            let leaf_litter = fbm(u * 3.5, v * 3.5, seed ^ 0x1eaf7, 8, 2.0, 0.45);
            let moss_patches = fbm(u * 1.8, v * 1.8, seed ^ 0x90557, 7, 2.0, 0.5);
            let soil_base = fbm(u * 1.2, v * 1.2, seed ^ 0x5011, 6, 2.0, 0.55);
            let twigs_debris = fbm(u * 8.0, v * 8.0, seed ^ 0x791a7, 6, 2.0, 0.35);
            let decomposition = fbm(u * 0.4, v * 0.4, seed ^ 0xdeca7, 5, 2.0, 0.65);
            let fungal_growth = fbm(u * 12.0, v * 12.0, seed ^ 0xabcd, 4, 2.0, 0.3);
            let root_exposure = fbm(u * 2.5, v * 6.0, seed ^ 0x1234, 3, 2.0, 0.4);

            // Height variation from organic matter with more complexity
            let height = (0.55
                + 0.2 * leaf_litter
                + 0.15 * moss_patches
                + 0.12 * soil_base
                + 0.08 * twigs_debris
                + 0.08 * decomposition
                + 0.04 * fungal_growth
                + 0.03 * root_exposure)
                .clamp(0.0, 1.0);

            // Color composition based on forest floor materials with seasonal variation
            let moss_factor = (moss_patches > 0.25) as i32 as f32;
            let decay_factor = (decomposition > 0.45) as i32 as f32;
            let fungal_factor = (fungal_growth > 0.6) as i32 as f32;
            let root_factor = (root_exposure > 0.5) as i32 as f32;

            // Base brown soil colors with organic variation
            let soil_r = 95.0 + 35.0 * height;
            let soil_g = 75.0 + 30.0 * height;
            let soil_b = 45.0 + 20.0 * height;

            // Moss green tinting with seasonal variation
            let moss_r = 55.0 + 25.0 * height;
            let moss_g = 105.0 + 50.0 * height;
            let moss_b = 45.0 + 15.0 * height;

            // Decayed organic matter (darker, richer)
            let decay_r = 45.0 + 15.0 * height;
            let decay_g = 40.0 + 12.0 * height;
            let decay_b = 25.0 + 8.0 * height;

            // Fungal growth (subtle color shifts)
            let fungal_r = 85.0 + 20.0 * height;
            let fungal_g = 65.0 + 15.0 * height;
            let fungal_b = 55.0 + 10.0 * height;

            // Root exposure (darker, more soil-like)
            let root_r = 65.0 + 25.0 * height;
            let root_g = 55.0 + 20.0 * height;
            let root_b = 35.0 + 15.0 * height;

            // Blend colors based on composition with priority system
            let (r, g, b) = if fungal_factor > 0.0 {
                // Fungal growth takes priority
                (
                    (soil_r * 0.3 + fungal_r * 0.7) as u8,
                    (soil_g * 0.3 + fungal_g * 0.7) as u8,
                    (soil_b * 0.3 + fungal_b * 0.7) as u8,
                )
            } else if root_factor > 0.0 {
                // Root exposure
                (
                    (soil_r * 0.4 + root_r * 0.6) as u8,
                    (soil_g * 0.4 + root_g * 0.6) as u8,
                    (soil_b * 0.4 + root_b * 0.6) as u8,
                )
            } else if moss_factor > 0.0 && decay_factor > 0.0 {
                // Mixed moss and decay
                (
                    (soil_r * 0.3 + moss_r * 0.35 + decay_r * 0.35) as u8,
                    (soil_g * 0.3 + moss_g * 0.35 + decay_g * 0.35) as u8,
                    (soil_b * 0.3 + moss_b * 0.35 + decay_b * 0.35) as u8,
                )
            } else if moss_factor > 0.0 {
                // Mossy areas
                (
                    (soil_r * 0.5 + moss_r * 0.5) as u8,
                    (soil_g * 0.5 + moss_g * 0.5) as u8,
                    (soil_b * 0.5 + moss_b * 0.5) as u8,
                )
            } else if decay_factor > 0.0 {
                // Decayed organic matter
                (
                    (soil_r * 0.6 + decay_r * 0.4) as u8,
                    (soil_g * 0.6 + decay_g * 0.4) as u8,
                    (soil_b * 0.6 + decay_b * 0.4) as u8,
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

fn synth_tree_bark(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 18.0;
            let v = y as f32 / h as f32 * 6.0;
            let ridges = fbm(u * 2.4, v * 0.8, seed ^ 0xBAA5, 7, 2.0, 0.6);
            let cracks = fbm(u * 4.5, v * 2.0, seed ^ 0xC0DE, 6, 2.2, 0.45);
            let knots = fbm(u * 0.6, v * 0.8, seed ^ 0x77AA, 4, 2.0, 0.5);
            let moss = fbm(u * 1.1, v * 3.0, seed ^ 0x1357, 5, 2.0, 0.4);

            let height = (0.55 + 0.25 * ridges - 0.12 * cracks + 0.08 * knots).clamp(0.0, 1.0);
            let moss_mask = (moss > 0.45) as i32 as f32 * 0.35;

            let base_r = 96.0 + 60.0 * height;
            let base_g = 62.0 + 38.0 * height;
            let base_b = 42.0 + 30.0 * height;

            let moss_r = 65.0 + 25.0 * height;
            let moss_g = 110.0 + 50.0 * height;
            let moss_b = 60.0 + 20.0 * height;

            let r = (base_r * (1.0 - moss_mask) + moss_r * moss_mask).clamp(0.0, 255.0) as u8;
            let g = (base_g * (1.0 - moss_mask) + moss_g * moss_mask).clamp(0.0, 255.0) as u8;
            let b = (base_b * (1.0 - moss_mask) + moss_b * moss_mask).clamp(0.0, 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_tree_leaves(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 28.0;
            let v = y as f32 / h as f32 * 28.0;
            let canopy = fbm(u, v, seed ^ 0x5151, 8, 2.1, 0.5);
            let veins = fbm(u * 3.6, v * 3.6, seed ^ 0xA1A1, 6, 2.0, 0.35);
            let sun_bleach = fbm(u * 0.4, v * 0.4, seed ^ 0xFEED, 3, 2.0, 0.55);
            let moisture = fbm(u * 1.5, v * 1.5, seed ^ 0xB00B, 5, 2.0, 0.45);

            let height = (0.58 + 0.2 * canopy + 0.12 * veins + 0.05 * moisture).clamp(0.0, 1.0);
            let highlight = (sun_bleach + 1.0) * 0.4 + 0.6;

            let r = (46.0 + 45.0 * height * highlight + 12.0 * moisture).clamp(0.0, 255.0) as u8;
            let g = (88.0 + 105.0 * height * highlight + 20.0 * moisture).clamp(0.0, 255.0) as u8;
            let b = (42.0 + 60.0 * height * highlight).clamp(0.0, 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_rock_lichen(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 24.0;
            let v = y as f32 / h as f32 * 24.0;
            let rock = fbm(u * 1.4, v * 1.4, seed ^ 0x6D6D, 7, 2.0, 0.45);
            let lichen = fbm(u * 3.8, v * 3.8, seed ^ 0xAB12, 6, 2.0, 0.35);
            let speckles = fbm(u * 9.0, v * 9.0, seed ^ 0xCAFE, 5, 2.0, 0.3);
            let moisture = fbm(u * 0.7, v * 0.7, seed ^ 0xF00D, 4, 2.0, 0.5);

            let base_gray = 120.0 + 60.0 * rock;
            let lichen_mask = (lichen > 0.15) as i32 as f32 * (0.5 + 0.35 * moisture.max(0.0));
            let speckle_mask = (speckles > 0.55) as i32 as f32 * 0.2;

            let lichen_r = 138.0 + 40.0 * moisture;
            let lichen_g = 146.0 + 55.0 * moisture;
            let lichen_b = 104.0 + 25.0 * moisture;

            let r = (base_gray * (1.0 - lichen_mask) + lichen_r * lichen_mask + 25.0 * speckle_mask)
                .clamp(0.0, 255.0) as u8;
            let g = (base_gray * (1.0 - lichen_mask) + lichen_g * lichen_mask + 20.0 * speckle_mask)
                .clamp(0.0, 255.0) as u8;
            let b = (base_gray * (1.0 - lichen_mask) + lichen_b * lichen_mask + 18.0 * speckle_mask)
                .clamp(0.0, 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_rock_slate(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 20.0;
            let v = y as f32 / h as f32 * 20.0;
            let layering = fbm(v * 2.8, u * 0.6, seed ^ 0x4242, 8, 2.1, 0.5);
            let fractures = fbm(u * 3.2, v * 3.2, seed ^ 0x8181, 6, 2.0, 0.38);
            let mineral = fbm(u * 1.5, v * 1.0, seed ^ 0x3434, 5, 2.0, 0.45);

            let height = (0.6 + 0.2 * layering - 0.1 * fractures).clamp(0.0, 1.0);
            let mineral_tint = (mineral + 1.0) * 0.5;

            let r = (88.0 + 80.0 * height + 30.0 * mineral_tint).clamp(0.0, 255.0) as u8;
            let g = (94.0 + 70.0 * height + 20.0 * mineral_tint).clamp(0.0, 255.0) as u8;
            let b = (112.0 + 65.0 * height + 25.0 * mineral_tint).clamp(0.0, 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_plaster(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 8.0;
            let v = y as f32 / h as f32 * 8.0;
            let base = fbm(u, v, seed ^ 0x1919, 5, 2.0, 0.5);
            let cracks = fbm(u * 3.2, v * 3.2, seed ^ 0x3232, 6, 2.0, 0.4);
            let stains = fbm(u * 1.1, v * 1.1, seed ^ 0x4545, 4, 2.0, 0.45);

            let base_color = 205.0 + 18.0 * base;
            let crack_dark = (cracks > 0.55) as i32 as f32 * 18.0;
            let stain_tint = stains * 12.0;

            let r = (base_color - crack_dark + stain_tint).clamp(0.0, 255.0) as u8;
            let g = (base_color - crack_dark + stain_tint * 0.6).clamp(0.0, 255.0) as u8;
            let b = (base_color - crack_dark * 1.2 + stain_tint * 0.4).clamp(0.0, 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_roof_tile(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let tile_w = 256;
    let tile_h = 180;
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let tile_x = x as i32 / tile_w;
            let tile_y = y as i32 / tile_h;
            let local_x = (x % tile_w as u32) as f32 / tile_w as f32;
            let local_y = (y % tile_h as u32) as f32 / tile_h as f32;

            let arch = (-((local_x - 0.5).powi(2)) * 12.0 + 1.0).clamp(0.0, 1.0);
            let overlap = (local_y < 0.18) as i32 as f32 * 0.3;
            let ridge = ((local_x - 0.5).abs() < 0.08) as i32 as f32 * 0.4;

            let color_variation = fbm(
                (tile_x as f32 + local_x) * 1.2,
                (tile_y as f32 + local_y) * 1.2,
                seed ^ 0xDEAD,
                4,
                2.0,
                0.5,
            );

            let base_r = 150.0 + 52.0 * arch + 25.0 * color_variation - 18.0 * overlap;
            let base_g = 66.0 + 32.0 * arch + 20.0 * color_variation - 12.0 * overlap;
            let base_b = 52.0 + 22.0 * arch + 18.0 * color_variation - 10.0 * overlap;

            let ridge_highlight = ridge * 35.0;

            let r = (base_r + ridge_highlight).clamp(0.0, 255.0) as u8;
            let g = (base_g + ridge_highlight * 0.7).clamp(0.0, 255.0) as u8;
            let b = (base_b + ridge_highlight * 0.5).clamp(0.0, 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn synth_cloth(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let weave_u = x as f32 / w as f32 * 80.0;
            let weave_v = y as f32 / h as f32 * 80.0;

            let warp = ((weave_u.sin() * 0.5 + 0.5) + fbm(weave_u, weave_v, seed ^ 0x1A2B, 3, 2.0, 0.5) * 0.2).clamp(0.0, 1.0);
            let weft = ((weave_v.cos() * 0.5 + 0.5) + fbm(weave_v, weave_u, seed ^ 0x3C4D, 3, 2.0, 0.5) * 0.2).clamp(0.0, 1.0);

            let sheen = fbm(weave_u * 0.4, weave_v * 0.4, seed ^ 0x5E6F, 4, 2.0, 0.45).max(0.0);
            let thread_mix = (warp * 0.6 + weft * 0.4).clamp(0.0, 1.0);

            let base_r = 130.0 + 45.0 * thread_mix + 30.0 * sheen;
            let base_g = 78.0 + 35.0 * thread_mix + 22.0 * sheen;
            let base_b = 150.0 + 50.0 * thread_mix + 35.0 * sheen;

            img.put_pixel(x, y, Rgba([
                base_r.clamp(0.0, 255.0) as u8,
                base_g.clamp(0.0, 255.0) as u8,
                base_b.clamp(0.0, 255.0) as u8,
                255,
            ]));
        }
    }
    img
}
