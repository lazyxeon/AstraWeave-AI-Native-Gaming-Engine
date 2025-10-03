// Post-processing WGSL placeholder modules and compile-only tests.
// These shaders are not wired into runtime yet; we just ensure they parse via naga.

use anyhow::Result;

// ---------------------------------------------------------------------------------
// Bloom Post-Processing
// ---------------------------------------------------------------------------------

/// Bloom configuration parameters
#[derive(Clone, Debug)]
pub struct BloomConfig {
    /// Luminance threshold for bloom (values below are filtered out)
    pub threshold: f32,
    /// Bloom intensity multiplier
    pub intensity: f32,
    /// Number of downsample/upsample mip levels (1-8)
    pub mip_count: u32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            intensity: 0.05,
            mip_count: 5,
        }
    }
}

impl BloomConfig {
    /// Validate parameters are within safe ranges
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.threshold >= 0.0 && self.threshold <= 10.0,
            "Bloom threshold must be in [0.0, 10.0], got {}",
            self.threshold
        );
        anyhow::ensure!(
            self.intensity >= 0.0 && self.intensity <= 1.0,
            "Bloom intensity must be in [0.0, 1.0], got {}",
            self.intensity
        );
        anyhow::ensure!(
            self.mip_count >= 1 && self.mip_count <= 8,
            "Bloom mip_count must be in [1, 8], got {}",
            self.mip_count
        );
        Ok(())
    }
}

/// Bloom post-processing pipeline
/// Implements threshold + downsample mip chain → upsample composite (classic bloom)
#[cfg(feature = "bloom")]
pub struct BloomPipeline {
    config: BloomConfig,
    // Pipelines
    threshold_pipeline: wgpu::RenderPipeline,
    downsample_pipeline: wgpu::RenderPipeline,
    upsample_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    // Bind group layouts
    threshold_bgl: wgpu::BindGroupLayout,
    mip_bgl: wgpu::BindGroupLayout,
    composite_bgl: wgpu::BindGroupLayout,
    // Sampler
    sampler: wgpu::Sampler,
}

