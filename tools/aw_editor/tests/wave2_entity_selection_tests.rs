//! Wave 2 Mutation-Resistant Tests: Entity Manager, Selection & Material
//!
//! Targets mutation-prone patterns in entity_manager.rs:
//! - EditorEntity AABB half-size arithmetic (position ± half_size)
//! - EntityValidation scale boundary (<=0), NaN detection, rotation normalization
//! - EntityManager CRUD with ID auto-increment and reset
//! - EntityManager find_by_name case-insensitive matching
//! - EntityManager find_in_region boundary comparisons (>= and <=)
//! - SelectionSet toggle, range selection (start/end ordering)
//! - EntityMaterial threshold checks (metallic>0.5, roughness>0.5, emissive length)
//! - MaterialSlot color vs data classification
//! - EntityManagerValidation success_rate arithmetic
//! - EntityManagerStats percentage calculations and zero-division guards
//! - Display implementations for all types

use aw_editor_lib::entity_manager::{
    EditorEntity, EntityManager, EntityMaterial, MaterialSlot, SelectionSet,
};
use glam::{Quat, Vec3, Vec4};
use std::path::PathBuf;

// ============================================================================
// Section A: EditorEntity — AABB Arithmetic
// ============================================================================

#[test]
fn aabb_default_entity_unit_cube() {
    let entity = EditorEntity::new(1, "Test".to_string());
    // Default scale is (1,1,1), position (0,0,0)
    // half_size = (0.5, 0.5, 0.5)
    // min = (-0.5, -0.5, -0.5), max = (0.5, 0.5, 0.5)
    let (min, max) = entity.aabb();
    assert!((min.x - (-0.5)).abs() < 0.001);
    assert!((min.y - (-0.5)).abs() < 0.001);
    assert!((min.z - (-0.5)).abs() < 0.001);
    assert!((max.x - 0.5).abs() < 0.001);
    assert!((max.y - 0.5).abs() < 0.001);
    assert!((max.z - 0.5).abs() < 0.001);
}

#[test]
fn aabb_scaled_entity() {
    let mut entity = EditorEntity::new(1, "Scaled".to_string());
    entity.scale = Vec3::new(4.0, 2.0, 6.0);
    // half_size = (2.0, 1.0, 3.0)
    // min = (-2, -1, -3), max = (2, 1, 3)
    let (min, max) = entity.aabb();
    assert!((min.x - (-2.0)).abs() < 0.001);
    assert!((min.y - (-1.0)).abs() < 0.001);
    assert!((min.z - (-3.0)).abs() < 0.001);
    assert!((max.x - 2.0).abs() < 0.001);
    assert!((max.y - 1.0).abs() < 0.001);
    assert!((max.z - 3.0).abs() < 0.001);
}

#[test]
fn aabb_offset_position() {
    let mut entity = EditorEntity::new(1, "Offset".to_string());
    entity.position = Vec3::new(10.0, 20.0, 30.0);
    entity.scale = Vec3::new(2.0, 2.0, 2.0);
    // half_size = (1, 1, 1)
    let (min, max) = entity.aabb();
    assert!((min.x - 9.0).abs() < 0.001);
    assert!((min.y - 19.0).abs() < 0.001);
    assert!((min.z - 29.0).abs() < 0.001);
    assert!((max.x - 11.0).abs() < 0.001);
    assert!((max.y - 21.0).abs() < 0.001);
    assert!((max.z - 31.0).abs() < 0.001);
}

#[test]
fn aabb_min_always_less_than_max() {
    let mut entity = EditorEntity::new(1, "Check".to_string());
    entity.position = Vec3::new(-5.0, 3.0, 0.0);
    entity.scale = Vec3::splat(0.1);

    let (min, max) = entity.aabb();
    assert!(min.x < max.x);
    assert!(min.y < max.y);
    assert!(min.z < max.z);
}

// ============================================================================
// Section B: EntityValidation — Boundary Conditions
// ============================================================================

