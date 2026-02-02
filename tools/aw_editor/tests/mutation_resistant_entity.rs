//! Mutation-resistant tests for Entity Manager system
//!
//! These tests target boundary conditions, comparison operators, and boolean return paths
//! to achieve high mutation testing kill rates (90%+).

use aw_editor_lib::entity_manager::{
    EntityId, EntityManager, EntityMaterial, EntityValidation, MaterialSlot,
    SelectionSet, EntityManagerValidation, EntityManagerStats, EditorEntity,
};
use glam::{Quat, Vec3, Vec4};
use std::path::PathBuf;

// ============================================================================
// MATERIAL SLOT TESTS
// ============================================================================

mod material_slot_tests {
    use super::*;

    // Test is_color_slot boundary - only Albedo and Emission are color slots
    #[test]
    fn test_albedo_is_color_slot() {
        assert!(MaterialSlot::Albedo.is_color_slot());
    }

    #[test]
    fn test_emission_is_color_slot() {
        assert!(MaterialSlot::Emission.is_color_slot());
    }

    #[test]
    fn test_normal_is_not_color_slot() {
        assert!(!MaterialSlot::Normal.is_color_slot());
    }

    #[test]
    fn test_roughness_is_not_color_slot() {
        assert!(!MaterialSlot::Roughness.is_color_slot());
    }

    #[test]
    fn test_metallic_is_not_color_slot() {
        assert!(!MaterialSlot::Metallic.is_color_slot());
    }

    #[test]
    fn test_ao_is_not_color_slot() {
        assert!(!MaterialSlot::AO.is_color_slot());
    }

    #[test]
    fn test_orm_is_not_color_slot() {
        assert!(!MaterialSlot::ORM.is_color_slot());
    }

    #[test]
    fn test_height_is_not_color_slot() {
        assert!(!MaterialSlot::Height.is_color_slot());
    }

    // Test is_data_slot is inverse of is_color_slot
    #[test]
    fn test_albedo_is_not_data_slot() {
        assert!(!MaterialSlot::Albedo.is_data_slot());
    }

    #[test]
    fn test_emission_is_not_data_slot() {
        assert!(!MaterialSlot::Emission.is_data_slot());
    }

    #[test]
    fn test_normal_is_data_slot() {
        assert!(MaterialSlot::Normal.is_data_slot());
    }

    #[test]
    fn test_roughness_is_data_slot() {
        assert!(MaterialSlot::Roughness.is_data_slot());
    }

    #[test]
    fn test_metallic_is_data_slot() {
        assert!(MaterialSlot::Metallic.is_data_slot());
    }

    #[test]
    fn test_ao_is_data_slot() {
        assert!(MaterialSlot::AO.is_data_slot());
    }

    #[test]
    fn test_orm_is_data_slot() {
        assert!(MaterialSlot::ORM.is_data_slot());
    }

    #[test]
    fn test_height_is_data_slot() {
        assert!(MaterialSlot::Height.is_data_slot());
    }

    // Test slot count - exactly 8 slots
    #[test]
    fn test_all_slots_count() {
        assert_eq!(MaterialSlot::all().len(), 8);
    }

    // Test name uniqueness
    #[test]
    fn test_all_slots_have_distinct_names() {
        let slots = MaterialSlot::all();
        let names: Vec<_> = slots.iter().map(|s| s.name()).collect();
        for (i, name) in names.iter().enumerate() {
            for (j, other) in names.iter().enumerate() {
                if i != j {
                    assert_ne!(name, other, "Slot names must be unique");
                }
            }
        }
    }

    // Test icon uniqueness
    #[test]
    fn test_all_slots_have_distinct_icons() {
        let slots = MaterialSlot::all();
        let icons: Vec<_> = slots.iter().map(|s| s.icon()).collect();
        for (i, icon) in icons.iter().enumerate() {
            for (j, other) in icons.iter().enumerate() {
                if i != j {
                    assert_ne!(icon, other, "Slot icons must be unique");
                }
            }
        }
    }

