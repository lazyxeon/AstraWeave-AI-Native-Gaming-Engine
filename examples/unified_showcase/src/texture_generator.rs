use image::{ImageBuffer, RgbaImage};
use noise::{NoiseFn, Perlin, Turbulence, Worley};
use std::path::Path;

// Procedural texture generation for the unified showcase
pub struct TextureGenerator {
    perlin: Perlin,
    worley: Worley,
}

impl TextureGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            // Use Worley's default distance function (Euclidean) for compatibility across noise versions
            worley: Worley::new(seed),
        }
    }

    // Generate a simple grass texture
    pub fn generate_grass(&self, width: u32, height: u32) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        let turbulence = Turbulence::<_, Perlin>::new(self.perlin)
            .set_frequency(0.05)
            .set_power(4.0)
            .set_roughness(4);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                let noise_val = turbulence.get([nx * 4.0, ny * 4.0, 0.0]);
                let noise_mapped = (noise_val * 0.5 + 0.5).powf(0.8);
                
                // Base green with variations
                let r = (0.2 + noise_mapped * 0.2) * 255.0;
                let g = (0.5 + noise_mapped * 0.3) * 255.0;
                let b = (0.1 + noise_mapped * 0.1) * 255.0;
                
                // Add some small details/variation
                let detail = self.perlin.get([nx * 32.0, ny * 32.0, 0.0]) * 30.0;
                
                image.put_pixel(
                    x, 
                    y, 
                    image::Rgba([
                        (r + detail).clamp(0.0, 255.0) as u8,
                        (g + detail).clamp(0.0, 255.0) as u8,
                        (b + detail).clamp(0.0, 255.0) as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }
    
    // Generate a normal map from a height map
    pub fn generate_normal_map(&self, height_map: &RgbaImage) -> RgbaImage {
        let (width, height) = height_map.dimensions();
        let mut normal_map = ImageBuffer::new(width, height);
        
        for y in 1..height-1 {
            for x in 1..width-1 {
                // Sample neighboring pixels for height
                let l = height_map.get_pixel(x - 1, y)[0] as f32 / 255.0;
                let r = height_map.get_pixel(x + 1, y)[0] as f32 / 255.0;
                let t = height_map.get_pixel(x, y - 1)[0] as f32 / 255.0;
                let b = height_map.get_pixel(x, y + 1)[0] as f32 / 255.0;
                
                // Calculate normal with Sobel filter
                let dx = (r - l) * 2.0;
                let dy = (b - t) * 2.0;
                let dz = 1.0 / 8.0; // Strength factor
                
                // Normalize
                let length = (dx * dx + dy * dy + dz * dz).sqrt();
                
                // Convert to RGB normal map format (128, 128, 255 is flat)
                let nx = (dx / length * 0.5 + 0.5) * 255.0;
                let ny = (dy / length * 0.5 + 0.5) * 255.0;
                let nz = (dz / length * 0.5 + 0.5) * 255.0;
                
                normal_map.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        nx as u8,
                        ny as u8,
                        nz as u8,
                        255
                    ])
                );
            }
        }
        
        // Fill in borders
        for y in 0..height {
            normal_map.put_pixel(0, y, image::Rgba([128, 128, 255, 255]));
            normal_map.put_pixel(width - 1, y, image::Rgba([128, 128, 255, 255]));
        }
        
        for x in 0..width {
            normal_map.put_pixel(x, 0, image::Rgba([128, 128, 255, 255]));
            normal_map.put_pixel(x, height - 1, image::Rgba([128, 128, 255, 255]));
        }
        
        normal_map
    }
    
    // Generate a simple dirt texture
    pub fn generate_dirt(&self, width: u32, height: u32) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                let noise_val = self.perlin.get([nx * 8.0, ny * 8.0, 0.0]);
                let noise_detail = self.perlin.get([nx * 32.0, ny * 32.0, 0.5]) * 0.3;
                let combined = (noise_val + noise_detail).clamp(-1.0, 1.0) * 0.5 + 0.5;
                
                // Dirt colors
                let r = (0.45 + combined * 0.15) * 255.0;
                let g = (0.3 + combined * 0.1) * 255.0;
                let b = (0.15 + combined * 0.1) * 255.0;
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r as u8,
                        g as u8,
                        b as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }
    
    // Generate a simple sand texture
    pub fn generate_sand(&self, width: u32, height: u32) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        let turbulence = Turbulence::<_, Perlin>::new(self.perlin)
            .set_frequency(0.04)
            .set_power(2.0)
            .set_roughness(3);
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                let noise_val = turbulence.get([nx * 6.0, ny * 6.0, 0.0]);
                let noise_mapped = (noise_val * 0.5 + 0.5).powf(0.7);
                
                // Fine grain detail
                let detail = self.perlin.get([nx * 40.0, ny * 40.0, 0.0]) * 0.1;
                
                // Sand colors
                let r = (0.85 + noise_mapped * 0.1 + detail) * 255.0;
                let g = (0.75 + noise_mapped * 0.1 + detail) * 255.0;
                let b = (0.55 + noise_mapped * 0.1 + detail) * 255.0;
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r.clamp(0.0, 255.0) as u8,
                        g.clamp(0.0, 255.0) as u8,
                        b.clamp(0.0, 255.0) as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }
    
    // Generate a simple stone texture
    pub fn generate_stone(&self, width: u32, height: u32) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                // Base stone texture
                let base_noise = self.perlin.get([nx * 4.0, ny * 4.0, 0.0]);
                
                // Cracks and variations
                let cracks = self.worley.get([nx * 8.0, ny * 8.0, 0.0]) * 0.5;
                let detail = self.perlin.get([nx * 32.0, ny * 32.0, 0.0]) * 0.1;
                
                let combined = (base_noise + cracks + detail).clamp(-1.0, 1.0) * 0.5 + 0.5;
                
                // Stone gray colors
                let value = (0.4 + combined * 0.3) * 255.0;
                let r = value * 0.95;
                let g = value * 1.0;
                let b = value * 1.05;
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r as u8,
                        g as u8,
                        b as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }
    
    // Generate forest floor texture
    pub fn generate_forest_floor(&self, width: u32, height: u32) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        let turbulence = Turbulence::<_, Perlin>::new(self.perlin)
            .set_frequency(0.05)
            .set_power(3.0)
            .set_roughness(4);
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                let noise_val = turbulence.get([nx * 5.0, ny * 5.0, 0.0]);
                let noise_mapped = (noise_val * 0.5 + 0.5).powf(0.9);
                
                // Add some moss/leaf pattern
                let moss_pattern = self.perlin.get([nx * 20.0, ny * 20.0, 0.0]) * 0.5 + 0.5;
                
                // Forest floor colors - dark soil with moss/leaf litter
                let r = (0.25 + noise_mapped * 0.2) * 255.0;
                let g = (0.2 + noise_mapped * 0.15 + moss_pattern * 0.15) * 255.0;
                let b = (0.15 + noise_mapped * 0.1) * 255.0;
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r as u8,
                        g as u8,
                        b as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }
    
    // Generate tree bark texture
    pub fn generate_tree_bark(&self, width: u32, height: u32) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        let turbulence = Turbulence::<_, Perlin>::new(self.perlin)
            .set_frequency(0.1)
            .set_power(2.0)
            .set_roughness(3);
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                // Vertical striping for bark
                let vert_noise = self.perlin.get([nx * 20.0, ny * 2.0, 0.0]) * 0.7;
                let horiz_detail = self.perlin.get([nx * 8.0, ny * 8.0, 0.0]) * 0.3;
                let combined = (vert_noise + horiz_detail) * 0.5 + 0.5;
                
                // Bark color - brown with variations
                let r = (0.35 + combined * 0.2) * 255.0;
                let g = (0.25 + combined * 0.1) * 255.0;
                let b = (0.15 + combined * 0.05) * 255.0;
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r as u8,
                        g as u8,
                        b as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }

    // Generate leaves texture for trees
    pub fn generate_leaves(&self, width: u32, height: u32, leaf_type: &str) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        
        let (r_base, g_base, b_base) = match leaf_type {
            "oak" => (0.2, 0.5, 0.15),     // Oak - more green
            "pine" => (0.15, 0.4, 0.2),    // Pine - darker green
            "palm" => (0.3, 0.6, 0.2),     // Palm - lighter green
            _ => (0.2, 0.5, 0.15),         // Default to oak
        };
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                // Leaf pattern
                let pattern = self.perlin.get([nx * 12.0, ny * 12.0, 0.0]) * 0.5 + 0.5;
                let detail = self.perlin.get([nx * 32.0, ny * 32.0, 0.0]) * 0.2;
                
                // Small scale noise for leaf details
                let fine_detail = self.perlin.get([nx * 64.0, ny * 64.0, 0.0]) * 0.1;
                
                let combined = pattern + detail + fine_detail;
                
                // Leaf color based on type with variations
                let r = (r_base + combined * 0.15) * 255.0;
                let g = (g_base + combined * 0.15) * 255.0;
                let b = (b_base + combined * 0.1) * 255.0;
                
                // Alpha for leaf edges
                let edge_noise = self.perlin.get([nx * 16.0, ny * 16.0, 0.5]);
                let alpha = if edge_noise > 0.3 { 255 } else { 0 };
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r.clamp(0.0, 255.0) as u8,
                        g.clamp(0.0, 255.0) as u8,
                        b.clamp(0.0, 255.0) as u8,
                        alpha
                    ])
                );
            }
        }
        
        image
    }

    // Generate a house wall texture
    pub fn generate_wall(&self, width: u32, height: u32, wall_type: &str) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        
        let (pattern_scale, color_base) = match wall_type {
            "wood" => (
                (12.0, 4.0),  // Horizontal wood grain
                (0.6, 0.4, 0.25)  // Brown wood
            ),
            "stone" => (
                (8.0, 8.0),  // Stone block pattern
                (0.7, 0.7, 0.7)  // Gray stone
            ),
            "adobe" => (
                (6.0, 6.0),  // Adobe texture
                (0.8, 0.7, 0.5)  // Tan adobe
            ),
            _ => (
                (12.0, 4.0),
                (0.6, 0.4, 0.25)
            ),
        };
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                let mut pattern_value = 0.0;
                
                match wall_type {
                    "wood" => {
                        // Horizontal wood planks
                        let plank_y = (ny * 10.0).floor() / 10.0;
                        let wood_grain = self.perlin.get([nx * pattern_scale.0, plank_y * 50.0, 0.0]) * 0.5 + 0.5;
                        let detail = self.perlin.get([nx * 40.0, ny * 40.0, 0.0]) * 0.1;
                        pattern_value = wood_grain + detail;
                        
                        // Add plank separations
                        let plank_edge = ((ny * 10.0) % 1.0) < 0.05;
                        if plank_edge {
                            pattern_value *= 0.7;
                        }
                    },
                    "stone" => {
                        // Stone blocks
                        let block_x = (nx * 4.0).floor() / 4.0;
                        let block_y = (ny * 4.0).floor() / 4.0;
                        let block_id = (block_x * 100.0 + block_y * 50.0) as u32;
                        
                        let block_color = self.perlin.get([block_x * 100.0, block_y * 100.0, 0.0]) * 0.2 + 0.8;
                        let stone_texture = self.perlin.get([nx * 30.0, ny * 30.0, 0.0]) * 0.15;
                        
                        // Block edges
                        let edge_x = ((nx * 4.0) % 1.0) < 0.05;
                        let edge_y = ((ny * 4.0) % 1.0) < 0.05;
                        
                        pattern_value = block_color + stone_texture;
                        if edge_x || edge_y {
                            pattern_value *= 0.7;
                        }
                    },
                    "adobe" => {
                        // Smooth adobe with slight variations
                        let base = self.perlin.get([nx * pattern_scale.0, ny * pattern_scale.1, 0.0]) * 0.3 + 0.7;
                        let fine_grain = self.perlin.get([nx * 40.0, ny * 40.0, 0.0]) * 0.1;
                        pattern_value = base + fine_grain;
                        
                        // Occasional cracks
                        let crack = self.worley.get([nx * 3.0, ny * 3.0, 0.0]);
                        if crack > 0.95 {
                            pattern_value *= 0.85;
                        }
                    },
                    _ => {
                        pattern_value = self.perlin.get([nx * pattern_scale.0, ny * pattern_scale.1, 0.0]) * 0.5 + 0.5;
                    }
                };
                
                // Apply color based on wall type and pattern
                let r = (color_base.0 * pattern_value) * 255.0;
                let g = (color_base.1 * pattern_value) * 255.0;
                let b = (color_base.2 * pattern_value) * 255.0;
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r.clamp(0.0, 255.0) as u8,
                        g.clamp(0.0, 255.0) as u8,
                        b.clamp(0.0, 255.0) as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }

    // Generate a character texture
    pub fn generate_character(&self, width: u32, height: u32, character_type: &str) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        
        let (primary_color, secondary_color) = match character_type {
            "villager" => (
                (0.8, 0.6, 0.4),  // Skin tone
                (0.4, 0.6, 0.8)   // Blue clothing
            ),
            "guard" => (
                (0.7, 0.5, 0.3),  // Darker skin tone
                (0.3, 0.3, 0.7)   // Blue/metal armor
            ),
            "merchant" => (
                (0.75, 0.55, 0.4),  // Medium skin tone
                (0.7, 0.3, 0.7)     // Purple/rich clothing
            ),
            "animal" => (
                (0.8, 0.7, 0.4),  // Light fur/hide
                (0.6, 0.5, 0.3)   // Darker fur/hide
            ),
            _ => (
                (0.8, 0.6, 0.4),
                (0.5, 0.5, 0.5)
            ),
        };
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                // UV mapping for character "paper doll" layout
                let part = if ny < 0.25 {
                    "head"
                } else if ny < 0.7 {
                    "body"
                } else {
                    "legs"
                };
                
                let pattern_value = self.perlin.get([nx * 10.0, ny * 10.0, 0.0]) * 0.2 + 0.9;
                let detail = self.perlin.get([nx * 30.0, ny * 30.0, 0.0]) * 0.1;
                
                // Apply different colors based on body part
                let (r, g, b) = match part {
                    "head" => {
                        let is_face = nx > 0.3 && nx < 0.7;
                        if is_face {
                            (
                                primary_color.0 * pattern_value,
                                primary_color.1 * pattern_value,
                                primary_color.2 * pattern_value
                            )
                        } else {
                            // Hair or hat
                            (
                                secondary_color.0 * pattern_value * 0.7,
                                secondary_color.1 * pattern_value * 0.7,
                                secondary_color.2 * pattern_value * 0.7
                            )
                        }
                    },
                    "body" => {
                        // Clothing
                        (
                            secondary_color.0 * (pattern_value + detail),
                            secondary_color.1 * (pattern_value + detail),
                            secondary_color.2 * (pattern_value + detail)
                        )
                    },
                    "legs" => {
                        // Pants/lower clothing
                        let darkness = if character_type == "guard" { 0.8 } else { 1.0 };
                        (
                            secondary_color.0 * pattern_value * darkness,
                            secondary_color.1 * pattern_value * darkness,
                            secondary_color.2 * pattern_value * darkness
                        )
                    },
                    _ => (0.5, 0.5, 0.5)
                };
                
                // Add some character-specific details
                let special_detail = match character_type {
                    "guard" => {
                        // Armor highlights
                        if (nx * 10.0).floor() % 2.0 < 1.0 && part == "body" {
                            0.2
                        } else {
                            0.0
                        }
                    },
                    "merchant" => {
                        // Rich clothing pattern
                        if part == "body" && 
                           (((nx * 8.0).floor() + (ny * 8.0).floor()) % 2.0 < 1.0) {
                            0.15
                        } else {
                            0.0
                        }
                    },
                    "animal" => {
                        // Fur/spots pattern
                        let spots = self.worley.get([nx * 5.0, ny * 5.0, 0.0]);
                        if spots > 0.7 {
                            -0.2
                        } else {
                            0.0
                        }
                    },
                    _ => 0.0
                };
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        ((r + special_detail) * 255.0).clamp(0.0, 255.0) as u8,
                        ((g + special_detail) * 255.0).clamp(0.0, 255.0) as u8,
                        ((b + special_detail) * 255.0).clamp(0.0, 255.0) as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }

    // Generate skybox textures
    pub fn generate_skybox_side(&self, width: u32, height: u32, side: &str, time_of_day: &str) -> RgbaImage {
        let mut image = ImageBuffer::new(width, height);
        
        let (horizon_color, zenith_color, sun_color) = match time_of_day {
            "day" => (
                (0.7, 0.8, 0.9),   // Horizon - light blue
                (0.4, 0.6, 1.0),   // Zenith - deep blue
                (1.0, 0.95, 0.8)   // Sun - warm white
            ),
            "night" => (
                (0.1, 0.1, 0.2),   // Horizon - dark blue
                (0.0, 0.0, 0.05),  // Zenith - near black
                (0.8, 0.8, 0.9)    // Moon - cool white
            ),
            _ => (
                (0.7, 0.8, 0.9),
                (0.4, 0.6, 1.0),
                (1.0, 0.95, 0.8)
            ),
        };
        
        // Sun position parameters
        let sun_dir = match side {
            "top" => (0.5, 0.5),
            "front" => (0.5, 0.2),
            "back" => (0.5, 0.2),
            "left" => (0.8, 0.2),
            "right" => (0.2, 0.2),
            "bottom" => (0.5, 0.5),
            _ => (0.5, 0.5),
        };
        
        // Stars parameters (only visible at night)
        let star_density = if time_of_day == "night" { 0.0005 } else { 0.0 };
        let star_seed = 42;
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                // Base sky gradient
                let gradient_t = match side {
                    "top" => 0.0,
                    "bottom" => 1.0,
                    _ => ny
                };
                
                // Mix horizon and zenith colors based on height
                let r = lerp(zenith_color.0, horizon_color.0, gradient_t);
                let g = lerp(zenith_color.1, horizon_color.1, gradient_t);
                let b = lerp(zenith_color.2, horizon_color.2, gradient_t);
                
                // Cloud system for day sky
                let mut cloud_factor = 0.0;
                if time_of_day == "day" && side != "bottom" {
                    let cloud_scale = 4.0;
                    let cloud_coverage = 0.45;
                    
                    let cloud_base = self.perlin.get([nx * cloud_scale, ny * cloud_scale, 0.0]) * 0.5 + 0.5;
                    let cloud_detail = self.perlin.get([nx * cloud_scale * 3.0, ny * cloud_scale * 3.0, 0.0]) * 0.25;
                    
                    if cloud_base + cloud_detail > cloud_coverage {
                        let cloud_density = ((cloud_base + cloud_detail - cloud_coverage) / (1.0 - cloud_coverage)).powf(0.5);
                        cloud_factor = cloud_density * 0.3; // Adjust for subtlety
                    }
                }
                
                // Stars for night sky
                let mut star_factor = 0.0;
                if time_of_day == "night" && side != "bottom" {
                    // Simple hash function for stars
                    let px = (x as u32 * 17 + y as u32 * 31 + star_seed) % 100000;
                    if px as f64 / 100000.0 < star_density {
                        star_factor = 0.8;
                    }
                }
                
                // Sun/moon placement
                let mut celestial_factor = 0.0;
                if side != "bottom" {
                    let sun_distance = ((nx - sun_dir.0).powf(2.0) + (ny - sun_dir.1).powf(2.0)).sqrt();
                    let sun_radius = if time_of_day == "day" { 0.05 } else { 0.03 };
                    
                    if sun_distance < sun_radius {
                        celestial_factor = (1.0 - (sun_distance / sun_radius)).powf(0.5);
                    }
                }
                
                // Combine all factors
                let r_out = r * (1.0 - celestial_factor - cloud_factor) + sun_color.0 * celestial_factor + cloud_factor + star_factor;
                let g_out = g * (1.0 - celestial_factor - cloud_factor) + sun_color.1 * celestial_factor + cloud_factor + star_factor;
                let b_out = b * (1.0 - celestial_factor - cloud_factor) + sun_color.2 * celestial_factor + cloud_factor + star_factor;
                
                image.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        (r_out * 255.0).clamp(0.0, 255.0) as u8,
                        (g_out * 255.0).clamp(0.0, 255.0) as u8,
                        (b_out * 255.0).clamp(0.0, 255.0) as u8,
                        255
                    ])
                );
            }
        }
        
        image
    }

    // Generate water texture with a normal map for waves
    pub fn generate_water(&self, width: u32, height: u32) -> (RgbaImage, RgbaImage) {
        let mut diffuse = ImageBuffer::new(width, height);
        let mut normal = ImageBuffer::new(width, height);
        
        // Create a wave height map first
        let mut height_map = ImageBuffer::<image::Luma<u8>, Vec<u8>>::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                
                // Generate wave pattern
                let wave1 = (nx * 10.0 + ny * 5.0).sin() * 0.5 + 0.5;
                let wave2 = (nx * 5.0 - ny * 8.0).sin() * 0.3 + 0.5;
                let detail = self.perlin.get([nx * 20.0, ny * 20.0, 0.0]) * 0.2 + 0.5;
                
                let height_value = (wave1 * 0.5 + wave2 * 0.3 + detail * 0.2) * 255.0;
                
                height_map.put_pixel(
                    x, 
                    y, 
                    image::Luma([height_value as u8])
                );
            }
        }
        
        // Generate diffuse color from height
        for y in 0..height {
            for x in 0..width {
                let height = height_map.get_pixel(x, y)[0] as f64 / 255.0;
                
                // Water colors - from deep to shallow
                let r = (0.1 + height * 0.2) * 255.0;
                let g = (0.2 + height * 0.3) * 255.0;
                let b = (0.5 + height * 0.3) * 255.0;
                
                diffuse.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        r as u8,
                        g as u8,
                        b as u8,
                        200 // Slight transparency
                    ])
                );
            }
        }
        
        // Generate normal map from height
        for y in 1..height-1 {
            for x in 1..width-1 {
                // Sample neighboring pixels for height
                let l = height_map.get_pixel(x - 1, y)[0] as f32 / 255.0;
                let r = height_map.get_pixel(x + 1, y)[0] as f32 / 255.0;
                let t = height_map.get_pixel(x, y - 1)[0] as f32 / 255.0;
                let b = height_map.get_pixel(x, y + 1)[0] as f32 / 255.0;
                
                // Calculate normal with Sobel filter
                let dx = (r - l) * 2.0;
                let dy = (b - t) * 2.0;
                let dz = 0.8; // Strength factor - higher for more dramatic wave normals
                
                // Normalize
                let length = (dx * dx + dy * dy + dz * dz).sqrt();
                
                // Convert to RGB normal map format (128, 128, 255 is flat)
                let nx = (dx / length * 0.5 + 0.5) * 255.0;
                let ny = (dy / length * 0.5 + 0.5) * 255.0;
                let nz = (dz / length * 0.5 + 0.5) * 255.0;
                
                normal.put_pixel(
                    x,
                    y,
                    image::Rgba([
                        nx as u8,
                        ny as u8,
                        nz as u8,
                        255
                    ])
                );
            }
        }
        
        // Fill in borders
        for y in 0..height {
            normal.put_pixel(0, y, image::Rgba([128, 128, 255, 255]));
            normal.put_pixel(width - 1, y, image::Rgba([128, 128, 255, 255]));
        }
        
        for x in 0..width {
            normal.put_pixel(x, 0, image::Rgba([128, 128, 255, 255]));
            normal.put_pixel(x, height - 1, image::Rgba([128, 128, 255, 255]));
        }
        
        (diffuse, normal)
    }
}

