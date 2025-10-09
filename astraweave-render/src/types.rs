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
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tangent at location 12 (to match skinned variant convention)
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // uv at location 13
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 13,
                    format: wgpu::VertexFormat::Float32x2,
                },
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
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tangent (xyz, w = handedness)
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // joints
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Uint16x4,
                },
                // weights
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4,
                },
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
    pub material_id: u32,
    pub _padding: [u32; 3], // 16-byte alignment
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
                // material_id (uint)
                wgpu::VertexAttribute {
                    offset: 116,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Uint32,
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
    pub material_id: u32,
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
            material_id: self.material_id,
            _padding: [0; 3],
        }
    }

    pub fn from_pos_scale_color(pos: Vec3, scale: Vec3, color: [f32; 4]) -> Self {
        let transform = Mat4::from_scale_rotation_translation(scale, glam::Quat::IDENTITY, pos);
        Self {
            transform,
            color,
            material_id: 0,
        }
    }
}

// ---- Phase 2 foundations: clustered lighting binning helpers ----
#[derive(Clone, Copy, Debug)]
pub struct ClusterDims {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// Compute cluster index for a screen-space pixel and linear depth in view space.
/// Inputs:
/// - screen coords in [0,width) x [0,height)
/// - near/far planes
/// - dims: number of clusters in x/y/z
pub fn cluster_index(
    px: u32,
    py: u32,
    width: u32,
    height: u32,
    depth: f32,
    near: f32,
    far: f32,
    dims: ClusterDims,
) -> u32 {
    let sx = (px as f32 * dims.x as f32 / width as f32).clamp(0.0, dims.x as f32 - 1.0) as u32;
    let sy = (py as f32 * dims.y as f32 / height as f32).clamp(0.0, dims.y as f32 - 1.0) as u32;
    // Logarithmic z slicing improves distribution
    let z_lin = ((depth - near) / (far - near)).clamp(0.0, 0.99999);
    let z_log = (z_lin * (dims.z as f32)).floor() as u32;
    sx + sy * dims.x + z_log * dims.x * dims.y
}

#[cfg(test)]
mod tests_cluster {
    use super::*;
    #[test]
    fn bins_within_bounds() {
        let dims = ClusterDims { x: 16, y: 9, z: 24 };
        let idx = cluster_index(100, 50, 1920, 1080, 5.0, 0.1, 100.0, dims);
        assert!(idx < dims.x * dims.y * dims.z);
    }
    #[test]
    fn corners_map_to_edges() {
        let d = ClusterDims { x: 8, y: 8, z: 8 };
        let i0 = cluster_index(0, 0, 800, 800, 0.1, 0.1, 100.0, d);
        let i1 = cluster_index(799, 799, 800, 800, 99.9, 0.1, 100.0, d);
        assert_ne!(i0, i1);
    }
}
