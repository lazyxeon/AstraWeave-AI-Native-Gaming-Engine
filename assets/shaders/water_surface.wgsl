// Water Surface Shader
// Renders the volumetric water surface with flow effects, fresnel reflections,
// depth-based coloring, and animated waves.

// ============================================
// Constants
// ============================================

const PI: f32 = 3.14159265359;
const WATER_COLOR_SHALLOW: vec3<f32> = vec3<f32>(0.3, 0.7, 0.9);  // Cyan-ish
const WATER_COLOR_DEEP: vec3<f32> = vec3<f32>(0.0, 0.15, 0.4);    // Dark blue
const WATER_FOAM_COLOR: vec3<f32> = vec3<f32>(0.9, 0.95, 1.0);    // White foam
const WATER_ABSORPTION: f32 = 0.8;  // How quickly light is absorbed in deep water
const FRESNEL_POWER: f32 = 4.0;
const FRESNEL_BIAS: f32 = 0.04;
const SPECULAR_POWER: f32 = 64.0;

// ============================================
// Uniforms
// ============================================

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    camera_position: vec3<f32>,
    near_plane: f32,
    far_plane: f32,
    time: f32,
    _padding: vec2<f32>,
}

struct WaterUniforms {
    dimensions: vec4<u32>,      // x, y, z, padding
    origin: vec4<f32>,          // world origin
    cell_size: vec4<f32>,       // cell size in world units
    animation: vec4<f32>,       // time, flow_speed, wave_height, wave_frequency
}

struct LightUniforms {
    sun_direction: vec3<f32>,
    sun_color: vec3<f32>,
    sun_intensity: f32,
    ambient_color: vec3<f32>,
    ambient_intensity: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> light: LightUniforms;

@group(1) @binding(0) var water_volume: texture_3d<f32>;
@group(1) @binding(1) var water_sampler: sampler;
@group(1) @binding(2) var<uniform> water: WaterUniforms;

// Optional: reflection and refraction textures
@group(2) @binding(0) var reflection_texture: texture_2d<f32>;
@group(2) @binding(1) var reflection_sampler: sampler;
@group(2) @binding(2) var refraction_texture: texture_2d<f32>;
@group(2) @binding(3) var depth_texture: texture_depth_2d;

// ============================================
// Vertex Input/Output
// ============================================

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) flow_velocity: vec2<f32>,
    @location(4) water_depth: f32,
}

// ============================================
// Noise Functions
// ============================================

// Simple hash function for noise
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.13);
    p3 = p3 + dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

