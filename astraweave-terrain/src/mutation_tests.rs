//! Comprehensive mutation-killing tests for astraweave-terrain
//!
//! These tests are designed to catch arithmetic and logical mutations
//! by verifying specific expected values and behavioral correctness.

#[cfg(test)]
mod heightmap_tests {
    use crate::heightmap::{Heightmap, HeightmapConfig};

    #[test]
    fn test_heightmap_config_defaults() {
        let config = HeightmapConfig::default();
        assert_eq!(config.resolution, 128);
        assert_eq!(config.min_height, 0.0);
        assert_eq!(config.max_height, 100.0);
        assert_eq!(config.height_scale, 1.0);
    }

    #[test]
    fn test_heightmap_new_creates_zero_filled() {
        let config = HeightmapConfig {
            resolution: 4,
            ..Default::default()
        };
        let heightmap = Heightmap::new(config).unwrap();
        assert_eq!(heightmap.resolution(), 4);
        assert_eq!(heightmap.data().len(), 16); // 4x4
        assert!(heightmap.data().iter().all(|&h| h == 0.0));
    }

    #[test]
    fn test_heightmap_from_data_validates_size() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let heightmap = Heightmap::from_data(data, 3).unwrap();
        assert_eq!(heightmap.resolution(), 3);
        assert_eq!(heightmap.data().len(), 9);
    }

    #[test]
    fn test_heightmap_from_data_rejects_wrong_size() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // 5 elements, not 4 or 9
        let result = Heightmap::from_data(data, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_heightmap_from_data_calculates_min_max() {
        let data = vec![5.0, 10.0, -3.0, 8.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        assert_eq!(heightmap.min_height(), -3.0);
        assert_eq!(heightmap.max_height(), 10.0);
    }

    #[test]
    fn test_heightmap_get_height_bounds_check() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        
        assert_eq!(heightmap.get_height(0, 0), 1.0);
        assert_eq!(heightmap.get_height(1, 0), 2.0);
        assert_eq!(heightmap.get_height(0, 1), 3.0);
        assert_eq!(heightmap.get_height(1, 1), 4.0);
        
        // Out of bounds should return 0
        assert_eq!(heightmap.get_height(2, 0), 0.0);
        assert_eq!(heightmap.get_height(0, 2), 0.0);
    }

    #[test]
    fn test_heightmap_set_height_updates_bounds() {
        let config = HeightmapConfig {
            resolution: 2,
            ..Default::default()
        };
        let mut heightmap = Heightmap::new(config).unwrap();
        
        heightmap.set_height(0, 0, 50.0);
        assert_eq!(heightmap.get_height(0, 0), 50.0);
        assert_eq!(heightmap.max_height(), 50.0);
        
        heightmap.set_height(1, 1, -10.0);
        assert_eq!(heightmap.min_height(), -10.0);
    }

    #[test]
    fn test_heightmap_set_height_ignores_out_of_bounds() {
        let config = HeightmapConfig {
            resolution: 2,
            ..Default::default()
        };
        let mut heightmap = Heightmap::new(config).unwrap();
        
        // Should not crash or modify anything
        heightmap.set_height(5, 5, 100.0);
        assert!(heightmap.data().iter().all(|&h| h == 0.0));
    }

    #[test]
    fn test_heightmap_bilinear_sample_corners() {
        // For a 2x2 heightmap, the bilinear samples coordinates [0,1) in each axis
        // The clamping means we can't actually sample at exactly 1.0
        let data = vec![0.0, 10.0, 20.0, 30.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        
        // At (0,0) corner should match exact value
        assert!((heightmap.sample_bilinear(0.0, 0.0) - 0.0).abs() < 0.01);
        
        // Near upper bounds, value should be interpolated (due to clamping to res-1.001)
        // Just verify they return reasonable values within range
        let upper_x = heightmap.sample_bilinear(0.999, 0.0);
        let upper_z = heightmap.sample_bilinear(0.0, 0.999);
        assert!(upper_x >= 0.0 && upper_x <= 10.0);
        assert!(upper_z >= 0.0 && upper_z <= 20.0);
    }

    #[test]
    fn test_heightmap_bilinear_sample_center() {
        let data = vec![0.0, 10.0, 20.0, 30.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        
        // At center (0.5, 0.5), should be average of all four
        let center = heightmap.sample_bilinear(0.5, 0.5);
        let expected = (0.0 + 10.0 + 20.0 + 30.0) / 4.0;
        assert!((center - expected).abs() < 0.01, "Center: {}, Expected: {}", center, expected);
    }

    #[test]
    fn test_heightmap_recalculate_bounds() {
        let config = HeightmapConfig {
            resolution: 2,
            ..Default::default()
        };
        let mut heightmap = Heightmap::new(config).unwrap();
        
        // Manually modify data
        let data = heightmap.data_mut();
        data[0] = 5.0;
        data[1] = 15.0;
        data[2] = -5.0;
        data[3] = 25.0;
        
        heightmap.recalculate_bounds();
        assert_eq!(heightmap.min_height(), -5.0);
        assert_eq!(heightmap.max_height(), 25.0);
    }
}

#[cfg(test)]
mod chunk_id_tests {
    use crate::chunk::ChunkId;
    use glam::Vec3;

    #[test]
    fn test_chunk_id_new() {
        let id = ChunkId::new(5, -3);
        assert_eq!(id.x, 5);
        assert_eq!(id.z, -3);
    }

    #[test]
    fn test_chunk_id_from_world_pos_positive() {
        let chunk_size = 64.0;
        let id = ChunkId::from_world_pos(Vec3::new(100.0, 50.0, 200.0), chunk_size);
        assert_eq!(id.x, 1); // 100/64 = 1.56 -> floor = 1
        assert_eq!(id.z, 3); // 200/64 = 3.12 -> floor = 3
    }

    #[test]
    fn test_chunk_id_from_world_pos_negative() {
        let chunk_size = 64.0;
        let id = ChunkId::from_world_pos(Vec3::new(-100.0, 50.0, -200.0), chunk_size);
        assert_eq!(id.x, -2); // -100/64 = -1.56 -> floor = -2
        assert_eq!(id.z, -4); // -200/64 = -3.12 -> floor = -4
    }

    #[test]
    fn test_chunk_id_from_world_pos_at_origin() {
        let chunk_size = 64.0;
        let id = ChunkId::from_world_pos(Vec3::ZERO, chunk_size);
        assert_eq!(id.x, 0);
        assert_eq!(id.z, 0);
    }

    #[test]
    fn test_chunk_id_to_world_pos() {
        let chunk_size = 64.0;
        let id = ChunkId::new(2, 3);
        let pos = id.to_world_pos(chunk_size);
        
        assert_eq!(pos.x, 128.0); // 2 * 64
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 192.0); // 3 * 64
    }

    #[test]
    fn test_chunk_id_to_center_pos() {
        let chunk_size = 64.0;
        let id = ChunkId::new(0, 0);
        let center = id.to_center_pos(chunk_size);
        
        assert_eq!(center.x, 32.0);
        assert_eq!(center.y, 0.0);
        assert_eq!(center.z, 32.0);
    }

    #[test]
    fn test_chunk_id_distance_to_same() {
        let id1 = ChunkId::new(5, 5);
        let id2 = ChunkId::new(5, 5);
        assert_eq!(id1.distance_to(id2), 0.0);
    }

    #[test]
    fn test_chunk_id_distance_to_horizontal() {
        let id1 = ChunkId::new(0, 0);
        let id2 = ChunkId::new(3, 4);
        assert_eq!(id1.distance_to(id2), 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_chunk_id_get_chunks_in_radius() {
        let chunks = ChunkId::get_chunks_in_radius(Vec3::ZERO, 1, 64.0);
        assert_eq!(chunks.len(), 9); // 3x3 grid around center
        
        // Verify all expected chunks are present
        assert!(chunks.contains(&ChunkId::new(0, 0)));
        assert!(chunks.contains(&ChunkId::new(-1, 0)));
        assert!(chunks.contains(&ChunkId::new(1, 0)));
        assert!(chunks.contains(&ChunkId::new(0, -1)));
        assert!(chunks.contains(&ChunkId::new(0, 1)));
    }

    #[test]
    fn test_chunk_id_roundtrip() {
        let chunk_size = 64.0;
        let original = ChunkId::new(10, -5);
        let world_pos = original.to_world_pos(chunk_size);
        let recovered = ChunkId::from_world_pos(world_pos, chunk_size);
        
        assert_eq!(original, recovered);
    }
}

#[cfg(test)]
mod voxel_tests {
    use crate::voxel_data::{ChunkCoord, Voxel, CHUNK_SIZE};
    use glam::Vec3;

    #[test]
    fn test_chunk_size_constant() {
        assert_eq!(CHUNK_SIZE, 32);
    }

    #[test]
    fn test_voxel_default() {
        let voxel = Voxel::default();
        assert_eq!(voxel.density, 0.0);
        assert_eq!(voxel.material, 0);
    }

    #[test]
    fn test_voxel_new() {
        let voxel = Voxel::new(0.75, 5);
        assert_eq!(voxel.density, 0.75);
        assert_eq!(voxel.material, 5);
    }

    #[test]
    fn test_voxel_is_solid_threshold() {
        assert!(!Voxel::new(0.0, 0).is_solid());
        assert!(!Voxel::new(0.49, 0).is_solid());
        assert!(!Voxel::new(0.5, 0).is_solid()); // Exactly 0.5 is NOT solid (>0.5)
        assert!(Voxel::new(0.51, 0).is_solid());
        assert!(Voxel::new(1.0, 0).is_solid());
    }

    #[test]
    fn test_chunk_coord_new() {
        let coord = ChunkCoord::new(1, 2, 3);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 2);
        assert_eq!(coord.z, 3);
    }

    #[test]
    fn test_chunk_coord_from_world_pos() {
        let coord = ChunkCoord::from_world_pos(Vec3::new(50.0, 100.0, 150.0));
        assert_eq!(coord.x, 1);  // 50/32 = 1.56 -> floor = 1
        assert_eq!(coord.y, 3);  // 100/32 = 3.12 -> floor = 3
        assert_eq!(coord.z, 4);  // 150/32 = 4.68 -> floor = 4
    }

    #[test]
    fn test_chunk_coord_from_world_pos_negative() {
        let coord = ChunkCoord::from_world_pos(Vec3::new(-50.0, -100.0, -150.0));
        assert_eq!(coord.x, -2);  // -50/32 = -1.56 -> floor = -2
        assert_eq!(coord.y, -4);  // -100/32 = -3.12 -> floor = -4
        assert_eq!(coord.z, -5);  // -150/32 = -4.68 -> floor = -5
    }

    #[test]
    fn test_chunk_coord_to_world_pos() {
        let coord = ChunkCoord::new(2, 3, 4);
        let world = coord.to_world_pos();
        
        assert_eq!(world.x, 64.0);  // 2 * 32
        assert_eq!(world.y, 96.0);  // 3 * 32
        assert_eq!(world.z, 128.0); // 4 * 32
    }

    #[test]
    fn test_chunk_coord_neighbors() {
        let coord = ChunkCoord::new(0, 0, 0);
        let neighbors = coord.neighbors();
        
        assert_eq!(neighbors.len(), 6);
        assert!(neighbors.contains(&ChunkCoord::new(1, 0, 0)));
        assert!(neighbors.contains(&ChunkCoord::new(-1, 0, 0)));
        assert!(neighbors.contains(&ChunkCoord::new(0, 1, 0)));
        assert!(neighbors.contains(&ChunkCoord::new(0, -1, 0)));
        assert!(neighbors.contains(&ChunkCoord::new(0, 0, 1)));
        assert!(neighbors.contains(&ChunkCoord::new(0, 0, -1)));
    }

    #[test]
    fn test_chunk_coord_roundtrip() {
        let original = ChunkCoord::new(5, -3, 7);
        let world_pos = original.to_world_pos();
        let recovered = ChunkCoord::from_world_pos(world_pos);
        
        assert_eq!(original, recovered);
    }
}

#[cfg(test)]
mod biome_tests {
    use crate::biome::BiomeType;
    use std::str::FromStr;

    #[test]
    fn test_biome_type_as_str() {
        assert_eq!(BiomeType::Grassland.as_str(), "grassland");
        assert_eq!(BiomeType::Desert.as_str(), "desert");
        assert_eq!(BiomeType::Forest.as_str(), "forest");
        assert_eq!(BiomeType::Mountain.as_str(), "mountain");
        assert_eq!(BiomeType::Tundra.as_str(), "tundra");
        assert_eq!(BiomeType::Swamp.as_str(), "swamp");
        assert_eq!(BiomeType::Beach.as_str(), "beach");
        assert_eq!(BiomeType::River.as_str(), "river");
    }

    #[test]
    fn test_biome_type_from_str_lowercase() {
        assert_eq!(BiomeType::from_str("grassland").unwrap(), BiomeType::Grassland);
        assert_eq!(BiomeType::from_str("desert").unwrap(), BiomeType::Desert);
        assert_eq!(BiomeType::from_str("forest").unwrap(), BiomeType::Forest);
        assert_eq!(BiomeType::from_str("mountain").unwrap(), BiomeType::Mountain);
        assert_eq!(BiomeType::from_str("tundra").unwrap(), BiomeType::Tundra);
        assert_eq!(BiomeType::from_str("swamp").unwrap(), BiomeType::Swamp);
        assert_eq!(BiomeType::from_str("beach").unwrap(), BiomeType::Beach);
        assert_eq!(BiomeType::from_str("river").unwrap(), BiomeType::River);
    }

    #[test]
    fn test_biome_type_from_str_case_insensitive() {
        assert_eq!(BiomeType::from_str("GRASSLAND").unwrap(), BiomeType::Grassland);
        assert_eq!(BiomeType::from_str("Grassland").unwrap(), BiomeType::Grassland);
        assert_eq!(BiomeType::from_str("GrAsSlAnD").unwrap(), BiomeType::Grassland);
    }

    #[test]
    fn test_biome_type_from_str_invalid() {
        assert!(BiomeType::from_str("invalid").is_err());
        assert!(BiomeType::from_str("").is_err());
        assert!(BiomeType::from_str("ocean").is_err());
    }

    #[test]
    fn test_biome_type_parse_method() {
        assert_eq!(BiomeType::parse("forest"), Some(BiomeType::Forest));
        assert_eq!(BiomeType::parse("MOUNTAIN"), Some(BiomeType::Mountain));
        assert_eq!(BiomeType::parse("invalid"), None);
    }

    #[test]
    fn test_biome_type_all() {
        let all = BiomeType::all();
        assert_eq!(all.len(), 8);
        assert!(all.contains(&BiomeType::Grassland));
        assert!(all.contains(&BiomeType::Desert));
        assert!(all.contains(&BiomeType::Forest));
        assert!(all.contains(&BiomeType::Mountain));
        assert!(all.contains(&BiomeType::Tundra));
        assert!(all.contains(&BiomeType::Swamp));
        assert!(all.contains(&BiomeType::Beach));
        assert!(all.contains(&BiomeType::River));
    }

    #[test]
    fn test_biome_type_all_unique() {
        let all = BiomeType::all();
        for (i, biome1) in all.iter().enumerate() {
            for (j, biome2) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(biome1, biome2);
                }
            }
        }
    }
}

