use crate::types::Vertex;

/// Unit cube centered at origin, with face normals (no UVs).
pub fn cube() -> (Vec<Vertex>, Vec<u32>) {
    // positions and normals for each face (duplicated vertices for flat shading)
    let mut v = Vec::new();
    let mut i = Vec::new();
    let faces = [
        // +X
        ([1.0, -1.0, -1.0], [1.0, 0.0, 0.0]),
        ([1.0, 1.0, -1.0], [1.0, 0.0, 0.0]),
        ([1.0, 1.0, 1.0], [1.0, 0.0, 0.0]),
        ([1.0, -1.0, 1.0], [1.0, 0.0, 0.0]),
        // -X
        ([-1.0, -1.0, 1.0], [-1.0, 0.0, 0.0]),
        ([-1.0, 1.0, 1.0], [-1.0, 0.0, 0.0]),
        ([-1.0, 1.0, -1.0], [-1.0, 0.0, 0.0]),
        ([-1.0, -1.0, -1.0], [-1.0, 0.0, 0.0]),
        // +Y
        ([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
        ([1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
        ([1.0, 1.0, 1.0], [0.0, 1.0, 0.0]),
        ([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0]),
        // -Y
        ([-1.0, -1.0, 1.0], [0.0, -1.0, 0.0]),
        ([1.0, -1.0, 1.0], [0.0, -1.0, 0.0]),
        ([1.0, -1.0, -1.0], [0.0, -1.0, 0.0]),
        ([-1.0, -1.0, -1.0], [0.0, -1.0, 0.0]),
        // +Z
        ([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        ([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
        ([1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
        ([1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        // -Z
        ([1.0, -1.0, -1.0], [0.0, 0.0, -1.0]),
        ([1.0, 1.0, -1.0], [0.0, 0.0, -1.0]),
        ([-1.0, 1.0, -1.0], [0.0, 0.0, -1.0]),
        ([-1.0, -1.0, -1.0], [0.0, 0.0, -1.0]),
    ];
    for (idx, (p, n)) in faces.iter().enumerate() {
        v.push(Vertex {
            position: *p,
            normal: *n,
        });
        if idx % 4 == 3 {
            let base = idx as u32 - 3;
            i.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    (v, i)
}

/// Ground plane (square) on XZ at y=0, size 1 (we’ll scale it up at instance time).
pub fn plane() -> (Vec<Vertex>, Vec<u32>) {
    let v = vec![
        Vertex {
            position: [-1.0, 0.0, -1.0],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, -1.0],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            normal: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [-1.0, 0.0, 1.0],
            normal: [0.0, 1.0, 0.0],
        },
    ];
    let i = vec![0, 1, 2, 0, 2, 3];
    (v, i)
}
