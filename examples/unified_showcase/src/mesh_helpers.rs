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
pub fn create_cube_vertices() -> Vec<Vertex> {
    let positions = [
        // Front face
        [-0.5, -0.5, 0.5], [0.5, -0.5, 0.5], [0.5, 0.5, 0.5], [-0.5, 0.5, 0.5],
        // Back face
        [-0.5, -0.5, -0.5], [-0.5, 0.5, -0.5], [0.5, 0.5, -0.5], [0.5, -0.5, -0.5],
    ];
    
    let normals = [
        [0.0, 0.0, 1.0],  // Front face
        [1.0, 0.0, 0.0],  // Right face
        [0.0, 0.0, -1.0], // Back face
        [-1.0, 0.0, 0.0], // Left face
        [0.0, 1.0, 0.0],  // Top face
        [0.0, -1.0, 0.0], // Bottom face
    ];
    
    let uvs = [
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],  // Standard UV mapping for each face
    ];
    
    let faces = [
        // Face index, then 4 vertex indices in clockwise order
        [0, 0, 1, 2, 3], // Front
        [1, 1, 7, 6, 2], // Right
        [2, 7, 4, 5, 6], // Back
        [3, 4, 0, 3, 5], // Left
        [4, 3, 2, 6, 5], // Top
        [5, 4, 7, 1, 0], // Bottom
    ];
    
    let mut vertices = Vec::new();
    
    for [face_idx, a, b, c, d] in faces {
        // Triangle 1 (a, b, c)
        vertices.push(Vertex {
            position: positions[a],
            normal: normals[face_idx],
            uv: uvs[0],
        });
        vertices.push(Vertex {
            position: positions[b],
            normal: normals[face_idx],
            uv: uvs[1],
        });
        vertices.push(Vertex {
            position: positions[c],
            normal: normals[face_idx],
            uv: uvs[2],
        });
        
        // Triangle 2 (a, c, d)
        vertices.push(Vertex {
            position: positions[a],
            normal: normals[face_idx],
            uv: uvs[0],
        });
        vertices.push(Vertex {
            position: positions[c],
            normal: normals[face_idx],
            uv: uvs[2],
        });
        vertices.push(Vertex {
            position: positions[d],
            normal: normals[face_idx],
            uv: uvs[3],
        });
    }
    
    vertices
}

