//! Mutation-resistant tests for Interaction system (GizmoMeasurement, GizmoOperationKind, etc.)
//!
//! These tests target boundary conditions, comparison operators, and boolean return paths
//! to achieve high mutation testing kill rates (90%+).

use aw_editor_lib::interaction::{
    GizmoCommitMetadata, GizmoCancelMetadata, GizmoMeasurement, GizmoOperationKind,
};
use aw_editor_lib::gizmo::state::TransformSnapshot;
use astraweave_core::IVec2;
use glam::{Quat, Vec3};

/// Helper to create IVec2 at origin (astraweave_core::IVec2 has no ZERO const)
fn ivec2_zero() -> IVec2 {
    IVec2 { x: 0, y: 0 }
}

// ============================================================================
// GIZMO OPERATION KIND TESTS
// ============================================================================

mod gizmo_operation_kind_tests {
    use super::*;

    // Test all() returns exactly 3 elements
    #[test]
    fn test_all_count() {
        assert_eq!(GizmoOperationKind::all().len(), 3);
    }

    #[test]
    fn test_all_contains_translate() {
        assert!(GizmoOperationKind::all().contains(&GizmoOperationKind::Translate));
    }

    #[test]
    fn test_all_contains_rotate() {
        assert!(GizmoOperationKind::all().contains(&GizmoOperationKind::Rotate));
    }

    #[test]
    fn test_all_contains_scale() {
        assert!(GizmoOperationKind::all().contains(&GizmoOperationKind::Scale));
    }

    // Test name()
    #[test]
    fn test_translate_name() {
        assert_eq!(GizmoOperationKind::Translate.name(), "Translate");
    }

    #[test]
    fn test_rotate_name() {
        assert_eq!(GizmoOperationKind::Rotate.name(), "Rotate");
    }

    #[test]
    fn test_scale_name() {
        assert_eq!(GizmoOperationKind::Scale.name(), "Scale");
    }

    // Test icon()
    #[test]
    fn test_translate_icon() {
        assert_eq!(GizmoOperationKind::Translate.icon(), "↔");
    }

    #[test]
    fn test_rotate_icon() {
        assert_eq!(GizmoOperationKind::Rotate.icon(), "⟳");
    }

    #[test]
    fn test_scale_icon() {
        assert_eq!(GizmoOperationKind::Scale.icon(), "⤢");
    }

    // Test shortcut()
    #[test]
    fn test_translate_shortcut() {
        assert_eq!(GizmoOperationKind::Translate.shortcut(), "G");
    }

    #[test]
    fn test_rotate_shortcut() {
        assert_eq!(GizmoOperationKind::Rotate.shortcut(), "R");
    }

    #[test]
    fn test_scale_shortcut() {
        assert_eq!(GizmoOperationKind::Scale.shortcut(), "S");
    }

    // Test Display trait
    #[test]
    fn test_display_matches_name() {
        for kind in GizmoOperationKind::all() {
            assert_eq!(format!("{}", kind), kind.name());
        }
    }

    // Test names are unique
    #[test]
    fn test_names_unique() {
        let kinds = GizmoOperationKind::all();
        let names: Vec<_> = kinds.iter().map(|k| k.name()).collect();
        for (i, name) in names.iter().enumerate() {
            for (j, other) in names.iter().enumerate() {
                if i != j {
                    assert_ne!(name, other);
                }
            }
        }
    }

    // Test icons are unique
    #[test]
    fn test_icons_unique() {
        let kinds = GizmoOperationKind::all();
        let icons: Vec<_> = kinds.iter().map(|k| k.icon()).collect();
        for (i, icon) in icons.iter().enumerate() {
            for (j, other) in icons.iter().enumerate() {
                if i != j {
                    assert_ne!(icon, other);
                }
            }
        }
    }

    // Test shortcuts are unique
    #[test]
    fn test_shortcuts_unique() {
        let kinds = GizmoOperationKind::all();
        let shortcuts: Vec<_> = kinds.iter().map(|k| k.shortcut()).collect();
        for (i, shortcut) in shortcuts.iter().enumerate() {
            for (j, other) in shortcuts.iter().enumerate() {
                if i != j {
                    assert_ne!(shortcut, other);
                }
            }
        }
    }
}

// ============================================================================
// GIZMO MEASUREMENT TESTS
// ============================================================================

mod gizmo_measurement_tests {
    use super::*;