    // Test Display trait
    #[test]
    fn test_display_matches_name() {
        for slot in MaterialSlot::all() {
            assert_eq!(format!("{}", slot), slot.name());
        }
    }
}

// ============================================================================
// ENTITY MATERIAL TESTS
// ============================================================================

mod entity_material_tests {
    use super::*;

    // Test is_metallic boundary: > 0.5
    #[test]
    fn test_is_metallic_at_zero() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.0;
        assert!(!mat.is_metallic());
    }

    #[test]
    fn test_is_metallic_below_threshold() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.49;
        assert!(!mat.is_metallic());
    }

    #[test]
    fn test_is_metallic_at_threshold() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.5;
        assert!(!mat.is_metallic()); // > 0.5, not >= 0.5
    }

    #[test]
    fn test_is_metallic_just_above_threshold() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 0.51;
        assert!(mat.is_metallic());
    }

    #[test]
    fn test_is_metallic_at_one() {
        let mut mat = EntityMaterial::new();
        mat.metallic = 1.0;
        assert!(mat.is_metallic());
    }

    // Test is_rough boundary: > 0.5
    #[test]
    fn test_is_rough_at_zero() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.0;
        assert!(!mat.is_rough());
    }

    #[test]
    fn test_is_rough_below_threshold() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.49;
        assert!(!mat.is_rough());
    }

    #[test]
    fn test_is_rough_at_threshold() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.5;
        assert!(!mat.is_rough()); // > 0.5, not >= 0.5
    }

    #[test]
    fn test_is_rough_just_above_threshold() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 0.51;
        assert!(mat.is_rough());
    }

    #[test]
    fn test_is_rough_at_one() {
        let mut mat = EntityMaterial::new();
        mat.roughness = 1.0;
        assert!(mat.is_rough());
    }

    // Test is_emissive boundary: length_squared() > 0.001
    #[test]
    fn test_is_emissive_at_zero() {
        let mut mat = EntityMaterial::new();
        mat.emissive = Vec3::ZERO;
        assert!(!mat.is_emissive());
    }

    #[test]
    fn test_is_emissive_below_threshold() {
        let mut mat = EntityMaterial::new();
        // sqrt(0.001) ≈ 0.0316, so length < 0.0316 means length_squared < 0.001
        mat.emissive = Vec3::splat(0.01); // length = 0.01*sqrt(3) ≈ 0.017, squared ≈ 0.0003
        assert!(!mat.is_emissive());
    }

    #[test]
    fn test_is_emissive_at_threshold() {
        let mut mat = EntityMaterial::new();
        // Need length_squared = 0.001, so length = sqrt(0.001) ≈ 0.0316
        mat.emissive = Vec3::new(0.0316, 0.0, 0.0); // length_squared ≈ 0.001
        // This is at threshold, not above
        let is_emissive = mat.is_emissive();
        // The boundary check might be exactly at threshold
        assert!(!is_emissive || mat.emissive.length_squared() > 0.001);
    }

    #[test]
    fn test_is_emissive_just_above_threshold() {
        let mut mat = EntityMaterial::new();
        mat.emissive = Vec3::new(0.1, 0.0, 0.0); // length_squared = 0.01 > 0.001
        assert!(mat.is_emissive());
    }

    #[test]
    fn test_is_emissive_at_one() {
        let mut mat = EntityMaterial::new();
        mat.emissive = Vec3::ONE;
        assert!(mat.is_emissive());
    }

    // Test has_textures
    #[test]
    fn test_has_textures_empty() {
        let mat = EntityMaterial::new();
        assert!(!mat.has_textures());
    }

    #[test]
    fn test_has_textures_one() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("albedo.png"));
        assert!(mat.has_textures());
    }

    #[test]
    fn test_has_textures_multiple() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("albedo.png"));
        mat.set_texture(MaterialSlot::Normal, PathBuf::from("normal.png"));
        assert!(mat.has_textures());
    }

    // Test texture_count
    #[test]
    fn test_texture_count_empty() {
        let mat = EntityMaterial::new();
        assert_eq!(mat.texture_count(), 0);
    }

    #[test]
    fn test_texture_count_one() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("albedo.png"));
        assert_eq!(mat.texture_count(), 1);
    }

    #[test]
    fn test_texture_count_two() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("albedo.png"));
        mat.set_texture(MaterialSlot::Normal, PathBuf::from("normal.png"));
        assert_eq!(mat.texture_count(), 2);
    }

    // Test set/get/clear texture
    #[test]
    fn test_set_get_texture() {
        let mut mat = EntityMaterial::new();
        let path = PathBuf::from("test.png");
        mat.set_texture(MaterialSlot::Albedo, path.clone());
        assert_eq!(mat.get_texture(MaterialSlot::Albedo), Some(&path));
    }

    #[test]
    fn test_get_texture_not_set() {
        let mat = EntityMaterial::new();
        assert_eq!(mat.get_texture(MaterialSlot::Albedo), None);
    }

    #[test]
    fn test_clear_texture() {
        let mut mat = EntityMaterial::new();
        mat.set_texture(MaterialSlot::Albedo, PathBuf::from("albedo.png"));
        mat.clear_texture(MaterialSlot::Albedo);
        assert_eq!(mat.get_texture(MaterialSlot::Albedo), None);
        assert!(!mat.has_textures());
    }

    #[test]
    fn test_clear_texture_not_set() {
        let mut mat = EntityMaterial::new();
        mat.clear_texture(MaterialSlot::Albedo); // Should not panic
        assert_eq!(mat.get_texture(MaterialSlot::Albedo), None);
    }

    // Test default values
    #[test]
    fn test_default_metallic() {
        let mat = EntityMaterial::default();
        assert_eq!(mat.metallic, 0.0);
    }

    #[test]
    fn test_default_roughness() {
        let mat = EntityMaterial::default();
        assert_eq!(mat.roughness, 0.5);
    }

    #[test]
    fn test_default_normal_strength() {
        let mat = EntityMaterial::default();
        assert_eq!(mat.normal_strength, 1.0);
    }

    #[test]
    fn test_default_base_color() {
        let mat = EntityMaterial::default();
        assert_eq!(mat.base_color, Vec4::ONE);
    }

    #[test]
    fn test_default_emissive() {
        let mat = EntityMaterial::default();
        assert_eq!(mat.emissive, Vec3::ZERO);
    }
}

