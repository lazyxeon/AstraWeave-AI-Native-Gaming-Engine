// Anchor VFX Shader - Emissive glow and state-based visual effects
// 
// This shader renders the Anchor's reality-stabilizing field with:
// - State-based color transitions (Perfect → Stable → Unstable → Critical → Broken)
// - Emissive glow intensity based on stability
// - Animated distortion effects for damaged states
// - Particle emission integration

// ============================================================================
// Uniforms and Bindings
// ============================================================================

struct AnchorUniforms {
    // Anchor state
    stability: f32,              // 0.0-1.0 stability value
    vfx_state: u32,              // 0=Perfect, 1=Stable, 2=Unstable, 3=Critical, 4=Broken
    time_since_repair: f32,      // For repair animation (0.0-5.0s)
    is_repaired: u32,            // 0=false, 1=true
    
    // World transform
    world_position: vec3<f32>,
    world_scale: f32,
    
    // Animation
    time: f32,                   // Global time for effects
    delta_time: f32,             // Frame delta for smooth transitions
    
    // Camera
    camera_position: vec3<f32>,
    camera_forward: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> anchor: AnchorUniforms;

@group(0) @binding(1)
var anchor_texture: texture_2d<f32>;

@group(0) @binding(2)
var anchor_sampler: sampler;

// ============================================================================
// Constants - VFX State Colors
// ============================================================================

const COLOR_PERFECT: vec3<f32> = vec3<f32>(0.2, 0.6, 1.0);     // Bright blue
const COLOR_STABLE: vec3<f32> = vec3<f32>(0.15, 0.4, 0.7);     // Dim blue
const COLOR_UNSTABLE: vec3<f32> = vec3<f32>(0.9, 0.7, 0.2);    // Yellow
const COLOR_CRITICAL: vec3<f32> = vec3<f32>(1.0, 0.2, 0.1);    // Red
const COLOR_BROKEN: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);      // Black (no glow)

// Glow intensities
const GLOW_PERFECT: f32 = 1.0;      // 100% emissive
const GLOW_STABLE: f32 = 0.6;       // 60% emissive
const GLOW_UNSTABLE: f32 = 0.8;     // 80% emissive (warning glow)
const GLOW_CRITICAL: f32 = 1.0;     // 100% emissive (danger glow)
const GLOW_BROKEN: f32 = 0.0;       // 0% emissive (dead)

// Distortion parameters
const DISTORTION_UNSTABLE: f32 = 0.02;   // 2% position distortion
const DISTORTION_CRITICAL: f32 = 0.05;   // 5% position distortion
const FLICKER_FREQUENCY: f32 = 5.0;      // Hz for flickering

// Repair animation
const REPAIR_DURATION: f32 = 5.0;        // 5s animation

// ============================================================================
// Helper Functions
// ============================================================================

// Get base color for current VFX state
fn get_state_color(state: u32) -> vec3<f32> {
    switch state {
        case 0u: { return COLOR_PERFECT; }    // Perfect
        case 1u: { return COLOR_STABLE; }     // Stable
        case 2u: { return COLOR_UNSTABLE; }   // Unstable
        case 3u: { return COLOR_CRITICAL; }   // Critical
        default: { return COLOR_BROKEN; }     // Broken
    }
}

// Get glow intensity for current VFX state
fn get_glow_intensity(state: u32) -> f32 {
    switch state {
        case 0u: { return GLOW_PERFECT; }
        case 1u: { return GLOW_STABLE; }
        case 2u: { return GLOW_UNSTABLE; }
        case 3u: { return GLOW_CRITICAL; }
        default: { return GLOW_BROKEN; }
    }
}

// Smooth fade between two colors based on stability
fn lerp_color(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a + (b - a) * clamp(t, 0.0, 1.0);
}

// Smooth step for fade transitions
fn smooth_fade(t: f32) -> f32 {
    let clamped = clamp(t, 0.0, 1.0);
    return clamped * clamped * (3.0 - 2.0 * clamped);
}

// Flicker effect for stable/unstable states
fn get_flicker(time: f32, frequency: f32, state: u32) -> f32 {
    if state == 0u {
        return 1.0;  // Perfect - no flicker
    }
    
    if state == 1u {
        // Stable - subtle flicker (5% variation)
        return 0.975 + 0.025 * sin(time * frequency);
    }
    
    if state == 2u {
        // Unstable - moderate flicker (15% variation)
        return 0.925 + 0.075 * sin(time * frequency * 2.0);
    }
    
    // Critical - harsh flicker (30% variation)
    return 0.85 + 0.15 * sin(time * frequency * 4.0);
}

// Position distortion for damaged states
fn get_distortion(position: vec3<f32>, time: f32, state: u32) -> vec3<f32> {
    if state < 2u {
        return position;  // Perfect/Stable - no distortion
    }
    
    var distortion_amount = 0.0;
    if state == 2u {
        distortion_amount = DISTORTION_UNSTABLE;
    } else if state == 3u {
        distortion_amount = DISTORTION_CRITICAL;
    }
    
    // Perlin-like distortion using sine waves
    let offset_x = sin(time * 3.0 + position.y * 10.0) * distortion_amount;
    let offset_y = sin(time * 4.0 + position.z * 10.0) * distortion_amount;
    let offset_z = sin(time * 5.0 + position.x * 10.0) * distortion_amount;
    
    return position + vec3<f32>(offset_x, offset_y, offset_z);
}