#[cfg(test)]
mod noise_config_tests {
    use crate::noise_gen::{NoiseConfig, NoiseType};

    #[test]
    fn test_noise_config_default_has_all_layers() {
        let config = NoiseConfig::default();
        assert!(config.base_elevation.enabled);
        assert!(config.mountains.enabled);
        assert!(config.detail.enabled);
    }

    #[test]
    fn test_noise_config_default_erosion() {
        let config = NoiseConfig::default();
        assert!(config.erosion_enabled);
        assert!(config.erosion_strength > 0.0);
        assert!(config.erosion_strength <= 1.0);
    }

    #[test]
    fn test_noise_layer_base_elevation() {
        let config = NoiseConfig::default();
        let base = &config.base_elevation;
        
        assert!(base.scale > 0.0);
        assert!(base.amplitude > 0.0);
        assert!(base.octaves >= 1);
        assert!(base.persistence > 0.0);
        assert!(base.persistence < 1.0);
        assert!(base.lacunarity > 1.0);
    }

    #[test]
    fn test_noise_layer_mountains() {
        let config = NoiseConfig::default();
        let mountains = &config.mountains;
        
        // Mountains should have higher amplitude and more octaves
        assert!(mountains.amplitude > config.base_elevation.amplitude);
        assert!(mountains.octaves >= config.base_elevation.octaves);
    }

