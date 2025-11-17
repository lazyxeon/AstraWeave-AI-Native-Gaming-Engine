// Loomspire Sanctum Greybox Generator
use anyhow::Result;
use base64::Engine;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::fs;

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Vertex { position: [f32; 3], normal: [f32; 3], texcoord: [f32; 2] }
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

fn main() -> Result<()> {
    println!("Generating Loomspire Sanctum greybox mesh...");
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    create_platform(&mut vertices, &mut indices, 20.0, 1.0);
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 1.0, 0.0), 2.0);
    
    let mut buffer_data = Vec::new();
    for v in &vertices { buffer_data.extend_from_slice(bytemuck::bytes_of(v)); }
    for i in &indices { buffer_data.extend_from_slice(&i.to_le_bytes()); }
    let buffer_base64 = base64::engine::general_purpose::STANDARD.encode(&buffer_data);
    let buffer_uri = format!("data:application/octet-stream;base64,{}", buffer_base64);
    
    let gltf = serde_json::json!({
        "asset": {"version": "2.0"}, "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [0.5, 0.5, 0.5, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.8}}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": vertices.len(), "type": "VEC3", "min": [-10.0, 0.0, -10.0], "max": [10.0, 3.0, 10.0]},
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
    fs::write("../../assets/models/greybox/loomspire_sanctum_greybox.gltf", serde_json::to_string_pretty(&gltf)?)?;
    println!("Success! {} vertices, {} triangles", vertices.len(), indices.len() / 3);
    Ok(())
}

fn create_platform(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, size: f32, height: f32) {
    let h = size / 2.0; let b = vertices.len() as u16;
    for (x, z) in [(-h, -h), (h, -h), (h, h), (-h, h)] {
        vertices.push(Vertex { position: [x, height, z], normal: [0.0, 1.0, 0.0], texcoord: [(x+h)/size, (z+h)/size] });
    }
    indices.extend_from_slice(&[b, b+1, b+2, b, b+2, b+3]);
    let b = vertices.len() as u16;
    for (x, z) in [(-h, -h), (h, -h), (h, h), (-h, h)] {
        vertices.push(Vertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], texcoord: [(x+h)/size, (z+h)/size] });
    }
    indices.extend_from_slice(&[b, b+2, b+1, b, b+3, b+2]);
    for i in 0..4 {
        let n = (i + 1) % 4; let t0 = b - 4 + i; let t1 = b - 4 + n; let b0 = b + i; let b1 = b + n;
        indices.extend_from_slice(&[t0, b0, t1, t1, b0, b1]);
    }
}

fn create_cube(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, c: Vec3, size: f32) {
    let h = size / 2.0;
    let faces = [
        ([0.0, 0.0, 1.0], [[-h,-h,h], [h,-h,h], [h,h,h], [-h,h,h]]),
        ([0.0, 0.0, -1.0], [[h,-h,-h], [-h,-h,-h], [-h,h,-h], [h,h,-h]]),
        ([1.0, 0.0, 0.0], [[h,-h,h], [h,-h,-h], [h,h,-h], [h,h,h]]),
        ([-1.0, 0.0, 0.0], [[-h,-h,-h], [-h,-h,h], [-h,h,h], [-h,h,-h]]),
        ([0.0, 1.0, 0.0], [[-h,h,h], [h,h,h], [h,h,-h], [-h,h,-h]]),
        ([0.0, -1.0, 0.0], [[-h,-h,-h], [h,-h,-h], [h,-h,h], [-h,-h,h]]),
    ];
    for (n, ps) in &faces {
        let b = vertices.len() as u16;
        for p in ps { let w = Vec3::from(*p) + c; vertices.push(Vertex { position: [w.x, w.y, w.z], normal: *n, texcoord: [0.0, 0.0] }); }
        indices.extend_from_slice(&[b, b+1, b+2, b, b+2, b+3]);
    }
}
