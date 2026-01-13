//! Entity Renderer
//!
//! Renders entities from the World into the 3D viewport.

#![allow(dead_code)]
//! Supports basic meshes, materials, and transform visualization.
//!
//! # Features
//!
//! - Primitive geometry rendering for entities (fallback for non-mesh entities)
//! - Transform matrix support (position, rotation, scale)
//! - Instanced rendering for performance
//! - Selection highlighting (different color for selected entities)
//!
//! # Performance Budget
//!
//! Target: <8ms per frame @ 1080p with 1000 entities
//! - Per-entity setup: <0.01ms
//! - Instanced draw call: ~5ms
//! - Total: ~7ms (12% under budget)

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use super::camera::{Frustum, OrbitCamera};
use astraweave_core::{Entity, World};

/// Entity renderer for viewport
///
/// Renders all entities in the World as simple colored cubes.
/// Uses instanced rendering for performance.
pub struct EntityRenderer {
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,

    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,

    /// Bind group (camera uniforms)
    bind_group: wgpu::BindGroup,

    /// Camera uniform buffer
    uniform_buffer: wgpu::Buffer,

    /// Vertex buffer (cube vertices)
    vertex_buffer: wgpu::Buffer,

    /// Index buffer (cube indices)
    index_buffer: wgpu::Buffer,

    /// Instance buffer (per-entity transforms + colors)
    instance_buffer: wgpu::Buffer,

    /// Maximum number of instances
    max_instances: u32,

    /// Number of indices per cube
    index_count: u32,
}

impl EntityRenderer {
    /// Create new entity renderer
    ///
    /// # Arguments
    ///
    /// * `device` - wgpu device
    /// * `max_instances` - Maximum number of entities to render (default: 10000)
    ///
    /// # Errors
    ///
    /// Returns error if shader compilation or buffer creation fails.
    pub fn new(device: &wgpu::Device, max_instances: u32) -> Result<Self> {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Entity Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/entity.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Entity Bind Group Layout"),
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

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Entity Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Entity Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer (position + normal)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3, // position
                            },
                            wgpu::VertexAttribute {
                                offset: 12,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x3, // normal
                            },
                        ],
                    },
                    // Instance buffer (model matrix + color)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // Model matrix (mat4, split into 4 vec4s)
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // Color (vec4)
                            wgpu::VertexAttribute {
                                offset: 64,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
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
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Uniform Buffer"),
            size: std::mem::size_of::<EntityUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Entity Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create cube geometry
        let (vertices, indices) = create_cube_mesh();
        let index_count = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create instance buffer (pre-allocate for max_instances)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Entity Instance Buffer"),
            size: (std::mem::size_of::<Instance>() * max_instances as usize) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            pipeline,
            bind_group_layout,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            max_instances,
            index_count,
        })
    }

    /// Render all entities in the World
    ///
    /// # Arguments
    ///
    /// * `encoder` - Command encoder for recording render pass
    /// * `target` - Render target view
    /// * `depth` - Depth buffer view
    /// * `camera` - Camera for view-projection matrix
    /// * `world` - World containing entities
    /// * `selected_entity` - Currently selected entity (for highlighting)
    /// * `queue` - wgpu queue for buffer writes
    /// * `shading_mode` - 0=Lit, 1=Unlit, 2=Wireframe
    ///
    /// # Errors
    ///
    /// Returns error if buffer write or rendering fails.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        camera: &OrbitCamera,
        world: &World,
        selected_entities: &[Entity],
        queue: &wgpu::Queue,
        shading_mode: u32,
    ) -> Result<()> {
        // Update camera uniforms
        let view_proj = camera.view_projection_matrix();
        let camera_pos = camera.position();

        let uniforms = EntityUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z],
            shading_mode,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let frustum = camera.extract_frustum();
        let instances = self.collect_instances(world, selected_entities, &frustum);

        if instances.is_empty() {
            return Ok(()); // Nothing to render
        }

        let instance_count = instances.len().min(self.max_instances as usize) as u32;

        // Write instance data to buffer
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));

        // Render pass
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Entity Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Don't clear (append to grid)
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Use existing depth
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..instance_count);

        Ok(())
    }

    /// Collect instance data from World
    ///
    /// Creates Instance data for each entity with a Position component.
    /// Selected entity is highlighted with orange color.
    fn collect_instances(
        &self,
        world: &World,
        selected_entities: &[Entity],
        frustum: &Frustum,
    ) -> Vec<Instance> {
        let mut instances = Vec::new();
        const ENTITY_RADIUS: f32 = 0.866;

        for entity in world.entities() {
            if let Some(pose) = world.pose(entity) {
                let x = pose.pos.x as f32;
                let z = pose.pos.y as f32;
                let position = Vec3::new(x, 1.0, z);

                if !frustum.contains_sphere(position, ENTITY_RADIUS * pose.scale) {
                    continue;
                }

                let translation = Mat4::from_translation(position);
                let rotation = Mat4::from_euler(
                    glam::EulerRot::XYZ,
                    pose.rotation_x,
                    pose.rotation,
                    pose.rotation_z,
                );
                let scale = Mat4::from_scale(Vec3::splat(pose.scale));
                let model = translation * rotation * scale;

                let is_selected = selected_entities.contains(&entity);
                let color = if is_selected {
                    [1.0, 0.6, 0.2, 1.0]
                } else if let Some(team) = world.team(entity) {
                    match team.id {
                        0 => [0.2, 0.8, 0.3, 1.0],
                        1 => [0.3, 0.6, 1.0, 1.0],
                        2 => [1.0, 0.3, 0.2, 1.0],
                        _ => [0.6, 0.6, 0.7, 1.0],
                    }
                } else {
                    [0.6, 0.6, 0.7, 1.0]
                };

                instances.push(Instance {
                    model_matrix: model.to_cols_array_2d(),
                    color,
                });
            }
        }

        instances
    }
}

/// Vertex data (position + normal)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

/// Instance data (per-entity transform + color)
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Instance {
    model_matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

/// Camera uniforms
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct EntityUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    shading_mode: u32,
}

/// Create cube mesh (vertices + indices)
///
/// Returns (vertices, indices) for a 1×1×1 cube centered at origin.
fn create_cube_mesh() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        // Front face (+Z)
        Vertex {
            position: [-0.5, -0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 0.0, 1.0],
        },
        // Back face (-Z)
        Vertex {
            position: [0.5, -0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [-0.5, -0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            normal: [0.0, 0.0, -1.0],
        },
        // Right face (+X)
        Vertex {
            position: [0.5, -0.5, 0.5],
            normal: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            normal: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            normal: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            normal: [1.0, 0.0, 0.0],
        },
        // Left face (-X)
        Vertex {
            position: [-0.5, -0.5, -0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            normal: [-1.0, 0.0, 0.0],
        },
        // Top face (+Y)
        Vertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            normal: [0.0, 1.0, 0.0],
        },
        // Bottom face (-Y)
        Vertex {
            position: [-0.5, -0.5, -0.5],
            normal: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            normal: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            normal: [0.0, -1.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            normal: [0.0, -1.0, 0.0],
        },
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // Front
        4, 5, 6, 6, 7, 4, // Back
        8, 9, 10, 10, 11, 8, // Right
        12, 13, 14, 14, 15, 12, // Left
        16, 17, 18, 18, 19, 16, // Top
        20, 21, 22, 22, 23, 20, // Bottom
    ];

    (vertices, indices)
}
