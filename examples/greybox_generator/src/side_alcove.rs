// Side Alcove (Z2a) Greybox Generator
// 10m × 10m hidden alcove with 2m × 3m entrance tunnel
// Optional exploration zone branching off Z2 Fractured Cliffs
use anyhow::Result;
use base64::Engine;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::fs;

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    texcoord: [f32; 2],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

fn main() -> Result<()> {
    println!("Generating Side Alcove greybox mesh...");
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Alcove floor: 10m × 10m at Y=14
    create_platform(&mut vertices, &mut indices, 10.0, 10.0, 14.0);

    // Alcove walls (4m high, Y=14 to Y=18)
    // North wall (back wall with lore glyph)
    create_wall(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 14.0, 5.0),
        10.0,
        4.0,
        [0.0, 0.0, -1.0],
    );
    // East wall
    create_wall(
        &mut vertices,
        &mut indices,
        Vec3::new(5.0, 14.0, 0.0),
        10.0,
        4.0,
        [-1.0, 0.0, 0.0],
    );
    // West wall
    create_wall(
        &mut vertices,
        &mut indices,
        Vec3::new(-5.0, 14.0, 0.0),
        10.0,
        4.0,
        [1.0, 0.0, 0.0],
    );
    // South wall (with 2m wide gap for entrance)
    // Left segment
    create_wall(
        &mut vertices,
        &mut indices,
        Vec3::new(-3.0, 14.0, -5.0),
        4.0,
        4.0,
        [0.0, 0.0, 1.0],
    );
    // Right segment
    create_wall(
        &mut vertices,
        &mut indices,
        Vec3::new(3.0, 14.0, -5.0),
        4.0,
        4.0,
        [0.0, 0.0, 1.0],
    );

    // Ceiling
    create_ceiling(&mut vertices, &mut indices, 10.0, 10.0, 18.0);

    // Entrance tunnel: 2m wide × 3m tall × 6m long, slopes down from Y=14 to Y=12
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 13.5, -8.0),
        2.0,
        3.0,
        6.0,
    );

    // --- Interior elements ---

    // Center pedestal (Echo Shard location)
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 14.45, 0.0),
        1.0,
        0.9,
        1.0,
    );

    // Echo Shard marker (small cube on pedestal)
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 15.1, 0.0), 0.4);

    // Hidden anchor marker (behind pedestal, faintly glowing)
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 14.8, 2.0), 0.6);

    // Lore glyph marker on north wall (eye-level tablet)
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 15.7, 4.8),
        1.5,
        1.0,
        0.2,
    );

    // Crystal clusters (ambient decoration)
    create_cube(&mut vertices, &mut indices, Vec3::new(-3.0, 14.4, 3.0), 0.8);
    create_cube(&mut vertices, &mut indices, Vec3::new(3.0, 14.4, 3.0), 0.8);

    // --- Encode to GLTF ---
    let mut buffer_data = Vec::new();
    for v in &vertices {
        buffer_data.extend_from_slice(bytemuck::bytes_of(v));
    }
    for i in &indices {
        buffer_data.extend_from_slice(&i.to_le_bytes());
    }
    let buffer_base64 = base64::engine::general_purpose::STANDARD.encode(&buffer_data);
    let buffer_uri = format!("data:application/octet-stream;base64,{}", buffer_base64);

    let gltf = serde_json::json!({
        "asset": {"version": "2.0", "generator": "AstraWeave Greybox Generator v1.0 - Side Alcove"},
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [0.35, 0.4, 0.5, 1.0], "metallicFactor": 0.1, "roughnessFactor": 0.9}}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": vertices.len(), "type": "VEC3", "min": [-6.0, 11.0, -12.0], "max": [6.0, 19.0, 6.0]},
            {"bufferView": 0, "byteOffset": 12, "componentType": 5126, "count": vertices.len(), "type": "VEC3"},
            {"bufferView": 0, "byteOffset": 24, "componentType": 5126, "count": vertices.len(), "type": "VEC2"},
            {"bufferView": 1, "componentType": 5123, "count": indices.len(), "type": "SCALAR"}
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": vertices.len() * std::mem::size_of::<Vertex>(), "byteStride": std::mem::size_of::<Vertex>(), "target": 34962},
            {"buffer": 0, "byteOffset": vertices.len() * std::mem::size_of::<Vertex>(), "byteLength": indices.len() * 2, "target": 34963}
        ],
        "buffers": [{"byteLength": buffer_data.len(), "uri": buffer_uri}]
    });

    fs::create_dir_all("../../assets/models/greybox")?;
    fs::write(
        "../../assets/models/greybox/side_alcove_greybox.gltf",
        serde_json::to_string_pretty(&gltf)?,
    )?;
    println!(
        "Success! {} vertices, {} triangles",
        vertices.len(),
        indices.len() / 3
    );
    Ok(())
}

