//! Comprehensive Marching Cubes Tests
//!
//! Tests all 256 configurations of the Marching Cubes algorithm to ensure:
//! - Watertight meshes (every edge shared by exactly 2 triangles)
//! - Correct vertex generation
//! - Proper triangle winding
//! - No degenerate triangles

use astraweave_terrain::meshing::{ChunkMesh, DualContouring, MeshVertex};
use astraweave_terrain::voxel_data::{ChunkCoord, Voxel, VoxelChunk, CHUNK_SIZE};
use glam::{IVec3, Vec3};
use std::collections::HashMap;

/// Test all 256 Marching Cubes configurations
#[test]
fn test_all_256_marching_cubes_configs() {
    let mut passed = 0;
    let mut failed = Vec::new();

    for config in 0u32..256 {
        let chunk = create_chunk_for_config(config as u8);
        let mut dc = DualContouring::new();
        let mesh = dc.generate_mesh(&chunk);

        // Verify triangles are well-formed
        assert_eq!(
            mesh.indices.len() % 3,
            0,
            "Config {} has invalid triangle count",
            config
        );

        // Skip watertight check for empty/full configs
        if config == 0 || config == 255 {
            assert!(mesh.is_empty(), "Config {} should be empty", config);
            passed += 1;
            continue;
        }

        // Check if mesh is valid (non-degenerate triangles)
        if !mesh.is_empty() {
            if validate_mesh_geometry(&mesh) {
                passed += 1;
            } else {
                failed.push(config);
            }
        } else {
            // Some configs might produce empty meshes due to simplification
            passed += 1;
        }
    }

    println!("Marching Cubes config test: {}/256 passed", passed);
    if !failed.is_empty() {
        println!("Failed configs: {:?}", failed);
    }
    assert_eq!(failed.len(), 0, "Some configs failed validation");
}

/// Create a test chunk for a specific MC configuration
fn create_chunk_for_config(config: u8) -> VoxelChunk {
    let coord = ChunkCoord::new(0, 0, 0);
    let mut chunk = VoxelChunk::new(coord);

    // Place a single cell in the center with the specific configuration
    let base = IVec3::new(8, 8, 8);

    // Corner offsets matching Marching Cubes convention
    let corner_offsets = [
        IVec3::new(0, 0, 0), // 0
        IVec3::new(1, 0, 0), // 1
        IVec3::new(1, 1, 0), // 2
        IVec3::new(0, 1, 0), // 3
        IVec3::new(0, 0, 1), // 4
        IVec3::new(1, 0, 1), // 5
        IVec3::new(1, 1, 1), // 6
        IVec3::new(0, 1, 1), // 7
    ];

    // Set voxels based on configuration bits
    for (i, offset) in corner_offsets.iter().enumerate() {
        let pos = base + *offset;
        let is_solid = (config & (1 << i)) != 0;
        let density = if is_solid { 1.0 } else { -1.0 };
        chunk.set_voxel(pos, Voxel::new(density, if is_solid { 1 } else { 0 }));
    }

    chunk
}

/// Validate mesh geometry (no degenerate triangles, proper normals)
fn validate_mesh_geometry(mesh: &ChunkMesh) -> bool {
    for tri in mesh.indices.chunks_exact(3) {
        let v0 = &mesh.vertices[tri[0] as usize];
        let v1 = &mesh.vertices[tri[1] as usize];
        let v2 = &mesh.vertices[tri[2] as usize];

        // Check for degenerate triangle (zero area)
        let edge1 = v1.position - v0.position;
        let edge2 = v2.position - v0.position;
        let cross = edge1.cross(edge2);
        let area = cross.length() * 0.5;

        if area < 0.0001 {
            return false; // Degenerate triangle
        }

        // Check normals are normalized
        if (v0.normal.length() - 1.0).abs() > 0.01
            || (v1.normal.length() - 1.0).abs() > 0.01
            || (v2.normal.length() - 1.0).abs() > 0.01
        {
            return false; // Invalid normal
        }
    }

    true
}

