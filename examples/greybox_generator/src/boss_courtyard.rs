// Boss Courtyard (Z4) Greybox Generator
// 55m × 55m octagonal arena at +28m elevation with gravity pylons,
// arena lip, binding sigils, and thread anchors
use anyhow::Result;
use base64::Engine;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
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
    println!("Generating Boss Courtyard greybox mesh...");
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Octagonal arena platform at Y=28, radius ~22m (inscribed in 55m bounding box)
    // 8-sided polygon approximation
    create_octagon_platform(&mut vertices, &mut indices, 22.0, 28.0, 3.0);

    // Arena lip (1.5m high rim around the outer edge)
    // Approximate as 8 box segments around the perimeter
    create_arena_lip(&mut vertices, &mut indices, 22.0, 28.0, 1.5, 2.0);

    // South entrance corridor (connects to Z3): 6m wide × 10m long
    create_box(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 28.25, -25.0),
        6.0,
        0.5,
        10.0,
    );

    // --- Gravity Pylons (3 at 120° intervals) ---
    // Pylon NE: (9, 28, 12)
    create_pillar(
        &mut vertices,
        &mut indices,
        Vec3::new(9.0, 28.0, 12.0),
        2.0,
        8.0,
    );
    // Pylon NW: (-9, 28, 12)
    create_pillar(
        &mut vertices,
        &mut indices,
        Vec3::new(-9.0, 28.0, 12.0),
        2.0,
        8.0,
    );
    // Pylon S: (0, 28, -12)
    create_pillar(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 28.0, -12.0),
        2.0,
        8.0,
    );

    // --- Thread Anchors (2 repairable, east + west) ---
    // East anchor marker
    create_cube(&mut vertices, &mut indices, Vec3::new(10.0, 29.0, 0.0), 1.5);
    // West anchor marker
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(-10.0, 29.0, 0.0),
        1.5,
    );

    // --- Binding Sigils (3 breakable markers near pylons) ---
    // Sigil NE: (9, 29.5, 12)
    create_cube(&mut vertices, &mut indices, Vec3::new(9.0, 29.5, 12.0), 0.8);
    // Sigil NW: (-9, 29.5, 12)
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(-9.0, 29.5, 12.0),
        0.8,
    );
    // Sigil S: (0, 29.5, -12)
    create_cube(
        &mut vertices,
        &mut indices,
        Vec3::new(0.0, 29.5, -12.0),
        0.8,
    );

    // --- Boss spawn pedestal (center-north) ---
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 28.75, 8.0), 3.0);

    // --- Center nexus marker (phase transition focal point) ---
    create_cube(&mut vertices, &mut indices, Vec3::new(0.0, 28.5, 0.0), 1.0);

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
        "asset": {"version": "2.0", "generator": "AstraWeave Greybox Generator v1.0 - Boss Courtyard"},
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": 3, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [0.4, 0.35, 0.45, 1.0], "metallicFactor": 0.2, "roughnessFactor": 0.6}}],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": vertices.len(), "type": "VEC3", "min": [-28.0, 25.0, -30.0], "max": [28.0, 37.0, 30.0]},
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
        "../../assets/models/greybox/boss_courtyard_greybox.gltf",
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