#[test]
fn validation_valid_entity_passes() {
    let entity = EditorEntity::new(1, "Valid".to_string());
    let v = entity.validate();
    assert!(v.is_valid);
    assert!(v.errors.is_empty());
    assert!(v.warnings.is_empty());
    assert!(entity.is_valid());
}

#[test]
fn validation_empty_name_is_warning_not_error() {
    let entity = EditorEntity::new(1, "".to_string());
    let v = entity.validate();
    assert!(v.is_valid); // Warning, not error
    assert!(!v.warnings.is_empty());
    assert!(v.warnings[0].to_lowercase().contains("empty"));
}

#[test]
fn validation_long_name_over_256_is_error() {
    let long_name = "x".repeat(300);
    let entity = EditorEntity::new(1, long_name);
    let v = entity.validate();
    assert!(!v.is_valid);
    assert!(v.errors.iter().any(|e| e.contains("256")));
}

#[test]
fn validation_name_exactly_256_is_ok() {
    let name = "x".repeat(256);
    let entity = EditorEntity::new(1, name);
    let v = entity.validate();
    // 256 should not trigger the >256 check
    assert!(!v.errors.iter().any(|e| e.to_lowercase().contains("length")));
}

#[test]
fn validation_zero_scale_x_is_error() {
    let mut entity = EditorEntity::new(1, "Bad".to_string());
    entity.scale = Vec3::new(0.0, 1.0, 1.0);
    assert!(!entity.is_valid());
    assert!(entity
        .validate()
        .errors
        .iter()
        .any(|e| e.to_lowercase().contains("scale")));
}

#[test]
fn validation_negative_scale_y_is_error() {
    let mut entity = EditorEntity::new(1, "Bad".to_string());
    entity.scale = Vec3::new(1.0, -0.5, 1.0);
    assert!(!entity.is_valid());
}

#[test]
fn validation_negative_scale_z_is_error() {
    let mut entity = EditorEntity::new(1, "Bad".to_string());
    entity.scale = Vec3::new(1.0, 1.0, -0.001);
    assert!(!entity.is_valid());
}

#[test]
fn validation_very_small_positive_scale_is_ok() {
    let mut entity = EditorEntity::new(1, "Tiny".to_string());
    entity.scale = Vec3::splat(0.001);
    assert!(entity.is_valid());
}

#[test]
fn validation_nan_position_x_is_error() {
    let mut entity = EditorEntity::new(1, "NaN".to_string());
    entity.position = Vec3::new(f32::NAN, 0.0, 0.0);
    let v = entity.validate();
    assert!(!v.is_valid);
    assert!(v.errors.iter().any(|e| e.contains("Position")));
}

#[test]
fn validation_inf_position_y_is_error() {
    let mut entity = EditorEntity::new(1, "Inf".to_string());
    entity.position = Vec3::new(0.0, f32::INFINITY, 0.0);
    assert!(!entity.is_valid());
}

#[test]
fn validation_nan_rotation_is_error() {
    let mut entity = EditorEntity::new(1, "NaN".to_string());
    entity.rotation = Quat::from_xyzw(f32::NAN, 0.0, 0.0, 1.0);
    let v = entity.validate();
    assert!(!v.is_valid);
    assert!(v.errors.iter().any(|e| e.contains("Rotation")));
}

#[test]
fn validation_unnormalized_quaternion_is_warning() {
    let mut entity = EditorEntity::new(1, "Unnorm".to_string());
    entity.rotation = Quat::from_xyzw(1.0, 1.0, 1.0, 1.0); // length 2
    let v = entity.validate();
    assert!(v.is_valid); // Warning only
    assert!(v.warnings.iter().any(|w| w.contains("normalized")));
}

#[test]
fn validation_normalized_quaternion_no_warning() {
    let entity = EditorEntity::new(1, "Normal".to_string());
    // Default is IDENTITY which is normalized
    let v = entity.validate();
    assert!(!v.warnings.iter().any(|w| w.contains("normalized")));
}

#[test]
fn validation_issue_count_sums_errors_and_warnings() {
    let mut entity = EditorEntity::new(1, "".to_string()); // warning: empty name
    entity.scale = Vec3::ZERO; // error: zero scale
    let v = entity.validate();
    assert!(v.issue_count() >= 2);
}