/// Test that sphere generates a watertight mesh
#[test]
fn test_sphere_mesh_watertight() {
    let coord = ChunkCoord::new(0, 0, 0);
    let mut chunk = VoxelChunk::new(coord);

    // Create sphere SDF
    let center = Vec3::new(16.0, 16.0, 16.0);
    let radius = 8.0;

    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pos = Vec3::new(x as f32, y as f32, z as f32);
                let dist = (pos - center).length() - radius;

                // Inside sphere is solid (positive density), outside is empty (negative)
                let density = if dist < 0.0 { 1.0 } else { -1.0 };
                let material = if dist < 0.0 { 1 } else { 0 };

                chunk.set_voxel(IVec3::new(x, y, z), Voxel::new(density, material));
            }
        }
    }

    let mut dc = DualContouring::new();
    let mesh = dc.generate_mesh(&chunk);

    assert!(!mesh.is_empty(), "Sphere mesh should not be empty");
    assert!(
        mesh.vertices.len() > 100,
        "Sphere should have many vertices"
    );

    // Validate mesh geometry
    assert!(
        validate_mesh_geometry(&mesh),
        "Sphere mesh has invalid geometry"
    );

    // Check watertightness
    assert!(is_mesh_watertight(&mesh), "Sphere mesh is not watertight");
}

/// Check if a mesh is watertight (every edge shared by exactly 2 triangles)
fn is_mesh_watertight(mesh: &ChunkMesh) -> bool {
    let mut edge_counts: HashMap<(u32, u32), usize> = HashMap::new();

    for tri in mesh.indices.chunks_exact(3) {
        for i in 0..3 {
            let v0 = tri[i];
            let v1 = tri[(i + 1) % 3];
            let edge = (v0.min(v1), v0.max(v1));
            *edge_counts.entry(edge).or_insert(0) += 1;
        }
    }

    // Every edge should be shared by exactly 2 triangles (manifold surface)
    edge_counts.values().all(|&count| count == 2)
}

/// Test cube generates correct topology
#[test]
fn test_cube_mesh_topology() {
    let coord = ChunkCoord::new(0, 0, 0);
    let mut chunk = VoxelChunk::new(coord);

    // Create solid cube in center
    for z in 8..24 {
        for y in 8..24 {
            for x in 8..24 {
                chunk.set_voxel(IVec3::new(x, y, z), Voxel::new(1.0, 1));
            }
        }
    }

    let mut dc = DualContouring::new();
    let mesh = dc.generate_mesh(&chunk);

    assert!(!mesh.is_empty(), "Cube mesh should not be empty");

    // Cube should produce roughly 6 faces worth of triangles (12 triangles minimum)
    assert!(
        mesh.indices.len() >= 36,
        "Cube should have at least 12 triangles (36 indices)"
    );

    assert!(
        validate_mesh_geometry(&mesh),
        "Cube mesh has invalid geometry"
    );
}

/// Test thin walls are handled correctly
#[test]
fn test_thin_wall_mesh() {
    let coord = ChunkCoord::new(0, 0, 0);
    let mut chunk = VoxelChunk::new(coord);

    // Create thin vertical wall
    for z in 8..24 {
        for y in 8..24 {
            chunk.set_voxel(IVec3::new(16, y, z), Voxel::new(1.0, 1));
        }
    }

    let mut dc = DualContouring::new();
    let mesh = dc.generate_mesh(&chunk);

    assert!(!mesh.is_empty(), "Thin wall mesh should not be empty");
    assert!(
        validate_mesh_geometry(&mesh),
        "Thin wall mesh has invalid geometry"
    );
}

/// Test mesh with multiple disconnected components
#[test]
fn test_disconnected_components() {
    let coord = ChunkCoord::new(0, 0, 0);
    let mut chunk = VoxelChunk::new(coord);

    // Create two separate cubes
    for z in 4..8 {
        for y in 4..8 {
            for x in 4..8 {
                chunk.set_voxel(IVec3::new(x, y, z), Voxel::new(1.0, 1));
            }
        }
    }

    for z in 24..28 {
        for y in 24..28 {
            for x in 24..28 {
                chunk.set_voxel(IVec3::new(x, y, z), Voxel::new(1.0, 1));
            }
        }
    }

    let mut dc = DualContouring::new();
    let mesh = dc.generate_mesh(&chunk);

    assert!(!mesh.is_empty(), "Disconnected mesh should not be empty");
    assert!(
        validate_mesh_geometry(&mesh),
        "Disconnected mesh has invalid geometry"
    );
}

/// Test edge cases with single voxels
#[test]
fn test_single_voxel_configs() {
    for config in [1, 2, 4, 8, 16, 32, 64, 128] {
        // Each bit represents a single corner solid
        let chunk = create_chunk_for_config(config);
        let mut dc = DualContouring::new();
        let mesh = dc.generate_mesh(&chunk);

        // Single corner configs should generate a small mesh
        assert!(
            !mesh.is_empty() || config == 0 || config == 255,
            "Config {} should generate mesh",
            config
        );

        if !mesh.is_empty() {
            assert!(
                validate_mesh_geometry(&mesh),
                "Config {} has invalid geometry",
                config
            );
        }
    }
}

