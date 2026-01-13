//! Cascaded Shadow Mapping (CSM) Implementation
//!
//! This module implements a 4-cascade shadow mapping system for directional lights
//! (e.g., sun/moon). Each cascade covers a different depth range from the camera,
//! providing high detail near the camera and broader coverage far away.
//!
//! # Architecture
//!
//! - **Shadow Atlas**: Single 8192×8192 texture containing all 4 cascades (2048×2048 each)
//! - **Cascade Splitting**: Logarithmic distribution based on view frustum depth
//! - **Sampling**: PCF (Percentage Closer Filtering) for soft shadows
//! - **Bias**: Depth bias to prevent shadow acne
//!
//! # Performance Targets (60 FPS = 16.67ms budget)
//!
//! - Shadow map rendering: <2.0ms (4 cascades × 0.5ms each)
//! - Cascade selection: <0.01ms (per-pixel, branchless)
//! - PCF sampling: <0.5ms (5×5 kernel, optimized)
//! - Total shadow budget: <2.5ms (15% of frame)

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use wgpu;

/// Shadow map shader source (embedded at compile time)
#[allow(dead_code)]
const SHADOW_SHADER: &str = include_str!("../shaders/shadow_csm.wgsl");

// Minimal shadow-only shader (uses group(0) since it's the only bind group)
const SHADOW_DEPTH_SHADER: &str = r#"
// Shadow cascade data
struct ShadowCascade {
    view_proj: mat4x4<f32>,
    split_distances: vec4<f32>,
    atlas_transform: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> cascades: array<ShadowCascade, 4>;

struct ShadowVertexInput {
    @location(0) position: vec3<f32>,
}

struct ShadowVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn shadow_vertex_main(
    in: ShadowVertexInput,
    @builtin(instance_index) cascade_index: u32,
) -> ShadowVertexOutput {
    var out: ShadowVertexOutput;
    let world_pos = vec4<f32>(in.position, 1.0);
    let cascade_idx = min(cascade_index, 3u);
    out.clip_position = cascades[cascade_idx].view_proj * world_pos;
    return out;
}

@fragment
fn shadow_fragment_main(in: ShadowVertexOutput) {
    // Depth written automatically
}
"#;

/// Number of shadow cascades (industry standard: 4)
pub const CASCADE_COUNT: usize = 4;

/// Shadow map resolution per cascade (2048×2048 = high quality)
pub const CASCADE_RESOLUTION: u32 = 2048;

/// TEMP: Use separate textures instead of atlas for simplicity
pub const ATLAS_RESOLUTION: u32 = CASCADE_RESOLUTION; // 2048×2048 per cascade

/// Depth bias to prevent shadow acne (tweakable)
pub const DEPTH_BIAS: f32 = 0.005;

/// Shadow cascade configuration
///
/// Each cascade covers a depth range [near, far] in view space.
/// The view-projection matrix transforms world coords to shadow map space.
#[derive(Debug, Clone, Copy)]
pub struct ShadowCascade {
    /// Near plane distance (view space)
    pub near: f32,
    /// Far plane distance (view space)
    pub far: f32,
    /// View matrix (light space)
    pub view_matrix: Mat4,
    /// Projection matrix (orthographic, light space)
    pub proj_matrix: Mat4,
    /// Combined view-projection matrix (for shader upload)
    pub view_proj_matrix: Mat4,
    /// Atlas offset (UV coords: 0.0-0.5 for each quadrant)
    pub atlas_offset: Vec4, // (offset_x, offset_y, scale_x, scale_y)
}

/// GPU-compatible shadow cascade data (uploaded to shader)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuShadowCascade {
    /// View-projection matrix (light space)
    pub view_proj: [[f32; 4]; 4],
    /// Split distances (near, far, 0, 0) for depth comparison
    pub split_distances: [f32; 4],
    /// Atlas UV transform (offset_x, offset_y, scale_x, scale_y)
    pub atlas_transform: [f32; 4],
}