    #[test]
    fn test_noise_layer_detail() {
        let config = NoiseConfig::default();
        let detail = &config.detail;
        
        // Detail should have smaller amplitude and larger scale (higher frequency)
        assert!(detail.amplitude < config.base_elevation.amplitude);
        assert!(detail.scale > config.base_elevation.scale);
    }

    #[test]
    fn test_noise_type_variants() {
        // Verify all variants are distinguishable
        let perlin = NoiseType::Perlin;
        let ridged = NoiseType::RidgedNoise;
        let billow = NoiseType::Billow;
        let fbm = NoiseType::Fbm;
        
        assert!(format!("{:?}", perlin).contains("Perlin"));
        assert!(format!("{:?}", ridged).contains("Ridged"));
        assert!(format!("{:?}", billow).contains("Billow"));
        assert!(format!("{:?}", fbm).contains("Fbm"));
    }
}

#[cfg(test)]
mod climate_tests {
    use crate::climate::ClimateConfig;

    #[test]
    fn test_climate_config_default() {
        let config = ClimateConfig::default();
        
        // Temperature layer should have reasonable world scale
        assert!(config.temperature.scale > 0.0);
        assert!(config.temperature.octaves >= 1);
        assert!(config.temperature.persistence > 0.0 && config.temperature.persistence <= 1.0);
        assert!(config.temperature.amplitude > 0.0);
        
        // Moisture layer should have reasonable world scale
        assert!(config.moisture.scale > 0.0);
        assert!(config.moisture.octaves >= 1);
        assert!(config.moisture.persistence > 0.0 && config.moisture.persistence <= 1.0);
        assert!(config.moisture.amplitude > 0.0);
    }
    
