//! Wave 2 Mutation Remediation: game_project.rs + editor_preferences.rs
//!
//! These integration tests complement the existing unit tests with:
//!   • GameProjectError — all 4 variants × Display/category/is_*/message
//!   • AssetSettings — has_high_compression boundary (>=10), pattern_count,
//!     compression_summary format branches, has_excludes
//!   • GameProject — validate() boundaries (empty name / empty entry_scene),
//!     targets_platform, has_platform_config for all 4 + unknown,
//!     summary format, has_icon, has_identifier
//!   • EditorPreferences — default field values, auto_save_keep_count

use aw_editor_lib::editor_preferences::EditorPreferences;
use aw_editor_lib::game_project::{
    AssetSettings, GameProject, GameProjectError, PlatformConfig,
};

// ============================================================================
// GameProjectError — Display
// ============================================================================

#[test]
fn game_project_error_display_io() {
    let e = GameProjectError::Io("disk full".to_string());
    let s = e.to_string();
    assert!(s.contains("IO error"), "got: {s}");
    assert!(s.contains("disk full"));
}

#[test]
fn game_project_error_display_parse() {
    let e = GameProjectError::Parse("unexpected token".to_string());
    let s = e.to_string();
    assert!(s.contains("Parse error"));
    assert!(s.contains("unexpected token"));
}

#[test]
fn game_project_error_display_serialize() {
    let e = GameProjectError::Serialize("bad field".to_string());
    let s = e.to_string();
    assert!(s.contains("Serialize error"));
    assert!(s.contains("bad field"));
}

#[test]
fn game_project_error_display_validation() {
    let e = GameProjectError::Validation(vec!["err1".to_string(), "err2".to_string()]);
    let s = e.to_string();
    assert!(s.contains("Validation errors"));
    assert!(s.contains("err1"));
    assert!(s.contains("err2"));
}

// ============================================================================
// GameProjectError — category()
// ============================================================================

#[test]
fn game_project_error_category_io() {
    assert_eq!(GameProjectError::Io("x".into()).category(), "IO");
}

#[test]
fn game_project_error_category_parse() {
    assert_eq!(GameProjectError::Parse("x".into()).category(), "Parse");
}

#[test]
fn game_project_error_category_serialize() {
    assert_eq!(
        GameProjectError::Serialize("x".into()).category(),
        "Serialize"
    );
}

#[test]
fn game_project_error_category_validation() {
    assert_eq!(
        GameProjectError::Validation(vec![]).category(),
        "Validation"
    );
}

// ============================================================================
// GameProjectError — is_io / is_parse / is_validation
// ============================================================================

#[test]
fn game_project_error_is_io_true() {
    assert!(GameProjectError::Io("x".into()).is_io());
}

#[test]
fn game_project_error_is_io_false_for_parse() {
    assert!(!GameProjectError::Parse("x".into()).is_io());
}

#[test]
fn game_project_error_is_io_false_for_serialize() {
    assert!(!GameProjectError::Serialize("x".into()).is_io());
}

#[test]
fn game_project_error_is_io_false_for_validation() {
    assert!(!GameProjectError::Validation(vec![]).is_io());
}

#[test]
fn game_project_error_is_parse_true() {
    assert!(GameProjectError::Parse("x".into()).is_parse());
}

#[test]
fn game_project_error_is_parse_false_for_io() {
    assert!(!GameProjectError::Io("x".into()).is_parse());
}

#[test]
fn game_project_error_is_parse_false_for_serialize() {
    assert!(!GameProjectError::Serialize("x".into()).is_parse());
}

#[test]
fn game_project_error_is_parse_false_for_validation() {
    assert!(!GameProjectError::Validation(vec![]).is_parse());
}

#[test]
fn game_project_error_is_validation_true() {
    assert!(GameProjectError::Validation(vec![]).is_validation());
}

#[test]
fn game_project_error_is_validation_false_for_io() {
    assert!(!GameProjectError::Io("x".into()).is_validation());
}

