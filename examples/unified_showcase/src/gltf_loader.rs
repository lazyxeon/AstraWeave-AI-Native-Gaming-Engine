/// GLTF/GLB loader for unified_showcase
/// Loads 3D models and converts them to wgpu mesh format
use anyhow::{Context, Result};
use glam::Vec3;
use std::path::Path;

/// Vertex format with vertex colors (for Kenney models)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GltfVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],   // Vertex color (RGBA)
    pub tangent: [f32; 4], // Tangent (xyz) + handedness (w)
}

/// Loaded mesh data ready for GPU upload
pub struct LoadedMesh {
    pub vertices: Vec<GltfVertex>,
    pub indices: Vec<u32>,
    pub name: String,
    pub material_name: Option<String>,
}

/// Compute AABB (axis-aligned bounding box) for a set of positions
fn compute_aabb(positions: &[[f32; 3]]) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);

    for pos in positions {
        let p = Vec3::from(*pos);
        min = min.min(p);
        max = max.max(p);
    }

    (min, max)
}

/// Compute per-vertex tangent frames (tangent.x, tangent.y, tangent.z, handedness)
fn compute_tangents(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    normals: &[[f32; 3]],
    indices: &[u32],
) -> Vec<[f32; 4]> {
    let mut tangents: Vec<Vec3> = vec![Vec3::ZERO; positions.len()];
    let mut bitangents: Vec<Vec3> = vec![Vec3::ZERO; positions.len()];

    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let p0 = Vec3::from(positions[i0]);
        let p1 = Vec3::from(positions[i1]);
        let p2 = Vec3::from(positions[i2]);

        let uv0 = Vec3::new(uvs[i0][0], uvs[i0][1], 0.0);
        let uv1 = Vec3::new(uvs[i1][0], uvs[i1][1], 0.0);
        let uv2 = Vec3::new(uvs[i2][0], uvs[i2][1], 0.0);

        let dp1 = p1 - p0;
        let dp2 = p2 - p0;
        let duv1 = uv1 - uv0;
        let duv2 = uv2 - uv0;

        let r = 1.0 / (duv1.x * duv2.y - duv2.x * duv1.y + 1e-8);
        let tangent = (dp1 * duv2.y - dp2 * duv1.y) * r;
        let bitangent = (dp2 * duv1.x - dp1 * duv2.x) * r;

        tangents[i0] += tangent;
        tangents[i1] += tangent;
        tangents[i2] += tangent;
        bitangents[i0] += bitangent;
        bitangents[i1] += bitangent;
        bitangents[i2] += bitangent;
    }

    tangents
        .into_iter()
        .zip(bitangents.into_iter())
        .zip(normals.iter())
        .map(|((t, b), n)| {
            let t = t.normalize_or_zero();
            let b = b.normalize_or_zero();
            let n = Vec3::from(*n);
            let w = if n.cross(t).dot(b) < 0.0 { -1.0 } else { 1.0 };
            [t.x, t.y, t.z, w]
        })
        .collect()
}

/// Generate planar UVs based on XZ projection (suitable for terrain and buildings)
fn generate_planar_uvs(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let (min, max) = compute_aabb(positions);
    let size = max - min;

    // Avoid division by zero
    let size_x = if size.x.abs() < 0.001 { 1.0 } else { size.x };
    let size_z = if size.z.abs() < 0.001 { 1.0 } else { size.z };

    positions
        .iter()
        .map(|pos| {
            let p = Vec3::from(*pos);
            let u = (p.x - min.x) / size_x;
            let v = (p.z - min.z) / size_z;
            [u, v]
        })
        .collect()
}

