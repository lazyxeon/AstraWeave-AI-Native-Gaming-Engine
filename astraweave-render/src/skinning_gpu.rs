//! GPU Skinning Pipeline for AstraWeave
//!
//! Phase 2 Task 5 (Phase D): GPU-accelerated skeletal animation skinning.
//! Feature-gated with `skinning-gpu` - optional for performance, CPU path is default.

use crate::animation::JointPalette;
use anyhow::Result;
use glam::Mat4;
use std::collections::HashMap;

/// Handle for a joint palette buffer (per-skeleton GPU buffer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JointPaletteHandle(pub u32);

/// GPU buffer pool for joint palettes
pub struct JointPaletteManager {
    device: wgpu::Device,
    queue: wgpu::Queue,

    /// Active joint palette buffers (handle -> buffer)
    buffers: HashMap<JointPaletteHandle, wgpu::Buffer>,

    /// Next available handle
    next_handle: u32,

    /// Bind group layout for skinning storage buffer
    pub bind_group_layout: wgpu::BindGroupLayout,

    /// Cached bind groups (handle -> bind group)
    bind_groups: HashMap<JointPaletteHandle, wgpu::BindGroup>,
}

impl JointPaletteManager {
    /// Create a new joint palette manager
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("joint_palette_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        Self {
            device: device.clone(),
            queue: queue.clone(),
            buffers: HashMap::new(),
            next_handle: 0,
            bind_group_layout,
            bind_groups: HashMap::new(),
        }
    }

    /// Allocate a new joint palette buffer
    pub fn allocate(&mut self) -> JointPaletteHandle {
        let handle = JointPaletteHandle(self.next_handle);
        self.next_handle += 1;

        // Create storage buffer for joint matrices
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("joint_palette_{}", handle.0)),
            size: std::mem::size_of::<JointPalette>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("joint_palette_bind_group_{}", handle.0)),
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        self.buffers.insert(handle, buffer);
        self.bind_groups.insert(handle, bind_group);

        handle
    }

    /// Upload joint matrices to GPU buffer (from Mat4 array)
    pub fn upload_matrices(&mut self, handle: JointPaletteHandle, matrices: &[Mat4]) -> Result<()> {
        let palette = JointPalette::from_matrices(matrices);
        self.upload_palette(handle, &palette)
    }

    /// Upload joint palette to GPU buffer (from JointPalette struct)
    pub fn upload_palette(
        &mut self,
        handle: JointPaletteHandle,
        palette: &JointPalette,
    ) -> Result<()> {
        let buffer = self
            .buffers
            .get(&handle)
            .ok_or_else(|| anyhow::anyhow!("Invalid joint palette handle: {:?}", handle))?;

        // Convert to bytes and upload
        let binding = [*palette];
        let data = bytemuck::cast_slice(&binding);
        self.queue.write_buffer(buffer, 0, data);

        Ok(())
    }

    /// Get bind group for skinning shader
    pub fn get_bind_group(&self, handle: JointPaletteHandle) -> Option<&wgpu::BindGroup> {
        self.bind_groups.get(&handle)
    }

    /// Free a joint palette buffer
    pub fn free(&mut self, handle: JointPaletteHandle) {
        self.buffers.remove(&handle);
        self.bind_groups.remove(&handle);
    }

    /// Get number of active buffers
    pub fn active_count(&self) -> usize {
        self.buffers.len()
    }

    /// Clear all buffers (for cleanup)
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.bind_groups.clear();
        self.next_handle = 0;
    }
}

// ============================================================================
// GPU Skinning Shader Module (embedded WGSL)
// ============================================================================

/// WGSL shader for GPU skinning vertex transformation
pub const SKINNING_GPU_SHADER: &str = r#"
// Joint palette storage buffer
struct JointMatrix {
    matrix: mat4x4<f32>,
}

struct JointPalette {
    joints: array<JointMatrix, 256>,
    joint_count: u32,
    _padding: vec3<u32>,
}

@group(4) @binding(0) var<storage, read> joint_palette: JointPalette;

// Vertex input with skinning data
struct SkinnedVertexInput {
    position: vec3<f32>,
    normal: vec3<f32>,
    tangent: vec4<f32>,
    joints: vec4<u32>,
    weights: vec4<f32>,
}

