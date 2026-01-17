pub mod editor;
pub mod emitter;
pub mod lod;
pub mod profiling;
pub mod renderer;
pub mod sdf;
pub mod serialization;
pub mod terrain_integration;

pub use editor::FluidEditorConfig;
pub use emitter::{EmitterShape, FluidDrain, FluidEmitter};
pub use lod::{FluidLodConfig, FluidLodManager, LodLevel};
pub use profiling::{FluidProfiler, FluidTimingStats};
pub use renderer::FluidRenderer;
pub use serialization::{FluidSnapshot, SnapshotParams};
pub use terrain_integration::{
    DetectedWaterBody, LakeConfig, OceanConfig, RiverConfig, TerrainFluidConfig,
    WaterBodyType, WaterfallConfig, analyze_terrain_for_water,
};

use std::borrow::Cow;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub predicted_position: [f32; 4],
    pub lambda: f32,
    pub density: f32,
    pub phase: u32,       // 0=water, 1=oil, 2=custom phase
    pub temperature: f32, // Kelvin (ambient ~293K)
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

    // Optimized Bind Groups
    global_bind_group: wgpu::BindGroup,
    particles_bind_groups: [wgpu::BindGroup; 2], // Ping-pong
    secondary_bind_group: wgpu::BindGroup,
    // Group 3 (Scene) is handled per-frame since SDF view can change

    // Optimized Bind Group Layouts
    #[allow(dead_code)]
    global_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    particles_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    secondary_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    scene_layout: wgpu::BindGroupLayout,

    #[allow(dead_code)]
    head_pointers: wgpu::Buffer,
    #[allow(dead_code)]
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
    pub default_sampler: wgpu::Sampler,
    secondary_particle_buffer: wgpu::Buffer,
    secondary_counter: wgpu::Buffer,
    density_error_buffer: wgpu::Buffer,
    density_error_staging_buffers: [wgpu::Buffer; 2],
    staging_mapped: [bool; 2],

    // Dynamic Particle Management
    particle_flags: wgpu::Buffer, // 0=inactive, 1=active for each particle
    pub active_count: u32,        // Currently active particles
    pub max_particles: u32,       // Buffer capacity
    free_list: Vec<u32>,          // CPU-side free list for respawning
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
                phase: 0,
                temperature: 293.0,
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
        let _grid_size = (grid_width * grid_height * grid_depth) as usize;

        let params = SimParams {
            dt: 0.016,
            smoothing_radius: 1.0,
            target_density: 12.0,
            pressure_multiplier: 300.0,
            viscosity: 10.0,
            surface_tension: 0.02,
            particle_count,
            gravity: -9.8,
            cell_size,
            grid_width,
            grid_height,
            grid_depth,
            object_count: 0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sim Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // --- Bind Group Layouts ---

        // Group 0: Global Infrastructure (Params, Counters, Pointers)
        let global_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Global Layout"),
            entries: &[
                // 0: SimParams
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
                // 1: Head Pointers
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
                // 2: Next Pointers
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
                // 3: Secondary Counter
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
                // 4: Density Error
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
            ],
        });

        // Group 1: Particles (Ping-Pong)
        let particles_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Particles Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
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
            ],
        });

        // Group 2: Secondary Particles
        let secondary_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Secondary Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Group 3: Scene Data (Objects, SDF)
        let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fluid Scene Layout"),
            entries: &[
                // 0: Objects
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
                // 1: SDF Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // 2: Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Fluid Default Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // --- Infrastructure Buffers ---
        let head_pointers = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Head Pointers"),
            size: (grid_width * grid_height * grid_depth * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let next_pointers = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Next Pointers"),
            size: (particle_count * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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

        let density_error_staging_buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Density Error Staging Buffer 0"),
                size: 4,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Density Error Staging Buffer 1"),
                size: 4,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];

        // Dynamic Particle Management: flags buffer (1=active, 0=inactive)
        let initial_flags: Vec<u32> = vec![1u32; particle_count as usize];
        let particle_flags = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Flags Buffer"),
            contents: bytemuck::cast_slice(&initial_flags),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let objects_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic Objects Buffer"),
            size: (128 * std::mem::size_of::<DynamicObject>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // --- Pre-allocate Bind Groups ---
        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fluid Global BG"),
            layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: head_pointers.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: next_pointers.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: secondary_counter.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: density_error_buffer.as_entire_binding(),
                },
            ],
        });

        let particles_bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Particles BG 0"),
                layout: &particles_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[0].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[1].as_entire_binding(),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Particles BG 1"),
                layout: &particles_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[1].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[0].as_entire_binding(),
                    },
                ],
            }),
        ];

        let secondary_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Secondary BG"),
            layout: &secondary_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: secondary_particle_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Pipeline Layout"),
            bind_group_layouts: &[
                &global_layout,
                &particles_layout,
                &secondary_layout,
                &scene_layout,
            ],
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
            global_bind_group,
            particles_bind_groups,
            secondary_bind_group,
            global_layout,
            particles_layout,
            secondary_layout,
            scene_layout,
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
            default_sampler,
            secondary_particle_buffer,
            secondary_counter,
            density_error_buffer,
            density_error_staging_buffers,
            staging_mapped: [false; 2],
            mix_dye_pipeline,
            emit_whitewater_pipeline,
            update_whitewater_pipeline,
            particle_flags,
            active_count: particle_count,
            max_particles: particle_count,
            free_list: Vec::new(),
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
        // Reset all flags to active
        let flags: Vec<u32> = vec![1u32; particles.len()];
        queue.write_buffer(&self.particle_flags, 0, bytemuck::cast_slice(&flags));
        self.active_count = particles.len() as u32;
        self.free_list.clear();
    }

    /// Spawn new particles at runtime. Returns the number of particles actually spawned.
    /// Particles are spawned from the free list if available, or fails if at capacity.
    pub fn spawn_particles(
        &mut self,
        queue: &wgpu::Queue,
        positions: &[[f32; 3]],
        velocities: &[[f32; 3]],
        colors: Option<&[[f32; 4]]>,
    ) -> usize {
        let count = positions.len().min(velocities.len());
        let spawned = count.min(self.free_list.len());

        for i in 0..spawned {
            let idx = self.free_list.pop().unwrap() as usize;
            let pos = positions[i];
            let vel = velocities[i];
            let color = colors.map(|c| c[i]).unwrap_or([0.2, 0.5, 0.8, 1.0]);

            let particle = Particle {
                position: [pos[0], pos[1], pos[2], 1.0],
                velocity: [vel[0], vel[1], vel[2], 0.0],
                predicted_position: [pos[0], pos[1], pos[2], 1.0],
                lambda: 0.0,
                density: 0.0,
                phase: 0,
                temperature: 293.0,
                color,
            };

            // Write to both ping-pong buffers
            let offset = (idx * std::mem::size_of::<Particle>()) as u64;
            for buf in &self.particle_buffers {
                queue.write_buffer(buf, offset, bytemuck::bytes_of(&particle));
            }

            // Set flag to active
            let flag_offset = (idx * 4) as u64;
            queue.write_buffer(&self.particle_flags, flag_offset, bytemuck::bytes_of(&1u32));
        }

        self.active_count += spawned as u32;
        spawned
    }

    /// Despawn all particles within the given axis-aligned bounding box.
    /// Returns the number of particles despawned.
    pub fn despawn_region(&mut self, _queue: &wgpu::Queue, min: [f32; 3], max: [f32; 3]) -> usize {
        // Note: This would ideally be a GPU compute pass for performance.
        // For now, we mark indices for despawn; actual GPU update happens in step().
        // This is a placeholder that records the region for the next step().
        let _ = (min, max);
        // TODO: Implement GPU-side region despawn in compute shader
        0
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

        let particle_workgroups = self.particle_count.div_ceil(64);
        let current_src = self.frame_index % 2;

        // --- Setup Bind Groups ---
        let global_bg = &self.global_bind_group;
        let particles_bg = &self.particles_bind_groups[current_src];
        let secondary_bg = &self.secondary_bind_group;

        // Group 3: Scene Data (Objects + SDF)
        // We create this per frame because the SDF texture view might change
        let sdf_view = self.sdf_system.texture_a.create_view(&Default::default());
        let scene_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fluid Scene BG"),
            layout: &self.scene_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.objects_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&sdf_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.default_sampler),
                },
            ],
        });

        // 0. Reset density error and counters
        encoder.clear_buffer(&self.density_error_buffer, 0, None);
        encoder.clear_buffer(&self.secondary_counter, 0, None);

        // --- Execute Compute Pipeline ---

        // Common Bindings: 0:Global, 1:Particles, 2:Secondary, 3:Scene

        // 1. Predict and Clear Grid
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::Predict"),
                ..Default::default()
            });
            cpass.set_pipeline(&self.predict_pipeline);
            cpass.set_bind_group(0, global_bg, &[]);
            cpass.set_bind_group(1, particles_bg, &[]);
            cpass.set_bind_group(3, &scene_bg, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::ClearGrid"),
                ..Default::default()
            });
            cpass.set_pipeline(&self.clear_grid_pipeline);
            cpass.set_bind_group(0, global_bg, &[]);
            cpass.dispatch_workgroups(
                (self.grid_width * self.grid_height * self.grid_depth).div_ceil(64),
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
            cpass.set_pipeline(&self.build_grid_pipeline);
            cpass.set_bind_group(0, global_bg, &[]);
            cpass.set_bind_group(1, particles_bg, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        // 3. PBD Iterations
        for _ in 0..self.iterations {
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Fluid::Lambda"),
                    ..Default::default()
                });
                cpass.set_pipeline(&self.lambda_pipeline);
                cpass.set_bind_group(0, global_bg, &[]);
                cpass.set_bind_group(1, particles_bg, &[]);
                cpass.set_bind_group(3, &scene_bg, &[]);
                cpass.dispatch_workgroups(particle_workgroups, 1, 1);
            }
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Fluid::DeltaPos"),
                    ..Default::default()
                });
                cpass.set_pipeline(&self.delta_pos_pipeline);
                cpass.set_bind_group(0, global_bg, &[]);
                cpass.set_bind_group(1, particles_bg, &[]);
                cpass.set_bind_group(3, &scene_bg, &[]);
                cpass.dispatch_workgroups(particle_workgroups, 1, 1);
            }
        }

        // 4. Integrate
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::Integrate"),
                ..Default::default()
            });
            cpass.set_pipeline(&self.integrate_pipeline);
            cpass.set_bind_group(0, global_bg, &[]);
            cpass.set_bind_group(1, particles_bg, &[]);
            cpass.set_bind_group(3, &scene_bg, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }

        // 5. Dye Mixing & Whitewater
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid::Dye&Whitewater"),
                ..Default::default()
            });
            cpass.set_bind_group(0, global_bg, &[]);
            cpass.set_bind_group(1, particles_bg, &[]);
            cpass.set_bind_group(2, secondary_bg, &[]);

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
        // 6. Copy error to staging for adaptive iterations (asynchronously)
        let staging_idx = self.frame_index % 2;
        let other_idx = 1 - staging_idx;

        // Ensure the current staging buffer is unmapped before copy
        if self.staging_mapped[staging_idx] {
            self.density_error_staging_buffers[staging_idx].unmap();
            self.staging_mapped[staging_idx] = false;
        }

        encoder.copy_buffer_to_buffer(
            &self.density_error_buffer,
            0,
            &self.density_error_staging_buffers[staging_idx],
            0,
            4,
        );

        self.frame_index += 1;

        // --- Adaptive Iteration Adjust (Non-Blocking) ---
        // We read from the *other* buffer, which was submitted in the previous frame.
        if self.staging_mapped[other_idx] {
            let buffer_slice = self.density_error_staging_buffers[other_idx].slice(..);
            {
                let data = buffer_slice.get_mapped_range();
                // Safe conversion: we know staging buffer is exactly 4 bytes
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(&data[0..4]);
                let error_scaled = u32::from_ne_bytes(bytes);
                let avg_error = (error_scaled as f32 / 1000.0) / self.particle_count as f32;

                // Adjust iterations based on error
                if avg_error > 0.05 {
                    self.iterations = (self.iterations + 1).min(8);
                } else if avg_error < 0.01 {
                    self.iterations = (self.iterations.saturating_sub(1)).max(2);
                }
            }
            self.density_error_staging_buffers[other_idx].unmap();
            self.staging_mapped[other_idx] = false;
        }

        // Map the current buffer for retrieval in the next frame
        let current_slice = self.density_error_staging_buffers[staging_idx].slice(..);
        current_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.staging_mapped[staging_idx] = true;

        // Poll to progress the mapping, but don't wait.
        let _ = device.poll(wgpu::MaintainBase::Poll);
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
            phase: 0,
            temperature: 293.0,
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
            phase: 0,
            temperature: 293.0,
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
                phase: 0,
                temperature: 293.0,
                color: [0.0; 4],
            },
            Particle {
                position: [4.0, 5.0, 6.0, 1.0],
                velocity: [1.0, 1.0, 1.0, 0.0],
                predicted_position: [5.0, 6.0, 7.0, 1.0],
                lambda: 2.0,
                density: 2.0,
                phase: 0,
                temperature: 293.0,
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

    #[test]
    fn test_grid_indexing_consistency() {
        let cell_size = 1.2;
        let grid_width = 128u32;
        let grid_height = 128u32;

        let pos = [1.5, 2.5, 3.5];
        let gx = (pos[0] as f32 / cell_size).floor() as i32;
        let gy = (pos[1] as f32 / cell_size).floor() as i32;
        let gz = (pos[2] as f32 / cell_size).floor() as i32;

        assert_eq!(gx, 1);
        assert_eq!(gy, 2);
        assert_eq!(gz, 2);

        let cell_idx =
            (gx as u32) + (gy as u32) * grid_width + (gz as u32) * (grid_width * grid_height);

        assert_eq!(cell_idx, 1 + 2 * 128 + 2 * 128 * 128);
    }

    // ================== DynamicObject Tests ==================

    #[test]
    fn test_dynamic_object_size() {
        // DynamicObject should be 144 bytes:
        // transform: 4x4 f32 = 64 bytes
        // inv_transform: 4x4 f32 = 64 bytes
        // half_extents: 4 f32 = 16 bytes
        // Total = 144 bytes
        assert_eq!(std::mem::size_of::<DynamicObject>(), 144);
    }

    #[test]
    fn test_dynamic_object_identity() {
        let obj = DynamicObject {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            inv_transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            half_extents: [1.0, 1.0, 1.0, 0.0], // Box type (w=0)
        };

        assert_eq!(obj.transform[0][0], 1.0);
        assert_eq!(obj.transform[3][3], 1.0);
        assert_eq!(obj.half_extents[3], 0.0); // Box type
    }

    #[test]
    fn test_dynamic_object_sphere() {
        let sphere = DynamicObject {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [5.0, 10.0, 15.0, 1.0], // Position at (5, 10, 15)
            ],
            inv_transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [-5.0, -10.0, -15.0, 1.0],
            ],
            half_extents: [2.0, 2.0, 2.0, 1.0], // Sphere type (w=1), radius=2
        };

        assert_eq!(sphere.half_extents[3], 1.0); // Sphere type
        assert_eq!(sphere.transform[3][0], 5.0); // X position
    }

    #[test]
    fn test_dynamic_object_bytemuck_cast() {
        let obj = DynamicObject {
            transform: [[1.0; 4]; 4],
            inv_transform: [[2.0; 4]; 4],
            half_extents: [3.0, 4.0, 5.0, 0.0],
        };

        let bytes: &[u8] = bytemuck::bytes_of(&obj);
        assert_eq!(bytes.len(), std::mem::size_of::<DynamicObject>());

        let recovered: &DynamicObject = bytemuck::from_bytes(bytes);
        assert_eq!(recovered.transform[0][0], 1.0);
        assert_eq!(recovered.inv_transform[0][0], 2.0);
        assert_eq!(recovered.half_extents[0], 3.0);
    }

    #[test]
    fn test_dynamic_object_copy() {
        let obj = DynamicObject {
            transform: [[1.0; 4]; 4],
            inv_transform: [[1.0; 4]; 4],
            half_extents: [1.0; 4],
        };

        let copied = obj; // Copy trait
        assert_eq!(copied.transform[0][0], obj.transform[0][0]);
    }

    #[test]
    fn test_dynamic_object_clone() {
        let obj = DynamicObject {
            transform: [[5.0; 4]; 4],
            inv_transform: [[6.0; 4]; 4],
            half_extents: [1.0, 2.0, 3.0, 4.0],
        };

        let cloned = obj.clone();
        assert_eq!(cloned.half_extents, obj.half_extents);
    }

    // ================== SecondaryParticle Tests ==================

    #[test]
    fn test_secondary_particle_size() {
        // SecondaryParticle should be 48 bytes:
        // position: 4 f32 = 16 bytes
        // velocity: 4 f32 = 16 bytes
        // info: 4 f32 = 16 bytes
        // Total = 48 bytes
        assert_eq!(std::mem::size_of::<SecondaryParticle>(), 48);
    }

    #[test]
    fn test_secondary_particle_creation() {
        let particle = SecondaryParticle {
            position: [1.0, 2.0, 3.0, 1.0],
            velocity: [0.0, -1.0, 0.0, 0.0],
            info: [1.0, 0.0, 1.0, 0.5], // lifetime=1s, type=spray, alpha=1, scale=0.5
        };

        assert_eq!(particle.position[0], 1.0);
        assert_eq!(particle.velocity[1], -1.0);
        assert_eq!(particle.info[0], 1.0); // lifetime
        assert_eq!(particle.info[2], 1.0); // alpha
    }

    #[test]
    fn test_secondary_particle_types() {
        // Type 0: Spray
        let spray = SecondaryParticle {
            position: [0.0; 4],
            velocity: [0.0; 4],
            info: [1.0, 0.0, 1.0, 1.0], // type = 0 (spray)
        };
        assert_eq!(spray.info[1], 0.0);

        // Type 1: Foam
        let foam = SecondaryParticle {
            position: [0.0; 4],
            velocity: [0.0; 4],
            info: [2.0, 1.0, 0.8, 0.5], // type = 1 (foam)
        };
        assert_eq!(foam.info[1], 1.0);

        // Type 2: Bubbles
        let bubble = SecondaryParticle {
            position: [0.0; 4],
            velocity: [0.0; 4],
            info: [0.5, 2.0, 0.5, 0.25], // type = 2 (bubble)
        };
        assert_eq!(bubble.info[1], 2.0);
    }

    #[test]
    fn test_secondary_particle_bytemuck_cast() {
        let particles = vec![
            SecondaryParticle {
                position: [1.0, 2.0, 3.0, 1.0],
                velocity: [0.1, 0.2, 0.3, 0.0],
                info: [1.0, 0.0, 1.0, 1.0],
            },
            SecondaryParticle {
                position: [4.0, 5.0, 6.0, 1.0],
                velocity: [0.4, 0.5, 0.6, 0.0],
                info: [2.0, 1.0, 0.5, 0.5],
            },
        ];

        let bytes: &[u8] = bytemuck::cast_slice(&particles);
        assert_eq!(bytes.len(), 2 * std::mem::size_of::<SecondaryParticle>());

        let recovered: &[SecondaryParticle] = bytemuck::cast_slice(bytes);
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].position[0], 1.0);
        assert_eq!(recovered[1].position[0], 4.0);
    }

    #[test]
    fn test_secondary_particle_copy() {
        let particle = SecondaryParticle {
            position: [1.0, 2.0, 3.0, 1.0],
            velocity: [0.0, -1.0, 0.0, 0.0],
            info: [1.0, 0.0, 1.0, 0.5],
        };

        let copied = particle; // Copy trait
        assert_eq!(copied.position, particle.position);
    }

    #[test]
    fn test_secondary_particle_clone() {
        let particle = SecondaryParticle {
            position: [10.0, 20.0, 30.0, 1.0],
            velocity: [1.0, 2.0, 3.0, 0.0],
            info: [5.0, 1.0, 0.8, 2.0],
        };

        let cloned = particle.clone();
        assert_eq!(cloned.info, particle.info);
    }

    // ================== Additional Particle Tests ==================

    #[test]
    fn test_particle_phase_values() {
        // Test different phase types
        let water = Particle {
            position: [0.0; 4],
            velocity: [0.0; 4],
            predicted_position: [0.0; 4],
            lambda: 0.0,
            density: 1.0,
            phase: 0, // Water
            temperature: 293.0,
            color: [0.2, 0.5, 0.8, 1.0],
        };
        assert_eq!(water.phase, 0);

        let oil = Particle {
            phase: 1, // Oil
            ..water
        };
        assert_eq!(oil.phase, 1);

        let custom = Particle {
            phase: 2, // Custom
            ..water
        };
        assert_eq!(custom.phase, 2);
    }

    #[test]
    fn test_particle_temperature() {
        // Test temperature handling
        let cold = Particle {
            position: [0.0; 4],
            velocity: [0.0; 4],
            predicted_position: [0.0; 4],
            lambda: 0.0,
            density: 1.0,
            phase: 0,
            temperature: 273.0, // 0°C
            color: [0.0; 4],
        };
        assert_eq!(cold.temperature, 273.0);

        let hot = Particle {
            temperature: 373.0, // 100°C
            ..cold
        };
        assert_eq!(hot.temperature, 373.0);
    }

    #[test]
    fn test_particle_color_channels() {
        let particle = Particle {
            position: [0.0; 4],
            velocity: [0.0; 4],
            predicted_position: [0.0; 4],
            lambda: 0.0,
            density: 1.0,
            phase: 0,
            temperature: 293.0,
            color: [1.0, 0.5, 0.25, 0.75], // RGBA
        };

        assert_eq!(particle.color[0], 1.0);   // Red
        assert_eq!(particle.color[1], 0.5);   // Green
        assert_eq!(particle.color[2], 0.25);  // Blue
        assert_eq!(particle.color[3], 0.75);  // Alpha
    }

    #[test]
    fn test_sim_params_dt_values() {
        let params_60fps = SimParams {
            smoothing_radius: 1.0,
            target_density: 1.0,
            pressure_multiplier: 1.0,
            viscosity: 1.0,
            surface_tension: 1.0,
            gravity: -9.81,
            dt: 1.0 / 60.0, // 60 FPS
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

        // dt for 60 FPS should be approximately 0.0167
        assert!((params_60fps.dt - 0.0166666).abs() < 0.001);

        let params_144fps = SimParams {
            dt: 1.0 / 144.0, // 144 FPS
            ..params_60fps
        };

        // dt for 144 FPS should be approximately 0.00694
        assert!((params_144fps.dt - 0.00694).abs() < 0.001);
    }
}