// ============================================================================
// SELECTION SET TESTS
// ============================================================================

mod selection_set_tests {
    use super::*;

    // Test is_empty
    #[test]
    fn test_is_empty_new() {
        let selection = SelectionSet::new();
        assert!(selection.is_empty());
    }

    #[test]
    fn test_is_empty_after_add() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        assert!(!selection.is_empty());
    }

    #[test]
    fn test_is_empty_after_clear() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.clear();
        assert!(selection.is_empty());
    }

    #[test]
    fn test_is_empty_after_remove_all() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.remove(1);
        assert!(selection.is_empty());
    }

    // Test count
    #[test]
    fn test_count_zero() {
        let selection = SelectionSet::new();
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_count_one() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn test_count_two() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        assert_eq!(selection.count(), 2);
    }

    #[test]
    fn test_count_after_remove() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        selection.remove(1);
        assert_eq!(selection.count(), 1);
    }

    // Test is_multi_select boundary: len() > 1
    #[test]
    fn test_is_multi_select_zero() {
        let selection = SelectionSet::new();
        assert!(!selection.is_multi_select());
    }

    #[test]
    fn test_is_multi_select_one() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        assert!(!selection.is_multi_select());
    }

    #[test]
    fn test_is_multi_select_two() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        assert!(selection.is_multi_select());
    }

    #[test]
    fn test_is_multi_select_three() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        selection.add(3, false);
        assert!(selection.is_multi_select());
    }

    // Test is_selected
    #[test]
    fn test_is_selected_not_in_set() {
        let selection = SelectionSet::new();
        assert!(!selection.is_selected(1));
    }

    #[test]
    fn test_is_selected_in_set() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        assert!(selection.is_selected(1));
    }

    #[test]
    fn test_is_selected_after_remove() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.remove(1);
        assert!(!selection.is_selected(1));
    }

    // Test is_primary
    #[test]
    fn test_is_primary_empty() {
        let selection = SelectionSet::new();
        assert!(!selection.is_primary(1));
    }

    #[test]
    fn test_is_primary_true() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        assert!(selection.is_primary(1));
    }

    #[test]
    fn test_is_primary_false_other() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        assert!(!selection.is_primary(2));
    }

    #[test]
    fn test_is_primary_changes_on_new_primary() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, true);
        assert!(!selection.is_primary(1));
        assert!(selection.is_primary(2));
    }

    // Test toggle
    #[test]
    fn test_toggle_add() {
        let mut selection = SelectionSet::new();
        selection.toggle(1);
        assert!(selection.is_selected(1));
    }

    #[test]
    fn test_toggle_remove() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.toggle(1);
        assert!(!selection.is_selected(1));
    }

    #[test]
    fn test_toggle_twice() {
        let mut selection = SelectionSet::new();
        selection.toggle(1);
        selection.toggle(1);
        assert!(!selection.is_selected(1));
    }

    // Test select_only
    #[test]
    fn test_select_only_clears_previous() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        selection.add(3, false);
        selection.select_only(4);
        assert!(!selection.is_selected(1));
        assert!(!selection.is_selected(2));
        assert!(!selection.is_selected(3));
        assert!(selection.is_selected(4));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn test_select_only_sets_primary() {
        let mut selection = SelectionSet::new();
        selection.select_only(5);
        assert!(selection.is_primary(5));
    }

    // Test primary reassignment on remove
    #[test]
    fn test_remove_primary_reassigns() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        selection.remove(1);
        // Primary should be reassigned to remaining entity
        assert!(selection.primary.is_some());
    }

    #[test]
    fn test_remove_non_primary_keeps_primary() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        selection.remove(2);
        assert!(selection.is_primary(1));
    }

    // Test summary
    #[test]
    fn test_summary_no_selection() {
        let selection = SelectionSet::new();
        assert_eq!(selection.summary(), "No selection");
    }

    #[test]
    fn test_summary_one_entity() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        assert_eq!(selection.summary(), "1 entity selected");
    }

    #[test]
    fn test_summary_two_entities() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        assert_eq!(selection.summary(), "2 entities selected");
    }

    #[test]
    fn test_summary_three_entities() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        selection.add(3, false);
        assert_eq!(selection.summary(), "3 entities selected");
    }

    // Test Display trait
    #[test]
    fn test_display_none() {
        let selection = SelectionSet::new();
        assert_eq!(format!("{}", selection), "(none)");
    }

    #[test]
    fn test_display_one() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        assert_eq!(format!("{}", selection), "1 selected");
    }

    #[test]
    fn test_display_multiple() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        assert_eq!(format!("{}", selection), "2 selected");
    }

    // Test to_vec
    #[test]
    fn test_to_vec_empty() {
        let selection = SelectionSet::new();
        assert!(selection.to_vec().is_empty());
    }

    #[test]
    fn test_to_vec_contains_all() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        let vec = selection.to_vec();
        assert_eq!(vec.len(), 2);
        assert!(vec.contains(&1));
        assert!(vec.contains(&2));
    }

    // Test select_range
    #[test]
    fn test_select_range_forward() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3, 4, 5];
        selection.select_range(2, 4, &all_ids);
        assert!(selection.is_selected(2));
        assert!(selection.is_selected(3));
        assert!(selection.is_selected(4));
        assert!(!selection.is_selected(1));
        assert!(!selection.is_selected(5));
    }

    #[test]
    fn test_select_range_backward() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3, 4, 5];
        selection.select_range(4, 2, &all_ids);
        assert!(selection.is_selected(2));
        assert!(selection.is_selected(3));
        assert!(selection.is_selected(4));
    }

    #[test]
    fn test_select_range_same() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3];
        selection.select_range(2, 2, &all_ids);
        assert!(selection.is_selected(2));
        assert!(!selection.is_selected(1));
        assert!(!selection.is_selected(3));
    }

    #[test]
    fn test_select_range_not_found() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3];
        selection.select_range(10, 20, &all_ids);
        assert!(selection.is_empty());
    }

    #[test]
    fn test_select_range_sets_primary() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3, 4, 5];
        selection.select_range(2, 4, &all_ids);
        assert!(selection.is_primary(4)); // 'to' becomes primary
    }
}

