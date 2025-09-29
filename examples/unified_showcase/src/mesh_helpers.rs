#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// Helper functions to generate vertices with normals and UVs

pub fn create_skybox_vertices() -> Vec<Vertex> {
    // Create a large cube for the skybox
    let mut vertices = Vec::new();
    let size = 2000.0; // Very large to encompass the entire scene

    // Skybox positions - inverted cube
    let positions = [
        // Front face (inside facing)
        [size, -size, size],
        [-size, -size, size],
        [-size, size, size],
        [size, size, size],
        // Back face
        [-size, -size, -size],
        [size, -size, -size],
        [size, size, -size],
        [-size, size, -size],
        // Top face
        [-size, size, size],
        [-size, size, -size],
        [size, size, -size],
        [size, size, size],
        // Bottom face
        [-size, -size, -size],
        [-size, -size, size],
        [size, -size, size],
        [size, -size, -size],
        // Right face
        [size, -size, -size],
        [size, -size, size],
        [size, size, size],
        [size, size, -size],
        // Left face
        [-size, -size, size],
        [-size, -size, -size],
        [-size, size, -size],
        [-size, size, size],
    ];

    // Inward-facing normals for skybox
    let normals = [
        [0.0, 0.0, -1.0], // Front face (inward)
        [0.0, 0.0, 1.0],  // Back face (inward)
        [0.0, -1.0, 0.0], // Top face (inward)
        [0.0, 1.0, 0.0],  // Bottom face (inward)
        [-1.0, 0.0, 0.0], // Right face (inward)
        [1.0, 0.0, 0.0],  // Left face (inward)
    ];

    // UV coordinates for each face
    // We'll map each face with full texture coordinates
    let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    // Add vertices for each face
    for face in 0..6 {
        let face_start = face * 4;

        // Triangle 1
        vertices.push(Vertex {
            position: positions[face_start],
            normal: normals[face],
            uv: uvs[0],
        });
        vertices.push(Vertex {
            position: positions[face_start + 1],
            normal: normals[face],
            uv: uvs[1],
        });
        vertices.push(Vertex {
            position: positions[face_start + 2],
            normal: normals[face],
            uv: uvs[2],
        });

        // Triangle 2
        vertices.push(Vertex {
            position: positions[face_start],
            normal: normals[face],
            uv: uvs[0],
        });
        vertices.push(Vertex {
            position: positions[face_start + 2],
            normal: normals[face],
            uv: uvs[2],
        });
        vertices.push(Vertex {
            position: positions[face_start + 3],
            normal: normals[face],
            uv: uvs[3],
        });
    }

    vertices
}
