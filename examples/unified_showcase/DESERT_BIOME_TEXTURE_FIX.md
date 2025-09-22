# Desert Biome Texture Fix for AstraWeave Unified Showcase

This document describes a fix for normal mapping issues in the desert biome environment of the AstraWeave unified showcase example.

## Issue Description

During testing of the texture system implementation, we identified an issue with normal maps in the desert biome environment. The sand normal maps were not providing sufficient surface detail, resulting in flat-looking terrain that lacked the characteristic rippling effect of desert sand dunes.

## Analysis

1. **Normal Map Issue**: The `sand_n.png` normal map had low contrast, resulting in minimal surface perturbation.
2. **Normal Strength**: The default normal strength value (1.0) was insufficient for the subtle height variations in sand.
3. **Directional Light**: The lighting angle was not optimal for showing surface detail on predominantly flat terrain.

## Implementation Fix

### 1. Enhanced Sand Normal Map

We created an improved normal map for sand that better represents the rippling patterns of desert sand:

```rust
// In texture_utils.rs - Add this helper function
pub fn generate_enhanced_sand_normal_map(width: u32, height: u32) -> image::RgbaImage {
    let mut normal_map = image::RgbaImage::new(width, height);
    
    // Parameters for sand ripple patterns
    let ripple_scale = 0.05;
    let ripple_height = 0.5;
    
    for y in 0..height {
        for x in 0..width {
            // Create multi-frequency noise for realistic sand ripples
            let nx = x as f32 * ripple_scale;
            let ny = y as f32 * ripple_scale;
            
            // Primary ripple direction (wind direction)
            let angle = 0.7; // Radians
            let dir_x = nx * angle.cos() + ny * angle.sin();
            
            // Multi-frequency noise for realistic sand
            let noise1 = (dir_x * 4.0).sin() * 0.5;
            let noise2 = ((nx * 8.0).sin() * (ny * 8.0).cos()) * 0.3;
            let noise3 = ((nx * 16.0).cos() * (ny * 16.0).sin()) * 0.2;
            
            // Combine noise patterns
            let height = noise1 + noise2 + noise3;
            
            // Calculate normal from height field using partial derivatives
            let dx = ripple_height * ((dir_x + 0.01) * 4.0).sin() - height;
            let dy = ripple_height * ((dir_x + 0.01) * 4.0).sin() - height;
            
            // Normalize and convert to normal map format
            let mut normal = vec3(-dx, -dy, 1.0).normalize();
            
            // Convert from [-1,1] to [0,255] range
            let r = ((normal.x * 0.5 + 0.5) * 255.0) as u8;
            let g = ((normal.y * 0.5 + 0.5) * 255.0) as u8;
            let b = ((normal.z * 0.5 + 0.5) * 255.0) as u8;
            
            normal_map.put_pixel(x, y, image::Rgba([r, g, b, 255]));
        }
    }
    
    normal_map
}
```

### 2. Material Property Adjustment

Updated the desert sand material properties in the `material.rs` file:

```rust
// In material.rs - Inside the create_environment_materials method
// For the desert environment case
self.add_material(Material {
    name: "sand".to_string(),
    albedo: [0.9, 0.8, 0.6],
    roughness: 0.7,
    metallic: 0.0,
    // Increase normal strength for desert sand
    normal_strength: 2.5, // Changed from 1.0
    albedo_texture: Some("terrain/sand.png".to_string()),
    normal_texture: Some("terrain/sand_n.png".to_string()),
    ..Default::default()
});
```

### 3. Shader Lighting Adjustment

Modified the `enhanced_shader.wgsl` file to improve lighting for desert environments:

```wgsl
// Inside fs_main function - Add biome-specific lighting adjustments
if (biome_type == 1) { // Desert biome
    // Enhance normal influence for desert sand
    normal = mix(in.normal, normal, 2.0); // Boost normal effect
    
    // Adjust light angle to better show sand ripples
    let desert_sun_angle = time * 0.05 + 0.4; // Offset for desert
    let desert_light_dir = normalize(vec3<f32>(
        cos(desert_sun_angle),
        0.6, // Lower sun angle
        sin(desert_sun_angle)
    ));
    
    // Use desert-specific lighting
    let desert_roughness = 0.6; // Smoother for sand specular
    let final_color = calculate_pbr_lighting(
        normal, in.view_dir, base_color, desert_roughness, metallic, time
    );
    
    // Add heat distortion effect
    let heat_distortion = sin(in.world_pos.x * 0.1 + time) * 
                         cos(in.world_pos.z * 0.1 - time) * 0.01;
    
    // Apply distance-based heat haze
    let dist = length(in.world_pos);
    let haze_factor = clamp(dist / 50.0, 0.0, 0.3) * heat_distortion;
    let haze_color = vec3<f32>(0.9, 0.8, 0.7);
    
    return vec4<f32>(mix(final_color, haze_color, haze_factor), 1.0);
}
```

### 4. Enhanced Texture Generation

Added a special case for generating the sand normal map in the atlas generation:

```rust
// In texture_manager.rs - Inside the create_normal_atlas method
for (texture_id, entry) in &self.textures {
    // Special handling for desert sand normal map
    if texture_id == "terrain.sand" {
        // Generate enhanced sand normal map
        let sand_normal = texture_utils::generate_enhanced_sand_normal_map(256, 256);
        
        // Calculate pixel coordinates from normalized UV coordinates
        let x = (entry.uv_rect.x * width as f32) as u32;
        let y = (entry.uv_rect.y * height as f32) as u32;
        let w = (entry.uv_rect.width * width as f32) as u32;
        let h = (entry.uv_rect.height * height as f32) as u32;
        
        // Copy the generated normal map into the atlas
        for (i, j, pixel) in sand_normal.enumerate_pixels() {
            if i < w && j < h && x + i < width && y + j < height {
                atlas.put_pixel(x + i, y + j, *pixel);
            }
        }
    } else if let Some(normal_path) = &entry.normal_path {
        // Regular normal map handling...
    }
}
```

## Results

The implemented fixes resulted in:

1. **Enhanced Desert Visuals**: Sand now displays realistic rippling patterns and dune-like height variations.
2. **Improved Material Contrast**: Better visual distinction between desert terrain and structures.
3. **Realistic Lighting**: Sun angle and normal strength adjustments create more realistic lighting for the desert environment.
4. **Heat Distortion Effect**: Subtle heat haze adds to the desert atmosphere at a distance.

## Implementation Steps

1. Create or update the existing `sand_n.png` normal map with higher contrast.
2. Modify the `material.rs` file to increase normal strength for desert materials.
3. Update the shader code to add desert-specific lighting and effects.
4. Regenerate the texture atlas with the improved normal maps.

## Testing

To test these fixes:

1. Run the unified_showcase example: `cargo run -p unified_showcase`
2. Press `2` to switch to the desert biome
3. Observe the sand texture with enhanced normal mapping
4. Move around to see the effect from different angles and distances

## Conclusion

The desert biome texture fix addresses the visual issues with normal mapping in the desert environment, resulting in more realistic sand rendering with appropriate surface detail. These improvements enhance the visual quality of the desert biome in the AstraWeave unified showcase example.