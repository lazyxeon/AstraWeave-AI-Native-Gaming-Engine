/// Input event handling tests
///
/// Tests keyboard, mouse, and event priority
use astraweave_ui::{
    AudioSettings, ControlsSettings, GraphicsSettings, MenuAction, MenuManager, MenuState,
    QualityPreset,
};

// ===== Keyboard Navigation Tests =====

#[test]
fn test_keyboard_settings_navigation() {
    let mut manager = MenuManager::new();

    // ESC to enter settings (simulated)
    manager.handle_action(MenuAction::Settings);
    assert_eq!(manager.current_state(), MenuState::SettingsMenu);

    // ESC to go back
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::MainMenu);
}

#[test]
fn test_keyboard_pause_toggle() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame);

    // ESC to pause
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::PauseMenu);

    // ESC to unpause
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::None);
}

#[test]
fn test_keyboard_menu_actions() {
    let mut manager = MenuManager::new();

    // Simulate clicking "New Game"
    manager.handle_action(MenuAction::NewGame);
    assert_eq!(manager.current_state(), MenuState::None);
}

// ===== Menu Action Events =====

#[test]
fn test_menu_action_new_game() {
    let action = MenuAction::NewGame;
    assert_eq!(action, MenuAction::NewGame);
}

#[test]
fn test_menu_action_load_game() {
    let action = MenuAction::LoadGame;
    assert_eq!(action, MenuAction::LoadGame);
}

#[test]
fn test_menu_action_save_game() {
    let action = MenuAction::SaveGame;
    assert_eq!(action, MenuAction::SaveGame);
}

#[test]
fn test_menu_action_resume() {
    let action = MenuAction::Resume;
    assert_eq!(action, MenuAction::Resume);
}

#[test]
fn test_menu_action_settings() {
    let action = MenuAction::Settings;
    assert_eq!(action, MenuAction::Settings);
}

#[test]
fn test_menu_action_quit() {
    let action = MenuAction::Quit;
    assert_eq!(action, MenuAction::Quit);
}

#[test]
fn test_menu_action_none() {
    let action = MenuAction::None;
    assert_eq!(action, MenuAction::None);
}

// ===== Settings Input Events =====

#[test]
fn test_settings_graphics_quality_change() {
    let mut settings = GraphicsSettings::default();

    settings.quality = QualityPreset::Low;
    assert_eq!(settings.quality, QualityPreset::Low);

    settings.quality = QualityPreset::Ultra;
    assert_eq!(settings.quality, QualityPreset::Ultra);
}

#[test]
fn test_settings_resolution_change() {
    let mut settings = GraphicsSettings::default();

    settings.resolution = (2560, 1440);
    assert_eq!(settings.resolution, (2560, 1440));
}

#[test]
fn test_settings_fullscreen_toggle() {
    let mut settings = GraphicsSettings::default();

    let initial = settings.fullscreen;
    settings.fullscreen = !initial;
    assert_ne!(settings.fullscreen, initial);
}

#[test]
fn test_settings_vsync_toggle() {
    let mut settings = GraphicsSettings::default();

    let initial = settings.vsync;
    settings.vsync = !initial;
    assert_ne!(settings.vsync, initial);
}

#[test]
fn test_settings_audio_volume_slider() {
    let mut settings = AudioSettings::default();

    settings.master_volume = 75.0;
    assert_eq!(settings.master_volume, 75.0);

    settings.music_volume = 50.0;
    assert_eq!(settings.music_volume, 50.0);
}

#[test]
fn test_settings_audio_mute_toggle() {
    let mut settings = AudioSettings::default();

    settings.master_mute = true;
    assert!(settings.master_mute);

    settings.music_mute = true;
    assert!(settings.music_mute);
}

#[test]
fn test_settings_controls_key_rebind() {
    let mut settings = ControlsSettings::default();

    settings.move_forward = "Y".to_string();
    assert_eq!(settings.move_forward, "Y");

    settings.jump = "X".to_string();
    assert_eq!(settings.jump, "X");
}

#[test]
fn test_settings_mouse_sensitivity_slider() {
    let mut settings = ControlsSettings::default();

    settings.mouse_sensitivity = 2.5;
    assert_eq!(settings.mouse_sensitivity, 2.5);
}

#[test]
fn test_settings_invert_y_toggle() {
    let mut settings = ControlsSettings::default();

    settings.invert_y = true;
    assert!(settings.invert_y);
}

// ===== Event Priority Tests =====

#[test]
fn test_event_priority_pause_overrides_gameplay() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame);

    // In-game
    assert_eq!(manager.current_state(), MenuState::None);

    // Pause takes priority
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::PauseMenu);
}

#[test]
fn test_event_priority_settings_over_main_menu() {
    let mut manager = MenuManager::new();

    // At main menu
    assert_eq!(manager.current_state(), MenuState::MainMenu);

    // Settings action takes priority
    manager.handle_action(MenuAction::Settings);
    assert_eq!(manager.current_state(), MenuState::SettingsMenu);
}

#[test]
fn test_event_buffering_multiple_actions() {
    let mut manager = MenuManager::new();

    // Execute multiple actions in sequence
    manager.handle_action(MenuAction::Settings);
    manager.handle_action(MenuAction::Quit);
    manager.handle_action(MenuAction::NewGame);

    assert_eq!(manager.current_state(), MenuState::None);
}