    // Test kind()
    #[test]
    fn test_translate_kind() {
        let m = GizmoMeasurement::Translate {
            from: ivec2_zero(),
            to: IVec2 { x: 1, y: 0 },
        };
        assert_eq!(m.kind(), GizmoOperationKind::Translate);
    }

    #[test]
    fn test_rotate_kind() {
        let m = GizmoMeasurement::Rotate {
            from: (0.0, 0.0, 0.0),
            to: (1.0, 0.0, 0.0),
        };
        assert_eq!(m.kind(), GizmoOperationKind::Rotate);
    }

    #[test]
    fn test_scale_kind() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 2.0,
        };
        assert_eq!(m.kind(), GizmoOperationKind::Scale);
    }

    // Test magnitude() for Translate
    #[test]
    fn test_translate_magnitude_zero() {
        let m = GizmoMeasurement::Translate {
            from: ivec2_zero(),
            to: ivec2_zero(),
        };
        assert_eq!(m.magnitude(), 0.0);
    }

    #[test]
    fn test_translate_magnitude_unit_x() {
        let m = GizmoMeasurement::Translate {
            from: ivec2_zero(),
            to: IVec2 { x: 1, y: 0 },
        };
        assert_eq!(m.magnitude(), 1.0);
    }

    #[test]
    fn test_translate_magnitude_unit_y() {
        let m = GizmoMeasurement::Translate {
            from: ivec2_zero(),
            to: IVec2 { x: 0, y: 1 },
        };
        assert_eq!(m.magnitude(), 1.0);
    }

    #[test]
    fn test_translate_magnitude_diagonal() {
        let m = GizmoMeasurement::Translate {
            from: ivec2_zero(),
            to: IVec2 { x: 3, y: 4 },
        };
        assert_eq!(m.magnitude(), 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_translate_magnitude_negative() {
        let m = GizmoMeasurement::Translate {
            from: IVec2 { x: 5, y: 5 },
            to: IVec2 { x: 2, y: 1 }, // -3, -4
        };
        assert_eq!(m.magnitude(), 5.0); // Still 5.0 (absolute)
    }

    // Test magnitude() for Rotate
    #[test]
    fn test_rotate_magnitude_zero() {
        let m = GizmoMeasurement::Rotate {
            from: (0.0, 0.0, 0.0),
            to: (0.0, 0.0, 0.0),
        };
        assert_eq!(m.magnitude(), 0.0);
    }

    #[test]
    fn test_rotate_magnitude_single_axis() {
        let m = GizmoMeasurement::Rotate {
            from: (0.0, 0.0, 0.0),
            to: (1.0, 0.0, 0.0),
        };
        assert_eq!(m.magnitude(), 1.0);
    }

    #[test]
    fn test_rotate_magnitude_multiple_axes() {
        let m = GizmoMeasurement::Rotate {
            from: (0.0, 0.0, 0.0),
            to: (1.0, 2.0, 2.0),
        };
        assert_eq!(m.magnitude(), 3.0); // sqrt(1 + 4 + 4) = 3
    }

    // Test magnitude() for Scale
    #[test]
    fn test_scale_magnitude_zero() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 1.0,
        };
        assert_eq!(m.magnitude(), 0.0);
    }

    #[test]
    fn test_scale_magnitude_increase() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 2.0,
        };
        assert_eq!(m.magnitude(), 1.0);
    }

    #[test]
    fn test_scale_magnitude_decrease() {
        let m = GizmoMeasurement::Scale {
            from: 2.0,
            to: 1.0,
        };
        assert_eq!(m.magnitude(), 1.0); // Absolute value
    }

    #[test]
    fn test_scale_magnitude_large() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 10.0,
        };
        assert_eq!(m.magnitude(), 9.0);
    }

    // Test is_significant boundary: magnitude() > 0.01
    #[test]
    fn test_is_significant_zero() {
        let m = GizmoMeasurement::Translate {
            from: ivec2_zero(),
            to: ivec2_zero(),
        };
        assert!(!m.is_significant());
    }

    #[test]
    fn test_is_significant_scale_at_threshold() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 1.01, // magnitude = 0.01 (at threshold)
        };
        assert!(!m.is_significant()); // > 0.01, not >= 0.01
    }

    #[test]
    fn test_is_significant_scale_above_threshold() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 1.02, // magnitude = 0.02 (above threshold)
        };
        assert!(m.is_significant());
    }

    #[test]
    fn test_is_significant_translate_significant() {
        let m = GizmoMeasurement::Translate {
            from: ivec2_zero(),
            to: IVec2 { x: 1, y: 0 },
        };
        assert!(m.is_significant());
    }

    #[test]
    fn test_is_significant_rotate_significant() {
        let m = GizmoMeasurement::Rotate {
            from: (0.0, 0.0, 0.0),
            to: (0.1, 0.0, 0.0),
        };
        assert!(m.is_significant());
    }

    // Test summary()
    #[test]
    fn test_summary_translate() {
        let m = GizmoMeasurement::Translate {
            from: IVec2 { x: 1, y: 2 },
            to: IVec2 { x: 3, y: 4 },
        };
        let summary = m.summary();
        assert!(summary.contains("Moved"));
        assert!(summary.contains("1"));
        assert!(summary.contains("2"));
        assert!(summary.contains("3"));
        assert!(summary.contains("4"));
    }

    #[test]
    fn test_summary_rotate() {
        let m = GizmoMeasurement::Rotate {
            from: (0.0, 0.0, 0.0),
            to: (1.0, 0.0, 0.0),
        };
        let summary = m.summary();
        assert!(summary.contains("Rotated"));
    }

    #[test]
    fn test_summary_scale() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 2.0,
        };
        let summary = m.summary();
        assert!(summary.contains("Scaled"));
        assert!(summary.contains("1.00"));
        assert!(summary.contains("2.00"));
    }

    // Test Display trait
    #[test]
    fn test_display_translate() {
        let m = GizmoMeasurement::Translate {
            from: IVec2 { x: 0, y: 0 },
            to: IVec2 { x: 1, y: 2 },
        };
        let display = format!("{}", m);
        assert!(display.contains("Move"));
        assert!(display.contains("0"));
        assert!(display.contains("1"));
        assert!(display.contains("2"));
    }

    #[test]
    fn test_display_rotate() {
        let m = GizmoMeasurement::Rotate {
            from: (0.0, 0.0, 0.0),
            to: (1.0, 0.0, 0.0),
        };
        let display = format!("{}", m);
        assert!(display.contains("Rotate"));
    }

    #[test]
    fn test_display_scale() {
        let m = GizmoMeasurement::Scale {
            from: 1.0,
            to: 2.0,
        };
        let display = format!("{}", m);
        assert!(display.contains("Scale"));
    }
}

