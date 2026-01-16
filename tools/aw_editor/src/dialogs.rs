//! Dialog rendering for the AstraWeave Editor
//!
//! This module contains helper functions for rendering modal dialogs
//! such as quit confirmation, new scene confirmation, settings, and help.

use eframe::egui;

/// Helper to render a modal overlay (semi-transparent background)
pub fn show_modal_overlay(ctx: &egui::Context, id: &str) {
    let screen_rect = ctx.screen_rect();
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Background)
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128),
            );
        });
}

/// Keyboard shortcuts data for help and settings dialogs
pub const KEYBOARD_SHORTCUTS: &[(&str, &str, &str)] = &[
    // Category, Shortcut, Description
    ("File", "Ctrl+N", "New Scene"),
    ("File", "Ctrl+S", "Save Scene"),
    ("File", "Ctrl+Shift+S", "Save As"),
    ("File", "Ctrl+O", "Open Scene"),
    ("Edit", "Ctrl+Z", "Undo"),
    ("Edit", "Ctrl+Y", "Redo"),
    ("Edit", "Ctrl+Shift+Z", "Redo (Alt)"),
    ("Edit", "Ctrl+A", "Select All"),
    ("Edit", "Ctrl+D", "Duplicate"),
    ("Edit", "Ctrl+C", "Copy"),
    ("Edit", "Ctrl+V", "Paste"),
    ("Edit", "Delete", "Delete Selected"),
    ("Edit", "Escape", "Deselect All"),
    ("Camera", "F", "Focus on Selected"),
    ("Camera", "Home", "Reset Camera"),
    ("Camera", "Alt+1", "Front View"),
    ("Camera", "Alt+3", "Right View"),
    ("Camera", "Alt+7", "Top View"),
    ("Camera", "Alt+0", "Perspective View"),
    ("Gizmo", "W", "Translate Mode"),
    ("Gizmo", "E", "Rotate Mode"),
    ("Gizmo", "R", "Scale Mode"),
    ("Play", "F5", "Play"),
    ("Play", "F6", "Pause"),
    ("Play", "F7", "Stop"),
    ("Play", "F8", "Step Frame"),
    ("View", "F1", "Show Help"),
    ("View", "G", "Toggle Grid"),
];

/// Render keyboard shortcuts as a grid
pub fn show_shortcuts_grid(ui: &mut egui::Ui) {
    let mut current_category = "";
    
    for (category, shortcut, description) in KEYBOARD_SHORTCUTS {
        if *category != current_category {
            if !current_category.is_empty() {
                ui.add_space(8.0);
            }
            ui.heading(*category);
            current_category = category;
        }
        
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(*shortcut).monospace().strong());
            ui.add_space(20.0);
            ui.label(*description);
        });
    }
}

/// Render shortcuts in a compact grid format for settings dialog
pub fn show_shortcuts_compact_grid(ui: &mut egui::Ui) {
    egui::Grid::new("shortcuts_grid_compact")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            for (_, shortcut, description) in KEYBOARD_SHORTCUTS.iter().take(16) {
                ui.label(*shortcut);
                ui.label(*description);
                ui.end_row();
            }
        });
}
