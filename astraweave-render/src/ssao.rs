//! Screen-Space Ambient Occlusion (SSAO)
//!
//! Provides depth-aware ambient occlusion for enhanced visual quality.
//! Uses hemisphere sampling with depth-aware blur for smooth results.

use wgpu::util::DeviceExt;

/// SSAO quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsaoQuality {
    /// 8 samples, small radius, no blur
    Low,
    /// 16 samples, medium radius, 3x3 blur
    Medium,
    /// 32 samples, large radius, 5x5 blur
    High,
    /// 64 samples, largest radius, 7x7 bilateral blur
    Ultra,
}

impl SsaoQuality {
    pub fn sample_count(&self) -> u32 {
        match self {
            SsaoQuality::Low => 8,
            SsaoQuality::Medium => 16,
            SsaoQuality::High => 32,
            SsaoQuality::Ultra => 64,
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            SsaoQuality::Low => 0.5,
            SsaoQuality::Medium => 1.0,
            SsaoQuality::High => 1.5,
            SsaoQuality::Ultra => 2.0,
        }
    }

    pub fn blur_kernel_size(&self) -> u32 {
        match self {
            SsaoQuality::Low => 0,
            SsaoQuality::Medium => 3,
            SsaoQuality::High => 5,
            SsaoQuality::Ultra => 7,
        }
    }
}

/// SSAO configuration
#[derive(Debug, Clone)]
pub struct SsaoConfig {
    pub quality: SsaoQuality,
    pub radius: f32,
    pub bias: f32,
    pub intensity: f32,
    pub enabled: bool,
}

impl Default for SsaoConfig {
    fn default() -> Self {
        Self {
            quality: SsaoQuality::Medium,
            radius: 1.0,
            bias: 0.025,
            intensity: 1.0,
            enabled: true,
        }
    }
}

/// SSAO GPU uniforms
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SsaoUniforms {
    /// Projection matrix for depth reconstruction
    pub proj: [[f32; 4]; 4],
    /// Inverse projection
    pub inv_proj: [[f32; 4]; 4],
    /// Screen dimensions (width, height, 1/width, 1/height)
    pub screen_params: [f32; 4],
    /// (radius, bias, intensity, sample_count)
    pub ssao_params: [f32; 4],
}

/// SSAO kernel samples (hemisphere)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SsaoKernel {
    pub samples: [[f32; 4]; 64], // max 64 samples
}

impl SsaoKernel {
    /// Generate hemisphere samples with improved distribution
    pub fn generate(sample_count: u32) -> Self {
        use std::f32::consts::PI;

        let mut samples = [[0.0f32; 4]; 64];

        for i in 0..sample_count.min(64) as usize {
            // Cosine-weighted hemisphere sampling
            let xi1 = (i as f32 + 0.5) / sample_count as f32;
            let xi2 = Self::halton(i as u32, 2);

            let phi = 2.0 * PI * xi1;
            let cos_theta = (1.0 - xi2).sqrt();
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

            let x = sin_theta * phi.cos();
            let y = sin_theta * phi.sin();
            let z = cos_theta;

            // Scale sample to distribute more towards origin
            let scale = (i as f32 + 1.0) / sample_count as f32;
            let scale = 0.1 + scale * scale * 0.9;

            samples[i] = [x * scale, y * scale, z * scale, 0.0];
        }

        Self { samples }
    }

    /// Halton sequence for quasi-random sampling
    fn halton(index: u32, base: u32) -> f32 {
        let mut f = 1.0f32;
        let mut r = 0.0f32;
        let mut i = index;

        while i > 0 {
            f /= base as f32;
            r += f * (i % base) as f32;
            i /= base;
        }

        r
    }
}

/// SSAO render resources
pub struct SsaoResources {
    pub ao_texture: wgpu::Texture,
    pub ao_view: wgpu::TextureView,
    pub blur_texture: wgpu::Texture,
    pub blur_view: wgpu::TextureView,
    pub noise_texture: wgpu::Texture,
    pub noise_view: wgpu::TextureView,
    pub uniform_buffer: wgpu::Buffer,
    pub kernel_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
    pub blur_pipeline: wgpu::RenderPipeline,
    pub config: SsaoConfig,
}

impl SsaoResources {
    /// Create SSAO resources for given screen dimensions
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        depth_view: &wgpu::TextureView,
        normal_view: &wgpu::TextureView,
        config: SsaoConfig,
    ) -> Self {
        // AO textures (single channel)
        let ao_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let ao_view = ao_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let blur_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Blur Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let blur_view = blur_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 4x4 noise texture for random rotation
        let noise_data = Self::generate_noise();
        let noise_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("SSAO Noise"),
                size: wgpu::Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rg32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            bytemuck::cast_slice(&noise_data),
        );
        let noise_view = noise_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Uniforms
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Uniforms"),
            size: std::mem::size_of::<SsaoUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Kernel
        let kernel = SsaoKernel::generate(config.quality.sample_count());
        let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSAO Kernel"),
            contents: bytemuck::cast_slice(&[kernel]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Bind Group Layout"),
            entries: &[
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Kernel
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Depth texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Normal texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Noise texture
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("SSAO Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSAO Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: kernel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&noise_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // SSAO shader
        let ssao_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SSAO Shader"),
            source: wgpu::ShaderSource::Wgsl(SSAO_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SSAO Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &ssao_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssao_shader,
                entry_point: Some("fs_ssao"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Blur shader
        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SSAO Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(SSAO_BLUR_SHADER.into()),
        });

        let blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SSAO Blur Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &blur_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blur_shader,
                entry_point: Some("fs_blur"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            ao_texture,
            ao_view,
            blur_texture,
            blur_view,
            noise_texture,
            noise_view,
            uniform_buffer,
            kernel_buffer,
            bind_group_layout,
            bind_group,
            pipeline,
            blur_pipeline,
            config,
        }
    }

    /// Generate 4x4 rotation noise texture
    fn generate_noise() -> Vec<[f32; 2]> {
        use std::f32::consts::PI;
        let mut noise = Vec::with_capacity(16);

        for i in 0..16 {
            let angle = (i as f32 / 16.0) * 2.0 * PI;
            noise.push([angle.cos(), angle.sin()]);
        }

        noise
    }
}

/// SSAO main shader
const SSAO_SHADER: &str = r#"
struct Uniforms {
    proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    screen_params: vec4<f32>,
    ssao_params: vec4<f32>,
}

struct Kernel {
    samples: array<vec4<f32>, 64>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<uniform> kernel: Kernel;
@group(0) @binding(2) var depth_tex: texture_depth_2d;
@group(0) @binding(3) var normal_tex: texture_2d<f32>;
@group(0) @binding(4) var noise_tex: texture_2d<f32>;
@group(0) @binding(5) var point_sampler: sampler;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + 1.0) * 0.5;
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

fn reconstruct_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let view_pos = uniforms.inv_proj * ndc;
    return view_pos.xyz / view_pos.w;
}

