//! Docking Layout Manager
//!
//! Provides professional dockable panel layout using `egui_dock`.
//! Supports drag-and-drop panel repositioning, tab grouping,
//! split layouts, and layout persistence.
//!
//! # Architecture
//!
//! The layout system consists of:
//! - `DockLayout`: Main layout manager holding `egui_dock::DockState`
//! - `EditorTabViewer`: Implements `egui_dock::TabViewer` for panel rendering
//! - `LayoutPreset`: Predefined layouts (Default, Wide, Compact, etc.)
//!
//! # Example
//!
//! ```ignore
//! use aw_editor::dock_layout::{DockLayout, LayoutPreset};
//!
//! // Create default layout
//! let mut layout = DockLayout::new();
//!
//! // Or load from preset
//! let layout = DockLayout::from_preset(LayoutPreset::Wide);
//!
//! // Show in egui
//! layout.show(ctx, &mut app_state);
//! ```

use crate::panel_type::PanelType;
use egui_dock::egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabIndex, TabViewer};
use serde::{Deserialize, Serialize};

/// Predefined layout configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutPreset {
    /// Default layout: Left (Hierarchy/Assets), Center (Viewport), Right (Inspector), Bottom (Console)
    Default,
    /// Wide layout: Maximized viewport with minimal side panels
    Wide,
    /// Compact layout: All panels visible in smaller configuration
    Compact,
    /// Modeling layout: Large viewport with transform tools
    Modeling,
    /// Animation layout: Timeline at bottom, graph on side
    Animation,
    /// Debug layout: Console and profiler prominent
    Debug,
}

impl Default for LayoutPreset {
    fn default() -> Self {
        Self::Default
    }
}

/// Docking layout manager
///
/// Manages the `egui_dock::DockState` and provides methods
/// for creating, saving, and restoring panel layouts.
pub struct DockLayout {
    /// The dock state containing all panel positions
    dock_state: DockState<PanelType>,

    /// Currently focused panel (if any)
    focused_panel: Option<PanelType>,

    /// Style configuration for the dock area
    style: Style,
}

impl Default for DockLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DockLayout {
    /// Create a new dock layout with default configuration
    pub fn new() -> Self {
        let dock_state = Self::create_default_layout();

        Self {
            dock_state,
            focused_panel: None,
            style: Self::create_editor_style(),
        }
    }

    /// Create layout from a preset configuration
    pub fn from_preset(preset: LayoutPreset) -> Self {
        let dock_state = match preset {
            LayoutPreset::Default => Self::create_default_layout(),
            LayoutPreset::Wide => Self::create_wide_layout(),
            LayoutPreset::Compact => Self::create_compact_layout(),
            LayoutPreset::Modeling => Self::create_modeling_layout(),
            LayoutPreset::Animation => Self::create_animation_layout(),
            LayoutPreset::Debug => Self::create_debug_layout(),
        };

        Self {
            dock_state,
            focused_panel: None,
            style: Self::create_editor_style(),
        }
    }

    /// Get mutable reference to the dock state
    pub fn dock_state_mut(&mut self) -> &mut DockState<PanelType> {
        &mut self.dock_state
    }

    /// Get reference to the dock state
    pub fn dock_state(&self) -> &DockState<PanelType> {
        &self.dock_state
    }

    /// Get reference to the dock style
    pub fn style(&self) -> &Style {
        &self.style
    }

    /// Get mutable reference to the dock style
    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Get the currently focused panel
    pub fn focused_panel(&self) -> Option<PanelType> {
        self.focused_panel
    }

    /// Set the focused panel
    pub fn set_focused_panel(&mut self, panel: Option<PanelType>) {
        self.focused_panel = panel;
    }

    /// Add a panel to the layout
    ///
    /// If the panel already exists, it will be focused.
    /// Otherwise, it will be added to the right side.
    pub fn add_panel(&mut self, panel: PanelType) {
        // Check if panel already exists
        if self.find_panel(&panel).is_some() {
            // Focus existing panel
            self.focused_panel = Some(panel);
            return;
        }

        // Add to main surface at root
        let surface = self.dock_state.main_surface_mut();
        surface.push_to_focused_leaf(panel);
    }