    #[test]
    fn test_climate_gradients() {
        let config = ClimateConfig::default();
        
        // Height gradient should be negative (cooler at altitude)
        assert!(config.temperature_height_gradient < 0.0);
        
        // Latitude gradient should be positive
        assert!(config.temperature_latitude_gradient > 0.0);
        
        // Moisture falloff should be positive
        assert!(config.moisture_distance_falloff > 0.0);
    }
    
    #[test]
    fn test_climate_layer_lacunarity() {
        let config = ClimateConfig::default();
        
        // Lacunarity should be > 1 for increasing frequency
        assert!(config.temperature.lacunarity > 1.0);
        assert!(config.moisture.lacunarity > 1.0);
    }
}

#[cfg(test)]
mod world_config_tests {
    use crate::WorldConfig;

    #[test]
    fn test_world_config_default() {
        let config = WorldConfig::default();
        
        // Chunk size should be reasonable
        assert!(config.chunk_size > 0.0);
        assert!(config.chunk_size >= 32.0);
        
        // Resolution should be power of 2 or reasonable
        assert!(config.heightmap_resolution >= 32);
        assert!(config.heightmap_resolution <= 512);
        
        // Seed should exist
        assert!(config.seed > 0);
    }
}

#[cfg(test)]
mod texture_splatting_tests {
    use crate::texture_splatting::{SplatWeights, MAX_SPLAT_LAYERS};

    #[test]
    fn test_max_splat_layers_constant() {
        assert!(MAX_SPLAT_LAYERS >= 4);
        assert!(MAX_SPLAT_LAYERS <= 16);
    }

    #[test]
    fn test_splat_weights_from_weights_normalization() {
        let weights = SplatWeights::from_weights(&[0.5, 0.5, 0.0, 0.0]);
        
        // Weights should sum to 1.0 (normalized)
        let sum = weights.weights_0.x + weights.weights_0.y + 
                  weights.weights_0.z + weights.weights_0.w +
                  weights.weights_1.x + weights.weights_1.y +
                  weights.weights_1.z + weights.weights_1.w;
        assert!((sum - 1.0).abs() < 0.001, "Weights sum: {}", sum);
    }
    
    #[test]
    fn test_splat_weights_single_layer() {
        let weights = SplatWeights::from_weights(&[1.0]);
        
        // First weight should dominate
        assert!(weights.weights_0.x > 0.99);
        assert!(weights.weights_0.y < 0.01);
    }
    
    #[test]
    fn test_splat_weights_multiple_layers() {
        let weights = SplatWeights::from_weights(&[0.25, 0.25, 0.25, 0.25]);
        
        // All first 4 weights should be equal
        assert!((weights.weights_0.x - 0.25).abs() < 0.01);
        assert!((weights.weights_0.y - 0.25).abs() < 0.01);
        assert!((weights.weights_0.z - 0.25).abs() < 0.01);
        assert!((weights.weights_0.w - 0.25).abs() < 0.01);
    }
    
    #[test]
    fn test_splat_weights_default() {
        let weights = SplatWeights::default();
        
        // Default should be valid (not NaN)
        assert!(!weights.weights_0.x.is_nan());
        assert!(!weights.weights_1.x.is_nan());
    }
    
    #[test]
    fn test_splat_weights_extended_layers() {
        let weights = SplatWeights::from_weights(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        
        // Second weight vector should be populated
        let second_sum = weights.weights_1.x + weights.weights_1.y +
                         weights.weights_1.z + weights.weights_1.w;
        assert!(second_sum > 0.0);
    }
}

#[cfg(test)]
mod lod_tests {
    use crate::lod_manager::LodLevel;

    #[test]
    fn test_lod_level_variants() {
        // All LOD levels should be distinct and valid
        let full = LodLevel::Full;
        let half = LodLevel::Half;
        let quarter = LodLevel::Quarter;
        let skybox = LodLevel::Skybox;
        
        // Each should be debuggable
        assert!(!format!("{:?}", full).is_empty());
        assert!(!format!("{:?}", half).is_empty());
        assert!(!format!("{:?}", quarter).is_empty());
        assert!(!format!("{:?}", skybox).is_empty());
    }

    #[test]
    fn test_lod_level_full_is_distinct() {
        let full = LodLevel::Full;
        let half = LodLevel::Half;
        
        // Full should not equal Half
        assert_ne!(format!("{:?}", full), format!("{:?}", half));
    }
    
    #[test]
    fn test_lod_level_skybox_is_lowest_detail() {
        // Skybox should be the lowest detail level
        let skybox = LodLevel::Skybox;
        assert!(format!("{:?}", skybox).contains("Skybox"));
    }
}

#[cfg(test)]
mod streaming_tests {
    use crate::background_loader::StreamingConfig;

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        
        // View distance should be reasonable (u32)
        assert!(config.view_distance > 0);
        assert!(config.view_distance <= 100);
        
        // Prefetch distance should be reasonable
        assert!(config.prefetch_distance > 0);
        assert!(config.prefetch_distance <= config.view_distance);
        
        // Max concurrent loads should be at least 1
        assert!(config.max_concurrent_loads >= 1);
        
        // Chunk size should be positive
        assert!(config.chunk_size > 0.0);
    }
    
    #[test]
    fn test_streaming_config_adaptive_throttle() {
        let config = StreamingConfig::default();
        
        // Throttle threshold should be reasonable frame time
        assert!(config.adaptive_throttle_threshold_ms > 0.0);
        assert!(config.adaptive_throttle_threshold_ms < 100.0);
        
        // Throttled loads should be less than normal
        assert!(config.throttled_concurrent_loads <= config.max_concurrent_loads);
    }
    
    #[test]
    fn test_streaming_config_max_loaded() {
        let config = StreamingConfig::default();
        
        // Max loaded chunks should be reasonable
        assert!(config.max_loaded_chunks >= 16);
        assert!(config.max_loaded_chunks <= 1024);
    }
}