/// Regular octagon platform with top, bottom, and side faces
fn create_octagon_platform(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    radius: f32,
    elevation: f32,
    thickness: f32,
) {
    let top = elevation;
    let bottom = elevation - thickness;
    let n_sides = 8u16;

    // Compute octagon corners
    let corners: Vec<(f32, f32)> = (0..n_sides)
        .map(|i| {
            let angle = 2.0 * PI * i as f32 / n_sides as f32 - PI / 2.0; // Start from south
            (radius * angle.cos(), radius * angle.sin())
        })
        .collect();

    // Top face (fan from center)
    let center_idx = vertices.len() as u16;
    vertices.push(Vertex {
        position: [0.0, top, 0.0],
        normal: [0.0, 1.0, 0.0],
        texcoord: [0.5, 0.5],
    });
    for (x, z) in &corners {
        vertices.push(Vertex {
            position: [*x, top, *z],
            normal: [0.0, 1.0, 0.0],
            texcoord: [(x / radius + 1.0) / 2.0, (z / radius + 1.0) / 2.0],
        });
    }
    for i in 0..n_sides {
        let next = (i + 1) % n_sides;
        indices.extend_from_slice(&[center_idx, center_idx + 1 + i, center_idx + 1 + next]);
    }

    // Bottom face (fan from center, reversed winding)
    let center_bot = vertices.len() as u16;
    vertices.push(Vertex {
        position: [0.0, bottom, 0.0],
        normal: [0.0, -1.0, 0.0],
        texcoord: [0.5, 0.5],
    });
    for (x, z) in &corners {
        vertices.push(Vertex {
            position: [*x, bottom, *z],
            normal: [0.0, -1.0, 0.0],
            texcoord: [(x / radius + 1.0) / 2.0, (z / radius + 1.0) / 2.0],
        });
    }
    for i in 0..n_sides {
        let next = (i + 1) % n_sides;
        indices.extend_from_slice(&[center_bot, center_bot + 1 + next, center_bot + 1 + i]);
    }

    // Side faces (8 quads)
    for i in 0..n_sides {
        let next = (i + 1) % n_sides;
        let (x0, z0) = corners[i as usize];
        let (x1, z1) = corners[next as usize];

        // Outward normal
        let dx = x1 - x0;
        let dz = z1 - z0;
        let len = (dx * dx + dz * dz).sqrt();
        let nx = dz / len;
        let nz = -dx / len;

        let b = vertices.len() as u16;
        vertices.push(Vertex {
            position: [x0, top, z0],
            normal: [nx, 0.0, nz],
            texcoord: [0.0, 1.0],
        });
        vertices.push(Vertex {
            position: [x1, top, z1],
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

/// Arena lip: 1.5m high wall segments around the octagonal perimeter
fn create_arena_lip(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    inner_radius: f32,
    elevation: f32,
    lip_height: f32,
    lip_thickness: f32,
) {
    let n_sides = 8u16;
    let outer_radius = inner_radius + lip_thickness;

    for i in 0..n_sides {
        let angle0 = 2.0 * PI * i as f32 / n_sides as f32 - PI / 2.0;
        let angle1 = 2.0 * PI * ((i + 1) % n_sides) as f32 / n_sides as f32 - PI / 2.0;

        let ix0 = inner_radius * angle0.cos();
        let iz0 = inner_radius * angle0.sin();
        let ix1 = inner_radius * angle1.cos();
        let iz1 = inner_radius * angle1.sin();

        let ox0 = outer_radius * angle0.cos();
        let oz0 = outer_radius * angle0.sin();
        let ox1 = outer_radius * angle1.cos();
        let oz1 = outer_radius * angle1.sin();

        let y_bot = elevation;
        let y_top = elevation + lip_height;

        // Inner face (facing inward)
        let dx = ix1 - ix0;
        let dz = iz1 - iz0;
        let len = (dx * dx + dz * dz).sqrt();
        let nx = -dz / len;
        let nz = dx / len;

        let b = vertices.len() as u16;
        vertices.push(Vertex {
            position: [ix0, y_bot, iz0],
            normal: [nx, 0.0, nz],
            texcoord: [0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [ix1, y_bot, iz1],
            normal: [nx, 0.0, nz],
            texcoord: [1.0, 0.0],
        });
        vertices.push(Vertex {
            position: [ix1, y_top, iz1],
            normal: [nx, 0.0, nz],
            texcoord: [1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [ix0, y_top, iz0],
            normal: [nx, 0.0, nz],
            texcoord: [0.0, 1.0],
        });
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        // Outer face (facing outward)
        let b = vertices.len() as u16;
        vertices.push(Vertex {
            position: [ox0, y_bot, oz0],
            normal: [-nx, 0.0, -nz],
            texcoord: [0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [ox1, y_bot, oz1],
            normal: [-nx, 0.0, -nz],
            texcoord: [1.0, 0.0],
        });
        vertices.push(Vertex {
            position: [ox1, y_top, oz1],
            normal: [-nx, 0.0, -nz],
            texcoord: [1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [ox0, y_top, oz0],
            normal: [-nx, 0.0, -nz],
            texcoord: [0.0, 1.0],
        });
        indices.extend_from_slice(&[b, b + 2, b + 1, b, b + 3, b + 2]);

        // Top face of lip segment
        let b = vertices.len() as u16;
        vertices.push(Vertex {
            position: [ix0, y_top, iz0],
            normal: [0.0, 1.0, 0.0],
            texcoord: [0.0, 0.0],
        });
        vertices.push(Vertex {
            position: [ix1, y_top, iz1],
            normal: [0.0, 1.0, 0.0],
            texcoord: [1.0, 0.0],
        });
        vertices.push(Vertex {
            position: [ox1, y_top, oz1],
            normal: [0.0, 1.0, 0.0],
            texcoord: [1.0, 1.0],
        });
        vertices.push(Vertex {
            position: [ox0, y_top, oz0],
            normal: [0.0, 1.0, 0.0],
            texcoord: [0.0, 1.0],
        });
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
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

/// Pillar (tall box approximating a cylinder at ground level)
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