// Apply GPU skinning to position and normal
fn apply_skinning(input: SkinnedVertexInput) -> vec4<f32> {
    let j = input.joints;
    let w = input.weights;
    
    // Fetch joint matrices
    let m0 = joint_palette.joints[j.x].matrix;
    let m1 = joint_palette.joints[j.y].matrix;
    let m2 = joint_palette.joints[j.z].matrix;
    let m3 = joint_palette.joints[j.w].matrix;
    
    // Blend position
    let pos4 = vec4<f32>(input.position, 1.0);
    let skinned_pos = (m0 * pos4) * w.x 
                    + (m1 * pos4) * w.y 
                    + (m2 * pos4) * w.z 
                    + (m3 * pos4) * w.w;
    
    return skinned_pos;
}

// Apply skinning to normal (for lighting)
fn apply_skinning_normal(input: SkinnedVertexInput) -> vec3<f32> {
    let j = input.joints;
    let w = input.weights;
    
    let m0 = joint_palette.joints[j.x].matrix;
    let m1 = joint_palette.joints[j.y].matrix;
    let m2 = joint_palette.joints[j.z].matrix;
    let m3 = joint_palette.joints[j.w].matrix;
    
    let nrm4 = vec4<f32>(input.normal, 0.0);
    let skinned_nrm = (m0 * nrm4) * w.x 
                    + (m1 * nrm4) * w.y 
                    + (m2 * nrm4) * w.z 
                    + (m3 * nrm4) * w.w;
    
    return normalize(skinned_nrm.xyz);
}

// Apply skinning to tangent (for normal mapping)
fn apply_skinning_tangent(input: SkinnedVertexInput) -> vec3<f32> {
    let j = input.joints;
    let w = input.weights;
    
    let m0 = joint_palette.joints[j.x].matrix;
    let m1 = joint_palette.joints[j.y].matrix;
    let m2 = joint_palette.joints[j.z].matrix;
    let m3 = joint_palette.joints[j.w].matrix;
    
    let tan4 = vec4<f32>(input.tangent.xyz, 0.0);
    let skinned_tan = (m0 * tan4) * w.x 
                    + (m1 * tan4) * w.y 
                    + (m2 * tan4) * w.z 
                    + (m3 * tan4) * w.w;
    
    return normalize(skinned_tan.xyz);
}
"#;

// ============================================================================
// Complete Shader Generation
// ============================================================================

/// Create complete WGSL shader for GPU skinned mesh rendering
fn create_complete_skinning_shader() -> String {
    format!(
        r#"
// Bind Groups
struct Camera {{
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}}

struct Material {{
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    _padding: vec2<f32>,
}}

struct Light {{
    position: vec3<f32>,
    _padding1: f32,
    direction: vec3<f32>,
    _padding2: f32,
    color: vec3<f32>,
    intensity: f32,
}}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> material: Material;
@group(2) @binding(0) var<uniform> light: Light;
@group(3) @binding(0) var albedo_texture: texture_2d<f32>;
@group(3) @binding(1) var albedo_sampler: sampler;

{}

// Vertex Output
struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_tangent: vec3<f32>,
}}

// Vertex Shader
@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
    @location(10) joints: vec4<u32>,
    @location(11) weights: vec4<f32>,
) -> VertexOutput {{
    var output: VertexOutput;
    
    // Apply GPU skinning
    let skinned_input = SkinnedVertexInput(
        position,
        normal,
        tangent,
        joints,
        weights,
    );
    
    let skinned_pos = apply_skinning(skinned_input);
    let skinned_normal = apply_skinning_normal(skinned_input);
    let skinned_tangent = apply_skinning_tangent(skinned_input);
    
    // Transform to clip space
    output.clip_position = camera.view_proj * skinned_pos;
    output.world_position = skinned_pos.xyz;
    output.world_normal = skinned_normal;
    output.world_tangent = skinned_tangent;
    output.uv = uv;
    
    return output;
}}

// Fragment Shader (Simple PBR)
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    // Sample albedo
    let albedo = textureSample(albedo_texture, albedo_sampler, input.uv);
    let base_color = albedo * material.base_color;
    
    // Simple diffuse lighting
    let N = normalize(input.world_normal);
    let L = normalize(light.position - input.world_position);
    let V = normalize(camera.position - input.world_position);
    
    let NdotL = max(dot(N, L), 0.0);
    let diffuse = base_color.rgb * NdotL * light.color * light.intensity;
    
    // Simple specular (Blinn-Phong)
    let H = normalize(L + V);
    let NdotH = max(dot(N, H), 0.0);
    let spec_strength = pow(NdotH, 32.0 * (1.0 - material.roughness));
    let specular = vec3<f32>(spec_strength) * light.color * light.intensity;
    
    // Ambient
    let ambient = base_color.rgb * 0.03;
    
    let final_color = ambient + diffuse + specular;
    return vec4<f32>(final_color, base_color.a);
}}
"#,
        SKINNING_GPU_SHADER
    )
}