    /// Remove a panel from the layout
    ///
    /// Returns true if the panel was removed, false if it wasn't found
    /// or couldn't be removed (e.g., Viewport is not closable).
    pub fn remove_panel(&mut self, panel: &PanelType) -> bool {
        if !panel.is_closable() {
            return false;
        }

        if let Some((surface_idx, node_idx, tab_idx)) = self.find_panel(panel) {
            self.dock_state.remove_tab((surface_idx, node_idx, tab_idx));
            return true;
        }

        false
    }

    /// Find a panel in the layout
    ///
    /// Returns the surface, node, and tab indices if found.
    pub fn find_panel(&self, panel: &PanelType) -> Option<(SurfaceIndex, NodeIndex, TabIndex)> {
        self.dock_state.find_tab(panel)
    }

    /// Check if a panel is visible in the layout
    pub fn is_panel_visible(&self, panel: &PanelType) -> bool {
        self.find_panel(panel).is_some()
    }

    /// Alias for is_panel_visible - checks if panel exists in layout
    pub fn has_panel(&self, panel: &PanelType) -> bool {
        self.is_panel_visible(panel)
    }

    /// Toggle a panel's visibility
    pub fn toggle_panel(&mut self, panel: PanelType) {
        if self.is_panel_visible(&panel) {
            self.remove_panel(&panel);
        } else {
            self.add_panel(panel);
        }
    }

    /// Reset layout to default configuration
    pub fn reset_to_default(&mut self) {
        self.dock_state = Self::create_default_layout();
    }

    /// Apply a preset layout
    pub fn apply_preset(&mut self, preset: LayoutPreset) {
        self.dock_state = match preset {
            LayoutPreset::Default => Self::create_default_layout(),
            LayoutPreset::Wide => Self::create_wide_layout(),
            LayoutPreset::Compact => Self::create_compact_layout(),
            LayoutPreset::Modeling => Self::create_modeling_layout(),
            LayoutPreset::Animation => Self::create_animation_layout(),
            LayoutPreset::Debug => Self::create_debug_layout(),
        };
    }

