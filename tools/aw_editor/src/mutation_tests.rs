//! Mutation-resistant tests for aw_editor systems.
//!
//! These tests are designed to catch common mutations in:
//! - Editor mode transitions
//! - Layout preset handling
//! - Gizmo state management
//! - Entity material system
//! - Panel category logic
//! - Undo stack operations

use crate::dock_layout::LayoutPreset;
use crate::editor_mode::EditorMode;
use crate::entity_manager::{EntityMaterial, MaterialSlot};
use crate::gizmo::state::{AxisConstraint, GizmoMode};
use crate::panel_type::PanelCategory;
use glam::Vec3;
use std::path::PathBuf;

// ============================================================================
// Editor Mode Tests
// ============================================================================

mod editor_mode_tests {
    use super::*;

    #[test]
    fn test_default_mode_is_edit() {
        let mode = EditorMode::default();
        assert!(matches!(mode, EditorMode::Edit), "Default mode should be Edit");
    }

    #[test]
    fn test_is_playing_returns_true_for_play() {
        let mode = EditorMode::Play;
        assert!(mode.is_playing(), "is_playing should return true for Play mode");
    }

    #[test]
    fn test_is_playing_returns_false_for_edit() {
        let mode = EditorMode::Edit;
        assert!(!mode.is_playing(), "is_playing should return false for Edit mode");
    }

    #[test]
    fn test_is_paused_returns_true_for_paused() {
        let mode = EditorMode::Paused;
        assert!(mode.is_paused(), "is_paused should return true for Paused mode");
    }

    #[test]
    fn test_can_edit_returns_true_for_edit() {
        let mode = EditorMode::Edit;
        assert!(mode.can_edit(), "can_edit should return true for Edit mode");
    }

    #[test]
    fn test_can_edit_returns_false_for_play() {
        let mode = EditorMode::Play;
        assert!(!mode.can_edit(), "can_edit should return false for Play mode");
    }

    #[test]
    fn test_valid_transition_edit_to_play() {
        let mode = EditorMode::Edit;
        assert!(mode.can_transition_to(EditorMode::Play), "Edit should transition to Play");
    }

    #[test]
    fn test_invalid_transition_edit_to_paused() {
        let mode = EditorMode::Edit;
        assert!(!mode.can_transition_to(EditorMode::Paused), "Edit should NOT transition to Paused");
    }

    #[test]
    fn test_valid_transitions_from_play() {
        let mode = EditorMode::Play;
        let transitions = mode.valid_transitions();
        assert!(transitions.contains(&EditorMode::Edit), "Play should transition to Edit");
        assert!(transitions.contains(&EditorMode::Paused), "Play should transition to Paused");
    }
}

// ============================================================================
// Layout Preset Tests
// ============================================================================

mod layout_preset_tests {
    use super::*;

    #[test]
    fn test_default_preset_is_default() {
        let preset = LayoutPreset::default();
        assert!(matches!(preset, LayoutPreset::Default), "Default preset should be Default");
    }

    #[test]
    fn test_is_debug_layout_true_for_debug() {
        let preset = LayoutPreset::Debug;
        assert!(preset.is_debug_layout(), "is_debug_layout should return true for Debug");
    }

    #[test]
    fn test_is_debug_layout_false_for_default() {
        let preset = LayoutPreset::Default;
        assert!(!preset.is_debug_layout(), "is_debug_layout should return false for Default");
    }

    #[test]
    fn test_is_content_creation_true_for_modeling() {
        let preset = LayoutPreset::Modeling;
        assert!(preset.is_content_creation_layout(), "is_content_creation_layout should be true for Modeling");
    }

    #[test]
    fn test_is_content_creation_true_for_animation() {
        let preset = LayoutPreset::Animation;
        assert!(preset.is_content_creation_layout(), "is_content_creation_layout should be true for Animation");
    }

    #[test]
    fn test_is_content_creation_false_for_debug() {
        let preset = LayoutPreset::Debug;
        assert!(!preset.is_content_creation_layout(), "is_content_creation_layout should be false for Debug");
    }