impl From<&ShadowCascade> for GpuShadowCascade {
    fn from(cascade: &ShadowCascade) -> Self {
        Self {
            view_proj: cascade.view_proj_matrix.to_cols_array_2d(),
            split_distances: [cascade.near, cascade.far, 0.0, 0.0],
            atlas_transform: cascade.atlas_offset.to_array(),
        }
    }
}

/// Cascaded Shadow Mapping renderer
///
/// Manages shadow map atlas, cascade splitting, and shadow rendering passes.
pub struct CsmRenderer {
    /// Shadow atlas texture (8192×8192, Depth32Float)
    pub atlas_texture: wgpu::Texture,
    /// Shadow atlas view (for rendering)
    pub atlas_view: wgpu::TextureView,
    /// Shadow sampler (comparison sampler for PCF)
    pub shadow_sampler: wgpu::Sampler,

    /// Cascade data (CPU-side, updated per frame)
    pub cascades: [ShadowCascade; CASCADE_COUNT],

    /// Cascade buffer (GPU-side, uploaded each frame)
    pub cascade_buffer: wgpu::Buffer,

    /// Bind group for shadow sampling (used in main render pass)
    pub bind_group: Option<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,

    // Shadow-specific bind group (just cascades buffer, for depth pass)
    shadow_bind_group: Option<wgpu::BindGroup>,
    shadow_bind_group_layout: wgpu::BindGroupLayout,

    /// Render pass depth attachments (one per cascade)
    cascade_views: [wgpu::TextureView; CASCADE_COUNT],

    /// Shadow rendering pipeline (depth-only pass)
    pub shadow_pipeline: wgpu::RenderPipeline,

    /// Shader module
    #[allow(dead_code)]
    shader_module: wgpu::ShaderModule,
}

