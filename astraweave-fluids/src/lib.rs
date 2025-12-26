pub mod renderer;
pub mod sdf;

pub use renderer::FluidRenderer;

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
    pub _pad: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DynamicObject {
    pub transform: [[f32; 4]; 4],
    pub inv_transform: [[f32; 4]; 4],
    pub half_extents: [f32; 4], // w = type (0=box, 1=sphere)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimParams {
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub viscosity: f32,
    pub surface_tension: f32,
    pub gravity: f32,
    pub dt: f32,
    pub particle_count: u32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_depth: u32,
    pub cell_size: f32,
    pub object_count: u32,
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
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
    mix_dye_pipeline: wgpu::ComputePipeline,
    emit_whitewater_pipeline: wgpu::ComputePipeline,
    update_whitewater_pipeline: wgpu::ComputePipeline,

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

    pub sdf_system: crate::sdf::SdfSystem,
    pub objects_buffer: wgpu::Buffer,
    pub objects_bind_group: wgpu::BindGroup,
    pub default_sampler: wgpu::Sampler,
    secondary_particle_buffer: wgpu::Buffer,
    secondary_counter: wgpu::Buffer,
    density_error_buffer: wgpu::Buffer,
    density_error_staging_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SecondaryParticle {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub info: [f32; 4], // x: lifetime, y: type, z: alpha, w: scale
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
                _pad: [0.0; 2],
                color: [0.2, 0.5, 0.8, 1.0],
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
            dt: 0.016,
            smoothing_radius: 1.0,
            target_density: 10.0,
            pressure_multiplier: 250.0,
            viscosity: 50.0,
            surface_tension: 0.1,
            particle_count,
            gravity: -9.8,
            cell_size,
            grid_width,
            grid_height,
            grid_depth,
            object_count: 0, // Will be updated by update_objects
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create secondary particle buffer (65536 * 48 bytes)
        let secondary_particle_count = 65536;
        let secondary_particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Secondary Particle Buffer"),
            size: (secondary_particle_count * std::mem::size_of::<SecondaryParticle>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let secondary_counter = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Secondary Counter"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let density_error_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Density Error Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let density_error_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Density Error Staging Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: secondary_particle_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: secondary_counter.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: density_error_buffer.as_entire_binding(),
                    },
                ],
            });
            bind_groups.push(bg);
        }

        // Create objects buffer (max 128 dynamic objects)
        let objects_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic Objects Buffer"),
            size: (128 * std::mem::size_of::<DynamicObject>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let objects_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Dynamic Objects Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let objects_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Dynamic Objects Bind Group"),
            layout: &objects_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: objects_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &objects_bind_group_layout],
            push_constant_ranges: &[],
        });

        let (
            clear_grid_pipeline,
            build_grid_pipeline,
            predict_pipeline,
            lambda_pipeline,
            delta_pos_pipeline,
            integrate_pipeline,
            mix_dye_pipeline,
            emit_whitewater_pipeline,
            update_whitewater_pipeline,
        ) = {
            let create_p = |label, entry_point| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
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
                create_p("Mix Dye", "mix_dye"),
                create_p("Emit Whitewater", "emit_whitewater"),
                create_p("Update Whitewater", "update_whitewater"),
            )
        };

        let sdf_system = crate::sdf::SdfSystem::new(device, &objects_buffer, 64, 60.0);

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
            target_density: 12.0,
            pressure_multiplier: 300.0,
            viscosity: 10.0,
            surface_tension: 0.02,
            gravity: -9.8,
            iterations: 4,
            cell_size,
            grid_width,
            grid_height,
            grid_depth,
            sdf_system,
            objects_buffer,
            objects_bind_group,
            default_sampler,
            secondary_particle_buffer,
            secondary_counter,
            density_error_buffer,
            density_error_staging_buffer,
            mix_dye_pipeline,
            emit_whitewater_pipeline,
            update_whitewater_pipeline,
        }
    }

    pub fn update_objects(&mut self, queue: &wgpu::Queue, objects: &[DynamicObject]) {
        if !objects.is_empty() {
            queue.write_buffer(&self.objects_buffer, 0, bytemuck::cast_slice(objects));
        }
    }

    pub fn reset_particles(&mut self, queue: &wgpu::Queue, particles: &[Particle]) {
        assert_eq!(particles.len() as u32, self.particle_count);
        for buf in &self.particle_buffers {
            queue.write_buffer(buf, 0, bytemuck::cast_slice(particles));
        }
    }

    pub fn step(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        dt: f32,
    ) {
        // Update Uniforms
        let params = SimParams {
            smoothing_radius: self.smoothing_radius,
            target_density: self.target_density,
            pressure_multiplier: self.pressure_multiplier,
            viscosity: self.viscosity,
            surface_tension: self.surface_tension,
            gravity: self.gravity,
            dt: dt.min(0.016),
            particle_count: self.particle_count,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
            grid_depth: self.grid_depth,
            cell_size: self.cell_size,
            object_count: 0, // Placeholder, can be set by update_objects
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        // 1. Generate SDF
        self.sdf_system.generate(encoder, queue);

        let particle_workgroups = (self.particle_count + 63) / 64;
        let current_src = (self.frame_index % 2) as usize;
        let bg = &self.bind_groups[current_src];
        let obj_bg = &self.objects_bind_group;

        // Create a temporary bind group for the SDF texture and sampler
        // This should really be part of the FluidSystem's bind group layout for performance,
        // but for now we'll bind it to group 2.
        let sdf_view = self
            .sdf_system
            .texture_a
            .create_view(&wgpu::TextureViewDescriptor::default());

        // We need a bind group layout for the SDF
        let sdf_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid SDF Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let sdf_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fluid SDF BG"),
            layout: &sdf_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sdf_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.default_sampler),
                },
            ],
        });

        // 0. Reset density error and counters
        encoder.clear_buffer(&self.density_error_buffer, 0, None);
        encoder.clear_buffer(&self.secondary_counter, 0, None);

        // 1. Predict and Clear Grid
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::Predict"),
                ..Default::default()
            });
            cpass.set_bind_group(0, bg, &[]);
            cpass.set_bind_group(1, obj_bg, &[]);
            cpass.set_pipeline(&self.predict_pipeline);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::ClearGrid"),
                ..Default::default()
            });
            cpass.set_bind_group(0, bg, &[]);
            cpass.set_pipeline(&self.clear_grid_pipeline);
            cpass.dispatch_workgroups(
                (self.grid_width * self.grid_height * self.grid_depth + 63) / 64,
                1,
                1,
            );
        }

        // 3. Build Grid
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::BuildGrid"),
                ..Default::default()
            });
            cpass.set_bind_group(0, bg, &[]);
            cpass.set_pipeline(&self.build_grid_pipeline);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        // 3. PBD Iterations
        for _ in 0..self.iterations {
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Fluid::Lambda"),
                    ..Default::default()
                });
                cpass.set_bind_group(0, bg, &[]);
                cpass.set_bind_group(2, &sdf_bg, &[]); // Global SDF
                cpass.set_pipeline(&self.lambda_pipeline);
                cpass.dispatch_workgroups(particle_workgroups, 1, 1);
            }
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Fluid::DeltaPos"),
                    ..Default::default()
                });
                cpass.set_bind_group(0, bg, &[]);
                cpass.set_bind_group(1, obj_bg, &[]);
                cpass.set_bind_group(2, &sdf_bg, &[]); // Global SDF
                cpass.set_pipeline(&self.delta_pos_pipeline);
                cpass.dispatch_workgroups(particle_workgroups, 1, 1);
            }
        }

        // 4. Integrate
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::Integrate"),
                ..Default::default()
            });
            cpass.set_bind_group(0, bg, &[]);
            cpass.set_bind_group(2, &sdf_bg, &[]); // Global SDF
            cpass.set_pipeline(&self.integrate_pipeline);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        // 5. Dye Mixing & Whitewater
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::Dye&Whitewater"),
                ..Default::default()
            });
            cpass.set_bind_group(0, bg, &[]);

            // Dye mixing
            cpass.set_pipeline(&self.mix_dye_pipeline);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);

            // Whitewater emission
            cpass.set_pipeline(&self.emit_whitewater_pipeline);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);

            // Whitewater update (max 65536 particles)
            cpass.set_pipeline(&self.update_whitewater_pipeline);
            cpass.dispatch_workgroups(1024, 1, 1); // 1024 * 64 = 65536
        }
        // 6. Copy error to staging for adaptive iterations next frame
        encoder.copy_buffer_to_buffer(
            &self.density_error_buffer,
            0,
            &self.density_error_staging_buffer,
            0,
            4,
        );

        self.frame_index += 1;

        // --- Adaptive Iteration Adjust ---
        // For production, we'd use a non-blocking read.
        // For the demo, we'll do a simple poll to avoid stalling too much if possible.
        // In a real game engine, we'd read this 1-2 frames late.
        let buffer_slice = self.density_error_staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait); // Stalls slightly, but okay for demo verification

        if let Ok(data) = buffer_slice.get_mapped_range() {
            let error_scaled = u32::from_ne_bytes(data[0..4].try_into().unwrap());
            let avg_error = (error_scaled as f32 / 1000.0) / self.particle_count as f32;

            // Adjust iterations based on error
            if avg_error > 0.05 {
                self.iterations = (self.iterations + 1).min(8);
            } else if avg_error < 0.01 {
                self.iterations = (self.iterations.saturating_sub(1)).max(2);
            }
        }
        self.density_error_staging_buffer.unmap();
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

    pub fn secondary_particle_buffer(&self) -> &wgpu::Buffer {
        &self.secondary_particle_buffer
    }

    pub fn secondary_particle_count(&self) -> u32 {
        65536
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
            _pad: [0.0; 2],
            color: [0.0; 4],
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
            _pad: [0.0; 2],
            color: [1.0, 0.0, 0.0, 1.0],
        };
        assert_eq!(particle.velocity[1], -1.0);
        assert_eq!(particle.lambda, 1.0);
        assert_eq!(particle.density, 1.0);
    }

    #[test]
    fn test_particle_size() {
        // Ensure Particle is exactly the size we expect for GPU alignment
        // 3 * vec4 (pos, vel, pred) + 1 * vec4 (lambda, density, pad) + 1 * vec4 (color)
        // = 80 bytes
        assert_eq!(std::mem::size_of::<Particle>(), 80);
    }

    #[test]
    fn test_sim_params_default() {
        let _params = SimParams {
            smoothing_radius: 1.0,
            target_density: 1.0,
            pressure_multiplier: 1.0,
            viscosity: 1.0,
            surface_tension: 1.0,
            gravity: -9.81,
            dt: 0.016,
            particle_count: 100,
            grid_width: 10,
            grid_height: 10,
            grid_depth: 10,
            cell_size: 1.0,
            object_count: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
    }

    #[test]
    fn test_sim_params_size() {
        // SimParams should be 64 bytes (16 * 4 bytes)
        assert_eq!(std::mem::size_of::<SimParams>(), 64);
    }

    #[test]
    fn test_sim_params_grid_configuration() {
        let params = SimParams {
            smoothing_radius: 1.0,
            target_density: 1.0,
            pressure_multiplier: 1.0,
            viscosity: 1.0,
            surface_tension: 1.0,
            gravity: -9.81,
            dt: 0.016,
            particle_count: 100,
            grid_width: 64,
            grid_height: 32,
            grid_depth: 16,
            cell_size: 1.0,
            object_count: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        assert_eq!(params.grid_width, 64);
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
                _pad: [0.0; 2],
                color: [0.0; 4],
            },
            Particle {
                position: [4.0, 5.0, 6.0, 1.0],
                velocity: [1.0, 1.0, 1.0, 0.0],
                predicted_position: [5.0, 6.0, 7.0, 1.0],
                lambda: 2.0,
                density: 2.0,
                _pad: [0.0; 2],
                color: [1.0; 4],
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
            smoothing_radius: 1.0,
            target_density: 10.0,
            pressure_multiplier: 250.0,
            viscosity: 50.0,
            surface_tension: 0.1,
            gravity: -9.8,
            dt: 0.016,
            particle_count: 1000,
            grid_width: 128,
            grid_height: 128,
            grid_depth: 128,
            cell_size: 1.2,
            object_count: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
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
