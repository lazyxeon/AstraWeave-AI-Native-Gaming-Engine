//! Nanite visibility buffer and culling system
//!
//! This module implements the visibility buffer approach for rendering meshlets,
//! including frustum culling, occlusion culling, and LOD selection.

use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;

/// Frustum planes for culling
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub planes: [Vec4; 6], // left, right, bottom, top, near, far
}

impl Frustum {
    /// Extract frustum planes from a view-projection matrix
    pub fn from_matrix(view_proj: Mat4) -> Self {
        let m = view_proj.to_cols_array_2d();
        
        // Extract planes (Gribb-Hartmann method)
        let planes = [
            // Left plane
            Vec4::new(
                m[0][3] + m[0][0],
                m[1][3] + m[1][0],
                m[2][3] + m[2][0],
                m[3][3] + m[3][0],
            ).normalize(),
            // Right plane
            Vec4::new(
                m[0][3] - m[0][0],
                m[1][3] - m[1][0],
                m[2][3] - m[2][0],
                m[3][3] - m[3][0],
            ).normalize(),
            // Bottom plane
            Vec4::new(
                m[0][3] + m[0][1],
                m[1][3] + m[1][1],
                m[2][3] + m[2][1],
                m[3][3] + m[3][1],
            ).normalize(),
            // Top plane
            Vec4::new(
                m[0][3] - m[0][1],
                m[1][3] - m[1][1],
                m[2][3] - m[2][1],
                m[3][3] - m[3][1],
            ).normalize(),
            // Near plane
            Vec4::new(
                m[0][3] + m[0][2],
                m[1][3] + m[1][2],
                m[2][3] + m[2][2],
                m[3][3] + m[3][2],
            ).normalize(),
            // Far plane
            Vec4::new(
                m[0][3] - m[0][2],
                m[1][3] - m[1][2],
                m[2][3] - m[2][2],
                m[3][3] - m[3][2],
            ).normalize(),
        ];

        Self { planes }
    }

    /// Test if an AABB is inside or intersecting the frustum
    pub fn test_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            let normal = Vec3::new(plane.x, plane.y, plane.z);
            let d = plane.w;

            // Find the positive vertex (furthest along plane normal)
            let p = Vec3::new(
                if normal.x >= 0.0 { max.x } else { min.x },
                if normal.y >= 0.0 { max.y } else { min.y },
                if normal.z >= 0.0 { max.z } else { min.z },
            );

            // If positive vertex is outside, AABB is outside
            if normal.dot(p) + d < 0.0 {
                return false;
            }
        }
        true
    }

    /// Test if a sphere is inside or intersecting the frustum
    pub fn test_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let normal = Vec3::new(plane.x, plane.y, plane.z);
            let d = plane.w;
            
            let distance = normal.dot(center) + d;
            if distance < -radius {
                return false;
            }
        }
        true
    }
}

/// LOD selection based on screen-space error
pub struct LODSelector {
    /// Screen height in pixels
    pub screen_height: f32,
    /// Vertical field of view in radians
    pub fov: f32,
    /// LOD bias (higher = prefer lower detail)
    pub lod_bias: f32,
}

impl LODSelector {
    pub fn new(screen_height: f32, fov: f32) -> Self {
        Self {
            screen_height,
            fov,
            lod_bias: 1.0,
        }
    }

    /// Select appropriate LOD level based on screen-space error
    pub fn select_lod(
        &self,
        bounds_center: Vec3,
        bounds_radius: f32,
        lod_error: f32,
        camera_pos: Vec3,
        max_lod: u32,
    ) -> u32 {
        let distance = (bounds_center - camera_pos).length();
        
        // Compute projected size in pixels
        let projected_size = self.compute_projected_size(bounds_radius, distance);
        
        // Compute screen-space error threshold
        let error_threshold = lod_error * self.lod_bias;
        
        // Select LOD based on projected size and error
        let lod = if projected_size < error_threshold {
            // Object is small on screen, use lower detail
            ((projected_size / error_threshold).log2().abs() as u32).min(max_lod)
        } else {
            0 // Use highest detail
        };

        lod
    }

    /// Compute projected size of an object in pixels
    fn compute_projected_size(&self, radius: f32, distance: f32) -> f32 {
        if distance <= 0.0 {
            return self.screen_height;
        }

        // Compute projected radius using perspective projection
        let half_fov = self.fov * 0.5;
        let projected_radius = (radius / distance) / half_fov.tan();
        projected_radius * self.screen_height * 0.5
    }
}

/// Visibility buffer for meshlet rendering
pub struct VisibilityBuffer {
    /// Visibility texture (stores meshlet ID and triangle ID)
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    
    /// Depth texture for Hi-Z occlusion culling
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    
    /// Width and height
    pub width: u32,
    pub height: u32,
}