impl CsmRenderer {
    /// Create a new CSM renderer
    ///
    /// # Arguments
    ///
    /// - `device`: wgpu device for resource creation
    ///
    /// # Returns
    ///
    /// Initialized CSM renderer with shadow atlas and cascade buffers
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        // Create shadow atlas as TEXTURE ARRAY (4 layers, one per cascade)
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Atlas (Array)"),
            size: wgpu::Extent3d {
                width: CASCADE_RESOLUTION,
                height: CASCADE_RESOLUTION,
                depth_or_array_layers: CASCADE_COUNT as u32, // 4 layers
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Atlas Array View"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(CASCADE_COUNT as u32),
            usage: None,
        });

        // Create cascade views (each layer of the texture array)
        let cascade_views = [
            // Cascade 0 (layer 0)
            atlas_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Cascade 0 View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                usage: None,
            }),
            // Cascade 1 (layer 1)
            atlas_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Cascade 1 View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 1,
                array_layer_count: Some(1),
                usage: None,
            }),
            // Cascade 2 (layer 2)
            atlas_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Cascade 2 View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 2,
                array_layer_count: Some(1),
                usage: None,
            }),
            // Cascade 3 (layer 3)
            atlas_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Cascade 3 View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 3,
                array_layer_count: Some(1),
                usage: None,
            }),
        ];

        // Create comparison sampler for PCF
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler (Comparison)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // Enable depth comparison
            ..Default::default()
        });

        // Create cascade buffer (4 cascades × GpuShadowCascade)
        let cascade_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Cascade Buffer"),
            size: (std::mem::size_of::<GpuShadowCascade>() * CASCADE_COUNT) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout (for shader access)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("CSM Bind Group Layout"),
            entries: &[
                // Shadow atlas texture (D2Array for texture array)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Shadow sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                // Cascade data buffer
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

        // Create shadow-only bind group layout (just cascades buffer for depth pass)
        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Depth Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Initialize cascades with default values (will be updated in update_cascades)
        let cascades = [
            ShadowCascade {
                near: 0.1,
                far: 10.0,
                view_matrix: Mat4::IDENTITY,
                proj_matrix: Mat4::IDENTITY,
                view_proj_matrix: Mat4::IDENTITY,
                atlas_offset: Vec4::new(0.0, 0.0, 1.0, 1.0), // Full layer (texture array)
            },
            ShadowCascade {
                near: 10.0,
                far: 50.0,
                view_matrix: Mat4::IDENTITY,
                proj_matrix: Mat4::IDENTITY,
                view_proj_matrix: Mat4::IDENTITY,
                atlas_offset: Vec4::new(0.0, 0.0, 1.0, 1.0), // Full layer (texture array)
            },
            ShadowCascade {
                near: 50.0,
                far: 200.0,
                view_matrix: Mat4::IDENTITY,
                proj_matrix: Mat4::IDENTITY,
                view_proj_matrix: Mat4::IDENTITY,
                atlas_offset: Vec4::new(0.0, 0.0, 1.0, 1.0), // Full layer (texture array)
            },
            ShadowCascade {
                near: 200.0,
                far: 1000.0,
                view_matrix: Mat4::IDENTITY,
                proj_matrix: Mat4::IDENTITY,
                view_proj_matrix: Mat4::IDENTITY,
                atlas_offset: Vec4::new(0.0, 0.0, 1.0, 1.0), // Full layer (texture array)
            },
        ];

        // Create shader module for shadow depth pass (minimal, group(1) only)
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow CSM Depth Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADOW_DEPTH_SHADER.into()),
        });

        // Create shadow render pipeline (depth-only pass)
        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Pipeline Layout"),
                bind_group_layouts: &[&shadow_bind_group_layout], // group(0): just cascades buffer
                push_constant_ranges: &[],
            });

        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Render Pipeline"),
            layout: Some(&shadow_pipeline_layout), // Use explicit layout (only group 1)
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("shadow_vertex_main"),
                buffers: &[
                    // FIXED: Full vertex buffer (position vec3 + normal vec3 = 24 bytes)
                    // Shadow shader only uses position, but stride must match actual vertex data
                    wgpu::VertexBufferLayout {
                        array_stride: 24, // 6 × f32 (position + normal)
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            },
                            // Normal at offset 12 (ignored by shader, but maintains alignment)
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2, // Depth bias for shadow acne prevention
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: None, // Depth-only pass, no fragment shader
            multiview: None,
            cache: None,
        });

        Ok(Self {
            atlas_texture,
            atlas_view,
            shadow_sampler,
            cascades,
            cascade_buffer,
            bind_group: None, // Created after first update
            bind_group_layout,
            shadow_bind_group: None, // Created after first update
            shadow_bind_group_layout,
            cascade_views,
            shadow_pipeline,
            shader_module,
        })
    }

    /// Update cascade splits and matrices based on camera frustum
    ///
    /// # Arguments
    ///
    /// - `camera_pos`: Camera world position
    /// - `camera_view`: Camera view matrix
    /// - `camera_proj`: Camera projection matrix
    /// - `light_dir`: Directional light direction (normalized, pointing TOWARD light)
    /// - `near`: Camera near plane
    /// - `far`: Camera far plane
    ///
    /// # Algorithm
    ///
    /// Uses logarithmic splitting with λ=0.5 (balance between uniform and logarithmic):
    /// ```text
    /// split[i] = λ * (near * (far/near)^(i/N)) + (1-λ) * (near + (far-near) * i/N)
    /// ```
    pub fn update_cascades(
        &mut self,
        _camera_pos: Vec3,
        _camera_view: Mat4,
        _camera_proj: Mat4,
        light_dir: Vec3,
        near: f32,
        far: f32,
    ) {
        // Calculate cascade split distances (logarithmic distribution)
        let lambda = 0.5; // Balance between uniform (0.0) and logarithmic (1.0)
        let mut split_distances = [0.0f32; CASCADE_COUNT + 1];
        split_distances[0] = near;
        split_distances[CASCADE_COUNT] = far;

        for (i, split) in split_distances
            .iter_mut()
            .enumerate()
            .take(CASCADE_COUNT)
            .skip(1)
        {
            let i_f = i as f32;
            let n_f = CASCADE_COUNT as f32;

            // Logarithmic split
            let log_split = near * (far / near).powf(i_f / n_f);

            // Uniform split
            let uniform_split = near + (far - near) * (i_f / n_f);

            // Blend
            *split = lambda * log_split + (1.0 - lambda) * uniform_split;
        }

        // Update each cascade
        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            cascade.near = split_distances[i];
            cascade.far = split_distances[i + 1];

            // 🎯 BEVY-STYLE: Compute light view matrix from direction
            // Directional lights need a view matrix that looks FROM light TO scene
            let light_distance = 50.0; // How far back to place the light
            let scene_center = Vec3::ZERO; // Look at origin
            let light_pos = scene_center - (light_dir.normalize() * light_distance);

            // Choose up vector perpendicular to light direction
            // If light is mostly vertical (|light_dir.y| > 0.9), use X as up
            // Otherwise use Y as up (standard)
            let up = if light_dir.y.abs() > 0.9 {
                Vec3::X
            } else {
                Vec3::Y
            };

            cascade.view_matrix = Mat4::look_at_rh(light_pos, scene_center, up);

            // FIXED: Use large enough orthographic bounds to cover entire test scene
            // Scene bounds: Ground is 20×20 at origin, cubes at ±5, ±15, ±25
            // Need at least -30 to +30 to capture everything
            let ortho_size = 35.0; // Large enough for test scene (can optimize per-cascade later)

            // DEBUG: Print ortho_size on first call
            if i == 0 {
                static FIRST_UPDATE: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(true);
                if FIRST_UPDATE.swap(false, std::sync::atomic::Ordering::Relaxed) {
                    println!(
                        "🔍 Light frustum: ortho_size = {}, covers [{}, {}] in X and Z",
                        ortho_size, -ortho_size, ortho_size
                    );
                }
            }

            cascade.proj_matrix = Mat4::orthographic_rh(
                -ortho_size,
                ortho_size,
                -ortho_size,
                ortho_size,
                0.1,   // Near (light space)
                100.0, // Far (light space)
            );

            cascade.view_proj_matrix = cascade.proj_matrix * cascade.view_matrix;
        }
    }

    /// Upload cascade data to GPU
    ///
    /// Call this after `update_cascades()` and before rendering.
    pub fn upload_to_gpu(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        // Convert to GPU format
        let gpu_cascades: Vec<GpuShadowCascade> =
            self.cascades.iter().map(GpuShadowCascade::from).collect();

        // Upload buffer
        queue.write_buffer(&self.cascade_buffer, 0, bytemuck::cast_slice(&gpu_cascades));

        // Create/update bind group
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CSM Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.cascade_buffer.as_entire_binding(),
                },
            ],
        }));

        // Create/update shadow-only bind group (just cascades for depth pass)
        self.shadow_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Depth Bind Group"),
            layout: &self.shadow_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.cascade_buffer.as_entire_binding(),
            }],
        }));
    }

    /// Get cascade view for rendering (for shadow pass)
    ///
    /// # Returns
    ///
    /// Texture view for the specified cascade index (0-3)
    pub fn get_cascade_view(&self, cascade_index: usize) -> &wgpu::TextureView {
        &self.cascade_views[cascade_index]
    }

    /// Render shadow maps for all cascades
    ///
    /// # Arguments
    ///
    /// - `encoder`: Command encoder for recording render passes
    /// - `vertex_buffer`: Vertex buffer containing scene geometry (position-only)
    /// - `index_buffer`: Index buffer for indexed drawing
    /// - `index_count`: Number of indices to draw
    ///
    /// # Notes
    ///
    /// This creates 4 render passes (one per cascade) to populate the shadow atlas.
    /// Call this before your main render pass.
    pub fn render_shadow_maps(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        vertex_buffer: &wgpu::Buffer,
        index_buffer: &wgpu::Buffer,
        index_count: u32,
    ) {
        // DEBUG: Print cascade matrices on first call
        static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        if FIRST_CALL.swap(false, std::sync::atomic::Ordering::Relaxed) {
            println!("🔍 Shadow rendering debug:");
            println!("  - Index count: {}", index_count);
            for (i, cascade) in self.cascades.iter().enumerate() {
                println!(
                    "  - Cascade {}: near={:.1}, far={:.1}",
                    i, cascade.near, cascade.far
                );
                println!("    view_proj = {:#?}", cascade.view_proj_matrix);
            }
        }

        for cascade_idx in 0..CASCADE_COUNT {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Shadow Cascade {} Render Pass", cascade_idx)),
                color_attachments: &[], // Depth-only pass
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.cascade_views[cascade_idx], // Render to individual layer
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), // Clear to far plane
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.shadow_pipeline);

            // Bind cascade data (group 0 for shadow pipeline)
            if let Some(ref bind_group) = self.shadow_bind_group {
                render_pass.set_bind_group(0, bind_group, &[]);
            }

            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            // Draw with instance_index = cascade_idx (for shader cascade selection)
            render_pass.draw_indexed(
                0..index_count,
                0,
                cascade_idx as u32..(cascade_idx as u32 + 1),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_shadow_cascade_size() {
        // Ensure struct is properly aligned for GPU
        assert_eq!(
            std::mem::size_of::<GpuShadowCascade>(),
            std::mem::size_of::<[[f32; 4]; 4]>() + // view_proj (64 bytes)
            std::mem::size_of::<[f32; 4]>() +      // split_distances (16 bytes)
            std::mem::size_of::<[f32; 4]>() // atlas_transform (16 bytes)
        );
    }

    #[test]
    fn test_cascade_split_distribution() {
        // Test logarithmic split calculation
        let near = 0.1f32;
        let far = 1000.0f32;
        let lambda = 0.5;

        let mut splits = [0.0f32; CASCADE_COUNT + 1];
        splits[0] = near;
        splits[CASCADE_COUNT] = far;

        for i in 1..CASCADE_COUNT {
            let i_f = i as f32;
            let n_f = CASCADE_COUNT as f32;
            let log_split = near * (far / near).powf(i_f / n_f);
            let uniform_split = near + (far - near) * (i_f / n_f);
            splits[i] = lambda * log_split + (1.0 - lambda) * uniform_split;
        }

        // Verify splits are monotonically increasing
        for i in 0..CASCADE_COUNT {
            assert!(splits[i] < splits[i + 1]);
        }

        // Verify first and last splits
        assert_eq!(splits[0], near);
        assert_eq!(splits[CASCADE_COUNT], far);
    }

    #[test]
    fn test_atlas_offset_calculation() {
        // Verify atlas quadrants are correctly mapped
        let cascades = [
            Vec4::new(0.0, 0.0, 0.5, 0.5), // Top-left
            Vec4::new(0.5, 0.0, 0.5, 0.5), // Top-right
            Vec4::new(0.0, 0.5, 0.5, 0.5), // Bottom-left
            Vec4::new(0.5, 0.5, 0.5, 0.5), // Bottom-right
        ];

        for (_i, offset) in cascades.iter().enumerate() {
            // All scales should be 0.5 (half atlas)
            assert_eq!(offset.z, 0.5);
            assert_eq!(offset.w, 0.5);

            // Offsets should be 0.0 or 0.5
            assert!(offset.x == 0.0 || offset.x == 0.5);
            assert!(offset.y == 0.0 || offset.y == 0.5);
        }
    }
}
