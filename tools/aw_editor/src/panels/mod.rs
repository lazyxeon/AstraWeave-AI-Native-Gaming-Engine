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
pub mod asset_browser;
pub mod charts_panel;
pub mod entity_panel;
pub mod graph_panel;
pub mod hierarchy_panel;
pub mod performance_panel;
pub mod transform_panel;
pub mod world_panel;

// Re-exports
pub use advanced_widgets_panel::AdvancedWidgetsPanel;
pub use animation::AnimationPanel;
pub use asset_browser::AssetBrowser;
pub use charts_panel::ChartsPanel;
pub use entity_panel::EntityPanel;
pub use graph_panel::GraphPanel;
pub use hierarchy_panel::HierarchyPanel;
pub use performance_panel::PerformancePanel;
pub use transform_panel::TransformPanel;
pub use world_panel::WorldPanel;
