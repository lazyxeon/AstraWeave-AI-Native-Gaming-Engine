//! Mutation-Resistant Tests: Gizmo System
//!
//! Comprehensive tests for GizmoMode, AxisConstraint
//! to achieve ≥92% mutation kill rate.

use aw_editor_lib::gizmo::{GizmoMode, AxisConstraint, GizmoState};
use glam::Vec3;

// =============================================================================
// GIZMO MODE - IS_ACTIVE() TESTS
// =============================================================================

mod gizmo_mode_is_active_tests {
    use super::*;

    #[test]
    fn inactive_is_not_active() {
        assert!(!GizmoMode::Inactive.is_active());
    }

    #[test]
    fn translate_is_active() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert!(mode.is_active());
    }

    #[test]
    fn rotate_is_active() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::None };
        assert!(mode.is_active());
    }

    #[test]
    fn scale_is_active() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false };
        assert!(mode.is_active());
    }

    #[test]
    fn scale_uniform_is_active() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: true };
        assert!(mode.is_active());
    }
}

// =============================================================================
// GIZMO MODE - IS_TRANSLATE() TESTS
// =============================================================================

mod gizmo_mode_is_translate_tests {
    use super::*;

    #[test]
    fn translate_is_translate_true() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert!(mode.is_translate());
    }

    #[test]
    fn translate_with_x_constraint_is_translate_true() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::X };
        assert!(mode.is_translate());
    }

    #[test]
    fn inactive_is_translate_false() {
        assert!(!GizmoMode::Inactive.is_translate());
    }

    #[test]
    fn rotate_is_translate_false() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::None };
        assert!(!mode.is_translate());
    }

    #[test]
    fn scale_is_translate_false() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false };
        assert!(!mode.is_translate());
    }
}

// =============================================================================
// GIZMO MODE - IS_ROTATE() TESTS
// =============================================================================

mod gizmo_mode_is_rotate_tests {
    use super::*;

    #[test]
    fn rotate_is_rotate_true() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::None };
        assert!(mode.is_rotate());
    }

    #[test]
    fn rotate_with_constraint_is_rotate_true() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::Y };
        assert!(mode.is_rotate());
    }

    #[test]
    fn inactive_is_rotate_false() {
        assert!(!GizmoMode::Inactive.is_rotate());
    }

    #[test]
    fn translate_is_rotate_false() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert!(!mode.is_rotate());
    }

    #[test]
    fn scale_is_rotate_false() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false };
        assert!(!mode.is_rotate());
    }
}

// =============================================================================
// GIZMO MODE - IS_SCALE() TESTS
// =============================================================================

mod gizmo_mode_is_scale_tests {
    use super::*;

    #[test]
    fn scale_is_scale_true() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false };
        assert!(mode.is_scale());
    }

    #[test]
    fn scale_uniform_is_scale_true() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: true };
        assert!(mode.is_scale());
    }

    #[test]
    fn scale_with_constraint_is_scale_true() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::Z, uniform: false };
        assert!(mode.is_scale());
    }

    #[test]
    fn inactive_is_scale_false() {
        assert!(!GizmoMode::Inactive.is_scale());
    }

    #[test]
    fn translate_is_scale_false() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert!(!mode.is_scale());
    }

    #[test]
    fn rotate_is_scale_false() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::None };
        assert!(!mode.is_scale());
    }
}

// =============================================================================
// GIZMO MODE - CONSTRAINT() TESTS
// =============================================================================

mod gizmo_mode_constraint_tests {
    use super::*;

    #[test]
    fn inactive_has_no_constraint() {
        assert!(GizmoMode::Inactive.constraint().is_none());
    }

    #[test]
    fn translate_with_none_returns_none_constraint() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert_eq!(mode.constraint(), Some(AxisConstraint::None));
    }

    #[test]
    fn translate_with_x_returns_x_constraint() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::X };
        assert_eq!(mode.constraint(), Some(AxisConstraint::X));
    }

    #[test]
    fn rotate_with_y_returns_y_constraint() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::Y };
        assert_eq!(mode.constraint(), Some(AxisConstraint::Y));
    }

    #[test]
    fn scale_with_z_returns_z_constraint() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::Z, uniform: false };
        assert_eq!(mode.constraint(), Some(AxisConstraint::Z));
    }
}