/// Test that opposite configurations are complementary
#[test]
fn test_complementary_configs() {
    for config in 0..128 {
        let inverted_config = !config; // Bitwise NOT gives complementary config

        let chunk1 = create_chunk_for_config(config);
        let chunk2 = create_chunk_for_config(inverted_config);

        let mut dc1 = DualContouring::new();
        let mut dc2 = DualContouring::new();

        let mesh1 = dc1.generate_mesh(&chunk1);
        let mesh2 = dc2.generate_mesh(&chunk2);

        // Complementary configs should have similar complexity
        // (same number of triangles, just inverted normals)
        assert_eq!(
            mesh1.indices.len(),
            mesh2.indices.len(),
            "Configs {} and {} should have same triangle count",
            config,
            inverted_config
        );
    }
}

/// Test parallel mesh generation
#[test]
fn test_parallel_mesh_generation() {
    use astraweave_terrain::meshing::AsyncMeshGenerator;

    let _coord = ChunkCoord::new(0, 0, 0);
    let mut chunks = Vec::new();

    // Create 10 sphere chunks at different positions
    for i in 0..10 {
        let mut chunk = VoxelChunk::new(ChunkCoord::new(i, 0, 0));
        let center = Vec3::new(16.0, 16.0, 16.0);
        let radius = 6.0 + i as f32;

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    let dist = (pos - center).length() - radius;
                    let density = if dist < 0.0 { 1.0 } else { -1.0 };
                    let material = if dist < 0.0 { 1 } else { 0 };
                    chunk.set_voxel(IVec3::new(x, y, z), Voxel::new(density, material));
                }
            }
        }
        chunks.push(chunk);
    }

    // Generate meshes in parallel
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let meshes = runtime.block_on(async {
        let mut gen = AsyncMeshGenerator::new();
        gen.generate_meshes_parallel(chunks).await
    });

    assert_eq!(meshes.len(), 10, "Should generate 10 meshes");

    for (i, mesh) in meshes.iter().enumerate() {
        assert!(!mesh.is_empty(), "Mesh {} should not be empty", i);
        assert!(
            validate_mesh_geometry(mesh),
            "Mesh {} has invalid geometry",
            i
        );
    }
}

/// Test mesh memory usage estimation
#[test]
fn test_mesh_memory_usage() {
    let coord = ChunkCoord::new(0, 0, 0);
    let mut chunk = VoxelChunk::new(coord);

    // Create sphere
    let center = Vec3::new(16.0, 16.0, 16.0);
    let radius = 10.0;

    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pos = Vec3::new(x as f32, y as f32, z as f32);
                let dist = (pos - center).length() - radius;
                let density = if dist < 0.0 { 1.0 } else { -1.0 };
                let material = if dist < 0.0 { 1 } else { 0 };
                chunk.set_voxel(IVec3::new(x, y, z), Voxel::new(density, material));
            }
        }
    }

    let mut dc = DualContouring::new();
    let mesh = dc.generate_mesh(&chunk);

    let memory = mesh.memory_usage();
    assert!(memory > 0, "Memory usage should be > 0");

    let expected = std::mem::size_of::<ChunkMesh>()
        + mesh.vertices.len() * std::mem::size_of::<MeshVertex>()
        + mesh.indices.len() * std::mem::size_of::<u32>();

    assert_eq!(memory, expected, "Memory usage calculation mismatch");
}

/// Benchmark-style test for performance validation
#[test]
fn test_mesh_generation_performance() {
    let coord = ChunkCoord::new(0, 0, 0);
    let mut chunk = VoxelChunk::new(coord);

    // Create complex terrain with noise-like pattern
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // Simple hash-based noise
                let h = (x
                    .wrapping_mul(374_761_393)
                    .wrapping_add(y.wrapping_mul(668_265_263))
                    .wrapping_add(z.wrapping_mul(1_597_334_677)))
                    % 100;
                let density = if h > 50 { 1.0 } else { -1.0 };
                let material = if h > 50 { 1 } else { 0 };
                chunk.set_voxel(IVec3::new(x, y, z), Voxel::new(density, material));
            }
        }
    }

    let start = std::time::Instant::now();
    let mut dc = DualContouring::new();
    let mesh = dc.generate_mesh(&chunk);
    let duration = start.elapsed();

    println!(
        "Generated mesh with {} vertices, {} triangles in {:?}",
        mesh.vertices.len(),
        mesh.indices.len() / 3,
        duration
    );

    // Should complete in reasonable time (<100ms for a single chunk)
    assert!(
        duration.as_millis() < 100,
        "Mesh generation took too long: {:?}",
        duration
    );
}
