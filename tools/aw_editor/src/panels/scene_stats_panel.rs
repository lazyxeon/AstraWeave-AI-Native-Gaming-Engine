use super::Panel;
use egui::Ui;

pub struct SceneStatsPanel {
    last_update: std::time::Instant,
    cached_stats: SceneStats,
}

#[derive(Default, Clone)]
pub struct SceneStats {
    // Basic entity counts
    pub entity_count: usize,
    pub selected_count: usize,
    pub component_count: usize,
    pub prefab_count: usize,
    pub undo_stack_size: usize,
    pub redo_stack_size: usize,
    pub memory_estimate_kb: usize,
    pub scene_path: Option<String>,
    pub is_dirty: bool,
    
    // Week 5 Day 5: Enhanced mesh statistics
    pub mesh_count: usize,
    pub total_triangles: usize,
    pub total_vertices: usize,
    pub mesh_memory_kb: usize,
    
    // Texture statistics  
    pub texture_count: usize,
    pub texture_memory_kb: usize,
    pub max_texture_resolution: (u32, u32),
    
    // Material statistics
    pub material_count: usize,
    pub unique_shader_count: usize,
    
    // Performance estimates
    pub estimated_draw_calls: usize,
    pub estimated_state_changes: usize,
    pub performance_warning: Option<String>,
}

