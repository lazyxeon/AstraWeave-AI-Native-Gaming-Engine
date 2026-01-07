struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    shading_mode: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) biome_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) biome_id: u32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn biome_color(biome_id: u32) -> vec3<f32> {
    switch biome_id {
        case 0u: { return vec3<f32>(0.45, 0.65, 0.35); } // Grassland - green
        case 1u: { return vec3<f32>(0.85, 0.75, 0.55); } // Desert - sandy
        case 2u: { return vec3<f32>(0.25, 0.50, 0.25); } // Forest - dark green
        case 3u: { return vec3<f32>(0.55, 0.55, 0.55); } // Mountain - gray
        case 4u: { return vec3<f32>(0.90, 0.95, 0.98); } // Tundra - white/blue
        case 5u: { return vec3<f32>(0.35, 0.45, 0.35); } // Swamp - murky green
        case 6u: { return vec3<f32>(0.90, 0.85, 0.70); } // Beach - light sand
        case 7u: { return vec3<f32>(0.30, 0.50, 0.70); } // River - blue
        default: { return vec3<f32>(0.5, 0.5, 0.5); }    // Unknown - gray
    }
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.view_proj * vec4<f32>(vertex.position, 1.0);
    output.world_position = vertex.position;
    output.world_normal = normalize(vertex.normal);
    output.uv = vertex.uv;
    output.biome_id = vertex.biome_id;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = biome_color(in.biome_id);
    
    if uniforms.shading_mode == 1u {
        return vec4<f32>(base_color, 1.0);
    }
    
    if uniforms.shading_mode == 2u {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    }
    
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.35;
    let diffuse = max(dot(in.world_normal, light_dir), 0.0) * 0.65;
    
    let view_dir = normalize(uniforms.camera_pos - in.world_position);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0) * 0.1;
    
    let lighting = ambient + diffuse + specular;
    let lit_color = base_color * lighting;
    
    let height_factor = clamp(in.world_position.y / 50.0, 0.0, 1.0);
    let final_color = mix(lit_color, lit_color * 1.1, height_factor * 0.2);
    
    return vec4<f32>(final_color, 1.0);
}