@fragment
fn fs_ssao(in: VSOut) -> @location(0) f32 {
    let radius = uniforms.ssao_params.x;
    let bias = uniforms.ssao_params.y;
    let intensity = uniforms.ssao_params.z;
    let sample_count = u32(uniforms.ssao_params.w);

    let depth = textureSample(depth_tex, point_sampler, in.uv);
    if depth >= 1.0 { return 1.0; }

    let pos = reconstruct_position(in.uv, depth);
    let normal = textureSample(normal_tex, point_sampler, in.uv).xyz * 2.0 - 1.0;

    // Tile noise
    let noise_uv = in.uv * uniforms.screen_params.xy / 4.0;
    let noise = textureSample(noise_tex, point_sampler, noise_uv).xy;

    // Build TBN with random rotation
    let tangent = normalize(vec3<f32>(noise.x, noise.y, 0.0) - normal * dot(vec3<f32>(noise.x, noise.y, 0.0), normal));
    let bitangent = cross(normal, tangent);
    let tbn = mat3x3<f32>(tangent, bitangent, normal);

    var occlusion = 0.0;
    for (var i = 0u; i < sample_count; i++) {
        let sample_dir = tbn * kernel.samples[i].xyz;
        let sample_pos = pos + sample_dir * radius;

        // Project to screen
        var offset = uniforms.proj * vec4<f32>(sample_pos, 1.0);
        offset = offset / offset.w;
        let sample_uv = offset.xy * 0.5 + 0.5;

        let sample_depth = textureSample(depth_tex, point_sampler, vec2<f32>(sample_uv.x, 1.0 - sample_uv.y));
        let sample_z = reconstruct_position(sample_uv, sample_depth).z;

        let range_check = smoothstep(0.0, 1.0, radius / abs(pos.z - sample_z));
        occlusion += select(0.0, 1.0, sample_z >= sample_pos.z + bias) * range_check;
    }

    occlusion = 1.0 - (occlusion / f32(sample_count)) * intensity;
    return occlusion;
}
"#;

/// SSAO blur shader (depth-aware bilateral blur)
const SSAO_BLUR_SHADER: &str = r#"
struct Uniforms {
    proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    screen_params: vec4<f32>,
    ssao_params: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(2) var depth_tex: texture_depth_2d;
@group(0) @binding(3) var ao_tex: texture_2d<f32>;
@group(0) @binding(5) var point_sampler: sampler;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + 1.0) * 0.5;
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@fragment
fn fs_blur(in: VSOut) -> @location(0) f32 {
    let texel_size = uniforms.screen_params.zw;
    let center_depth = textureSample(depth_tex, point_sampler, in.uv);

    var result = 0.0;
    var total_weight = 0.0;

    for (var x = -2; x <= 2; x++) {
        for (var y = -2; y <= 2; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_uv = in.uv + offset;

            let sample_depth = textureSample(depth_tex, point_sampler, sample_uv);
            let sample_ao = textureSample(ao_tex, point_sampler, sample_uv).r;

            // Depth-weighted bilateral filter
            let depth_diff = abs(center_depth - sample_depth);
            let weight = exp(-depth_diff * 1000.0);

            result += sample_ao * weight;
            total_weight += weight;
        }
    }

    return result / max(total_weight, 0.0001);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssao_quality_params() {
        assert_eq!(SsaoQuality::Low.sample_count(), 8);
        assert_eq!(SsaoQuality::Ultra.sample_count(), 64);
        assert!(SsaoQuality::High.radius() > SsaoQuality::Low.radius());
    }

    #[test]
    fn test_kernel_generation() {
        let kernel = SsaoKernel::generate(16);

        // All samples should be in hemisphere (z >= 0)
        for i in 0..16 {
            let sample = &kernel.samples[i];
            assert!(sample[2] >= 0.0, "Sample {} has negative Z", i);

            // Should be normalized (accounting for scale)
            let len =
                (sample[0] * sample[0] + sample[1] * sample[1] + sample[2] * sample[2]).sqrt();
            assert!(len <= 1.0, "Sample {} exceeds unit sphere", i);
        }
    }

    #[test]
    fn test_config_defaults() {
        let config = SsaoConfig::default();
        assert!(config.enabled);
        assert!((config.intensity - 1.0).abs() < 0.01);
    }
}
