// Enhanced shader for unified_showcase with improved PBR lighting, normal mapping, and texture blending

struct Camera { view_proj: mat4x4<f32> };
struct TimeUniform { time: f32, _padding: vec3<f32> };

@group(0) @binding(0) var<uniform> u_camera: Camera;
@group(1) @binding(0) var albedo_texture: texture_2d<f32>;
@group(1) @binding(1) var albedo_sampler: sampler;
@group(1) @binding(2) var normal_texture: texture_2d<f32>;
@group(1) @binding(3) var normal_sampler: sampler;

// Vertex input structure with proper attributes
struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) m0: vec4<f32>,
    @location(2) m1: vec4<f32>,
    @location(3) m2: vec4<f32>,
    @location(4) m3: vec4<f32>,
    @location(5) color: vec4<f32>,
    @location(6) mesh_type: u32,
};

// Enhanced vertex output with all necessary attributes for advanced rendering
struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) mesh_type: u32,
    @location(5) view_dir: vec3<f32>,
    @location(6) tangent: vec3<f32>,
    @location(7) bitangent: vec3<f32>,
};

// Calculate tangent and bitangent for normal mapping
fn compute_tangent_basis(normal: vec3<f32>) -> vec3<f32> {
    // Find least significant component to avoid numerical issues
    let c = abs(normal.x) > abs(normal.z) ? 
        vec3<f32>(0.0, 0.0, 1.0) : 
        vec3<f32>(1.0, 0.0, 0.0);
    
    // Compute tangent using cross product
    let tangent = normalize(cross(c, normal));
    return tangent;
}

@vertex
fn vs_main(in: VsIn) -> VsOut {
    // Extract model matrix from instance attributes
    let model = mat4x4<f32>(in.m0, in.m1, in.m2, in.m3);
    
    // Calculate world position
    let world_pos = (model * vec4<f32>(in.position, 1.0)).xyz;
    
    // Calculate UV coordinates based on position for terrain and procedural texturing
    var uv = vec2<f32>(0.0);
    if (in.mesh_type == 0u) { // Ground/terrain
        // Use world position for terrain UV coordinates with proper scaling
        uv = vec2<f32>(world_pos.x / 20.0, world_pos.z / 20.0);
    } else if (in.mesh_type == 4u) { // Skybox
        // For skybox, derive UVs from normalized position
        let pos_norm = normalize(in.position);
        uv = vec2<f32>(
            0.5 + atan2(pos_norm.z, pos_norm.x) / (2.0 * 3.14159),
            0.5 - asin(pos_norm.y) / 3.14159
        );
    } else {
        // For other meshes, generate planar mapping based on position
        // This will be replaced with proper UV mapping when mesh_helpers.rs is updated
        uv = vec2<f32>(
            (in.position.x + in.position.z) * 0.5, 
            (in.position.y + 0.5) * 0.5
        );
    }
    
    // Estimate normal based on position since we don't have it in input
    // This will be replaced with actual normals from mesh_helpers.rs
    var normal: vec3<f32>;
    if (in.mesh_type == 4u) { // Skybox - inward facing normals
        normal = normalize(-in.position);
    } else if (in.mesh_type == 0u) { // Ground/terrain
        // For terrain, estimate normals from the heightmap function
        // This will be replaced with actual computed normals
        normal = vec3<f32>(0.0, 1.0, 0.0);
    } else {
        // For other objects, estimate based on position
        normal = normalize(in.position);
    }
    
    // Transform normal to world space
    let normal_matrix = mat3x3<f32>(
        normalize(model[0].xyz),
        normalize(model[1].xyz),
        normalize(model[2].xyz)
    );
    let world_normal = normalize(normal_matrix * normal);
    
    // Compute tangent and bitangent for normal mapping
    let tangent = compute_tangent_basis(world_normal);
    let bitangent = cross(world_normal, tangent);
    
    // Calculate view direction for lighting
    let camera_pos = vec3<f32>(
        u_camera.view_proj[3][0],
        u_camera.view_proj[3][1],
        u_camera.view_proj[3][2]
    );
    let view_dir = normalize(camera_pos - world_pos);
    
    // Special handling for skybox - position at far plane
    var clip_pos = u_camera.view_proj * vec4<f32>(world_pos, 1.0);
    if (in.mesh_type == 4u) {
        clip_pos.z = clip_pos.w * 0.999; // Place skybox at far plane
    }
    
    var out: VsOut;
    out.position = clip_pos;
    out.world_pos = world_pos;
    out.normal = world_normal;
    out.uv = uv;
    out.color = in.color;
    out.mesh_type = in.mesh_type;
    out.view_dir = view_dir;
    out.tangent = tangent;
    out.bitangent = bitangent;
    
    return out;
}