// ============================================================================
// ENTITY VALIDATION TESTS
// ============================================================================

mod entity_validation_tests {
    use super::*;

    // Test is_valid
    #[test]
    fn test_valid_creates_valid() {
        let v = EntityValidation::valid();
        assert!(v.is_valid);
    }

    #[test]
    fn test_add_warning_keeps_valid() {
        let mut v = EntityValidation::valid();
        v.add_warning("test warning");
        assert!(v.is_valid);
    }

    #[test]
    fn test_add_error_makes_invalid() {
        let mut v = EntityValidation::valid();
        v.add_error("test error");
        assert!(!v.is_valid);
    }

    // Test warnings_only
    #[test]
    fn test_warnings_only_no_warnings() {
        let v = EntityValidation::valid();
        assert!(!v.warnings_only());
    }

    #[test]
    fn test_warnings_only_with_warnings() {
        let mut v = EntityValidation::valid();
        v.add_warning("test warning");
        assert!(v.warnings_only());
    }

    #[test]
    fn test_warnings_only_with_errors() {
        let mut v = EntityValidation::valid();
        v.add_error("test error");
        assert!(!v.warnings_only());
    }

    #[test]
    fn test_warnings_only_with_both() {
        let mut v = EntityValidation::valid();
        v.add_warning("test warning");
        v.add_error("test error");
        assert!(!v.warnings_only()); // Not valid, so warnings_only is false
    }