#[test]
fn game_project_error_is_validation_false_for_parse() {
    assert!(!GameProjectError::Parse("x".into()).is_validation());
}

#[test]
fn game_project_error_is_validation_false_for_serialize() {
    assert!(!GameProjectError::Serialize("x".into()).is_validation());
}

// ============================================================================
// GameProjectError — message()
// ============================================================================

#[test]
fn game_project_error_message_io() {
    let e = GameProjectError::Io("file not found".into());
    assert_eq!(e.message(), "file not found");
}

#[test]
fn game_project_error_message_parse() {
    let e = GameProjectError::Parse("bad toml".into());
    assert_eq!(e.message(), "bad toml");
}

#[test]
fn game_project_error_message_serialize() {
    let e = GameProjectError::Serialize("encoding error".into());
    assert_eq!(e.message(), "encoding error");
}

#[test]
fn game_project_error_message_validation_joins() {
    let e = GameProjectError::Validation(vec!["a".into(), "b".into(), "c".into()]);
    let msg = e.message();
    assert!(msg.contains("a"));
    assert!(msg.contains("b"));
    assert!(msg.contains("c"));
}

#[test]
fn game_project_error_message_validation_empty() {
    let e = GameProjectError::Validation(vec![]);
    assert_eq!(e.message(), "");
}

// ============================================================================
// AssetSettings — has_high_compression boundary (>= 10)
// ============================================================================

#[test]
fn asset_settings_high_compression_level_9_false() {
    let s = AssetSettings {
        compress: true,
        compression_level: 9,
        ..Default::default()
    };
    assert!(!s.has_high_compression());
}

#[test]
fn asset_settings_high_compression_level_10_true() {
    let s = AssetSettings {
        compress: true,
        compression_level: 10,
        ..Default::default()
    };
    assert!(s.has_high_compression());
}

#[test]
fn asset_settings_high_compression_level_11_true() {
    let s = AssetSettings {
        compress: true,
        compression_level: 11,
        ..Default::default()
    };
    assert!(s.has_high_compression());
}

#[test]
fn asset_settings_high_compression_disabled_false() {
    let s = AssetSettings {
        compress: false,
        compression_level: 22,
        ..Default::default()
    };
    assert!(!s.has_high_compression());
}

#[test]
fn asset_settings_high_compression_level_0_false() {
    let s = AssetSettings {
        compress: true,
        compression_level: 0,
        ..Default::default()
    };
    assert!(!s.has_high_compression());
}

// ============================================================================
// AssetSettings — pattern_count
// ============================================================================

#[test]
fn asset_settings_pattern_count_default() {
    let s = AssetSettings::default();
    // 3 include + 0 exclude = 3
    assert_eq!(s.pattern_count(), 3);
}

#[test]
fn asset_settings_pattern_count_with_excludes() {
    let mut s = AssetSettings::default();
    s.exclude.push("*.tmp".into());
    s.exclude.push("*.bak".into());
    assert_eq!(s.pattern_count(), 5); // 3 + 2
}

#[test]
fn asset_settings_pattern_count_empty() {
    let s = AssetSettings {
        include: vec![],
        exclude: vec![],
        compress: true,
        compression_level: 3,
    };
    assert_eq!(s.pattern_count(), 0);
}

// ============================================================================
// AssetSettings — has_excludes
// ============================================================================

#[test]
fn asset_settings_has_excludes_default_false() {
    let s = AssetSettings::default();
    assert!(!s.has_excludes());
}

#[test]
fn asset_settings_has_excludes_with_entry() {
    let mut s = AssetSettings::default();
    s.exclude.push("*.psd".into());
    assert!(s.has_excludes());
}

// ============================================================================
// AssetSettings — compression_summary
// ============================================================================

