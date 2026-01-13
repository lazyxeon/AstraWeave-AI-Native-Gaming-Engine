use super::Panel;
use egui::Ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    All,
}

impl LogLevel {
    pub fn matches(&self, log: &str) -> bool {
        match self {
            LogLevel::All => true,
            LogLevel::Info => !log.contains("⚠️") && !log.contains("❌"),
            LogLevel::Warning => log.contains("⚠️"),
            LogLevel::Error => log.contains("❌"),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LogLevel::All => "All",
            LogLevel::Info => "Info",
            LogLevel::Warning => "Warnings",
            LogLevel::Error => "Errors",
        }
    }
}

pub struct ConsolePanel {
    filter_level: LogLevel,
    search_text: String,
    auto_scroll: bool,
    show_timestamps: bool,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            filter_level: LogLevel::All,
            search_text: String::new(),
            auto_scroll: true,
            show_timestamps: true,
        }
    }

    pub fn show_with_logs(&mut self, ui: &mut Ui, logs: &mut Vec<String>) -> bool {
        let mut cleared = false;
        
        let total_count = logs.len();
        let warning_count = logs.iter().filter(|l| l.contains("⚠️")).count();
        let error_count = logs.iter().filter(|l| l.contains("❌")).count();
        let info_count = total_count - warning_count - error_count;

        ui.horizontal(|ui| {
            ui.heading("Console");
            ui.separator();
            ui.label(format!("{} logs", total_count));
            
            if info_count > 0 {
                ui.label(
                    egui::RichText::new(format!("{} info", info_count))
                        .color(egui::Color32::from_rgb(100, 149, 237))
                );
            }
            if warning_count > 0 {
                ui.label(
                    egui::RichText::new(format!("{} warn", warning_count))
                        .color(egui::Color32::YELLOW)
                );
            }
            if error_count > 0 {
                ui.label(
                    egui::RichText::new(format!("{} err", error_count))
                        .color(egui::Color32::RED)
                );
            }
        });
        
        ui.separator();

        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("log_level_filter")
                .selected_text(self.filter_level.name())
                .width(90.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_level, LogLevel::All, "All");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Info, "Info");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Warning, "Warnings");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Error, "Errors");
                });

            ui.add(
                egui::TextEdit::singleline(&mut self.search_text)
                    .hint_text("Search logs...")
                    .desired_width(150.0)
            );

            if ui.small_button("X").on_hover_text("Clear search").clicked() {
                self.search_text.clear();
            }

            ui.separator();

            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");

            if ui.button("Clear").on_hover_text("Clear all logs").clicked() {
                logs.clear();
                cleared = true;
            }

            if ui.button("Copy All").on_hover_text("Copy logs to clipboard").clicked() {
                let all_logs = logs.join("\n");
                ui.ctx().copy_text(all_logs);
            }
        });

        ui.add_space(4.0);

        let filtered_logs: Vec<&String> = logs
            .iter()
            .filter(|log| {
                self.filter_level.matches(log)
                    && (self.search_text.is_empty()
                        || log.to_lowercase().contains(&self.search_text.to_lowercase()))
            })
            .collect();

        let scroll_area = egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, false]);

        let scroll = if self.auto_scroll {
            scroll_area.stick_to_bottom(true)
        } else {
            scroll_area
        };

        scroll.show(ui, |ui| {
            if filtered_logs.is_empty() {
                ui.colored_label(
                    egui::Color32::GRAY,
                    if logs.is_empty() {
                        "No logs yet."
                    } else {
                        "No logs match the current filter."
                    },
                );
            } else {
                for log in filtered_logs {
                    let color = if log.contains("❌") {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else if log.contains("⚠️") {
                        egui::Color32::from_rgb(255, 200, 100)
                    } else if log.contains("✅") {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else {
                        egui::Color32::LIGHT_GRAY
                    };

                    ui.colored_label(color, log);
                }
            }
        });

        cleared
    }
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for ConsolePanel {
    fn name(&self) -> &str {
        "Console"
    }

    fn show(&mut self, _ui: &mut Ui) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_filter() {
        assert!(LogLevel::All.matches("Any log"));
        assert!(LogLevel::All.matches("⚠️ Warning"));
        assert!(LogLevel::All.matches("❌ Error"));

        assert!(LogLevel::Warning.matches("⚠️ Warning message"));
        assert!(!LogLevel::Warning.matches("Normal message"));
        assert!(!LogLevel::Warning.matches("❌ Error message"));

        assert!(LogLevel::Error.matches("❌ Error message"));
        assert!(!LogLevel::Error.matches("Normal message"));
        assert!(!LogLevel::Error.matches("⚠️ Warning message"));

        assert!(LogLevel::Info.matches("Normal info message"));
        assert!(!LogLevel::Info.matches("⚠️ Warning"));
        assert!(!LogLevel::Info.matches("❌ Error"));
    }

    #[test]
    fn test_console_panel_creation() {
        let panel = ConsolePanel::new();
        assert_eq!(panel.filter_level, LogLevel::All);
        assert!(panel.search_text.is_empty());
        assert!(panel.auto_scroll);
    }
}