    #[test]
    fn test_expected_panel_count_varies_by_preset() {
        let default_count = LayoutPreset::Default.expected_panel_count();
        let wide_count = LayoutPreset::Wide.expected_panel_count();
        assert_ne!(default_count, wide_count, "Different presets should have different panel counts");
    }
}

// ============================================================================
// Gizmo Mode Tests
// ============================================================================

mod gizmo_mode_tests {
    use super::*;

    #[test]
    fn test_default_mode_is_inactive() {
        let mode = GizmoMode::default();
        assert!(matches!(mode, GizmoMode::Inactive), "Default gizmo mode should be Inactive");
    }

    #[test]
    fn test_is_active_false_for_inactive() {
        let mode = GizmoMode::Inactive;
        assert!(!mode.is_active(), "is_active should return false for Inactive");
    }

    #[test]
    fn test_is_active_true_for_translate() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert!(mode.is_active(), "is_active should return true for Translate");
    }

    #[test]
    fn test_is_translate_true_for_translate() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::X };
        assert!(mode.is_translate(), "is_translate should return true for Translate");
    }

    #[test]
    fn test_is_translate_false_for_rotate() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::X };
        assert!(!mode.is_translate(), "is_translate should return false for Rotate");
    }

    #[test]
    fn test_is_rotate_true_for_rotate() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::Y };
        assert!(mode.is_rotate(), "is_rotate should return true for Rotate");
    }

    #[test]
    fn test_is_scale_true_for_scale() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::Z, uniform: false };
        assert!(mode.is_scale(), "is_scale should return true for Scale");
    }

    #[test]
    fn test_constraint_returns_some_for_active_modes() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::XY };
        assert_eq!(mode.constraint(), Some(AxisConstraint::XY), "constraint should return Some for Translate");
    }

    #[test]
    fn test_constraint_returns_none_for_inactive() {
        let mode = GizmoMode::Inactive;
        assert_eq!(mode.constraint(), None, "constraint should return None for Inactive");
    }
}

// ============================================================================
// Axis Constraint Tests
// ============================================================================

mod axis_constraint_tests {
    use super::*;

    #[test]
    fn test_default_constraint_is_none() {
        let constraint = AxisConstraint::default();
        assert!(matches!(constraint, AxisConstraint::None), "Default constraint should be None");
    }

    #[test]
    fn test_single_axis_constraints_are_distinct() {
        let x = AxisConstraint::X;
        let y = AxisConstraint::Y;
        let z = AxisConstraint::Z;
        assert_ne!(x, y, "X != Y");
        assert_ne!(y, z, "Y != Z");
        assert_ne!(x, z, "X != Z");
    }

    #[test]
    fn test_planar_constraints_are_distinct() {
        let xy = AxisConstraint::XY;
        let xz = AxisConstraint::XZ;
        let yz = AxisConstraint::YZ;
        assert_ne!(xy, xz, "XY != XZ");
        assert_ne!(xz, yz, "XZ != YZ");
        assert_ne!(xy, yz, "XY != YZ");
    }
}

// ============================================================================
// Entity Material Tests
// ============================================================================

mod entity_material_tests {
    use super::*;

    #[test]
    fn test_default_material_properties() {
        let mat = EntityMaterial::default();
        assert_eq!(mat.metallic, 0.0, "Default metallic should be 0.0");
        assert_eq!(mat.roughness, 0.5, "Default roughness should be 0.5");
        assert!(mat.emissive.length_squared() < 0.001, "Default should not be emissive");
    }

    #[test]
    fn test_has_textures_false_for_new() {
        let mat = EntityMaterial::new();
        assert!(!mat.has_textures(), "New material should have no textures");
    }

