// Loom Crossroads (Z3) Greybox Generator
// 35m × 30m elevated hexagonal platform with 3 conduit pillars at 120° intervals
// Storm-routing narrative choice point
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
    println!("Generating Loom Crossroads greybox mesh...");
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Main platform: 35m × 30m at +20m elevation
    // Approximated as rectangle for greybox phase (hexagonal in final art pass)
    create_platform(&mut vertices, &mut indices, 35.0, 30.0, 20.0);

    // Entry ramp from Z2 (south side): 6m wide × 12m long, slopes from Y=14 to Y=20
    create_ramp(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 14.0, -15.0),
        6.0,
        12.0,
        6.0,
    );

    // Exit corridor to Z4 (north side): 4m wide × 8m long at Y=20
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 20.25, 19.0),
        4.0,
        0.5,
        8.0,
    );

    // --- Conduit Pillars at 120° intervals ---
    // Conduit North: (0, 20, 8)
    create_pillar(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 20.0, 8.0),
        1.5,
        6.0,
    );

    // Conduit Southeast: (6.93, 20, -4) — 120° from north
    create_pillar(
        &mut vertices,
        &mut indices,
        Vec3::new(6.93, 20.0, -4.0),
        1.5,
        6.0,
    );

    // Conduit Southwest: (-6.93, 20, -4) — 240° from north
    create_pillar(
        &mut vertices,
        &mut indices,
        Vec3::new(-6.93, 20.0, -4.0),
        1.5,
        6.0,
    );

    // --- Storm conduit nexus at center (decision point marker) ---
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 21.5, 0.0), 2.0);

    // --- Low walls / railings along platform edges (1m high) ---
    // North edge railing
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 20.5, 15.0),
        35.0,
        1.0,
        0.5,
    );
    // East edge railing
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(17.5, 20.5, 0.0),
        0.5,
        1.0,
        30.0,
    );
    // West edge railing
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(-17.5, 20.5, 0.0),
        0.5,
        1.0,
        30.0,
    );

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
        "asset": {"version": "2.0", "generator": "AstraWeave Greybox Generator v1.0 - Loom Crossroads"},
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [0.6, 0.45, 0.55, 1.0], "metallicFactor": 0.1, "roughnessFactor": 0.7}}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": vertices.len(), "type": "VEC3", "min": [-20.0, 14.0, -27.0], "max": [20.0, 27.0, 27.0]},
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
        "../../assets/models/greybox/loom_crossroads_greybox.gltf",
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

/// Rectangular platform (top + bottom + sides) at given elevation
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

    // Bottom face (3m thick slab)
    let bottom = elevation - 3.0;
    let b = vertices.len() as u16;
    for (x, z) in [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)] {
        vertices.push(Vertex {
            position: [x, bottom, z],
            normal: [0.0, -1.0, 0.0],
            texcoord: [(x + hw) / width, (z + hd) / depth],
        });
    }
    indices.extend_from_slice(&[b, b + 2, b + 1, b, b + 3, b + 2]);

    // Side faces
    let corners = [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)];
    for i in 0..4 {
        let (x0, z0) = corners[i];
        let (x1, z1) = corners[(i + 1) % 4];
        let dx = x1 - x0;
        let dz = z1 - z0;
        let len = (dx * dx + dz * dz).sqrt();
        let nx = dz / len;
        let nz = -dx / len;
        let b = vertices.len() as u16;
        vertices.push(Vertex {
            position: [x0, elevation, z0],
            normal: [nx, 0.0, nz],
            texcoord: [0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [x1, elevation, z1],
            normal: [nx, 0.0, nz],
            texcoord: [1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [x1, bottom, z1],
            normal: [nx, 0.0, nz],
            texcoord: [1.0, 0.0],
        });
        vertices.push(Vertex {
            position: [x0, bottom, z0],
            normal: [nx, 0.0, nz],
            texcoord: [0.0, 0.0],
        });
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
}

/// Sloped ramp: starts at `origin.y`, rises by `rise` over `length` in +Z direction
fn create_ramp(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    origin: Vec3,
    width: f32,
    length: f32,
    rise: f32,
) {
    let hw = width / 2.0;
    let b = vertices.len() as u16;

    // Ramp surface (4 corners: 2 at bottom-south, 2 at top-north)
    let y_lo = origin.y;
    let y_hi = origin.y + rise;
    let z_lo = origin.z;
    let z_hi = origin.z + length;

    // Normal: perpendicular to slope surface (approximation)
    let slope_len = (length * length + rise * rise).sqrt();
    let ny = length / slope_len;
    let nz = -rise / slope_len;

    vertices.push(Vertex {
        position: [origin.x - hw, y_lo, z_lo],
        normal: [0.0, ny, nz],
        texcoord: [0.0, 0.0],
    });
    vertices.push(Vertex {
        position: [origin.x + hw, y_lo, z_lo],
        normal: [0.0, ny, nz],
        texcoord: [1.0, 0.0],
    });
    vertices.push(Vertex {
        position: [origin.x + hw, y_hi, z_hi],
        normal: [0.0, ny, nz],
        texcoord: [1.0, 1.0],
    });
    vertices.push(Vertex {
        position: [origin.x - hw, y_hi, z_hi],
        normal: [0.0, ny, nz],
        texcoord: [0.0, 1.0],
    });
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

/// Pillar (tall narrow box approximating a cylinder)
fn create_pillar(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    base: Vec3,
    radius: f32,
    height: f32,
) {
    let center = Vec3::new(base.x, base.y + height / 2.0, base.z);
    let diameter = radius * 2.0;
    create_box(vertices, indices, center, diameter, height, diameter);
}
