//! Distribution panel for the editor UI
//!
//! Provides a GUI for creating platform-specific distributions
//! (Windows, macOS, Linux, Steam).

use egui::{Color32, RichText, Ui};

use crate::distribution::{
    DistributionBuilder, DistributionConfig, DistributionFormat, DistributionResult,
};
use crate::panels::Panel;

/// Panel for creating game distributions
pub struct DistributionPanel {
    config: DistributionConfig,
    selected_format: DistributionFormat,
    build_dir: String,
    output_dir: String,
    last_result: Option<Result<DistributionResult, String>>,
    is_building: bool,
}

impl Default for DistributionPanel {
    fn default() -> Self {
        Self {
            config: DistributionConfig::default(),
            selected_format: DistributionFormat::WindowsPortable,
            build_dir: "target/release".to_string(),
            output_dir: "dist".to_string(),
            last_result: None,
            is_building: false,
        }
    }
}

impl DistributionPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn show_config_section(&mut self, ui: &mut Ui) {
        ui.heading("📦 Game Configuration");
        ui.separator();

        egui::Grid::new("config_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Game Name:");
                ui.text_edit_singleline(&mut self.config.game_name);
                ui.end_row();

                ui.label("Version:");
                ui.text_edit_singleline(&mut self.config.version);
                ui.end_row();

                ui.label("Publisher:");
                ui.text_edit_singleline(&mut self.config.publisher);
                ui.end_row();

                ui.label("Description:");
                ui.text_edit_singleline(&mut self.config.description);
                ui.end_row();
            });

        ui.add_space(10.0);

        // Steam settings (collapsible)
        ui.collapsing("🎮 Steam Settings", |ui| {
            egui::Grid::new("steam_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("App ID:");
                    let mut app_id_str = self
                        .config
                        .steam_app_id
                        .map(|id| id.to_string())
                        .unwrap_or_default();
                    if ui.text_edit_singleline(&mut app_id_str).changed() {
                        self.config.steam_app_id = app_id_str.parse().ok();
                    }
                    ui.end_row();

                    ui.label("Depot ID:");
                    let mut depot_id_str = self
                        .config
                        .steam_depot_id
                        .map(|id| id.to_string())
                        .unwrap_or_default();
                    if ui.text_edit_singleline(&mut depot_id_str).changed() {
                        self.config.steam_depot_id = depot_id_str.parse().ok();
                    }
                    ui.end_row();
                });
        });
    }

    fn show_format_section(&mut self, ui: &mut Ui) {
        ui.add_space(15.0);
        ui.heading("🖥️ Distribution Format");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            let formats = [
                (
                    DistributionFormat::WindowsPortable,
                    "📁 Windows ZIP",
                    "Portable ZIP archive",
                ),
                (
                    DistributionFormat::WindowsInstaller,
                    "💿 Windows Installer",
                    "NSIS installer (.exe)",
                ),
                (
                    DistributionFormat::MacOSBundle,
                    "🍎 macOS Bundle",
                    "Application bundle (.app)",
                ),
                (
                    DistributionFormat::MacOSDmg,
                    "💿 macOS DMG",
                    "Disk image (.dmg)",
                ),
                (
                    DistributionFormat::LinuxTarball,
                    "🐧 Linux Tarball",
                    "Compressed archive",
                ),
                (
                    DistributionFormat::LinuxAppImage,
                    "📦 Linux AppImage",
                    "Portable executable",
                ),
                (
                    DistributionFormat::SteamDepot,
                    "🎮 Steam Depot",
                    "Steam content depot",
                ),
            ];

            for (format, label, description) in formats {
                let is_selected = self.selected_format == format;
                let button = egui::Button::new(label).fill(if is_selected {
                    Color32::from_rgb(60, 100, 160)
                } else {
                    Color32::from_rgb(50, 50, 55)
                });

                if ui.add(button).on_hover_text(description).clicked() {
                    self.selected_format = format;
                }
            }
        });
    }

    fn show_paths_section(&mut self, ui: &mut Ui) {
        ui.add_space(15.0);
        ui.heading("📂 Paths");
        ui.separator();

        egui::Grid::new("paths_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Build Directory:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.build_dir);
                    if ui.button("📁").clicked() {
                        // TODO: File browser
                    }
                });
                ui.end_row();

                ui.label("Output Directory:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.output_dir);
                    if ui.button("📁").clicked() {
                        // TODO: File browser
                    }
                });
                ui.end_row();
            });
    }

    fn show_build_section(&mut self, ui: &mut Ui) {
        ui.add_space(20.0);
        ui.separator();

        ui.horizontal(|ui| {
            let build_button = egui::Button::new(RichText::new("🚀 Build Distribution").size(16.0))
                .fill(Color32::from_rgb(40, 120, 80))
                .min_size(egui::vec2(200.0, 40.0));

            if ui.add_enabled(!self.is_building, build_button).clicked() {
                self.start_build();
            }

            if self.is_building {
                ui.spinner();
                ui.label("Building...");
            }
        });

        // Show last result
        if let Some(result) = &self.last_result {
            ui.add_space(10.0);
            match result {
                Ok(dist) => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✅ Success!").color(Color32::GREEN));
                        ui.label(format!(
                            "Created {} ({:.2} MB in {:.1}s)",
                            dist.output_path.display(),
                            dist.size_bytes as f64 / 1024.0 / 1024.0,
                            dist.duration_secs
                        ));
                    });
                }
                Err(e) => {
                    ui.label(RichText::new(format!("❌ Error: {}", e)).color(Color32::RED));
                }
            }
        }
    }

    fn start_build(&mut self) {
        let builder = DistributionBuilder::new(self.config.clone(), self.selected_format)
            .build_dir(&self.build_dir)
            .output_dir(&self.output_dir);

        match builder.build() {
            Ok(result) => {
                self.last_result = Some(Ok(result));
            }
            Err(e) => {
                self.last_result = Some(Err(e.to_string()));
            }
        }
    }
}

impl Panel for DistributionPanel {
    fn name(&self) -> &str {
        "Distribution"
    }

    fn show(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_config_section(ui);
            self.show_format_section(ui);
            self.show_paths_section(ui);
            self.show_build_section(ui);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distribution_panel_default() {
        let panel = DistributionPanel::new();
        assert_eq!(panel.name(), "Distribution");
        assert!(!panel.is_building);
        assert!(panel.last_result.is_none());
    }

    #[test]
    fn test_format_selection() {
        let mut panel = DistributionPanel::new();
        assert_eq!(panel.selected_format, DistributionFormat::WindowsPortable);

        panel.selected_format = DistributionFormat::SteamDepot;
        assert_eq!(panel.selected_format, DistributionFormat::SteamDepot);
    }
}