    #[test]
    fn test_has_textures_true_after_set() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("texture.png"));
        assert!(mat.has_textures(), "Material should have textures after set");
    }

    #[test]
    fn test_texture_count_increases() {
        let mut mat = EntityMaterial::new();
        assert_eq!(mat.texture_count(), 0, "Initial count should be 0");
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("a.png"));
        assert_eq!(mat.texture_count(), 1, "Count should be 1 after first texture");
        mat.set_texture(MaterialSlot::Normal, PathBuf::from("n.png"));
        assert_eq!(mat.texture_count(), 2, "Count should be 2 after second texture");
    }

    #[test]
    fn test_clear_texture_removes() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("a.png"));
        assert!(mat.get_texture(MaterialSlot::Albedo).is_some());
        mat.clear_texture(MaterialSlot::Albedo);
        assert!(mat.get_texture(MaterialSlot::Albedo).is_none(), "Texture should be removed");
    }

    #[test]
    fn test_is_metallic_threshold() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.5;
        assert!(!mat.is_metallic(), "0.5 should NOT be metallic (> 0.5 required)");
        mat.metallic = 0.51;
        assert!(mat.is_metallic(), "0.51 should be metallic");
    }

    #[test]
    fn test_is_rough_threshold() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.5;
        assert!(!mat.is_rough(), "0.5 should NOT be rough (> 0.5 required)");
        mat.roughness = 0.51;
        assert!(mat.is_rough(), "0.51 should be rough");
    }

    #[test]
    fn test_is_emissive_threshold() {
        let mut mat = EntityMaterial::new();
        mat.emissive = Vec3::ZERO;
        assert!(!mat.is_emissive(), "Zero emission should not be emissive");
        mat.emissive = Vec3::new(0.1, 0.0, 0.0);
        assert!(mat.is_emissive(), "Non-zero emission should be emissive");
    }
}

// ============================================================================
// MUTATION-RESISTANT TEST MODULE 1: Boundary Condition Tests
// ============================================================================

mod boundary_condition_tests {
    use super::*;

    // --- Layout Preset Panel Count Boundaries ---

    #[test]
    fn test_wide_layout_minimum_panels() {
        let preset = LayoutPreset::Wide;
        assert!(preset.expected_panel_count() >= 2, "Wide should have at least 2 panels");
    }

    #[test]
    fn test_compact_layout_maximum_panels() {
        let preset = LayoutPreset::Compact;
        assert!(preset.expected_panel_count() >= 6, "Compact should have many panels");
    }

    // --- Metallic/Roughness Boundaries ---