// ============================================================================
// GIZMO COMMIT METADATA TESTS
// ============================================================================

mod gizmo_commit_metadata_tests {
    use super::*;

    fn make_commit_metadata(constraint: Option<String>) -> GizmoCommitMetadata {
        GizmoCommitMetadata {
            entity: 42,
            operation: GizmoOperationKind::Translate,
            measurement: GizmoMeasurement::Translate {
                from: ivec2_zero(),
                to: IVec2 { x: 1, y: 0 },
            },
            constraint,
        }
    }

    // Test is_constrained
    #[test]
    fn test_is_constrained_none() {
        let meta = make_commit_metadata(None);
        assert!(!meta.is_constrained());
    }

    #[test]
    fn test_is_constrained_some() {
        let meta = make_commit_metadata(Some("X".to_string()));
        assert!(meta.is_constrained());
    }

    #[test]
    fn test_is_constrained_empty_string() {
        let meta = make_commit_metadata(Some("".to_string()));
        assert!(meta.is_constrained()); // Some even if empty
    }

    // Test summary
    #[test]
    fn test_summary_contains_entity() {
        let meta = make_commit_metadata(None);
        let summary = meta.summary();
        assert!(summary.contains("42"));
    }

    #[test]
    fn test_summary_constraint_none() {
        let meta = make_commit_metadata(None);
        let summary = meta.summary();
        assert!(summary.contains("None"));
    }

    #[test]
    fn test_summary_constraint_x() {
        let meta = make_commit_metadata(Some("X".to_string()));
        let summary = meta.summary();
        assert!(summary.contains("X"));
    }

    // Test Display
    #[test]
    fn test_display_contains_committed() {
        let meta = make_commit_metadata(None);
        let display = format!("{}", meta);
        assert!(display.contains("committed"));
    }

    #[test]
    fn test_display_translate() {
        let meta = make_commit_metadata(None);
        let display = format!("{}", meta);
        assert!(display.contains("Translate"));
    }

