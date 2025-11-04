// tools/aw_editor/src/panels/world_panel.rs - World state viewer using Astract

use super::Panel;
use astract::prelude::*;
use egui::Ui;

/// World panel - displays and edits world state
///
/// Demonstrates Astract hooks:
/// - use_state for panel-local state
/// - RSX macros for declarative UI (when available)
/// - Component composition
pub struct WorldPanel {
    // Panel doesn't need state - hooks manage it
}

impl WorldPanel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WorldPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for WorldPanel {
    fn name(&self) -> &str {
        "World"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.heading("🌍 World State");
        ui.separator();

        // Use Astract hooks for state management
        let (biome, set_biome) = use_state(ui, "world_biome", "Forest".to_string());
        let (seed, set_seed) = use_state(ui, "world_seed", 12345u64);
        let (time_of_day, set_time_of_day) = use_state(ui, "world_time", 12.0f32);

        ui.group(|ui| {
            ui.label("🌲 Biome");
            ui.horizontal(|ui| {
                if ui.button("Forest").clicked() {
                    set_biome.set(ui, "Forest".to_string());
                }
                if ui.button("Desert").clicked() {
                    set_biome.set(ui, "Desert".to_string());
                }
                if ui.button("Tundra").clicked() {
                    set_biome.set(ui, "Tundra".to_string());
                }
            });
            ui.label(format!("Current: {}", biome));
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("🎲 Seed");
            let mut seed_val = seed;
            if ui
                .add(egui::Slider::new(&mut seed_val, 0..=99999).text("seed"))
                .changed()
            {
                set_seed.set(ui, seed_val);
            }
            if ui.button("🔀 Randomize").clicked() {
                set_seed.set(ui, rand::random::<u64>() % 100000);
            }
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("🌞 Time of Day");
            let mut time_val = time_of_day;
            if ui
                .add(
                    egui::Slider::new(&mut time_val, 0.0..=24.0)
                        .text("hours")
                        .suffix(" h"),
                )
                .changed()
            {
                set_time_of_day.set(ui, time_val);
            }

            ui.horizontal(|ui| {
                if ui.button("🌅 Dawn (6h)").clicked() {
                    set_time_of_day.set(ui, 6.0);
                }
                if ui.button("☀️ Noon (12h)").clicked() {
                    set_time_of_day.set(ui, 12.0);
                }
                if ui.button("🌆 Dusk (18h)").clicked() {
                    set_time_of_day.set(ui, 18.0);
                }
                if ui.button("🌙 Midnight (0h)").clicked() {
                    set_time_of_day.set(ui, 0.0);
                }
            });
        });

        ui.add_space(10.0);

        // Memoized derived state (sky color based on time)
        let sky_color = use_memo(ui, "sky_color", time_of_day, |&time| {
            if time >= 6.0 && time < 12.0 {
                "🌅 Orange (Dawn)"
            } else if time >= 12.0 && time < 18.0 {
                "☀️ Blue (Day)"
            } else if time >= 18.0 && time < 21.0 {
                "🌆 Purple (Dusk)"
            } else {
                "🌙 Dark Blue (Night)"
            }
        });

        ui.group(|ui| {
            ui.label("Sky");
            ui.label(format!("Color: {}", sky_color));
        });

        // Effect example: log when seed changes
        use_effect(ui, "seed_change_log", seed, |&s| {
            println!("🌍 World seed changed to: {}", s);
        });
    }
}
