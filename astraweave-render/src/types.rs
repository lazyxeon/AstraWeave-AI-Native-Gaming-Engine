use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                // normal
                wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
                // tangent at location 12 (to match skinned variant convention)
                wgpu::VertexAttribute { offset: 24, shader_location: 12, format: wgpu::VertexFormat::Float32x4 },
                // uv at location 13
                wgpu::VertexAttribute { offset: 40, shader_location: 13, format: wgpu::VertexFormat::Float32x2 },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub joints: [u16; 4],
    pub weights: [f32; 4],
}

impl SkinnedVertex {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<SkinnedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                // normal
                wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
                // tangent (xyz, w = handedness)
                wgpu::VertexAttribute { offset: 24, shader_location: 12, format: wgpu::VertexFormat::Float32x4 },
                // joints
                wgpu::VertexAttribute { offset: 40, shader_location: 10, format: wgpu::VertexFormat::Uint16x4 },
                // weights
                wgpu::VertexAttribute { offset: 48, shader_location: 11, format: wgpu::VertexFormat::Float32x4 },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 3]; 3],
    pub color: [f32; 4],
}

impl InstanceRaw {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model matrix (4x vec4)
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
                // normal matrix (3x vec3 packed as Float32x3)
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 76,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 88,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 100,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
}

#[derive(Clone, Debug)]
pub struct Material {
    pub color: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Instance {
    pub transform: Mat4,
    pub color: [f32; 4],
}

impl Instance {
    pub fn raw(&self) -> InstanceRaw {
        let model = self.transform;
        let normal = model.inverse().transpose();
        InstanceRaw {
            model: model.to_cols_array_2d(),
            normal_matrix: [
                normal.x_axis.truncate().to_array(),
                normal.y_axis.truncate().to_array(),
                normal.z_axis.truncate().to_array(),
            ],
            color: self.color,
        }
    }

    pub fn from_pos_scale_color(pos: Vec3, scale: Vec3, color: [f32; 4]) -> Self {
        let transform = Mat4::from_scale_rotation_translation(scale, glam::Quat::IDENTITY, pos);
        Self { transform, color }
    }
}