/// Generate smooth per-vertex normals by averaging adjacent face normals
fn generate_smooth_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals_accum: Vec<Vec3> = vec![Vec3::ZERO; positions.len()];

    // Accumulate face normals for each vertex
    for triangle in indices.chunks(3) {
        if triangle.len() < 3 {
            continue;
        }

        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        let v0 = Vec3::from(positions[i0]);
        let v1 = Vec3::from(positions[i1]);
        let v2 = Vec3::from(positions[i2]);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2);

        // Accumulate weighted by face area (magnitude of cross product)
        normals_accum[i0] += face_normal;
        normals_accum[i1] += face_normal;
        normals_accum[i2] += face_normal;
    }

    // Normalize accumulated normals
    normals_accum
        .iter()
        .map(|n| {
            let normalized = n.normalize_or_zero();
            // Fallback to up vector if normalization fails
            if normalized.length_squared() < 0.001 {
                [0.0, 1.0, 0.0]
            } else {
                normalized.to_array()
            }
        })
        .collect()
}

/// Load a GLTF/GLB file and extract ALL primitives (merged into single mesh)
/// This is critical for models like Kenney trees where trunk and leaves are separate primitives.
pub fn load_gltf(path: impl AsRef<Path>) -> Result<LoadedMesh> {
    let path = path.as_ref();

    log::info!("Attempting to load GLTF: {}", path.display());

    // Try to import via path first (convenience), but if it fails, read file and
    // attempt import_slice - this can provide more detailed diagnostics
    let import_res = match gltf::import(path) {
        Ok(x) => Ok(x),
        Err(e) => {
            log::warn!(
                "gltf::import(path) failed for {}: {:?}, attempting import_slice fallback",
                path.display(),
                e
            );
            // Try to read file bytes and import from memory to capture more granular error
            match std::fs::read(path) {
                Ok(bytes) => {
                    log::info!("  Read file: {} bytes", bytes.len());
                    if bytes.len() >= 4 {
                        log::info!("  Header: {:02x?}", &bytes[..4]);
                    }
                    gltf::import_slice(&bytes)
                }
                Err(err) => {
                    log::error!("Failed to read GLTF file {}: {}", path.display(), err);
                    Err(e)
                }
            }
        }
    };
    if let Err(ref err) = import_res {
        // Print detailed diagnostic error chain for debugging
        log::error!("gltf::import failed for {}: {:?}", path.display(), err);
    }
    let (document, buffers, _images) =
        import_res.with_context(|| format!("Failed to load GLTF file: {}", path.display()))?;

    log::info!("GLTF loaded successfully, parsing meshes...");

    // Collect ALL primitives from ALL meshes
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut mesh_name = String::from("unnamed");

    for mesh in document.meshes() {
        mesh_name = mesh.name().unwrap_or("unnamed").to_string();
        log::info!(
            "Processing mesh: '{}' with {} primitives",
            mesh_name,
            mesh.primitives().len()
        );

        for (prim_idx, primitive) in mesh.primitives().enumerate() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .with_context(|| format!("No position data in primitive {}", prim_idx))?
                .collect();

            let indices: Vec<u32> = reader
                .read_indices()
                .with_context(|| format!("No index data in primitive {}", prim_idx))?
                .into_u32()
                .collect();

            // Extract or generate normals
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| {
                    log::info!(
                        "  Primitive {}: No normals found, generating smooth normals",
                        prim_idx
                    );
                    generate_smooth_normals(&positions, &indices)
                });

            // Extract or generate UVs (CRITICAL: Preserve original model UVs!)
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| {
                    log::info!(
                        "  Primitive {}: No UVs found, generating planar UVs",
                        prim_idx
                    );
                    generate_planar_uvs(&positions)
                });

            // Extract vertex colors (Kenney models use these!)
            let colors: Vec<[f32; 4]> = reader
                .read_colors(0)
                .map(|iter| iter.into_rgba_f32().collect())
                .unwrap_or_else(|| {
                    log::info!("  Primitive {}: No vertex colors, using white", prim_idx);
                    vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]
                });

            // Extract or compute tangents
            let tangents: Vec<[f32; 4]> = if let Some(tangents_iter) = reader.read_tangents() {
                tangents_iter.map(|t| [t[0], t[1], t[2], t[3]]).collect()
            } else {
                log::info!(
                    "  Primitive {}: No tangents found, computing tangents",
                    prim_idx
                );
                compute_tangents(&positions, &uvs, &normals, &indices)
            };

            // Offset indices by current vertex count (for merging primitives)
            let index_offset = all_vertices.len() as u32;

            // Store triangle count before moving indices
            let triangle_count = indices.len() / 3;

            // Append vertices with colors
            for i in 0..positions.len() {
                all_vertices.push(GltfVertex {
                    position: positions[i],
                    normal: normals[i],
                    uv: uvs[i],
                    color: colors[i],
                    tangent: tangents[i],
                });
            }

            // Append indices (with offset)
            for idx in indices {
                all_indices.push(idx + index_offset);
            }

            log::info!(
                "  Primitive {}: {} vertices, {} triangles",
                prim_idx,
                positions.len(),
                triangle_count
            );
        }
    }

    if all_vertices.is_empty() {
        anyhow::bail!("No valid primitives found in GLTF file");
    }

    log::info!(
        "Loaded GLTF mesh '{}': {} total vertices, {} total triangles (from {} primitives)",
        mesh_name,
        all_vertices.len(),
        all_indices.len() / 3,
        document
            .meshes()
            .map(|m| m.primitives().len())
            .sum::<usize>()
    );

    Ok(LoadedMesh {
        vertices: all_vertices,
        indices: all_indices,
        name: mesh_name,
    })
}