// --- Geometry helpers ---

/// Rectangular platform (top face + bottom face + 4 side faces)
fn create_platform(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    width: f32,
    depth: f32,
    elevation: f32,
) {
    let hw = width / 2.0;
    let hd = depth / 2.0;

    // Top face
    let b = vertices.len() as u16;
    for (x, z) in [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)] {
        vertices.push(Vertex {
            position: [x, elevation, z],
            normal: [0.0, 1.0, 0.0],
            texcoord: [(x + hw) / width, (z + hd) / depth],
        });
    }
    indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

    // Bottom face (0.5m thick slab)
    let bottom = elevation - 0.5;
    let b = vertices.len() as u16;
    for (x, z) in [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)] {
        vertices.push(Vertex {
            position: [x, bottom, z],
            normal: [0.0, -1.0, 0.0],
            texcoord: [(x + hw) / width, (z + hd) / depth],
        });
    }
    indices.extend_from_slice(&[b, b + 2, b + 1, b, b + 3, b + 2]);
}

/// Ceiling panel (single downward-facing quad)
fn create_ceiling(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    width: f32,
    depth: f32,
    elevation: f32,
) {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let b = vertices.len() as u16;
    for (x, z) in [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)] {
        vertices.push(Vertex {
            position: [x, elevation, z],
            normal: [0.0, -1.0, 0.0],
            texcoord: [(x + hw) / width, (z + hd) / depth],
        });
    }
    indices.extend_from_slice(&[b, b + 2, b + 1, b, b + 3, b + 2]);
}

/// Single-sided wall quad (inward-facing)
fn create_wall(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    origin: Vec3,
    length: f32,
    height: f32,
    normal: [f32; 3],
) {
    let b = vertices.len() as u16;
    let hl = length / 2.0;

    // Determine wall orientation from normal
    if normal[0].abs() > 0.5 {
        // East/west wall (extends along Z)
        let x = origin.x;
        vertices.push(Vertex {
            position: [x, origin.y, origin.z - hl],
            normal,
            texcoord: [0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [x, origin.y, origin.z + hl],
            normal,
            texcoord: [1.0, 0.0],
        });
        vertices.push(Vertex {
            position: [x, origin.y + height, origin.z + hl],
            normal,
            texcoord: [1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [x, origin.y + height, origin.z - hl],
            normal,
            texcoord: [0.0, 1.0],
        });
    } else {
        // North/south wall (extends along X)
        let z = origin.z;
        vertices.push(Vertex {
            position: [origin.x - hl, origin.y, z],
            normal,
            texcoord: [0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [origin.x + hl, origin.y, z],
            normal,
            texcoord: [1.0, 0.0],
        });
        vertices.push(Vertex {
            position: [origin.x + hl, origin.y + height, z],
            normal,
            texcoord: [1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [origin.x - hl, origin.y + height, z],
            normal,
            texcoord: [0.0, 1.0],
        });
    }
    indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
}

/// Axis-aligned box (6 faces)
fn create_box(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    c: Vec3,
    width: f32,
    height: f32,
    depth: f32,
) {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let hd = depth / 2.0;
    let faces = [
        (
            [0.0, 0.0, 1.0],
            [[-hw, -hh, hd], [hw, -hh, hd], [hw, hh, hd], [-hw, hh, hd]],
        ),
        (
            [0.0, 0.0, -1.0],
            [
                [hw, -hh, -hd],
                [-hw, -hh, -hd],
                [-hw, hh, -hd],
                [hw, hh, -hd],
            ],
        ),
        (
            [1.0, 0.0, 0.0],
            [[hw, -hh, hd], [hw, -hh, -hd], [hw, hh, -hd], [hw, hh, hd]],
        ),
        (
            [-1.0, 0.0, 0.0],
            [
                [-hw, -hh, -hd],
                [-hw, -hh, hd],
                [-hw, hh, hd],
                [-hw, hh, -hd],
            ],
        ),
        (
            [0.0, 1.0, 0.0],
            [[-hw, hh, hd], [hw, hh, hd], [hw, hh, -hd], [-hw, hh, -hd]],
        ),
        (
            [0.0, -1.0, 0.0],
            [
                [-hw, -hh, -hd],
                [hw, -hh, -hd],
                [hw, -hh, hd],
                [-hw, -hh, hd],
            ],
        ),
    ];
    for (n, ps) in &faces {
        let b = vertices.len() as u16;
        for p in ps {
            let w = Vec3::new(p[0], p[1], p[2]) + c;
            vertices.push(Vertex {
                position: [w.x, w.y, w.z],
                normal: *n,
                texcoord: [0.0, 0.0],
            });
        }
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
}

/// Cube (uniform dimensions)
fn create_cube(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, c: Vec3, size: f32) {
    create_box(vertices, indices, c, size, size, size);
}