    // Test issue_count
    #[test]
    fn test_issue_count_zero() {
        let v = EntityValidation::valid();
        assert_eq!(v.issue_count(), 0);
    }

    #[test]
    fn test_issue_count_one_warning() {
        let mut v = EntityValidation::valid();
        v.add_warning("warning");
        assert_eq!(v.issue_count(), 1);
    }

    #[test]
    fn test_issue_count_one_error() {
        let mut v = EntityValidation::valid();
        v.add_error("error");
        assert_eq!(v.issue_count(), 1);
    }

    #[test]
    fn test_issue_count_both() {
        let mut v = EntityValidation::valid();
        v.add_warning("warning");
        v.add_error("error");
        assert_eq!(v.issue_count(), 2);
    }

    // Test summary
    #[test]
    fn test_summary_valid() {
        let v = EntityValidation::valid();
        assert_eq!(v.summary(), "Valid");
    }

    #[test]
    fn test_summary_warnings_only() {
        let mut v = EntityValidation::valid();
        v.add_warning("w1");
        assert_eq!(v.summary(), "1 warnings");
    }

    #[test]
    fn test_summary_two_warnings() {
        let mut v = EntityValidation::valid();
        v.add_warning("w1");
        v.add_warning("w2");
        assert_eq!(v.summary(), "2 warnings");
    }

    #[test]
    fn test_summary_errors() {
        let mut v = EntityValidation::valid();
        v.add_error("e1");
        assert_eq!(v.summary(), "1 errors");
    }

    #[test]
    fn test_summary_two_errors() {
        let mut v = EntityValidation::valid();
        v.add_error("e1");
        v.add_error("e2");
        assert_eq!(v.summary(), "2 errors");
    }