#[test]
fn validation_warnings_only_true_when_valid_with_warnings() {
    let entity = EditorEntity::new(1, "".to_string());
    let v = entity.validate();
    assert!(v.warnings_only()); // valid=true, has warning
}

#[test]
fn validation_warnings_only_false_when_errors() {
    let mut entity = EditorEntity::new(1, "Bad".to_string());
    entity.scale = Vec3::ZERO;
    let v = entity.validate();
    assert!(!v.warnings_only()); // has errors
}

#[test]
fn validation_summary_valid() {
    let entity = EditorEntity::new(1, "Good".to_string());
    assert_eq!(entity.validate().summary(), "Valid");
}

#[test]
fn validation_summary_with_warnings() {
    let entity = EditorEntity::new(1, "".to_string());
    let s = entity.validate().summary();
    assert!(s.to_lowercase().contains("warning"));
}

#[test]
fn validation_summary_with_errors() {
    let mut entity = EditorEntity::new(1, "X".to_string());
    entity.scale = Vec3::ZERO;
    let s = entity.validate().summary();
    assert!(s.to_lowercase().contains("error"));
}

#[test]
fn validation_display_valid() {
    let entity = EditorEntity::new(1, "Good".to_string());
    let display = format!("{}", entity.validate());
    assert!(display.contains("Valid") || display.contains("✓"));
}

#[test]
fn validation_display_invalid() {
    let mut entity = EditorEntity::new(1, "X".to_string());
    entity.scale = Vec3::ZERO;
    let display = format!("{}", entity.validate());
    assert!(display.contains("error") || display.contains("✗"));
}

// ============================================================================
// Section C: EntityManager — CRUD & ID Management
// ============================================================================

#[test]
fn manager_create_auto_increments_ids() {
    let mut mgr = EntityManager::new();
    let id1 = mgr.create("A".to_string());
    let id2 = mgr.create("B".to_string());
    let id3 = mgr.create("C".to_string());
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
fn manager_get_returns_correct_entity() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("TestEntity".to_string());
    let entity = mgr.get(id).unwrap();
    assert_eq!(entity.name, "TestEntity");
    assert_eq!(entity.id, id);
}

#[test]
fn manager_get_nonexistent_returns_none() {
    let mgr = EntityManager::new();
    assert!(mgr.get(999).is_none());
}

#[test]
fn manager_remove_returns_entity_and_removes() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Temporary".to_string());
    assert_eq!(mgr.count(), 1);

    let removed = mgr.remove(id).unwrap();
    assert_eq!(removed.name, "Temporary");
    assert_eq!(mgr.count(), 0);
    assert!(mgr.get(id).is_none());
}

#[test]
fn manager_remove_nonexistent_returns_none() {
    let mut mgr = EntityManager::new();
    assert!(mgr.remove(999).is_none());
}

#[test]
fn manager_clear_resets_count_and_next_id() {
    let mut mgr = EntityManager::new();
    mgr.create("A".to_string());
    mgr.create("B".to_string());
    assert_eq!(mgr.count(), 2);

    mgr.clear();
    assert_eq!(mgr.count(), 0);

    // Next ID should reset to 1
    let id = mgr.create("After".to_string());
    assert_eq!(id, 1);
}

#[test]
fn manager_add_adjusts_next_id() {
    let mut mgr = EntityManager::new();
    let entity = EditorEntity::new(100, "External".to_string());
    mgr.add(entity);

    // Next id should be > 100
    let id = mgr.create("After".to_string());
    assert!(id > 100);
}

#[test]
fn manager_update_position_only_changes_position() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Entity".to_string());
    let initial_rot = mgr.get(id).unwrap().rotation;

    mgr.update_position(id, Vec3::new(10.0, 20.0, 30.0));
    let e = mgr.get(id).unwrap();
    assert_eq!(e.position, Vec3::new(10.0, 20.0, 30.0));
    assert_eq!(e.rotation, initial_rot); // Rotation unchanged
}