// Value noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f); // Smooth interpolation

    return mix(
        mix(hash(i + vec2<f32>(0.0, 0.0)), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// Fractal Brownian Motion (FBM) for wave detail
fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var pos = p;
    
    for (var i = 0; i < octaves; i = i + 1) {
        value = value + amplitude * noise(pos * frequency);
        amplitude = amplitude * 0.5;
        frequency = frequency * 2.0;
    }
    
    return value;
}

// ============================================
// Wave Functions
// ============================================

// Gerstner wave (realistic ocean waves)
fn gerstner_wave(
    position: vec2<f32>,
    time: f32,
    direction: vec2<f32>,
    steepness: f32,
    wavelength: f32
) -> vec3<f32> {
    let k = 2.0 * PI / wavelength;
    let c = sqrt(9.8 / k);  // Wave speed from gravity
    let d = normalize(direction);
    let f = k * (dot(d, position) - c * time);
    let a = steepness / k;  // Amplitude
    
    return vec3<f32>(
        d.x * a * cos(f),
        a * sin(f),
        d.y * a * cos(f)
    );
}

// Combined wave displacement
fn wave_displacement(position: vec2<f32>, time: f32, wave_height: f32, wave_freq: f32) -> vec3<f32> {
    var displacement = vec3<f32>(0.0);
    
    // Primary wave
    displacement = displacement + gerstner_wave(position, time, vec2<f32>(1.0, 0.3), 0.3, 10.0 / wave_freq);
    
    // Secondary waves for detail
    displacement = displacement + gerstner_wave(position, time * 1.1, vec2<f32>(-0.7, 0.7), 0.15, 5.0 / wave_freq);
    displacement = displacement + gerstner_wave(position, time * 0.9, vec2<f32>(0.3, -1.0), 0.1, 3.0 / wave_freq);
    
    // Scale by wave height parameter
    displacement = displacement * wave_height;
    
    // Add small-scale detail noise
    let detail = fbm(position * 2.0 + time * 0.5, 3) * wave_height * 0.1;
    displacement.y = displacement.y + detail;
    
    return displacement;
}

// ============================================
// Normal Calculation
// ============================================

// Calculate wave normal from displacement gradient
fn wave_normal(position: vec2<f32>, time: f32, wave_height: f32, wave_freq: f32) -> vec3<f32> {
    let epsilon = 0.1;
    
    let h0 = wave_displacement(position, time, wave_height, wave_freq).y;
    let hx = wave_displacement(position + vec2<f32>(epsilon, 0.0), time, wave_height, wave_freq).y;
    let hz = wave_displacement(position + vec2<f32>(0.0, epsilon), time, wave_height, wave_freq).y;
    
    let dx = (hx - h0) / epsilon;
    let dz = (hz - h0) / epsilon;
    
    return normalize(vec3<f32>(-dx, 1.0, -dz));
}

// ============================================
// Vertex Shader
// ============================================

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let time = water.animation.x;
    let flow_speed = water.animation.y;
    let wave_height = water.animation.z;
    let wave_freq = water.animation.w;
    
    // Sample flow velocity from volume texture
    let volume_uv = (input.position - water.origin.xyz) / (vec3<f32>(water.dimensions.xyz) * water.cell_size.xyz);
    let volume_sample = textureSampleLevel(water_volume, water_sampler, volume_uv, 0.0);
    let flow_velocity = vec2<f32>(volume_sample.y, volume_sample.w); // velocity_x, velocity_z
    
    // Apply wave displacement
    let wave_pos = input.position.xz + flow_velocity * time * flow_speed;
    let displacement = wave_displacement(wave_pos, time, wave_height, wave_freq);
    
    var world_position = input.position + displacement;
    
    // Calculate wave normal
    let wave_norm = wave_normal(wave_pos, time, wave_height, wave_freq);
    let final_normal = normalize(input.normal + wave_norm * 0.5);
    
    output.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    output.world_position = world_position;
    output.world_normal = final_normal;
    output.uv = input.uv + flow_velocity * time * flow_speed * 0.1;
    output.flow_velocity = flow_velocity;
    output.water_depth = volume_sample.x; // water level
    
    return output;
}

// ============================================
// Fragment Shader
// ============================================

// Fresnel effect for realistic water reflection
fn fresnel(view_dir: vec3<f32>, normal: vec3<f32>) -> f32 {
    let dot_nv = max(dot(normal, view_dir), 0.0);
    return FRESNEL_BIAS + (1.0 - FRESNEL_BIAS) * pow(1.0 - dot_nv, FRESNEL_POWER);
}

// Specular highlight
fn specular(view_dir: vec3<f32>, light_dir: vec3<f32>, normal: vec3<f32>) -> f32 {
    let half_vec = normalize(view_dir + light_dir);
    let spec = pow(max(dot(normal, half_vec), 0.0), SPECULAR_POWER);
    return spec;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let time = water.animation.x;
    
    // Calculate view direction
    let view_dir = normalize(camera.camera_position - input.world_position);
    
    // Perturb normal with flow-based detail
    var normal = input.world_normal;
    let flow_perturbation = vec3<f32>(
        noise(input.uv * 20.0 + time * 0.5) * 0.1,
        0.0,
        noise(input.uv * 20.0 + time * 0.5 + 1000.0) * 0.1
    );
    normal = normalize(normal + flow_perturbation);
    
    // Fresnel for reflection/refraction mix
    let fres = fresnel(view_dir, normal);
    
    // Base water color based on depth
    let depth_factor = saturate(input.water_depth * WATER_ABSORPTION);
    var water_color = mix(WATER_COLOR_SHALLOW, WATER_COLOR_DEEP, depth_factor);
    
    // Add flow-based foam at high velocities
    let flow_speed = length(input.flow_velocity);
    let foam_factor = saturate((flow_speed - 0.5) * 2.0);
    water_color = mix(water_color, WATER_FOAM_COLOR, foam_factor * 0.5);
    
    // Lighting
    let light_dir = normalize(-light.sun_direction);
    let ndotl = max(dot(normal, light_dir), 0.0);
    
    // Diffuse lighting (subsurface scattering approximation)
    let sss_factor = saturate(dot(-view_dir, light_dir) * 0.5 + 0.5);
    let sss_color = WATER_COLOR_SHALLOW * sss_factor * 0.3;
    
    let diffuse = water_color * (ndotl * light.sun_intensity + light.ambient_intensity) * light.sun_color;
    
    // Specular highlight
    let spec = specular(view_dir, light_dir, normal);
    let specular_color = light.sun_color * spec * light.sun_intensity * fres;
    
    // Combine reflection (sky color approximation) with water color
    let sky_color = vec3<f32>(0.5, 0.7, 1.0); // Simplified sky reflection
    let reflection = sky_color * fres;
    
    // Final color
    var final_color = diffuse + specular_color + reflection * 0.3 + sss_color;
    
    // Apply depth-based transparency
    let alpha = mix(0.7, 0.95, depth_factor);
    
    // Edge foam where water meets surfaces (based on UV proximity to edges)
    let edge_foam = saturate(1.0 - min(input.uv.x, min(1.0 - input.uv.x, min(input.uv.y, 1.0 - input.uv.y))) * 10.0);
    final_color = mix(final_color, WATER_FOAM_COLOR, edge_foam * 0.3);
    
    return vec4<f32>(final_color, alpha);
}

