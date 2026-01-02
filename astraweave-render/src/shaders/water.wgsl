// Water shader with Gerstner wave displacement
// Implements animated ocean surface with realistic wave simulation

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) wave_height: f32,
};

struct WaterUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
    water_color_deep: vec3<f32>,
    _pad1: f32,
    water_color_shallow: vec3<f32>,
    _pad2: f32,
    foam_color: vec3<f32>,
    foam_threshold: f32,
};

@group(0) @binding(0) var<uniform> uniforms: WaterUniforms;

// Gerstner wave parameters
// Each wave: (direction.x, direction.y, amplitude, frequency)
const WAVE_COUNT: u32 = 4u;

fn gerstner_wave(
    pos: vec2<f32>,
    time: f32,
    amplitude: f32,
    frequency: f32,
    speed: f32,
    direction: vec2<f32>,
    steepness: f32,
) -> vec3<f32> {
    let d = normalize(direction);
    let phase = frequency * (dot(d, pos) - speed * time);
    let Q = steepness / (frequency * amplitude * f32(WAVE_COUNT));
    
    return vec3<f32>(
        Q * amplitude * d.x * cos(phase),
        amplitude * sin(phase),
        Q * amplitude * d.y * cos(phase)
    );
}

fn gerstner_normal(
    pos: vec2<f32>,
    time: f32,
    amplitude: f32,
    frequency: f32,
    speed: f32,
    direction: vec2<f32>,
    steepness: f32,
) -> vec3<f32> {
    let d = normalize(direction);
    let phase = frequency * (dot(d, pos) - speed * time);
    let Q = steepness / (frequency * amplitude * f32(WAVE_COUNT));
    let WA = frequency * amplitude;
    
    let s = sin(phase);
    let c = cos(phase);
    
    return vec3<f32>(
        -d.x * WA * c,
        1.0 - Q * WA * s,
        -d.y * WA * c
    );
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let time = uniforms.time;
    var pos = input.position;
    
    // Apply 4 Gerstner waves with different parameters
    var displacement = vec3<f32>(0.0);
    var normal_accum = vec3<f32>(0.0, 1.0, 0.0);
    
    // Wave 1: Primary swell (large, slow)
    displacement += gerstner_wave(pos.xz, time, 0.8, 0.15, 2.0, vec2<f32>(1.0, 0.3), 0.5);
    normal_accum += gerstner_normal(pos.xz, time, 0.8, 0.15, 2.0, vec2<f32>(1.0, 0.3), 0.5);
    
    // Wave 2: Secondary swell (medium)
    displacement += gerstner_wave(pos.xz, time, 0.5, 0.25, 2.5, vec2<f32>(-0.5, 1.0), 0.4);
    normal_accum += gerstner_normal(pos.xz, time, 0.5, 0.25, 2.5, vec2<f32>(-0.5, 1.0), 0.4);
    
    // Wave 3: Chop (small, fast)
    displacement += gerstner_wave(pos.xz, time, 0.25, 0.5, 3.5, vec2<f32>(0.7, -0.7), 0.3);
    normal_accum += gerstner_normal(pos.xz, time, 0.25, 0.5, 3.5, vec2<f32>(0.7, -0.7), 0.3);
    
    // Wave 4: Ripples (tiny, very fast)
    displacement += gerstner_wave(pos.xz, time, 0.1, 1.0, 4.0, vec2<f32>(-0.3, 0.9), 0.2);
    normal_accum += gerstner_normal(pos.xz, time, 0.1, 1.0, 4.0, vec2<f32>(-0.3, 0.9), 0.2);
    
    pos.x += displacement.x;
    pos.y += displacement.y;
    pos.z += displacement.z;
    
    output.world_pos = pos;
    output.clip_position = uniforms.view_proj * vec4<f32>(pos, 1.0);
    output.uv = input.uv;
    output.normal = normalize(normal_accum);
    output.wave_height = displacement.y;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(input.normal);
    let V = normalize(uniforms.camera_pos - input.world_pos);
    
    // Fresnel effect for reflection blend
    let fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0);
    
    // Depth-based color blend (shallow vs deep)
    let depth_factor = clamp(input.wave_height * 2.0 + 0.5, 0.0, 1.0);
    let water_color = mix(uniforms.water_color_deep, uniforms.water_color_shallow, depth_factor);
    
    // Fake sky reflection (blue-ish)
    let sky_color = vec3<f32>(0.6, 0.75, 0.95);
    let reflected = mix(water_color, sky_color, fresnel * 0.6);
    
    // Sun specular highlight
    let sun_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));
    let H = normalize(V + sun_dir);
    let spec = pow(max(dot(N, H), 0.0), 128.0);
    let sun_color = vec3<f32>(1.0, 0.95, 0.8);
    
    // Foam on wave peaks
    let foam_intensity = smoothstep(uniforms.foam_threshold, uniforms.foam_threshold + 0.2, input.wave_height);
    let with_foam = mix(reflected, uniforms.foam_color, foam_intensity * 0.7);
    
    // Final color with specular
    let final_color = with_foam + sun_color * spec * 0.8;
    
    // Slight transparency for water
    let alpha = mix(0.85, 0.95, fresnel);
    
    // Final color with specular and alpha
    return vec4<f32>(final_color, alpha);
}
