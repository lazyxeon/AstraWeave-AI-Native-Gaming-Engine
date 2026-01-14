use super::Panel;
use egui::Ui;

pub struct SceneStatsPanel {
    last_update: std::time::Instant,
    cached_stats: SceneStats,
}

#[derive(Default, Clone)]
pub struct SceneStats {
    pub entity_count: usize,
    pub selected_count: usize,
    pub component_count: usize,
    pub prefab_count: usize,
    pub undo_stack_size: usize,
    pub redo_stack_size: usize,
    pub memory_estimate_kb: usize,
    pub scene_path: Option<String>,
    pub is_dirty: bool,
}

impl SceneStatsPanel {
    pub fn new() -> Self {
        Self {
            last_update: std::time::Instant::now(),
            cached_stats: SceneStats::default(),
        }
    }

    pub fn update_stats(&mut self, stats: SceneStats) {
        self.cached_stats = stats;
        self.last_update = std::time::Instant::now();
    }

    pub fn show_inline(&self, ui: &mut Ui) {
        let stats = &self.cached_stats;

        ui.horizontal(|ui| {
            ui.heading("Scene Statistics");
            if stats.is_dirty {
                ui.label(
                    egui::RichText::new("(unsaved)")
                        .color(egui::Color32::from_rgb(255, 180, 100))
                        .small(),
                );
            }
        });
        ui.separator();

        egui::Grid::new("scene_stats_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("Entities:");
                ui.label(format!("{}", stats.entity_count));
                ui.end_row();

                ui.label("Selected:");
                ui.label(format!("{}", stats.selected_count));
                ui.end_row();

                ui.label("Components:");
                ui.label(format!("{}", stats.component_count));
                ui.end_row();

                ui.label("Prefab Instances:");
                ui.label(format!("{}", stats.prefab_count));
                ui.end_row();

                ui.label("Undo History:");
                ui.label(format!(
                    "{} / {}",
                    stats.undo_stack_size, stats.redo_stack_size
                ));
                ui.end_row();

                ui.label("Memory (est):");
                ui.label(format!("{} KB", stats.memory_estimate_kb));
                ui.end_row();
            });

        if let Some(path) = &stats.scene_path {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Scene:");
                ui.monospace(path);
            });
        }
    }
}

impl Default for SceneStatsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for SceneStatsPanel {
    fn name(&self) -> &str {
        "Scene Statistics"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_inline(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_stats_panel_creation() {
        let panel = SceneStatsPanel::new();
        assert_eq!(panel.cached_stats.entity_count, 0);
    }

    #[test]
    fn test_stats_update() {
        let mut panel = SceneStatsPanel::new();
        panel.update_stats(SceneStats {
            entity_count: 42,
            selected_count: 2,
            component_count: 100,
            prefab_count: 5,
            undo_stack_size: 10,
            redo_stack_size: 3,
            memory_estimate_kb: 1024,
            scene_path: Some("test.scene".into()),
            is_dirty: true,
        });
        assert_eq!(panel.cached_stats.entity_count, 42);
        assert!(panel.cached_stats.is_dirty);
    }
}