    /// Serialize the layout to JSON for persistence
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        // Note: DockState serialization requires custom handling
        // For now, we'll serialize the panel list and positions
        let layout_data = LayoutData {
            panels: self.collect_visible_panels(),
        };
        serde_json::to_string_pretty(&layout_data)
    }

    /// Deserialize layout from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let _layout_data: LayoutData = serde_json::from_str(json)?;
        // For now, return default layout
        // Full restoration requires more complex DockState serialization
        Ok(Self::new())
    }

    /// Collect list of all visible panels
    fn collect_visible_panels(&self) -> Vec<PanelType> {
        self.dock_state
            .iter_all_tabs()
            .map(|(_, tab)| *tab)
            .collect()
    }

    // === Layout Creation Methods ===

    /// Create the default layout
    ///
    /// Layout structure:
    /// ```text
    /// +----------------------------------------+
    /// |                   |  Inspector         |
    /// |     Viewport      |  Transform         |
    /// |                   |                    |
    /// +-------------------+--------------------+
    /// |      Console | Profiler | Stats        |
    /// +----------------------------------------+
    /// ```
    /// Note: Left panels (Hierarchy, Assets) are handled by the legacy
    /// Astract Panels side panel, not the docking system.
    fn create_default_layout() -> DockState<PanelType> {
        let mut dock_state = DockState::new(vec![PanelType::Viewport]);
        let surface = dock_state.main_surface_mut();

        // No left panel here - already handled by legacy Astract Panels side panel

        // Split right panel (18% width for inspector/transform)
        let [_center, _right_panel] = surface.split_right(
            NodeIndex::root(),
            0.82,
            vec![PanelType::Inspector, PanelType::Transform],
        );

        // Split bottom panel (20% height for console/profiler)
        let [_top, _bottom] = surface.split_below(
            NodeIndex::root(),
            0.80,
            vec![
                PanelType::Console,
                PanelType::Profiler,
                PanelType::SceneStats,
            ],
        );

        dock_state
    }

    /// Create wide layout (maximized viewport)
    fn create_wide_layout() -> DockState<PanelType> {
        let mut dock_state = DockState::new(vec![PanelType::Viewport]);
        let surface = dock_state.main_surface_mut();

        // No left panel - handled by legacy Astract Panels

        // Minimal right panel (15%)
        let [_center, _right_panel] =
            surface.split_right(NodeIndex::root(), 0.85, vec![PanelType::Inspector]);

        dock_state
    }

    /// Create compact layout (all panels visible)
    fn create_compact_layout() -> DockState<PanelType> {
        let mut dock_state = DockState::new(vec![PanelType::Viewport]);
        let surface = dock_state.main_surface_mut();

        // No left panel - handled by legacy Astract Panels

        // Right panel (25%)
        let [_center, _right_panel] = surface.split_right(
            NodeIndex::root(),
            0.75,
            vec![
                PanelType::Inspector,
                PanelType::Transform,
                PanelType::EntityPanel,
            ],
        );

        // Bottom panel (30%)
        let [_top, _bottom] = surface.split_below(
            NodeIndex::root(),
            0.70,
            vec![
                PanelType::Console,
                PanelType::Profiler,
                PanelType::SceneStats,
                PanelType::Performance,
            ],
        );

        dock_state
    }

    /// Create modeling layout
    fn create_modeling_layout() -> DockState<PanelType> {
        let mut dock_state = DockState::new(vec![PanelType::Viewport]);
        let surface = dock_state.main_surface_mut();

        // No left panel - handled by legacy Astract Panels

        // Right panel with transform tools (20%)
        let [_center, _right_panel] = surface.split_right(
            NodeIndex::root(),
            0.80,
            vec![PanelType::Transform, PanelType::Inspector],
        );

        dock_state
    }

    /// Create animation layout
    fn create_animation_layout() -> DockState<PanelType> {
        let mut dock_state = DockState::new(vec![PanelType::Viewport]);
        let surface = dock_state.main_surface_mut();

        // No left panel - handled by legacy Astract Panels

        // Right panel with animation graph (25%)
        let [_center, _right_panel] = surface.split_right(
            NodeIndex::root(),
            0.75,
            vec![PanelType::BehaviorGraph, PanelType::Inspector],
        );

        // Large bottom panel for timeline (35%)
        let [_top, _bottom] = surface.split_below(
            NodeIndex::root(),
            0.65,
            vec![PanelType::Animation, PanelType::Graph],
        );

        dock_state
    }

    /// Create debug layout
    fn create_debug_layout() -> DockState<PanelType> {
        let mut dock_state = DockState::new(vec![PanelType::Viewport]);
        let surface = dock_state.main_surface_mut();

        // No left panel - handled by legacy Astract Panels

        // Right panel with performance (25%)
        let [_center, _right_panel] = surface.split_right(
            NodeIndex::root(),
            0.75,
            vec![PanelType::Performance, PanelType::SceneStats],
        );

        // Large bottom panel for console/profiler (40%)
        let [_top, _bottom] = surface.split_below(
            NodeIndex::root(),
            0.60,
            vec![PanelType::Console, PanelType::Profiler],
        );

        dock_state
    }

    /// Create editor-specific dock style
    fn create_editor_style() -> Style {
        let mut style = Style::from_egui(&egui::Style::default());

        // Customize tab bar appearance
        style.tab_bar.height = 24.0;
        // Note: show_scroll_buttons_on_hover removed in egui_dock 0.17

        // Customize tab appearance
        style.tab.tab_body.inner_margin = egui::Margin::symmetric(8, 4);
        // Tab body uses default theme colors

        // Separator styling
        style.separator.width = 4.0;
        // Separator uses default theme colors for idle/hover states

        // Tab bar uses default theme background

        style
    }

    /// Show the dock layout using a TabViewer implementation
    ///
    /// This renders the entire docking area with all panels.
    /// The `tab_viewer` parameter controls how each panel is rendered.
    pub fn show<T: TabViewer<Tab = PanelType>>(&mut self, ctx: &egui::Context, tab_viewer: &mut T) {
        // Update style from current egui style for theme consistency
        self.style = Style::from_egui(ctx.style().as_ref());
        self.style.tab_bar.height = 24.0;
        self.style.tab.tab_body.inner_margin = egui::Margin::symmetric(8, 4);
        self.style.separator.width = 4.0;
        // Production theme uses default egui_dock colors derived from egui style

        DockArea::new(&mut self.dock_state)
            .style(self.style.clone())
            .show(ctx, tab_viewer);
    }

    /// Show the dock layout inside a UI region
    ///
    /// Use this when embedding the dock in an existing panel.
    pub fn show_inside<T: TabViewer<Tab = PanelType>>(
        &mut self,
        ui: &mut egui::Ui,
        tab_viewer: &mut T,
    ) {
        // Update style from current egui style for theme consistency
        self.style = Style::from_egui(ui.style().as_ref());
        self.style.tab_bar.height = 24.0;
        self.style.tab.tab_body.inner_margin = egui::Margin::symmetric(8, 4);
        self.style.separator.width = 4.0;
        // Production theme uses default egui_dock colors derived from egui style

        DockArea::new(&mut self.dock_state)
            .style(self.style.clone())
            .show_inside(ui, tab_viewer);
    }
}