    #[test]
    fn test_metallic_exactly_half() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.5;
        assert!(!mat.is_metallic(), "Exactly 0.5 should NOT be metallic (boundary)");
    }

    #[test]
    fn test_metallic_just_above_half() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.500001;
        assert!(mat.is_metallic(), "Just above 0.5 should be metallic");
    }

    #[test]
    fn test_metallic_just_below_half() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.499999;
        assert!(!mat.is_metallic(), "Just below 0.5 should NOT be metallic");
    }

    #[test]
    fn test_roughness_exactly_half() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.5;
        assert!(!mat.is_rough(), "Exactly 0.5 should NOT be rough (boundary)");
    }

    #[test]
    fn test_roughness_just_above_half() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.500001;
        assert!(mat.is_rough(), "Just above 0.5 should be rough");
    }

    // --- Emissive Threshold Boundaries ---

    #[test]
    fn test_emissive_zero() {
        let mut mat = EntityMaterial::new();
        mat.emissive = Vec3::ZERO;
        assert!(!mat.is_emissive(), "Zero emission should not be emissive");
    }

    #[test]
    fn test_emissive_just_above_threshold() {
        let mut mat = EntityMaterial::new();
        // length_squared needs to be > 0.001
        // sqrt(0.001) ≈ 0.0316
        mat.emissive = Vec3::new(0.033, 0.0, 0.0);
        assert!(mat.is_emissive(), "Above threshold should be emissive");
    }

    #[test]
    fn test_emissive_exactly_at_threshold() {
        let mut mat = EntityMaterial::new();
        // length_squared = 0.001 exactly
        // sqrt(0.001) ≈ 0.031622...
        mat.emissive = Vec3::new(0.031622, 0.0, 0.0);
        // 0.031622^2 ≈ 0.000999, which is < 0.001, so NOT emissive
        assert!(!mat.is_emissive(), "At threshold should not be emissive (exclusive)");
    }

    // --- Normal Strength Boundaries ---

    #[test]
    fn test_normal_strength_default() {
        let mat = EntityMaterial::default();
        assert!((mat.normal_strength - 1.0).abs() < f32::EPSILON, "Default normal strength should be 1.0");
    }

    #[test]
    fn test_normal_strength_zero() {
        let mut mat = EntityMaterial::new();
        mat.normal_strength = 0.0;
        assert!((mat.normal_strength).abs() < f32::EPSILON, "Zero normal strength");
    }

    // --- Texture Count Boundaries ---

    #[test]
    fn test_texture_count_zero_initially() {
        let mat = EntityMaterial::new();
        assert_eq!(mat.texture_count(), 0, "Initial texture count should be 0");
    }

    #[test]
    fn test_texture_count_one_after_single_add() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("a.png"));
        assert_eq!(mat.texture_count(), 1, "One texture should give count 1");
    }

    #[test]
    fn test_texture_count_all_slots_filled() {
        let mut mat = EntityMaterial::new();
        for (i, slot) in MaterialSlot::all().iter().enumerate() {
            mat.set_texture(*slot, PathBuf::from(format!("{}.png", i)));
        }
        assert_eq!(mat.texture_count(), MaterialSlot::all().len(), "All slots should be filled");
    }

    // --- Valid Transitions Count Boundaries ---

    #[test]
    fn test_edit_mode_has_one_transition() {
        let mode = EditorMode::Edit;
        assert_eq!(mode.valid_transitions().len(), 1, "Edit should have exactly 1 valid transition");
    }

    #[test]
    fn test_play_mode_has_two_transitions() {
        let mode = EditorMode::Play;
        assert_eq!(mode.valid_transitions().len(), 2, "Play should have exactly 2 valid transitions");
    }

    #[test]
    fn test_paused_mode_has_two_transitions() {
        let mode = EditorMode::Paused;
        assert_eq!(mode.valid_transitions().len(), 2, "Paused should have exactly 2 valid transitions");
    }
}

// ============================================================================
// MUTATION-RESISTANT TEST MODULE 2: Comparison Operator Tests
// ============================================================================

mod comparison_operator_tests {
    use super::*;

    // --- EditorMode Equality ---

    #[test]
    fn test_editor_mode_edit_equals_edit() {
        assert_eq!(EditorMode::Edit, EditorMode::Edit);
    }

    #[test]
    fn test_editor_mode_edit_not_equals_play() {
        assert_ne!(EditorMode::Edit, EditorMode::Play);
    }

    #[test]
    fn test_editor_mode_play_not_equals_paused() {
        assert_ne!(EditorMode::Play, EditorMode::Paused);
    }

    // --- LayoutPreset Equality ---

    #[test]
    fn test_layout_preset_equality() {
        assert_eq!(LayoutPreset::Default, LayoutPreset::Default);
        assert_ne!(LayoutPreset::Default, LayoutPreset::Wide);
        assert_ne!(LayoutPreset::Wide, LayoutPreset::Compact);
    }

    #[test]
    fn test_layout_preset_all_variants_distinct() {
        let presets = LayoutPreset::all();
        for i in 0..presets.len() {
            for j in 0..presets.len() {
                if i != j {
                    assert_ne!(presets[i], presets[j], "Different presets should be distinct");
                }
            }
        }
    }

    // --- GizmoMode Equality ---

    #[test]
    fn test_gizmo_mode_inactive_equals_inactive() {
        assert_eq!(GizmoMode::Inactive, GizmoMode::Inactive);
    }

