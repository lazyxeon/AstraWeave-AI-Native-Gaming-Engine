//! Panel Type Definitions for Dockable Editor Panels
//!
//! This module defines the `PanelType` enum that represents all available
//! editor panels. Each variant corresponds to a panel that can be docked,
//! undocked, tabbed, or hidden within the editor layout.
//!
//! # Architecture
//!
//! The panel system integrates with `egui_dock` to provide:
//! - Drag-and-drop panel repositioning
//! - Tab grouping (multiple panels in same area)
//! - Split layouts (horizontal/vertical)
//! - Layout persistence across sessions
//!
//! # Example
//!
//! ```
//! use aw_editor_lib::panel_type::PanelType;
//!
//! // Get display title for UI
//! let panel = PanelType::Hierarchy;
//! assert_eq!(panel.title(), "Hierarchy");
//!
//! // Check if panel is closable
//! assert!(panel.is_closable());
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Enumeration of all available editor panels
///
/// Each panel type represents a distinct functional area of the editor.
/// Panels can be docked in various configurations using `egui_dock`.
///
/// # Panel Categories
///
/// - **Scene Panels**: Hierarchy, Inspector, Viewport, World
/// - **Asset Panels**: AssetBrowser
/// - **Debug Panels**: Console, Profiler, Performance, SceneStats
/// - **Tool Panels**: Transform, Animation, Graph, Charts, AdvancedWidgets
/// - **System Panels**: BuildManager, ThemeManager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelType {
    /// Scene hierarchy tree showing entity parent-child relationships
    Hierarchy,

    /// Property inspector for selected entities/assets
    Inspector,

    /// Asset browser for browsing and managing project assets
    AssetBrowser,

    /// 3D viewport for scene editing and visualization
    Viewport,

    /// Console output for logs, warnings, and errors
    Console,

    /// Performance profiler showing frame timing and system metrics
    Profiler,

    /// Build manager for project compilation and export
    BuildManager,

    /// Scene statistics (entity count, memory usage, etc.)
    SceneStats,

    /// Transform editing panel (position, rotation, scale)
    Transform,

    /// Performance monitoring graphs
    Performance,

    /// Data visualization charts
    Charts,

    /// Advanced widgets panel (color pickers, curves, etc.)
    AdvancedWidgets,

    /// Node graph editor for behavior/shader graphs
    Graph,

    /// Animation timeline and keyframe editor
    Animation,

    /// Theme manager for editor appearance customization
    ThemeManager,

    /// World settings and environment configuration
    World,

    /// Material editor with PBR preview
    MaterialEditor,

    /// Entity properties panel (alternative inspector view)
    EntityPanel,

    /// Behavior graph editor for AI logic
    BehaviorGraph,

    /// Terrain generation and editing panel
    Terrain,
}