/// Performance threshold constants
const WARN_TRIANGLES: usize = 1_000_000;
const WARN_DRAW_CALLS: usize = 500;
const WARN_TEXTURE_MEMORY_MB: usize = 512;

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
    
    /// Generate performance warnings based on current stats
    pub fn generate_performance_warning(&self) -> Option<String> {
        let stats = &self.cached_stats;
        let mut warnings = Vec::new();
        
        if stats.total_triangles > WARN_TRIANGLES {
            warnings.push(format!(
                "High triangle count ({:.1}M) may impact GPU performance",
                stats.total_triangles as f64 / 1_000_000.0
            ));
        }
        
        if stats.estimated_draw_calls > WARN_DRAW_CALLS {
            warnings.push(format!(
                "High draw call count ({}) may impact CPU/driver performance",
                stats.estimated_draw_calls
            ));
        }
        
        if stats.texture_memory_kb > WARN_TEXTURE_MEMORY_MB * 1024 {
            warnings.push(format!(
                "High texture memory usage ({} MB) may cause stuttering",
                stats.texture_memory_kb / 1024
            ));
        }
        
        if warnings.is_empty() {
            None
        } else {
            Some(warnings.join("; "))
        }
    }
    
    /// Format bytes for display
    fn format_memory(bytes_kb: usize) -> String {
        if bytes_kb >= 1024 * 1024 {
            format!("{:.1} GB", bytes_kb as f64 / (1024.0 * 1024.0))
        } else if bytes_kb >= 1024 {
            format!("{:.1} MB", bytes_kb as f64 / 1024.0)
        } else {
            format!("{} KB", bytes_kb)
        }
    }
    
    /// Format large numbers with commas
    fn format_number(n: usize) -> String {
        if n >= 1_000_000 {
            format!("{:.2}M", n as f64 / 1_000_000.0)
        } else if n >= 1000 {
            format!("{:.1}K", n as f64 / 1000.0)
        } else {
            n.to_string()
        }
    }

    pub fn show_inline(&self, ui: &mut Ui) {
        let stats = &self.cached_stats;

        ui.horizontal(|ui| {
            ui.heading("📊 Scene Statistics");
            if stats.is_dirty {
                ui.label(
                    egui::RichText::new("(unsaved)")
                        .color(egui::Color32::from_rgb(255, 180, 100))
                        .small(),
                );
            }
        });
        ui.separator();
        
        // Performance warning banner
        if let Some(warning) = self.generate_performance_warning() {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚠️ Performance Warning")
                        .color(egui::Color32::from_rgb(255, 100, 100))
                        .strong(),
                );
            });
            ui.label(
                egui::RichText::new(&warning)
                    .color(egui::Color32::from_rgb(255, 180, 100))
                    .small(),
            );
            ui.separator();
        }

        // Entity section
        ui.collapsing("🎮 Entities", |ui| {
            egui::Grid::new("entity_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Total:");
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
                });
        });
        
        // Mesh statistics section
        ui.collapsing("🔷 Meshes", |ui| {
            egui::Grid::new("mesh_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Mesh Count:");
                    ui.label(format!("{}", stats.mesh_count));
                    ui.end_row();
                    
                    ui.label("Triangles:");
                    let tri_text = Self::format_number(stats.total_triangles);
                    let tri_color = if stats.total_triangles > WARN_TRIANGLES {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else {
                        ui.style().visuals.text_color()
                    };
                    ui.colored_label(tri_color, tri_text);
                    ui.end_row();
                    
                    ui.label("Vertices:");
                    ui.label(Self::format_number(stats.total_vertices));
                    ui.end_row();
                    
                    ui.label("Mesh Memory:");
                    ui.label(Self::format_memory(stats.mesh_memory_kb));
                    ui.end_row();
                });
        });
        
        // Texture statistics section
        ui.collapsing("🖼️ Textures", |ui| {
            egui::Grid::new("texture_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Texture Count:");
                    ui.label(format!("{}", stats.texture_count));
                    ui.end_row();
                    
                    ui.label("VRAM Usage:");
                    let vram_color = if stats.texture_memory_kb > WARN_TEXTURE_MEMORY_MB * 1024 {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else {
                        ui.style().visuals.text_color()
                    };
                    ui.colored_label(vram_color, Self::format_memory(stats.texture_memory_kb));
                    ui.end_row();
                    
                    ui.label("Max Resolution:");
                    ui.label(format!("{}×{}", stats.max_texture_resolution.0, stats.max_texture_resolution.1));
                    ui.end_row();
                });
        });
        
        // Material statistics section
        ui.collapsing("🎨 Materials", |ui| {
            egui::Grid::new("material_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Materials:");
                    ui.label(format!("{}", stats.material_count));
                    ui.end_row();
                    
                    ui.label("Unique Shaders:");
                    ui.label(format!("{}", stats.unique_shader_count));
                    ui.end_row();
                });
        });
        
        // Performance estimates section
        ui.collapsing("⚡ Performance Estimates", |ui| {
            egui::Grid::new("perf_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Draw Calls:");
                    let dc_color = if stats.estimated_draw_calls > WARN_DRAW_CALLS {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else if stats.estimated_draw_calls > WARN_DRAW_CALLS / 2 {
                        egui::Color32::from_rgb(255, 180, 100)
                    } else {
                        egui::Color32::from_rgb(100, 255, 100)
                    };
                    ui.colored_label(dc_color, format!("{}", stats.estimated_draw_calls));
                    ui.end_row();
                    
                    ui.label("State Changes:");
                    ui.label(format!("{}", stats.estimated_state_changes));
                    ui.end_row();
                    
                    // Performance grade
                    let grade = if stats.estimated_draw_calls < 100 && stats.total_triangles < 100_000 {
                        ("A+", egui::Color32::from_rgb(100, 255, 100))
                    } else if stats.estimated_draw_calls < 250 && stats.total_triangles < 500_000 {
                        ("A", egui::Color32::from_rgb(150, 255, 100))
                    } else if stats.estimated_draw_calls < WARN_DRAW_CALLS && stats.total_triangles < WARN_TRIANGLES {
                        ("B", egui::Color32::from_rgb(255, 255, 100))
                    } else {
                        ("C", egui::Color32::from_rgb(255, 100, 100))
                    };
                    ui.label("Perf Grade:");
                    ui.colored_label(grade.1, grade.0);
                    ui.end_row();
                });
        });
        
        // Undo/Redo section
        ui.collapsing("↩️ Undo History", |ui| {
            egui::Grid::new("undo_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Undo Stack:");
                    ui.label(format!("{}", stats.undo_stack_size));
                    ui.end_row();
                    
                    ui.label("Redo Stack:");
                    ui.label(format!("{}", stats.redo_stack_size));
                    ui.end_row();
                    
                    ui.label("Total Memory:");
                    ui.label(Self::format_memory(stats.memory_estimate_kb));
                    ui.end_row();
                });
        });

        if let Some(path) = &stats.scene_path {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("📁 Scene:");
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
        assert_eq!(panel.cached_stats.total_triangles, 0);
        assert_eq!(panel.cached_stats.texture_memory_kb, 0);
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
            mesh_count: 10,
            total_triangles: 50000,
            total_vertices: 25000,
            mesh_memory_kb: 512,
            texture_count: 5,
            texture_memory_kb: 2048,
            max_texture_resolution: (2048, 2048),
            material_count: 8,
            unique_shader_count: 3,
            estimated_draw_calls: 42,
            estimated_state_changes: 15,
            performance_warning: None,
        });
        assert_eq!(panel.cached_stats.entity_count, 42);
        assert!(panel.cached_stats.is_dirty);
        assert_eq!(panel.cached_stats.total_triangles, 50000);
    }
    
    #[test]
    fn test_performance_warning_triangles() {
        let mut panel = SceneStatsPanel::new();
        panel.update_stats(SceneStats {
            total_triangles: 2_000_000, // Over 1M threshold
            ..Default::default()
        });
        let warning = panel.generate_performance_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("triangle"));
    }
    
    #[test]
    fn test_performance_warning_draw_calls() {
        let mut panel = SceneStatsPanel::new();
        panel.update_stats(SceneStats {
            estimated_draw_calls: 600, // Over 500 threshold
            ..Default::default()
        });
        let warning = panel.generate_performance_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("draw call"));
    }
    
    #[test]
    fn test_performance_warning_texture_memory() {
        let mut panel = SceneStatsPanel::new();
        panel.update_stats(SceneStats {
            texture_memory_kb: 600 * 1024, // Over 512MB threshold
            ..Default::default()
        });
        let warning = panel.generate_performance_warning();
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("texture memory"));
    }
    
    #[test]
    fn test_no_performance_warning_under_thresholds() {
        let mut panel = SceneStatsPanel::new();
        panel.update_stats(SceneStats {
            total_triangles: 100_000,
            estimated_draw_calls: 100,
            texture_memory_kb: 256 * 1024,
            ..Default::default()
        });
        let warning = panel.generate_performance_warning();
        assert!(warning.is_none());
    }
    
    #[test]
    fn test_format_memory() {
        assert_eq!(SceneStatsPanel::format_memory(512), "512 KB");
        assert_eq!(SceneStatsPanel::format_memory(1024), "1.0 MB");
        assert_eq!(SceneStatsPanel::format_memory(2048), "2.0 MB");
        assert_eq!(SceneStatsPanel::format_memory(1024 * 1024), "1.0 GB");
    }
    
    #[test]
    fn test_format_number() {
        assert_eq!(SceneStatsPanel::format_number(500), "500");
        assert_eq!(SceneStatsPanel::format_number(1500), "1.5K");
        assert_eq!(SceneStatsPanel::format_number(1_500_000), "1.50M");
    }
}