/// Load multiple meshes from a GLTF file
#[allow(dead_code)]
pub fn load_gltf_all_meshes(path: impl AsRef<Path>) -> Result<Vec<LoadedMesh>> {
    let path = path.as_ref();
    let (document, buffers, _images) = gltf::import(path)
        .with_context(|| format!("Failed to load GLTF file: {}", path.display()))?;

    let mut loaded_meshes = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .context("No position data in mesh")?
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let colors: Vec<[f32; 4]> = reader
                .read_colors(0)
                .map(|iter| iter.into_rgba_f32().collect())
                .unwrap_or_else(|| vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]);

            let indices: Vec<u32> = reader
                .read_indices()
                .context("No index data in mesh")?
                .into_u32()
                .collect();

            let tangents: Vec<[f32; 4]> = if let Some(titer) = reader.read_tangents() {
                titer.map(|t| [t[0], t[1], t[2], t[3]]).collect()
            } else {
                compute_tangents(&positions, &uvs, &normals, &indices)
            };

            let vertices: Vec<GltfVertex> = positions
                .iter()
                .zip(normals.iter())
                .zip(uvs.iter())
                .zip(colors.iter())
                .map(|(((pos, normal), uv), color)| GltfVertex {
                    position: *pos,
                    normal: *normal,
                    uv: *uv,
                    color: *color,
                    tangent: [1.0, 0.0, 0.0, 1.0], // default, replaced below
                })
                .collect();

            // Fill actual tangents (compute_tangents produced one per-vertex)
            let mut vertices_with_tangents: Vec<GltfVertex> = vertices
                .into_iter()
                .enumerate()
                .map(|(i, mut v)| {
                    v.tangent = tangents[i];
                    v
                })
                .collect();

            // indices already read earlier

            log::info!(
                "Loaded mesh '{}': {} vertices, {} triangles",
                mesh.name().unwrap_or("unnamed"),
                vertices_with_tangents.len(),
                indices.len() / 3
            );

            loaded_meshes.push(LoadedMesh {
                vertices: vertices_with_tangents,
                indices,
                name: mesh.name().unwrap_or("unnamed").to_string(),
            });
        }
    }

    Ok(loaded_meshes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_demo_plane() {
        // This test requires the demo_plane.gltf file to exist
        let result = load_gltf("../../assets/demo_plane.gltf");
        if result.is_ok() {
            let mesh = result.unwrap();
            assert!(!mesh.vertices.is_empty(), "Should have vertices");
            assert!(!mesh.indices.is_empty(), "Should have indices");
            assert_eq!(mesh.indices.len() % 3, 0, "Indices should be triangles");
        }
    }
}
