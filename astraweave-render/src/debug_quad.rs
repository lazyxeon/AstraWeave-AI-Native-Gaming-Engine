use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct DebugQuadVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
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