// =============================================================================
// GIZMO MODE - NAME() TESTS
// =============================================================================

mod gizmo_mode_name_tests {
    use super::*;

    #[test]
    fn inactive_name_is_inactive() {
        assert_eq!(GizmoMode::Inactive.name(), "Inactive");
    }

    #[test]
    fn translate_name_is_translate() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert_eq!(mode.name(), "Translate");
    }

    #[test]
    fn rotate_name_is_rotate() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::None };
        assert_eq!(mode.name(), "Rotate");
    }

    #[test]
    fn scale_name_is_scale() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false };
        assert_eq!(mode.name(), "Scale");
    }
}

// =============================================================================
// GIZMO MODE - ICON() TESTS
// =============================================================================

mod gizmo_mode_icon_tests {
    use super::*;

    #[test]
    fn inactive_icon_is_not_empty() {
        assert!(!GizmoMode::Inactive.icon().is_empty());
    }

    #[test]
    fn translate_icon_is_not_empty() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert!(!mode.icon().is_empty());
    }

    #[test]
    fn rotate_icon_is_not_empty() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::None };
        assert!(!mode.icon().is_empty());
    }

    #[test]
    fn scale_icon_is_not_empty() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false };
        assert!(!mode.icon().is_empty());
    }
}

// =============================================================================
// GIZMO MODE - SHORTCUT() TESTS
// =============================================================================

mod gizmo_mode_shortcut_tests {
    use super::*;

    #[test]
    fn inactive_has_no_shortcut() {
        assert!(GizmoMode::Inactive.shortcut().is_none());
    }

    #[test]
    fn translate_shortcut_is_g() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert_eq!(mode.shortcut(), Some("G"));
    }

    #[test]
    fn rotate_shortcut_is_r() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::None };
        assert_eq!(mode.shortcut(), Some("R"));
    }

    #[test]
    fn scale_shortcut_is_s() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false };
        assert_eq!(mode.shortcut(), Some("S"));
    }
}

// =============================================================================
// GIZMO MODE - ALL() TESTS
// =============================================================================

mod gizmo_mode_all_tests {
    use super::*;

    #[test]
    fn all_returns_4_modes() {
        assert_eq!(GizmoMode::all().len(), 4);
    }
}

// =============================================================================
// AXIS CONSTRAINT - IS_PLANAR() TESTS
// =============================================================================

mod axis_constraint_is_planar_tests {
    use super::*;

    #[test]
    fn xy_is_planar_true() {
        assert!(AxisConstraint::XY.is_planar());
    }

    #[test]
    fn xz_is_planar_true() {
        assert!(AxisConstraint::XZ.is_planar());
    }

    #[test]
    fn yz_is_planar_true() {
        assert!(AxisConstraint::YZ.is_planar());
    }

    #[test]
    fn none_is_planar_false() {
        assert!(!AxisConstraint::None.is_planar());
    }

    #[test]
    fn x_is_planar_false() {
        assert!(!AxisConstraint::X.is_planar());
    }

    #[test]
    fn y_is_planar_false() {
        assert!(!AxisConstraint::Y.is_planar());
    }

    #[test]
    fn z_is_planar_false() {
        assert!(!AxisConstraint::Z.is_planar());
    }
}

// =============================================================================
// AXIS CONSTRAINT - IS_SINGLE_AXIS() TESTS
// =============================================================================

mod axis_constraint_is_single_axis_tests {
    use super::*;

    #[test]
    fn x_is_single_axis_true() {
        assert!(AxisConstraint::X.is_single_axis());
    }

    #[test]
    fn y_is_single_axis_true() {
        assert!(AxisConstraint::Y.is_single_axis());
    }

    #[test]
    fn z_is_single_axis_true() {
        assert!(AxisConstraint::Z.is_single_axis());
    }

    #[test]
    fn none_is_single_axis_false() {
        assert!(!AxisConstraint::None.is_single_axis());
    }

    #[test]
    fn xy_is_single_axis_false() {
        assert!(!AxisConstraint::XY.is_single_axis());
    }

