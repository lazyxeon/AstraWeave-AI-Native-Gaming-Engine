//! Comprehensive tests for chunk.rs
//!
//! Phase 5: Pure Logic Tests - Terrain Chunk Management
//! Target: 26.36% → 80%+ coverage for chunk.rs

#[cfg(test)]
mod tests {
    use crate::chunk::*;
    use crate::{BiomeType, Heightmap};
    use glam::Vec3;

    // ============================================================================
    // ChunkId Tests (Core Functionality)
    // ============================================================================

    #[test]
    fn test_chunk_id_creation() {
        let id = ChunkId::new(10, -5);
        assert_eq!(id.x, 10);
        assert_eq!(id.z, -5);
    }

    #[test]
    fn test_chunk_id_from_world_pos() {
        let chunk_size = 64.0;

        // Positive coordinates
        let pos = Vec3::new(100.0, 0.0, 200.0);
        let id = ChunkId::from_world_pos(pos, chunk_size);
        assert_eq!(id.x, 1); // floor(100/64) = 1
        assert_eq!(id.z, 3); // floor(200/64) = 3

        // Negative coordinates
        let neg_pos = Vec3::new(-64.0, 0.0, -1.0);
        let neg_id = ChunkId::from_world_pos(neg_pos, chunk_size);
        assert_eq!(neg_id.x, -1); // floor(-64/64) = -1
        assert_eq!(neg_id.z, -1); // floor(-1/64) = -1

        // Zero
        let zero = Vec3::ZERO;
        let zero_id = ChunkId::from_world_pos(zero, chunk_size);
        assert_eq!(zero_id.x, 0);
        assert_eq!(zero_id.z, 0);
    }

    #[test]
    fn test_chunk_id_to_world_pos() {
        let chunk_size = 64.0;
        let id = ChunkId::new(5, -3);
        let pos = id.to_world_pos(chunk_size);

        assert_eq!(pos.x, 5.0 * chunk_size);
        assert_eq!(pos.y, 0.0); // Y is always 0 for terrain chunks
        assert_eq!(pos.z, -3.0 * chunk_size);
    }

    #[test]
    fn test_chunk_id_to_center_pos() {
        let chunk_size = 64.0;
        let id = ChunkId::new(2, 4);
        let center = id.to_center_pos(chunk_size);

        // Center should be origin + half chunk size
        assert_eq!(center.x, 2.0 * chunk_size + chunk_size * 0.5);
        assert_eq!(center.y, 0.0);
        assert_eq!(center.z, 4.0 * chunk_size + chunk_size * 0.5);

        // Test zero chunk center
        let zero_id = ChunkId::new(0, 0);
        let zero_center = zero_id.to_center_pos(chunk_size);
        assert_eq!(zero_center.x, chunk_size * 0.5);
        assert_eq!(zero_center.z, chunk_size * 0.5);
    }

    #[test]
    fn test_chunk_id_round_trip() {
        let chunk_size = 64.0;
        let original_id = ChunkId::new(10, -5);

        let world_pos = original_id.to_world_pos(chunk_size);
        let reconstructed = ChunkId::from_world_pos(world_pos, chunk_size);

        assert_eq!(original_id, reconstructed);
    }

    #[test]
    fn test_chunk_id_get_chunks_in_radius() {
        let chunk_size = 64.0;
        let center_pos = Vec3::new(128.0, 0.0, 192.0);

        // Radius 0 should return only center chunk
        let chunks_r0 = ChunkId::get_chunks_in_radius(center_pos, 0, chunk_size);
        assert_eq!(chunks_r0.len(), 1);
        let center_id = ChunkId::from_world_pos(center_pos, chunk_size);
        assert_eq!(chunks_r0[0], center_id);

        // Radius 1 should return 3x3 = 9 chunks
        let chunks_r1 = ChunkId::get_chunks_in_radius(center_pos, 1, chunk_size);
        assert_eq!(chunks_r1.len(), 9); // (2*1+1)^2 = 9

        // Radius 2 should return 5x5 = 25 chunks
        let chunks_r2 = ChunkId::get_chunks_in_radius(center_pos, 2, chunk_size);
        assert_eq!(chunks_r2.len(), 25); // (2*2+1)^2 = 25

        // Verify center chunk is included
        assert!(chunks_r1.contains(&center_id));
        assert!(chunks_r2.contains(&center_id));
    }