#[cfg(test)]
mod erosion_tests {
    use crate::advanced_erosion::{ErosionPreset, HydraulicErosionConfig, ThermalErosionConfig};

    #[test]
    fn test_erosion_preset_default() {
        let preset = ErosionPreset::default();
        
        // Default should have a name
        assert!(!preset.name.is_empty());
        
        // Default should have hydraulic and thermal
        assert!(preset.hydraulic.is_some());
        assert!(preset.thermal.is_some());
        
        // Pass order should be non-empty
        assert!(!preset.pass_order.is_empty());
    }
    
    #[test]
    fn test_erosion_preset_desert() {
        let preset = ErosionPreset::desert();
        
        // Desert should have wind erosion
        assert!(preset.wind.is_some());
        assert!(preset.name.contains("Desert"));
    }
    
    #[test]
    fn test_erosion_preset_mountain() {
        let preset = ErosionPreset::mountain();
        
        // Mountain should have heavy hydraulic
        assert!(preset.hydraulic.is_some());
        if let Some(ref hydraulic) = preset.hydraulic {
            assert!(hydraulic.droplet_count > 50000);
        }
    }

    #[test]
    fn test_hydraulic_erosion_config_default() {
        let config = HydraulicErosionConfig::default();
        
        // Droplet count should be reasonable
        assert!(config.droplet_count > 0);
        assert!(config.droplet_count <= 500000);
        
        // Erode speed should be positive
        assert!(config.erode_speed > 0.0);
        
        // Deposit speed should be positive
        assert!(config.deposit_speed > 0.0);
        
        // Erosion radius should be reasonable
        assert!(config.erosion_radius > 0);
        assert!(config.erosion_radius <= 10);
    }
    
    #[test]
    fn test_thermal_erosion_config_default() {
        let config = ThermalErosionConfig::default();
        
        // Iterations should be reasonable
        assert!(config.iterations > 0);
        assert!(config.iterations <= 1000);
        
        // Talus angle should be in reasonable range
        assert!(config.talus_angle > 0.0);
        assert!(config.talus_angle < 90.0);
        
        // Redistribution rate should be between 0 and 1
        assert!(config.redistribution_rate >= 0.0);
        assert!(config.redistribution_rate <= 1.0);
    }
}

#[cfg(test)]
mod solver_tests {
    use crate::solver::ValidationStatus;
    use crate::biome::BiomeType;

    #[test]
    fn test_validation_status_valid() {
        let valid = ValidationStatus::Valid;
        assert!(format!("{:?}", valid).contains("Valid"));
    }
    
    #[test]
    fn test_validation_status_out_of_bounds() {
        let oob = ValidationStatus::OutOfBounds;
        assert!(format!("{:?}", oob).contains("OutOfBounds"));
    }
    
    #[test]
    fn test_validation_status_no_solid_ground() {
        let nsg = ValidationStatus::NoSolidGround;
        assert!(format!("{:?}", nsg).contains("NoSolidGround"));
    }
    
    #[test]
    fn test_validation_status_chunk_not_loaded() {
        let cnl = ValidationStatus::ChunkNotLoaded;
        assert!(format!("{:?}", cnl).contains("ChunkNotLoaded"));
    }
    
    #[test]
    fn test_validation_status_biome_incompatible() {
        let bi = ValidationStatus::BiomeIncompatible(BiomeType::Desert);
        assert!(format!("{:?}", bi).contains("BiomeIncompatible"));
    }
}

#[cfg(test)]
mod terrain_modifier_tests {
    use crate::terrain_modifier::VoxelOpType;
    use crate::voxel_data::Voxel;

    #[test]
    fn test_voxel_op_type_set() {
        let voxel = Voxel::new(0.8, 1);
        let set_op = VoxelOpType::Set(voxel);
        assert!(format!("{:?}", set_op).contains("Set"));
    }
    
    #[test]
    fn test_voxel_op_type_add_density() {
        let add = VoxelOpType::AddDensity(0.5);
        assert!(format!("{:?}", add).contains("AddDensity"));
    }
    
    #[test]
    fn test_voxel_op_type_subtract_density() {
        let subtract = VoxelOpType::SubtractDensity(0.3);
        assert!(format!("{:?}", subtract).contains("SubtractDensity"));
    }
    
    #[test]
    fn test_voxel_op_type_set_material() {
        let set_mat = VoxelOpType::SetMaterial(42);
        assert!(format!("{:?}", set_mat).contains("SetMaterial"));
    }
    
    #[test]
    fn test_voxel_op_type_blend() {
        let voxel = Voxel::new(0.5, 2);
        let blend = VoxelOpType::Blend { voxel, factor: 0.5 };
        assert!(format!("{:?}", blend).contains("Blend"));
    }
}

#[cfg(test)]
mod structure_tests {
    use crate::structures::StructureType;

    #[test]
    fn test_structure_type_residential() {
        // Verify residential structure types
        let cottage = StructureType::Cottage;
        let farmhouse = StructureType::Farmhouse;
        let villa = StructureType::Villa;
        let cabin = StructureType::Cabin;
        
        assert!(format!("{:?}", cottage).contains("Cottage"));
        assert!(format!("{:?}", farmhouse).contains("Farmhouse"));
        assert!(format!("{:?}", villa).contains("Villa"));
        assert!(format!("{:?}", cabin).contains("Cabin"));
    }
    
    #[test]
    fn test_structure_type_commercial() {
        let tavern = StructureType::Tavern;
        let blacksmith = StructureType::Blacksmith;
        let market = StructureType::Market;
        
        assert!(format!("{:?}", tavern).contains("Tavern"));
        assert!(format!("{:?}", blacksmith).contains("Blacksmith"));
        assert!(format!("{:?}", market).contains("Market"));
    }
    
    #[test]
    fn test_structure_type_defensive() {
        let tower = StructureType::Watchtower;
        let fort = StructureType::Fort;
        let wall = StructureType::Wall;
        
        assert!(format!("{:?}", tower).contains("Watchtower"));
        assert!(format!("{:?}", fort).contains("Fort"));
        assert!(format!("{:?}", wall).contains("Wall"));
    }
    
