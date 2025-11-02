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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_geometry() {
        let (verts, indices) = cube();
        // 6 faces × 4 vertices = 24 vertices
        assert_eq!(verts.len(), 24, "Cube should have 24 vertices");
        // 6 faces × 2 triangles × 3 indices = 36 indices
        assert_eq!(indices.len(), 36, "Cube should have 36 indices");
        
        // Verify all indices are valid
        for &idx in &indices {
            assert!((idx as usize) < verts.len(), "Index out of bounds");
        }
    }

    #[test]
    fn test_cube_normals() {
        let (verts, _) = cube();
        // Verify normals are unit length
        for v in &verts {
            let len_sq = v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2);
            assert!((len_sq - 1.0).abs() < 1e-5, "Normal should be unit length");
        }
        
        // Verify face normals are correct (6 faces with 4 verts each)
        for face_idx in 0..6 {
            let start = face_idx * 4;
            let first_normal = verts[start].normal;
            // All 4 vertices in a face should have same normal (flat shading)
            for i in 1..4 {
                let n = verts[start + i].normal;
                assert_eq!(n, first_normal, "Face vertices should share normal");
            }
        }
    }

    #[test]
    fn test_cube_uvs() {
        let (verts, _) = cube();
        // Verify UVs are in [0,1] range
        for v in &verts {
            assert!(v.uv[0] >= 0.0 && v.uv[0] <= 1.0, "UV.x should be in [0,1]");
            assert!(v.uv[1] >= 0.0 && v.uv[1] <= 1.0, "UV.y should be in [0,1]");
        }
        
        // Verify each face has 4 corners with proper UVs
        for face_idx in 0..6 {
            let start = face_idx * 4;
            let uvs: Vec<_> = (0..4).map(|i| verts[start + i].uv).collect();
            // Should have (0,0), (1,0), (1,1), (0,1) in some order
            assert!(uvs.contains(&[0.0, 0.0]), "Face should have (0,0) UV");
            assert!(uvs.contains(&[1.0, 0.0]), "Face should have (1,0) UV");
            assert!(uvs.contains(&[1.0, 1.0]), "Face should have (1,1) UV");
            assert!(uvs.contains(&[0.0, 1.0]), "Face should have (0,1) UV");
        }
    }

    #[test]
    fn test_plane_geometry() {
        let (verts, indices) = plane();
        // Quad with 4 vertices
        assert_eq!(verts.len(), 4, "Plane should have 4 vertices");
        // 2 triangles × 3 indices = 6 indices
        assert_eq!(indices.len(), 6, "Plane should have 6 indices");
        
        // Verify indices form 2 triangles
        assert_eq!(&indices[..], &[0, 1, 2, 0, 2, 3], "Plane indices should be correct");
    }

    #[test]
    fn test_plane_normals() {
        let (verts, _) = plane();
        // All vertices should have +Y normal (pointing up)
        for v in &verts {
            assert_eq!(v.normal, [0.0, 1.0, 0.0], "Plane normal should be +Y");
        }
    }

    #[test]
    fn test_plane_positions() {
        let (verts, _) = plane();
        // Plane should be on XZ plane (y=0) from -1 to +1
        for v in &verts {
            assert_eq!(v.position[1], 0.0, "Plane should be at y=0");
            assert!(v.position[0] >= -1.0 && v.position[0] <= 1.0, "X in [-1,1]");
            assert!(v.position[2] >= -1.0 && v.position[2] <= 1.0, "Z in [-1,1]");
        }
        
        // Verify corners are correct
        let positions: Vec<_> = verts.iter().map(|v| v.position).collect();
        assert!(positions.contains(&[-1.0, 0.0, -1.0]), "Should have corner (-1,0,-1)");
        assert!(positions.contains(&[1.0, 0.0, -1.0]), "Should have corner (1,0,-1)");
        assert!(positions.contains(&[1.0, 0.0, 1.0]), "Should have corner (1,0,1)");
        assert!(positions.contains(&[-1.0, 0.0, 1.0]), "Should have corner (-1,0,1)");
    }

    #[test]
    fn test_sphere_minimum_resolution() {
        let (verts, indices) = sphere(3, 3, 1.0);
        // (stacks+1) × (slices+1) = 4 × 4 = 16 vertices
        assert_eq!(verts.len(), 16, "3×3 sphere should have 16 vertices");
        // stacks × slices × 6 = 3 × 3 × 6 = 54 indices
        assert_eq!(indices.len(), 54, "3×3 sphere should have 54 indices");
    }

    #[test]
    fn test_sphere_radius() {
        let (verts, _) = sphere(8, 8, 2.5);
        // Verify all vertices are approximately at radius 2.5
        for v in &verts {
            let dist = (v.position[0].powi(2) + v.position[1].powi(2) + v.position[2].powi(2)).sqrt();
            assert!((dist - 2.5).abs() < 1e-4, "Vertex distance should match radius");
        }
    }

    #[test]
    fn test_sphere_normals() {
        let (verts, _) = sphere(8, 8, 1.0);
        // Normals should point outward and be unit length
        for v in &verts {
            let len_sq = v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2);
            assert!((len_sq - 1.0).abs() < 1e-5, "Normal should be unit length");
            
            // Normal should align with position (for unit sphere)
            let dot = v.normal[0] * v.position[0] 
                    + v.normal[1] * v.position[1]
                    + v.normal[2] * v.position[2];
            assert!(dot > 0.99, "Normal should point outward from center");
        }
    }

    #[test]
    fn test_sphere_uvs() {
        let (verts, _) = sphere(8, 8, 1.0);
        // Verify UVs are in [0,1] range
        for v in &verts {
            assert!(v.uv[0] >= 0.0 && v.uv[0] <= 1.0, "UV.x should be in [0,1]");
            assert!(v.uv[1] >= 0.0 && v.uv[1] <= 1.0, "UV.y should be in [0,1]");
        }
    }

    #[test]
    fn test_sphere_below_minimum() {
        // Should clamp to minimum 3 stacks/slices
        let (verts1, _) = sphere(1, 1, 1.0);
        let (verts2, _) = sphere(3, 3, 1.0);
        assert_eq!(verts1.len(), verts2.len(), "Should clamp to 3×3 minimum");
    }

    #[test]
    fn test_sphere_indices_validity() {
        let (verts, indices) = sphere(8, 8, 1.0);
        // Verify all indices are valid
        for &idx in &indices {
            assert!((idx as usize) < verts.len(), "Index should be in bounds");
        }
        
        // Verify indices form triangles (multiple of 3)
        assert_eq!(indices.len() % 3, 0, "Indices should form complete triangles");
    }
}
