// Despawn Region Compute Shader
// Marks particles for despawn based on axis-aligned bounding box test

struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    predicted_position: vec4<f32>,
    lambda: f32,
    density: f32,
    phase: u32,
    temperature: f32,
    color: vec4<f32>,
};

struct DespawnParams {
    min_bounds: vec4<f32>,
    max_bounds: vec4<f32>,
    particle_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<uniform> params: DespawnParams;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> particle_flags: array<atomic<u32>>;
@group(0) @binding(3) var<storage, read_write> despawn_counter: atomic<u32>;

@compute @workgroup_size(64)
fn despawn_region(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    // Check if particle is active
    let flag = atomicLoad(&particle_flags[id]);
    if (flag == 0u) { return; } // Already inactive
    
    let pos = particles[id].position.xyz;
    let min_b = params.min_bounds.xyz;
    let max_b = params.max_bounds.xyz;
    
    // Check if particle is within despawn region
    if (pos.x >= min_b.x && pos.x <= max_b.x &&
        pos.y >= min_b.y && pos.y <= max_b.y &&
        pos.z >= min_b.z && pos.z <= max_b.z) {
        // Mark particle as inactive
        atomicStore(&particle_flags[id], 0u);
        
        // Move particle to out-of-bounds position (will be recycled)
        particles[id].position = vec4<f32>(-1000.0, -1000.0, -1000.0, 0.0);
        particles[id].velocity = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        
        // Increment despawn counter
        atomicAdd(&despawn_counter, 1u);
    }
}