    #[test]
    fn test_display_rotate() {
        let meta = GizmoCommitMetadata {
            entity: 1,
            operation: GizmoOperationKind::Rotate,
            measurement: GizmoMeasurement::Rotate {
                from: (0.0, 0.0, 0.0),
                to: (1.0, 0.0, 0.0),
            },
            constraint: None,
        };
        let display = format!("{}", meta);
        assert!(display.contains("Rotate"));
    }

    #[test]
    fn test_display_scale() {
        let meta = GizmoCommitMetadata {
            entity: 1,
            operation: GizmoOperationKind::Scale,
            measurement: GizmoMeasurement::Scale { from: 1.0, to: 2.0 },
            constraint: None,
        };
        let display = format!("{}", meta);
        assert!(display.contains("Scale"));
    }
}

// ============================================================================
// GIZMO CANCEL METADATA TESTS
// ============================================================================

mod gizmo_cancel_metadata_tests {
    use super::*;

    fn make_cancel_metadata() -> GizmoCancelMetadata {
        GizmoCancelMetadata {
            entity: 99,
            operation: GizmoOperationKind::Translate,
            snapshot: TransformSnapshot {
                position: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::new(2.0, 2.0, 2.0),
            },
        }
    }

    // Test summary
    #[test]
    fn test_summary_contains_cancelled() {
        let meta = make_cancel_metadata();
        let summary = meta.summary();
        assert!(summary.contains("cancelled"));
    }

    #[test]
    fn test_summary_contains_entity() {
        let meta = make_cancel_metadata();
        let summary = meta.summary();
        assert!(summary.contains("99"));
    }

    #[test]
    fn test_summary_contains_operation() {
        let meta = make_cancel_metadata();
        let summary = meta.summary();
        assert!(summary.contains("Translate"));
    }

    // Test original_position
    #[test]
    fn test_original_position() {
        let meta = make_cancel_metadata();
        assert_eq!(meta.original_position(), Vec3::new(1.0, 2.0, 3.0));
    }

    // Test original_rotation
    #[test]
    fn test_original_rotation() {
        let meta = make_cancel_metadata();
        assert_eq!(meta.original_rotation(), Quat::IDENTITY);
    }

    // Test original_scale
    #[test]
    fn test_original_scale() {
        let meta = make_cancel_metadata();
        assert_eq!(meta.original_scale(), Vec3::new(2.0, 2.0, 2.0));
    }

    // Test Display
    #[test]
    fn test_display_contains_cancelled() {
        let meta = make_cancel_metadata();
        let display = format!("{}", meta);
        assert!(display.contains("cancelled"));
    }

    #[test]
    fn test_display_rotate() {
        let meta = GizmoCancelMetadata {
            entity: 1,
            operation: GizmoOperationKind::Rotate,
            snapshot: TransformSnapshot {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        };
        let display = format!("{}", meta);
        assert!(display.contains("Rotate"));
    }

    #[test]
    fn test_display_scale() {
        let meta = GizmoCancelMetadata {
            entity: 1,
            operation: GizmoOperationKind::Scale,
            snapshot: TransformSnapshot {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        };
        let display = format!("{}", meta);
        assert!(display.contains("Scale"));
    }
}

// ============================================================================
// TRANSFORM SNAPSHOT TESTS
// ============================================================================

mod transform_snapshot_tests {
    use super::*;

    #[test]
    fn test_snapshot_position_access() {
        let snapshot = TransformSnapshot {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        assert_eq!(snapshot.position.x, 1.0);
        assert_eq!(snapshot.position.y, 2.0);
        assert_eq!(snapshot.position.z, 3.0);
    }

    #[test]
    fn test_snapshot_rotation_identity() {
        let snapshot = TransformSnapshot {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        assert_eq!(snapshot.rotation, Quat::IDENTITY);
    }

    #[test]
    fn test_snapshot_scale_access() {
        let snapshot = TransformSnapshot {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(2.0, 3.0, 4.0),
        };
        assert_eq!(snapshot.scale.x, 2.0);
        assert_eq!(snapshot.scale.y, 3.0);
        assert_eq!(snapshot.scale.z, 4.0);
    }

    #[test]
    fn test_snapshot_clone() {
        let original = TransformSnapshot {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(2.0),
        };
        let cloned = original.clone();
        assert_eq!(original.position, cloned.position);
        assert_eq!(original.rotation, cloned.rotation);
        assert_eq!(original.scale, cloned.scale);
    }
}
