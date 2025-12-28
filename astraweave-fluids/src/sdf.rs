use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SdfConfig {
    resolution: u32,
    world_size: f32,
    triangle_count: u32,
    padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct JfaParams {
    step_size: u32,
    padding: [u32; 3],
}

pub struct SdfSystem {
    init_pipeline: wgpu::ComputePipeline,
    step_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::ComputePipeline,

    #[allow(dead_code)]
    config_buffer: wgpu::Buffer,
    jfa_params_buffer: wgpu::Buffer,

    pub texture_a: wgpu::Texture,
    pub texture_b: wgpu::Texture,

    bind_group_a: wgpu::BindGroup, // Read A, Write B
    bind_group_b: wgpu::BindGroup, // Read B, Write A

    config_bind_group: wgpu::BindGroup,
    jfa_bind_group: wgpu::BindGroup,
    pub resolution: u32,
}

impl SdfSystem {
    pub fn new(
        device: &wgpu::Device,
        objects_buffer: &wgpu::Buffer,
        resolution: u32,
        world_size: f32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SDF Gen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sdf_gen.wgsl").into()),
        });

        let config = SdfConfig {
            resolution,
            world_size,
            triangle_count: 0,
            padding: 0.0,
        };

        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SDF Config"),
            contents: bytemuck::bytes_of(&config),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let jfa_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("JFA Params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("SDF Texture"),
            size: wgpu::Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: resolution,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let texture_a = device.create_texture(&texture_desc);
        let texture_b = device.create_texture(&texture_desc);

        let view_a = texture_a.create_view(&wgpu::TextureViewDescriptor::default());
        let view_b = texture_b.create_view(&wgpu::TextureViewDescriptor::default());

        // Layouts
        let config_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SDF Config Layout"),
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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SDF Texture Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
            ],
        });

        let jfa_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("JFA Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let config_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SDF Config BG"),
            layout: &config_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: config_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: objects_buffer.as_entire_binding(),
                },
            ],
        });

        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SDF BG A"),
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_a),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view_b),
                },
            ],
        });

        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SDF BG B"),
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_b),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view_a),
                },
            ],
        });

        let jfa_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("JFA BG"),
            layout: &jfa_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: jfa_params_buffer.as_entire_binding(),
            }],
        });
        // We actually need the pipelines to store the jfa layout if we use multiple JFA steps.
        // But let's just make the final system.

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SDF Pipeline Layout"),
            bind_group_layouts: &[&config_layout, &texture_layout, &jfa_layout],
            push_constant_ranges: &[],
        });

        let init_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("SDF Init Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("init"),
            compilation_options: Default::default(),
            cache: None,
        });

        let step_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("SDF Step Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("jfa_step"),
            compilation_options: Default::default(),
            cache: None,
        });

        let finalize_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("SDF Finalize Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("finalize"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            init_pipeline,
            step_pipeline,
            finalize_pipeline,
            config_buffer,
            jfa_params_buffer,
            texture_a,
            texture_b,
            bind_group_a,
            bind_group_b,
            config_bind_group,
            jfa_bind_group,
            resolution,
        }
    }

    pub fn generate(&self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue) {
        let workgroups = (self.resolution + 7) / 8;

        // 1. Init
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SDF Init"),
                ..Default::default()
            });
            cpass.set_pipeline(&self.init_pipeline);
            cpass.set_bind_group(0, &self.config_bind_group, &[]);
            cpass.set_bind_group(1, &self.bind_group_a, &[]); // Write to B
            cpass.set_bind_group(2, &self.jfa_bind_group, &[]); // Dummy
            cpass.dispatch_workgroups(workgroups, workgroups, workgroups);
        }

        // 2. JFA Steps
        let mut step_size = (self.resolution / 2) as u32;
        let mut current_read_a = false; // After init, texture_b has data, so next step reads B, writes A.

        while step_size >= 1 {
            let params = JfaParams {
                step_size,
                padding: [0; 3],
            };
            queue.write_buffer(&self.jfa_params_buffer, 0, bytemuck::bytes_of(&params));

            {
                let label = format!("JFA Step {}", step_size);
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(label.as_str()),
                    ..Default::default()
                });
                cpass.set_pipeline(&self.step_pipeline);
                cpass.set_bind_group(0, &self.config_bind_group, &[]);
                if current_read_a {
                    // Read B, Write A
                    cpass.set_bind_group(1, &self.bind_group_b, &[]);
                } else {
                    // Read A, Write B
                    cpass.set_bind_group(1, &self.bind_group_a, &[]);
                }
                cpass.set_bind_group(2, &self.jfa_bind_group, &[]);

                cpass.dispatch_workgroups(workgroups, workgroups, workgroups);
            }

            step_size /= 2;
            current_read_a = !current_read_a;
        }

        // 3. Finalize
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SDF Finalize"),
                ..Default::default()
            });
            cpass.set_pipeline(&self.finalize_pipeline);
            cpass.set_bind_group(0, &self.config_bind_group, &[]);
            if current_read_a {
                // Final result was in A, so read A?
                // Wait, if current_read_a is true, it means last step Read A, Wrote B.
                // So result is in B.
                cpass.set_bind_group(1, &self.bind_group_b, &[]);
            } else {
                // Last step Read B, Wrote A. Result in A.
                cpass.set_bind_group(1, &self.bind_group_a, &[]);
            }
            cpass.set_bind_group(2, &self.jfa_bind_group, &[]);
            cpass.dispatch_workgroups(workgroups, workgroups, workgroups);
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_jfa_convergence_steps() {
        let resolution = 64u32;
        let mut steps = Vec::new();
        let mut step_size = resolution / 2;
        while step_size > 0 {
            steps.push(step_size);
            step_size /= 2;
        }

        // For 64, we expect 32, 16, 8, 4, 2, 1 (6 steps)
        assert_eq!(steps.len(), 6);
        assert_eq!(steps[0], 32);
        assert_eq!(steps[5], 1);
    }
}