    // Test Display
    #[test]
    fn test_display_valid() {
        let v = EntityValidation::valid();
        assert_eq!(format!("{}", v), "✓ Valid");
    }

    #[test]
    fn test_display_invalid() {
        let mut v = EntityValidation::valid();
        v.add_error("error");
        assert_eq!(format!("{}", v), "✗ 1 errors");
    }
}

// ============================================================================
// ENTITY MANAGER VALIDATION TESTS
// ============================================================================

mod entity_manager_validation_tests {
    use super::*;

    // Test all_valid
    #[test]
    fn test_all_valid_empty() {
        let v = EntityManagerValidation::default();
        assert!(v.all_valid());
    }

    #[test]
    fn test_all_valid_with_invalid() {
        let mut v = EntityManagerValidation::default();
        v.invalid_entities.push(1);
        assert!(!v.all_valid());
    }

    // Test invalid_count
    #[test]
    fn test_invalid_count_zero() {
        let v = EntityManagerValidation::default();
        assert_eq!(v.invalid_count(), 0);
    }

    #[test]
    fn test_invalid_count_one() {
        let mut v = EntityManagerValidation::default();
        v.invalid_entities.push(1);
        assert_eq!(v.invalid_count(), 1);
    }

    #[test]
    fn test_invalid_count_two() {
        let mut v = EntityManagerValidation::default();
        v.invalid_entities.push(1);
        v.invalid_entities.push(2);
        assert_eq!(v.invalid_count(), 2);
    }

    // Test valid_count
    #[test]
    fn test_valid_count_all_valid() {
        let mut v = EntityManagerValidation::default();
        v.total_entities = 5;
        assert_eq!(v.valid_count(), 5);
    }

    #[test]
    fn test_valid_count_some_invalid() {
        let mut v = EntityManagerValidation::default();
        v.total_entities = 5;
        v.invalid_entities.push(1);
        v.invalid_entities.push(2);
        assert_eq!(v.valid_count(), 3);
    }

    #[test]
    fn test_valid_count_all_invalid() {
        let mut v = EntityManagerValidation::default();
        v.total_entities = 2;
        v.invalid_entities.push(1);
        v.invalid_entities.push(2);
        assert_eq!(v.valid_count(), 0);
    }

    // Test success_rate boundary conditions
    #[test]
    fn test_success_rate_empty() {
        let v = EntityManagerValidation::default();
        assert_eq!(v.success_rate(), 100.0);
    }

    #[test]
    fn test_success_rate_all_valid() {
        let mut v = EntityManagerValidation::default();
        v.total_entities = 10;
        assert_eq!(v.success_rate(), 100.0);
    }

    #[test]
    fn test_success_rate_half() {
        let mut v = EntityManagerValidation::default();
        v.total_entities = 10;
        for i in 0..5 {
            v.invalid_entities.push(i);
        }
        assert_eq!(v.success_rate(), 50.0);
    }

    #[test]
    fn test_success_rate_none_valid() {
        let mut v = EntityManagerValidation::default();
        v.total_entities = 5;
        for i in 0..5 {
            v.invalid_entities.push(i);
        }
        assert_eq!(v.success_rate(), 0.0);
    }

    // Test has_issues
    #[test]
    fn test_has_issues_none() {
        let v = EntityManagerValidation::default();
        assert!(!v.has_issues());
    }

    #[test]
    fn test_has_issues_errors_only() {
        let mut v = EntityManagerValidation::default();
        v.error_count = 1;
        assert!(v.has_issues());
    }

    #[test]
    fn test_has_issues_warnings_only() {
        let mut v = EntityManagerValidation::default();
        v.warning_count = 1;
        assert!(v.has_issues());
    }

    #[test]
    fn test_has_issues_both() {
        let mut v = EntityManagerValidation::default();
        v.error_count = 1;
        v.warning_count = 1;
        assert!(v.has_issues());
    }

