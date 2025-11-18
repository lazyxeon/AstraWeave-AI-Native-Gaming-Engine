// Fractured Cliffs Greybox Generator
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
    println!("Generating Fractured Cliffs greybox mesh...");
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // 200m linear path (5-10m width) from Z=0 to Z=200
    // Use 8m average width for walkable area
    create_path(&mut vertices, &mut indices, 200.0, 8.0, 0.5);

    // Cliff walls (20m high) on left and right of path
    // Left wall: X = -5 (4m from path edge)
    create_cliff_wall(&mut vertices, &mut indices, -5.0, 200.0, 20.0, 1.0);
    // Right wall: X = +5 (4m from path edge)
    create_cliff_wall(&mut vertices, &mut indices, 5.0, 200.0, 20.0, 1.0);

    // Vista platform at end (15m x 15m at Z=200, elevated +10m)
    create_platform(
        &mut vertices,
        &mut indices,
        15.0,
        1.0,
        Vec3::new(0.0, 10.0, 200.0),
    );

    // Slope section (Z=150-200, 30 degree incline approximation)
    // Use stepped platforms for simplicity (5 steps × 10m each)
    for i in 0..5 {
        let z = 150.0 + i as f32 * 10.0;
        let y = i as f32 * 2.0; // 2m rise per 10m run = ~11 degrees (simplified from 30)
        create_step(
            &mut vertices,
            &mut indices,
            8.0,
            10.0,
            0.3,
            Vec3::new(0.0, y, z),
        );
    }

    // Dialogue trigger markers (3 positions along path)
    // Start: Z=0, Mid: Z=100, Vista: Z=200
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 0.5, 0.0), 1.0);
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 0.5, 100.0), 1.0);
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 10.5, 200.0),
        1.0,
    );

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
        "asset": {"version": "2.0", "generator": "AstraWeave Greybox Generator v1.0 - Fractured Cliffs"},
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [0.5, 0.5, 0.5, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.8}}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": vertices.len(), "type": "VEC3", "min": [-10.0, 0.0, -5.0], "max": [10.0, 30.0, 210.0]},
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
        "../../assets/models/greybox/fractured_cliffs_greybox.gltf",
        serde_json::to_string_pretty(&gltf)?,
    )?;
    println!(
        "Success! {} vertices, {} triangles",
        vertices.len(),
        indices.len() / 3
    );
    Ok(())
}

fn create_path(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    length: f32,
    width: f32,
    height: f32,
) {
    let hw = width / 2.0;
    let b = vertices.len() as u16;

    // Path as elongated platform (top surface only for performance)
    for (x, z) in [(-hw, 0.0), (hw, 0.0), (hw, length), (-hw, length)] {
        vertices.push(Vertex {
            position: [x, height, z],
            normal: [0.0, 1.0, 0.0],
            texcoord: [(x + hw) / width, z / length],
        });
    }
    indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
}

fn create_cliff_wall(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    x_offset: f32,
    length: f32,
    height: f32,
    thickness: f32,
) {
    let b = vertices.len() as u16;
    let ht = thickness / 2.0;

    // Single tall wall face (front facing inward toward path)
    let normal = if x_offset < 0.0 {
        [1.0, 0.0, 0.0]
    } else {
        [-1.0, 0.0, 0.0]
    };
    for (y, z) in [(0.0, 0.0), (0.0, length), (height, length), (height, 0.0)] {
        vertices.push(Vertex {
            position: [x_offset, y, z],
            normal,
            texcoord: [z / length, y / height],
        });
    }
    indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
}

fn create_platform(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    size: f32,
    height: f32,
    center: Vec3,
) {
    let h = size / 2.0;
    let b = vertices.len() as u16;
    for (x, z) in [(-h, -h), (h, -h), (h, h), (-h, h)] {
        let world_pos = Vec3::new(x, height, z) + center;
        vertices.push(Vertex {
            position: [world_pos.x, world_pos.y, world_pos.z],
            normal: [0.0, 1.0, 0.0],
            texcoord: [(x + h) / size, (z + h) / size],
        });
    }
    indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
}

fn create_step(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    width: f32,
    depth: f32,
    height: f32,
    center: Vec3,
) {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let b = vertices.len() as u16;

    // Top surface only (simplified step)
    for (x, z) in [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)] {
        let world_pos = Vec3::new(x, height, z) + center;
        vertices.push(Vertex {
            position: [world_pos.x, world_pos.y, world_pos.z],
            normal: [0.0, 1.0, 0.0],
            texcoord: [(x + hw) / width, (z + hd) / depth],
        });
    }
    indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
}

fn create_cube(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, c: Vec3, size: f32) {
    let h = size / 2.0;
    let faces = [
        (
            [0.0, 0.0, 1.0],
            [[-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]],
        ),
        (
            [0.0, 0.0, -1.0],
            [[h, -h, -h], [-h, -h, -h], [-h, h, -h], [h, h, -h]],
        ),
        (
            [1.0, 0.0, 0.0],
            [[h, -h, h], [h, -h, -h], [h, h, -h], [h, h, h]],
        ),
        (
            [-1.0, 0.0, 0.0],
            [[-h, -h, -h], [-h, -h, h], [-h, h, h], [-h, h, -h]],
        ),
        (
            [0.0, 1.0, 0.0],
            [[-h, h, h], [h, h, h], [h, h, -h], [-h, h, -h]],
        ),
        (
            [0.0, -1.0, 0.0],
            [[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h]],
        ),
    ];
    for (n, ps) in &faces {
        let b = vertices.len() as u16;
        for p in ps {
            let w = Vec3::from(*p) + c;
            vertices.push(Vertex {
                position: [w.x, w.y, w.z],
                normal: *n,
                texcoord: [0.0, 0.0],
            });
        }
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
}
