//! # AstraWeave UI
//!
//! In-game UI framework for AstraWeave, built on **egui**.
//!
//! Provides a complete UI stack for game development:
//!
//! - **Menus** — Main menu, pause menu, and settings panels (graphics, audio, controls)
//!   with TOML persistence ([`persistence::save_settings`], [`persistence::load_settings`]).
//! - **HUD** — Health bars (smooth transitions, flash/glow), damage numbers (arc motion,
//!   combos), quest tracker, minimap with POI markers, dialogue system with branching
//!   choices, and tooltip overlays.
//! - **Accessibility** — Colorblind modes (protanopia, deuteranopia, tritanopia),
//!   high-contrast support, and configurable color transforms.
//! - **Gamepad** — Controller support via gilrs with configurable bindings.
//!
//! # Key Types
//!
//! - [`hud::HudManager`] — Central HUD state and rendering coordinator.
//! - [`menu::MenuManager`] — Menu navigation state machine.
//! - [`panels::draw_ui`] — Main rendering entry point.
//! - [`accessibility::AccessibilitySettings`] — Colorblind and contrast options.
//! - [`gamepad::GamepadManager`] — Controller input handling.

pub mod accessibility;
pub mod gamepad;
pub mod hud;
pub mod layer;
pub mod menu;
pub mod menus;
pub mod panels;
pub mod persistence;
pub mod state;

#[cfg(test)]
mod mutation_tests;

pub use hud::{
    DamageNumber,
    DamageType,
    DialogueChoice,
    // Week 3 Day 4: Dialogue & tooltips
    DialogueNode,
    EnemyData,
    EnemyFaction,
    HudManager,
    HudState,
    Objective,
    PlayerStats,
    PoiMarker,
    PoiType,
    // Week 3 Day 3: Quest tracker & minimap
    Quest,
    TooltipData,
};
pub use layer::UiLayer;
pub use menu::{
    AudioSettings, ControlsSettings, GraphicsSettings, MenuAction, MenuManager, MenuState,
    QualityPreset, SettingsState,
};
pub use panels::draw_ui;
pub use persistence::{get_config_path, load_settings, save_settings};
pub use state::{Accessibility, UiData, UiFlags};

// Week 5: Controller and Accessibility exports
pub use accessibility::{
    colors, get_health_colors, to_egui_color, transform_color, AccessibilitySettings,
    ColorblindMode,
};
pub use gamepad::{GamepadAction, GamepadBindings, GamepadManager};

// Re-export egui for external use
pub use egui;