#[cfg(feature = "bloom")]
impl BloomPipeline {
    pub fn new(device: &wgpu::Device, config: BloomConfig) -> Result<Self> {
        config.validate()?;

        // Sampler for all bloom passes (linear filtering)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Bind group layouts
        let threshold_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom-threshold-bgl"),
            entries: &[
                // 0: input HDR texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 1: sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // 2: uniform buffer (threshold value)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let mip_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom-mip-bgl"),
            entries: &[
                // 0: input texture (previous mip)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 1: sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let composite_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom-composite-bgl"),
            entries: &[
                // 0: original HDR input
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 1: bloom blur result
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 2: sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // 3: uniform buffer (intensity)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Shader modules
        let threshold_sm = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom-threshold-sm"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_THRESHOLD_WGSL)),
        });
        let downsample_sm = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom-downsample-sm"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_DOWNSAMPLE_WGSL)),
        });
        let upsample_sm = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom-upsample-sm"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_UPSAMPLE_WGSL)),
        });
        let composite_sm = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom-composite-sm"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_COMPOSITE_WGSL)),
        });

        // Pipeline layouts
        let threshold_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom-threshold-pl"),
            bind_group_layouts: &[&threshold_bgl],
            push_constant_ranges: &[],
        });
        let mip_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom-mip-pl"),
            bind_group_layouts: &[&mip_bgl],
            push_constant_ranges: &[],
        });
        let composite_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom-composite-pl"),
            bind_group_layouts: &[&composite_bgl],
            push_constant_ranges: &[],
        });

        // Render pipelines
        let threshold_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom-threshold-pipeline"),
            layout: Some(&threshold_pl),
            vertex: wgpu::VertexState {
                module: &threshold_sm,
                entry_point: "vs",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &threshold_sm,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let downsample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom-downsample-pipeline"),
            layout: Some(&mip_pl),
            vertex: wgpu::VertexState {
                module: &downsample_sm,
                entry_point: "vs",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &downsample_sm,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let upsample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom-upsample-pipeline"),
            layout: Some(&mip_pl),
            vertex: wgpu::VertexState {
                module: &upsample_sm,
                entry_point: "vs",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &upsample_sm,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bloom-composite-pipeline"),
            layout: Some(&composite_pl),
            vertex: wgpu::VertexState {
                module: &composite_sm,
                entry_point: "vs",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &composite_sm,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            config,
            threshold_pipeline,
            downsample_pipeline,
            upsample_pipeline,
            composite_pipeline,
            threshold_bgl,
            mip_bgl,
            composite_bgl,
            sampler,
        })
    }

    pub fn config(&self) -> &BloomConfig {
        &self.config
    }

    /// Execute bloom pass: threshold → downsample → upsample → composite
    /// Returns the final composited output view
    pub fn execute(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        hdr_input: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<wgpu::TextureView> {
        // Create bloom chain texture (mip chain for downsample/upsample)
        let mip_count = self
            .config
            .mip_count
            .min(((width.min(height) as f32).log2().floor() as u32).max(1));

        let bloom_chain = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bloom-chain"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        // Create output texture for final composite
        let output_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bloom-output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());

        // Uniform buffers
        let threshold_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bloom-threshold-ub"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &threshold_buf,
            0,
            bytemuck::bytes_of(&[self.config.threshold, 0.0f32, 0.0, 0.0]),
        );

        let intensity_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bloom-intensity-ub"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &intensity_buf,
            0,
            bytemuck::bytes_of(&[self.config.intensity, 0.0f32, 0.0, 0.0]),
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("bloom-encoder"),
        });

        // 1. Threshold pass (mip 0)
        {
            let mip0_view = bloom_chain.create_view(&wgpu::TextureViewDescriptor {
                label: Some("bloom-mip0"),
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let threshold_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bloom-threshold-bg"),
                layout: &self.threshold_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(hdr_input),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: threshold_buf.as_entire_binding(),
                    },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom-threshold-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &mip0_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.threshold_pipeline);
            rp.set_bind_group(0, &threshold_bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // 2. Downsample pass (mip 0 → mip 1, mip 1 → mip 2, ...)
        for mip in 1..mip_count {
            let src_view = bloom_chain.create_view(&wgpu::TextureViewDescriptor {
                label: Some("bloom-downsample-src"),
                base_mip_level: mip - 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst_view = bloom_chain.create_view(&wgpu::TextureViewDescriptor {
                label: Some("bloom-downsample-dst"),
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let mip_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bloom-downsample-bg"),
                layout: &self.mip_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom-downsample-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.downsample_pipeline);
            rp.set_bind_group(0, &mip_bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // 3. Upsample pass (mip N-1 ← mip N, ..., mip 0 ← mip 1)
        // Upsample uses additive blending to accumulate blur contributions
        for mip in (0..(mip_count - 1)).rev() {
            let src_view = bloom_chain.create_view(&wgpu::TextureViewDescriptor {
                label: Some("bloom-upsample-src"),
                base_mip_level: mip + 1,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let dst_view = bloom_chain.create_view(&wgpu::TextureViewDescriptor {
                label: Some("bloom-upsample-dst"),
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let mip_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bloom-upsample-bg"),
                layout: &self.mip_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom-upsample-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Keep existing data for additive blend
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.upsample_pipeline);
            rp.set_bind_group(0, &mip_bg, &[]);
            rp.draw(0..3, 0..1);
        }

        // 4. Composite pass (original + bloom blur → output)
        {
            let bloom_view = bloom_chain.create_view(&wgpu::TextureViewDescriptor {
                label: Some("bloom-final-blur"),
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let composite_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bloom-composite-bg"),
                layout: &self.composite_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(hdr_input),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&bloom_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: intensity_buf.as_entire_binding(),
                    },
                ],
            });
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom-composite-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.composite_pipeline);
            rp.set_bind_group(0, &composite_bg, &[]);
            rp.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        Ok(output_view)
    }
}

// ---------------------------------------------------------------------------------
// Bloom WGSL Shaders
// ---------------------------------------------------------------------------------

const BLOOM_THRESHOLD_WGSL: &str = r#"
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
    let xy = p[vi];
    out.pos = vec4<f32>(xy, 0.0, 1.0);
    out.uv = (xy+1.0)*0.5;
    return out;
}
@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> u_threshold: vec4<f32>;
fn luminance(rgb: vec3<f32>) -> f32 { return dot(rgb, vec3<f32>(0.299, 0.587, 0.114)); }
@fragment fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let hdr = textureSample(input_tex, samp, in.uv).rgb;
    let lum = luminance(hdr);
    let threshold = u_threshold.x;
    if (lum > threshold) {
        return vec4<f32>(hdr * ((lum - threshold) / max(lum, 1e-4)), 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
"#;

const BLOOM_DOWNSAMPLE_WGSL: &str = r#"
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
    let xy = p[vi];
    out.pos = vec4<f32>(xy, 0.0, 1.0);
    out.uv = (xy+1.0)*0.5;
    return out;
}
@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@fragment fn fs(in: VsOut) -> @location(0) vec4<f32> {
    // 13-tap Karis average filter for better quality downsampling
    let dim = vec2<f32>(textureDimensions(input_tex));
    let texel = 1.0 / dim;
    var col = vec3<f32>(0.0);
    // Center
    col += textureSample(input_tex, samp, in.uv).rgb * 0.5;
    // 4 corners (half weight)
    col += textureSample(input_tex, samp, in.uv + vec2<f32>(-texel.x, -texel.y)).rgb * 0.125;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>( texel.x, -texel.y)).rgb * 0.125;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>(-texel.x,  texel.y)).rgb * 0.125;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>( texel.x,  texel.y)).rgb * 0.125;
    return vec4<f32>(col, 1.0);
}
"#;

const BLOOM_UPSAMPLE_WGSL: &str = r#"
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
    let xy = p[vi];
    out.pos = vec4<f32>(xy, 0.0, 1.0);
    out.uv = (xy+1.0)*0.5;
    return out;
}
@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@fragment fn fs(in: VsOut) -> @location(0) vec4<f32> {
    // Tent filter upsample (3x3 bilinear)
    let dim = vec2<f32>(textureDimensions(input_tex));
    let texel = 1.0 / dim;
    var col = vec3<f32>(0.0);
    // 9-tap tent filter
    col += textureSample(input_tex, samp, in.uv + vec2<f32>(-texel.x, -texel.y)).rgb * 0.0625;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>( 0.0,     -texel.y)).rgb * 0.125;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>( texel.x, -texel.y)).rgb * 0.0625;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>(-texel.x,  0.0    )).rgb * 0.125;
    col += textureSample(input_tex, samp, in.uv                              ).rgb * 0.25;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>( texel.x,  0.0    )).rgb * 0.125;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>(-texel.x,  texel.y)).rgb * 0.0625;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>( 0.0,      texel.y)).rgb * 0.125;
    col += textureSample(input_tex, samp, in.uv + vec2<f32>( texel.x,  texel.y)).rgb * 0.0625;
    return vec4<f32>(col, 1.0);
}
"#;

