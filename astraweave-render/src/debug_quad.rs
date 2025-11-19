use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct DebugQuadVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl DebugQuadVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugQuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub fn create_screen_quad() -> Vec<DebugQuadVertex> {
    vec![
        DebugQuadVertex {
            position: [-1.0, -1.0, 0.0],
            uv: [0.0, 1.0],
        },
        DebugQuadVertex {
            position: [1.0, -1.0, 0.0],
            uv: [1.0, 1.0],
        },
        DebugQuadVertex {
            position: [1.0, 1.0, 0.0],
            uv: [1.0, 0.0],
        },
        DebugQuadVertex {
            position: [-1.0, -1.0, 0.0],
            uv: [0.0, 1.0],
        },
        DebugQuadVertex {
            position: [1.0, 1.0, 0.0],
            uv: [1.0, 0.0],
        },
        DebugQuadVertex {
            position: [-1.0, 1.0, 0.0],
            uv: [0.0, 0.0],
        },
    ]
}