#[test]
fn manager_update_rotation_only_changes_rotation() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Entity".to_string());
    let initial_pos = mgr.get(id).unwrap().position;

    let rot = Quat::from_rotation_y(1.57);
    mgr.update_rotation(id, rot);
    let e = mgr.get(id).unwrap();
    assert_eq!(e.position, initial_pos); // Position unchanged
    assert!((e.rotation.y - rot.y).abs() < 0.001);
}

#[test]
fn manager_update_scale_only_changes_scale() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Entity".to_string());

    mgr.update_scale(id, Vec3::splat(2.0));
    assert_eq!(mgr.get(id).unwrap().scale, Vec3::splat(2.0));
}

#[test]
fn manager_update_transform_changes_all() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Entity".to_string());

    let pos = Vec3::new(1.0, 2.0, 3.0);
    let rot = Quat::from_rotation_z(0.5);
    let scale = Vec3::new(2.0, 3.0, 4.0);
    mgr.update_transform(id, pos, rot, scale);

    let (p, r, s) = mgr.get(id).unwrap().transform();
    assert_eq!(p, pos);
    assert!((r.z - rot.z).abs() < 0.001);
    assert_eq!(s, scale);
}

// ============================================================================
// Section D: EntityManager — find_by_name (Case Insensitive)
// ============================================================================

#[test]
fn find_by_name_exact_match() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Player".to_string());
    mgr.create("Enemy".to_string());

    let results = mgr.find_by_name("Player");
    assert_eq!(results.len(), 1);
    assert!(results.contains(&id));
}

#[test]
fn find_by_name_case_insensitive() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("PlayerCharacter".to_string());

    assert_eq!(mgr.find_by_name("playercharacter").len(), 1);
    assert_eq!(mgr.find_by_name("PLAYERCHARACTER").len(), 1);
    assert_eq!(mgr.find_by_name("PlAyErChArAcTeR").len(), 1);
    assert!(mgr.find_by_name("player").first().copied() == Some(id));
}

#[test]
fn find_by_name_partial_match() {
    let mut mgr = EntityManager::new();
    mgr.create("Enemy_Soldier".to_string());
    mgr.create("Enemy_Tank".to_string());
    mgr.create("Player".to_string());

    let results = mgr.find_by_name("Enemy");
    assert_eq!(results.len(), 2);
}

#[test]
fn find_by_name_no_match() {
    let mut mgr = EntityManager::new();
    mgr.create("Entity".to_string());

    assert!(mgr.find_by_name("NotHere").is_empty());
}

// ============================================================================
// Section E: EntityManager — find_in_region (Boundary Comparisons)
// ============================================================================

#[test]
fn find_in_region_includes_boundary() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("OnBorder".to_string());
    mgr.update_position(id, Vec3::new(10.0, 10.0, 10.0));

    // Region boundary exactly at entity position
    let results = mgr.find_in_region(Vec3::splat(10.0), Vec3::splat(10.0));
    assert_eq!(results.len(), 1);
    assert!(results.contains(&id));
}

#[test]
fn find_in_region_excludes_outside() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Outside".to_string());
    mgr.update_position(id, Vec3::new(15.0, 5.0, 5.0));

    let results = mgr.find_in_region(Vec3::ZERO, Vec3::splat(10.0));
    assert!(!results.contains(&id)); // x=15 > 10
}

#[test]
fn find_in_region_boundary_all_axes() {
    let mut mgr = EntityManager::new();
    let id1 = mgr.create("Min".to_string());
    let id2 = mgr.create("Max".to_string());
    mgr.update_position(id1, Vec3::ZERO);
    mgr.update_position(id2, Vec3::splat(100.0));

    let results = mgr.find_in_region(Vec3::ZERO, Vec3::splat(100.0));
    assert!(results.contains(&id1));
    assert!(results.contains(&id2));
}

#[test]
fn find_in_region_just_outside_x() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("JustOut".to_string());
    mgr.update_position(id, Vec3::new(10.001, 5.0, 5.0));

    let results = mgr.find_in_region(Vec3::ZERO, Vec3::splat(10.0));
    assert!(!results.contains(&id));
}