    #[test]
    fn test_gizmo_mode_translate_not_equals_rotate() {
        let translate = GizmoMode::Translate { constraint: AxisConstraint::X };
        let rotate = GizmoMode::Rotate { constraint: AxisConstraint::X };
        assert_ne!(translate, rotate, "Translate and Rotate should be different");
    }

    #[test]
    fn test_gizmo_mode_same_type_different_constraint() {
        let tx = GizmoMode::Translate { constraint: AxisConstraint::X };
        let ty = GizmoMode::Translate { constraint: AxisConstraint::Y };
        assert_ne!(tx, ty, "Same mode with different constraints should differ");
    }

    // --- AxisConstraint Equality ---

    #[test]
    fn test_axis_constraint_equality() {
        assert_eq!(AxisConstraint::None, AxisConstraint::None);
        assert_eq!(AxisConstraint::X, AxisConstraint::X);
        assert_ne!(AxisConstraint::X, AxisConstraint::Y);
    }

    #[test]
    fn test_axis_constraint_single_vs_planar() {
        assert_ne!(AxisConstraint::X, AxisConstraint::XY, "Single axis != planar");
        assert_ne!(AxisConstraint::Y, AxisConstraint::XY, "Single axis != planar");
    }

    // --- MaterialSlot Equality ---

    #[test]
    fn test_material_slot_equality() {
        assert_eq!(MaterialSlot::Albedo, MaterialSlot::Albedo);
        assert_ne!(MaterialSlot::Albedo, MaterialSlot::Normal);
        assert_ne!(MaterialSlot::Roughness, MaterialSlot::Metallic);
    }

    #[test]
    fn test_material_slot_all_distinct() {
        let slots = MaterialSlot::all();
        for i in 0..slots.len() {
            for j in 0..slots.len() {
                if i != j {
                    assert_ne!(slots[i], slots[j], "Different slots should be distinct");
                }
            }
        }
    }

    // --- PanelCategory Equality ---

    #[test]
    fn test_panel_category_equality() {
        assert_eq!(PanelCategory::Scene, PanelCategory::Scene);
        assert_ne!(PanelCategory::Scene, PanelCategory::Assets);
        assert_ne!(PanelCategory::Debug, PanelCategory::Tools);
    }

    // --- Transition Comparison ---

    #[test]
    fn test_can_transition_to_same_mode() {
        assert!(EditorMode::Edit.can_transition_to(EditorMode::Edit), "Same mode transition should be valid");
        assert!(EditorMode::Play.can_transition_to(EditorMode::Play), "Same mode transition should be valid");
    }

    #[test]
    fn test_asymmetric_transitions() {
        // Edit -> Paused is invalid, but Paused -> Edit is valid
        assert!(!EditorMode::Edit.can_transition_to(EditorMode::Paused), "Edit -> Paused invalid");
        assert!(EditorMode::Paused.can_transition_to(EditorMode::Edit), "Paused -> Edit valid");
    }
}

// ============================================================================
// MUTATION-RESISTANT TEST MODULE 3: Boolean Return Path Tests
// ============================================================================

mod boolean_return_path_tests {
    use super::*;

    // --- EditorMode Boolean Methods ---

    #[test]
    fn test_is_playing_true_path() {
        let result = EditorMode::Play.is_playing();
        assert!(result, "is_playing should be true for Play");
        assert_eq!(result, true, "Result should be exactly true");
    }

    #[test]
    fn test_is_playing_false_path() {
        let result = EditorMode::Edit.is_playing();
        assert!(!result, "is_playing should be false for Edit");
        assert_eq!(result, false, "Result should be exactly false");
    }

    #[test]
    fn test_is_paused_true_path() {
        let result = EditorMode::Paused.is_paused();
        assert!(result, "is_paused should be true for Paused");
        assert_eq!(result, true);
    }

    #[test]
    fn test_is_paused_false_path() {
        let result = EditorMode::Play.is_paused();
        assert!(!result, "is_paused should be false for Play");
        assert_eq!(result, false);
    }

