//! Nanite rendering integration with the clustered forward renderer
//!
//! This module integrates the meshlet-based rendering system with the existing
//! clustered forward renderer, material system, and global illumination.

use crate::nanite_visibility::{
    Frustum, GpuMeshlet, LODSelector, MeshletRenderer, VisibilityBuffer,
};
use crate::types::Instance;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Nanite rendering context
pub struct NaniteRenderContext {
    /// Meshlet renderer
    pub meshlet_renderer: MeshletRenderer,

    /// LOD selector
    pub lod_selector: LODSelector,

    /// Material resolve pipeline (reads visibility buffer and applies materials)
    pub material_pipeline: wgpu::RenderPipeline,
    pub material_bind_group_layout: wgpu::BindGroupLayout,

    /// Camera uniform buffer
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: f32,
}

impl NaniteRenderContext {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        fov: f32,
        meshlets: &[GpuMeshlet],
        vertices: &[u8],
        indices: &[u8],
        output_format: wgpu::TextureFormat,
    ) -> Self {
        // Create meshlet renderer
        let meshlet_renderer =
            MeshletRenderer::new(device, width, height, meshlets, vertices, indices);

        // Create LOD selector
        let lod_selector = LODSelector::new(height as f32, fov);

        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Nanite Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Nanite Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Nanite Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create material resolve bind group layout
        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Nanite Material Bind Group Layout"),
                entries: &[
                    // Visibility buffer texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Meshlet data
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create material resolve shader
        let material_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Nanite Material Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/nanite_material.wgsl").into()),
        });

        // Create material resolve pipeline
        let material_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Nanite Material Pipeline Layout"),
                bind_group_layouts: &[&material_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let material_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Nanite Material Pipeline"),
            layout: Some(&material_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &material_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &material_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Equal,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            meshlet_renderer,
            lod_selector,
            material_pipeline,
            material_bind_group_layout,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        }
    }

    /// Update camera uniform buffer
    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: Mat4, position: Vec3) {
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            position: position.to_array(),
            _padding: 0.0,
        };
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    /// Render meshlets to visibility buffer
    pub fn render_visibility_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        meshlets: &[GpuMeshlet],
        view_proj: Mat4,
        camera_pos: Vec3,
    ) {
        // Perform CPU-side culling
        let frustum = Frustum::from_matrix(view_proj);
        let visible_meshlets = self
            .meshlet_renderer
            .cull_meshlets(meshlets, &frustum, camera_pos);

        if visible_meshlets.is_empty() {
            return;
        }

        // Render visible meshlets
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Nanite Visibility Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.meshlet_renderer.visibility_buffer.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.meshlet_renderer.visibility_buffer.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.meshlet_renderer.pipeline);
        render_pass.set_bind_group(0, &self.meshlet_renderer.bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        // Draw visible meshlets
        for &meshlet_id in &visible_meshlets {
            let meshlet = &meshlets[meshlet_id as usize];
            let vertex_count = meshlet.triangle_count * 3;
            render_pass.draw(0..vertex_count as u32, meshlet_id..meshlet_id + 1);
        }
    }

    /// Resolve visibility buffer and apply materials
    pub fn render_material_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        output_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        // Create bind group for material resolve
        let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Nanite Material Resolve Bind Group"),
            layout: &self.material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.meshlet_renderer.visibility_buffer.view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.meshlet_renderer.meshlet_buffer.as_entire_binding(),
                },
            ],
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Nanite Material Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.material_pipeline);
        render_pass.set_bind_group(0, &material_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        // Draw fullscreen quad to resolve materials
        render_pass.draw(0..3, 0..1);
    }

    /// Resize the rendering context
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.meshlet_renderer
            .visibility_buffer
            .resize(device, width, height);
        self.lod_selector.screen_height = height as f32;
    }
}

/// Helper function to convert mesh data to GPU meshlet format
pub fn convert_to_gpu_meshlets(
    meshlets: &[crate::nanite_visibility::GpuMeshlet],
) -> Vec<GpuMeshlet> {
    meshlets.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_uniform_size() {
        assert_eq!(
            std::mem::size_of::<CameraUniform>(),
            80 // 64 bytes for mat4x4 + 12 bytes for vec3 + 4 bytes padding
        );
    }
}
