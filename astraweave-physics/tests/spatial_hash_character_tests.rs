//! Day 6: Tests for spatial_hash.rs and character controller (astraweave-physics)
//!
//! Target coverage:
//! - spatial_hash.rs: Expand existing tests, add edge cases
//! - lib.rs: Character controller API (add_character, control_character, body_transform)
//!
//! Test categories:
//! - Spatial Hash: AABB operations, grid operations, query performance
//! - Character Controller: Movement, state, collision

use astraweave_physics::{CharState, PhysicsWorld, SpatialHash, AABB};
use glam::Vec3;

// ============================================================================
// Spatial Hash AABB Tests (10 tests)
// ============================================================================

#[test]
fn test_aabb_from_center_extents() {
    let aabb = AABB::from_center_extents(Vec3::new(5.0, 5.0, 5.0), Vec3::new(1.0, 2.0, 3.0));

    assert_eq!(aabb.min, Vec3::new(4.0, 3.0, 2.0));
    assert_eq!(aabb.max, Vec3::new(6.0, 7.0, 8.0));
}

#[test]
fn test_aabb_from_sphere() {
    let aabb = AABB::from_sphere(Vec3::new(10.0, 10.0, 10.0), 5.0);

    assert_eq!(aabb.min, Vec3::new(5.0, 5.0, 5.0));
    assert_eq!(aabb.max, Vec3::new(15.0, 15.0, 15.0));
}

#[test]
fn test_aabb_center() {
    let aabb = AABB {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(10.0, 10.0, 10.0),
    };

    assert_eq!(aabb.center(), Vec3::new(5.0, 5.0, 5.0));
}

#[test]
fn test_aabb_half_extents() {
    let aabb = AABB {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(10.0, 20.0, 30.0),
    };

    assert_eq!(aabb.half_extents(), Vec3::new(5.0, 10.0, 15.0));
}

#[test]
fn test_aabb_intersection_overlapping() {
    let aabb1 = AABB::from_sphere(Vec3::new(0.0, 0.0, 0.0), 2.0);
    let aabb2 = AABB::from_sphere(Vec3::new(3.0, 0.0, 0.0), 2.0);

    // AABBs overlap: aabb1 [−2,2] vs aabb2 [1,5] → overlap at [1,2]
    assert!(aabb1.intersects(&aabb2));
    assert!(aabb2.intersects(&aabb1)); // Symmetry
}

#[test]
fn test_aabb_intersection_touching() {
    let aabb1 = AABB::from_sphere(Vec3::new(0.0, 0.0, 0.0), 1.0);
    let aabb2 = AABB::from_sphere(Vec3::new(2.0, 0.0, 0.0), 1.0);

    // AABBs touch exactly: aabb1 [−1,1] vs aabb2 [1,3] → touch at x=1
    assert!(aabb1.intersects(&aabb2));
}

#[test]
fn test_aabb_intersection_separated() {
    let aabb1 = AABB::from_sphere(Vec3::new(0.0, 0.0, 0.0), 1.0);
    let aabb2 = AABB::from_sphere(Vec3::new(10.0, 0.0, 0.0), 1.0);

    // AABBs separated: aabb1 [−1,1] vs aabb2 [9,11] → no overlap
    assert!(!aabb1.intersects(&aabb2));
}

#[test]
fn test_aabb_intersection_fully_contained() {
    let large = AABB::from_sphere(Vec3::new(0.0, 0.0, 0.0), 10.0);
    let small = AABB::from_sphere(Vec3::new(0.0, 0.0, 0.0), 1.0);

    // Small AABB fully contained in large
    assert!(large.intersects(&small));
    assert!(small.intersects(&large)); // Symmetry
}

#[test]
fn test_aabb_intersection_3d() {
    let aabb1 = AABB {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(5.0, 5.0, 5.0),
    };
    let aabb2 = AABB {
        min: Vec3::new(4.0, 4.0, 4.0),
        max: Vec3::new(10.0, 10.0, 10.0),
    };

    // 3D overlap at corner
    assert!(aabb1.intersects(&aabb2));
}

#[test]
fn test_aabb_intersection_negative_coords() {
    let aabb1 = AABB::from_sphere(Vec3::new(-5.0, -5.0, -5.0), 2.0);
    let aabb2 = AABB::from_sphere(Vec3::new(-3.0, -5.0, -5.0), 2.0);

    // Overlap in negative coordinate space
    assert!(aabb1.intersects(&aabb2));
}

// ============================================================================
// Spatial Hash Grid Tests (15 tests)
// ============================================================================

#[test]
fn test_spatial_hash_new() {
    let grid = SpatialHash::<u32>::new(5.0);

    assert_eq!(grid.cell_size(), 5.0);
    assert_eq!(grid.object_count(), 0);
    assert_eq!(grid.cell_count(), 0);
}

#[test]
#[should_panic(expected = "Cell size must be positive")]
fn test_spatial_hash_new_invalid_cell_size() {
    let _grid = SpatialHash::<u32>::new(0.0);
}

