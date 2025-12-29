// GPU Frustum Culling Compute Shader
// Filters visible particles for rendering optimization

struct CameraPlanes {
    planes: array<vec4<f32>, 6>,  // xyz = normal, w = distance
    particle_count: u32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    predicted_position: vec4<f32>,
    lambda: f32,
    density: f32,
    _pad: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraPlanes;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> visible_indices: array<u32>;
@group(0) @binding(3) var<storage, read_write> visible_count: atomic<u32>;

fn test_plane(plane: vec4<f32>, pos: vec3<f32>, radius: f32) -> bool {
    return dot(plane.xyz, pos) + plane.w >= -radius;
}

fn is_in_frustum(pos: vec3<f32>, radius: f32) -> bool {
    for (var i = 0u; i < 6u; i++) {
        if (!test_plane(camera.planes[i], pos, radius)) {
            return false;
        }
    }
    return true;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= camera.particle_count) {
        return;
    }

    let p = particles[id];
    let pos = p.position.xyz;
    
    // Test against frustum with particle radius for conservative culling
    let particle_radius = 0.5;
    
    if (is_in_frustum(pos, particle_radius)) {
        let slot = atomicAdd(&visible_count, 1u);
        if (slot < arrayLength(&visible_indices)) {
            visible_indices[slot] = id;
        }
    }
}
