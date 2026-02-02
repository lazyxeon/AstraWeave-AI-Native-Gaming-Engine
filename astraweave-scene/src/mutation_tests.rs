//! Mutation-killing tests for astraweave-scene.
//! These tests are designed to catch subtle mutations like boundary condition changes,
//! arithmetic operator substitutions, and conditional logic inversions.

use crate::*;
use glam::{Mat4, Quat, Vec3};
use std::f32::consts::PI;

// ============================================================================
// Transform Boundary Tests - Catch mutations in transform operations
// ============================================================================

#[cfg(test)]
mod transform_mutation_tests {
    use super::*;

    #[test]
    fn test_transform_new_preserves_exact_values() {
        let translation = Vec3::new(1.23456, -7.89012, 3.45678);
        let rotation = Quat::from_xyzw(0.1, 0.2, 0.3, 0.9).normalize();
        let scale = Vec3::new(0.5, 2.0, 1.5);
        
        let t = Transform::new(translation, rotation, scale);
        
        // Exact value preservation - catches field swaps
        assert_eq!(t.translation.x, 1.23456);
        assert_eq!(t.translation.y, -7.89012);
        assert_eq!(t.translation.z, 3.45678);
        assert_eq!(t.scale.x, 0.5);
        assert_eq!(t.scale.y, 2.0);
        assert_eq!(t.scale.z, 1.5);
    }

    #[test]
    fn test_identity_returns_correct_defaults() {
        let t = Transform::identity();
        
        // Ensure identity actually returns identity values
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
        
        // Explicit checks for each component
        assert_eq!(t.translation.x, 0.0);
        assert_eq!(t.translation.y, 0.0);
        assert_eq!(t.translation.z, 0.0);
        assert_eq!(t.scale.x, 1.0);
        assert_eq!(t.scale.y, 1.0);
        assert_eq!(t.scale.z, 1.0);
    }