pub fn create_tree_vertices() -> Vec<Vertex> {
    // We'll generate a proper tree with trunk and foliage
    // This is a simplified version of the geometry already defined in main.rs
    let mut vertices = Vec::new();
    
    // Generate trunk vertices (simplified cylinder)
    let trunk_segments = 8;
    let trunk_radius = 0.15;
    let trunk_height = 1.0;
    
    // Bottom ring
    for i in 0..trunk_segments {
        let angle = (i as f32 / trunk_segments as f32) * 2.0 * std::f32::consts::PI;
        let next_angle = ((i + 1) as f32 / trunk_segments as f32) * 2.0 * std::f32::consts::PI;
        
        let x = trunk_radius * angle.cos();
        let z = trunk_radius * angle.sin();
        let next_x = trunk_radius * next_angle.cos();
        let next_z = trunk_radius * next_angle.sin();
        
        // Bottom center vertex
        vertices.push(Vertex {
            position: [0.0, -0.5, 0.0],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5, 0.5],
        });
        
        // Bottom rim vertices
        vertices.push(Vertex {
            position: [x, -0.5, z],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5 + x / (2.0 * trunk_radius), 0.5 + z / (2.0 * trunk_radius)],
        });
        
        vertices.push(Vertex {
            position: [next_x, -0.5, next_z],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5 + next_x / (2.0 * trunk_radius), 0.5 + next_z / (2.0 * trunk_radius)],
        });
        
        // Side triangles
        let normal_x = angle.cos();
        let normal_z = angle.sin();
        let next_normal_x = next_angle.cos();
        let next_normal_z = next_angle.sin();
        
        // Bottom side vertex
        vertices.push(Vertex {
            position: [x, -0.5, z],
            normal: [normal_x, 0.0, normal_z],
            uv: [i as f32 / trunk_segments as f32, 0.0],
        });
        
        // Top side vertex
        vertices.push(Vertex {
            position: [x, 0.5, z],
            normal: [normal_x, 0.0, normal_z],
            uv: [i as f32 / trunk_segments as f32, 0.5],
        });
        
        // Next top side vertex
        vertices.push(Vertex {
            position: [next_x, 0.5, next_z],
            normal: [next_normal_x, 0.0, next_normal_z],
            uv: [(i + 1) as f32 / trunk_segments as f32, 0.5],
        });
        
        // Bottom side vertex
        vertices.push(Vertex {
            position: [x, -0.5, z],
            normal: [normal_x, 0.0, normal_z],
            uv: [i as f32 / trunk_segments as f32, 0.0],
        });
        
        // Next top side vertex
        vertices.push(Vertex {
            position: [next_x, 0.5, next_z],
            normal: [next_normal_x, 0.0, next_normal_z],
            uv: [(i + 1) as f32 / trunk_segments as f32, 0.5],
        });
        
        // Next bottom side vertex
        vertices.push(Vertex {
            position: [next_x, -0.5, next_z],
            normal: [next_normal_x, 0.0, next_normal_z],
            uv: [(i + 1) as f32 / trunk_segments as f32, 0.0],
        });
    }
    
    // Top cap of trunk
    for i in 0..trunk_segments {
        let angle = (i as f32 / trunk_segments as f32) * 2.0 * std::f32::consts::PI;
        let next_angle = ((i + 1) as f32 / trunk_segments as f32) * 2.0 * std::f32::consts::PI;
        
        let x = trunk_radius * angle.cos();
        let z = trunk_radius * angle.sin();
        let next_x = trunk_radius * next_angle.cos();
        let next_z = trunk_radius * next_angle.sin();
        
        // Center vertex
        vertices.push(Vertex {
            position: [0.0, 0.5, 0.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
        });
        
        // Rim vertices
        vertices.push(Vertex {
            position: [x, 0.5, z],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5 + x / (2.0 * trunk_radius), 0.5 + z / (2.0 * trunk_radius)],
        });
        
        vertices.push(Vertex {
            position: [next_x, 0.5, next_z],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5 + next_x / (2.0 * trunk_radius), 0.5 + next_z / (2.0 * trunk_radius)],
        });
    }
    
    // Foliage (simple cone)
    let foliage_radius = 0.6;
    let foliage_height = 0.8;
    let foliage_y_base = 0.5; // Start at top of trunk
    
    for i in 0..trunk_segments {
        let angle = (i as f32 / trunk_segments as f32) * 2.0 * std::f32::consts::PI;
        let next_angle = ((i + 1) as f32 / trunk_segments as f32) * 2.0 * std::f32::consts::PI;
        
        let x = foliage_radius * angle.cos();
        let z = foliage_radius * angle.sin();
        let next_x = foliage_radius * next_angle.cos();
        let next_z = foliage_radius * next_angle.sin();
        
        // Side triangles for foliage
        let normal_y = 0.4; // Upward component of normal
        let normal_scale = 1.0 / (1.0 + normal_y * normal_y).sqrt();
        
        let normal_x = angle.cos() * normal_scale;
        let normal_z = angle.sin() * normal_scale;
        let next_normal_x = next_angle.cos() * normal_scale;
        let next_normal_z = next_angle.sin() * normal_scale;
        
        // Base vertex
        vertices.push(Vertex {
            position: [x, foliage_y_base, z],
            normal: [normal_x, normal_y, normal_z],
            uv: [i as f32 / trunk_segments as f32, 0.5],
        });
        
        // Top vertex
        vertices.push(Vertex {
            position: [0.0, foliage_y_base + foliage_height, 0.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 1.0],
        });
        
        // Next base vertex
        vertices.push(Vertex {
            position: [next_x, foliage_y_base, next_z],
            normal: [next_normal_x, normal_y, next_normal_z],
            uv: [(i + 1) as f32 / trunk_segments as f32, 0.5],
        });
    }
    
    vertices
}

