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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_quad_has_six_vertices() {
        let quad = create_screen_quad();
        assert_eq!(quad.len(), 6, "Fullscreen quad must have 6 vertices (2 tris)");
    }

    #[test]
    fn screen_quad_covers_ndc_range() {
        let quad = create_screen_quad();
        let min_x = quad.iter().map(|v| v.position[0]).fold(f32::INFINITY, f32::min);
        let max_x = quad.iter().map(|v| v.position[0]).fold(f32::NEG_INFINITY, f32::max);
        let min_y = quad.iter().map(|v| v.position[1]).fold(f32::INFINITY, f32::min);
        let max_y = quad.iter().map(|v| v.position[1]).fold(f32::NEG_INFINITY, f32::max);

        assert!((min_x - (-1.0)).abs() < f32::EPSILON, "min_x should be -1.0");
        assert!((max_x - 1.0).abs() < f32::EPSILON, "max_x should be 1.0");
        assert!((min_y - (-1.0)).abs() < f32::EPSILON, "min_y should be -1.0");
        assert!((max_y - 1.0).abs() < f32::EPSILON, "max_y should be 1.0");
    }

    #[test]
    fn screen_quad_uv_corners() {
        let quad = create_screen_quad();
        // Collect unique (position, uv) pairs – all 4 NDC corners should appear
        let mut corners: Vec<([i32; 2], [i32; 2])> = quad
            .iter()
            .map(|v| {
                (
                    [v.position[0] as i32, v.position[1] as i32],
                    [v.uv[0] as i32, v.uv[1] as i32],
                )
            })
            .collect();
        corners.sort();
        corners.dedup();
        assert_eq!(corners.len(), 4, "Should cover all 4 NDC corners");
    }

    #[test]
    fn screen_quad_z_is_zero() {
        let quad = create_screen_quad();
        for v in &quad {
            assert!(
                v.position[2].abs() < f32::EPSILON,
                "z should be 0 for all vertices"
            );
        }
    }

    #[test]
    fn vertex_desc_has_two_attributes() {
        let layout = DebugQuadVertex::desc();
        assert_eq!(layout.attributes.len(), 2);
        assert_eq!(layout.attributes[0].format, wgpu::VertexFormat::Float32x3);
        assert_eq!(layout.attributes[1].format, wgpu::VertexFormat::Float32x2);
    }

    #[test]
    fn vertex_desc_stride_matches_struct_size() {
        let layout = DebugQuadVertex::desc();
        assert_eq!(
            layout.array_stride as usize,
            std::mem::size_of::<DebugQuadVertex>()
        );
    }
}
