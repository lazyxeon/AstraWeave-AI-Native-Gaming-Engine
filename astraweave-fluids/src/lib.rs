use wgpu::util::DeviceExt;
use std::borrow::Cow;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub position: [f32; 4],
    pub velocity: [f32; 4],
    pub force: [f32; 4],
    pub density: f32,
    pub pressure: f32,
    pub padding1: f32,
    pub padding2: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimParams {
    pub delta_time: f32,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub viscosity: f32,
    pub particle_count: u32,
    pub gravity: f32,
    pub padding: u32,
    pub cell_size: f32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_depth: u32,
}

pub struct FluidSystem {
    particle_buffers: Vec<wgpu::Buffer>,
    // We need bind groups that swap Src/Dst
    // layout: 0: Params, 1: Src, 2: Dst, 3: head_pointers, 4: next_pointers
    // bg0: Src=Buf0, Dst=Buf1
    // bg1: Src=Buf1, Dst=Buf0
    bind_groups: Vec<wgpu::BindGroup>, 
    
    head_pointers: wgpu::Buffer,
    next_pointers: wgpu::Buffer,
    
    clear_grid_pipeline: wgpu::ComputePipeline,
    build_grid_pipeline: wgpu::ComputePipeline,
    density_pipeline: wgpu::ComputePipeline,
    force_pipeline: wgpu::ComputePipeline,
    integrate_pipeline: wgpu::ComputePipeline,
    
    params_buffer: wgpu::Buffer,
    pub particle_count: u32,
    pub frame_index: usize,
    
    // Sim constants
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub viscosity: f32,
    pub gravity: f32,
    
    // Grid params
    pub cell_size: f32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_depth: u32,
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
        let size = (particle_count as f32).powf(1.0/3.0).ceil() as usize;
        
        for i in 0..particle_count as usize {
            let x = (i % size) as f32 * spacing - 5.0;
            let y = ((i / size) % size) as f32 * spacing + 2.0;
            let z = (i / (size * size)) as f32 * spacing - 5.0;
            
            initial_particles.push(Particle {
                position: [x, y, z, 1.0],
                velocity: [0.0; 4],
                force: [0.0; 4],
                density: 0.0,
                pressure: 0.0,
                padding1: 0.0,
                padding2: 0.0,
            });
        }

        let buffer_size = (particle_count as usize * std::mem::size_of::<Particle>()) as u64;
        
        let buf0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Buffer 0"),
            contents: bytemuck::cast_slice(&initial_particles),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });
        
        let buf1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer 1"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
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
            particle_count,
            gravity: -9.8,
            padding: 0,
            cell_size,
            grid_width,
            grid_height,
            grid_depth,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sim Params Buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Bind Group Layout
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
            ],
        });

        // Create 2 Bind Groups (Ping-Pong)
        let bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fluid BG 0 (Src=0, Dst=1)"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: particle_buffers[0].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: particle_buffers[1].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: head_pointers.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: next_pointers.as_entire_binding() },
            ],
        });
        
        let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Fluid BG 1 (Src=1, Dst=0)"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: particle_buffers[1].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: particle_buffers[0].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: head_pointers.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: next_pointers.as_entire_binding() },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let clear_grid_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Clear Grid Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("clear_grid"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let build_grid_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Build Grid Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("build_grid"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let density_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Density Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("compute_density_pressure"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let force_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("compute_forces"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let integrate_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Integrate Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("integrate"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            particle_buffers,
            bind_groups: vec![bg0, bg1],
            head_pointers,
            next_pointers,
            clear_grid_pipeline,
            build_grid_pipeline,
            density_pipeline,
            force_pipeline,
            integrate_pipeline,
            params_buffer,
            particle_count,
            frame_index: 0,
            smoothing_radius: 1.0,
            target_density: 10.0,
            pressure_multiplier: 250.0,
            viscosity: 50.0,
            gravity: -9.8,
            cell_size,
            grid_width,
            grid_height,
            grid_depth,
        }
    }

    pub fn step(&mut self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, dt: f32) {
        // Update Uniforms
        let params = SimParams {
            delta_time: dt.min(0.032), // Cap dt
            smoothing_radius: self.smoothing_radius,
            target_density: self.target_density,
            pressure_multiplier: self.pressure_multiplier,
            viscosity: self.viscosity,
            particle_count: self.particle_count,
            gravity: self.gravity,
            padding: 0,
            cell_size: self.cell_size,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
            grid_depth: self.grid_depth,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let current_src = self.frame_index % 2;
        let grid_size = self.grid_width * self.grid_height * self.grid_depth;
        let grid_workgroups = (grid_size + 63) / 64;
        let particle_workgroups = (self.particle_count + 63) / 64;
        
        // Pass 0: Clear Grid
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
                label: Some("Fluid Clear Grid"), 
                timestamp_writes: None 
            });
            cpass.set_pipeline(&self.clear_grid_pipeline);
            cpass.set_bind_group(0, &self.bind_groups[current_src], &[]);
            cpass.dispatch_workgroups(grid_workgroups, 1, 1);
        }
        
        // Pass 1: Build Grid
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
                label: Some("Fluid Build Grid"), 
                timestamp_writes: None 
            });
            cpass.set_pipeline(&self.build_grid_pipeline);
            cpass.set_bind_group(0, &self.bind_groups[current_src], &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }
        
        // Pass 2: Density
        let bg_density = &self.bind_groups[current_src]; 
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
                label: Some("Fluid Density"), 
                timestamp_writes: None 
            });
            cpass.set_pipeline(&self.density_pipeline);
            cpass.set_bind_group(0, bg_density, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }
        
        // Pass 3: Force
        let bg_force = &self.bind_groups[1 - current_src];
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
                label: Some("Fluid Force"), 
                timestamp_writes: None 
            });
            cpass.set_pipeline(&self.force_pipeline);
            cpass.set_bind_group(0, bg_force, &[]);
            cpass.dispatch_workgroups(particle_workgroups, 1, 1);
        }
        
        // Pass 4: Integrate
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
                label: Some("Fluid Integrate"), 
                timestamp_writes: None 
            });
            cpass.set_pipeline(&self.integrate_pipeline);
            cpass.set_bind_group(0, bg_density, &[]);
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