impl VisibilityBuffer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        // Create visibility texture (R32Uint for meshlet/triangle IDs)
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Nanite Visibility Buffer"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Nanite Depth Buffer"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            depth_texture,
            depth_view,
            width,
            height,
        }
    }

    /// Resize the visibility buffer
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        *self = Self::new(device, width, height);
    }
}

/// GPU buffer for meshlet data
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMeshlet {
    /// Bounding box min
    pub bounds_min: [f32; 3],
    pub vertex_offset: u32,
    
    /// Bounding box max
    pub bounds_max: [f32; 3],
    pub vertex_count: u32,
    
    /// Bounding cone apex
    pub cone_apex: [f32; 3],
    pub triangle_offset: u32,
    
    /// Bounding cone axis
    pub cone_axis: [f32; 3],
    pub triangle_count: u32,
    
    /// Cone cutoff and LOD info
    pub cone_cutoff: f32,
    pub lod_level: u32,
    pub lod_error: f32,
    pub _padding: u32,
}

/// Meshlet culling and rendering system
pub struct MeshletRenderer {
    /// Visibility buffer
    pub visibility_buffer: VisibilityBuffer,
    
    /// Meshlet data buffer
    pub meshlet_buffer: wgpu::Buffer,
    
    /// Vertex data buffer
    pub vertex_buffer: wgpu::Buffer,
    
    /// Index data buffer
    pub index_buffer: wgpu::Buffer,
    
    /// Bind group for meshlet data
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    
    /// Render pipeline
    pub pipeline: wgpu::RenderPipeline,
    
    /// Number of meshlets
    pub meshlet_count: u32,
}

impl MeshletRenderer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        meshlets: &[GpuMeshlet],
        vertices: &[u8],
        indices: &[u8],
    ) -> Self {
        let visibility_buffer = VisibilityBuffer::new(device, width, height);

        // Create meshlet buffer
        let meshlet_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meshlet Buffer"),
            contents: bytemuck::cast_slice(meshlets),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meshlet Vertex Buffer"),
            contents: vertices,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meshlet Index Buffer"),
            contents: indices,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Meshlet Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Meshlet Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: meshlet_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: index_buffer.as_entire_binding(),
                },
            ],
        });

        // Create shader module (placeholder - will be implemented in rendering integration)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Meshlet Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/nanite_visibility.wgsl").into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Meshlet Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Meshlet Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Uint,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            visibility_buffer,
            meshlet_buffer,
            vertex_buffer,
            index_buffer,
            bind_group,
            bind_group_layout,
            pipeline,
            meshlet_count: meshlets.len() as u32,
        }
    }

    /// Perform CPU-side frustum culling
    pub fn cull_meshlets(
        &self,
        meshlets: &[GpuMeshlet],
        frustum: &Frustum,
        camera_pos: Vec3,
    ) -> Vec<u32> {
        let mut visible_meshlets = Vec::new();

        for (i, meshlet) in meshlets.iter().enumerate() {
            let min = Vec3::from_array(meshlet.bounds_min);
            let max = Vec3::from_array(meshlet.bounds_max);

            // Frustum culling
            if !frustum.test_aabb(min, max) {
                continue;
            }

            // Backface culling using bounding cone
            let cone_apex = Vec3::from_array(meshlet.cone_apex);
            let cone_axis = Vec3::from_array(meshlet.cone_axis);
            let view_dir = (cone_apex - camera_pos).normalize();
            
            if cone_axis.dot(view_dir) < meshlet.cone_cutoff {
                continue; // Backfacing
            }

            visible_meshlets.push(i as u32);
        }

        visible_meshlets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_aabb_culling() {
        // Create a simple orthographic-like frustum
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = Frustum::from_matrix(view_proj);

        // Test AABB inside frustum
        assert!(frustum.test_aabb(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0)));

        // Test AABB outside frustum
        assert!(!frustum.test_aabb(Vec3::new(20.0, 20.0, 20.0), Vec3::new(25.0, 25.0, 25.0)));
    }

    #[test]
    fn test_lod_selection() {
        let selector = LODSelector::new(1080.0, std::f32::consts::FRAC_PI_3);

        // Close object should use high detail (LOD 0)
        let lod = selector.select_lod(
            Vec3::new(0.0, 0.0, -5.0),
            1.0,
            0.1,
            Vec3::ZERO,
            3,
        );
        assert_eq!(lod, 0);

        // Far object should use lower detail
        let lod = selector.select_lod(
            Vec3::new(0.0, 0.0, -100.0),
            1.0,
            0.1,
            Vec3::ZERO,
            3,
        );
        assert!(lod > 0);
    }
}