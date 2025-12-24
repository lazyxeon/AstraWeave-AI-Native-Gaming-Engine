use wgpu::util::DeviceExt;

pub struct SdfSystem {
    init_pipeline: wgpu::ComputePipeline,
    step_pipeline: wgpu::ComputePipeline,
    finalize_pipeline: wgpu::ComputePipeline,

    config_buffer: wgpu::Buffer,
    jfa_params_buffer: wgpu::Buffer,

    pub texture_a: wgpu::Texture,
    pub texture_b: wgpu::Texture,

    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,

    pub resolution: u32,
}

impl SdfSystem {
    pub fn new(device: &wgpu::Device, resolution: u32, world_size: f32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SDF Gen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sdf_gen.wgsl").into()),
        });

        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SDF Config"),
            contents: bytemuck::cast_slice(&[resolution, world_size.to_bits(), 0]), // triangle_count=0 for now
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
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture_a = device.create_texture(&texture_desc);
        let texture_b = device.create_texture(&texture_desc);

        // Bind group layouts ...
        // (Simplified for this step)

        // Return dummy instance for now to get compilation going
        // I will implement the full pipelines in the next step
        unimplemented!(
            "Full SDF system implementation requires more boilerplate, providing skeleton first"
        )
    }
}
