// tools/aw_editor/src/panels/entity_panel.rs - Entity inspector using Astract

use super::Panel;
use astract::prelude::*;
use egui::Ui;

/// Entity panel - inspect and edit entity properties
///
/// Demonstrates Astract hooks:
/// - use_state for entity selection
/// - use_memo for filtered lists
/// - Component composition patterns
pub struct EntityPanel {
    // Panel state managed by hooks
}

impl EntityPanel {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EntityPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for EntityPanel {
    fn name(&self) -> &str {
        "Entities"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.heading("🎮 Entity Inspector");
        ui.separator();

        // Entity selection state
        let (selected_entity, set_selected_entity) = use_state(ui, "selected_entity", 0usize);
        let (filter, set_filter) = use_state(ui, "entity_filter", String::new());

        // Mock entity list (in real app, this would come from ECS world)
        let entities = vec![
            ("Player", "Companion", 100, 10),
            ("Enemy_1", "Grunt", 50, 5),
            ("Enemy_2", "Elite", 150, 15),
            ("NPC_Merchant", "Civilian", 80, 0),
            ("Boss_Dragon", "Boss", 500, 50),
        ];

        // Memoized filtered list
        let filtered_entities = use_memo(ui, "filtered_entities", filter.clone(), |f| {
            if f.is_empty() {
                (0..entities.len()).collect::<Vec<_>>()
            } else {
                entities
                    .iter()
                    .enumerate()
                    .filter(|(_, (name, _, _, _))| name.to_lowercase().contains(&f.to_lowercase()))
                    .map(|(i, _)| i)
                    .collect()
            }
        });

        // Search/filter UI
        ui.group(|ui| {
            ui.label("🔍 Search");
            let mut filter_val = filter.clone();
            if ui.text_edit_singleline(&mut filter_val).changed() {
                set_filter.set(ui, filter_val);
            }
            ui.label(format!("Found: {} entities", filtered_entities.len()));
        });

        ui.add_space(10.0);

        // Entity list
        ui.group(|ui| {
            ui.label("📜 Entity List");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for &idx in &filtered_entities {
                        let (name, archetype, _, _) = entities[idx];
                        let is_selected = idx == selected_entity;

                        if ui
                            .selectable_label(is_selected, format!("{} ({})", name, archetype))
                            .clicked()
                        {
                            set_selected_entity.set(ui, idx);
                        }
                    }
                });
        });

        ui.add_space(10.0);

        // Entity details
        if selected_entity < entities.len() {
            let (name, archetype, health, damage) = entities[selected_entity];

            ui.group(|ui| {
                ui.heading(format!("📋 {}", name));
                ui.separator();

                ui.label(format!("Type: {}", archetype));

                ui.add_space(5.0);

                // Health display
                let health_pct = (health as f32 / 500.0).min(1.0);
                let health_color = if health_pct > 0.6 {
                    egui::Color32::GREEN
                } else if health_pct > 0.3 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };

                ui.horizontal(|ui| {
                    ui.label("❤️ Health:");
                    ui.label(egui::RichText::new(format!("{}/500", health)).color(health_color));
                });

                let health_bar_width = ui.available_width();
                let (rect, _response) = ui
                    .allocate_exact_size(egui::vec2(health_bar_width, 10.0), egui::Sense::hover());

                ui.painter()
                    .rect_filled(rect, 2.0, egui::Color32::DARK_GRAY);

                let filled_width = rect.width() * health_pct;
                let filled_rect =
                    egui::Rect::from_min_size(rect.min, egui::vec2(filled_width, rect.height()));
                ui.painter().rect_filled(filled_rect, 2.0, health_color);

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("⚔️ Damage:");
                    ui.label(format!("{}", damage));
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("🔫 Damage").clicked() {
                        println!("💥 Damaged {}", name);
                    }
                    if ui.button("❤️‍🩹 Heal").clicked() {
                        println!("💚 Healed {}", name);
                    }
                    if ui.button("🗑️ Delete").clicked() {
                        println!("🗑️ Deleted {}", name);
                    }
                });
            });
        } else {
            ui.label("Select an entity to view details");
        }

        // Effect: log when selection changes
        use_effect(ui, "entity_selection_log", selected_entity, |&idx| {
            if idx < entities.len() {
                println!("🎯 Selected entity: {}", entities[idx].0);
            }
        });
    }
}