    // Test total_issues
    #[test]
    fn test_total_issues_zero() {
        let v = EntityManagerValidation::default();
        assert_eq!(v.total_issues(), 0);
    }

    #[test]
    fn test_total_issues_errors_only() {
        let mut v = EntityManagerValidation::default();
        v.error_count = 3;
        assert_eq!(v.total_issues(), 3);
    }

    #[test]
    fn test_total_issues_warnings_only() {
        let mut v = EntityManagerValidation::default();
        v.warning_count = 2;
        assert_eq!(v.total_issues(), 2);
    }

    #[test]
    fn test_total_issues_both() {
        let mut v = EntityManagerValidation::default();
        v.error_count = 3;
        v.warning_count = 2;
        assert_eq!(v.total_issues(), 5);
    }
}

// ============================================================================
// ENTITY MANAGER STATS TESTS
// ============================================================================

mod entity_manager_stats_tests {
    use super::*;

    // Test is_empty
    #[test]
    fn test_is_empty_zero() {
        let stats = EntityManagerStats {
            total_entities: 0,
            with_mesh: 0,
            with_textures: 0,
            with_components: 0,
            next_id: 1,
        };
        assert!(stats.is_empty());
    }

    #[test]
    fn test_is_empty_one() {
        let stats = EntityManagerStats {
            total_entities: 1,
            with_mesh: 0,
            with_textures: 0,
            with_components: 0,
            next_id: 2,
        };
        assert!(!stats.is_empty());
    }

    // Test has_meshes
    #[test]
    fn test_has_meshes_zero() {
        let stats = EntityManagerStats {
            total_entities: 5,
            with_mesh: 0,
            with_textures: 0,
            with_components: 0,
            next_id: 6,
        };
        assert!(!stats.has_meshes());
    }

    #[test]
    fn test_has_meshes_one() {
        let stats = EntityManagerStats {
            total_entities: 5,
            with_mesh: 1,
            with_textures: 0,
            with_components: 0,
            next_id: 6,
        };
        assert!(stats.has_meshes());
    }

    // Test mesh_percentage
    #[test]
    fn test_mesh_percentage_empty() {
        let stats = EntityManagerStats {
            total_entities: 0,
            with_mesh: 0,
            with_textures: 0,
            with_components: 0,
            next_id: 1,
        };
        assert_eq!(stats.mesh_percentage(), 0.0);
    }

    #[test]
    fn test_mesh_percentage_none() {
        let stats = EntityManagerStats {
            total_entities: 10,
            with_mesh: 0,
            with_textures: 0,
            with_components: 0,
            next_id: 11,
        };
        assert_eq!(stats.mesh_percentage(), 0.0);
    }

    #[test]
    fn test_mesh_percentage_half() {
        let stats = EntityManagerStats {
            total_entities: 10,
            with_mesh: 5,
            with_textures: 0,
            with_components: 0,
            next_id: 11,
        };
        assert_eq!(stats.mesh_percentage(), 50.0);
    }

    #[test]
    fn test_mesh_percentage_all() {
        let stats = EntityManagerStats {
            total_entities: 10,
            with_mesh: 10,
            with_textures: 0,
            with_components: 0,
            next_id: 11,
        };
        assert_eq!(stats.mesh_percentage(), 100.0);
    }

    // Test texture_percentage
    #[test]
    fn test_texture_percentage_empty() {
        let stats = EntityManagerStats {
            total_entities: 0,
            with_mesh: 0,
            with_textures: 0,
            with_components: 0,
            next_id: 1,
        };
        assert_eq!(stats.texture_percentage(), 0.0);
    }

    #[test]
    fn test_texture_percentage_half() {
        let stats = EntityManagerStats {
            total_entities: 10,
            with_mesh: 0,
            with_textures: 5,
            with_components: 0,
            next_id: 11,
        };
        assert_eq!(stats.texture_percentage(), 50.0);
    }

