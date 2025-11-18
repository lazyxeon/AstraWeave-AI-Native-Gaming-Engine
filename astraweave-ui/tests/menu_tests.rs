/// Menu system integration tests
///
/// Tests menu navigation, state machine, and input handling
use astraweave_ui::{MenuAction, MenuManager, MenuState};

// ===== Menu Navigation Tests =====

#[test]
fn test_menu_navigation_main_to_settings() {
    let mut manager = MenuManager::new();

    assert_eq!(manager.current_state(), MenuState::MainMenu);

    // Navigate to settings
    manager.handle_action(MenuAction::Settings);

    assert_eq!(manager.current_state(), MenuState::SettingsMenu);
}

#[test]
fn test_menu_navigation_settings_back_to_main() {
    let mut manager = MenuManager::new();

    // Go to settings
    manager.handle_action(MenuAction::Settings);
    assert_eq!(manager.current_state(), MenuState::SettingsMenu);

    // Back to main menu
    manager.handle_action(MenuAction::Quit);

    assert_eq!(manager.current_state(), MenuState::MainMenu);
}

#[test]
fn test_menu_navigation_pause_to_main() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame); // Enter game

    // Toggle pause
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::PauseMenu);

    // Quit from pause = go to main menu
    manager.handle_action(MenuAction::Quit);
    assert_eq!(manager.current_state(), MenuState::MainMenu);
}

#[test]
fn test_menu_navigation_new_game_to_gameplay() {
    let mut manager = MenuManager::new();

    manager.handle_action(MenuAction::NewGame);

    assert_eq!(manager.current_state(), MenuState::None);
    assert!(!manager.is_menu_visible());
}

// ===== Menu State Machine Tests =====

#[test]
fn test_menu_state_initial_is_main_menu() {
    let manager = MenuManager::new();
    assert_eq!(manager.current_state(), MenuState::MainMenu);
    assert!(manager.is_main_menu());
}

#[test]
fn test_menu_state_transition_tracking() {
    let mut manager = MenuManager::new();

    // Transition to settings
    manager.handle_action(MenuAction::Settings);
    assert_eq!(manager.current_state(), MenuState::SettingsMenu);

    // Go back (uses previous state)
    manager.handle_action(MenuAction::Quit);
    assert_eq!(manager.current_state(), MenuState::MainMenu);
}

#[test]
fn test_menu_state_multiple_transitions() {
    let mut manager = MenuManager::new();

    // Main -> Settings
    manager.handle_action(MenuAction::Settings);
    assert_eq!(manager.current_state(), MenuState::SettingsMenu);

    // Settings -> Main
    manager.handle_action(MenuAction::Quit);
    assert_eq!(manager.current_state(), MenuState::MainMenu);

    // Main -> Game
    manager.handle_action(MenuAction::NewGame);
    assert_eq!(manager.current_state(), MenuState::None);

    // Game -> Pause
    manager.toggle_pause();
    assert_eq!(manager.current_state(), MenuState::PauseMenu);
}

#[test]
fn test_menu_state_visibility_tracking() {
    let mut manager = MenuManager::new();

    // Main menu is visible
    assert!(manager.is_menu_visible());

    // Enter game (no menu visible)
    manager.handle_action(MenuAction::NewGame);
    assert!(!manager.is_menu_visible());

    // Pause (menu visible again)
    manager.toggle_pause();
    assert!(manager.is_menu_visible());
}

// ===== Input Event Handling Tests =====

#[test]
fn test_input_toggle_pause_on() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame); // Enter game

    assert_eq!(manager.current_state(), MenuState::None);

    // Toggle pause on
    manager.toggle_pause();

    assert_eq!(manager.current_state(), MenuState::PauseMenu);
}

#[test]
fn test_input_toggle_pause_off() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame); // Enter game
    manager.toggle_pause(); // Pause on

    assert_eq!(manager.current_state(), MenuState::PauseMenu);

    // Toggle pause off
    manager.toggle_pause();

    assert_eq!(manager.current_state(), MenuState::None);
}

#[test]
fn test_input_resume_from_pause() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::NewGame);
    manager.toggle_pause();

    // Resume action
    manager.handle_action(MenuAction::Resume);

    assert_eq!(manager.current_state(), MenuState::None);
}

#[test]
fn test_input_escape_from_settings() {
    let mut manager = MenuManager::new();
    manager.handle_action(MenuAction::Settings);

    // ESC from settings (same as Quit)
    manager.toggle_pause();

    assert_eq!(manager.current_state(), MenuState::MainMenu);
}

// ===== Settings Tests =====

#[test]
fn test_settings_modification_tracking() {
    let mut manager = MenuManager::new();

    assert!(!manager.settings_modified());

    // Modify graphics settings
    manager.settings.graphics.fullscreen = !manager.settings.graphics.fullscreen;

    assert!(manager.settings_modified());
}

#[test]
fn test_settings_apply_saves_changes() {
    let mut manager = MenuManager::new();

    let original_fullscreen = manager.settings.graphics.fullscreen;
    manager.settings.graphics.fullscreen = !original_fullscreen;

    // Apply settings
    manager.apply_settings();

    assert!(!manager.settings_modified());
}

#[test]
fn test_settings_revert_discards_changes() {
    let mut manager = MenuManager::new();

    let original_volume = manager.settings.audio.master_volume;
    manager.settings.audio.master_volume = 50.0;
    assert!(manager.settings_modified());

    // Revert changes
    manager.revert_settings();

    assert!(!manager.settings_modified());
    assert_eq!(manager.settings.audio.master_volume, original_volume);
}

#[test]
fn test_settings_reset_controls() {
    let mut manager = MenuManager::new();

    // Modify controls
    manager.settings.controls.move_forward = "Y".to_string();
    manager.settings.controls.mouse_sensitivity = 3.0;

    // Reset to defaults
    manager.reset_controls_to_default();

    assert_eq!(manager.settings.controls.move_forward, "W");
    assert_eq!(manager.settings.controls.mouse_sensitivity, 1.0);
}

#[test]
fn test_settings_key_rebinding_state() {
    let mut manager = MenuManager::new();

    assert_eq!(manager.rebinding_key, None);

    // Start rebinding
    manager.rebinding_key = Some("move_forward".to_string());
    assert_eq!(manager.rebinding_key, Some("move_forward".to_string()));

    // Clear rebinding
    manager.rebinding_key = None;
    assert_eq!(manager.rebinding_key, None);
}