#[test]
fn asset_settings_compression_summary_enabled() {
    let s = AssetSettings::default(); // compress=true, level=3
    let summary = s.compression_summary();
    assert!(summary.contains("Enabled"));
    assert!(summary.contains("3"));
}

#[test]
fn asset_settings_compression_summary_enabled_high() {
    let s = AssetSettings {
        compress: true,
        compression_level: 22,
        ..Default::default()
    };
    let summary = s.compression_summary();
    assert!(summary.contains("Enabled"));
    assert!(summary.contains("22"));
}

#[test]
fn asset_settings_compression_summary_disabled() {
    let s = AssetSettings {
        compress: false,
        ..Default::default()
    };
    assert_eq!(s.compression_summary(), "Disabled");
}

// ============================================================================
// GameProject — new() defaults
// ============================================================================

#[test]
fn game_project_new_name() {
    let p = GameProject::new("TestGame", "scenes/main.scene");
    assert_eq!(p.name(), "TestGame");
}

#[test]
fn game_project_new_version() {
    let p = GameProject::new("TestGame", "scenes/main.scene");
    assert_eq!(p.version(), "0.1.0");
}

#[test]
fn game_project_new_no_icon() {
    let p = GameProject::new("TestGame", "scenes/main.scene");
    assert!(!p.has_icon());
}

#[test]
fn game_project_new_no_identifier() {
    let p = GameProject::new("TestGame", "scenes/main.scene");
    assert!(!p.has_identifier());
}

#[test]
fn game_project_new_zero_features() {
    let p = GameProject::new("TestGame", "scenes/main.scene");
    assert_eq!(p.feature_count(), 0);
}

// ============================================================================
// GameProject — validate() boundaries
// ============================================================================

#[test]
fn game_project_validate_ok() {
    let p = GameProject::default();
    assert!(p.validate().is_ok());
}

#[test]
fn game_project_validate_empty_name() {
    let mut p = GameProject::default();
    p.project.name = String::new();
    let res = p.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.contains("name")));
}

#[test]
fn game_project_validate_empty_entry_scene() {
    let mut p = GameProject::default();
    p.build.entry_scene = std::path::PathBuf::new();
    let res = p.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.iter().any(|e| e.contains("scene")));
}

#[test]
fn game_project_validate_both_empty() {
    let mut p = GameProject::default();
    p.project.name = String::new();
    p.build.entry_scene = std::path::PathBuf::new();
    let res = p.validate();
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert!(errs.len() >= 2);
}

// ============================================================================
// GameProject — summary()
// ============================================================================

#[test]
fn game_project_summary_contains_name() {
    let p = GameProject::new("MyGame", "main.scene");
    assert!(p.summary().contains("MyGame"));
}

#[test]
fn game_project_summary_contains_version() {
    let p = GameProject::new("MyGame", "main.scene");
    assert!(p.summary().contains("0.1.0"));
}

#[test]
fn game_project_summary_contains_target() {
    let p = GameProject::new("MyGame", "main.scene");
    assert!(p.summary().contains("windows"));
}

// ============================================================================
// GameProject — targets_platform
// ============================================================================

#[test]
fn game_project_targets_platform_windows_default() {
    let p = GameProject::default();
    assert!(p.targets_platform("windows"));
}

#[test]
fn game_project_targets_platform_linux_false() {
    let p = GameProject::default();
    assert!(!p.targets_platform("linux"));
}

#[test]
fn game_project_targets_platform_unknown_false() {
    let p = GameProject::default();
    assert!(!p.targets_platform("playstation"));
}

// ============================================================================
// GameProject — has_platform_config
// ============================================================================

#[test]
fn game_project_has_platform_config_none_by_default() {
    let p = GameProject::default();
    assert!(!p.has_platform_config("windows"));
    assert!(!p.has_platform_config("linux"));
    assert!(!p.has_platform_config("macos"));
    assert!(!p.has_platform_config("web"));
}