// Repair animation overlay (blue restoration wave)
fn get_repair_color(base_color: vec3<f32>, progress: f32, uv: vec2<f32>) -> vec3<f32> {
    if progress >= 1.0 {
        return base_color;  // Repair complete
    }
    
    // Wave moves from bottom to top (0.0 → 1.0 over 5s)
    let wave_position = progress;
    let distance_to_wave = abs(uv.y - wave_position);
    
    // Wave width (10% of height)
    let wave_width = 0.1;
    
    if distance_to_wave < wave_width {
        // Inside wave - blend with blue restoration color
        let wave_intensity = 1.0 - (distance_to_wave / wave_width);
        let restoration_color = vec3<f32>(0.3, 0.8, 1.0);  // Bright cyan
        return lerp_color(base_color, restoration_color, wave_intensity * 0.7);
    }
    
    return base_color;
}

// ============================================================================
// Vertex Shader
// ============================================================================

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_direction: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply distortion for damaged states
    let distorted_position = get_distortion(in.position, anchor.time, anchor.vfx_state);
    
    // Transform to world space
    let world_pos = anchor.world_position + distorted_position * anchor.world_scale;
    out.world_position = world_pos;
    out.world_normal = normalize(in.normal);
    out.uv = in.uv;
    
    // View direction for Fresnel effect
    out.view_direction = normalize(anchor.camera_position - world_pos);
    
    // For now, use simple projection (will be replaced by proper camera matrix)
    let clip_pos = world_pos - anchor.camera_position;
    out.clip_position = vec4<f32>(clip_pos.x, clip_pos.y, -clip_pos.z, 1.0);
    
    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Base color from VFX state
    var base_color = get_state_color(anchor.vfx_state);
    
    // Apply flicker effect
    let flicker = get_flicker(anchor.time, FLICKER_FREQUENCY, anchor.vfx_state);
    base_color *= flicker;
    
    // Sample texture (if provided, otherwise use solid color)
    let texture_color = textureSample(anchor_texture, anchor_sampler, in.uv);
    base_color *= texture_color.rgb;
    
    // Fresnel effect for edge glow
    let fresnel = pow(1.0 - max(dot(in.view_direction, in.world_normal), 0.0), 3.0);
    let edge_glow = fresnel * get_glow_intensity(anchor.vfx_state);
    
    // Emissive glow
    var final_color = base_color * (1.0 + edge_glow);
    
    // Repair animation overlay
    if anchor.is_repaired == 1u && anchor.time_since_repair < REPAIR_DURATION {
        let repair_progress = anchor.time_since_repair / REPAIR_DURATION;
        final_color = get_repair_color(final_color, repair_progress, in.uv);
    }
    
    // Alpha based on state (broken = fully transparent)
    var alpha = 1.0;
    if anchor.vfx_state == 4u {
        alpha = 0.0;  // Broken - invisible
    } else {
        alpha = texture_color.a * flicker;
    }
    
    return vec4<f32>(final_color, alpha);
}

// ============================================================================
// Particle Emission Helper (for CPU-side integration)
// ============================================================================

// This section documents the particle emission rates for CPU-side systems.
// The actual particle spawning is handled by anchor_particle.rs, but this
// shader provides visual feedback for the particle system state.

// Emission rates (particles per second):
// - Perfect (1.0):     0 particles/sec (stable, no glitches)
// - Stable (0.7-0.99): 5 particles/sec (rare glitches)
// - Unstable (0.4-0.69): 20 particles/sec (frequent glitches)
// - Critical (0.1-0.39): 50 particles/sec (reality tears)
// - Broken (0.0):      100 particles/sec (catastrophic)

// Particle types by state:
// - Stable: Small blue sparks (0.5s lifetime, fade out)
// - Unstable: Yellow glitches (1.0s lifetime, erratic motion)
// - Critical: Red tears (2.0s lifetime, expanding)
// - Broken: Black void particles (3.0s lifetime, gravity pull)

// ============================================================================
// Audio Integration Notes
// ============================================================================

// This shader's VFX state transitions should trigger corresponding audio:
// - Perfect → Stable: Fade in 440 Hz hum (1s fade)
// - Stable → Unstable: Crossfade to distorted 300-350 Hz (0.5s crossfade)
// - Unstable → Critical: Crossfade to harsh static 200-250 Hz (0.5s crossfade)
// - Critical → Broken: Fade out all audio (2s fade)
// - Any → Repair: Play anchor_repair.ogg (5s sound, overlays hum)

// Audio volumes by state:
// - Perfect: 0% (silent, pristine reality)
// - Stable: 20% (subtle hum, stable)
// - Unstable: 50% (moderate hum, warning)
// - Critical: 80% (loud static, danger)
// - Broken: 0% (silent, dead)

// ============================================================================
// Performance Notes
// ============================================================================

// Estimated GPU cost (per anchor @ 1080p):
// - Vertex shader: ~50 ns per vertex (200 verts = 10 µs)
// - Fragment shader: ~100 ns per pixel (10k pixels = 1 ms)
// - Total per anchor: ~1 ms
// - 60 FPS budget: 16.67 ms (59 anchors max @ 1 ms each, ~3.6% per anchor)
//
// Optimization opportunities:
// - LOD system: Reduce vertex count at distance
// - Culling: Skip anchors outside camera frustum
// - Instancing: Batch multiple anchors with same state
// - Mipmap textures: Reduce texture sampling cost
//
// Current implementation targets 10-20 anchors per scene, well within budget.