    #[test]
    fn test_chunk_id_get_chunks_in_radius_large() {
        let chunk_size = 64.0;
        let center_pos = Vec3::ZERO;

        // Test larger radius
        let radius = 5;
        let chunks = ChunkId::get_chunks_in_radius(center_pos, radius, chunk_size);

        // Should be (2*r+1)^2
        let expected_count = ((2 * radius + 1) * (2 * radius + 1)) as usize;
        assert_eq!(chunks.len(), expected_count);

        // Verify bounds
        let center_id = ChunkId::new(0, 0);
        for chunk_id in chunks.iter() {
            let dx = (chunk_id.x - center_id.x).abs();
            let dz = (chunk_id.z - center_id.z).abs();
            assert!(dx <= radius as i32);
            assert!(dz <= radius as i32);
        }
    }

    #[test]
    fn test_chunk_id_distance_to() {
        // Same chunk
        let id1 = ChunkId::new(0, 0);
        assert_eq!(id1.distance_to(id1), 0.0);

        // Horizontal distance
        let id2 = ChunkId::new(3, 0);
        assert_eq!(id1.distance_to(id2), 3.0);

        // Vertical distance
        let id3 = ChunkId::new(0, 4);
        assert_eq!(id1.distance_to(id3), 4.0);

        // Diagonal distance (Pythagorean)
        let id4 = ChunkId::new(3, 4);
        assert_eq!(id1.distance_to(id4), 5.0); // 3-4-5 triangle

        // Negative coordinates
        let id5 = ChunkId::new(-3, -4);
        assert_eq!(id1.distance_to(id5), 5.0);
    }

    #[test]
    fn test_chunk_id_distance_symmetry() {
        let id1 = ChunkId::new(5, 10);
        let id2 = ChunkId::new(-3, 7);

        let dist1 = id1.distance_to(id2);
        let dist2 = id2.distance_to(id1);

        assert!(
            (dist1 - dist2).abs() < 0.0001,
            "Distance should be symmetric"
        );
    }