#[test]
fn find_in_region_just_outside_negative() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("Below".to_string());
    mgr.update_position(id, Vec3::new(5.0, -0.001, 5.0));

    let results = mgr.find_in_region(Vec3::ZERO, Vec3::splat(10.0));
    assert!(!results.contains(&id));
}

// ============================================================================
// Section F: EntityManager — Validate All
// ============================================================================

#[test]
fn validate_all_empty_manager() {
    let mgr = EntityManager::new();
    let v = mgr.validate_all();
    assert_eq!(v.total_entities, 0);
    assert!(v.all_valid());
    assert_eq!(v.success_rate(), 100.0);
}

#[test]
fn validate_all_all_valid() {
    let mut mgr = EntityManager::new();
    mgr.create("A".to_string());
    mgr.create("B".to_string());

    let v = mgr.validate_all();
    assert!(v.all_valid());
    assert_eq!(v.invalid_count(), 0);
    assert_eq!(v.valid_count(), 2);
    assert!((v.success_rate() - 100.0).abs() < 0.1);
}

#[test]
fn validate_all_with_one_invalid() {
    let mut mgr = EntityManager::new();
    mgr.create("Good".to_string());
    let bad_id = mgr.create("Bad".to_string());
    mgr.get_mut(bad_id).unwrap().scale = Vec3::ZERO;

    let v = mgr.validate_all();
    assert!(!v.all_valid());
    assert_eq!(v.invalid_count(), 1);
    assert_eq!(v.valid_count(), 1);
    assert!((v.success_rate() - 50.0).abs() < 0.1);
}

#[test]
fn validate_all_has_issues_with_warnings() {
    let mut mgr = EntityManager::new();
    mgr.create("".to_string()); // empty name = warning

    let v = mgr.validate_all();
    assert!(v.has_issues());
    assert!(v.warning_count > 0);
    assert!(v.total_issues() > 0);
}

// ============================================================================
// Section G: EntityManagerStats — Percentage Arithmetic
// ============================================================================

#[test]
fn stats_empty_manager_all_zeros() {
    let mgr = EntityManager::new();
    let s = mgr.stats();
    assert_eq!(s.total_entities, 0);
    assert!(s.is_empty());
    assert_eq!(s.mesh_percentage(), 0.0);
    assert_eq!(s.texture_percentage(), 0.0);
    assert_eq!(s.component_percentage(), 0.0);
}

#[test]
fn stats_mesh_percentage_calculation() {
    let mut mgr = EntityManager::new();
    let id1 = mgr.create("WithMesh".to_string());
    mgr.get_mut(id1).unwrap().mesh = Some("mesh.obj".to_string());
    mgr.create("NoMesh".to_string());

    let s = mgr.stats();
    assert!((s.mesh_percentage() - 50.0).abs() < 0.1);
    assert!(s.has_meshes());
}

#[test]
fn stats_texture_percentage_calculation() {
    let mut mgr = EntityManager::new();
    let id = mgr.create("WithTex".to_string());
    mgr.get_mut(id)
        .unwrap()
        .set_texture(MaterialSlot::Albedo, PathBuf::from("t.png"));
    mgr.create("NoTex1".to_string());
    mgr.create("NoTex2".to_string());
    mgr.create("NoTex3".to_string());

    let s = mgr.stats();
    assert!((s.texture_percentage() - 25.0).abs() < 0.1);
}

#[test]
fn stats_component_percentage_calculation() {
    let mut mgr = EntityManager::new();
    let id1 = mgr.create("WithComp".to_string());
    let id2 = mgr.create("WithComp2".to_string());
    mgr.create("NoComp".to_string());

    mgr.get_mut(id1)
        .unwrap()
        .components
        .insert("Health".to_string(), serde_json::json!(100));
    mgr.get_mut(id2)
        .unwrap()
        .components
        .insert("AI".to_string(), serde_json::json!(true));

    let s = mgr.stats();
    assert!((s.component_percentage() - 66.66).abs() < 1.0);
}