#[test]
fn test_spatial_hash_insert_single() {
    let mut grid = SpatialHash::<u32>::new(10.0);
    let aabb = AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 1.0);

    grid.insert(1, aabb);

    assert_eq!(grid.object_count(), 1);
    assert!(grid.cell_count() > 0);
}

#[test]
fn test_spatial_hash_insert_multiple_same_cell() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // All objects tightly packed in same cell (0,0,0)
    for i in 0..5 {
        let aabb = AABB::from_sphere(Vec3::new(2.0 + i as f32 * 0.2, 0.0, 0.0), 0.5);
        grid.insert(i, aabb);
    }

    assert_eq!(grid.object_count(), 5);
    // Note: Objects might span multiple cells if they overlap boundaries
    // Just check that at least one cell exists
    assert!(grid.cell_count() >= 1);
}

#[test]
fn test_spatial_hash_insert_multiple_different_cells() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // Objects in different cells
    grid.insert(1, AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 1.0)); // Cell (0,0,0)
    grid.insert(2, AABB::from_sphere(Vec3::new(15.0, 0.0, 0.0), 1.0)); // Cell (1,0,0)
    grid.insert(3, AABB::from_sphere(Vec3::new(25.0, 0.0, 0.0), 1.0)); // Cell (2,0,0)

    assert_eq!(grid.object_count(), 3);
    assert!(grid.cell_count() >= 3); // At least 3 cells
}

#[test]
fn test_spatial_hash_query_empty() {
    let grid = SpatialHash::<u32>::new(10.0);
    let query_aabb = AABB::from_sphere(Vec3::ZERO, 1.0);

    let results = grid.query(query_aabb);

    assert!(results.is_empty());
}

#[test]
fn test_spatial_hash_query_finds_object() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    grid.insert(42, AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 1.0));

    // Query same cell
    let results = grid.query(AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 1.0));

    assert!(results.contains(&42));
}

#[test]
fn test_spatial_hash_query_spatial_filtering() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // Object at x=5 (cell 0)
    grid.insert(1, AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 1.0));
    // Object at x=25 (cell 2)
    grid.insert(2, AABB::from_sphere(Vec3::new(25.0, 0.0, 0.0), 1.0));

    // Query cell 0
    let results = grid.query(AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 1.0));

    assert!(results.contains(&1));
    assert!(!results.contains(&2)); // Object 2 is in different cell
}

#[test]
fn test_spatial_hash_clear() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    grid.insert(1, AABB::from_sphere(Vec3::ZERO, 1.0));
    grid.insert(2, AABB::from_sphere(Vec3::new(10.0, 0.0, 0.0), 1.0));

    assert_eq!(grid.object_count(), 2);

    grid.clear();

    assert_eq!(grid.object_count(), 0);
    assert_eq!(grid.cell_count(), 0);
}

#[test]
fn test_spatial_hash_multi_cell_spanning() {
    let mut grid = SpatialHash::<u32>::new(5.0);

    // Large AABB spanning 4 cells in X-Z plane
    let large_aabb = AABB {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(10.0, 1.0, 10.0),
    };

    grid.insert(1, large_aabb);

    // Should be in 2×1×2 = 4 cells
    assert!(
        grid.cell_count() >= 4,
        "Large object should span multiple cells"
    );
}

#[test]
fn test_spatial_hash_query_unique() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // Insert object spanning 2 cells
    let spanning_aabb = AABB {
        min: Vec3::new(8.0, 0.0, 0.0),
        max: Vec3::new(12.0, 0.0, 0.0),
    };

    grid.insert(1, spanning_aabb);

    // Query both cells
    let query_aabb = AABB {
        min: Vec3::new(8.0, 0.0, 0.0),
        max: Vec3::new(12.0, 0.0, 0.0),
    };

    let results = grid.query(query_aabb);
    let unique_results = grid.query_unique(query_aabb);

    // query() may return duplicates (object 1 appears in 2 cells)
    // query_unique() should deduplicate
    assert!(results.len() >= unique_results.len());
    assert_eq!(unique_results.len(), 1);
    assert_eq!(unique_results[0], 1);
}

#[test]
fn test_spatial_hash_average_cell_density() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // Insert 6 small objects in well-separated cells
    for i in 0..3 {
        grid.insert(i, AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 0.1)); // Small radius to stay in 1 cell
    }
    for i in 3..6 {
        grid.insert(i, AABB::from_sphere(Vec3::new(25.0, 0.0, 0.0), 0.1)); // Far away, different cell
    }

    let density = grid.average_cell_density();

    assert_eq!(grid.object_count(), 6);
    // Objects may still span multiple cells depending on cell boundaries
    // Just check density is reasonable
    assert!(density > 0.0);
    assert!(density <= 6.0); // At most all objects in 1 cell
}

#[test]
fn test_spatial_hash_stats() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // Insert 3 objects in same cell
    for i in 0..3 {
        grid.insert(i, AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 0.5));
    }

    // Insert 1 object in different cell
    grid.insert(3, AABB::from_sphere(Vec3::new(25.0, 25.0, 25.0), 0.5));

    let stats = grid.stats();

    assert_eq!(stats.object_count, 4);
    assert_eq!(stats.cell_count, 2);
    assert_eq!(stats.max_objects_per_cell, 3);
    assert_eq!(stats.average_objects_per_cell, 2.0); // 4 objects / 2 cells
    assert_eq!(stats.cell_size, 10.0);
}