// Linear interpolation helper
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

// Function to generate and save all required textures
pub fn generate_all_textures(output_dir: &Path, texture_size: u32) -> Result<(), Box<dyn std::error::Error>> {
    let generator = TextureGenerator::new(42);
    
    // Create directories if they don't exist
    let terrain_dir = output_dir.join("terrain");
    let structures_dir = output_dir.join("structures");
    let characters_dir = output_dir.join("characters");
    let skybox_dir = output_dir.join("skybox");
    let effects_dir = output_dir.join("effects");
    
    for dir in &[&terrain_dir, &structures_dir, &characters_dir, &skybox_dir, &effects_dir] {
        std::fs::create_dir_all(dir)?;
    }
    
    // Generate terrain textures
    println!("Generating terrain textures...");
    let grass = generator.generate_grass(texture_size, texture_size);
    grass.save(terrain_dir.join("grass.png"))?;
    
    let grass_normal = generator.generate_normal_map(&grass);
    grass_normal.save(terrain_dir.join("grass_n.png"))?;
    
    let dirt = generator.generate_dirt(texture_size, texture_size);
    dirt.save(terrain_dir.join("dirt.png"))?;
    
    let dirt_normal = generator.generate_normal_map(&dirt);
    dirt_normal.save(terrain_dir.join("dirt_n.png"))?;
    
    let sand = generator.generate_sand(texture_size, texture_size);
    sand.save(terrain_dir.join("sand.png"))?;
    
    let sand_normal = generator.generate_normal_map(&sand);
    sand_normal.save(terrain_dir.join("sand_n.png"))?;
    
    let stone = generator.generate_stone(texture_size, texture_size);
    stone.save(terrain_dir.join("stone.png"))?;
    
    let stone_normal = generator.generate_normal_map(&stone);
    stone_normal.save(terrain_dir.join("stone_n.png"))?;
    
    let forest_floor = generator.generate_forest_floor(texture_size, texture_size);
    forest_floor.save(terrain_dir.join("forest_floor.png"))?;
    
    let forest_floor_normal = generator.generate_normal_map(&forest_floor);
    forest_floor_normal.save(terrain_dir.join("forest_floor_n.png"))?;
    
    // Generate structure textures
    println!("Generating structure textures...");
    let tree_bark = generator.generate_tree_bark(texture_size, texture_size);
    tree_bark.save(structures_dir.join("tree_bark.png"))?;
    
    let tree_bark_normal = generator.generate_normal_map(&tree_bark);
    tree_bark_normal.save(structures_dir.join("tree_bark_n.png"))?;
    
    for leaf_type in &["oak", "pine", "palm"] {
        let leaves = generator.generate_leaves(texture_size, texture_size, leaf_type);
        leaves.save(structures_dir.join(format!("leaves_{}.png", leaf_type)))?;
        
        let leaves_normal = generator.generate_normal_map(&leaves);
        leaves_normal.save(structures_dir.join(format!("leaves_{}_n.png", leaf_type)))?;
    }
    
    for wall_type in &["wood", "stone", "adobe"] {
        let wall = generator.generate_wall(texture_size, texture_size, wall_type);
        wall.save(structures_dir.join(format!("{}_wall.png", wall_type)))?;
        
        let wall_normal = generator.generate_normal_map(&wall);
        wall_normal.save(structures_dir.join(format!("{}_wall_n.png", wall_type)))?;
    }
    
    // Generate character textures
    println!("Generating character textures...");
    for char_type in &["villager", "guard", "merchant", "animal"] {
        let character = generator.generate_character(texture_size, texture_size, char_type);
        character.save(characters_dir.join(format!("{}.png", char_type)))?;
        
        let char_normal = generator.generate_normal_map(&character);
        char_normal.save(characters_dir.join(format!("{}_n.png", char_type)))?;
    }
    
    // Generate skybox textures
    println!("Generating skybox textures...");
    for time in &["day", "night"] {
        for side in &["top", "bottom", "left", "right", "front", "back"] {
            let skybox_side = generator.generate_skybox_side(texture_size, texture_size, side, time);
            skybox_side.save(skybox_dir.join(format!("{}_{}.png", time, side)))?;
        }
    }
    
    // Generate water textures
    println!("Generating effect textures...");
    let (water, water_normal) = generator.generate_water(texture_size, texture_size);
    water.save(effects_dir.join("water.png"))?;
    water_normal.save(effects_dir.join("water_n.png"))?;
    
    println!("All textures generated successfully!");
    Ok(())
}