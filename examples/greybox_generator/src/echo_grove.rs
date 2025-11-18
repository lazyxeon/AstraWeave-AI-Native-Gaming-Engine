// Echo Grove Greybox Generator
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
    println!("Generating Echo Grove greybox mesh...");
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // 100m x 100m ground plane (from spec: Z2 45m x 45m, but using larger for combat freedom)
    create_platform(&mut vertices, &mut indices, 100.0, 0.5);

    // Cover elements (from spec):
    // 3 large rocks (3m cubes) at strategic positions
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(-30.0, 1.5, 20.0),
        3.0,
    );
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(30.0, 1.5, -20.0),
        3.0,
    );
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(-15.0, 1.5, -30.0),
        3.0,
    );

    // 2 fallen logs (10m x 1m x 1m boxes)
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 0.5, 30.0),
        10.0,
        1.0,
        1.0,
    );
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(20.0, 0.5, 0.0),
        10.0,
        1.0,
        1.0,
    );

    // 2 tree stumps (2m cubes, approximate cylinders)
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(-40.0, 1.0, -10.0),
        2.0,
    );
    create_cube(&mut vertices, &mut indices, Vec3::new(40.0, 1.0, 10.0), 2.0);

    // Weave cover anchors (from spec: (-6, 3, 0), (8, -5, 0))
    // Small 1m cubes to mark anchor positions
    create_cube(&mut vertices, &mut indices, Vec3::new(-6.0, 0.5, 3.0), 1.0);
    create_cube(&mut vertices, &mut indices, Vec3::new(8.0, 0.5, -5.0), 1.0);

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
        "asset": {"version": "2.0", "generator": "AstraWeave Greybox Generator v1.0 - Echo Grove"},
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [0.5, 0.5, 0.5, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.8}}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": vertices.len(), "type": "VEC3", "min": [-50.0, 0.0, -50.0], "max": [50.0, 5.0, 50.0]},
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
        "../../assets/models/greybox/echo_grove_greybox.gltf",
        serde_json::to_string_pretty(&gltf)?,
    )?;
    println!(
        "Success! {} vertices, {} triangles",
        vertices.len(),
        indices.len() / 3
    );
    Ok(())
}

fn create_platform(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, size: f32, height: f32) {
    let h = size / 2.0;
    let b = vertices.len() as u16;
    for (x, z) in [(-h, -h), (h, -h), (h, h), (-h, h)] {
        vertices.push(Vertex {
            position: [x, height, z],
            normal: [0.0, 1.0, 0.0],
            texcoord: [(x + h) / size, (z + h) / size],
        });
    }
    indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    let b = vertices.len() as u16;
    for (x, z) in [(-h, -h), (h, -h), (h, h), (-h, h)] {
        vertices.push(Vertex {
            position: [x, 0.0, z],
            normal: [0.0, -1.0, 0.0],
            texcoord: [(x + h) / size, (z + h) / size],
        });
    }
    indices.extend_from_slice(&[b, b + 2, b + 1, b, b + 3, b + 2]);
    for i in 0..4 {
        let n = (i + 1) % 4;
        let t0 = b - 4 + i;
        let t1 = b - 4 + n;
        let b0 = b + i;
        let b1 = b + n;
        indices.extend_from_slice(&[t0, b0, t1, t1, b0, b1]);
    }
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
