/// Menu system for in-game menus (main menu, pause menu, settings)
///
/// This module provides a menu state machine and menu UI components.
use serde::{Deserialize, Serialize};

/// Menu states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuState {
    /// Main menu (game not started)
    MainMenu,
    /// Pause menu (game paused)
    PauseMenu,
    /// Settings menu
    SettingsMenu,
    /// No menu visible (in-game)
    None,
}

/// Graphics quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
}

impl QualityPreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            QualityPreset::Low => "Low",
            QualityPreset::Medium => "Medium",
            QualityPreset::High => "High",
            QualityPreset::Ultra => "Ultra",
        }
    }

    pub fn all() -> &'static [QualityPreset] {
        &[
            QualityPreset::Low,
            QualityPreset::Medium,
            QualityPreset::High,
            QualityPreset::Ultra,
        ]
    }
}

/// Graphics settings state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub resolution: (u32, u32),
    pub quality: QualityPreset,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            quality: QualityPreset::High,
            fullscreen: false,
            vsync: true,
        }
    }
}

/// Audio settings for volume and mute controls
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Master volume (0-100)
    pub master_volume: f32,
    /// Music volume (0-100)
    pub music_volume: f32,
    /// SFX volume (0-100)
    pub sfx_volume: f32,
    /// Voice volume (0-100)
    pub voice_volume: f32,
    /// Master mute
    pub master_mute: bool,
    /// Music mute
    pub music_mute: bool,
    /// SFX mute
    pub sfx_mute: bool,
    /// Voice mute
    pub voice_mute: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 100.0,
            music_volume: 80.0,
            sfx_volume: 90.0,
            voice_volume: 100.0,
            master_mute: false,
            music_mute: false,
            sfx_mute: false,
            voice_mute: false,
        }
    }
}

/// Key bindings for game controls
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlsSettings {
    /// Move forward key
    pub move_forward: String,
    /// Move backward key
    pub move_backward: String,
    /// Move left key
    pub move_left: String,
    /// Move right key
    pub move_right: String,
    /// Jump key
    pub jump: String,
    /// Crouch key
    pub crouch: String,
    /// Sprint key
    pub sprint: String,
    /// Attack key
    pub attack: String,
    /// Interact key
    pub interact: String,
    /// Inventory key
    pub inventory: String,
    /// Mouse sensitivity (0.1 - 5.0x)
    pub mouse_sensitivity: f32,
    /// Invert Y-axis
    pub invert_y: bool,
}

impl Default for ControlsSettings {
    fn default() -> Self {
        Self {
            move_forward: "W".to_string(),
            move_backward: "S".to_string(),
            move_left: "A".to_string(),
            move_right: "D".to_string(),
            jump: "Space".to_string(),
            crouch: "LControl".to_string(),
            sprint: "LShift".to_string(),
            attack: "Mouse0".to_string(),
            interact: "E".to_string(),
            inventory: "Tab".to_string(),
            mouse_sensitivity: 1.0,
            invert_y: false,
        }
    }
}

/// Settings state (holds all settings categories)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SettingsState {
    #[serde(default)]
    pub graphics: GraphicsSettings,
    #[serde(default)]
    pub audio: AudioSettings,
    #[serde(default)]
    pub controls: ControlsSettings,
}

/// Actions that can be triggered by menus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Start new game
    NewGame,
    /// Load saved game
    LoadGame,
    /// Save current game
    SaveGame,
    /// Resume from pause
    Resume,
    /// Open settings menu
    Settings,
    /// Apply settings (save to disk)
    ApplySettings,
    /// Cancel settings (revert changes)
    CancelSettings,
    /// Quit to main menu or exit
    Quit,
    /// No action
    None,
}

/// Menu manager - handles menu state transitions
pub struct MenuManager {
    state: MenuState,
    /// Track previous state for "Back" functionality
    previous_state: Option<MenuState>,
    /// Settings state (current + original for revert)
    pub settings: SettingsState,
    settings_original: SettingsState,
    /// Key rebinding state (which key is being rebound, if any)
    pub rebinding_key: Option<String>,
}

