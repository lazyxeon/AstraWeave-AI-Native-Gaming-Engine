//! Comprehensive mutation-resistant tests for astraweave-scene
//!
//! These tests are designed to achieve 90%+ mutation kill rate by:
//! - Testing all enum variants and their behaviors
//! - Verifying state transitions and side effects
//! - Checking boundary conditions and edge cases
//! - Testing return values from all public methods

use astraweave_scene::streaming::*;
use astraweave_scene::world_partition::*;
use astraweave_scene::*;
use glam::{Mat4, Quat, Vec3, Vec4};

// ═══════════════════════════════════════════════════════════════════════════
// TRANSFORM TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod transform_tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let t = Transform::default();
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform::identity();
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_new() {
        let t = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_y(1.0),
            Vec3::new(2.0, 2.0, 2.0),
        );
        assert_eq!(t.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(t.scale, Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_transform_from_translation() {
        let t = Transform::from_translation(Vec3::new(5.0, 6.0, 7.0));
        assert_eq!(t.translation, Vec3::new(5.0, 6.0, 7.0));
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_from_rotation() {
        let rot = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let t = Transform::from_rotation(rot);
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, rot);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_from_scale() {
        let t = Transform::from_scale(3.0);
        assert_eq!(t.scale, Vec3::splat(3.0));
        assert_eq!(t.translation, Vec3::ZERO);
    }

    #[test]
    fn test_transform_from_scale_vec() {
        let t = Transform::from_scale_vec(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(t.scale, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_transform_matrix() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let m = t.matrix();
        // Translation should be in the 4th column
        let translation = m.w_axis;
        assert!((translation.x - 1.0).abs() < 0.001);
        assert!((translation.y - 2.0).abs() < 0.001);
        assert!((translation.z - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_is_identity_true() {
        let t = Transform::identity();
        assert!(t.is_identity());
    }

    #[test]
    fn test_transform_is_identity_false_translation() {
        let t = Transform::from_translation(Vec3::ONE);
        assert!(!t.is_identity());
    }

    #[test]
    fn test_transform_is_identity_false_scale() {
        let t = Transform::from_scale(2.0);
        assert!(!t.is_identity());
    }

    #[test]
    fn test_transform_is_uniform_scale_true() {
        let t = Transform::from_scale(3.0);
        assert!(t.is_uniform_scale());
    }

    #[test]
    fn test_transform_is_uniform_scale_false() {
        let t = Transform::from_scale_vec(Vec3::new(1.0, 2.0, 3.0));
        assert!(!t.is_uniform_scale());
    }

    #[test]
    fn test_transform_uniform_scale() {
        let t = Transform::from_scale_vec(Vec3::new(1.0, 2.0, 3.0));
        let avg = t.uniform_scale();
        assert!((avg - 2.0).abs() < 0.001); // (1+2+3)/3 = 2
    }

    #[test]
    fn test_transform_forward() {
        let t = Transform::identity();
        let fwd = t.forward();
        assert!((fwd - Vec3::NEG_Z).length() < 0.001);
    }

    #[test]
    fn test_transform_right() {
        let t = Transform::identity();
        let right = t.right();
        assert!((right - Vec3::X).length() < 0.001);
    }

    #[test]
    fn test_transform_up() {
        let t = Transform::identity();
        let up = t.up();
        assert!((up - Vec3::Y).length() < 0.001);
    }

    #[test]
    fn test_transform_inverse() {
        let t = Transform::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let inv = t.inverse();
        assert!((inv.translation.x + 5.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_transform_point() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let p = t.transform_point(Vec3::ZERO);
        assert_eq!(p, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_transform_transform_direction() {
        let t = Transform::from_translation(Vec3::new(100.0, 100.0, 100.0));
        let d = t.transform_direction(Vec3::X);
        assert_eq!(d, Vec3::X); // Translation shouldn't affect direction
    }

    #[test]
    fn test_transform_lerp() {
        let a = Transform::from_translation(Vec3::ZERO);
        let b = Transform::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let mid = a.lerp(&b, 0.5);
        assert!((mid.translation.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_clone() {
        let t = Transform::from_translation(Vec3::ONE);
        let cloned = t;
        assert_eq!(t.translation, cloned.translation);
    }

    #[test]
    fn test_transform_eq() {
        let a = Transform::from_scale(2.0);
        let b = Transform::from_scale(2.0);
        assert_eq!(a, b);
    }

    #[test]
    fn test_transform_ne() {
        let a = Transform::from_scale(2.0);
        let b = Transform::from_scale(3.0);
        assert_ne!(a, b);
    }

    #[test]
    fn test_transform_display() {
        let t = Transform::identity();
        let s = format!("{}", t);
        assert!(s.contains("Transform"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NODE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod node_tests {
    use super::*;

    #[test]
    fn test_node_new() {
        let n = Node::new("TestNode");
        assert_eq!(n.name, "TestNode");
        assert!(n.children.is_empty());
    }

    #[test]
    fn test_node_with_transform() {
        let t = Transform::from_translation(Vec3::ONE);
        let n = Node::with_transform("Node", t);
        assert_eq!(n.transform.translation, Vec3::ONE);
    }

    #[test]
    fn test_node_has_children_false() {
        let n = Node::new("Leaf");
        assert!(!n.has_children());
    }

    #[test]
    fn test_node_has_children_true() {
        let mut n = Node::new("Parent");
        n.add_child(Node::new("Child"));
        assert!(n.has_children());
    }

    #[test]
    fn test_node_child_count() {
        let mut n = Node::new("Parent");
        n.add_child(Node::new("A"));
        n.add_child(Node::new("B"));
        n.add_child(Node::new("C"));
        assert_eq!(n.child_count(), 3);
    }

    #[test]
    fn test_node_descendant_count() {
        let mut root = Node::new("Root");
        let mut a = Node::new("A");
        a.add_child(Node::new("A1"));
        a.add_child(Node::new("A2"));
        root.add_child(a);
        root.add_child(Node::new("B"));
        // Root has 2 children: A (with 2), B
        // Descendants: A + A1 + A2 + B = 4
        assert_eq!(root.descendant_count(), 4);
    }

    #[test]
    fn test_node_is_leaf_true() {
        let n = Node::new("Leaf");
        assert!(n.is_leaf());
    }

    #[test]
    fn test_node_is_leaf_false() {
        let mut n = Node::new("Parent");
        n.add_child(Node::new("Child"));
        assert!(!n.is_leaf());
    }

    #[test]
    fn test_node_find_child() {
        let mut n = Node::new("Parent");
        n.add_child(Node::new("Child1"));
        n.add_child(Node::new("Child2"));
        assert!(n.find_child("Child1").is_some());
        assert!(n.find_child("NotExists").is_none());
    }

    #[test]
    fn test_node_find_child_mut() {
        let mut n = Node::new("Parent");
        n.add_child(Node::new("Child"));
        let child = n.find_child_mut("Child").unwrap();
        child.name = "Renamed".to_string();
        assert_eq!(n.find_child("Renamed").unwrap().name, "Renamed");
    }

    #[test]
    fn test_node_find_descendant() {
        let mut root = Node::new("Root");
        let mut a = Node::new("A");
        a.add_child(Node::new("DeepChild"));
        root.add_child(a);
        assert!(root.find_descendant("DeepChild").is_some());
    }

    #[test]
    fn test_node_find_descendant_not_found() {
        let n = Node::new("Root");
        assert!(n.find_descendant("NotExists").is_none());
    }

    #[test]
    fn test_node_depth_leaf() {
        let n = Node::new("Leaf");
        assert_eq!(n.depth(), 0);
    }

    #[test]
    fn test_node_depth_with_children() {
        let mut root = Node::new("Root");
        let mut a = Node::new("A");
        a.add_child(Node::new("A1"));
        root.add_child(a);
        // Depth: Root→A→A1 = 2
        assert_eq!(root.depth(), 2);
    }

    #[test]
    fn test_node_clone() {
        let n = Node::new("Test");
        let cloned = n.clone();
        assert_eq!(n.name, cloned.name);
    }

    #[test]
    fn test_node_eq() {
        let a = Node::new("Same");
        let b = Node::new("Same");
        assert_eq!(a, b);
    }

    #[test]
    fn test_node_ne() {
        let a = Node::new("A");
        let b = Node::new("B");
        assert_ne!(a, b);
    }

    #[test]
    fn test_node_display() {
        let n = Node::new("TestNode");
        let s = format!("{}", n);
        assert!(s.contains("TestNode"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCENE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod scene_tests {
    use super::*;

    #[test]
    fn test_scene_new() {
        let s = Scene::new();
        assert_eq!(s.root.name, "root");
    }

    #[test]
    fn test_scene_with_root() {
        let root = Node::new("CustomRoot");
        let s = Scene::with_root(root);
        assert_eq!(s.root.name, "CustomRoot");
    }

    #[test]
    fn test_scene_node_count_empty() {
        let s = Scene::new();
        assert_eq!(s.node_count(), 1); // Just root
    }

    #[test]
    fn test_scene_node_count_with_children() {
        let mut s = Scene::new();
        s.root.add_child(Node::new("A"));
        s.root.add_child(Node::new("B"));
        assert_eq!(s.node_count(), 3); // Root + A + B
    }

    #[test]
    fn test_scene_is_empty_true() {
        let s = Scene::new();
        assert!(s.is_empty());
    }

    #[test]
    fn test_scene_is_empty_false() {
        let mut s = Scene::new();
        s.root.add_child(Node::new("Child"));
        assert!(!s.is_empty());
    }

    #[test]
    fn test_scene_depth_empty() {
        let s = Scene::new();
        assert_eq!(s.depth(), 0);
    }

    #[test]
    fn test_scene_depth_with_children() {
        let mut s = Scene::new();
        let mut a = Node::new("A");
        a.add_child(Node::new("A1"));
        s.root.add_child(a);
        assert_eq!(s.depth(), 2);
    }

    #[test]
    fn test_scene_find_node_root() {
        let s = Scene::new();
        assert!(s.find_node("root").is_some());
    }

    #[test]
    fn test_scene_find_node_child() {
        let mut s = Scene::new();
        s.root.add_child(Node::new("Target"));
        assert!(s.find_node("Target").is_some());
    }

    #[test]
    fn test_scene_find_node_not_found() {
        let s = Scene::new();
        assert!(s.find_node("NotExists").is_none());
    }

    #[test]
    fn test_scene_traverse() {
        let mut s = Scene::new();
        s.root.add_child(Node::new("A"));
        s.root.add_child(Node::new("B"));

        let mut count = 0;
        s.traverse(&mut |_node, _mat| {
            count += 1;
        });
        assert_eq!(count, 3); // Root + A + B
    }

    #[test]
    fn test_scene_traverse_with_path() {
        let mut s = Scene::new();
        s.root.add_child(Node::new("Child"));

        let mut paths: Vec<Vec<String>> = Vec::new();
        s.traverse_with_path(&mut |_node, _mat, path| {
            paths.push(path.iter().map(|s| s.to_string()).collect());
        });
        assert!(!paths.is_empty());
        assert_eq!(paths[0][0], "root");
    }

    #[test]
    fn test_scene_clone() {
        let s = Scene::new();
        let cloned = s.clone();
        assert_eq!(s.root.name, cloned.root.name);
    }

    #[test]
    fn test_scene_display() {
        let s = Scene::new();
        let display = format!("{}", s);
        assert!(display.contains("Scene"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GRID COORD TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod grid_coord_tests {
    use super::*;

    #[test]
    fn test_grid_coord_new() {
        let c = GridCoord::new(1, 2, 3);
        assert_eq!(c.x, 1);
        assert_eq!(c.y, 2);
        assert_eq!(c.z, 3);
    }

    #[test]
    fn test_grid_coord_from_world_pos_origin() {
        let c = GridCoord::from_world_pos(Vec3::ZERO, 100.0);
        assert_eq!(c.x, 0);
        assert_eq!(c.y, 0);
        assert_eq!(c.z, 0);
    }

    #[test]
    fn test_grid_coord_from_world_pos_positive() {
        let c = GridCoord::from_world_pos(Vec3::new(150.0, 250.0, 350.0), 100.0);
        assert_eq!(c.x, 1);
        assert_eq!(c.y, 2);
        assert_eq!(c.z, 3);
    }

    #[test]
    fn test_grid_coord_from_world_pos_negative() {
        let c = GridCoord::from_world_pos(Vec3::new(-50.0, -150.0, -250.0), 100.0);
        assert_eq!(c.x, -1);
        assert_eq!(c.y, -2);
        assert_eq!(c.z, -3);
    }

    #[test]
    fn test_grid_coord_to_world_center() {
        let c = GridCoord::new(0, 0, 0);
        let center = c.to_world_center(100.0);
        assert!((center.x - 50.0).abs() < 0.001);
        assert!((center.y - 50.0).abs() < 0.001);
        assert!((center.z - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_grid_coord_neighbors_3d_count() {
        let c = GridCoord::new(0, 0, 0);
        let neighbors = c.neighbors_3d();
        assert_eq!(neighbors.len(), 26);
    }

    #[test]
    fn test_grid_coord_neighbors_2d_count() {
        let c = GridCoord::new(0, 0, 0);
        let neighbors = c.neighbors_2d();
        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn test_grid_coord_manhattan_distance_zero() {
        let a = GridCoord::new(5, 5, 5);
        assert_eq!(a.manhattan_distance(a), 0);
    }

    #[test]
    fn test_grid_coord_manhattan_distance() {
        let a = GridCoord::new(0, 0, 0);
        let b = GridCoord::new(1, 2, 3);
        assert_eq!(a.manhattan_distance(b), 6);
    }

    #[test]
    fn test_grid_coord_eq() {
        let a = GridCoord::new(1, 2, 3);
        let b = GridCoord::new(1, 2, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn test_grid_coord_ne() {
        let a = GridCoord::new(1, 2, 3);
        let b = GridCoord::new(3, 2, 1);
        assert_ne!(a, b);
    }

    #[test]
    fn test_grid_coord_clone() {
        let c = GridCoord::new(5, 6, 7);
        let cloned = c;
        assert_eq!(c, cloned);
    }

    #[test]
    fn test_grid_coord_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(GridCoord::new(1, 2, 3));
        assert!(set.contains(&GridCoord::new(1, 2, 3)));
        assert!(!set.contains(&GridCoord::new(0, 0, 0)));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AABB TESTS (world_partition version)
// ═══════════════════════════════════════════════════════════════════════════

mod wp_aabb_tests {
    use super::*;
    use astraweave_scene::world_partition::AABB;

    #[test]
    fn test_aabb_new() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_aabb_from_center_half_extents() {
        let aabb = AABB::from_center_half_extents(Vec3::new(5.0, 5.0, 5.0), Vec3::ONE);
        assert_eq!(aabb.min, Vec3::new(4.0, 4.0, 4.0));
        assert_eq!(aabb.max, Vec3::new(6.0, 6.0, 6.0));
    }

    #[test]
    fn test_aabb_center() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));
        assert_eq!(aabb.center(), Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_aabb_half_extents() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));
        assert_eq!(aabb.half_extents(), Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_aabb_contains_point_inside() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));
        assert!(aabb.contains_point(Vec3::new(5.0, 5.0, 5.0)));
    }

    #[test]
    fn test_aabb_contains_point_outside() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(!aabb.contains_point(Vec3::new(5.0, 5.0, 5.0)));
    }

    #[test]
    fn test_aabb_intersects_true() {
        let a = AABB::new(Vec3::ZERO, Vec3::new(5.0, 5.0, 5.0));
        let b = AABB::new(Vec3::new(3.0, 3.0, 3.0), Vec3::new(10.0, 10.0, 10.0));
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_aabb_intersects_false() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(11.0, 11.0, 11.0));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_aabb_overlapping_cells() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(50.0, 50.0, 50.0));
        let cells = aabb.overlapping_cells(100.0); // All within cell (0,0,0)
        assert!(!cells.is_empty());
    }

    #[test]
    fn test_aabb_overlapping_cells_multiple() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::new(150.0, 50.0, 50.0));
        let cells = aabb.overlapping_cells(100.0);
        // Should span x=0 and x=1
        assert!(cells.len() >= 2);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STREAMING TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod streaming_tests {
    use super::*;

    #[test]
    fn test_streaming_event_cell_load_started() {
        let coord = GridCoord::new(1, 2, 3);
        let event = StreamingEvent::CellLoadStarted(coord);
        let debug = format!("{:?}", event);
        assert!(debug.contains("CellLoadStarted"));
    }

    #[test]
    fn test_streaming_event_cell_loaded() {
        let coord = GridCoord::new(0, 0, 0);
        let event = StreamingEvent::CellLoaded(coord);
        let debug = format!("{:?}", event);
        assert!(debug.contains("CellLoaded"));
    }

    #[test]
    fn test_streaming_event_cell_load_failed() {
        let coord = GridCoord::new(0, 0, 0);
        let event = StreamingEvent::CellLoadFailed(coord, "error".to_string());
        let debug = format!("{:?}", event);
        assert!(debug.contains("CellLoadFailed"));
    }

    #[test]
    fn test_streaming_event_cell_unload_started() {
        let coord = GridCoord::new(0, 0, 0);
        let event = StreamingEvent::CellUnloadStarted(coord);
        let debug = format!("{:?}", event);
        assert!(debug.contains("CellUnloadStarted"));
    }

    #[test]
    fn test_streaming_event_cell_unloaded() {
        let coord = GridCoord::new(0, 0, 0);
        let event = StreamingEvent::CellUnloaded(coord);
        let debug = format!("{:?}", event);
        assert!(debug.contains("CellUnloaded"));
    }

    #[test]
    fn test_streaming_event_clone() {
        let event = StreamingEvent::CellLoaded(GridCoord::new(1, 2, 3));
        let cloned = event.clone();
        let debug_a = format!("{:?}", event);
        let debug_b = format!("{:?}", cloned);
        assert_eq!(debug_a, debug_b);
    }

    #[test]
    fn test_streaming_config_default() {
        let config = StreamingConfig::default();
        assert_eq!(config.max_active_cells, 25);
        assert_eq!(config.lru_cache_size, 5);
        assert_eq!(config.streaming_radius, 500.0);
        assert_eq!(config.max_concurrent_loads, 4);
    }

    #[test]
    fn test_streaming_config_clone() {
        let config = StreamingConfig::default();
        let cloned = config.clone();
        assert_eq!(config.max_active_cells, cloned.max_active_cells);
    }

    #[test]
    fn test_streaming_metrics_default() {
        let metrics = StreamingMetrics::default();
        assert_eq!(metrics.active_cells, 0);
        assert_eq!(metrics.loading_cells, 0);
        assert_eq!(metrics.loaded_cells, 0);
        assert_eq!(metrics.cached_cells, 0);
        assert_eq!(metrics.memory_usage_bytes, 0);
        assert_eq!(metrics.total_loads, 0);
        assert_eq!(metrics.total_unloads, 0);
        assert_eq!(metrics.failed_loads, 0);
    }

    #[test]
    fn test_streaming_metrics_clone() {
        let metrics = StreamingMetrics {
            active_cells: 5,
            loading_cells: 2,
            ..Default::default()
        };
        let cloned = metrics.clone();
        assert_eq!(metrics.active_cells, cloned.active_cells);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CELL STATE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod cell_state_tests {
    use super::*;

    #[test]
    fn test_cell_state_unloaded() {
        let state = CellState::Unloaded;
        assert_eq!(state, CellState::Unloaded);
    }

    #[test]
    fn test_cell_state_loading() {
        let state = CellState::Loading;
        assert_eq!(state, CellState::Loading);
    }

    #[test]
    fn test_cell_state_loaded() {
        let state = CellState::Loaded;
        assert_eq!(state, CellState::Loaded);
    }

    #[test]
    fn test_cell_state_ne() {
        assert_ne!(CellState::Unloaded, CellState::Loading);
        assert_ne!(CellState::Loading, CellState::Loaded);
        assert_ne!(CellState::Loaded, CellState::Unloaded);
    }

    #[test]
    fn test_cell_state_clone() {
        let state = CellState::Loaded;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_cell_state_debug() {
        let state = CellState::Loading;
        let debug = format!("{:?}", state);
        assert!(debug.contains("Loading"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LRU CACHE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod lru_cache_tests {
    use super::*;

    #[test]
    fn test_lru_cache_new() {
        let cache = LRUCache::new(10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_lru_cache_touch_contains() {
        let mut cache = LRUCache::new(10);
        cache.touch(GridCoord::new(1, 2, 3));
        assert!(cache.contains(GridCoord::new(1, 2, 3)));
        assert!(!cache.contains(GridCoord::new(0, 0, 0)));
    }

    #[test]
    fn test_lru_cache_len() {
        let mut cache = LRUCache::new(10);
        cache.touch(GridCoord::new(0, 0, 0));
        cache.touch(GridCoord::new(1, 0, 0));
        cache.touch(GridCoord::new(2, 0, 0));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_lru_cache_is_empty() {
        let cache = LRUCache::new(5);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_lru_cache_not_empty() {
        let mut cache = LRUCache::new(5);
        cache.touch(GridCoord::new(0, 0, 0));
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_lru_cache_remove() {
        let mut cache = LRUCache::new(10);
        cache.touch(GridCoord::new(1, 2, 3));
        cache.remove(GridCoord::new(1, 2, 3));
        assert!(!cache.contains(GridCoord::new(1, 2, 3)));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LRUCache::new(3);
        cache.touch(GridCoord::new(0, 0, 0));
        cache.touch(GridCoord::new(1, 0, 0));
        cache.touch(GridCoord::new(2, 0, 0));
        cache.touch(GridCoord::new(3, 0, 0)); // Should evict (0,0,0)
        assert!(!cache.contains(GridCoord::new(0, 0, 0)));
        assert!(cache.contains(GridCoord::new(3, 0, 0)));
    }

    #[test]
    fn test_lru_cache_lru() {
        let mut cache = LRUCache::new(3);
        cache.touch(GridCoord::new(0, 0, 0));
        cache.touch(GridCoord::new(1, 0, 0));
        let lru = cache.lru();
        assert_eq!(lru, Some(GridCoord::new(0, 0, 0)));
    }

    #[test]
    fn test_lru_cache_lru_empty() {
        let cache = LRUCache::new(3);
        assert!(cache.lru().is_none());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WORLD PARTITION TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod world_partition_tests {
    use super::*;

    #[test]
    fn test_grid_config_default() {
        let config = GridConfig::default();
        assert!(config.cell_size > 0.0);
    }

    #[test]
    fn test_grid_config_cell_size() {
        let config = GridConfig {
            cell_size: 200.0,
            ..Default::default()
        };
        assert_eq!(config.cell_size, 200.0);
    }

    #[test]
    fn test_grid_config_world_bounds() {
        let config = GridConfig::default();
        assert_eq!(config.world_bounds.0, -5000.0);
        assert_eq!(config.world_bounds.1, 5000.0);
    }

    #[test]
    fn test_world_partition_new() {
        let config = GridConfig {
            cell_size: 100.0,
            ..Default::default()
        };
        let wp = WorldPartition::new(config);
        assert!(wp.cells.is_empty());
    }

    #[test]
    fn test_world_partition_config_cell_size() {
        let config = GridConfig {
            cell_size: 200.0,
            ..Default::default()
        };
        let wp = WorldPartition::new(config);
        assert_eq!(wp.config.cell_size, 200.0);
    }

    #[test]
    fn test_world_partition_get_cell_none() {
        let config = GridConfig::default();
        let wp = WorldPartition::new(config);
        assert!(wp.get_cell(GridCoord::new(0, 0, 0)).is_none());
    }

    #[test]
    fn test_world_partition_get_or_create_cell() {
        let config = GridConfig::default();
        let mut wp = WorldPartition::new(config);
        let _ = wp.get_or_create_cell(GridCoord::new(1, 2, 3));
        assert!(wp.get_cell(GridCoord::new(1, 2, 3)).is_some());
    }

    #[test]
    fn test_world_partition_cells_in_radius() {
        let config = GridConfig {
            cell_size: 100.0,
            ..Default::default()
        };
        let wp = WorldPartition::new(config);
        let cells = wp.cells_in_radius(Vec3::ZERO, 150.0);
        assert!(!cells.is_empty());
    }

    #[test]
    fn test_world_partition_memory_usage_empty() {
        let config = GridConfig::default();
        let wp = WorldPartition::new(config);
        assert_eq!(wp.memory_usage_estimate(), 0);
    }

    #[test]
    fn test_world_partition_memory_usage_with_cells() {
        let config = GridConfig::default();
        let mut wp = WorldPartition::new(config);
        let _ = wp.get_or_create_cell(GridCoord::new(0, 0, 0));
        assert!(wp.memory_usage_estimate() > 0);
    }

    #[test]
    fn test_world_partition_clone() {
        let config = GridConfig::default();
        let wp = WorldPartition::new(config);
        let cloned = wp.clone();
        assert_eq!(wp.config.cell_size, cloned.config.cell_size);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FRUSTUM TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod frustum_tests {
    use super::*;
    use astraweave_scene::world_partition::Frustum;

    #[test]
    fn test_frustum_from_view_projection() {
        let view_proj = Mat4::perspective_lh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 1000.0);
        let frustum = Frustum::from_view_projection(view_proj);
        // Should have 6 planes
        assert_eq!(frustum.planes.len(), 6);
    }

    #[test]
    fn test_frustum_planes_normalized() {
        let view_proj = Mat4::perspective_lh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);
        for plane in &frustum.planes {
            let normal_len = Vec3::new(plane.x, plane.y, plane.z).length();
            assert!((normal_len - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_frustum_intersects_aabb() {
        let view_proj = Mat4::IDENTITY; // Degenerate case but tests code path
        let frustum = Frustum::from_view_projection(view_proj);
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        // Just ensure it doesn't panic
        let _ = frustum.intersects_aabb(&aabb);
    }

    #[test]
    fn test_frustum_cells_in_frustum() {
        let view_proj = Mat4::perspective_lh(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 1000.0);
        let frustum = Frustum::from_view_projection(view_proj);
        let cells = frustum.cells_in_frustum(Vec3::ZERO, 100.0, 200.0);
        assert!(!cells.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CELL ENTITY BLUEPRINT TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod cell_entity_blueprint_tests {
    use super::*;

    #[test]
    fn test_cell_entity_blueprint_fields() {
        let blueprint = CellEntityBlueprint {
            name: Some("Entity1".to_string()),
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            components: vec![],
        };
        assert_eq!(blueprint.position, [1.0, 2.0, 3.0]);
        assert_eq!(blueprint.name, Some("Entity1".to_string()));
    }

    #[test]
    fn test_cell_entity_blueprint_default_rotation() {
        let blueprint = CellEntityBlueprint {
            name: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // Identity quaternion
            scale: [1.0, 1.0, 1.0],
            components: vec![],
        };
        assert_eq!(blueprint.rotation[3], 1.0);
    }

    #[test]
    fn test_cell_entity_blueprint_no_name() {
        let blueprint = CellEntityBlueprint {
            name: None,
            position: [5.0, 6.0, 7.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [2.0, 2.0, 2.0],
            components: vec![],
        };
        assert!(blueprint.name.is_none());
    }

    #[test]
    fn test_cell_entity_blueprint_scale() {
        let blueprint = CellEntityBlueprint {
            name: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [3.0, 4.0, 5.0],
            components: vec![],
        };
        assert_eq!(blueprint.scale, [3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_cell_entity_blueprint_clone() {
        let blueprint = CellEntityBlueprint {
            name: Some("Cloned".to_string()),
            position: [1.0, 1.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            components: vec![],
        };
        let cloned = blueprint.clone();
        assert_eq!(blueprint.name, cloned.name);
        assert_eq!(blueprint.position, cloned.position);
    }

    #[test]
    fn test_cell_entity_blueprint_debug() {
        let blueprint = CellEntityBlueprint {
            name: Some("Test".to_string()),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            components: vec![],
        };
        let debug = format!("{:?}", blueprint);
        assert!(debug.contains("CellEntityBlueprint"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GPU RESOURCE MANAGER TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod gpu_resource_tests {
    use super::*;
    use astraweave_scene::gpu_resource_manager::*;

    #[test]
    fn test_cell_gpu_resources_new() {
        let coord = GridCoord::new(1, 2, 3);
        let resources = CellGpuResources::new(coord);
        assert_eq!(resources.coord, coord);
        assert!(resources.vertex_buffers.is_empty());
        assert!(resources.index_buffers.is_empty());
        assert!(resources.textures.is_empty());
        assert_eq!(resources.memory_usage, 0);
    }

    #[test]
    fn test_cell_gpu_resources_coord() {
        let coord = GridCoord::new(5, 6, 7);
        let resources = CellGpuResources::new(coord);
        assert_eq!(resources.coord.x, 5);
        assert_eq!(resources.coord.y, 6);
        assert_eq!(resources.coord.z, 7);
    }
}
