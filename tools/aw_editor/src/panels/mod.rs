// tools/aw_editor/src/panels/mod.rs - Panel trait and module exports

use egui::Ui;

/// Trait for reusable editor panels
///
/// Panels are self-contained UI components that can be shown in
/// side panels, windows, or central areas.
pub trait Panel {
    /// Panel display name (shown in tabs/titles)
    fn name(&self) -> &str;

    /// Render the panel's UI
    fn show(&mut self, ui: &mut Ui);

    /// Optional: Called each frame for updates (e.g., polling data)
    fn update(&mut self) {}
}

// Panel implementations
pub mod advanced_widgets_panel;
pub mod animation;
pub mod animation_panel;
pub mod asset_browser;
pub mod audio_panel;
pub mod build_manager;
pub mod charts_panel;
pub mod cinematics_panel;
pub mod console_panel;
pub mod dialogue_editor_panel;
pub mod distribution_panel;
pub mod entity_panel;
pub mod foliage_panel;
pub mod graph_panel;
pub mod hierarchy_panel;
pub mod input_bindings_panel;
pub mod lighting_panel;
pub mod localization_panel;
pub mod lod_config_panel;
pub mod material_editor_panel;
pub mod navigation_panel;
pub mod networking_panel;
pub mod particle_system_panel;
pub mod pcg_panel;
pub mod performance_panel;
pub mod physics_panel;
pub mod polish_panel;
pub mod post_process_panel;
pub mod profiler_panel;
pub mod project_settings_panel;
pub mod scene_stats_panel;
pub mod spline_editor_panel;
pub mod terrain_panel;
pub mod theme_manager;
pub mod transform_panel;
pub mod ui_editor_panel;
pub mod world_panel;

// Re-exports
pub use advanced_widgets_panel::AdvancedWidgetsPanel;
pub use animation::AnimationPanel;
pub use asset_browser::{AssetAction, AssetBrowser, TextureType};
pub use build_manager::BuildManagerPanel;
pub use charts_panel::ChartsPanel;
pub use console_panel::ConsolePanel;
pub use entity_panel::EntityPanel;
pub use graph_panel::GraphPanel;
pub use hierarchy_panel::HierarchyPanel;
pub use performance_panel::PerformancePanel;
pub use profiler_panel::ProfilerPanel;
pub use scene_stats_panel::{SceneStats, SceneStatsPanel};
pub use terrain_panel::TerrainPanel;
pub use theme_manager::ThemeManagerPanel;
pub use transform_panel::TransformPanel;
pub use world_panel::WorldPanel;

// New panel re-exports (Phase 8 additions)
pub use audio_panel::AudioPanel;
pub use cinematics_panel::CinematicsPanel;
pub use dialogue_editor_panel::DialogueEditorPanel;
pub use foliage_panel::FoliagePanel;
pub use input_bindings_panel::InputBindingsPanel;
pub use lighting_panel::LightingPanel;
pub use localization_panel::LocalizationPanel;
pub use lod_config_panel::LodConfigPanel;
pub use material_editor_panel::MaterialEditorPanel;
pub use navigation_panel::NavigationPanel;
pub use networking_panel::NetworkingPanel;
pub use particle_system_panel::ParticleSystemPanel;
pub use pcg_panel::PcgPanel;
pub use physics_panel::PhysicsPanel;
#[allow(unused_imports)]
pub use polish_panel::PolishPanel;
pub use post_process_panel::PostProcessPanel;
pub use project_settings_panel::ProjectSettingsPanel;
pub use spline_editor_panel::SplineEditorPanel;
pub use ui_editor_panel::UiEditorPanel;
