//! Polish panel for the editor UI
//!
//! Provides a GUI for configuring splash screens, loading screens,
//! and save/load settings.

use egui::{Color32, RichText, Ui};
use std::time::Duration;

use crate::panels::Panel;
use crate::polish::{LoadingScreen, LoadingStyle, SaveConfig, SplashScreen, SplashSequence};

/// Panel for configuring game polish features
pub struct PolishPanel {
    splash_sequence: SplashSequence,
    loading_screen: LoadingScreen,
    save_config: SaveConfig,
    active_tab: PolishTab,
    new_tip: String,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum PolishTab {
    #[default]
    Splash,
    Loading,
    SaveLoad,
}

impl Default for PolishPanel {
    fn default() -> Self {
        Self {
            splash_sequence: SplashSequence::new().with_engine_logo(),
            loading_screen: LoadingScreen::default(),
            save_config: SaveConfig::default(),
            active_tab: PolishTab::Splash,
            new_tip: String::new(),
        }
    }
}

impl PolishPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (PolishTab::Splash, "🌟 Splash Screens"),
                (PolishTab::Loading, "⏳ Loading Screens"),
                (PolishTab::SaveLoad, "💾 Save/Load"),
            ];

            for (tab, label) in tabs {
                let is_selected = self.active_tab == tab;
                let button = egui::Button::new(label).fill(if is_selected {
                    Color32::from_rgb(60, 100, 160)
                } else {
                    Color32::from_rgb(50, 50, 55)
                });

                if ui.add(button).clicked() {
                    self.active_tab = tab;
                }
            }
        });
        ui.separator();
    }

    fn show_splash_tab(&mut self, ui: &mut Ui) {
        ui.heading("Splash Screen Sequence");
        ui.add_space(5.0);

        // Sequence settings
        ui.checkbox(
            &mut self.splash_sequence.skip_all_on_input,
            "Skip all on any input",
        );

        ui.add_space(10.0);
        ui.label(format!(
            "Total duration: {:.1}s",
            self.splash_sequence.total_duration().as_secs_f32()
        ));
        ui.add_space(10.0);

        // List current screens
        ui.group(|ui| {
            ui.label(RichText::new("Screens").strong());

            for (i, screen) in self.splash_sequence.screens.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", i + 1));
                    ui.label(screen.image_path.display().to_string());
                    ui.label(format!(
                        "({:.1}s)",
                        screen.duration.unwrap_or(Duration::ZERO).as_secs_f32()
                    ));
                    if screen.skippable {
                        ui.label("⏭️ Skippable");
                    }
                });
            }

            if self.splash_sequence.screens.is_empty() {
                ui.label("No splash screens configured");
            }
        });

        ui.add_space(10.0);

        // Add screen buttons
        ui.horizontal(|ui| {
            if ui.button("➕ Add Engine Logo").clicked() {
                self.splash_sequence = std::mem::take(&mut self.splash_sequence).with_engine_logo();
            }
            if ui.button("➕ Add Publisher Logo").clicked() {
                self.splash_sequence = std::mem::take(&mut self.splash_sequence)
                    .with_publisher_logo("publisher_logo.png");
            }
        });
    }

    fn show_loading_tab(&mut self, ui: &mut Ui) {
        ui.heading("Loading Screen Configuration");
        ui.add_space(5.0);

        // Style selection
        ui.label("Style:");
        ui.horizontal(|ui| {
            let styles = [
                (LoadingStyle::ProgressBar, "Progress Bar"),
                (LoadingStyle::Spinner, "Spinner"),
                (LoadingStyle::FullScreen, "Full Screen"),
                (LoadingStyle::Dots, "Dots"),
                (LoadingStyle::ArtworkWithTips, "Artwork + Tips"),
            ];

            for (style, label) in styles {
                if ui
                    .selectable_label(self.loading_screen.style == style, label)
                    .clicked()
                {
                    self.loading_screen.style = style;
                }
            }
        });

        ui.add_space(10.0);

        // Options
        ui.checkbox(&mut self.loading_screen.show_percentage, "Show percentage");
        ui.checkbox(
            &mut self.loading_screen.show_task_description,
            "Show task description",
        );

        ui.add_space(10.0);

        // Tips section
        ui.collapsing("💡 Loading Tips", |ui| {
            for (i, tip) in self.loading_screen.tips.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: {}", i + 1, tip));
                });
            }

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_tip);
                if ui.button("Add Tip").clicked() && !self.new_tip.is_empty() {
                    self.loading_screen
                        .tips
                        .push(std::mem::take(&mut self.new_tip));
                }
            });
        });

        // Color pickers
        ui.add_space(10.0);
        ui.collapsing("🎨 Colors", |ui| {
            ui.horizontal(|ui| {
                ui.label("Background:");
                let mut bg = Color32::from_rgba_unmultiplied(
                    (self.loading_screen.background_color[0] * 255.0) as u8,
                    (self.loading_screen.background_color[1] * 255.0) as u8,
                    (self.loading_screen.background_color[2] * 255.0) as u8,
                    (self.loading_screen.background_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut bg).changed() {
                    self.loading_screen.background_color = [
                        bg.r() as f32 / 255.0,
                        bg.g() as f32 / 255.0,
                        bg.b() as f32 / 255.0,
                        bg.a() as f32 / 255.0,
                    ];
                }
            });

            ui.horizontal(|ui| {
                ui.label("Progress Bar:");
                let mut pc = Color32::from_rgba_unmultiplied(
                    (self.loading_screen.progress_color[0] * 255.0) as u8,
                    (self.loading_screen.progress_color[1] * 255.0) as u8,
                    (self.loading_screen.progress_color[2] * 255.0) as u8,
                    (self.loading_screen.progress_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut pc).changed() {
                    self.loading_screen.progress_color = [
                        pc.r() as f32 / 255.0,
                        pc.g() as f32 / 255.0,
                        pc.b() as f32 / 255.0,
                        pc.a() as f32 / 255.0,
                    ];
                }
            });
        });
    }

    fn show_save_load_tab(&mut self, ui: &mut Ui) {
        ui.heading("Save/Load Configuration");
        ui.add_space(5.0);

        egui::Grid::new("save_config_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Save Extension:");
                ui.text_edit_singleline(&mut self.save_config.extension);
                ui.end_row();

                ui.label("Save Directory:");
                ui.text_edit_singleline(&mut self.save_config.directory);
                ui.end_row();

                ui.label("Max Autosaves:");
                ui.add(egui::Slider::new(
                    &mut self.save_config.max_autosaves,
                    1..=10,
                ));
                ui.end_row();
            });

        ui.add_space(10.0);

        // Checkboxes
        ui.checkbox(&mut self.save_config.compress, "Compress save files (zstd)");
        ui.checkbox(
            &mut self.save_config.include_screenshot,
            "Include screenshot in saves",
        );

        // Autosave interval
        ui.add_space(10.0);
        let mut autosave_enabled = self.save_config.autosave_interval.is_some();
        if ui
            .checkbox(&mut autosave_enabled, "Enable autosave")
            .changed()
        {
            if autosave_enabled {
                self.save_config.autosave_interval = Some(Duration::from_secs(300));
            } else {
                self.save_config.autosave_interval = None;
            }
        }

        if let Some(interval) = &mut self.save_config.autosave_interval {
            ui.horizontal(|ui| {
                ui.label("Autosave interval:");
                let mut minutes = interval.as_secs() / 60;
                if ui
                    .add(egui::Slider::new(&mut minutes, 1..=30).suffix(" min"))
                    .changed()
                {
                    *interval = Duration::from_secs(minutes * 60);
                }
            });
        }

        // Preview
        ui.add_space(15.0);
        ui.separator();
        ui.label(RichText::new("📋 Configuration Preview").strong());
        ui.label(format!(
            "Save path: <game_data>/{}/slot.{}",
            self.save_config.directory, self.save_config.extension
        ));
    }
}

impl Panel for PolishPanel {
    fn name(&self) -> &str {
        "Polish"
    }

    fn show(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_tab_bar(ui);

            match self.active_tab {
                PolishTab::Splash => self.show_splash_tab(ui),
                PolishTab::Loading => self.show_loading_tab(ui),
                PolishTab::SaveLoad => self.show_save_load_tab(ui),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polish_panel_default() {
        let panel = PolishPanel::new();
        assert_eq!(panel.name(), "Polish");
        assert_eq!(panel.active_tab, PolishTab::Splash);
    }

    #[test]
    fn test_tab_switching() {
        let mut panel = PolishPanel::new();
        panel.active_tab = PolishTab::Loading;
        assert_eq!(panel.active_tab, PolishTab::Loading);

        panel.active_tab = PolishTab::SaveLoad;
        assert_eq!(panel.active_tab, PolishTab::SaveLoad);
    }

    #[test]
    fn test_initial_splash_sequence() {
        let panel = PolishPanel::new();
        // Default has engine logo
        assert_eq!(panel.splash_sequence.screens.len(), 1);
    }
}