impl PanelType {
    /// Returns the display title for this panel
    ///
    /// This title is shown in tab headers and window titles.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Hierarchy => "Hierarchy",
            Self::Inspector => "Inspector",
            Self::AssetBrowser => "Asset Browser",
            Self::Viewport => "Viewport",
            Self::Console => "Console",
            Self::Profiler => "Profiler",
            Self::BuildManager => "Build Manager",
            Self::SceneStats => "Scene Stats",
            Self::Transform => "Transform",
            Self::Performance => "Performance",
            Self::Charts => "Charts",
            Self::AdvancedWidgets => "Advanced Widgets",
            Self::Graph => "Graph",
            Self::Animation => "Animation",
            Self::ThemeManager => "Theme Manager",
            Self::World => "World",
            Self::MaterialEditor => "Material Editor",
            Self::EntityPanel => "Entity",
            Self::BehaviorGraph => "Behavior Graph",
            Self::Terrain => "Terrain",
        }
    }

    /// Returns a short icon representation for this panel
    ///
    /// Icons are used in compact tab views and toolbar buttons.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Hierarchy => "🌳",
            Self::Inspector => "🔍",
            Self::AssetBrowser => "📁",
            Self::Viewport => "🎬",
            Self::Console => "📋",
            Self::Profiler => "📊",
            Self::BuildManager => "🔨",
            Self::SceneStats => "📈",
            Self::Transform => "🔄",
            Self::Performance => "⚡",
            Self::Charts => "📉",
            Self::AdvancedWidgets => "🎨",
            Self::Graph => "🔗",
            Self::Animation => "🎞️",
            Self::ThemeManager => "🎭",
            Self::World => "🌍",
            Self::MaterialEditor => "🧱",
            Self::EntityPanel => "📦",
            Self::BehaviorGraph => "🧠",
            Self::Terrain => "🏔️",
        }
    }

    /// Returns whether this panel can be closed by the user
    ///
    /// Some panels (like Viewport) may be marked as not closable
    /// to ensure the editor always has a main work area.
    pub fn is_closable(&self) -> bool {
        match self {
            // Viewport should always be visible
            Self::Viewport => false,
            // All other panels can be closed
            _ => true,
        }
    }

    /// Returns whether this panel should have scroll bars
    pub fn has_scroll(&self) -> bool {
        match self {
            // Viewport handles its own scrolling
            Self::Viewport => false,
            // Graph panels have their own pan/zoom
            Self::Graph | Self::BehaviorGraph => false,
            // All other panels should scroll
            _ => true,
        }
    }

    /// Returns a list of all available panel types
    pub fn all() -> &'static [PanelType] {
        &[
            Self::Hierarchy,
            Self::Inspector,
            Self::AssetBrowser,
            Self::Viewport,
            Self::Console,
            Self::Profiler,
            Self::BuildManager,
            Self::SceneStats,
            Self::Transform,
            Self::Performance,
            Self::Charts,
            Self::AdvancedWidgets,
            Self::Graph,
            Self::Animation,
            Self::ThemeManager,
            Self::World,
            Self::MaterialEditor,
            Self::EntityPanel,
            Self::BehaviorGraph,
            Self::Terrain,
        ]
    }

    /// Returns the default panels for the left dock area
    pub fn default_left_panels() -> Vec<PanelType> {
        vec![Self::Hierarchy, Self::AssetBrowser]
    }

    /// Returns the default panels for the right dock area
    pub fn default_right_panels() -> Vec<PanelType> {
        vec![Self::Inspector, Self::Transform]
    }

    /// Returns the default panels for the bottom dock area
    pub fn default_bottom_panels() -> Vec<PanelType> {
        vec![Self::Console, Self::Profiler, Self::SceneStats]
    }

    /// Returns the default panel for the center area
    pub fn default_center_panel() -> PanelType {
        Self::Viewport
    }
}

impl fmt::Display for PanelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.icon(), self.title())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_type_title() {
        assert_eq!(PanelType::Hierarchy.title(), "Hierarchy");
        assert_eq!(PanelType::Viewport.title(), "Viewport");
        assert_eq!(PanelType::AssetBrowser.title(), "Asset Browser");
    }

    #[test]
    fn test_panel_type_icon() {
        assert_eq!(PanelType::Hierarchy.icon(), "🌳");
        assert_eq!(PanelType::Viewport.icon(), "🎬");
        assert_eq!(PanelType::Console.icon(), "📋");
    }

    #[test]
    fn test_panel_type_closable() {
        assert!(!PanelType::Viewport.is_closable());
        assert!(PanelType::Console.is_closable());
        assert!(PanelType::Inspector.is_closable());
    }

    #[test]
    fn test_panel_type_all() {
        let all = PanelType::all();
        assert!(all.len() >= 16, "Should have at least 16 panel types");
        assert!(all.contains(&PanelType::Viewport));
        assert!(all.contains(&PanelType::Hierarchy));
    }

    #[test]
    fn test_panel_type_display() {
        let panel = PanelType::Hierarchy;
        let display = format!("{}", panel);
        assert!(display.contains("Hierarchy"));
        assert!(display.contains("🌳"));
    }

    #[test]
    fn test_panel_type_serialization() {
        let panel = PanelType::Inspector;
        let json = serde_json::to_string(&panel).unwrap();
        let deserialized: PanelType = serde_json::from_str(&json).unwrap();
        assert_eq!(panel, deserialized);
    }

    #[test]
    fn test_default_layout() {
        let left = PanelType::default_left_panels();
        let right = PanelType::default_right_panels();
        let bottom = PanelType::default_bottom_panels();
        let center = PanelType::default_center_panel();

        assert!(left.contains(&PanelType::Hierarchy));
        assert!(right.contains(&PanelType::Inspector));
        assert!(bottom.contains(&PanelType::Console));
        assert_eq!(center, PanelType::Viewport);
    }
}
