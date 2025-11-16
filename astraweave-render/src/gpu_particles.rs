// GPU Compute-Based Particle System
// High-performance particle simulation using compute shaders

use anyhow::Result;
use wgpu;

/// GPU particle structure (32 bytes, 16-byte aligned)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    /// Position (xyz) and lifetime (w)
    pub position: [f32; 4],
    /// Velocity (xyz) and age (w)
    pub velocity: [f32; 4],
    /// Color (rgba)
    pub color: [f32; 4],
    /// Scale (xyz) and mass (w)
    pub scale: [f32; 4],
}

/// Particle emitter configuration
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EmitterParams {
    /// Emission position
    pub position: [f32; 4],
    /// Emission velocity (direction and speed)
    pub velocity: [f32; 4],
    /// Emission rate (particles per second)
    pub emission_rate: f32,
    /// Particle lifetime
    pub lifetime: f32,
    /// Velocity randomness (0-1)
    pub velocity_randomness: f32,
    /// Current delta time
    pub delta_time: f32,
    /// Gravity force
    pub gravity: [f32; 4],
    /// Particle count
    pub particle_count: u32,
    /// Maximum particles
    pub max_particles: u32,
    /// Random seed
    pub random_seed: u32,
    /// Padding
    pub _padding: u32,
}

/// GPU particle system with compute-based simulation
pub struct GpuParticleSystem {
    /// Particle buffer (double-buffered for ping-pong)
    particle_buffer_a: wgpu::Buffer,
    particle_buffer_b: wgpu::Buffer,

    /// Emitter parameters buffer
    emitter_params_buffer: wgpu::Buffer,

    /// Compute pipeline for particle update
    update_pipeline: wgpu::ComputePipeline,

    /// Compute pipeline for particle emission (reserved for future use)
    #[allow(dead_code)]
    emit_pipeline: wgpu::ComputePipeline,

    /// Bind group layout (used in new())
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,

    /// Bind groups (ping-pong)
    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,

    /// Current active buffer (false = A, true = B)
    active_buffer: bool,

    /// Maximum particle count
    max_particles: u32,

    /// Current particle count
    particle_count: u32,
}

impl GpuParticleSystem {
    pub fn new(device: &wgpu::Device, max_particles: u32) -> Result<Self> {
        // Create particle buffers (double-buffered)
        let particle_buffer_size =
            (max_particles as u64) * std::mem::size_of::<GpuParticle>() as u64;

        let particle_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer A"),
            size: particle_buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let particle_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer B"),
            size: particle_buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create emitter params buffer
        let emitter_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Emitter Params Buffer"),
            size: std::mem::size_of::<EmitterParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Load compute shaders
        let update_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Update Shader"),
            source: wgpu::ShaderSource::Wgsl(PARTICLE_UPDATE_SHADER.into()),
        });

        let emit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Emit Shader"),
            source: wgpu::ShaderSource::Wgsl(PARTICLE_EMIT_SHADER.into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Compute BG Layout"),
            entries: &[
                // Input particles
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Output particles
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Emitter params
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create bind groups (ping-pong)
        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute BG A"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: emitter_params_buffer.as_entire_binding(),
                },
            ],
        });

        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Compute BG B"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: emitter_params_buffer.as_entire_binding(),
                },
            ],
        });

        // Create compute pipelines
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let update_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Update Pipeline"),
            layout: Some(&pipeline_layout),
            module: &update_shader,
            entry_point: Some("update_particles"),
            compilation_options: Default::default(),
            cache: None,
        });

        let emit_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Emit Pipeline"),
            layout: Some(&pipeline_layout),
            module: &emit_shader,
            entry_point: Some("emit_particles"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            particle_buffer_a,
            particle_buffer_b,
            emitter_params_buffer,
            update_pipeline,
            emit_pipeline,
            bind_group_layout,
            bind_group_a,
            bind_group_b,
            active_buffer: false,
            max_particles,
            particle_count: 0,
        })
    }

    /// Update particle system
    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        params: &EmitterParams,
    ) {
        // Upload emitter params
        queue.write_buffer(&self.emitter_params_buffer, 0, bytemuck::bytes_of(params));

        // Dispatch compute shader
        let bind_group = if self.active_buffer {
            &self.bind_group_b
        } else {
            &self.bind_group_a
        };

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Particle Update Pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.update_pipeline);
        pass.set_bind_group(0, bind_group, &[]);

        // Dispatch with 64 threads per workgroup
        let workgroups = (self.max_particles + 63) / 64;
        pass.dispatch_workgroups(workgroups, 1, 1);

        drop(pass);

        // Swap buffers
        self.active_buffer = !self.active_buffer;
        self.particle_count = params.particle_count.min(self.max_particles);
    }

    /// Get the current particle buffer for rendering
    pub fn particle_buffer(&self) -> &wgpu::Buffer {
        if self.active_buffer {
            &self.particle_buffer_b
        } else {
            &self.particle_buffer_a
        }
    }

    /// Get particle count
    pub fn particle_count(&self) -> u32 {
        self.particle_count
    }
}