// Enhanced biome detection for terrain texturing
fn get_biome_type(world_pos: vec2<f32>) -> i32 {
    let biome_scale = 0.02;
    let biome_pos = world_pos * biome_scale;
    
    // Enhanced biome detection with three distinct regions
    let primary_noise = sin(biome_pos.x * 3.0) * cos(biome_pos.y * 2.0);
    let secondary_noise = sin(biome_pos.x * 1.5 + 100.0) * cos(biome_pos.y * 1.8 + 200.0);
    let combined_noise = primary_noise * 0.7 + secondary_noise * 0.3;
    
    if (combined_noise > 0.3) {
        return 1; // Desert
    } else if (combined_noise < -0.2) {
        return 2; // Dense Forest
    } else {
        return 0; // Grassland
    }
}

// Enhanced PBR lighting calculation
fn calculate_pbr_lighting(
    normal: vec3<f32>, 
    view_dir: vec3<f32>, 
    albedo: vec3<f32>, 
    roughness: f32, 
    metallic: f32,
    time: f32
) -> vec3<f32> {
    // Base material properties
    let base_reflectivity = mix(vec3<f32>(0.04), albedo, metallic);
    
    // Environment ambient light
    let ambient_intensity = 0.2;
    let ambient = albedo * ambient_intensity;
    
    // Directional light (sun)
    let sun_angle = time * 0.05;
    let light_dir = normalize(vec3<f32>(cos(sun_angle), 0.8, sin(sun_angle)));
    let light_color = vec3<f32>(1.0, 0.98, 0.95);
    let light_intensity = 1.0;
    
    // Diffuse term (Lambert)
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let diffuse = albedo * light_color * n_dot_l;
    
    // Specular term (simplified GGX)
    let h = normalize(light_dir + view_dir);
    let n_dot_h = max(dot(normal, h), 0.0);
    let v_dot_h = max(dot(view_dir, h), 0.0);
    
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let n_dot_h2 = n_dot_h * n_dot_h;
    
    // Simplified specular BRDF
    let denom = (n_dot_h2 * (alpha2 - 1.0) + 1.0);
    let distribution = alpha2 / (3.14159 * denom * denom);
    
    let f0 = base_reflectivity;
    let fresnel = f0 + (1.0 - f0) * pow(1.0 - v_dot_h, 5.0);
    
    let specular = distribution * fresnel * light_color * n_dot_l;
    
    // Combine lighting terms
    return ambient + (diffuse + specular) * light_intensity;
}

// Enhanced sky gradient with atmospheric scattering
fn sky_color(view_dir: vec3<f32>, time: f32) -> vec3<f32> {
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let y = dot(normalize(view_dir), up);
    
    // Time-based sun position
    let sun_angle = time * 0.05;
    let sun_dir = normalize(vec3<f32>(cos(sun_angle), sin(sun_angle) * 0.4, sin(sun_angle) * 0.2));
    
    // Sun influence
    let sun_dot = max(dot(normalize(view_dir), sun_dir), 0.0);
    let sun_influence = pow(sun_dot, 512.0); // Sun disk
    let sun_halo = pow(sun_dot, 32.0) * 0.25; // Softer sun halo
    
    // Day/night cycle
    let day_factor = clamp(sun_dir.y + 0.2, 0.0, 1.0);
    
    // Sky gradients
    let zenith_day = vec3<f32>(0.2, 0.4, 0.8);
    let horizon_day = vec3<f32>(0.7, 0.8, 0.9);
    
    let zenith_night = vec3<f32>(0.02, 0.05, 0.1);
    let horizon_night = vec3<f32>(0.05, 0.1, 0.2);
    
    // Sunset colors
    let sunset_factor = clamp(1.0 - abs(sun_dir.y) * 5.0, 0.0, 1.0) * day_factor;
    let zenith_sunset = vec3<f32>(0.2, 0.1, 0.3);
    let horizon_sunset = vec3<f32>(0.8, 0.3, 0.1);
    
    // Blend sky gradients
    let horizon_color = mix(
        mix(horizon_night, horizon_day, day_factor),
        horizon_sunset, 
        sunset_factor
    );
    
    let zenith_color = mix(
        mix(zenith_night, zenith_day, day_factor),
        zenith_sunset,
        sunset_factor
    );
    
    // Calculate gradient based on view direction
    let atmosphere_curve = pow(clamp((y + 0.5) * 0.8, 0.0, 1.0), 0.75);
    let sky_base = mix(horizon_color, zenith_color, atmosphere_curve);
    
    // Add sun and halo
    let sun_color = vec3<f32>(1.0, 0.9, 0.7) * day_factor;
    let sun = sun_influence * sun_color + sun_halo * mix(sun_color, horizon_color, 0.5);
    
    // Stars at night
    let stars = vec3<f32>(0.0);
    if (day_factor < 0.2) {
        let star_field = fract(sin(dot(normalize(view_dir) * 100.0, vec3<f32>(12.9898, 78.233, 45.164))) * 43758.5453);
        if (star_field > 0.997) {
            let star_brightness = (1.0 - day_factor * 5.0) * (star_field - 0.997) * 100.0;
            let star_color = vec3<f32>(1.0, 0.9, 0.8);
            stars = star_color * star_brightness;
        }
    }
    
    return sky_base + sun + stars;
}