    #[test]
    fn xz_is_single_axis_false() {
        assert!(!AxisConstraint::XZ.is_single_axis());
    }

    #[test]
    fn yz_is_single_axis_false() {
        assert!(!AxisConstraint::YZ.is_single_axis());
    }
}

// =============================================================================
// AXIS CONSTRAINT - AXIS_VECTOR() TESTS
// =============================================================================

mod axis_constraint_axis_vector_tests {
    use super::*;

    #[test]
    fn none_vector_is_one() {
        let v = AxisConstraint::None.axis_vector();
        assert_eq!(v, Vec3::ONE);
    }

    #[test]
    fn x_vector_is_x() {
        let v = AxisConstraint::X.axis_vector();
        assert_eq!(v, Vec3::X);
    }

    #[test]
    fn y_vector_is_y() {
        let v = AxisConstraint::Y.axis_vector();
        assert_eq!(v, Vec3::Y);
    }

    #[test]
    fn z_vector_is_z() {
        let v = AxisConstraint::Z.axis_vector();
        assert_eq!(v, Vec3::Z);
    }

    #[test]
    fn xy_vector_is_1_1_0() {
        let v = AxisConstraint::XY.axis_vector();
        assert_eq!(v, Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn xz_vector_is_1_0_1() {
        let v = AxisConstraint::XZ.axis_vector();
        assert_eq!(v, Vec3::new(1.0, 0.0, 1.0));
    }

    #[test]
    fn yz_vector_is_0_1_1() {
        let v = AxisConstraint::YZ.axis_vector();
        assert_eq!(v, Vec3::new(0.0, 1.0, 1.0));
    }
}

// =============================================================================
// AXIS CONSTRAINT - COLOR() TESTS
// =============================================================================

mod axis_constraint_color_tests {
    use super::*;

    #[test]
    fn none_color_is_white() {
        let color = AxisConstraint::None.color();
        assert!((color[0] - 1.0).abs() < 0.01);
        assert!((color[1] - 1.0).abs() < 0.01);
        assert!((color[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn x_color_has_high_red() {
        let color = AxisConstraint::X.color();
        assert!(color[0] > 0.8, "X should have high red component");
    }

    #[test]
    fn y_color_has_high_green() {
        let color = AxisConstraint::Y.color();
        assert!(color[1] > 0.8, "Y should have high green component");
    }

    #[test]
    fn z_color_has_high_blue() {
        let color = AxisConstraint::Z.color();
        assert!(color[2] > 0.8, "Z should have high blue component");
    }

    #[test]
    fn xy_color_is_yellow() {
        let color = AxisConstraint::XY.color();
        assert!(color[0] > 0.8, "XY should have high red");
        assert!(color[1] > 0.8, "XY should have high green");
    }

    #[test]
    fn xz_color_is_magenta() {
        let color = AxisConstraint::XZ.color();
        assert!(color[0] > 0.8, "XZ should have high red");
        assert!(color[2] > 0.8, "XZ should have high blue");
    }

    #[test]
    fn yz_color_is_cyan() {
        let color = AxisConstraint::YZ.color();
        assert!(color[1] > 0.8, "YZ should have high green");
        assert!(color[2] > 0.8, "YZ should have high blue");
    }
}

// =============================================================================
// AXIS CONSTRAINT - CYCLE() TESTS
// =============================================================================

mod axis_constraint_cycle_tests {
    use super::*;

    #[test]
    fn none_plus_x_gives_x() {
        let result = AxisConstraint::None.cycle(AxisConstraint::X);
        assert_eq!(result, AxisConstraint::X);
    }

    #[test]
    fn none_plus_y_gives_y() {
        let result = AxisConstraint::None.cycle(AxisConstraint::Y);
        assert_eq!(result, AxisConstraint::Y);
    }

    #[test]
    fn none_plus_z_gives_z() {
        let result = AxisConstraint::None.cycle(AxisConstraint::Z);
        assert_eq!(result, AxisConstraint::Z);
    }

    #[test]
    fn x_plus_x_gives_yz() {
        let result = AxisConstraint::X.cycle(AxisConstraint::X);
        assert_eq!(result, AxisConstraint::YZ);
    }

    #[test]
    fn y_plus_y_gives_xz() {
        let result = AxisConstraint::Y.cycle(AxisConstraint::Y);
        assert_eq!(result, AxisConstraint::XZ);
    }

    #[test]
    fn z_plus_z_gives_xy() {
        let result = AxisConstraint::Z.cycle(AxisConstraint::Z);
        assert_eq!(result, AxisConstraint::XY);
    }

    #[test]
    fn yz_plus_x_gives_none() {
        let result = AxisConstraint::YZ.cycle(AxisConstraint::X);
        assert_eq!(result, AxisConstraint::None);
    }

    #[test]
    fn xz_plus_y_gives_none() {
        let result = AxisConstraint::XZ.cycle(AxisConstraint::Y);
        assert_eq!(result, AxisConstraint::None);
    }

    #[test]
    fn xy_plus_z_gives_none() {
        let result = AxisConstraint::XY.cycle(AxisConstraint::Z);
        assert_eq!(result, AxisConstraint::None);
    }
}

// =============================================================================
// AXIS CONSTRAINT - NAME() TESTS
// =============================================================================

mod axis_constraint_name_tests {
    use super::*;

    #[test]
    fn none_name_is_free() {
        assert_eq!(AxisConstraint::None.name(), "Free");
    }

    #[test]
    fn x_name_is_x_axis() {
        assert_eq!(AxisConstraint::X.name(), "X Axis");
    }

    #[test]
    fn y_name_is_y_axis() {
        assert_eq!(AxisConstraint::Y.name(), "Y Axis");
    }

    #[test]
    fn z_name_is_z_axis() {
        assert_eq!(AxisConstraint::Z.name(), "Z Axis");
    }

    #[test]
    fn xy_name_is_xy_plane() {
        assert_eq!(AxisConstraint::XY.name(), "XY Plane");
    }

    #[test]
    fn xz_name_is_xz_plane() {
        assert_eq!(AxisConstraint::XZ.name(), "XZ Plane");
    }

    #[test]
    fn yz_name_is_yz_plane() {
        assert_eq!(AxisConstraint::YZ.name(), "YZ Plane");
    }
}

// =============================================================================
// AXIS CONSTRAINT - ALL() TESTS
// =============================================================================

mod axis_constraint_all_tests {
    use super::*;

    #[test]
    fn all_returns_7_constraints() {
        assert_eq!(AxisConstraint::all().len(), 7);
    }

    #[test]
    fn all_contains_none() {
        assert!(AxisConstraint::all().contains(&AxisConstraint::None));
    }

    #[test]
    fn all_contains_x() {
        assert!(AxisConstraint::all().contains(&AxisConstraint::X));
    }

    #[test]
    fn all_contains_y() {
        assert!(AxisConstraint::all().contains(&AxisConstraint::Y));
    }

    #[test]
    fn all_contains_z() {
        assert!(AxisConstraint::all().contains(&AxisConstraint::Z));
    }

    #[test]
    fn all_contains_xy() {
        assert!(AxisConstraint::all().contains(&AxisConstraint::XY));
    }

    #[test]
    fn all_contains_xz() {
        assert!(AxisConstraint::all().contains(&AxisConstraint::XZ));
    }

    #[test]
    fn all_contains_yz() {
        assert!(AxisConstraint::all().contains(&AxisConstraint::YZ));
    }
}

// =============================================================================
// GIZMO STATE - DEFAULT TESTS
// =============================================================================

mod gizmo_state_default_tests {
    use super::*;

    #[test]
    fn default_mode_is_inactive() {
        let state = GizmoState::default();
        assert_eq!(state.mode, GizmoMode::Inactive);
    }

    #[test]
    fn default_selected_entity_is_none() {
        let state = GizmoState::default();
        assert!(state.selected_entity.is_none());
    }

    #[test]
    fn default_start_transform_is_none() {
        let state = GizmoState::default();
        assert!(state.start_transform.is_none());
    }

    #[test]
    fn default_start_mouse_is_none() {
        let state = GizmoState::default();
        assert!(state.start_mouse.is_none());
    }
}