// ============================================
// Underwater Post-Process Shader
// ============================================

struct UnderwaterUniforms {
    fog_color: vec3<f32>,
    fog_density: f32,
    caustics_intensity: f32,
    caustics_scale: f32,
    time: f32,
    water_depth: f32,
}

@group(3) @binding(0) var<uniform> underwater: UnderwaterUniforms;
@group(3) @binding(1) var screen_texture: texture_2d<f32>;
@group(3) @binding(2) var screen_sampler: sampler;

struct UnderwaterVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_underwater(@builtin(vertex_index) vertex_index: u32) -> UnderwaterVertexOutput {
    // Full-screen triangle
    var output: UnderwaterVertexOutput;
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;
    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return output;
}

// Caustics pattern
fn caustics(uv: vec2<f32>, time: f32) -> f32 {
    // Multiple overlapping wave patterns for caustics effect
    let scale = underwater.caustics_scale;
    var c = 0.0;
    
    // Layer 1
    c = c + sin(uv.x * scale + time) * sin(uv.y * scale + time * 0.7);
    // Layer 2
    c = c + sin(uv.x * scale * 1.3 - time * 0.8) * sin(uv.y * scale * 1.3 + time * 1.1);
    // Layer 3
    c = c + sin((uv.x + uv.y) * scale * 0.7 + time * 0.5);
    
    return (c + 3.0) / 6.0; // Normalize to 0-1
}

@fragment
fn fs_underwater(input: UnderwaterVertexOutput) -> @location(0) vec4<f32> {
    // Sample scene
    var uv = input.uv;
    
    // Distortion effect from water
    let distortion_strength = 0.01;
    let distort_time = underwater.time * 2.0;
    uv.x = uv.x + sin(uv.y * 20.0 + distort_time) * distortion_strength;
    uv.y = uv.y + cos(uv.x * 20.0 + distort_time) * distortion_strength;
    
    var scene_color = textureSample(screen_texture, screen_sampler, uv).rgb;
    
    // Apply caustics
    let caustic_pattern = caustics(uv * 10.0, underwater.time);
    let caustic_color = vec3<f32>(caustic_pattern) * underwater.caustics_intensity;
    scene_color = scene_color + caustic_color;
    
    // Underwater fog
    let fog_factor = 1.0 - exp(-underwater.fog_density * underwater.water_depth);
    scene_color = mix(scene_color, underwater.fog_color, fog_factor);
    
    // Color absorption (red fades first, then green)
    let absorption = vec3<f32>(
        exp(-underwater.water_depth * 0.3),  // Red absorbs quickly
        exp(-underwater.water_depth * 0.1),  // Green absorbs slower
        exp(-underwater.water_depth * 0.05)  // Blue absorbs slowest
    );
    scene_color = scene_color * absorption;
    
    // Vignette for underwater feeling
    let vignette = 1.0 - length(input.uv - 0.5) * 0.5;
    scene_color = scene_color * vignette;
    
    return vec4<f32>(scene_color, 1.0);
}