    #[test]
    fn test_chunk_id_equality() {
        let id1 = ChunkId::new(5, 10);
        let id2 = ChunkId::new(5, 10);
        let id3 = ChunkId::new(5, 11);
        let id4 = ChunkId::new(6, 10);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4);
    }

    #[test]
    fn test_chunk_id_hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let id1 = ChunkId::new(5, 10);
        let id2 = ChunkId::new(5, 10);

        set.insert(id1);
        assert!(set.contains(&id2), "Equal IDs should hash to same value");
        assert_eq!(set.len(), 1);

        let id3 = ChunkId::new(5, 11);
        set.insert(id3);
        assert_eq!(set.len(), 2);
    }

    // ============================================================================
    // TerrainChunk Tests (Data Structure)
    // ============================================================================

    fn create_test_heightmap() -> Heightmap {
        // Create a simple heightmap for testing
        use crate::HeightmapConfig;
        Heightmap::new(HeightmapConfig {
            resolution: 32,
            ..Default::default()
        })
        .expect("Failed to create test heightmap")
    }

    fn create_test_biome_map(size: usize) -> Vec<BiomeType> {
        vec![BiomeType::Grassland; size]
    }

    #[test]
    fn test_terrain_chunk_creation() {
        let id = ChunkId::new(5, 10);
        let heightmap = create_test_heightmap();
        let biome_map = create_test_biome_map(32 * 32);

        let chunk = TerrainChunk::new(id, heightmap, biome_map);

        assert_eq!(chunk.id(), id);
        assert!(chunk.is_mesh_dirty()); // Should start dirty
    }

    #[test]
    fn test_terrain_chunk_heightmap_access() {
        let id = ChunkId::new(0, 0);
        let heightmap = create_test_heightmap();
        let biome_map = create_test_biome_map(32 * 32);

        let chunk = TerrainChunk::new(id, heightmap, biome_map);

        let retrieved_heightmap = chunk.heightmap();
        assert_eq!(retrieved_heightmap.resolution(), 32);
    }

    #[test]
    fn test_terrain_chunk_biome_map_access() {
        let id = ChunkId::new(0, 0);
        let heightmap = create_test_heightmap();
        let biome_count = 32 * 32;
        let biome_map = create_test_biome_map(biome_count);

        let chunk = TerrainChunk::new(id, heightmap, biome_map);

        let retrieved_biome_map = chunk.biome_map();
        assert_eq!(retrieved_biome_map.len(), biome_count);
        assert!(retrieved_biome_map
            .iter()
            .all(|b| *b == BiomeType::Grassland));
    }

    #[test]
    fn test_terrain_chunk_mesh_dirty_flag() {
        let id = ChunkId::new(0, 0);
        let heightmap = create_test_heightmap();
        let biome_map = create_test_biome_map(32 * 32);

        let chunk = TerrainChunk::new(id, heightmap, biome_map);

        assert!(chunk.is_mesh_dirty(), "New chunk should be dirty");

        // Note: mark_clean() is not exposed in TerrainChunk API from the file read
        // If it's private, we can't test it directly
    }

    // ============================================================================
    // Integration Tests (Multi-Component)
    // ============================================================================

    #[test]
    fn test_chunk_streaming_scenario() {
        let chunk_size = 64.0;
        let player_pos = Vec3::new(0.0, 0.0, 0.0);

        // Get chunks in streaming radius
        let chunks = ChunkId::get_chunks_in_radius(player_pos, 3, chunk_size);

        // Verify chunks cover appropriate area
        assert_eq!(chunks.len(), 49); // (2*3+1)^2 = 49

        // Create terrain chunks for nearest chunks
        let mut terrain_chunks = Vec::new();
        for chunk_id in chunks.iter().take(9) {
            let heightmap = create_test_heightmap();
            let biome_map = create_test_biome_map(32 * 32);
            let chunk = TerrainChunk::new(*chunk_id, heightmap, biome_map);
            terrain_chunks.push(chunk);
        }

        assert_eq!(terrain_chunks.len(), 9);

        // Verify all chunks have correct IDs
        for (i, chunk) in terrain_chunks.iter().enumerate() {
            assert_eq!(chunk.id(), chunks[i]);
        }
    }

    #[test]
    fn test_chunk_grid_layout() {
        let chunk_size = 64.0;

        // Create a grid of chunks
        let center = ChunkId::new(0, 0);
        let neighbors =
            ChunkId::get_chunks_in_radius(center.to_center_pos(chunk_size), 1, chunk_size);

        // Should have 3x3 grid
        assert_eq!(neighbors.len(), 9);

        // Verify grid contains all expected chunks
        let expected_ids = [ChunkId::new(-1, -1),
            ChunkId::new(0, -1),
            ChunkId::new(1, -1),
            ChunkId::new(-1, 0),
            ChunkId::new(0, 0),
            ChunkId::new(1, 0),
            ChunkId::new(-1, 1),
            ChunkId::new(0, 1),
            ChunkId::new(1, 1)];

        for expected in expected_ids.iter() {
            assert!(
                neighbors.contains(expected),
                "Grid should contain chunk {:?}",
                expected
            );
        }
    }

    #[test]
    fn test_chunk_coordinate_systems() {
        let chunk_size = 64.0;

        // Test various coordinate system conversions
        let test_cases = vec![
            (Vec3::new(0.0, 0.0, 0.0), ChunkId::new(0, 0)),
            (Vec3::new(64.0, 0.0, 64.0), ChunkId::new(1, 1)),
            (Vec3::new(-64.0, 0.0, 0.0), ChunkId::new(-1, 0)),
            (Vec3::new(100.0, 0.0, -50.0), ChunkId::new(1, -1)),
            (Vec3::new(128.0, 0.0, 256.0), ChunkId::new(2, 4)),
        ];

        for (world_pos, expected_id) in test_cases {
            let computed_id = ChunkId::from_world_pos(world_pos, chunk_size);
            assert_eq!(
                computed_id, expected_id,
                "World pos {:?} should map to chunk {:?}",
                world_pos, expected_id
            );

            // Verify round-trip preserves chunk origin
            let origin = computed_id.to_world_pos(chunk_size);
            let recomputed = ChunkId::from_world_pos(origin, chunk_size);
            assert_eq!(computed_id, recomputed);
        }
    }

    #[test]
    fn test_chunk_id_boundary_conditions() {
        let chunk_size = 64.0;

        // Test positions exactly on chunk boundaries
        let boundary_cases = [
            Vec3::new(63.999, 0.0, 0.0), // Just before boundary
            Vec3::new(64.0, 0.0, 0.0),   // Exactly on boundary
            Vec3::new(64.001, 0.0, 0.0), // Just after boundary
        ];

        let id0 = ChunkId::from_world_pos(boundary_cases[0], chunk_size);
        let id1 = ChunkId::from_world_pos(boundary_cases[1], chunk_size);
        let id2 = ChunkId::from_world_pos(boundary_cases[2], chunk_size);

        assert_eq!(id0, ChunkId::new(0, 0));
        assert_eq!(id1, ChunkId::new(1, 0)); // Boundary belongs to next chunk
        assert_eq!(id2, ChunkId::new(1, 0));
    }

    #[test]
    fn test_chunk_distance_calculations() {
        // Test distance calculations for streaming/LOD
        let chunks = vec![
            (ChunkId::new(0, 0), ChunkId::new(0, 0), 0.0),
            (ChunkId::new(0, 0), ChunkId::new(1, 0), 1.0),
            (ChunkId::new(0, 0), ChunkId::new(0, 1), 1.0),
            (ChunkId::new(0, 0), ChunkId::new(1, 1), 1.414), // sqrt(2)
            (ChunkId::new(0, 0), ChunkId::new(3, 4), 5.0),
            (ChunkId::new(5, 5), ChunkId::new(5, 5), 0.0),
        ];

        for (id1, id2, expected_dist) in chunks {
            let dist = id1.distance_to(id2);
            assert!(
                (dist - expected_dist).abs() < 0.01,
                "Distance from {:?} to {:?} should be {}, got {}",
                id1,
                id2,
                expected_dist,
                dist
            );
        }
    }

    #[test]
    fn test_multiple_chunk_sizes() {
        // Test system works with different chunk sizes
        let chunk_sizes = [16.0, 32.0, 64.0, 128.0, 256.0];

        for chunk_size in chunk_sizes.iter() {
            let pos = Vec3::new(100.0, 0.0, 200.0);
            let id = ChunkId::from_world_pos(pos, *chunk_size);
            let origin = id.to_world_pos(*chunk_size);

            // Verify position is within chunk bounds
            assert!(origin.x <= pos.x && pos.x < origin.x + chunk_size);
            assert!(origin.z <= pos.z && pos.z < origin.z + chunk_size);
        }
    }

    // ============================================================================
    // Stress Tests
    // ============================================================================

    #[test]
    fn test_large_chunk_radius() {
        let chunk_size = 64.0;
        let center_pos = Vec3::ZERO;

        // Test large streaming radius (common in open-world games)
        let radius = 10;
        let chunks = ChunkId::get_chunks_in_radius(center_pos, radius, chunk_size);

        let expected_count = ((2 * radius + 1) * (2 * radius + 1)) as usize;
        assert_eq!(chunks.len(), expected_count);

        // Verify all chunks are within radius
        let center_id = ChunkId::new(0, 0);
        for chunk_id in chunks.iter() {
            let dx = (chunk_id.x - center_id.x).abs();
            let dz = (chunk_id.z - center_id.z).abs();
            assert!(dx <= radius as i32 && dz <= radius as i32);
        }
    }

    #[test]
    fn test_chunk_id_extreme_coordinates() {
        let chunk_size = 64.0;

        // Test with very large world coordinates
        let extreme_positions = vec![
            Vec3::new(1000000.0, 0.0, 0.0),
            Vec3::new(-1000000.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1000000.0),
            Vec3::new(0.0, 0.0, -1000000.0),
        ];

        for pos in extreme_positions {
            let id = ChunkId::from_world_pos(pos, chunk_size);
            let reconstructed_origin = id.to_world_pos(chunk_size);
            let reconstructed_id = ChunkId::from_world_pos(reconstructed_origin, chunk_size);

            assert_eq!(
                id, reconstructed_id,
                "Extreme position {:?} should round-trip correctly",
                pos
            );
        }
    }

    #[test]
    fn test_terrain_chunk_collection() {
        // Test creating many terrain chunks (memory/performance)
        let mut chunks = Vec::new();

        for x in -5..=5 {
            for z in -5..=5 {
                let id = ChunkId::new(x, z);
                let heightmap = create_test_heightmap();
                let biome_map = create_test_biome_map(32 * 32);
                let chunk = TerrainChunk::new(id, heightmap, biome_map);
                chunks.push(chunk);
            }
        }

        // Should have 11x11 = 121 chunks
        assert_eq!(chunks.len(), 121);

        // Verify all chunks have unique IDs
        let mut id_set = std::collections::HashSet::new();
        for chunk in chunks.iter() {
            assert!(
                id_set.insert(chunk.id()),
                "Chunk ID {:?} should be unique",
                chunk.id()
            );
        }
    }

    #[test]
    fn test_biome_map_variations() {
        let id = ChunkId::new(0, 0);
        let heightmap = create_test_heightmap();

        // Test different biome configurations
        let biome_types = vec![
            BiomeType::Grassland,
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Mountain,
            BiomeType::Tundra,
            BiomeType::Swamp,
        ];

        for biome in biome_types {
            let biome_map = vec![biome; 32 * 32];
            let chunk = TerrainChunk::new(id, heightmap.clone(), biome_map);

            let retrieved = chunk.biome_map();
            assert!(
                retrieved.iter().all(|b| *b == biome),
                "All biomes should be {:?}",
                biome
            );
        }
    }
}
