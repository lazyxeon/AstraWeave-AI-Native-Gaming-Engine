// Clean PBR Shader - No atlas complexity, just simple texturing
// Each material has its own texture binding

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

struct ModelUniforms {
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var albedo_texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var normal_texture: texture_2d<f32>;

@group(1) @binding(3)
var roughness_texture: texture_2d<f32>;

@group(2) @binding(0)
var<uniform> model: ModelUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,  // Vertex color
    @location(4) tangent: vec4<f32>, // Tangent.xyz and handedness in w
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,  // Pass through vertex color
    @location(4) tangent: vec4<f32>, // Tangent passed to fragment
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply model matrix to transform object into world space
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    
    // Transform normal to world space (assuming uniform scale for now)
    let normal_matrix = mat3x3<f32>(
        model.model[0].xyz,
        model.model[1].xyz,
        model.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * in.normal);
    out.uv = in.uv;
    out.color = in.color;  // Pass through vertex color
    // Transform tangent vector to world space (note: tangent.w is handedness)
    let tangent_ws = normalize(normal_matrix * in.tangent.xyz);
    out.tangent = vec4<f32>(tangent_ws, in.tangent.w);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Use vertex color as base color (Kenney models use vertex colors!)
    // Multiply by texture for detail (if texture exists, otherwise just vertex color)
    let texture_color = textureSample(albedo_texture, texture_sampler, in.uv).rgb;
    let normal_sample = textureSample(normal_texture, texture_sampler, in.uv).rgb;
    let roughness_sample = textureSample(roughness_texture, texture_sampler, in.uv).r;
    
    // Blend vertex color with texture (vertex color is the primary color)
    let albedo = in.color.rgb * texture_color;
    // Apply tangent-space normal mapping approximation
    // Convert normal_sample from [0,1] to [-1,1]
    let n_sample = normal_sample * 2.0 - vec3<f32>(1.0, 1.0, 1.0);
    // Build TBN using provided tangent
    let tangent_vec = normalize(in.tangent.xyz);
    let handedness = in.tangent.w;
    let bitangent = cross(in.world_normal, tangent_vec) * handedness;
    let TBN = mat3x3<f32>(tangent_vec, bitangent, in.world_normal);
    let mapped_normal = normalize(TBN * n_sample);
    
    // Enhanced lighting with multiple light sources
    let view_dir = normalize(camera.camera_pos - in.world_position);
    
    // Main directional light (sun)
    let sun_dir = normalize(vec3<f32>(0.5, 0.8, 0.3));
    let sun_color = vec3<f32>(1.0, 0.95, 0.9);
    let diffuse_sun = max(dot(in.world_normal, sun_dir), 0.0);
    
    // Specular highlight
    let halfway = normalize(sun_dir + view_dir);
    let specular = pow(max(dot(in.world_normal, halfway), 0.0), 32.0) * 0.3;
    
    // Sky ambient (blueish from above)
    let sky_ambient = vec3<f32>(0.4, 0.5, 0.6) * max(in.world_normal.y * 0.5 + 0.5, 0.0);
    
    // Ground ambient (brownish from below)
    let ground_ambient = vec3<f32>(0.2, 0.15, 0.1) * max(-in.world_normal.y * 0.5 + 0.5, 0.0);
    
    // Combine all lighting
    let ambient = sky_ambient + ground_ambient;
    let diffuse = diffuse_sun * sun_color;
    let lighting = ambient + diffuse + vec3<f32>(specular);
    
    // Use sampled roughness if available (mix with default roughness 0.5)
    let roughness = mix(0.5, roughness_sample, 0.8);
    let final_color = albedo * lighting * (1.0 - roughness * 0.1);
    
    return vec4<f32>(final_color, 1.0);
}