#[test]
fn test_spatial_hash_negative_coordinates() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // Insert in negative space
    grid.insert(1, AABB::from_sphere(Vec3::new(-5.0, -5.0, -5.0), 1.0));
    grid.insert(2, AABB::from_sphere(Vec3::new(-15.0, -15.0, -15.0), 1.0));

    // Query negative space
    let results = grid.query(AABB::from_sphere(Vec3::new(-5.0, -5.0, -5.0), 1.0));

    assert!(results.contains(&1));
    assert!(!results.contains(&2)); // Different cell
}

#[test]
fn test_spatial_hash_cell_boundary() {
    let mut grid = SpatialHash::<u32>::new(10.0);

    // Objects well within different cells (far from boundaries)
    grid.insert(1, AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 0.1)); // Cell (0,0,0), small radius
    grid.insert(2, AABB::from_sphere(Vec3::new(25.0, 0.0, 0.0), 0.1)); // Cell (2,0,0), small radius

    // Query cell 0
    let results1 = grid.query(AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 0.1));

    // Query cell 2
    let results2 = grid.query(AABB::from_sphere(Vec3::new(25.0, 0.0, 0.0), 0.1));

    // Object 1 should only be in cell 0
    assert!(results1.contains(&1));

    // Object 2 should only be in cell 2
    assert!(results2.contains(&2));

    // Cross-queries should not find objects in other cells
    assert!(!results1.contains(&2));
    assert!(!results2.contains(&1));
}

// ============================================================================
// Character Controller Tests (8 tests)
// ============================================================================

#[test]
fn test_add_character() {
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));

    let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

    // Character should be added to char_map
    assert!(pw.char_map.contains_key(&char_id));

    // Character should have Grounded state
    let char_ctrl = pw.char_map.get(&char_id).unwrap();
    assert_eq!(char_ctrl.state, CharState::Grounded);
}

#[test]
fn test_character_controller_properties() {
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));

    let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));

    let char_ctrl = pw.char_map.get(&char_id).unwrap();

    // Check default properties
    assert_eq!(char_ctrl.max_climb_angle_deg, 70.0); // Default value from lib.rs
    assert_eq!(char_ctrl.max_step, 0.4);
    // Radius and height should match half-extents
    assert!((char_ctrl.radius - 0.4).abs() < 0.01);
    assert!((char_ctrl.height - 1.8).abs() < 0.01); // height = 2 * y_half_extent
}

#[test]
fn test_body_transform_exists() {
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));

    let char_id = pw.add_character(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

    let transform = pw.body_transform(char_id);

    assert!(transform.is_some());

    // Check initial position (Y should be around 5.0)
    let mat = transform.unwrap();
    assert!((mat.w_axis.y - 5.0).abs() < 1.0); // Within 1 unit (physics settling)
}

#[test]
fn test_body_transform_nonexistent() {
    let pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));

    let transform = pw.body_transform(99999);

    assert!(transform.is_none());
}

#[test]
fn test_control_character_no_movement() {
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
    let _ground = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
    let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));

    // Apply zero movement for a few steps
    for _ in 0..10 {
        pw.control_character(char_id, Vec3::ZERO, 1.0 / 60.0, false);
        pw.step();
    }

    let transform = pw.body_transform(char_id).unwrap();

    // Character should stay near origin (no horizontal movement)
    assert!(transform.w_axis.x.abs() < 0.1);
    assert!(transform.w_axis.z.abs() < 0.1);
}

#[test]
fn test_control_character_vertical_movement() {
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
    let _ground = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
    let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));

    // Apply jump
    pw.jump(char_id, 1.0);

    // Apply upward movement (jump simulation)
    for _ in 0..5 {
        pw.control_character(char_id, Vec3::ZERO, 1.0 / 60.0, false);
        pw.step();
    }

    let transform = pw.body_transform(char_id).unwrap();

    // Character should move upward (Y increases)
    assert!(
        transform.w_axis.y > 1.0,
        "Character should move upward, y={}",
        transform.w_axis.y
    );
}

#[test]
fn test_create_ground_plane() {
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));

    let ground_id = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);

    // Ground should be added
    assert!(ground_id > 0);

    // Ground should have a transform
    let transform = pw.body_transform(ground_id);
    assert!(transform.is_some());
}

#[test]
fn test_multiple_characters() {
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));

    let char1 = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
    let char2 = pw.add_character(Vec3::new(5.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
    let char3 = pw.add_character(Vec3::new(-5.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

    // All characters should be unique
    assert_ne!(char1, char2);
    assert_ne!(char2, char3);
    assert_ne!(char1, char3);

    // All should have transforms
    assert!(pw.body_transform(char1).is_some());
    assert!(pw.body_transform(char2).is_some());
    assert!(pw.body_transform(char3).is_some());
}