#[test]
fn game_project_has_platform_config_unknown_false() {
    let p = GameProject::default();
    assert!(!p.has_platform_config("xbox"));
    assert!(!p.has_platform_config(""));
}

#[test]
fn game_project_has_platform_config_windows_set() {
    let mut p = GameProject::default();
    p.platforms.windows = Some(PlatformConfig::default());
    assert!(p.has_platform_config("windows"));
    assert!(!p.has_platform_config("linux"));
}

#[test]
fn game_project_has_platform_config_linux_set() {
    let mut p = GameProject::default();
    p.platforms.linux = Some(PlatformConfig::default());
    assert!(p.has_platform_config("linux"));
}

#[test]
fn game_project_has_platform_config_macos_set() {
    let mut p = GameProject::default();
    p.platforms.macos = Some(PlatformConfig::default());
    assert!(p.has_platform_config("macos"));
}

#[test]
fn game_project_has_platform_config_web_set() {
    let mut p = GameProject::default();
    p.platforms.web = Some(PlatformConfig::default());
    assert!(p.has_platform_config("web"));
}

// ============================================================================
// GameProject — has_icon / has_identifier with values set
// ============================================================================

#[test]
fn game_project_has_icon_when_set() {
    let mut p = GameProject::default();
    p.project.icon = Some(std::path::PathBuf::from("icon.png"));
    assert!(p.has_icon());
}

#[test]
fn game_project_has_identifier_when_set() {
    let mut p = GameProject::default();
    p.project.identifier = Some("com.studio.game".to_string());
    assert!(p.has_identifier());
}

// ============================================================================
// GameProject — feature_count with features
// ============================================================================

#[test]
fn game_project_feature_count_with_features() {
    let mut p = GameProject::default();
    p.build.features = vec!["physics".into(), "ai".into()];
    assert_eq!(p.feature_count(), 2);
}

// ============================================================================
// GameProject — Default
// ============================================================================

#[test]
fn game_project_default_name() {
    let p = GameProject::default();
    assert_eq!(p.name(), "Untitled Game");
}

#[test]
fn game_project_default_entry_scene() {
    let p = GameProject::default();
    assert_eq!(
        p.build.entry_scene,
        std::path::PathBuf::from("scenes/main.scene")
    );
}

// ============================================================================
// EditorPreferences — default field values
// ============================================================================

#[test]
fn editor_preferences_default_show_grid() {
    let prefs = EditorPreferences::default();
    assert!(prefs.show_grid);
}

#[test]
fn editor_preferences_default_auto_save_disabled() {
    let prefs = EditorPreferences::default();
    assert!(!prefs.auto_save_enabled);
}

#[test]
fn editor_preferences_default_auto_save_interval() {
    let prefs = EditorPreferences::default();
    assert_eq!(prefs.auto_save_interval_secs, 300.0);
}

#[test]
fn editor_preferences_default_auto_save_keep_count() {
    let prefs = EditorPreferences::default();
    assert_eq!(prefs.auto_save_keep_count, 3);
}

#[test]
fn editor_preferences_default_auto_save_to_separate_dir() {
    let prefs = EditorPreferences::default();
    assert!(prefs.auto_save_to_separate_dir);
}

#[test]
fn editor_preferences_default_show_hierarchy_panel() {
    let prefs = EditorPreferences::default();
    assert!(prefs.show_hierarchy_panel);
}

#[test]
fn editor_preferences_default_show_inspector_panel() {
    let prefs = EditorPreferences::default();
    assert!(prefs.show_inspector_panel);
}

#[test]
fn editor_preferences_default_show_console_panel() {
    let prefs = EditorPreferences::default();
    assert!(prefs.show_console_panel);
}

#[test]
fn editor_preferences_default_camera_none() {
    let prefs = EditorPreferences::default();
    assert!(prefs.camera.is_none());
}

#[test]
fn editor_preferences_default_snapping_none() {
    let prefs = EditorPreferences::default();
    assert!(prefs.snapping.is_none());
}