const BLOOM_COMPOSITE_WGSL: &str = r#"
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0,-1.0), vec2<f32>(3.0,-1.0), vec2<f32>(-1.0,3.0));
    let xy = p[vi];
    out.pos = vec4<f32>(xy, 0.0, 1.0);
    out.uv = (xy+1.0)*0.5;
    return out;
}
@group(0) @binding(0) var original_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> u_intensity: vec4<f32>;
@fragment fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let original = textureSample(original_tex, samp, in.uv).rgb;
    let bloom = textureSample(bloom_tex, samp, in.uv).rgb;
    let intensity = u_intensity.x;
    let result = original + bloom * intensity;
    return vec4<f32>(result, 1.0);
}
"#;

// ---------------------------------------------------------------------------------
// Legacy placeholder shaders (SSR, SSAO, SSGI)
// ---------------------------------------------------------------------------------

pub const WGSL_SSR: &str = r#"
// Screen-space reflections placeholder
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + vec2<f32>(1.0,1.0)) * 0.5;
    return out;
}

@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var samp: sampler;

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let col = textureSampleLevel(color_tex, samp, in.uv, 0.0);
    let _d = textureLoad(depth_tex, vec2<i32>(i32(in.uv.x), i32(in.uv.y)), 0);
    // Placeholder: just passthrough
    return vec4<f32>(col.rgb, 1.0);
}
"#;

pub const WGSL_SSAO: &str = r#"
// Screen-space ambient occlusion placeholder
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + vec2<f32>(1.0,1.0)) * 0.5;
    return out;
}