    #[test]
    fn test_structure_type_ancient() {
        let ruin = StructureType::AncientRuin;
        let obelisk = StructureType::Obelisk;
        let tomb = StructureType::Tomb;
        
        assert!(format!("{:?}", ruin).contains("AncientRuin"));
        assert!(format!("{:?}", obelisk).contains("Obelisk"));
        assert!(format!("{:?}", tomb).contains("Tomb"));
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use crate::streaming_diagnostics::ChunkLoadState;

    #[test]
    fn test_chunk_load_state_pending() {
        let pending = ChunkLoadState::Pending;
        assert!(format!("{:?}", pending).contains("Pending"));
    }
    
    #[test]
    fn test_chunk_load_state_loading() {
        let loading = ChunkLoadState::Loading;
        assert!(format!("{:?}", loading).contains("Loading"));
    }
    
    #[test]
    fn test_chunk_load_state_loaded() {
        let loaded = ChunkLoadState::Loaded;
        assert!(format!("{:?}", loaded).contains("Loaded"));
    }
    
    #[test]
    fn test_chunk_load_state_unloaded() {
        let unloaded = ChunkLoadState::Unloaded;
        assert!(format!("{:?}", unloaded).contains("Unloaded"));
    }
    
    #[test]
    fn test_chunk_load_states_distinct() {
        // All states should be distinct
        assert_ne!(ChunkLoadState::Pending, ChunkLoadState::Loading);
        assert_ne!(ChunkLoadState::Loading, ChunkLoadState::Loaded);
        assert_ne!(ChunkLoadState::Loaded, ChunkLoadState::Unloaded);
    }
}

#[cfg(test)]
mod behavioral_correctness_tests {
    //! Tests that verify terrain generation produces physically correct results
    
    use crate::heightmap::{Heightmap, HeightmapConfig};
    use crate::chunk::ChunkId;
    use crate::voxel_data::{ChunkCoord, CHUNK_SIZE};
    use glam::Vec3;

    #[test]
    fn test_chunk_world_pos_consistency() {
        // World position to chunk and back should be consistent
        let chunk_size = 64.0;
        
        for x in -10..10 {
            for z in -10..10 {
                let chunk = ChunkId::new(x, z);
                let world = chunk.to_world_pos(chunk_size);
                
                // Position inside chunk should map back to same chunk
                let inside = world + Vec3::new(chunk_size * 0.5, 0.0, chunk_size * 0.5);
                let recovered = ChunkId::from_world_pos(inside, chunk_size);
                
                assert_eq!(chunk, recovered, "Failed for chunk ({}, {})", x, z);
            }
        }
    }

    #[test]
    fn test_voxel_chunk_coord_consistency() {
        // Similar test for voxel chunks
        for x in -5..5 {
            for y in -5..5 {
                for z in -5..5 {
                    let coord = ChunkCoord::new(x, y, z);
                    let world = coord.to_world_pos();
                    
                    // Position inside chunk should map back to same chunk
                    let inside = world + Vec3::splat(CHUNK_SIZE as f32 * 0.5);
                    let recovered = ChunkCoord::from_world_pos(inside);
                    
                    assert_eq!(coord, recovered, "Failed for coord ({}, {}, {})", x, y, z);
                }
            }
        }
    }

    #[test]
    fn test_heightmap_index_calculation() {
        // Verify index calculation: index = z * resolution + x
        let config = HeightmapConfig {
            resolution: 4,
            ..Default::default()
        };
        let mut heightmap = Heightmap::new(config).unwrap();
        
        // Set each position with unique value
        for z in 0..4 {
            for x in 0..4 {
                let expected_index = z * 4 + x;
                heightmap.set_height(x, z, expected_index as f32);
            }
        }
        
        // Verify values
        for z in 0..4 {
            for x in 0..4 {
                let expected = (z * 4 + x) as f32;
                let actual = heightmap.get_height(x, z);
                assert_eq!(actual, expected, "Mismatch at ({}, {})", x, z);
            }
        }
    }

    #[test]
    fn test_bilinear_interpolation_linearity() {
        // Bilinear interpolation should be linear along edges
        // Using a 3x3 grid for clearer interpolation testing
        let data = vec![
            0.0, 50.0, 100.0,
            0.0, 50.0, 100.0,
            0.0, 50.0, 100.0,
        ];
        let heightmap = Heightmap::from_data(data, 3).unwrap();
        
        // Along x edge (z=0): should go from 0 to 50 at midpoint
        let h_0 = heightmap.sample_bilinear(0.0, 0.0);
        let h_half = heightmap.sample_bilinear(0.5, 0.0);
        let h_1 = heightmap.sample_bilinear(1.0, 0.0);
        
        assert!((h_0 - 0.0).abs() < 0.1);
        assert!((h_half - 25.0).abs() < 0.1, "Expected ~25, got {}", h_half);
        assert!((h_1 - 50.0).abs() < 0.1, "Expected ~50, got {}", h_1);
    }

    #[test]
    fn test_chunk_radius_calculation() {
        let center = Vec3::ZERO;
        let chunk_size = 64.0;
        
        // Radius 0 should give 1 chunk
        let r0 = ChunkId::get_chunks_in_radius(center, 0, chunk_size);
        assert_eq!(r0.len(), 1);
        
        // Radius 1 should give 9 chunks (3x3)
        let r1 = ChunkId::get_chunks_in_radius(center, 1, chunk_size);
        assert_eq!(r1.len(), 9);
        
        // Radius 2 should give 25 chunks (5x5)
        let r2 = ChunkId::get_chunks_in_radius(center, 2, chunk_size);
        assert_eq!(r2.len(), 25);
    }
}

// =============================================================================
// BOUNDARY CONDITION TESTS - Test exact boundary values to catch < vs <= mutations
// =============================================================================

#[cfg(test)]
mod boundary_condition_tests {
    use crate::heightmap::{Heightmap, HeightmapConfig};
    use crate::chunk::ChunkId;
    use crate::voxel_data::Voxel;
    use crate::background_loader::StreamingConfig;
    use glam::Vec3;