#[test]
fn stats_next_id_reflects_current() {
    let mut mgr = EntityManager::new();
    mgr.create("E1".to_string());
    mgr.create("E2".to_string());
    mgr.create("E3".to_string());

    assert_eq!(mgr.stats().next_id, 4);
}

// ============================================================================
// Section H: SelectionSet — Core Operations
// ============================================================================

#[test]
fn selection_empty_initially() {
    let sel = SelectionSet::new();
    assert!(sel.is_empty());
    assert_eq!(sel.count(), 0);
    assert!(sel.primary.is_none());
    assert!(!sel.is_multi_select());
}

#[test]
fn selection_add_as_primary() {
    let mut sel = SelectionSet::new();
    sel.add(1, true);

    assert!(sel.is_selected(1));
    assert_eq!(sel.primary, Some(1));
    assert!(sel.is_primary(1));
    assert!(!sel.is_empty());
}

#[test]
fn selection_add_not_primary() {
    let mut sel = SelectionSet::new();
    sel.add(1, true);
    sel.add(2, false);

    assert!(sel.is_selected(2));
    assert_eq!(sel.primary, Some(1)); // Primary unchanged
    assert!(!sel.is_primary(2));
}

#[test]
fn selection_toggle_adds_and_removes() {
    let mut sel = SelectionSet::new();

    sel.toggle(1);
    assert!(sel.is_selected(1));

    sel.toggle(1);
    assert!(!sel.is_selected(1));
    assert!(sel.is_empty());
}

#[test]
fn selection_remove_updates_primary() {
    let mut sel = SelectionSet::new();
    sel.add(1, true); // primary
    sel.add(2, false);

    sel.remove(1);
    assert!(!sel.is_selected(1));
    // Primary should shift to remaining entity
    assert!(sel.primary.is_some());
}

#[test]
fn selection_select_only_clears_others() {
    let mut sel = SelectionSet::new();
    sel.add(1, true);
    sel.add(2, false);
    sel.add(3, false);

    sel.select_only(5);
    assert_eq!(sel.count(), 1);
    assert!(sel.is_selected(5));
    assert!(!sel.is_selected(1));
    assert!(!sel.is_selected(2));
    assert!(!sel.is_selected(3));
    assert_eq!(sel.primary, Some(5));
}

#[test]
fn selection_multi_select_detection() {
    let mut sel = SelectionSet::new();
    sel.add(1, true);
    assert!(!sel.is_multi_select());

    sel.add(2, false);
    assert!(sel.is_multi_select());
}

#[test]
fn selection_to_vec_contains_all() {
    let mut sel = SelectionSet::new();
    sel.add(10, true);
    sel.add(20, false);
    sel.add(30, false);

    let v = sel.to_vec();
    assert_eq!(v.len(), 3);
    assert!(v.contains(&10));
    assert!(v.contains(&20));
    assert!(v.contains(&30));
}

#[test]
fn selection_summary_messages() {
    let mut sel = SelectionSet::new();
    assert_eq!(sel.summary(), "No selection");

    sel.add(1, true);
    assert!(sel.summary().contains("1"));

    sel.add(2, false);
    assert!(sel.summary().contains("2"));
}

#[test]
fn selection_display_formatting() {
    let mut sel = SelectionSet::new();
    assert_eq!(format!("{}", sel), "(none)");

    sel.add(1, true);
    assert!(format!("{}", sel).contains("1"));

    sel.add(2, false);
    assert!(format!("{}", sel).contains("2"));
}

// ============================================================================
// Section I: SelectionSet — Range Selection
// ============================================================================

#[test]
fn select_range_forward() {
    let mut sel = SelectionSet::new();
    let all_ids: Vec<u64> = (1..=10).collect();

    sel.select_range(3, 7, &all_ids);
    assert_eq!(sel.count(), 5); // 3, 4, 5, 6, 7
    for id in 3..=7 {
        assert!(sel.is_selected(id));
    }
    assert!(!sel.is_selected(2));
    assert!(!sel.is_selected(8));
    assert_eq!(sel.primary, Some(7));
}