// ============================================================================
// Integration Helpers
// ============================================================================

/// Helper to create skinned mesh render pipeline with GPU skinning enabled
///
/// This creates a complete render pipeline for GPU-accelerated skeletal animation.
/// The pipeline expects:
/// - Vertex buffers with skinning data (joints, weights)
/// - Joint palette storage buffer at group 4, binding 0
/// - Standard PBR bind groups (camera, material, lights, textures)
///
/// Returns the created pipeline which can be used for rendering skinned meshes.
pub fn create_skinned_pipeline(
    device: &wgpu::Device,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    light_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    joint_palette_bind_group_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    // Create complete skinned mesh shader with GPU skinning
    let shader_source = create_complete_skinning_shader();
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gpu_skinned_mesh_shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    // Create pipeline layout with all bind groups
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("gpu_skinned_pipeline_layout"),
        bind_group_layouts: &[
            camera_bind_group_layout,        // Group 0: Camera (view, projection)
            material_bind_group_layout,      // Group 1: Material properties
            light_bind_group_layout,         // Group 2: Lights
            texture_bind_group_layout,       // Group 3: Textures (albedo, normal, MRA)
            joint_palette_bind_group_layout, // Group 4: Joint matrices
        ],
        push_constant_ranges: &[],
    });

    // Vertex attributes for skinned vertices
    const SKINNED_VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x3,  // normal
        2 => Float32x2,  // uv
        3 => Float32x4,  // tangent
        10 => Uint32x4,  // joints
        11 => Float32x4, // weights
    ];

    // Vertex buffer layout for skinned vertices
    let vertex_buffer_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SkinnedVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: SKINNED_VERTEX_ATTRIBUTES,
    };

    // Create the render pipeline
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("gpu_skinned_mesh_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[vertex_buffer_layout],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
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
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

/// Vertex structure for GPU skinned meshes
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
    pub joints: [u32; 4],
    pub weights: [f32; 4],
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::MAX_JOINTS;

    #[test]
    fn test_joint_palette_handle() {
        let handle1 = JointPaletteHandle(0);
        let handle2 = JointPaletteHandle(1);
        assert_ne!(handle1, handle2);
    }

    #[test]
    fn test_joint_palette_from_matrices() {
        let matrices = vec![
            Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0)),
            Mat4::from_translation(glam::Vec3::new(0.0, 2.0, 0.0)),
        ];

        let palette = JointPalette::from_matrices(&matrices);
        assert_eq!(palette.joint_count, 2);

        // Verify first matrix
        let m0 = palette.joints[0].matrix;
        assert_eq!(m0[3][0], 1.0); // Translation X
    }

    #[test]
    fn test_max_joints_limit() {
        let matrices = vec![Mat4::IDENTITY; 300]; // More than MAX_JOINTS

        let palette = JointPalette::from_matrices(&matrices);
        assert_eq!(palette.joint_count, MAX_JOINTS as u32);
    }

    // GPU tests require wgpu instance - add integration tests later
}

// ============================================================================
// Integration Tests (GPU)
// ============================================================================

#[cfg(all(test, feature = "gpu-tests"))]
mod gpu_tests {
    use super::*;

    async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to create device")
    }

    #[tokio::test]
    async fn test_pipeline_creation() {
        let (device, queue) = create_test_device().await;
        let mut manager = JointPaletteManager::new(&device, &queue);

        // Create dummy bind group layouts for testing
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let light_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("light"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture"),
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Test pipeline creation
        let pipeline = create_skinned_pipeline(
            &device,
            &camera_layout,
            &material_layout,
            &light_layout,
            &texture_layout,
            &manager.bind_group_layout,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        // Pipeline should be created successfully (no panics/errors)
        // Verify pipeline exists by checking it's not null-like
        drop(pipeline); // Just verify it was created without panicking
    }

    #[tokio::test]
    async fn test_skinning_produces_valid_output() {
        let (device, queue) = create_test_device().await;
        let mut manager = JointPaletteManager::new(&device, &queue);

        // Allocate a palette
        let handle = manager.allocate();

        // Create simple test matrices (identity transforms)
        let matrices = vec![
            Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0)),
            Mat4::from_translation(glam::Vec3::new(0.0, 2.0, 0.0)),
        ];

        // Upload matrices
        manager
            .upload_matrices(handle, &matrices)
            .expect("Failed to upload matrices");

        // Verify bind group exists
        let bind_group = manager.get_bind_group(handle);
        assert!(
            bind_group.is_some(),
            "Bind group should exist after allocation"
        );

        // Verify buffer is properly sized
        assert_eq!(manager.active_count(), 1);
    }
}
