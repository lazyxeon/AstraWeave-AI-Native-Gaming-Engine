/// Comprehensive tests for menus.rs (show_main_menu, show_pause_menu, show_settings_menu)
///
/// Tests menu state, navigation flows, and settings interactions
use astraweave_ui::{MenuAction, MenuManager, MenuState, SettingsState, QualityPreset};

// ===== Menu Navigation Tests =====

#[test]
fn test_menu_navigation_main_to_pause() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame);
    
    // Toggle pause
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::PauseMenu);
}

#[test]
fn test_menu_navigation_pause_to_main() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame);
    manager.toggle_pause();
    
    // Quit from pause
    manager.handle_action(MenuAction::Quit);
    assert_eq!(manager.current_state(), MenuState::MainMenu);
}

#[test]
fn test_menu_navigation_main_to_settings() {
    let mut manager = MenuManager::new();
    
    manager.handle_action(MenuAction::Settings);
    assert_eq!(manager.current_state(), MenuState::SettingsMenu);
}

#[test]
fn test_menu_navigation_settings_back() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::Settings);
    
    manager.handle_action(MenuAction::Quit);
    assert_eq!(manager.current_state(), MenuState::MainMenu);
}

#[test]
fn test_menu_navigation_pause_resume() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame);
    manager.toggle_pause();
    
    manager.handle_action(MenuAction::Resume);
    assert_eq!(manager.current_state(), MenuState::None);
}

// ===== Settings State Tests =====

#[test]
fn test_settings_resolution_modification() {
    let mut settings = SettingsState::default();
    
    let original_resolution = settings.graphics.resolution;
    settings.graphics.resolution = (2560, 1440);
    
    assert_ne!(settings.graphics.resolution, original_resolution);
    assert_eq!(settings.graphics.resolution, (2560, 1440));
}

#[test]
fn test_settings_quality_preset_modification() {
    let mut settings = SettingsState::default();
    
    settings.graphics.quality = QualityPreset::Low;
    assert_eq!(settings.graphics.quality, QualityPreset::Low);
    
    settings.graphics.quality = QualityPreset::Ultra;
    assert_eq!(settings.graphics.quality, QualityPreset::Ultra);
}

#[test]
fn test_settings_fullscreen_toggle() {
    let mut settings = SettingsState::default();
    
    let original_fullscreen = settings.graphics.fullscreen;
    settings.graphics.fullscreen = !original_fullscreen;
    
    assert_ne!(settings.graphics.fullscreen, original_fullscreen);
}

#[test]
fn test_settings_vsync_toggle() {
    let mut settings = SettingsState::default();
    
    let original_vsync = settings.graphics.vsync;
    settings.graphics.vsync = !original_vsync;
    
    assert_ne!(settings.graphics.vsync, original_vsync);
}

#[test]
fn test_settings_audio_volume_modification() {
    let mut settings = SettingsState::default();
    
    settings.audio.master_volume = 50.0;
    settings.audio.music_volume = 60.0;
    settings.audio.sfx_volume = 70.0;
    settings.audio.voice_volume = 80.0;
    
    assert_eq!(settings.audio.master_volume, 50.0);
    assert_eq!(settings.audio.music_volume, 60.0);
    assert_eq!(settings.audio.sfx_volume, 70.0);
    assert_eq!(settings.audio.voice_volume, 80.0);
}

#[test]
fn test_settings_audio_mute_toggles() {
    let mut settings = SettingsState::default();
    
    settings.audio.master_mute = true;
    settings.audio.music_mute = true;
    settings.audio.sfx_mute = true;
    settings.audio.voice_mute = true;
    
    assert!(settings.audio.master_mute);
    assert!(settings.audio.music_mute);
    assert!(settings.audio.sfx_mute);
    assert!(settings.audio.voice_mute);
}

#[test]
fn test_settings_controls_key_binding_modification() {
    let mut settings = SettingsState::default();
    
    settings.controls.move_forward = "Up".to_string();
    settings.controls.move_backward = "Down".to_string();
    settings.controls.move_left = "Left".to_string();
    settings.controls.move_right = "Right".to_string();
    
    assert_eq!(settings.controls.move_forward, "Up");
    assert_eq!(settings.controls.move_backward, "Down");
    assert_eq!(settings.controls.move_left, "Left");
    assert_eq!(settings.controls.move_right, "Right");
}

#[test]
fn test_settings_mouse_sensitivity_modification() {
    let mut settings = SettingsState::default();
    
    settings.controls.mouse_sensitivity = 2.5;
    assert_eq!(settings.controls.mouse_sensitivity, 2.5);
}

#[test]
fn test_settings_invert_y_toggle() {
    let mut settings = SettingsState::default();
    
    settings.controls.invert_y = true;
    assert!(settings.controls.invert_y);
}

#[test]
fn test_settings_rebinding_key_state() {
    let mut manager = MenuManager::new();
    
    manager.rebinding_key = Some("move_forward".to_string());
    assert_eq!(manager.rebinding_key, Some("move_forward".to_string()));
}