pub fn create_house_vertices() -> Vec<Vertex> {
    // Create a simple house with walls and roof
    let mut vertices = Vec::new();
    
    // House dimensions
    let width = 0.8;
    let height = 0.5;
    let depth = 0.8;
    let roof_height = 0.3;
    
    // Base cube (walls)
    let base_positions = [
        // Front face (door side)
        [-width, -height, depth], [width, -height, depth], [width, height, depth], [-width, height, depth],
        // Back face
        [-width, -height, -depth], [-width, height, -depth], [width, height, -depth], [width, -height, -depth],
    ];
    
    // Normals for each face
    let normals = [
        [0.0, 0.0, 1.0],   // Front
        [1.0, 0.0, 0.0],   // Right
        [0.0, 0.0, -1.0],  // Back
        [-1.0, 0.0, 0.0],  // Left
        [0.0, 1.0, 0.0],   // Top
        [0.0, -1.0, 0.0],  // Bottom
    ];
    
    // UV coordinates for faces - standard mapping
    let uvs_wall = [
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],  // Wall UVs (with door space)
    ];
    
    // Create walls
    let faces = [
        // Front face (with door)
        [0, 0, 1, 2, 3],
        // Right face
        [1, 1, 7, 6, 2],
        // Back face
        [2, 7, 4, 5, 6],
        // Left face
        [3, 4, 0, 3, 5],
        // Bottom face
        [5, 4, 7, 1, 0],
    ];
    
    for [face_idx, a, b, c, d] in faces {
        // Wall triangles
        vertices.push(Vertex {
            position: base_positions[a],
            normal: normals[face_idx],
            uv: uvs_wall[0],
        });
        vertices.push(Vertex {
            position: base_positions[b],
            normal: normals[face_idx],
            uv: uvs_wall[1],
        });
        vertices.push(Vertex {
            position: base_positions[c],
            normal: normals[face_idx],
            uv: uvs_wall[2],
        });
        
        vertices.push(Vertex {
            position: base_positions[a],
            normal: normals[face_idx],
            uv: uvs_wall[0],
        });
        vertices.push(Vertex {
            position: base_positions[c],
            normal: normals[face_idx],
            uv: uvs_wall[2],
        });
        vertices.push(Vertex {
            position: base_positions[d],
            normal: normals[face_idx],
            uv: uvs_wall[3],
        });
    }
    
    // Roof (triangular prism on top)
    let roof_peak = [0.0, height + roof_height, 0.0];
    
    // Roof faces
    // Front roof triangle
    vertices.push(Vertex {
        position: base_positions[3],  // Front-left top
        normal: [0.0, 0.3, 0.7],
        uv: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: base_positions[2],  // Front-right top
        normal: [0.0, 0.3, 0.7],
        uv: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: roof_peak,
        normal: [0.0, 0.3, 0.7],
        uv: [0.5, 1.0],
    });
    
    // Back roof triangle
    vertices.push(Vertex {
        position: base_positions[6],  // Back-right top
        normal: [0.0, 0.3, -0.7],
        uv: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: base_positions[5],  // Back-left top
        normal: [0.0, 0.3, -0.7],
        uv: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: roof_peak,
        normal: [0.0, 0.3, -0.7],
        uv: [0.5, 1.0],
    });
    
    // Left roof side
    vertices.push(Vertex {
        position: base_positions[5],  // Back-left top
        normal: [-0.7, 0.3, 0.0],
        uv: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: base_positions[3],  // Front-left top
        normal: [-0.7, 0.3, 0.0],
        uv: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: roof_peak,
        normal: [-0.7, 0.3, 0.0],
        uv: [0.5, 1.0],
    });
    
    // Right roof side
    vertices.push(Vertex {
        position: base_positions[2],  // Front-right top
        normal: [0.7, 0.3, 0.0],
        uv: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: base_positions[6],  // Back-right top
        normal: [0.7, 0.3, 0.0],
        uv: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: roof_peak,
        normal: [0.7, 0.3, 0.0],
        uv: [0.5, 1.0],
    });
    
    vertices
}