    #[test]
    fn test_is_editing_true_path() {
        let result = EditorMode::Edit.is_editing();
        assert!(result, "is_editing should be true for Edit");
        assert_eq!(result, true);
    }

    #[test]
    fn test_is_editing_false_path() {
        let result = EditorMode::Play.is_editing();
        assert!(!result, "is_editing should be false for Play");
        assert_eq!(result, false);
    }

    #[test]
    fn test_can_edit_true_path() {
        let result = EditorMode::Edit.can_edit();
        assert!(result);
        assert_eq!(result, true);
    }

    #[test]
    fn test_can_edit_false_paths() {
        assert_eq!(EditorMode::Play.can_edit(), false);
        assert_eq!(EditorMode::Paused.can_edit(), false);
    }

    // --- GizmoMode Boolean Methods ---

    #[test]
    fn test_gizmo_is_active_true_paths() {
        assert!(GizmoMode::Translate { constraint: AxisConstraint::None }.is_active());
        assert!(GizmoMode::Rotate { constraint: AxisConstraint::None }.is_active());
        assert!(GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false }.is_active());
    }

    #[test]
    fn test_gizmo_is_active_false_path() {
        assert!(!GizmoMode::Inactive.is_active());
        assert_eq!(GizmoMode::Inactive.is_active(), false);
    }

    #[test]
    fn test_gizmo_is_translate_true_path() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::X };
        assert!(mode.is_translate());
        assert_eq!(mode.is_translate(), true);
    }

    #[test]
    fn test_gizmo_is_translate_false_paths() {
        assert_eq!(GizmoMode::Inactive.is_translate(), false);
        assert_eq!(GizmoMode::Rotate { constraint: AxisConstraint::X }.is_translate(), false);
        assert_eq!(GizmoMode::Scale { constraint: AxisConstraint::X, uniform: false }.is_translate(), false);
    }

    #[test]
    fn test_gizmo_is_rotate_true_path() {
        let mode = GizmoMode::Rotate { constraint: AxisConstraint::Y };
        assert!(mode.is_rotate());
        assert_eq!(mode.is_rotate(), true);
    }

    #[test]
    fn test_gizmo_is_rotate_false_paths() {
        assert_eq!(GizmoMode::Inactive.is_rotate(), false);
        assert_eq!(GizmoMode::Translate { constraint: AxisConstraint::Y }.is_rotate(), false);
    }

    #[test]
    fn test_gizmo_is_scale_true_path() {
        let mode = GizmoMode::Scale { constraint: AxisConstraint::Z, uniform: true };
        assert!(mode.is_scale());
        assert_eq!(mode.is_scale(), true);
    }

    #[test]
    fn test_gizmo_is_scale_false_paths() {
        assert_eq!(GizmoMode::Inactive.is_scale(), false);
        assert_eq!(GizmoMode::Translate { constraint: AxisConstraint::Z }.is_scale(), false);
    }

    // --- LayoutPreset Boolean Methods ---

    #[test]
    fn test_is_debug_layout_true_path() {
        let result = LayoutPreset::Debug.is_debug_layout();
        assert!(result);
        assert_eq!(result, true);
    }

    #[test]
    fn test_is_debug_layout_false_paths() {
        assert_eq!(LayoutPreset::Default.is_debug_layout(), false);
        assert_eq!(LayoutPreset::Wide.is_debug_layout(), false);
        assert_eq!(LayoutPreset::Compact.is_debug_layout(), false);
        assert_eq!(LayoutPreset::Modeling.is_debug_layout(), false);
        assert_eq!(LayoutPreset::Animation.is_debug_layout(), false);
    }

    #[test]
    fn test_is_content_creation_true_paths() {
        assert_eq!(LayoutPreset::Modeling.is_content_creation_layout(), true);
        assert_eq!(LayoutPreset::Animation.is_content_creation_layout(), true);
    }

    #[test]
    fn test_is_content_creation_false_paths() {
        assert_eq!(LayoutPreset::Default.is_content_creation_layout(), false);
        assert_eq!(LayoutPreset::Wide.is_content_creation_layout(), false);
        assert_eq!(LayoutPreset::Debug.is_content_creation_layout(), false);
    }

    // --- EntityMaterial Boolean Methods ---

    #[test]
    fn test_has_textures_true_path() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("a.png"));
        assert!(mat.has_textures());
        assert_eq!(mat.has_textures(), true);
    }

    #[test]
    fn test_has_textures_false_path() {
        let mat = EntityMaterial::new();
        assert!(!mat.has_textures());
        assert_eq!(mat.has_textures(), false);
    }

    #[test]
    fn test_is_metallic_true_path() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.8;
        assert!(mat.is_metallic());
        assert_eq!(mat.is_metallic(), true);
    }

    #[test]
    fn test_is_metallic_false_path() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.3;
        assert!(!mat.is_metallic());
        assert_eq!(mat.is_metallic(), false);
    }

    #[test]
    fn test_is_rough_true_path() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.9;
        assert!(mat.is_rough());
        assert_eq!(mat.is_rough(), true);
    }

    #[test]
    fn test_is_rough_false_path() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.2;
        assert!(!mat.is_rough());
        assert_eq!(mat.is_rough(), false);
    }

    #[test]
    fn test_is_emissive_true_path() {
        let mut mat = EntityMaterial::new();
        mat.emissive = Vec3::new(1.0, 0.0, 0.0);
        assert!(mat.is_emissive());
        assert_eq!(mat.is_emissive(), true);
    }

    #[test]
    fn test_is_emissive_false_path() {
        let mat = EntityMaterial::new();
        assert!(!mat.is_emissive());
        assert_eq!(mat.is_emissive(), false);
    }

    // --- Transition Boolean ---

    #[test]
    fn test_can_transition_to_true_paths() {
        assert_eq!(EditorMode::Edit.can_transition_to(EditorMode::Play), true);
        assert_eq!(EditorMode::Play.can_transition_to(EditorMode::Edit), true);
        assert_eq!(EditorMode::Play.can_transition_to(EditorMode::Paused), true);
        assert_eq!(EditorMode::Paused.can_transition_to(EditorMode::Play), true);
    }

    #[test]
    fn test_can_transition_to_false_path() {
        let result = EditorMode::Edit.can_transition_to(EditorMode::Paused);
        assert!(!result);
        assert_eq!(result, false);
    }

    // --- Option Return Paths ---

    #[test]
    fn test_constraint_some_path() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::XZ };
        assert!(mode.constraint().is_some());
        assert_eq!(mode.constraint(), Some(AxisConstraint::XZ));
    }

    #[test]
    fn test_constraint_none_path() {
        let mode = GizmoMode::Inactive;
        assert!(mode.constraint().is_none());
        assert_eq!(mode.constraint(), None);
    }

    #[test]
    fn test_shortcut_some_path() {
        let mode = GizmoMode::Translate { constraint: AxisConstraint::None };
        assert!(mode.shortcut().is_some());
        assert_eq!(mode.shortcut(), Some("G"));
    }

    #[test]
    fn test_shortcut_none_path() {
        let mode = GizmoMode::Inactive;
        assert!(mode.shortcut().is_none());
        assert_eq!(mode.shortcut(), None);
    }

    #[test]
    fn test_get_texture_some_path() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Normal, PathBuf::from("n.png"));
        assert!(mat.get_texture(MaterialSlot::Normal).is_some());
    }

    #[test]
    fn test_get_texture_none_path() {
        let mat = EntityMaterial::new();
        assert!(mat.get_texture(MaterialSlot::Normal).is_none());
    }

    #[test]
    fn test_shortcut_hint_some_paths() {
        assert!(LayoutPreset::Default.shortcut_hint().is_some());
        assert!(LayoutPreset::Wide.shortcut_hint().is_some());
    }
}