#[test]
fn select_range_reverse() {
    let mut sel = SelectionSet::new();
    let all_ids: Vec<u64> = (1..=10).collect();

    sel.select_range(7, 3, &all_ids);
    assert_eq!(sel.count(), 5); // 3, 4, 5, 6, 7
    assert_eq!(sel.primary, Some(3));
}

#[test]
fn select_range_single_element() {
    let mut sel = SelectionSet::new();
    let all_ids: Vec<u64> = (1..=5).collect();

    sel.select_range(3, 3, &all_ids);
    assert_eq!(sel.count(), 1);
    assert!(sel.is_selected(3));
    assert_eq!(sel.primary, Some(3));
}

#[test]
fn select_range_invalid_from_id_does_nothing() {
    let mut sel = SelectionSet::new();
    let all_ids: Vec<u64> = (1..=5).collect();

    sel.select_range(99, 3, &all_ids);
    assert!(sel.is_empty());
}

#[test]
fn select_range_invalid_to_id_does_nothing() {
    let mut sel = SelectionSet::new();
    let all_ids: Vec<u64> = (1..=5).collect();

    sel.select_range(1, 99, &all_ids);
    assert!(sel.is_empty());
}

// ============================================================================
// Section J: EntityMaterial — Threshold Checks
// ============================================================================

#[test]
fn material_default_values() {
    let mat = EntityMaterial::new();
    assert_eq!(mat.name, "Default");
    assert!(!mat.has_textures());
    assert_eq!(mat.texture_count(), 0);
    assert_eq!(mat.base_color, Vec4::ONE);
    assert_eq!(mat.metallic, 0.0);
    assert!((mat.roughness - 0.5).abs() < 0.001);
    assert_eq!(mat.emissive, Vec3::ZERO);
    assert!((mat.normal_strength - 1.0).abs() < 0.001);
}

#[test]
fn material_is_metallic_threshold_at_half() {
    let mut mat = EntityMaterial::new();

    mat.metallic = 0.5; // Exactly 0.5 should NOT be metallic (> 0.5 required)
    assert!(!mat.is_metallic());

    mat.metallic = 0.501;
    assert!(mat.is_metallic());

    mat.metallic = 0.0;
    assert!(!mat.is_metallic());

    mat.metallic = 1.0;
    assert!(mat.is_metallic());
}

#[test]
fn material_is_rough_threshold_at_half() {
    let mut mat = EntityMaterial::new();

    mat.roughness = 0.5; // Default — exactly 0.5 should NOT be rough
    assert!(!mat.is_rough());

    mat.roughness = 0.501;
    assert!(mat.is_rough());

    mat.roughness = 0.0;
    assert!(!mat.is_rough());

    mat.roughness = 1.0;
    assert!(mat.is_rough());
}

#[test]
fn material_is_emissive_threshold() {
    let mut mat = EntityMaterial::new();
    assert!(!mat.is_emissive()); // (0,0,0) → length² = 0

    mat.emissive = Vec3::new(0.001, 0.001, 0.001);
    // length² = 0.000003 — below 0.001 threshold
    assert!(!mat.is_emissive());

    mat.emissive = Vec3::new(0.1, 0.0, 0.0);
    // length² = 0.01 — above 0.001
    assert!(mat.is_emissive());
}

#[test]
fn material_texture_crud() {
    let mut mat = EntityMaterial::new();
    let path = PathBuf::from("texture.png");

    mat.set_texture(MaterialSlot::Albedo, path.clone());
    assert!(mat.has_textures());
    assert_eq!(mat.texture_count(), 1);
    assert_eq!(mat.get_texture(MaterialSlot::Albedo), Some(&path));

    mat.clear_texture(MaterialSlot::Albedo);
    assert!(!mat.has_textures());
    assert_eq!(mat.texture_count(), 0);
    assert!(mat.get_texture(MaterialSlot::Albedo).is_none());
}