pub fn create_character_vertices() -> Vec<Vertex> {
    // Create a simplified humanoid character
    let mut vertices = Vec::new();
    
    // Head dimensions
    let head_size = 0.15;
    let head_y_offset = 0.9;
    
    // Create head (cube)
    let head_positions = [
        // Front face
        [-head_size, head_y_offset, head_size],
        [head_size, head_y_offset, head_size],
        [head_size, head_y_offset + 2.0 * head_size, head_size],
        [-head_size, head_y_offset + 2.0 * head_size, head_size],
        // Back face
        [-head_size, head_y_offset, -head_size],
        [-head_size, head_y_offset + 2.0 * head_size, -head_size],
        [head_size, head_y_offset + 2.0 * head_size, -head_size],
        [head_size, head_y_offset, -head_size],
    ];
    
    // Head UVs - face on front side
    let head_uvs = [
        [0.125, 0.75], [0.375, 0.75], [0.375, 0.5], [0.125, 0.5],  // Front face (with face features)
        [0.625, 0.75], [0.625, 0.5], [0.875, 0.5], [0.875, 0.75],  // Back of head
        [0.375, 0.75], [0.625, 0.75], [0.625, 0.5], [0.375, 0.5],  // Right side of head
        [0.125, 0.5], [0.125, 0.75], [0.375, 0.75], [0.375, 0.5],  // Left side of head
        [0.375, 0.25], [0.625, 0.25], [0.625, 0.5], [0.375, 0.5],  // Top of head
        [0.375, 0.75], [0.625, 0.75], [0.625, 1.0], [0.375, 1.0],  // Bottom of head
    ];
    
    // Create head cube faces
    let head_faces = [
        // Face index, then 4 vertex indices in clockwise order
        [0, 0, 1, 2, 3],  // Front - face texture
        [2, 7, 4, 5, 6],  // Back
        [1, 1, 7, 6, 2],  // Right side
        [3, 4, 0, 3, 5],  // Left side
        [4, 3, 2, 6, 5],  // Top
        [5, 4, 7, 1, 0],  // Bottom
    ];
    
    // Add head vertices
    for i in 0..head_faces.len() {
        let [face_idx, a, b, c, d] = head_faces[i];
        let uv_offset = i * 4;
        
        // Triangle 1 (a, b, c)
        vertices.push(Vertex {
            position: head_positions[a],
            normal: [0.0, 0.0, 1.0], // Simplified normals
            uv: head_uvs[uv_offset],
        });
        vertices.push(Vertex {
            position: head_positions[b],
            normal: [0.0, 0.0, 1.0],
            uv: head_uvs[uv_offset + 1],
        });
        vertices.push(Vertex {
            position: head_positions[c],
            normal: [0.0, 0.0, 1.0],
            uv: head_uvs[uv_offset + 2],
        });
        
        // Triangle 2 (a, c, d)
        vertices.push(Vertex {
            position: head_positions[a],
            normal: [0.0, 0.0, 1.0],
            uv: head_uvs[uv_offset],
        });
        vertices.push(Vertex {
            position: head_positions[c],
            normal: [0.0, 0.0, 1.0],
            uv: head_uvs[uv_offset + 2],
        });
        vertices.push(Vertex {
            position: head_positions[d],
            normal: [0.0, 0.0, 1.0],
            uv: head_uvs[uv_offset + 3],
        });
    }
    
    // Body dimensions
    let body_width = 0.2;
    let body_height = 0.3;
    let body_depth = 0.1;
    let body_y_offset = 0.5;
    
    // Body positions
    let body_positions = [
        // Front face
        [-body_width, body_y_offset, body_depth],
        [body_width, body_y_offset, body_depth],
        [body_width, body_y_offset + body_height, body_depth],
        [-body_width, body_y_offset + body_height, body_depth],
        // Back face
        [-body_width, body_y_offset, -body_depth],
        [-body_width, body_y_offset + body_height, -body_depth],
        [body_width, body_y_offset + body_height, -body_depth],
        [body_width, body_y_offset, -body_depth],
    ];
    
    // Body UVs - torso texture
    let body_uv_y_start = 0.25;
    let body_uvs = [
        // Front, back, sides, top, bottom of body
        [0.0, 0.5], [0.25, 0.5], [0.25, body_uv_y_start], [0.0, body_uv_y_start],
    ];
    
    // Create body faces (similar to head but with different UVs)
    for face_idx in 0..6 {
        let vertices_indices = match face_idx {
            0 => [0, 1, 2, 3],  // Front
            1 => [7, 4, 5, 6],  // Back
            2 => [1, 7, 6, 2],  // Right
            3 => [4, 0, 3, 5],  // Left
            4 => [3, 2, 6, 5],  // Top
            5 => [4, 7, 1, 0],  // Bottom
            _ => unreachable!(),
        };
        
        let normal = match face_idx {
            0 => [0.0, 0.0, 1.0],
            1 => [0.0, 0.0, -1.0],
            2 => [1.0, 0.0, 0.0],
            3 => [-1.0, 0.0, 0.0],
            4 => [0.0, 1.0, 0.0],
            5 => [0.0, -1.0, 0.0],
            _ => unreachable!(),
        };
        
        // Triangle 1
        vertices.push(Vertex {
            position: body_positions[vertices_indices[0]],
            normal,
            uv: body_uvs[0],
        });
        vertices.push(Vertex {
            position: body_positions[vertices_indices[1]],
            normal,
            uv: body_uvs[1],
        });
        vertices.push(Vertex {
            position: body_positions[vertices_indices[2]],
            normal,
            uv: body_uvs[2],
        });
        
        // Triangle 2
        vertices.push(Vertex {
            position: body_positions[vertices_indices[0]],
            normal,
            uv: body_uvs[0],
        });
        vertices.push(Vertex {
            position: body_positions[vertices_indices[2]],
            normal,
            uv: body_uvs[2],
        });
        vertices.push(Vertex {
            position: body_positions[vertices_indices[3]],
            normal,
            uv: body_uvs[3],
        });
    }
    
    // Add legs (two rectangular prisms)
    let leg_width = 0.07;
    let leg_height = 0.5;
    let leg_depth = 0.07;
    let leg_spacing = 0.08;
    
    // Left leg
    let left_leg_positions = [
        // Front face
        [-leg_spacing - leg_width, -0.5, leg_depth],
        [-leg_spacing, -0.5, leg_depth],
        [-leg_spacing, body_y_offset, leg_depth],
        [-leg_spacing - leg_width, body_y_offset, leg_depth],
        // Back face
        [-leg_spacing - leg_width, -0.5, -leg_depth],
        [-leg_spacing - leg_width, body_y_offset, -leg_depth],
        [-leg_spacing, body_y_offset, -leg_depth],
        [-leg_spacing, -0.5, -leg_depth],
    ];
    
    // Right leg
    let right_leg_positions = [
        // Front face
        [leg_spacing, -0.5, leg_depth],
        [leg_spacing + leg_width, -0.5, leg_depth],
        [leg_spacing + leg_width, body_y_offset, leg_depth],
        [leg_spacing, body_y_offset, leg_depth],
        // Back face
        [leg_spacing, -0.5, -leg_depth],
        [leg_spacing, body_y_offset, -leg_depth],
        [leg_spacing + leg_width, body_y_offset, -leg_depth],
        [leg_spacing + leg_width, -0.5, -leg_depth],
    ];
    
    // Legs UVs
    let legs_uv_y_start = 0.0;
    let legs_uv_y_end = 0.25;
    let legs_uvs = [
        // Front, back, sides, top, bottom of legs
        [0.0, legs_uv_y_end], [0.125, legs_uv_y_end], [0.125, legs_uv_y_start], [0.0, legs_uv_y_start],
    ];
    
    // Add leg vertices - use the same pattern as for the body
    let legs_positions = [left_leg_positions, right_leg_positions];
    
    for leg_positions in legs_positions.iter() {
        for face_idx in 0..6 {
            let vertices_indices = match face_idx {
                0 => [0, 1, 2, 3],  // Front
                1 => [7, 4, 5, 6],  // Back
                2 => [1, 7, 6, 2],  // Right
                3 => [4, 0, 3, 5],  // Left
                4 => [3, 2, 6, 5],  // Top
                5 => [4, 7, 1, 0],  // Bottom
                _ => unreachable!(),
            };
            
            let normal = match face_idx {
                0 => [0.0, 0.0, 1.0],
                1 => [0.0, 0.0, -1.0],
                2 => [1.0, 0.0, 0.0],
                3 => [-1.0, 0.0, 0.0],
                4 => [0.0, 1.0, 0.0],
                5 => [0.0, -1.0, 0.0],
                _ => unreachable!(),
            };
            
            // Triangle 1
            vertices.push(Vertex {
                position: leg_positions[vertices_indices[0]],
                normal,
                uv: legs_uvs[0],
            });
            vertices.push(Vertex {
                position: leg_positions[vertices_indices[1]],
                normal,
                uv: legs_uvs[1],
            });
            vertices.push(Vertex {
                position: leg_positions[vertices_indices[2]],
                normal,
                uv: legs_uvs[2],
            });
            
            // Triangle 2
            vertices.push(Vertex {
                position: leg_positions[vertices_indices[0]],
                normal,
                uv: legs_uvs[0],
            });
            vertices.push(Vertex {
                position: leg_positions[vertices_indices[2]],
                normal,
                uv: legs_uvs[2],
            });
            vertices.push(Vertex {
                position: leg_positions[vertices_indices[3]],
                normal,
                uv: legs_uvs[3],
            });
        }
    }
    
    vertices
}