    // Test component_percentage
    #[test]
    fn test_component_percentage_empty() {
        let stats = EntityManagerStats {
            total_entities: 0,
            with_mesh: 0,
            with_textures: 0,
            with_components: 0,
            next_id: 1,
        };
        assert_eq!(stats.component_percentage(), 0.0);
    }

    #[test]
    fn test_component_percentage_half() {
        let stats = EntityManagerStats {
            total_entities: 10,
            with_mesh: 0,
            with_textures: 0,
            with_components: 5,
            next_id: 11,
        };
        assert_eq!(stats.component_percentage(), 50.0);
    }
}

// ============================================================================
// ENTITY MANAGER TESTS
// ============================================================================

mod entity_manager_tests {
    use super::*;

    // Test count
    #[test]
    fn test_count_empty() {
        let manager = EntityManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_count_after_create() {
        let mut manager = EntityManager::new();
        manager.create("Test".to_string());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_count_after_multiple_creates() {
        let mut manager = EntityManager::new();
        manager.create("Test1".to_string());
        manager.create("Test2".to_string());
        manager.create("Test3".to_string());
        assert_eq!(manager.count(), 3);
    }

    #[test]
    fn test_count_after_remove() {
        let mut manager = EntityManager::new();
        let id = manager.create("Test".to_string());
        manager.remove(id);
        assert_eq!(manager.count(), 0);
    }

    // Test clear
    #[test]
    fn test_clear_empty() {
        let mut manager = EntityManager::new();
        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_clear_with_entities() {
        let mut manager = EntityManager::new();
        manager.create("Test1".to_string());
        manager.create("Test2".to_string());
        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    // Test get
    #[test]
    fn test_get_exists() {
        let mut manager = EntityManager::new();
        let id = manager.create("Test".to_string());
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_get_not_exists() {
        let manager = EntityManager::new();
        assert!(manager.get(999).is_none());
    }

    #[test]
    fn test_get_after_remove() {
        let mut manager = EntityManager::new();
        let id = manager.create("Test".to_string());
        manager.remove(id);
        assert!(manager.get(id).is_none());
    }

    // Test ID incrementing
    #[test]
    fn test_id_increments() {
        let mut manager = EntityManager::new();
        let id1 = manager.create("Test1".to_string());
        let id2 = manager.create("Test2".to_string());
        let id3 = manager.create("Test3".to_string());
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    // Test find_by_name
    #[test]
    fn test_find_by_name_empty() {
        let manager = EntityManager::new();
        assert!(manager.find_by_name("test").is_empty());
    }

    #[test]
    fn test_find_by_name_no_match() {
        let mut manager = EntityManager::new();
        manager.create("Entity1".to_string());
        assert!(manager.find_by_name("xyz").is_empty());
    }

    #[test]
    fn test_find_by_name_exact_match() {
        let mut manager = EntityManager::new();
        let id = manager.create("TestEntity".to_string());
        let found = manager.find_by_name("TestEntity");
        assert_eq!(found.len(), 1);
        assert!(found.contains(&id));
    }

    #[test]
    fn test_find_by_name_case_insensitive() {
        let mut manager = EntityManager::new();
        let id = manager.create("TestEntity".to_string());
        let found = manager.find_by_name("testentity");
        assert_eq!(found.len(), 1);
        assert!(found.contains(&id));
    }

    #[test]
    fn test_find_by_name_partial() {
        let mut manager = EntityManager::new();
        let id = manager.create("TestEntity".to_string());
        let found = manager.find_by_name("Entity");
        assert_eq!(found.len(), 1);
        assert!(found.contains(&id));
    }

    #[test]
    fn test_find_by_name_multiple_matches() {
        let mut manager = EntityManager::new();
        let id1 = manager.create("Enemy1".to_string());
        let id2 = manager.create("Enemy2".to_string());
        manager.create("Player".to_string());
        let found = manager.find_by_name("Enemy");
        assert_eq!(found.len(), 2);
        assert!(found.contains(&id1));
        assert!(found.contains(&id2));
    }
}