impl MenuManager {
    /// Create new menu manager (starts at main menu, loads saved settings)
    pub fn new() -> Self {
        let settings = crate::persistence::load_settings();
        Self {
            state: MenuState::MainMenu,
            previous_state: None,
            settings: settings.clone(),
            settings_original: settings,
            rebinding_key: None,
        }
    }

    /// Show the current menu and return any action taken
    pub fn show(&mut self, ctx: &egui::Context) -> MenuAction {
        match self.state {
            MenuState::MainMenu => crate::menus::show_main_menu(ctx),
            MenuState::PauseMenu => crate::menus::show_pause_menu(ctx),
            MenuState::SettingsMenu => crate::menus::show_settings_menu(ctx, &mut self.settings, &mut self.rebinding_key),
            MenuState::None => MenuAction::None,
        }
    }

    /// Handle a menu action (updates state)
    pub fn handle_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::NewGame => {
                self.previous_state = Some(self.state);
                self.state = MenuState::None; // Go to game
            }
            MenuAction::Resume => {
                self.previous_state = Some(self.state);
                self.state = MenuState::None; // Resume game
            }
            MenuAction::Settings => {
                self.previous_state = Some(self.state);
                self.state = MenuState::SettingsMenu;
            }
            MenuAction::Quit => {
                // Context-sensitive quit behavior
                match self.state {
                    MenuState::PauseMenu => {
                        // Quit from pause = go to main menu
                        self.previous_state = Some(self.state);
                        self.state = MenuState::MainMenu;
                    }
                    MenuState::SettingsMenu => {
                        // Back from settings = go to previous menu
                        if let Some(prev) = self.previous_state {
                            self.state = prev;
                            self.previous_state = None;
                        } else {
                            // Fallback: go to main menu
                            self.state = MenuState::MainMenu;
                        }
                    }
                    MenuState::MainMenu => {
                        // Quit from main menu = close application (handled by caller)
                    }
                    MenuState::None => {
                        // Should not happen (no menu visible)
                    }
                }
            }
            MenuAction::ApplySettings => {
                // Save settings to disk and update original state
                self.apply_settings();
            }
            MenuAction::CancelSettings => {
                // Revert to last saved settings
                self.revert_settings();
            }
            _ => {}
        }
    }

    /// Toggle pause menu (called on ESC key)
    pub fn toggle_pause(&mut self) {
        match self.state {
            MenuState::None => {
                self.previous_state = Some(self.state);
                self.state = MenuState::PauseMenu;
            }
            MenuState::PauseMenu => {
                self.previous_state = Some(self.state);
                self.state = MenuState::None;
            }
            MenuState::SettingsMenu => {
                // ESC from settings = go back to previous menu
                if let Some(prev) = self.previous_state {
                    self.state = prev;
                    self.previous_state = None;
                } else {
                    // Fallback: go to pause menu if in-game, otherwise main menu
                    self.state = MenuState::PauseMenu;
                }
            }
            _ => {}
        }
    }

    /// Check if any menu is currently visible
    pub fn is_menu_visible(&self) -> bool {
        self.state != MenuState::None
    }

    /// Check if we're on the main menu
    pub fn is_main_menu(&self) -> bool {
        self.state == MenuState::MainMenu
    }

    /// Get current menu state
    pub fn current_state(&self) -> MenuState {
        self.state
    }

    /// Apply settings changes (saves to disk and updates original)
    pub fn apply_settings(&mut self) {
        // Save to disk
        if let Err(e) = crate::persistence::save_settings(&self.settings) {
            log::error!("Failed to save settings: {}", e);
        }
        
        // Update original state
        self.settings_original = self.settings.clone();
        // In future: Apply settings to window/renderer here
    }

    /// Revert settings changes (restore from original, does not save)
    pub fn revert_settings(&mut self) {
        self.settings = self.settings_original.clone();
    }

    /// Check if settings have been modified
    pub fn settings_modified(&self) -> bool {
        self.settings != self.settings_original
    }

    /// Reset controls to default key bindings
    pub fn reset_controls_to_default(&mut self) {
        self.settings.controls = ControlsSettings::default();
    }
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}