pub fn create_skybox_vertices() -> Vec<Vertex> {
    // Create a large cube for the skybox
    let mut vertices = Vec::new();
    let size = 2000.0; // Very large to encompass the entire scene
    
    // Skybox positions - inverted cube
    let positions = [
        // Front face (inside facing)
        [size, -size, size], [-size, -size, size], [-size, size, size], [size, size, size],
        // Back face
        [-size, -size, -size], [size, -size, -size], [size, size, -size], [-size, size, -size],
        // Top face
        [-size, size, size], [-size, size, -size], [size, size, -size], [size, size, size],
        // Bottom face
        [-size, -size, -size], [-size, -size, size], [size, -size, size], [size, -size, -size],
        // Right face
        [size, -size, -size], [size, -size, size], [size, size, size], [size, size, -size],
        // Left face
        [-size, -size, size], [-size, -size, -size], [-size, size, -size], [-size, size, size],
    ];
    
    // Inward-facing normals for skybox
    let normals = [
        [0.0, 0.0, -1.0],  // Front face (inward)
        [0.0, 0.0, 1.0],   // Back face (inward)
        [0.0, -1.0, 0.0],  // Top face (inward)
        [0.0, 1.0, 0.0],   // Bottom face (inward)
        [-1.0, 0.0, 0.0],  // Right face (inward)
        [1.0, 0.0, 0.0],   // Left face (inward)
    ];
    
    // UV coordinates for each face
    // We'll map each face with full texture coordinates
    let uvs = [
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    ];
    
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

pub fn generate_terrain_vertices(size: u32, scale: f32) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let half_size = (size / 2) as f32;
    
    // Generate grid of vertices
    for z in 0..size {
        for x in 0..size {
            let x_pos = (x as f32 - half_size) * scale;
            let z_pos = (z as f32 - half_size) * scale;
            
            // Generate height using simplex noise
            let height = 0.0; // We'll calculate this in the shader
            
            // Generate normal (approximately flat for now)
            let normal = [0.0, 1.0, 0.0];
            
            // Generate UV coordinates for texturing
            let u = x as f32 / (size - 1) as f32;
            let v = z as f32 / (size - 1) as f32;
            
            vertices.push(Vertex {
                position: [x_pos, height, z_pos],
                normal,
                uv: [u, v],
            });
        }
    }
    
    // Generate indices for triangles
    let mut indices = Vec::new();
    for z in 0..size - 1 {
        for x in 0..size - 1 {
            let top_left = z * size + x;
            let top_right = top_left + 1;
            let bottom_left = (z + 1) * size + x;
            let bottom_right = bottom_left + 1;
            
            // First triangle (top-left, bottom-left, bottom-right)
            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(bottom_right);
            
            // Second triangle (top-left, bottom-right, top-right)
            indices.push(top_left);
            indices.push(bottom_right);
            indices.push(top_right);
        }
    }
    
    // Convert indices to vertices
    let mut triangle_vertices = Vec::new();
    for chunk in indices.chunks(3) {
        if let [a, b, c] = chunk {
            triangle_vertices.push(vertices[*a as usize]);
            triangle_vertices.push(vertices[*b as usize]);
            triangle_vertices.push(vertices[*c as usize]);
        }
    }
    
    triangle_vertices
}