@group(0) @binding(0) var depth_tex: texture_depth_2d;
@group(0) @binding(1) var samp: sampler;

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // Placeholder: flat gray AO factor
    let ao = 0.2;
    return vec4<f32>(ao, ao, ao, 1.0);
}
"#;

pub const WGSL_SSGI: &str = r#"
// Screen-space global illumination placeholder
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + vec2<f32>(1.0,1.0)) * 0.5;
    return out;
}

@group(0) @binding(0) var normal_tex: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var samp: sampler;

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // Placeholder: tint by normals if provided; else white
    let nrm = textureSampleLevel(normal_tex, samp, in.uv, 0.0).xyz;
    return vec4<f32>(normalize(nrm), 1.0);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------------
    // Bloom tests
    // ---------------------------------------------------------------------------------

    #[test]
    fn bloom_config_default() {
        let config = BloomConfig::default();
        assert_eq!(config.threshold, 1.0);
        assert_eq!(config.intensity, 0.05);
        assert_eq!(config.mip_count, 5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn bloom_config_validate_threshold() {
        let mut config = BloomConfig::default();
        config.threshold = 11.0; // Above max
        assert!(config.validate().is_err());

        config.threshold = -1.0; // Below min
        assert!(config.validate().is_err());

        config.threshold = 2.5; // Valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn bloom_config_validate_intensity() {
        let mut config = BloomConfig::default();
        config.intensity = 1.5; // Above max
        assert!(config.validate().is_err());

        config.intensity = -0.1; // Below min
        assert!(config.validate().is_err());

        config.intensity = 0.15; // Valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn bloom_config_validate_mip_count() {
        let mut config = BloomConfig::default();
        config.mip_count = 0; // Below min
        assert!(config.validate().is_err());

        config.mip_count = 9; // Above max
        assert!(config.validate().is_err());

        config.mip_count = 6; // Valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn parse_bloom_threshold() {
        let module = naga::front::wgsl::parse_str(BLOOM_THRESHOLD_WGSL)
            .expect("WGSL bloom threshold should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "fs"));
        assert!(module.entry_points.iter().any(|e| e.name == "vs"));
    }

    #[test]
    fn parse_bloom_downsample() {
        let module = naga::front::wgsl::parse_str(BLOOM_DOWNSAMPLE_WGSL)
            .expect("WGSL bloom downsample should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "fs"));
    }

    #[test]
    fn parse_bloom_upsample() {
        let module = naga::front::wgsl::parse_str(BLOOM_UPSAMPLE_WGSL)
            .expect("WGSL bloom upsample should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "fs"));
    }

    #[test]
    fn parse_bloom_composite() {
        let module = naga::front::wgsl::parse_str(BLOOM_COMPOSITE_WGSL)
            .expect("WGSL bloom composite should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "fs"));
    }

    // ---------------------------------------------------------------------------------
    // Legacy placeholder shader tests
    // ---------------------------------------------------------------------------------

    #[test]
    fn parse_ssr() {
        let src = WGSL_SSR;
        let module = naga::front::wgsl::parse_str(src).expect("WGSL SSR should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "fs_main"));
    }
    #[test]
    fn parse_ssao() {
        let src = WGSL_SSAO;
        let module = naga::front::wgsl::parse_str(src).expect("WGSL SSAO should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "fs_main"));
    }
    #[test]
    fn parse_ssgi() {
        let src = WGSL_SSGI;
        let module = naga::front::wgsl::parse_str(src).expect("WGSL SSGI should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "fs_main"));
    }

    #[test]
    fn ssr_bindings_declared() {
        assert!(WGSL_SSR.contains("@group(0) @binding(0) var color_tex"));
        assert!(WGSL_SSR.contains("@group(0) @binding(1) var depth_tex"));
        assert!(WGSL_SSR.contains("@group(0) @binding(2) var samp"));
    }

    #[test]
    fn ssgi_bindings_declared() {
        assert!(WGSL_SSGI.contains("@group(0) @binding(0) var normal_tex"));
        assert!(WGSL_SSGI.contains("@group(0) @binding(1) var depth_tex"));
        assert!(WGSL_SSGI.contains("@group(0) @binding(2) var samp"));
    }
}
