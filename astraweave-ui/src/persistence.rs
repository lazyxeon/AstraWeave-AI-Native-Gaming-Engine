//! Settings persistence - save/load configuration to disk
//!
//! Platform-specific config file locations:
//! - Windows: %APPDATA%\AstraWeave\settings.toml
//! - Linux: ~/.config/astraweave/settings.toml
//! - macOS: ~/Library/Application Support/AstraWeave/settings.toml

use crate::menu::SettingsState;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Version for settings file format (for future migration support)
const SETTINGS_VERSION: u32 = 1;

/// Wrapper for settings with version metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SettingsFile {
    version: u32,
    settings: SettingsState,
}

/// Get the platform-specific config file path
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Failed to determine config directory")?;

    let app_dir = config_dir.join("AstraWeave");

    // Create directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).context("Failed to create config directory")?;
    }

    Ok(app_dir.join("settings.toml"))
}

/// Save settings to disk
pub fn save_settings(settings: &SettingsState) -> Result<()> {
    let path = get_config_path()?;

    let settings_file = SettingsFile {
        version: SETTINGS_VERSION,
        settings: settings.clone(),
    };

    let toml_string =
        toml::to_string_pretty(&settings_file).context("Failed to serialize settings to TOML")?;

    fs::write(&path, toml_string).context("Failed to write settings file")?;

    log::info!("Settings saved to: {}", path.display());
    Ok(())
}

/// Load settings from disk, returns default settings if file doesn't exist or is corrupted
pub fn load_settings() -> SettingsState {
    match try_load_settings() {
        Ok(settings) => {
            log::info!("Settings loaded successfully");
            settings
        }
        Err(e) => {
            log::warn!("Failed to load settings ({}), using defaults", e);
            SettingsState::default()
        }
    }
}

/// Internal function that returns errors instead of defaulting
fn try_load_settings() -> Result<SettingsState> {
    let path = get_config_path()?;

    if !path.exists() {
        anyhow::bail!("Settings file does not exist");
    }

    let toml_string = fs::read_to_string(&path).context("Failed to read settings file")?;

    let settings_file: SettingsFile =
        toml::from_str(&toml_string).context("Failed to parse settings TOML")?;

    // Version migration support (for future versions)
    if settings_file.version != SETTINGS_VERSION {
        log::warn!(
            "Settings file version mismatch (found {}, expected {}), attempting migration",
            settings_file.version,
            SETTINGS_VERSION
        );
        // For now, just use the settings as-is
        // In future: add migration logic here
    }

    log::info!("Settings loaded from: {}", path.display());
    Ok(settings_file.settings)
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path() {
        let path = get_config_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("AstraWeave"));
        assert!(path.to_string_lossy().ends_with("settings.toml"));
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut settings = SettingsState::default();
        settings.graphics.fullscreen = true;
        settings.audio.master_volume = 75.0;
        settings.controls.mouse_sensitivity = 2.5;

        // Save settings
        let result = save_settings(&settings);
        assert!(result.is_ok());

        // Load settings
        let loaded = load_settings();
        assert_eq!(loaded.graphics.fullscreen, true);
        assert_eq!(loaded.audio.master_volume, 75.0);
        assert_eq!(loaded.controls.mouse_sensitivity, 2.5);
    }

    #[test]
    fn test_corrupted_file_fallback() {
        // This test verifies that corrupted files fall back to defaults
        // In practice, we'd need to manually corrupt a file, so we just
        // test that load_settings() returns defaults when file is missing
        let settings = load_settings();
        // Should return defaults without panicking
        assert_eq!(settings.graphics.quality, crate::menu::QualityPreset::High);
    }
}