// Calculate biome-specific terrain texturing
fn get_terrain_texture(
    world_pos: vec3<f32>,
    biome_type: i32,
    normal: vec3<f32>,
    base_texture: vec3<f32>
) -> vec3<f32> {
    var result = base_texture;
    
    if (biome_type == 0) { // Grassland
        // Add subtle variation to grass
        let height_factor = sin(world_pos.x * 0.05) * cos(world_pos.z * 0.05) * 0.5 + 0.5;
        let grass_color = mix(
            vec3<f32>(0.3, 0.5, 0.2), // Darker grass
            vec3<f32>(0.5, 0.7, 0.3), // Lighter grass
            height_factor
        );
        
        // Blend with dirt on slopes
        let slope = 1.0 - normal.y;
        let dirt_color = vec3<f32>(0.5, 0.3, 0.2);
        result = mix(grass_color, dirt_color, slope * 0.7);
        
    } else if (biome_type == 1) { // Desert
        // Sand texture with height variation
        let height_factor = sin(world_pos.x * 0.03) * cos(world_pos.z * 0.03) * 0.5 + 0.5;
        let sand_color = mix(
            vec3<f32>(0.8, 0.7, 0.5), // Darker sand
            vec3<f32>(0.9, 0.85, 0.7), // Lighter sand
            height_factor
        );
        
        // Add rocky outcrops on steep slopes
        let slope = 1.0 - normal.y;
        let rock_color = vec3<f32>(0.6, 0.5, 0.4);
        result = mix(sand_color, rock_color, slope * 0.8);
        
    } else if (biome_type == 2) { // Forest
        // Rich forest floor with patches
        let pattern = sin(world_pos.x * 0.1) * cos(world_pos.z * 0.1) * 0.5 + 0.5;
        let forest_color = mix(
            vec3<f32>(0.3, 0.25, 0.2), // Dark soil
            vec3<f32>(0.25, 0.35, 0.2), // Mossy ground
            pattern
        );
        
        // Add fallen leaves effect
        let leaf_pattern = fract(sin(dot(floor(world_pos.xz * 0.5), vec2<f32>(12.9898, 78.233))) * 43758.5453);
        if (leaf_pattern > 0.8) {
            let leaf_color = vec3<f32>(0.6, 0.4, 0.3);
            result = mix(forest_color, leaf_color, (leaf_pattern - 0.8) * 5.0);
        } else {
            result = forest_color;
        }
    }
    
    return result;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Fixed time value (should be passed as a uniform in production)
    let time: f32 = 10.0;
    
    // Base color from vertex
    var base_color = in.color.rgb;
    
    // Transform normal from texture space to world space
    var normal = in.normal;
    let biome_type = get_biome_type(in.world_pos.xz);
    
    // Apply different rendering based on mesh type
    if (in.mesh_type == 0u) { // Ground/terrain
        // Sample base texture
        let tex_color = textureSample(albedo_texture, albedo_sampler, in.uv).rgb;
        
        // Sample normal map
        let normal_sample = textureSample(normal_texture, normal_sampler, in.uv).rgb;
        let normal_tangent = normalize(normal_sample * 2.0 - 1.0);
        
        // Create TBN matrix to transform normal from tangent to world space
        let N = normalize(in.normal);
        let T = normalize(in.tangent);
        let B = normalize(in.bitangent);
        let tbn = mat3x3<f32>(T, B, N);
        
        // Transform normal from tangent space to world space
        normal = normalize(tbn * normal_tangent);
        
        // Get biome-specific terrain texturing
        base_color = get_terrain_texture(in.world_pos, biome_type, normal, tex_color);
        
        // Calculate PBR lighting with medium roughness for terrain
        let roughness = 0.8;
        let metallic = 0.0; // Non-metallic terrain
        let final_color = calculate_pbr_lighting(normal, in.view_dir, base_color, roughness, metallic, time);
        
        // Add atmospheric distance fog
        let dist = length(in.world_pos);
        let fog_factor = clamp(dist / 100.0, 0.0, 0.8);
        let fog_color = sky_color(vec3<f32>(0.0, 0.5, 1.0), time);
        
        return vec4<f32>(mix(final_color, fog_color, fog_factor), 1.0);
        
    } else if (in.mesh_type == 1u) { // Trees
        // Trunk vs. leaves color
        var tree_color: vec3<f32>;
        var roughness: f32;
        var metallic: f32;
        
        if (in.world_pos.y > 0.0) { // Simple check for leaves vs trunk
            // Leaves
            tree_color = vec3<f32>(0.2, 0.5, 0.15) * base_color;
            roughness = 0.9;
            metallic = 0.0;
            
            // Add variation to leaves
            let leaf_pattern = sin(in.world_pos.x * 10.0) * cos(in.world_pos.z * 10.0) * 0.5 + 0.5;
            tree_color = mix(tree_color, tree_color * 1.3, leaf_pattern);
        } else {
            // Trunk
            tree_color = vec3<f32>(0.4, 0.3, 0.2) * base_color;
            roughness = 0.7;
            metallic = 0.0;
            
            // Add bark texture
            let bark_pattern = sin(in.world_pos.y * 20.0) * 0.5 + 0.5;
            tree_color = mix(tree_color, tree_color * 0.7, bark_pattern);
        }
        
        // Calculate PBR lighting
        let final_color = calculate_pbr_lighting(normal, in.view_dir, tree_color, roughness, metallic, time);
        return vec4<f32>(final_color, 1.0);
        
    } else if (in.mesh_type == 2u) { // Houses
        // House materials
        var house_color = base_color;
        var roughness: f32 = 0.6;
        var metallic: f32 = 0.0;
        
        // Add simple texturing for walls/roof
        if (in.world_pos.y > 0.3) { // Roof
            house_color = vec3<f32>(0.6, 0.3, 0.2); // Reddish roof
            roughness = 0.8;
        } else { // Walls
            house_color = vec3<f32>(0.8, 0.75, 0.6); // Light wall color
            roughness = 0.5;
            
            // Add subtle wall texture
            let wall_pattern = fract(in.world_pos.y * 5.0);
            if (wall_pattern < 0.05 || wall_pattern > 0.95) {
                house_color *= 0.8; // Darker lines for bricks/planks
            }
        }
        
        // Calculate PBR lighting
        let final_color = calculate_pbr_lighting(normal, in.view_dir, house_color, roughness, metallic, time);
        return vec4<f32>(final_color, 1.0);
        
    } else if (in.mesh_type == 3u) { // Characters
        // Character materials with enhanced detail
        var char_color = base_color;
        let roughness: f32 = 0.7;
        let metallic: f32 = 0.0;
        
        // Add simple body parts coloring
        if (in.world_pos.y > 0.7) { // Head
            char_color = vec3<f32>(0.8, 0.6, 0.5); // Skin tone
        } else if (in.world_pos.y > 0.2) { // Torso
            char_color = vec3<f32>(0.2, 0.4, 0.7) * base_color; // Clothing color influenced by base color
        } else { // Legs
            char_color = vec3<f32>(0.3, 0.3, 0.4) * base_color; // Darker clothing for legs
        }
        
        // Calculate PBR lighting
        let final_color = calculate_pbr_lighting(normal, in.view_dir, char_color, roughness, metallic, time);
        return vec4<f32>(final_color, 1.0);
        
    } else if (in.mesh_type == 4u) { // Skybox
        // Pure procedural sky rendering
        return vec4<f32>(sky_color(in.view_dir, time), 1.0);
    }
    
    // Fallback for unknown mesh types
    return vec4<f32>(base_color, 1.0);
}