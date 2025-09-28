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
        // Simple tangent aligned with +X by default, handedness +1
        let tangent = [1.0, 0.0, 0.0, 1.0];
        // Projected cube-mapped UVs per face quad (0..1). We'll assign based on vertex within the face.
        let corner = (idx % 4) as u32;
        let uv = match corner {
            0 => [0.0, 0.0],
            1 => [1.0, 0.0],
            2 => [1.0, 1.0],
            _ => [0.0, 1.0],
        };
        v.push(Vertex {
            position: *p,
            normal: *n,
            tangent,
            uv,
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
    let n = [0.0, 1.0, 0.0];
    let t = [1.0, 0.0, 0.0, 1.0];
    let v = vec![
        Vertex {
            position: [-1.0, 0.0, -1.0],
            normal: n,
            tangent: t,
            uv: [0.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, -1.0],
            normal: n,
            tangent: t,
            uv: [1.0, 0.0],
        },
        Vertex {
            position: [1.0, 0.0, 1.0],
            normal: n,
            tangent: t,
            uv: [1.0, 1.0],
        },
        Vertex {
            position: [-1.0, 0.0, 1.0],
            normal: n,
            tangent: t,
            uv: [0.0, 1.0],
        },
    ];
    let i = vec![0, 1, 2, 0, 2, 3];
    (v, i)
}

/// UV sphere centered at origin
pub fn sphere(stacks: u32, slices: u32, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
    let stacks = stacks.max(3);
    let slices = slices.max(3);
    let mut verts: Vec<Vertex> = Vec::with_capacity(((stacks + 1) * (slices + 1)) as usize);
    let mut indices: Vec<u32> = Vec::with_capacity((stacks * slices * 6) as usize);

    for i in 0..=stacks {
        let v = i as f32 / stacks as f32; // [0,1]
        let phi = v * std::f32::consts::PI; // [0, PI]
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        for j in 0..=slices {
            let u = j as f32 / slices as f32; // [0,1]
            let theta = u * std::f32::consts::PI * 2.0; // [0, 2PI]
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let nx = sin_phi * cos_theta;
            let ny = cos_phi;
            let nz = sin_phi * sin_theta;
            let px = radius * nx;
            let py = radius * ny;
            let pz = radius * nz;
            // Tangent is derivative w.r.t. theta on the sphere, approximate along longitude
            let tx = -sin_theta;
            let ty = 0.0;
            let tz = cos_theta;
            let tangent = [tx, ty, tz, 1.0];
            let uv = [u, 1.0 - v];
            verts.push(Vertex {
                position: [px, py, pz],
                normal: [nx, ny, nz],
                tangent,
                uv,
            });
        }
    }

    let row = slices + 1;
    for i in 0..stacks {
        for j in 0..slices {
            let a = i * row + j;
            let b = a + 1;
            let c = (i + 1) * row + j;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    (verts, indices)
}
