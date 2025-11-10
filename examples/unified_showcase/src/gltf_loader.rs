/// GLTF/GLB loader for unified_showcase
/// Loads 3D models and converts them to wgpu mesh format
use anyhow::{Context, Result};
use std::path::Path;

/// Simple vertex format matching our shader expectations
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GltfVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Loaded mesh data ready for GPU upload
pub struct LoadedMesh {
    pub vertices: Vec<GltfVertex>,
    pub indices: Vec<u32>,
    pub name: String,
}

/// Load a GLTF/GLB file and extract first mesh
pub fn load_gltf(path: impl AsRef<Path>) -> Result<LoadedMesh> {
    let path = path.as_ref();
    
    log::info!("Attempting to load GLTF: {}", path.display());
    
    let (document, buffers, _images) = gltf::import(path)
        .with_context(|| format!("Failed to load GLTF file: {}", path.display()))?;

    log::info!("GLTF loaded successfully, parsing meshes...");

    // Get first mesh from first node
    let mesh = document
        .meshes()
        .next()
        .context("No meshes found in GLTF file")?;

    log::info!("Found mesh: '{}'", mesh.name().unwrap_or("unnamed"));

    let primitive = mesh
        .primitives()
        .next()
        .context("No primitives found in mesh")?;

    // Extract position data
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .context("No position data in mesh")?
        .collect();

    // Extract normal data (or generate default)
    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|iter| iter.collect())
        .unwrap_or_else(|| {
            // Default normals pointing up
            vec![[0.0, 1.0, 0.0]; positions.len()]
        });

    // Extract UV data (or generate default)
    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|iter| iter.into_f32().collect())
        .unwrap_or_else(|| {
            // Default UVs
            vec![[0.0, 0.0]; positions.len()]
        });

    // Build vertices
    let vertices: Vec<GltfVertex> = positions
        .iter()
        .zip(normals.iter())
        .zip(uvs.iter())
        .map(|((pos, normal), uv)| GltfVertex {
            position: *pos,
            normal: *normal,
            uv: *uv,
        })
        .collect();

    // Extract indices
    let indices: Vec<u32> = reader
        .read_indices()
        .context("No index data in mesh")?
        .into_u32()
        .collect();

    log::info!(
        "Loaded GLTF mesh '{}': {} vertices, {} triangles",
        mesh.name().unwrap_or("unnamed"),
        vertices.len(),
        indices.len() / 3
    );

    Ok(LoadedMesh {
        vertices,
        indices,
        name: mesh.name().unwrap_or("unnamed").to_string(),
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

            let vertices: Vec<GltfVertex> = positions
                .iter()
                .zip(normals.iter())
                .zip(uvs.iter())
                .map(|((pos, normal), uv)| GltfVertex {
                    position: *pos,
                    normal: *normal,
                    uv: *uv,
                })
                .collect();

            let indices: Vec<u32> = reader
                .read_indices()
                .context("No index data in mesh")?
                .into_u32()
                .collect();

            log::info!(
                "Loaded mesh '{}': {} vertices, {} triangles",
                mesh.name().unwrap_or("unnamed"),
                vertices.len(),
                indices.len() / 3
            );

            loaded_meshes.push(LoadedMesh {
                vertices,
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
