pub mod layer;
pub mod panels;
pub mod state;
pub mod menu;
pub mod menus;
pub mod persistence;
pub mod hud;

pub use layer::UiLayer;
pub use panels::draw_ui;
pub use state::{Accessibility, UiData, UiFlags};
pub use menu::{
    AudioSettings, ControlsSettings, GraphicsSettings, MenuAction, MenuManager, MenuState,
    QualityPreset, SettingsState,
};
pub use persistence::{load_settings, save_settings, get_config_path};
pub use hud::{
    HudManager, HudState, PlayerStats, EnemyData, EnemyFaction, DamageNumber, DamageType,
    // Week 3 Day 3: Quest tracker & minimap
    Quest, Objective, PoiMarker, PoiType,
    // Week 3 Day 4: Dialogue & tooltips
    DialogueNode, DialogueChoice, TooltipData,
};

// Re-export egui for external use
pub use egui;
