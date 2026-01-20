//! Panel system tests
//!
//! Tests panel visibility, state, and UI flags

#![allow(clippy::field_reassign_with_default)]

use astraweave_ui::{Accessibility, UiFlags};

// ===== Panel Visibility Tests =====

#[test]
fn test_panel_visibility_toggle_inventory() {
    let mut flags = UiFlags::default();

    assert!(!flags.show_inventory);

    // Toggle on
    flags.show_inventory = true;
    assert!(flags.show_inventory);

    // Toggle off
    flags.show_inventory = false;
    assert!(!flags.show_inventory);
}

#[test]
fn test_panel_visibility_toggle_crafting() {
    let mut flags = UiFlags::default();

    flags.show_crafting = true;
    assert!(flags.show_crafting);

    flags.show_crafting = false;
    assert!(!flags.show_crafting);
}

#[test]
fn test_panel_visibility_multiple_panels() {
    let mut flags = UiFlags::default();

    // Open multiple panels simultaneously
    flags.show_inventory = true;
    flags.show_map = true;
    flags.show_quests = true;

    assert!(flags.show_inventory);
    assert!(flags.show_map);
    assert!(flags.show_quests);
}

#[test]
fn test_panel_visibility_all_panels() {
    let mut flags = UiFlags::default();

    // Test all panel types
    flags.show_menu = true;
    flags.show_inventory = true;
    flags.show_crafting = true;
    flags.show_map = true;
    flags.show_quests = true;
    flags.show_settings = true;

    assert!(flags.show_menu);
    assert!(flags.show_inventory);
    assert!(flags.show_crafting);
    assert!(flags.show_map);
    assert!(flags.show_quests);
    assert!(flags.show_settings);
}

// ===== Panel State Persistence =====

#[test]
fn test_panel_state_default_initialization() {
    let flags = UiFlags::default();

    // All panels should be closed by default
    assert!(!flags.show_menu);
    assert!(!flags.show_inventory);
    assert!(!flags.show_crafting);
    assert!(!flags.show_map);
    assert!(!flags.show_quests);
    assert!(!flags.show_settings);
}

#[test]
fn test_panel_state_persistence() {
    let mut flags = UiFlags::default();

    flags.show_inventory = true;
    flags.show_map = true;

    // State should persist
    assert!(flags.show_inventory);
    assert!(flags.show_map);

    // Other panels should remain closed
    assert!(!flags.show_crafting);
    assert!(!flags.show_quests);
}

// ===== Accessibility Settings Tests =====

#[test]
fn test_accessibility_high_contrast() {
    let mut acc = Accessibility::default();

    assert!(!acc.high_contrast_ui);

    acc.high_contrast_ui = true;
    assert!(acc.high_contrast_ui);
}

#[test]
fn test_accessibility_reduce_motion() {
    let mut acc = Accessibility::default();

    assert!(!acc.reduce_motion);

    acc.reduce_motion = true;
    assert!(acc.reduce_motion);
}

#[test]
fn test_accessibility_subtitles() {
    let mut acc = Accessibility::default();

    acc.subtitles = true;
    assert!(acc.subtitles);
}

#[test]
fn test_accessibility_subtitle_scale() {
    let mut acc = Accessibility::default();

    acc.subtitle_scale = 1.5;
    assert_eq!(acc.subtitle_scale, 1.5);

    // Test valid range
    acc.subtitle_scale = 0.6;
    assert_eq!(acc.subtitle_scale, 0.6);

    acc.subtitle_scale = 1.8;
    assert_eq!(acc.subtitle_scale, 1.8);
}

#[test]
fn test_accessibility_colorblind_modes() {
    let mut acc = Accessibility::default();

    assert_eq!(acc.colorblind_mode, None);

    // Set protanopia
    acc.colorblind_mode = Some("protanopia".to_string());
    assert_eq!(acc.colorblind_mode, Some("protanopia".to_string()));

    // Set deuteranopia
    acc.colorblind_mode = Some("deuteranopia".to_string());
    assert_eq!(acc.colorblind_mode, Some("deuteranopia".to_string()));

    // Set tritanopia
    acc.colorblind_mode = Some("tritanopia".to_string());
    assert_eq!(acc.colorblind_mode, Some("tritanopia".to_string()));

    // Clear mode
    acc.colorblind_mode = None;
    assert_eq!(acc.colorblind_mode, None);
}
