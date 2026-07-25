use crate::gizmo::snapping::SnappingConfig;
use crate::viewport::camera::{CameraStation, OrbitCamera};
use serde::{Deserialize, Serialize};
use std::fs;

const PREFERENCES_PATH: &str = ".editor_preferences.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorPreferences {
    pub show_grid: bool,
    pub auto_save_enabled: bool,
    pub auto_save_interval_secs: f32,
    // Week 7: Enhanced auto-save settings
    #[serde(default = "default_auto_save_count")]
    pub auto_save_keep_count: usize,
    #[serde(default)]
    pub auto_save_to_separate_dir: bool,
    pub show_hierarchy_panel: bool,
    pub show_inspector_panel: bool,
    pub show_console_panel: bool,
    pub camera: Option<OrbitCamera>,
    pub snapping: Option<SnappingConfig>,
    /// Serialized dock layout JSON for persistence
    #[serde(default)]
    pub layout_json: Option<String>,
    /// Whether the first-run tutorial has been completed or skipped.
    #[serde(default)]
    pub tutorial_completed: bool,
    /// Directories to scan for .blend scene assets.
    #[serde(default = "default_blend_asset_directories")]
    pub blend_asset_directories: Vec<String>,
    /// ED-2: named camera stations (pinned viewpoints) for visual A/B gates.
    ///
    /// `#[serde(default)]` so preference files written before ED-2 load
    /// unchanged — the same backward-compatibility discipline the rest of this
    /// struct uses.
    #[serde(default)]
    pub camera_stations: Vec<CameraStation>,
}

fn default_auto_save_count() -> usize {
    3
}

pub fn default_blend_asset_directories() -> Vec<String> {
    vec!["assets/blends".to_string(), "assets/scenes".to_string()]
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            show_grid: true,
            auto_save_enabled: false,
            auto_save_interval_secs: 300.0,
            auto_save_keep_count: 3,
            auto_save_to_separate_dir: true,
            show_hierarchy_panel: true,
            show_inspector_panel: true,
            show_console_panel: true,
            camera: None,
            snapping: None,
            layout_json: None,
            tutorial_completed: false,
            blend_asset_directories: default_blend_asset_directories(),
            camera_stations: Vec::new(),
        }
    }
}

