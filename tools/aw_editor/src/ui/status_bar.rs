use crate::command::UndoStack;
use crate::editor_mode::EditorMode;
use crate::entity_manager::SelectionSet;
use crate::gizmo::snapping::SnappingConfig;
use crate::gizmo::state::GizmoMode;
use egui::{Align, Layout, Ui};

/// Status bar component for the bottom of the editor
///
/// Shows:
/// - Editor mode (Edit/Play/Paused)
/// - Current gizmo mode (Translate/Rotate/Scale)
/// - Selection count
/// - Undo/redo state
/// - FPS counter
/// - Snap settings
pub struct StatusBar;

impl StatusBar {
    /// Render the status bar
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        editor_mode: &EditorMode,
        gizmo_mode: &GizmoMode,
        selection: &SelectionSet,
        undo_stack: &UndoStack,
        snap_config: &SnappingConfig,
        fps: f32,
        is_dirty: bool,
        entity_count: usize,
        scene_path: Option<&str>,
    ) {
        ui.horizontal(|ui| {
            if is_dirty {
                ui.label(
                    egui::RichText::new("*")
                        .color(egui::Color32::from_rgb(255, 100, 100))
                        .strong(),
                )
                .on_hover_text("Unsaved changes - Press Ctrl+S to save");
            }

            Self::show_editor_mode(ui, editor_mode);
            ui.separator();

            Self::show_gizmo_mode(ui, gizmo_mode);
            ui.separator();

            Self::show_selection(ui, selection);
            ui.separator();

            Self::show_undo_redo(ui, undo_stack);
            ui.separator();

            ui.label(format!("Entities: {}", entity_count))
                .on_hover_text("Total number of entities in the scene");
            ui.separator();

            if let Some(path) = scene_path {
                ui.label(format!("📁 {}", path))
                    .on_hover_text(format!("Scene file: {}", path));
            } else {
                ui.label("📁 Untitled")
                    .on_hover_text("Scene not saved - Press Ctrl+S to save");
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Self::show_fps(ui, fps);
                ui.separator();

                Self::show_snap_settings(ui, snap_config);
            });
        });
    }

    fn show_editor_mode(ui: &mut Ui, mode: &EditorMode) {
        let status_label = egui::RichText::new(mode.status_text())
            .color(mode.status_color())
            .strong();

        let hotkey_text = match mode {
            EditorMode::Edit => "Press F5 to Play",
            EditorMode::Play => "F6 to Pause, F7 to Stop",
            EditorMode::Paused => "F5 to Resume, F7 to Stop",
        };

        ui.label(status_label).on_hover_text(hotkey_text);
    }

    fn show_gizmo_mode(ui: &mut Ui, mode: &GizmoMode) {
        let (icon, text, hotkey) = match mode {
            GizmoMode::Inactive => ("⏸️", "Inactive", "ESC"),
            GizmoMode::Translate { .. } => ("🔀", "Translate", "G"),
            GizmoMode::Rotate { .. } => ("🔄", "Rotate", "R"),
            GizmoMode::Scale { .. } => ("📏", "Scale", "S"),
        };

        ui.label(format!("{} {} ({})", icon, text, hotkey))
            .on_hover_text(format!("Press {} to switch to {} mode", hotkey, text));
    }

    fn show_selection(ui: &mut Ui, selection: &SelectionSet) {
        let count = selection.count();

        if count == 0 {
            ui.label("Nothing selected");
        } else if count == 1 {
            ui.label("1 entity selected");
        } else {
            ui.label(format!("{} entities selected", count))
                .on_hover_text("Use Ctrl+Click to toggle selection, Shift+Click for range");
        }
    }

    fn show_undo_redo(ui: &mut Ui, undo_stack: &UndoStack) {
        let undo_count = undo_stack.undo_count();
        let redo_count = undo_stack.redo_count();

        if undo_count > 0 {
            let desc = undo_stack.undo_description().unwrap_or_default();
            ui.label(format!("⏮️ Undo ({}): {}", undo_count, desc))
                .on_hover_text("Ctrl+Z to undo");
        } else {
            ui.label("⏮️ Undo (0)")
                .on_hover_text("Make some changes to enable undo");
        }

        ui.add_space(8.0);

        if redo_count > 0 {
            let desc = undo_stack.redo_description().unwrap_or_default();
            ui.label(format!("⏭️ Redo ({}): {}", redo_count, desc))
                .on_hover_text("Ctrl+Y to redo");
        } else {
            ui.label("⏭️ Redo (0)")
                .on_hover_text("Undo something to enable redo");
        }
    }

    fn show_snap_settings(ui: &mut Ui, snap: &SnappingConfig) {
        if snap.grid_enabled {
            ui.label(format!("🔲 Grid: {:.1}u", snap.grid_size))
                .on_hover_text("Grid snapping enabled - Hold Ctrl to snap positions");
        }

        if snap.angle_enabled {
            ui.label(format!("🔄 Angle: {:.0}°", snap.angle_increment))
                .on_hover_text("Angle snapping enabled - Hold Ctrl to snap rotations");
        }

        if !snap.grid_enabled && !snap.angle_enabled {
            ui.label("⚡ Snap: OFF")
                .on_hover_text("Press S to toggle snapping");
        }
    }

    fn show_fps(ui: &mut Ui, fps: f32) {
        let color = if fps >= 55.0 {
            egui::Color32::GREEN
        } else if fps >= 30.0 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::RED
        };

        ui.colored_label(color, format!("FPS: {:.0}", fps))
            .on_hover_text("Target: 60 FPS for smooth editing");
    }
}

#[cfg(test)]
mod tests {
    use crate::command::UndoStack;
    use crate::entity_manager::SelectionSet;
    use crate::gizmo::snapping::SnappingConfig;
    use crate::gizmo::{state::GizmoMode, AxisConstraint};

    #[test]
    fn test_status_bar_creation() {
        let selection = SelectionSet::new();
        let undo_stack = UndoStack::new(100);
        let gizmo_mode = GizmoMode::Translate {
            constraint: AxisConstraint::None,
        };
        let snap_config = SnappingConfig::default();

        assert_eq!(selection.count(), 0);
        assert!(!undo_stack.can_undo());
        assert!(!undo_stack.can_redo());

        assert_eq!(
            gizmo_mode,
            GizmoMode::Translate {
                constraint: AxisConstraint::None
            }
        );
        assert!(snap_config.grid_enabled);
    }
}