#[test]
fn material_summary_contains_name_and_texture_count() {
    let mut mat = EntityMaterial::new();
    mat.set_texture(MaterialSlot::Normal, PathBuf::from("n.png"));
    mat.set_texture(MaterialSlot::Albedo, PathBuf::from("a.png"));

    let summary = mat.summary();
    assert!(summary.contains("Default"));
    assert!(summary.contains("2")); // 2 textures
}

#[test]
fn material_display_contains_name() {
    let mat = EntityMaterial::new();
    let display = format!("{}", mat);
    assert!(display.contains("Default"));
}

// ============================================================================
// Section K: MaterialSlot — Classification
// ============================================================================

#[test]
fn material_slot_all_has_eight() {
    assert_eq!(MaterialSlot::all().len(), 8);
}

#[test]
fn material_slot_color_vs_data() {
    // Color slots: Albedo, Emission
    assert!(MaterialSlot::Albedo.is_color_slot());
    assert!(MaterialSlot::Emission.is_color_slot());
    assert!(!MaterialSlot::Albedo.is_data_slot());
    assert!(!MaterialSlot::Emission.is_data_slot());

    // Data slots: everything else
    assert!(MaterialSlot::Normal.is_data_slot());
    assert!(MaterialSlot::Roughness.is_data_slot());
    assert!(MaterialSlot::Metallic.is_data_slot());
    assert!(MaterialSlot::AO.is_data_slot());
    assert!(MaterialSlot::ORM.is_data_slot());
    assert!(MaterialSlot::Height.is_data_slot());

    assert!(!MaterialSlot::Normal.is_color_slot());
    assert!(!MaterialSlot::Roughness.is_color_slot());
    assert!(!MaterialSlot::Metallic.is_color_slot());
}

#[test]
fn material_slot_names_unique_and_nonempty() {
    let all = MaterialSlot::all();
    for slot in all {
        assert!(!slot.name().is_empty());
    }
    // No duplicates
    let names: Vec<&str> = all.iter().map(|s| s.name()).collect();
    let unique: std::collections::HashSet<&&str> = names.iter().collect();
    assert_eq!(names.len(), unique.len());
}

#[test]
fn material_slot_icons_nonempty() {
    for slot in MaterialSlot::all() {
        assert!(!slot.icon().is_empty());
    }
}

#[test]
fn material_slot_display_matches_name() {
    for slot in MaterialSlot::all() {
        assert_eq!(format!("{}", slot), slot.name());
    }
}

// ============================================================================
// Section L: EditorEntity — Transform & Mesh
// ============================================================================

#[test]
fn entity_set_transform() {
    let mut entity = EditorEntity::new(1, "E".to_string());
    let pos = Vec3::new(5.0, 10.0, 15.0);
    let rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
    let scale = Vec3::new(2.0, 3.0, 4.0);

    entity.set_transform(pos, rot, scale);

    let (p, r, s) = entity.transform();
    assert_eq!(p, pos);
    assert!((r.y - rot.y).abs() < 0.001);
    assert_eq!(s, scale);
}

#[test]
fn entity_set_mesh() {
    let mut entity = EditorEntity::new(1, "E".to_string());
    assert!(entity.mesh.is_none());

    entity.set_mesh("cube.obj".to_string());
    assert_eq!(entity.mesh.as_deref(), Some("cube.obj"));
}

#[test]
fn entity_set_material() {
    let mut entity = EditorEntity::new(1, "E".to_string());
    let mut custom = EntityMaterial::new();
    custom.name = "Steel".to_string();
    custom.metallic = 0.9;

    entity.set_material(custom);
    assert_eq!(entity.material.name, "Steel");
    assert!((entity.material.metallic - 0.9).abs() < 0.001);
}

#[test]
fn entity_components_crud() {
    let mut entity = EditorEntity::new(1, "E".to_string());
    assert!(entity.components.is_empty());

    entity
        .components
        .insert("Health".to_string(), serde_json::json!({"value": 100}));
    assert_eq!(entity.components.len(), 1);
    assert!(entity.components.contains_key("Health"));

    entity.components.remove("Health");
    assert!(entity.components.is_empty());
}