impl EditorPreferences {
    pub fn load() -> Self {
        if let Ok(contents) = fs::read_to_string(PREFERENCES_PATH) {
            if let Ok(prefs) = serde_json::from_str(&contents) {
                return prefs;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        match serde_json::to_string_pretty(&self) {
            Ok(json) => {
                let tmp_path = format!("{}.tmp", PREFERENCES_PATH);
                if let Err(e) = fs::write(&tmp_path, &json)
                    .and_then(|_| fs::rename(&tmp_path, PREFERENCES_PATH))
                {
                    tracing::error!(
                        "Failed to write editor preferences to {}: {}",
                        PREFERENCES_PATH,
                        e
                    );
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize editor preferences: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences() {
        let prefs = EditorPreferences::default();
        assert!(prefs.show_grid);
        assert!(!prefs.auto_save_enabled);
        assert_eq!(prefs.auto_save_interval_secs, 300.0);
        assert!(prefs.show_hierarchy_panel);
        assert!(prefs.show_inspector_panel);
        assert!(prefs.show_console_panel);
    }

    #[test]
    fn test_week7_auto_save_defaults() {
        let prefs = EditorPreferences::default();
        assert_eq!(prefs.auto_save_keep_count, 3);
        assert!(prefs.auto_save_to_separate_dir);
    }

    #[test]
    fn test_preferences_serialization_roundtrip() {
        let prefs = EditorPreferences {
            show_grid: false,
            auto_save_enabled: true,
            auto_save_interval_secs: 120.0,
            auto_save_keep_count: 5,
            auto_save_to_separate_dir: false,
            show_hierarchy_panel: false,
            show_inspector_panel: true,
            show_console_panel: false,
            camera: None,
            snapping: None,
            layout_json: None,
            tutorial_completed: false,
            blend_asset_directories: vec!["test/blends".into()],
            camera_stations: Vec::new(),
        };

        let json = serde_json::to_string(&prefs).expect("serialize");
        let restored: EditorPreferences = serde_json::from_str(&json).expect("deserialize");

        assert!(!restored.show_grid);
        assert!(restored.auto_save_enabled);
        assert_eq!(restored.auto_save_interval_secs, 120.0);
        assert_eq!(restored.auto_save_keep_count, 5);
        assert!(!restored.auto_save_to_separate_dir);
    }

    #[test]
    fn test_preferences_deserialize_missing_week7_fields() {
        // Simulates loading old preferences without Week 7 fields
        let old_json = r#"{
            "show_grid": true,
            "auto_save_enabled": false,
            "auto_save_interval_secs": 300.0,
            "show_hierarchy_panel": true,
            "show_inspector_panel": true,
            "show_console_panel": true,
            "camera": null,
            "snapping": null
        }"#;

        let prefs: EditorPreferences = serde_json::from_str(old_json).expect("deserialize");

        // Should use defaults for missing Week 7 fields
        assert_eq!(prefs.auto_save_keep_count, 3);
        assert!(!prefs.auto_save_to_separate_dir); // default is false when missing
    }

    #[test]
    fn test_default_auto_save_count_fn() {
        assert_eq!(default_auto_save_count(), 3);
    }

    /// ED-2 Concern 2: named stations survive a real file round-trip.
    #[test]
    fn camera_stations_round_trip_through_a_file() {
        use crate::viewport::camera::{CameraStation, OrbitCamera};

        let mut cam = OrbitCamera::default();
        cam.set_focal_point(glam::Vec3::new(-12.5, 33.25, 1971.75));
        cam.set_distance(137.5);
        cam.set_yaw(2.1);
        cam.set_pitch(-0.45);
        cam.set_fov(72.0);
        cam.set_aspect(1024.0, 768.0);

        let prefs = EditorPreferences {
            camera_stations: vec![CameraStation {
                name: "desert_overview".to_string(),
                state: cam.capture_state(),
            }],
            ..EditorPreferences::default()
        };

        let dir = std::env::temp_dir().join("aw_ed2_prefs_round_trip");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("prefs.json");
        let json = serde_json::to_string_pretty(&prefs).expect("serialize");
        std::fs::write(&file, &json).expect("write");

        let read = std::fs::read_to_string(&file).expect("read");
        let restored: EditorPreferences = serde_json::from_str(&read).expect("deserialize");
        assert_eq!(restored.camera_stations.len(), 1);
        assert_eq!(restored.camera_stations[0].name, "desert_overview");
        assert_eq!(
            restored.camera_stations[0].state,
            prefs.camera_stations[0].state
        );

        // And the restored state still reproduces the exact view.
        let mut fresh = OrbitCamera::default();
        fresh.set_aspect(1024.0, 768.0);
        fresh.apply_state(&restored.camera_stations[0].state);
        assert_eq!(
            fresh.view_matrix().to_cols_array(),
            cam.view_matrix().to_cols_array(),
            "a station restored from disk must reproduce the pinned view exactly"
        );

        let _ = std::fs::remove_file(&file);
    }

    /// Preference files written before ED-2 have no `camera_stations` key and
    /// must still load — the same backward-compat contract the rest of this
    /// struct honours.
    #[test]
    fn preferences_without_camera_stations_still_load() {
        let old = r#"{
            "show_grid": true,
            "auto_save_enabled": false,
            "auto_save_interval_secs": 300.0,
            "show_hierarchy_panel": true,
            "show_inspector_panel": true,
            "show_console_panel": true,
            "camera": null,
            "snapping": null
        }"#;
        let prefs: EditorPreferences = serde_json::from_str(old).expect("deserialize legacy prefs");
        assert!(prefs.camera_stations.is_empty());
    }
}
