use crate::mesh::{compute_tangents, CpuMesh, MeshVertex};
use anyhow::{anyhow, Result};

pub struct GltfOptions {
    pub generate_tangents: bool,
}

impl Default for GltfOptions {
    fn default() -> Self {
        Self {
            generate_tangents: true,
        }
    }
}

#[cfg(feature = "gltf-assets")]
pub fn load_gltf(path: &std::path::Path, opts: &GltfOptions) -> Result<Vec<CpuMesh>> {
    use gltf::mesh::util::ReadIndices;
    let (doc, buffers, _images) = gltf::import(path)?;
    let mut meshes_out = Vec::new();
    for mesh in doc.meshes() {
        for prim in mesh.primitives() {
            let reader = prim.reader(|b| Some(&buffers[b.index()]));
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| anyhow!("missing POSITION"))?
                .collect();
            let normals: Vec<[f32; 3]> = match reader.read_normals() {
                Some(iter) => iter.collect(),
                None => positions.iter().map(|_| [0.0f32, 1.0, 0.0]).collect(),
            };
            let tangents_opt: Option<Vec<[f32; 4]>> = reader.read_tangents().map(|i| i.collect());
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect())
                .unwrap_or_else(|| positions.iter().map(|_| [0.0f32, 0.0]).collect());
            let indices: Vec<u32> = match reader.read_indices() {
                Some(ReadIndices::U16(i)) => i.map(|x| x as u32).collect(),
                Some(ReadIndices::U32(i)) => i.collect(),
                Some(ReadIndices::U8(i)) => i.map(|x| x as u32).collect(),
                None => (0..positions.len() as u32).collect(),
            };

            let mut cpu = CpuMesh {
                vertices: Vec::with_capacity(positions.len()),
                indices,
            };
            if let Some(t) = tangents_opt {
                for ((p, n), (u, t4)) in positions
                    .iter()
                    .zip(normals.iter())
                    .zip(uvs.iter().zip(t.iter()))
                {
                    cpu.vertices.push(MeshVertex::from_arrays(*p, *n, *t4, *u));
                }
            } else {
                // Fill with placeholder tangent; can be generated later
                for ((p, n), u) in positions.iter().zip(normals.iter()).zip(uvs.iter()) {
                    cpu.vertices
                        .push(MeshVertex::from_arrays(*p, *n, [1.0, 0.0, 0.0, 1.0], *u));
                }
                if opts.generate_tangents {
                    generate_mikktspace_tangents(&mut cpu)?;
                }
            }
            meshes_out.push(cpu);
        }
    }
    Ok(meshes_out)
}

#[cfg(not(feature = "gltf-assets"))]
pub fn load_gltf(_path: &std::path::Path, _opts: &GltfOptions) -> Result<Vec<CpuMesh>> {
    Err(anyhow!("gltf-assets feature not enabled"))
}

#[cfg(feature = "gltf-assets")]
pub fn generate_mikktspace_tangents(mesh: &mut CpuMesh) -> Result<()> {
    compute_tangents(mesh);
    Ok(())
}

#[cfg(not(feature = "gltf-assets"))]
pub fn generate_mikktspace_tangents(_mesh: &mut CpuMesh) -> Result<()> {
    Ok(())
}