// Particle update compute shader
const PARTICLE_UPDATE_SHADER: &str = r#"
struct Particle {
    position: vec4<f32>,  // xyz = position, w = lifetime
    velocity: vec4<f32>,  // xyz = velocity, w = age
    color: vec4<f32>,     // rgba
    scale: vec4<f32>,     // xyz = scale, w = mass
}

struct EmitterParams {
    position: vec4<f32>,
    velocity: vec4<f32>,
    emission_rate: f32,
    lifetime: f32,
    velocity_randomness: f32,
    delta_time: f32,
    gravity: vec4<f32>,
    particle_count: u32,
    max_particles: u32,
    random_seed: u32,
    _padding: u32,
}

@group(0) @binding(0) var<storage, read> particles_in: array<Particle>;
@group(0) @binding(1) var<storage, read_write> particles_out: array<Particle>;
@group(0) @binding(2) var<uniform> params: EmitterParams;

@compute @workgroup_size(64)
fn update_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    if (idx >= params.max_particles) {
        return;
    }
    
    var particle = particles_in[idx];
    
    // Update age
    particle.velocity.w += params.delta_time;
    
    // Check if particle is alive
    if (particle.velocity.w >= particle.position.w) {
        // Particle is dead, reset it
        particle.velocity.w = 0.0;
        particle.position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        particles_out[idx] = particle;
        return;
    }
    
    // Apply gravity
    let gravity_force = params.gravity.xyz * particle.scale.w * params.delta_time;
    particle.velocity = vec4<f32>(particle.velocity.xyz + gravity_force, particle.velocity.w);
    
    // Update position
    particle.position = vec4<f32>(
        particle.position.xyz + particle.velocity.xyz * params.delta_time,
        particle.position.w
    );
    
    // Write back
    particles_out[idx] = particle;
}
"#;

// Particle emission compute shader
const PARTICLE_EMIT_SHADER: &str = r#"
struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    color: vec4<f32>,
    scale: vec4<f32>,
}

struct EmitterParams {
    position: vec4<f32>,
    velocity: vec4<f32>,
    emission_rate: f32,
    lifetime: f32,
    velocity_randomness: f32,
    delta_time: f32,
    gravity: vec4<f32>,
    particle_count: u32,
    max_particles: u32,
    random_seed: u32,
    _padding: u32,
}

@group(0) @binding(0) var<storage, read> particles_in: array<Particle>;
@group(0) @binding(1) var<storage, read_write> particles_out: array<Particle>;
@group(0) @binding(2) var<uniform> params: EmitterParams;

// Simple pseudo-random number generator
fn rand(seed: u32) -> f32 {
    var state = seed;
    state = state ^ (state << 13u);
    state = state ^ (state >> 17u);
    state = state ^ (state << 5u);
    return f32(state) / 4294967296.0;
}

@compute @workgroup_size(64)
fn emit_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    if (idx >= params.max_particles) {
        return;
    }
    
    // Simple emission logic (can be extended)
    particles_out[idx] = particles_in[idx];
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_particle_size() {
        assert_eq!(std::mem::size_of::<GpuParticle>(), 64);
    }

    #[test]
    fn test_emitter_params_size() {
        assert_eq!(std::mem::size_of::<EmitterParams>(), 80);
    }
}
