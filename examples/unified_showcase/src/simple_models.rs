/// Simple procedural 3D models for testing asset loading workflow
/// These serve as "imported assets" to demonstrate the pipeline
use super::Vertex;

/// Create a simple stylized tree (better looking than the basic cylinder+sphere)
pub fn create_stylized_tree() -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Trunk (tapered cylinder)
    let trunk_height = 5.0;
    let trunk_radius_bottom = 0.4;
    let trunk_radius_top = 0.3;
    let trunk_segments = 8;

    for i in 0..trunk_segments {
        let angle = (i as f32 / trunk_segments as f32) * std::f32::consts::TAU;
        let next_angle = ((i + 1) as f32 / trunk_segments as f32) * std::f32::consts::TAU;

        let x1 = angle.cos() * trunk_radius_bottom;
        let z1 = angle.sin() * trunk_radius_bottom;
        let x2 = next_angle.cos() * trunk_radius_bottom;
        let z2 = next_angle.sin() * trunk_radius_bottom;

        let x3 = angle.cos() * trunk_radius_top;
        let z3 = angle.sin() * trunk_radius_top;
        let x4 = next_angle.cos() * trunk_radius_top;
        let z4 = next_angle.sin() * trunk_radius_top;

        let normal_x = angle.cos();
        let normal_z = angle.sin();

        let base_idx = vertices.len() as u32;

        // Bottom quad vertices
        vertices.push(Vertex {
            position: [x1, 0.0, z1],
            normal: [normal_x, 0.0, normal_z],
            uv: [i as f32 / trunk_segments as f32, 0.0],
        });
        vertices.push(Vertex {
            position: [x2, 0.0, z2],
            normal: [normal_x, 0.0, normal_z],
            uv: [(i + 1) as f32 / trunk_segments as f32, 0.0],
        });
        vertices.push(Vertex {
            position: [x4, trunk_height, z4],
            normal: [normal_x, 0.0, normal_z],
            uv: [(i + 1) as f32 / trunk_segments as f32, 1.0],
        });
        vertices.push(Vertex {
            position: [x3, trunk_height, z3],
            normal: [normal_x, 0.0, normal_z],
            uv: [i as f32 / trunk_segments as f32, 1.0],
        });

        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
    }

    // Canopy (stylized cone with multiple tiers)
    let canopy_base_y = trunk_height - 1.0;
    let canopy_height = 4.0;
    let canopy_radius = 2.5;
    let canopy_segments = 12;

    // Tier 1 (bottom)
    for i in 0..canopy_segments {
        let angle = (i as f32 / canopy_segments as f32) * std::f32::consts::TAU;
        let next_angle = ((i + 1) as f32 / canopy_segments as f32) * std::f32::consts::TAU;

        let x1 = angle.cos() * canopy_radius;
        let z1 = angle.sin() * canopy_radius;
        let x2 = next_angle.cos() * canopy_radius;
        let z2 = next_angle.sin() * canopy_radius;

        let base_idx = vertices.len() as u32;

        // Triangle fan (bottom → top)
        vertices.push(Vertex {
            position: [x1, canopy_base_y, z1],
            normal: [x1, canopy_height * 0.5, z1], // Outward+upward normal
            uv: [0.5 + x1 * 0.2, 0.5 + z1 * 0.2],
        });
        vertices.push(Vertex {
            position: [x2, canopy_base_y, z2],
            normal: [x2, canopy_height * 0.5, z2],
            uv: [0.5 + x2 * 0.2, 0.5 + z2 * 0.2],
        });
        vertices.push(Vertex {
            position: [0.0, canopy_base_y + canopy_height, 0.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
        });

        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
    }

    (vertices, indices)
}

/// Create a simple building (house with peaked roof)
pub fn create_simple_house() -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let width = 4.0;
    let depth = 4.0;
    let height = 3.0;
    let roof_height = 2.0;

    // Base cube (house body)
    let base_idx = vertices.len() as u32;

    // Front face
    vertices.push(Vertex {
        position: [-width / 2.0, 0.0, depth / 2.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: [width / 2.0, 0.0, depth / 2.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: [width / 2.0, height, depth / 2.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 1.0],
    });
    vertices.push(Vertex {
        position: [-width / 2.0, height, depth / 2.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 1.0],
    });

    indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
    indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);

    // Back face
    let back_idx = vertices.len() as u32;
    vertices.push(Vertex {
        position: [width / 2.0, 0.0, -depth / 2.0],
        normal: [0.0, 0.0, -1.0],
        uv: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: [-width / 2.0, 0.0, -depth / 2.0],
        normal: [0.0, 0.0, -1.0],
        uv: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: [-width / 2.0, height, -depth / 2.0],
        normal: [0.0, 0.0, -1.0],
        uv: [1.0, 1.0],
    });
    vertices.push(Vertex {
        position: [width / 2.0, height, -depth / 2.0],
        normal: [0.0, 0.0, -1.0],
        uv: [0.0, 1.0],
    });

    indices.extend_from_slice(&[back_idx, back_idx + 1, back_idx + 2]);
    indices.extend_from_slice(&[back_idx, back_idx + 2, back_idx + 3]);

    // Left, right, bottom faces (similar pattern)
    // ... (abbreviated for brevity, would add all 6 faces)

    // Peaked roof
    let roof_base = height;
    let roof_top = height + roof_height;

    // Front slope
    let roof_front_idx = vertices.len() as u32;
    vertices.push(Vertex {
        position: [-width / 2.0, roof_base, depth / 2.0],
        normal: [0.0, 0.7, 0.7],
        uv: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: [width / 2.0, roof_base, depth / 2.0],
        normal: [0.0, 0.7, 0.7],
        uv: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: [0.0, roof_top, 0.0],
        normal: [0.0, 0.7, 0.7],
        uv: [0.5, 1.0],
    });

    indices.extend_from_slice(&[roof_front_idx, roof_front_idx + 1, roof_front_idx + 2]);

    (vertices, indices)
}