/// Serializable layout data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LayoutData {
    panels: Vec<PanelType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_layout_creation() {
        let layout = DockLayout::new();

        // Should have viewport
        assert!(layout.is_panel_visible(&PanelType::Viewport));

        // Should have hierarchy
        assert!(layout.is_panel_visible(&PanelType::Hierarchy));

        // Should have console
        assert!(layout.is_panel_visible(&PanelType::Console));
    }

    #[test]
    fn test_preset_layouts() {
        for preset in [
            LayoutPreset::Default,
            LayoutPreset::Wide,
            LayoutPreset::Compact,
            LayoutPreset::Modeling,
            LayoutPreset::Animation,
            LayoutPreset::Debug,
        ] {
            let layout = DockLayout::from_preset(preset);
            // All presets should have viewport
            assert!(layout.is_panel_visible(&PanelType::Viewport));
        }
    }

    #[test]
    fn test_add_panel() {
        let mut layout = DockLayout::new();

        // Add a panel that might not be visible
        layout.add_panel(PanelType::Charts);

        // Should now be visible
        assert!(layout.is_panel_visible(&PanelType::Charts));
    }

    #[test]
    fn test_toggle_panel() {
        let mut layout = DockLayout::new();

        // Console should be visible by default
        assert!(layout.is_panel_visible(&PanelType::Console));

        // Toggle off
        layout.toggle_panel(PanelType::Console);
        assert!(!layout.is_panel_visible(&PanelType::Console));

        // Toggle on
        layout.toggle_panel(PanelType::Console);
        assert!(layout.is_panel_visible(&PanelType::Console));
    }

    #[test]
    fn test_viewport_not_closable() {
        let mut layout = DockLayout::new();

        // Try to remove viewport
        let removed = layout.remove_panel(&PanelType::Viewport);

        // Should fail because viewport is not closable
        assert!(!removed);
        assert!(layout.is_panel_visible(&PanelType::Viewport));
    }

    #[test]
    fn test_reset_to_default() {
        let mut layout = DockLayout::from_preset(LayoutPreset::Wide);

        // Reset
        layout.reset_to_default();

        // Should have default panels
        assert!(layout.is_panel_visible(&PanelType::Console));
        assert!(layout.is_panel_visible(&PanelType::AssetBrowser));
    }
}
