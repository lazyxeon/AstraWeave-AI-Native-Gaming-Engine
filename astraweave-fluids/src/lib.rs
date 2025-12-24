mod renderer;
mod sdf;

use std::borrow::Cow;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub predicted_position: [f32; 4],
    pub lambda: f32,
    pub density: f32,
    pub padding1: f32,
    pub padding2: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimParams {
    pub delta_time: f32,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32, // Reused as PBD compliance/constraint strength
    pub viscosity: f32,
    pub surface_tension: f32,
    pub particle_count: u32,
    pub gravity: f32,
    pub iterations: u32,
    pub cell_size: f32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_depth: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

pub struct FluidSystem {
    particle_buffers: Vec<wgpu::Buffer>,
    // We need bind groups that swap Src/Dst
    // layout: 0: Params, 1: Src, 2: Dst, 3: head_pointers, 4: next_pointers
    // bg0: Src=Buf0, Dst=Buf1
    // bg1: Src=Buf1, Dst=Buf0
    bind_groups: Vec<wgpu::BindGroup>,

    #[allow(dead_code)] // Reserved for SPH neighbor search grid
    head_pointers: wgpu::Buffer,
    #[allow(dead_code)] // Reserved for SPH neighbor search grid
    next_pointers: wgpu::Buffer,

    clear_grid_pipeline: wgpu::ComputePipeline,
    build_grid_pipeline: wgpu::ComputePipeline,
    predict_pipeline: wgpu::ComputePipeline,
    lambda_pipeline: wgpu::ComputePipeline,
    delta_pos_pipeline: wgpu::ComputePipeline,
    integrate_pipeline: wgpu::ComputePipeline,

    params_buffer: wgpu::Buffer,
    pub particle_count: u32,
    pub frame_index: usize,

    // Sim constants
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub viscosity: f32,
    pub surface_tension: f32,
    pub gravity: f32,
    pub iterations: u32,

    // Grid params
    pub cell_size: f32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_depth: u32,

    pub sdf_view: Option<Arc<wgpu::TextureView>>,
    pub default_sampler: wgpu::Sampler,
}

impl FluidSystem {
    pub fn new(device: &wgpu::Device, particle_count: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fluid Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/fluid.wgsl"))),
        });

        // Create Buffers
        let mut initial_particles = Vec::with_capacity(particle_count as usize);
        let spacing = 0.5;
        let size = (particle_count as f32).powf(1.0 / 3.0).ceil() as usize;

        for i in 0..particle_count as usize {
            let x = (i % size) as f32 * spacing - 5.0;
            let y = ((i / size) % size) as f32 * spacing + 2.0;
            let z = (i / (size * size)) as f32 * spacing - 5.0;

            initial_particles.push(Particle {
                position: [x, y, z, 1.0],
                velocity: [0.0; 4],
                predicted_position: [x, y, z, 1.0],
                lambda: 0.0,
                density: 0.0,
                padding1: 0.0,
                padding2: 0.0,
            });
        }

        let buffer_size = (particle_count as usize * std::mem::size_of::<Particle>()) as u64;

        let buf0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Buffer 0"),
            contents: bytemuck::cast_slice(&initial_particles),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let buf1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer 1"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let particle_buffers = vec![buf0, buf1];

        // Grid parameters
        let grid_width = 128u32;
        let grid_height = 128u32;
        let grid_depth = 128u32;
        let cell_size = 1.2; // Slightly larger than smoothing_radius
        let grid_size = (grid_width * grid_height * grid_depth) as usize;

        // Create grid buffers
        let head_pointers = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Head Pointers Buffer"),
            size: (grid_size * std::mem::size_of::<i32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let next_pointers = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Next Pointers Buffer"),
            size: (particle_count as usize * std::mem::size_of::<i32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let params = SimParams {
            delta_time: 0.016,
            smoothing_radius: 1.0,
            target_density: 10.0,
            pressure_multiplier: 250.0,
            viscosity: 50.0,
            surface_tension: 0.1,
            particle_count,
            gravity: -9.8,
            iterations: 4,
            cell_size,
            grid_width,
            grid_height,
            grid_depth,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sim Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Initialize Default Sampler
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create a dummy 1x1x1 SDF texture to avoid empty bindings
        let dummy_sdf = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dummy SDF"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_sdf_view = dummy_sdf.create_view(&Default::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create 2 Bind Groups (Ping-Pong)
        let mut bind_groups = Vec::new();
        for i in 0..2 {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Fluid BG {}", i)),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: particle_buffers[1 - i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: head_pointers.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: next_pointers.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(&dummy_sdf_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Sampler(&default_sampler),
                    },
                ],
            });
            bind_groups.push(bg);
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let (
            clear_grid_pipeline,
            build_grid_pipeline,
            predict_pipeline,
            lambda_pipeline,
            delta_pos_pipeline,
            integrate_pipeline,
        ) = {
            let create_p = |label, entry| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
            };
            (
                create_p("Clear Grid", "clear_grid"),
                create_p("Build Grid", "build_grid"),
                create_p("Predict", "predict"),
                create_p("Lambda", "compute_lambda"),
                create_p("Delta Pos", "compute_delta_pos"),
                create_p("Integrate", "integrate"),
            )
        };

        Self {
            particle_buffers,
            bind_groups,
            head_pointers,
            next_pointers,
            clear_grid_pipeline,
            build_grid_pipeline,
            predict_pipeline,
            lambda_pipeline,
            delta_pos_pipeline,
            integrate_pipeline,
            params_buffer,
            particle_count,
            frame_index: 0,
            smoothing_radius: 1.0,
            target_density: 10.0,
            pressure_multiplier: 250.0,
            viscosity: 50.0,
            surface_tension: 0.1,
            gravity: -9.8,
            iterations: 4,
            cell_size,
            grid_width,
            grid_height,
            grid_depth,
            sdf_view: None,
            default_sampler,
        }
    }

    pub fn step(&mut self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, dt: f32) {
        // Update Uniforms
        let params = SimParams {
            delta_time: dt.min(0.016), // Cap dt for PBD stability
            smoothing_radius: self.smoothing_radius,
            target_density: self.target_density,
            pressure_multiplier: self.pressure_multiplier,
            viscosity: self.viscosity,
            surface_tension: self.surface_tension,
            particle_count: self.particle_count,
            gravity: self.gravity,
            iterations: self.iterations,
            cell_size: self.cell_size,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
            grid_depth: self.grid_depth,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let current_src = self.frame_index % 2;
        let grid_size = self.grid_width * self.grid_height * self.grid_depth;
        let grid_workgroups = (grid_size + 63) / 64;
        let particle_workgroups = (self.particle_count + 63) / 64;

        let bg = &self.bind_groups[current_src];

        // Step 1: Predict (Euler)
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("PBD Predict"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.predict_pipeline);
            cpass.set_bind_group(0, bg, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        // Step 2: Spatial Hash
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("PBD Clear Grid"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.clear_grid_pipeline);
            cpass.set_bind_group(0, bg, &[]);
            cpass.dispatch_workgroups(grid_workgroups, 1, 1);
        }
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("PBD Build Grid"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.build_grid_pipeline);
            cpass.set_bind_group(0, bg, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        // Step 3: PBD Iterations
        for _ in 0..self.iterations {
            // Lambda
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("PBD Compute Lambda"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.lambda_pipeline);
                cpass.set_bind_group(0, bg, &[]);
                cpass.dispatch_workgroups(particle_workgroups, 1, 1);
            }
            // Delta Pos
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("PBD Compute Delta Pos"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.delta_pos_pipeline);
                cpass.set_bind_group(0, bg, &[]);
                cpass.dispatch_workgroups(particle_workgroups, 1, 1);
            }
        }

        // Step 4: Integrate & Finalize
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("PBD Integrate"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.integrate_pipeline);
            cpass.set_bind_group(0, bg, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        self.frame_index += 1;
    }

    pub fn get_particle_buffer(&self) -> &wgpu::Buffer {
        // The result is always in the "Dst" of the last pass (Integrate).
        // Integrate used `bg_density` where Dst = `particle_buffers[1 - current_src]`.
        // Since we incremented frame_index at end, we need to look back.
        // Frame 0 (start 0): Integ writes to 1. Incr to 1.
        // Frame 1 (start 1): Integ writes to 0. Incr to 2.
        // So if frame_index is Odd, result is in 1.
        // If frame_index is Even, result is in 0.
        &self.particle_buffers[self.frame_index % 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_default_values() {
        let particle = Particle {
            position: [0.0, 0.0, 0.0, 1.0],
            velocity: [0.0, 0.0, 0.0, 0.0],
            predicted_position: [0.0, 0.0, 0.0, 1.0],
            lambda: 0.0,
            density: 0.0,
            padding1: 0.0,
            padding2: 0.0,
        };
        assert_eq!(particle.position[3], 1.0);
        assert_eq!(particle.density, 0.0);
    }

    #[test]
    fn test_particle_with_velocity() {
        let particle = Particle {
            position: [1.0, 2.0, 3.0, 1.0],
            velocity: [0.5, -1.0, 0.0, 0.0],
            predicted_position: [1.0, 1.0, 3.0, 1.0],
            lambda: 1.0,
            density: 1.0,
            padding1: 0.0,
            padding2: 0.0,
        };
        assert_eq!(particle.velocity[1], -1.0);
        assert_eq!(particle.lambda, 1.0);
        assert_eq!(particle.density, 1.0);
    }

    #[test]
    fn test_particle_size() {
        // Ensure Particle is exactly the size we expect for GPU alignment
        // 4 f32 (position) + 4 f32 (velocity) + 4 f32 (force) + 4 f32 (density, pressure, padding)
        // = 16 * 4 = 64 bytes
        assert_eq!(std::mem::size_of::<Particle>(), 64);
    }

    #[test]
    fn test_sim_params_default() {
        let params = SimParams {
            delta_time: 0.016,
            smoothing_radius: 1.0,
            target_density: 10.0,
            pressure_multiplier: 250.0,
            viscosity: 50.0,
            surface_tension: 0.1,
            particle_count: 1000,
            gravity: -9.8,
            iterations: 4,
            cell_size: 1.2,
            grid_width: 128,
            grid_height: 128,
            grid_depth: 128,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };
        assert_eq!(params.delta_time, 0.016);
        assert_eq!(params.particle_count, 1000);
        assert_eq!(params.gravity, -9.8);
    }

    #[test]
    fn test_sim_params_size() {
        // SimParams should be 64 bytes (16 * 4 bytes)
        assert_eq!(std::mem::size_of::<SimParams>(), 64);
    }

    #[test]
    fn test_sim_params_grid_configuration() {
        let params = SimParams {
            delta_time: 0.016,
            smoothing_radius: 0.5,
            target_density: 1.0,
            pressure_multiplier: 100.0,
            viscosity: 10.0,
            surface_tension: 0.1,
            particle_count: 500,
            gravity: -10.0,
            iterations: 4,
            cell_size: 0.6, // Should be slightly larger than smoothing_radius
            grid_width: 64,
            grid_height: 64,
            grid_depth: 64,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };
        assert!(params.cell_size > params.smoothing_radius);
        assert_eq!(
            params.grid_width * params.grid_height * params.grid_depth,
            262144
        );
    }

    #[test]
    fn test_particle_bytemuck_cast() {
        let particles = vec![
            Particle {
                position: [1.0, 2.0, 3.0, 1.0],
                velocity: [0.0, 0.0, 0.0, 0.0],
                predicted_position: [1.0, 2.0, 3.0, 1.0],
                lambda: 0.0,
                density: 1.0,
                padding1: 0.0,
                padding2: 0.0,
            },
            Particle {
                position: [4.0, 5.0, 6.0, 1.0],
                velocity: [1.0, 1.0, 1.0, 0.0],
                predicted_position: [5.0, 6.0, 7.0, 1.0],
                lambda: 2.0,
                density: 2.0,
                padding1: 0.0,
                padding2: 0.0,
            },
        ];

        // Test that bytemuck::cast_slice works
        let bytes: &[u8] = bytemuck::cast_slice(&particles);
        assert_eq!(bytes.len(), 2 * std::mem::size_of::<Particle>());

        // Cast back
        let recovered: &[Particle] = bytemuck::cast_slice(bytes);
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].position[0], 1.0);
        assert_eq!(recovered[1].position[0], 4.0);
    }

    #[test]
    fn test_sim_params_bytemuck_cast() {
        let params = SimParams {
            delta_time: 0.016,
            smoothing_radius: 1.0,
            target_density: 10.0,
            pressure_multiplier: 250.0,
            viscosity: 50.0,
            surface_tension: 0.1,
            particle_count: 1000,
            gravity: -9.8,
            iterations: 4,
            cell_size: 1.2,
            grid_width: 128,
            grid_height: 128,
            grid_depth: 128,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };

        let bytes: &[u8] = bytemuck::bytes_of(&params);
        assert_eq!(bytes.len(), std::mem::size_of::<SimParams>());

        let recovered: &SimParams = bytemuck::from_bytes(bytes);
        assert_eq!(recovered.particle_count, 1000);
        assert_eq!(recovered.grid_width, 128);
    }

    #[test]
    fn test_particle_initial_spacing() {
        // Test the spacing logic used in FluidSystem::new
        let particle_count = 1000u32;
        let spacing = 0.5;
        let size = (particle_count as f32).powf(1.0 / 3.0).ceil() as usize;

        // Size should be 10 or 11 depending on floating point (cube root of 1000 ≈ 10.0)
        // The actual value is 10.0 exactly, but floating point may give 10.000001, so ceil gives 11
        assert!(size == 10 || size == 11, "Expected 10 or 11, got {}", size);

        // Check positions for first particle
        let i = 0;
        let x = (i % size) as f32 * spacing - 5.0;
        let y = ((i / size) % size) as f32 * spacing + 2.0;
        let z = (i / (size * size)) as f32 * spacing - 5.0;
        assert_eq!(x, -5.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, -5.0);
    }
}