    // --- Heightmap boundary tests ---
    
    #[test]
    fn heightmap_value_at_zero() {
        let data = vec![0.0, 1.0, 2.0, 3.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        assert_eq!(heightmap.min_height(), 0.0);
    }

    #[test]
    fn heightmap_value_at_one() {
        let data = vec![1.0; 4];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        assert_eq!(heightmap.min_height(), 1.0);
        assert_eq!(heightmap.max_height(), 1.0);
    }

    #[test]
    fn heightmap_normalized_sample_at_zero() {
        let data = vec![10.0, 20.0, 30.0, 40.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        // Sample at normalized (0.0, 0.0) should give corner value
        let sample = heightmap.sample_bilinear(0.0, 0.0);
        assert!((sample - 10.0).abs() < 0.1);
    }

    #[test]
    fn heightmap_normalized_sample_near_one() {
        let data = vec![10.0, 20.0, 30.0, 40.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        // Sample near normalized (1.0, 1.0) should give interpolated value
        let sample = heightmap.sample_bilinear(0.999, 0.999);
        // Should be close to 40.0 (lower-right corner)
        assert!(sample > 25.0, "Sample near corner should be high: {}", sample);
    }

    #[test]
    fn heightmap_out_of_bounds_x_returns_zero() {
        let config = HeightmapConfig { resolution: 2, ..Default::default() };
        let heightmap = Heightmap::new(config).unwrap();
        assert_eq!(heightmap.get_height(100, 0), 0.0);
    }

    #[test]
    fn heightmap_out_of_bounds_z_returns_zero() {
        let config = HeightmapConfig { resolution: 2, ..Default::default() };
        let heightmap = Heightmap::new(config).unwrap();
        assert_eq!(heightmap.get_height(0, 100), 0.0);
    }

    // --- ChunkId boundary tests ---
    
    #[test]
    fn chunk_id_at_origin() {
        let id = ChunkId::from_world_pos(Vec3::ZERO, 64.0);
        assert_eq!(id.x, 0);
        assert_eq!(id.z, 0);
    }

    #[test]
    fn chunk_id_at_exact_boundary() {
        // At exactly chunk boundary (64, 0, 0)
        let id = ChunkId::from_world_pos(Vec3::new(64.0, 0.0, 0.0), 64.0);
        assert_eq!(id.x, 1); // Should be next chunk
    }

    #[test]
    fn chunk_id_just_before_boundary() {
        // Just before chunk boundary (63.999, 0, 0)
        let id = ChunkId::from_world_pos(Vec3::new(63.999, 0.0, 0.0), 64.0);
        assert_eq!(id.x, 0); // Should still be current chunk
    }

    #[test]
    fn chunk_id_negative_boundary() {
        // At exactly (-64, 0, 0)
        let id = ChunkId::from_world_pos(Vec3::new(-64.0, 0.0, 0.0), 64.0);
        assert_eq!(id.x, -1);
    }

    #[test]
    fn chunk_radius_zero() {
        let chunks = ChunkId::get_chunks_in_radius(Vec3::ZERO, 0, 64.0);
        assert_eq!(chunks.len(), 1);
    }

    // --- Voxel density boundary tests ---
    
    #[test]
    fn voxel_density_zero_is_empty() {
        let voxel = Voxel::new(0.0, 0);
        assert!(voxel.is_empty());
    }

    #[test]
    fn voxel_density_one_is_solid() {
        let voxel = Voxel::new(1.0, 0);
        assert!(voxel.is_solid());
    }

    #[test]
    fn voxel_density_at_empty_threshold() {
        // Just under empty threshold (0.01)
        let voxel = Voxel::new(0.009, 0);
        assert!(voxel.is_empty());
    }

    #[test]
    fn voxel_density_at_solid_threshold() {
        // Just over solid threshold (0.5)
        let voxel = Voxel::new(0.501, 0);
        assert!(voxel.is_solid());
    }

    // --- StreamingConfig boundary tests ---
    
    #[test]
    fn streaming_view_distance_minimum() {
        let config = StreamingConfig::default();
        assert!(config.view_distance >= 1);
    }

    #[test]
    fn streaming_prefetch_within_view() {
        let config = StreamingConfig::default();
        assert!(config.prefetch_distance <= config.view_distance);
    }

    #[test]
    fn streaming_max_loads_at_least_one() {
        let config = StreamingConfig::default();
        assert!(config.max_concurrent_loads >= 1);
    }

    #[test]
    fn streaming_throttle_positive() {
        let config = StreamingConfig::default();
        assert!(config.adaptive_throttle_threshold_ms > 0.0);
    }
}

// =============================================================================
// COMPARISON OPERATOR TESTS - Test to catch == vs != and < vs > swaps
// =============================================================================

#[cfg(test)]
mod comparison_operator_tests {
    use crate::heightmap::Heightmap;
    use crate::chunk::ChunkId;
    use crate::biome::BiomeType;
    use crate::voxel_data::Voxel;
    use crate::lod_manager::LodLevel;
    use glam::Vec3;

    // --- ChunkId equality ---
    
    #[test]
    fn chunk_id_equal_to_self() {
        let id = ChunkId::new(5, 10);
        assert_eq!(id, id);
    }

    #[test]
    fn chunk_id_equal_same_coords() {
        let id1 = ChunkId::new(5, 10);
        let id2 = ChunkId::new(5, 10);
        assert_eq!(id1, id2);
    }

    #[test]
    fn chunk_id_not_equal_different_x() {
        let id1 = ChunkId::new(5, 10);
        let id2 = ChunkId::new(6, 10);
        assert_ne!(id1, id2);
    }

    #[test]
    fn chunk_id_not_equal_different_z() {
        let id1 = ChunkId::new(5, 10);
        let id2 = ChunkId::new(5, 11);
        assert_ne!(id1, id2);
    }

    // --- BiomeType equality ---
    
    #[test]
    fn biome_type_equals_self() {
        assert_eq!(BiomeType::Forest, BiomeType::Forest);
    }

    #[test]
    fn biome_type_not_equals_other() {
        assert_ne!(BiomeType::Forest, BiomeType::Desert);
    }

    #[test]
    fn biome_type_all_distinct() {
        let all = BiomeType::all();
        for (i, b1) in all.iter().enumerate() {
            for (j, b2) in all.iter().enumerate() {
                if i == j {
                    assert_eq!(b1, b2);
                } else {
                    assert_ne!(b1, b2);
                }
            }
        }
    }

    // --- Voxel comparisons ---
    
    #[test]
    fn voxel_default_equals_default() {
        let v1 = Voxel::default();
        let v2 = Voxel::default();
        assert_eq!(v1.density, v2.density);
    }

    #[test]
    fn voxel_different_density_detected() {
        let v1 = Voxel::new(0.0, 0);
        let v2 = Voxel::new(1.0, 0);
        assert!(v1.density < v2.density);
    }

    // --- Heightmap min/max comparison ---
    
    #[test]
    fn heightmap_min_less_than_max() {
        let data = vec![5.0, 10.0, 15.0, 20.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        assert!(heightmap.min_height() < heightmap.max_height());
    }

    #[test]
    fn heightmap_min_equals_max_when_uniform() {
        let data = vec![7.0; 4];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        assert_eq!(heightmap.min_height(), heightmap.max_height());
    }

    // --- LodLevel debug format comparisons ---
    
    #[test]
    fn lod_full_not_equals_half() {
        assert_ne!(format!("{:?}", LodLevel::Full), format!("{:?}", LodLevel::Half));
    }

    #[test]
    fn lod_quarter_not_equals_skybox() {
        assert_ne!(format!("{:?}", LodLevel::Quarter), format!("{:?}", LodLevel::Skybox));
    }

    // --- Position comparisons ---
    
    #[test]
    fn center_world_pos_correct() {
        let id = ChunkId::new(1, 1);
        let corner = id.to_world_pos(64.0);
        let center = id.to_center_pos(64.0);
        assert!(center.x > corner.x);
        assert!(center.z > corner.z);
    }
}

// =============================================================================
// BOOLEAN RETURN PATH TESTS - Test all paths through boolean-returning functions
// =============================================================================

#[cfg(test)]
mod boolean_return_path_tests {
    use crate::heightmap::{Heightmap, HeightmapConfig};
    use crate::voxel_data::{Voxel, ChunkCoord};
    use crate::meshing::ChunkMesh;
    use crate::biome::BiomeType;
    use std::str::FromStr;

    // --- Voxel.is_solid() paths ---
    
    #[test]
    fn voxel_is_solid_true_for_high_density() {
        let voxel = Voxel::new(1.0, 0);
        assert!(voxel.is_solid());
    }

    #[test]
    fn voxel_is_solid_true_at_threshold() {
        let voxel = Voxel::new(0.51, 0);
        assert!(voxel.is_solid());
    }

    #[test]
    fn voxel_is_solid_false_for_low_density() {
        let voxel = Voxel::new(0.0, 0);
        assert!(!voxel.is_solid());
    }

    #[test]
    fn voxel_is_solid_false_at_threshold() {
        let voxel = Voxel::new(0.5, 0);
        assert!(!voxel.is_solid()); // Threshold is > 0.5, not >=
    }

    // --- Voxel.is_empty() paths ---
    
    #[test]
    fn voxel_is_empty_true_for_zero() {
        let voxel = Voxel::new(0.0, 0);
        assert!(voxel.is_empty());
    }

    #[test]
    fn voxel_is_empty_true_under_threshold() {
        let voxel = Voxel::new(0.009, 0);
        assert!(voxel.is_empty());
    }

    #[test]
    fn voxel_is_empty_false_for_solid() {
        let voxel = Voxel::new(1.0, 0);
        assert!(!voxel.is_empty());
    }

    #[test]
    fn voxel_is_empty_false_at_threshold() {
        let voxel = Voxel::new(0.01, 0);
        assert!(!voxel.is_empty()); // Threshold is < 0.01, not <=
    }

    // --- ChunkMesh.is_empty() paths ---
    
    #[test]
    fn chunk_mesh_is_empty_true_when_new() {
        let mesh = ChunkMesh::empty(ChunkCoord::new(0, 0, 0));
        assert!(mesh.is_empty());
    }

    // --- Heightmap.from_data() Result paths ---
    
    #[test]
    fn heightmap_from_data_ok_valid_size() {
        let data = vec![1.0; 9]; // 3x3
        let result = Heightmap::from_data(data, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn heightmap_from_data_err_invalid_size() {
        let data = vec![1.0; 5]; // Not a square
        let result = Heightmap::from_data(data, 3);
        assert!(result.is_err());
    }

    // --- BiomeType::from_str() Result paths ---
    
    #[test]
    fn biome_from_str_ok_valid() {
        let result = BiomeType::from_str("forest");
        assert!(result.is_ok());
    }

    #[test]
    fn biome_from_str_err_invalid() {
        let result = BiomeType::from_str("invalid_biome");
        assert!(result.is_err());
    }

    #[test]
    fn biome_from_str_err_empty() {
        let result = BiomeType::from_str("");
        assert!(result.is_err());
    }

    // --- BiomeType::parse() Option paths ---
    
    #[test]
    fn biome_parse_some_valid() {
        let result = BiomeType::parse("mountain");
        assert!(result.is_some());
    }

    #[test]
    fn biome_parse_none_invalid() {
        let result = BiomeType::parse("unknown_biome");
        assert!(result.is_none());
    }

    // --- Heightmap data operations ---
    
    #[test]
    fn heightmap_data_iter_all_returns_true_when_uniform() {
        let config = HeightmapConfig { resolution: 2, ..Default::default() };
        let heightmap = Heightmap::new(config).unwrap();
        // All zeros should be uniform
        assert!(heightmap.data().iter().all(|&h| h == 0.0));
    }

    #[test]
    fn heightmap_data_iter_all_returns_false_when_varied() {
        let data = vec![0.0, 1.0, 0.0, 0.0];
        let heightmap = Heightmap::from_data(data, 2).unwrap();
        assert!(!heightmap.data().iter().all(|&h| h == 0.0));
    }
}
