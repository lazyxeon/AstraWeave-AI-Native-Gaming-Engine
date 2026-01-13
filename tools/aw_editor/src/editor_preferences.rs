use crate::gizmo::snapping::SnappingConfig;
use crate::viewport::camera::OrbitCamera;
use serde::{Deserialize, Serialize};
use std::fs;

const PREFERENCES_PATH: &str = ".editor_preferences.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorPreferences {
    pub show_grid: bool,
    pub auto_save_enabled: bool,
    pub auto_save_interval_secs: f32,
    pub show_hierarchy_panel: bool,
    pub show_inspector_panel: bool,
    pub show_console_panel: bool,
    pub camera: Option<OrbitCamera>,
    pub snapping: Option<SnappingConfig>,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            show_grid: true,
            auto_save_enabled: false,
            auto_save_interval_secs: 300.0,
            show_hierarchy_panel: true,
            show_inspector_panel: true,
            show_console_panel: true,
            camera: None,
            snapping: None,
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
                if let Err(e) = fs::write(PREFERENCES_PATH, json) {
                    tracing::error!("Failed to write editor preferences to {}: {}", PREFERENCES_PATH, e);
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
}