    #[test]
    fn test_from_translation_only_sets_translation() {
        let t = Transform::from_translation(Vec3::new(5.0, 10.0, 15.0));
        
        // Translation should be set
        assert_eq!(t.translation, Vec3::new(5.0, 10.0, 15.0));
        
        // Rotation and scale should be default
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_from_rotation_only_sets_rotation() {
        let rotation = Quat::from_rotation_y(PI / 4.0);
        let t = Transform::from_rotation(rotation);
        
        // Rotation should be set
        assert!((t.rotation.dot(rotation) - 1.0).abs() < 0.0001);
        
        // Translation and scale should be default
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_from_scale_creates_uniform_scale() {
        let t = Transform::from_scale(3.0);
        
        // Scale should be uniform
        assert_eq!(t.scale.x, 3.0);
        assert_eq!(t.scale.y, 3.0);
        assert_eq!(t.scale.z, 3.0);
        
        // Translation and rotation should be default
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
    }

    #[test]
    fn test_from_scale_vec_creates_non_uniform_scale() {
        let t = Transform::from_scale_vec(Vec3::new(1.0, 2.0, 3.0));
        
        // Each axis should be different
        assert_eq!(t.scale.x, 1.0);
        assert_eq!(t.scale.y, 2.0);
        assert_eq!(t.scale.z, 3.0);
        
        // Not uniform
        assert!(t.scale.x != t.scale.y);
        assert!(t.scale.y != t.scale.z);
    }

    #[test]
    fn test_is_identity_true_only_for_identity() {
        let identity = Transform::identity();
        assert!(identity.is_identity());
        
        // Any deviation should return false
        let mut t = Transform::identity();
        t.translation.x = 0.001;
        assert!(!t.is_identity());
        
        let mut t = Transform::identity();
        t.scale.y = 1.001;
        assert!(!t.is_identity());
    }

    #[test]
    fn test_is_uniform_scale_epsilon_handling() {
        // Exactly uniform
        let t = Transform::from_scale(2.0);
        assert!(t.is_uniform_scale());
        
        // Non-uniform
        let t = Transform::from_scale_vec(Vec3::new(2.0, 2.1, 2.0));
        assert!(!t.is_uniform_scale());
        
        // Very close but not equal (within epsilon)
        let t = Transform::from_scale_vec(Vec3::new(2.0, 2.0 + f32::EPSILON / 2.0, 2.0));
        assert!(t.is_uniform_scale());
    }

    #[test]
    fn test_uniform_scale_average_calculation() {
        let t = Transform::from_scale_vec(Vec3::new(1.0, 2.0, 3.0));
        let avg = t.uniform_scale();
        
        // Average of 1, 2, 3 = 2
        assert!((avg - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_forward_direction() {
        // Identity rotation: forward is -Z
        let t = Transform::identity();
        let fwd = t.forward();
        assert!((fwd - Vec3::NEG_Z).length() < 0.0001);
        
        // 90° rotation around Y: forward becomes -X
        let t = Transform::from_rotation(Quat::from_rotation_y(PI / 2.0));
        let fwd = t.forward();
        assert!((fwd - Vec3::NEG_X).length() < 0.001);
    }

    #[test]
    fn test_right_direction() {
        // Identity rotation: right is +X
        let t = Transform::identity();
        let right = t.right();
        assert!((right - Vec3::X).length() < 0.0001);
    }

    #[test]
    fn test_up_direction() {
        // Identity rotation: up is +Y
        let t = Transform::identity();
        let up = t.up();
        assert!((up - Vec3::Y).length() < 0.0001);
    }

    #[test]
    fn test_inverse_reverses_transform() {
        let t = Transform::new(
            Vec3::new(10.0, 20.0, 30.0),
            Quat::from_rotation_z(PI / 6.0),
            Vec3::new(2.0, 2.0, 2.0),
        );
        
        let inv = t.inverse();
        
        // Applying transform then inverse should return to original point
        let point = Vec3::new(1.0, 1.0, 1.0);
        let transformed = t.transform_point(point);
        let back = inv.transform_point(transformed);
        
        assert!((back - point).length() < 0.001);
    }

    #[test]
    fn test_transform_point_applies_scale_rotation_translation() {
        let t = Transform {
            translation: Vec3::new(10.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(2.0, 2.0, 2.0),
        };
        
        // Point (1, 0, 0) scaled by 2 = (2, 0, 0), then translated by (10, 0, 0) = (12, 0, 0)
        let result = t.transform_point(Vec3::new(1.0, 0.0, 0.0));
        assert!((result - Vec3::new(12.0, 0.0, 0.0)).length() < 0.0001);
    }

    #[test]
    fn test_transform_direction_ignores_translation() {
        let t = Transform {
            translation: Vec3::new(100.0, 100.0, 100.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        
        // Direction should not be affected by translation
        let result = t.transform_direction(Vec3::X);
        assert!((result - Vec3::X).length() < 0.0001);
    }

    #[test]
    fn test_lerp_at_boundaries() {
        let a = Transform::from_translation(Vec3::ZERO);
        let b = Transform::from_translation(Vec3::new(10.0, 10.0, 10.0));
        
        // t=0 should return a
        let result = a.lerp(&b, 0.0);
        assert!((result.translation - a.translation).length() < 0.0001);
        
        // t=1 should return b
        let result = a.lerp(&b, 1.0);
        assert!((result.translation - b.translation).length() < 0.0001);
        
        // t=0.5 should be midpoint
        let result = a.lerp(&b, 0.5);
        assert!((result.translation - Vec3::new(5.0, 5.0, 5.0)).length() < 0.0001);
    }
}

// ============================================================================
// Node Mutation Tests - Catch mutations in tree operations
// ============================================================================

#[cfg(test)]
mod node_mutation_tests {
    use super::*;

    #[test]
    fn test_node_new_creates_correct_name() {
        let node = Node::new("test_name");
        assert_eq!(node.name, "test_name");
        assert!(node.name != "");
        assert!(node.name != "other_name");
    }

    #[test]
    fn test_node_with_transform_sets_both() {
        let transform = Transform::from_translation(Vec3::new(5.0, 5.0, 5.0));
        let node = Node::with_transform("transformed", transform);
        
        assert_eq!(node.name, "transformed");
        assert_eq!(node.transform.translation, Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_has_children_boundary() {
        let mut node = Node::new("parent");
        
        // No children
        assert!(!node.has_children());
        
        // One child
        node.add_child(Node::new("child"));
        assert!(node.has_children());
        
        // Multiple children
        node.add_child(Node::new("child2"));
        assert!(node.has_children());
    }

    #[test]
    fn test_child_count_accuracy() {
        let mut node = Node::new("parent");
        
        assert_eq!(node.child_count(), 0);
        
        node.add_child(Node::new("c1"));
        assert_eq!(node.child_count(), 1);
        
        node.add_child(Node::new("c2"));
        assert_eq!(node.child_count(), 2);
        
        node.add_child(Node::new("c3"));
        assert_eq!(node.child_count(), 3);
    }

    #[test]
    fn test_descendant_count_recursive() {
        let mut root = Node::new("root");
        
        // No descendants
        assert_eq!(root.descendant_count(), 0);
        
        // Add one child
        root.add_child(Node::new("c1"));
        assert_eq!(root.descendant_count(), 1);
        
        // Add grandchild
        root.children[0].add_child(Node::new("gc1"));
        assert_eq!(root.descendant_count(), 2);
        
        // Add another child with grandchildren
        let mut c2 = Node::new("c2");
        c2.add_child(Node::new("gc2"));
        c2.add_child(Node::new("gc3"));
        root.add_child(c2);
        assert_eq!(root.descendant_count(), 5);
    }

    #[test]
    fn test_is_leaf_boundary() {
        let mut node = Node::new("node");
        
        // Initially a leaf
        assert!(node.is_leaf());
        
        // After adding child, not a leaf
        node.add_child(Node::new("child"));
        assert!(!node.is_leaf());
    }

    #[test]
    fn test_find_child_exact_match() {
        let mut parent = Node::new("parent");
        parent.add_child(Node::new("alpha"));
        parent.add_child(Node::new("beta"));
        parent.add_child(Node::new("gamma"));
        
        // Exact match
        assert!(parent.find_child("alpha").is_some());
        assert!(parent.find_child("beta").is_some());
        
        // No match
        assert!(parent.find_child("delta").is_none());
        assert!(parent.find_child("Alpha").is_none()); // Case sensitive
        assert!(parent.find_child("").is_none());
    }

    #[test]
    fn test_find_child_mut_allows_modification() {
        let mut parent = Node::new("parent");
        parent.add_child(Node::new("child"));
        
        if let Some(child) = parent.find_child_mut("child") {
            child.transform.translation = Vec3::new(1.0, 2.0, 3.0);
        }
        
        // Verify modification
        let child = parent.find_child("child").unwrap();
        assert_eq!(child.transform.translation, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_find_descendant_recursive_search() {
        let mut root = Node::new("root");
        let mut c1 = Node::new("c1");
        c1.add_child(Node::new("gc1"));
        c1.children[0].add_child(Node::new("ggc1"));
        root.add_child(c1);
        
        // Find at each level
        assert!(root.find_descendant("c1").is_some());
        assert!(root.find_descendant("gc1").is_some());
        assert!(root.find_descendant("ggc1").is_some());
        
        // Not found
        assert!(root.find_descendant("nonexistent").is_none());
    }

    #[test]
    fn test_depth_calculation() {
        let mut root = Node::new("root");
        
        // Leaf node has depth 0
        assert_eq!(root.depth(), 0);
        
        // Add one level
        root.add_child(Node::new("c1"));
        assert_eq!(root.depth(), 1);
        
        // Add second level
        root.children[0].add_child(Node::new("gc1"));
        assert_eq!(root.depth(), 2);
        
        // Add third level
        root.children[0].children[0].add_child(Node::new("ggc1"));
        assert_eq!(root.depth(), 3);
    }

    #[test]
    fn test_node_display_format() {
        let leaf = Node::new("leaf");
        assert!(format!("{}", leaf).contains("leaf"));
        
        let mut parent = Node::new("parent");
        parent.add_child(Node::new("c1"));
        parent.add_child(Node::new("c2"));
        let display = format!("{}", parent);
        assert!(display.contains("parent"));
        assert!(display.contains("2 children"));
    }
}

// ============================================================================
// Scene Mutation Tests - Catch mutations in scene graph operations
// ============================================================================

#[cfg(test)]
mod scene_mutation_tests {
    use super::*;

    #[test]
    fn test_scene_new_creates_root() {
        let scene = Scene::new();
        assert_eq!(scene.root.name, "root");
        assert!(scene.root.children.is_empty());
    }

    #[test]
    fn test_scene_with_root_uses_custom_root() {
        let custom_root = Node::new("custom_root");
        let scene = Scene::with_root(custom_root);
        
        assert_eq!(scene.root.name, "custom_root");
    }

    #[test]
    fn test_node_count_includes_root() {
        let scene = Scene::new();
        
        // Just root
        assert_eq!(scene.node_count(), 1);
    }

    #[test]
    fn test_node_count_recursive() {
        let mut scene = Scene::new();
        scene.root.add_child(Node::new("c1"));
        scene.root.add_child(Node::new("c2"));
        scene.root.children[0].add_child(Node::new("gc1"));
        
        // root + 2 children + 1 grandchild = 4
        assert_eq!(scene.node_count(), 4);
    }

    #[test]
    fn test_is_empty_boundary() {
        let scene = Scene::new();
        assert!(scene.is_empty());
        
        let mut scene = Scene::new();
        scene.root.add_child(Node::new("child"));
        assert!(!scene.is_empty());
    }

    #[test]
    fn test_scene_depth() {
        let mut scene = Scene::new();
        assert_eq!(scene.depth(), 0);
        
        scene.root.add_child(Node::new("c1"));
        assert_eq!(scene.depth(), 1);
        
        scene.root.children[0].add_child(Node::new("gc1"));
        assert_eq!(scene.depth(), 2);
    }

    #[test]
    fn test_find_node_finds_root() {
        let scene = Scene::new();
        
        let found = scene.find_node("root");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "root");
    }

    #[test]
    fn test_find_node_finds_descendants() {
        let mut scene = Scene::new();
        scene.root.add_child(Node::new("child"));
        scene.root.children[0].add_child(Node::new("grandchild"));
        
        assert!(scene.find_node("child").is_some());
        assert!(scene.find_node("grandchild").is_some());
        assert!(scene.find_node("nonexistent").is_none());
    }

    #[test]
    fn test_traverse_visits_all_nodes() {
        let mut scene = Scene::new();
        scene.root.add_child(Node::new("c1"));
        scene.root.add_child(Node::new("c2"));
        
        let mut visited = Vec::new();
        scene.traverse(&mut |node, _| {
            visited.push(node.name.clone());
        });
        
        assert_eq!(visited.len(), 3);
        assert!(visited.contains(&"root".to_string()));
        assert!(visited.contains(&"c1".to_string()));
        assert!(visited.contains(&"c2".to_string()));
    }

    #[test]
    fn test_traverse_computes_world_transforms() {
        let mut scene = Scene::new();
        
        // Root at origin
        // Child translated by (10, 0, 0)
        let mut child = Node::new("child");
        child.transform.translation = Vec3::new(10.0, 0.0, 0.0);
        scene.root.add_child(child);
        
        let mut child_world = Mat4::IDENTITY;
        scene.traverse(&mut |node, world| {
            if node.name == "child" {
                child_world = world;
            }
        });
        
        // Child's world position should be (10, 0, 0)
        let (_, _, translation) = child_world.to_scale_rotation_translation();
        assert!((translation - Vec3::new(10.0, 0.0, 0.0)).length() < 0.0001);
    }

    #[test]
    fn test_traverse_with_path_tracks_path() {
        let mut scene = Scene::new();
        let mut c1 = Node::new("c1");
        c1.add_child(Node::new("gc1"));
        scene.root.add_child(c1);
        
        let mut gc1_path = Vec::new();
        scene.traverse_with_path(&mut |node, _, path| {
            if node.name == "gc1" {
                gc1_path = path.iter().map(|s| s.to_string()).collect();
            }
        });
        
        assert_eq!(gc1_path, vec!["root", "c1", "gc1"]);
    }

    #[test]
    fn test_scene_display_format() {
        let mut scene = Scene::new();
        scene.root.add_child(Node::new("c1"));
        scene.root.add_child(Node::new("c2"));
        
        let display = format!("{}", scene);
        assert!(display.contains("Scene"));
        assert!(display.contains("3 nodes"));
        assert!(display.contains("depth=1"));
    }
}

// ============================================================================
// ECS Component Mutation Tests (when ecs feature is enabled)
// ============================================================================

#[cfg(all(test, feature = "ecs"))]
mod ecs_mutation_tests {
    use super::*;
    use crate::ecs::*;

    #[test]
    fn test_playback_state_is_playing() {
        assert!(PlaybackState::Playing.is_playing());
        assert!(!PlaybackState::Paused.is_playing());
        assert!(!PlaybackState::Stopped.is_playing());
    }

    #[test]
    fn test_playback_state_is_paused() {
        assert!(!PlaybackState::Playing.is_paused());
        assert!(PlaybackState::Paused.is_paused());
        assert!(!PlaybackState::Stopped.is_paused());
    }

    #[test]
    fn test_playback_state_is_stopped() {
        assert!(!PlaybackState::Playing.is_stopped());
        assert!(!PlaybackState::Paused.is_stopped());
        assert!(PlaybackState::Stopped.is_stopped());
    }

    #[test]
    fn test_playback_state_is_active() {
        assert!(PlaybackState::Playing.is_active());
        assert!(PlaybackState::Paused.is_active());
        assert!(!PlaybackState::Stopped.is_active());
    }

    #[test]
    fn test_playback_state_all() {
        let all = PlaybackState::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&PlaybackState::Playing));
        assert!(all.contains(&PlaybackState::Paused));
        assert!(all.contains(&PlaybackState::Stopped));
    }

    #[test]
    fn test_animator_default() {
        let anim = CAnimator::default();
        assert_eq!(anim.clip_index, 0);
        assert_eq!(anim.time, 0.0);
        assert_eq!(anim.speed, 1.0);
        assert_eq!(anim.state, PlaybackState::Stopped);
        assert!(anim.looping);
    }

    #[test]
    fn test_animator_new() {
        let anim = CAnimator::new(5);
        assert_eq!(anim.clip_index, 5);
        assert_eq!(anim.state, PlaybackState::Stopped);
    }

    #[test]
    fn test_animator_play_sets_state() {
        let mut anim = CAnimator::default();
        assert_eq!(anim.state, PlaybackState::Stopped);
        
        anim.play();
        assert_eq!(anim.state, PlaybackState::Playing);
    }

    #[test]
    fn test_animator_pause_only_when_playing() {
        let mut anim = CAnimator::default();
        
        // Pause from stopped does nothing
        anim.pause();
        assert_eq!(anim.state, PlaybackState::Stopped);
        
        // Pause from playing works
        anim.play();
        anim.pause();
        assert_eq!(anim.state, PlaybackState::Paused);
    }

    #[test]
    fn test_animator_stop_resets_time() {
        let mut anim = CAnimator::default();
        anim.play();
        anim.time = 5.0;
        
        anim.stop();
        assert_eq!(anim.state, PlaybackState::Stopped);
        assert_eq!(anim.time, 0.0);
    }

    #[test]
    fn test_animator_toggle_pause() {
        let mut anim = CAnimator::default();
        
        // From stopped -> playing
        anim.toggle_pause();
        assert_eq!(anim.state, PlaybackState::Playing);
        
        // From playing -> paused
        anim.toggle_pause();
        assert_eq!(anim.state, PlaybackState::Paused);
        
        // From paused -> playing
        anim.toggle_pause();
        assert_eq!(anim.state, PlaybackState::Playing);
    }

    #[test]
    fn test_animator_with_speed() {
        let anim = CAnimator::new(0).with_speed(2.0);
        assert_eq!(anim.speed, 2.0);
    }

    #[test]
    fn test_animator_with_looping() {
        let anim = CAnimator::new(0).with_looping(false);
        assert!(!anim.looping);
    }

    #[test]
    fn test_animator_reset_clears_time() {
        let mut anim = CAnimator::default();
        anim.time = 10.0;
        
        anim.reset();
        assert_eq!(anim.time, 0.0);
    }
}

// ============================================================================
// Transform Matrix Tests - Ensure matrix operations are correct
// ============================================================================

#[cfg(test)]
mod matrix_mutation_tests {
    use super::*;

    #[test]
    fn test_matrix_order_is_scale_rotation_translation() {
        let t = Transform {
            translation: Vec3::new(5.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(2.0, 2.0, 2.0),
        };
        
        let m = t.matrix();
        
        // Transform point (1, 0, 0):
        // Scale: (2, 0, 0)
        // Rotate: (2, 0, 0) (identity)
        // Translate: (7, 0, 0)
        let result = m.transform_point3(Vec3::new(1.0, 0.0, 0.0));
        assert!((result.x - 7.0).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_handles_negative_scale() {
        let t = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(-1.0, 1.0, 1.0),
        };
        
        let m = t.matrix();
        let result = m.transform_point3(Vec3::new(1.0, 0.0, 0.0));
        
        // Negative X scale should flip X
        assert!((result.x - (-1.0)).abs() < 0.0001);
    }

    #[test]
    fn test_matrix_zero_scale() {
        let t = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ZERO,
        };
        
        let m = t.matrix();
        let result = m.transform_point3(Vec3::new(100.0, 100.0, 100.0));
        
        // Zero scale collapses everything to origin
        assert!((result).length() < 0.0001);
    }
}

// ============================================================================
// Behavioral Correctness Tests - Scene Graph & Transform Invariants
// ============================================================================

#[cfg(test)]
mod behavioral_correctness_tests {
    use super::*;

    #[test]
    fn test_identity_transform_preserves_points() {
        // Behavioral: identity transform should not change any point
        let t = Transform::identity();
        let m = t.matrix();
        
        let test_points = [
            Vec3::ZERO,
            Vec3::ONE,
            Vec3::new(100.0, -50.0, 25.0),
            Vec3::new(-1.0, -1.0, -1.0),
        ];
        
        for point in test_points {
            let result = m.transform_point3(point);
            assert!((result - point).length() < 0.0001,
                "Identity should preserve point {:?}", point);
        }
    }

    #[test]
    fn test_transform_matrix_determinant_positive_for_valid_scale() {
        // Behavioral: matrix with positive scale should have positive determinant
        let t = Transform {
            translation: Vec3::new(10.0, 20.0, 30.0),
            rotation: Quat::from_rotation_y(PI / 4.0),
            scale: Vec3::new(1.0, 2.0, 3.0),
        };
        
        let m = t.matrix();
        let det = m.determinant();
        
        assert!(det > 0.0, "Determinant should be positive for positive scales: {}", det);
    }

    #[test]
    fn test_transform_matrix_determinant_equals_scale_product() {
        // Behavioral: determinant should equal product of scales (for no rotation shear)
        let t = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(2.0, 3.0, 4.0),
        };
        
        let m = t.matrix();
        let det = m.determinant();
        let expected = 2.0 * 3.0 * 4.0;
        
        assert!((det - expected).abs() < 0.001,
            "Determinant should equal scale product: expected {}, got {}", expected, det);
    }

    #[test]
    fn test_rotation_preserves_length() {
        // Behavioral: pure rotation should preserve vector length
        let t = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::from_rotation_y(PI / 3.0) * Quat::from_rotation_x(PI / 6.0),
            scale: Vec3::ONE,
        };
        
        let m = t.matrix();
        let original = Vec3::new(1.0, 2.0, 3.0);
        let original_len = original.length();
        
        let rotated = m.transform_vector3(original);
        let rotated_len = rotated.length();
        
        assert!((original_len - rotated_len).abs() < 0.0001,
            "Rotation should preserve length: {} vs {}", original_len, rotated_len);
    }

    #[test]
    fn test_translation_only_changes_position() {
        // Behavioral: translation should only affect point position, not direction
        let t = Transform::from_translation(Vec3::new(100.0, 200.0, 300.0));
        let m = t.matrix();
        
        // Direction vector should be unchanged
        let dir = Vec3::new(1.0, 0.0, 0.0);
        let transformed_dir = m.transform_vector3(dir);
        
        assert!((transformed_dir - dir).length() < 0.0001,
            "Translation should not affect directions");
    }

    #[test]
    fn test_uniform_scale_preserves_proportions() {
        // Behavioral: uniform scale should preserve proportions
        let t = Transform::from_scale(3.0);
        let m = t.matrix();
        
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scaled = m.transform_vector3(v);
        
        // Ratios should be preserved
        assert!((scaled.x / scaled.y - v.x / v.y).abs() < 0.0001, "X/Y ratio should be preserved");
        assert!((scaled.y / scaled.z - v.y / v.z).abs() < 0.0001, "Y/Z ratio should be preserved");
    }

    #[test]
    fn test_transform_composition_is_associative() {
        // Behavioral: (A * B) * C == A * (B * C) for matrices
        let a = Transform::from_translation(Vec3::X * 5.0).matrix();
        let b = Transform::from_rotation(Quat::from_rotation_y(PI / 4.0)).matrix();
        let c = Transform::from_scale(2.0).matrix();
        
        let left = (a * b) * c;
        let right = a * (b * c);
        
        // Compare all elements
        for i in 0..4 {
            for j in 0..4 {
                let l = left.col(i)[j];
                let r = right.col(i)[j];
                assert!((l - r).abs() < 0.0001,
                    "Matrix multiplication should be associative at ({},{}): {} vs {}", i, j, l, r);
            }
        }
    }

    #[test]
    fn test_scale_by_one_is_identity() {
        // Behavioral: scaling by 1 should be equivalent to identity
        let t = Transform::from_scale(1.0);
        let identity = Transform::identity();
        
        let point = Vec3::new(7.0, 8.0, 9.0);
        let result_scaled = t.matrix().transform_point3(point);
        let result_identity = identity.matrix().transform_point3(point);
        
        assert!((result_scaled - result_identity).length() < 0.0001,
            "Scale by 1 should equal identity transform");
    }
}