#[test]
fn test_settings_rebinding_clear() {
    let mut manager = MenuManager::new();
    
    manager.rebinding_key = Some("jump".to_string());
    manager.rebinding_key = None;
    
    assert_eq!(manager.rebinding_key, None);
}

// ===== Edge Cases and Boundary Tests =====

#[test]
fn test_settings_audio_volume_boundary_low() {
    let mut settings = SettingsState::default();
    
    settings.audio.master_volume = 0.0;
    assert_eq!(settings.audio.master_volume, 0.0);
}

#[test]
fn test_settings_audio_volume_boundary_high() {
    let mut settings = SettingsState::default();
    
    settings.audio.master_volume = 100.0;
    assert_eq!(settings.audio.master_volume, 100.0);
}

#[test]
fn test_settings_mouse_sensitivity_boundary_low() {
    let mut settings = SettingsState::default();
    
    settings.controls.mouse_sensitivity = 0.1;
    assert_eq!(settings.controls.mouse_sensitivity, 0.1);
}

#[test]
fn test_settings_mouse_sensitivity_boundary_high() {
    let mut settings = SettingsState::default();
    
    settings.controls.mouse_sensitivity = 5.0;
    assert_eq!(settings.controls.mouse_sensitivity, 5.0);
}

#[test]
fn test_settings_all_resolutions() {
    let mut settings = SettingsState::default();
    
    let resolutions = [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)];
    
    for &resolution in &resolutions {
        settings.graphics.resolution = resolution;
        assert_eq!(settings.graphics.resolution, resolution);
    }
}

#[test]
fn test_settings_all_quality_presets() {
    let mut settings = SettingsState::default();
    
    let presets = [
        QualityPreset::Low,
        QualityPreset::Medium,
        QualityPreset::High,
        QualityPreset::Ultra,
    ];
    
    for &preset in &presets {
        settings.graphics.quality = preset;
        assert_eq!(settings.graphics.quality, preset);
    }
}

// ===== MenuManager Integration Tests =====

#[test]
fn test_menu_manager_settings_modification_tracking() {
    let mut manager = MenuManager::new();
    
    assert!(!manager.settings_modified());
    
    manager.settings.graphics.fullscreen = !manager.settings.graphics.fullscreen;
    assert!(manager.settings_modified());
}

#[test]
fn test_menu_manager_apply_settings() {
    let mut manager = MenuManager::new();
    
    let original_fullscreen = manager.settings.graphics.fullscreen;
    manager.settings.graphics.fullscreen = !original_fullscreen;
    
    manager.apply_settings();
    assert!(!manager.settings_modified());
}

#[test]
fn test_menu_manager_revert_settings() {
    let mut manager = MenuManager::new();
    
    let original_volume = manager.settings.audio.master_volume;
    manager.settings.audio.master_volume = 50.0;
    assert!(manager.settings_modified());
    
    manager.revert_settings();
    assert!(!manager.settings_modified());
    assert_eq!(manager.settings.audio.master_volume, original_volume);
}

#[test]
fn test_menu_manager_reset_controls() {
    let mut manager = MenuManager::new();
    
    manager.settings.controls.move_forward = "Y".to_string();
    manager.settings.controls.mouse_sensitivity = 3.0;
    
    manager.reset_controls_to_default();
    
    assert_eq!(manager.settings.controls.move_forward, "W");
    assert_eq!(manager.settings.controls.mouse_sensitivity, 1.0);
}

#[test]
fn test_menu_manager_visibility_tracking() {
    let mut manager = MenuManager::new();
    
    assert!(manager.is_menu_visible());
    
    manager.handle_action(MenuAction::NewGame);
    assert!(!manager.is_menu_visible());
    
    manager.toggle_pause();
    assert!(manager.is_menu_visible());
}

#[test]
fn test_menu_manager_multiple_transitions() {
    let mut manager = MenuManager::new();
    
    manager.handle_action(MenuAction::Settings);
    assert_eq!(manager.current_state(), MenuState::SettingsMenu);
    
    manager.handle_action(MenuAction::Quit);
    assert_eq!(manager.current_state(), MenuState::MainMenu);
    
    manager.handle_action(MenuAction::NewGame);
    assert_eq!(manager.current_state(), MenuState::None);
    
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::PauseMenu);
}

#[test]
fn test_quality_preset_as_str() {
    assert_eq!(QualityPreset::Low.as_str(), "Low");
    assert_eq!(QualityPreset::Medium.as_str(), "Medium");
    assert_eq!(QualityPreset::High.as_str(), "High");
    assert_eq!(QualityPreset::Ultra.as_str(), "Ultra");
}

#[test]
fn test_quality_preset_all() {
    let all = QualityPreset::all();
    assert_eq!(all.len(), 4);
    assert_eq!(all[0], QualityPreset::Low);
    assert_eq!(all[3], QualityPreset::Ultra);
}

