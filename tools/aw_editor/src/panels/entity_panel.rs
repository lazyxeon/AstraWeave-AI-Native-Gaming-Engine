// tools/aw_editor/src/panels/entity_panel.rs - Entity inspector using Astract

use super::Panel;
use crate::component_ui::{ComponentEdit, ComponentRegistry};
use crate::scene_state::EditorSceneState;
use astract::prelude::*;
use astraweave_core::{Ammo, Entity, Health, IVec2, Team, World};
use egui::Ui;

/// Entity panel - inspect and edit entity properties
///
/// Now integrated with real ECS World instead of mock entities.
pub struct EntityPanel {
    component_registry: ComponentRegistry,
}

impl EntityPanel {
    pub fn new() -> Self {
        Self {
            component_registry: ComponentRegistry::new(),
        }
    }

    /// Show entity panel with real world integration
    ///
    /// # Arguments
    ///
    /// * `ui` - egui UI context
    /// * `world` - Mutable reference to ECS world (optional)
    /// * `selected_entity` - Currently selected entity (optional)
    ///
    /// # Returns
    ///
    /// Optional ComponentEdit if a component was modified (for undo system)
    pub fn show_with_world(
        &mut self,
        ui: &mut Ui,
        scene_state: &mut Option<EditorSceneState>,
        selected_entity: Option<Entity>,
        prefab_instance: Option<&crate::prefab::PrefabInstance>,
    ) -> Option<ComponentEdit> {
        ui.heading("🎮 Entity Inspector");
        ui.separator();

        // Entity management buttons
        ui.horizontal(|ui| {
            if ui.button("➕ Spawn Companion").clicked() {
                if let Some(state) = scene_state {
                    let world = state.world_mut();
                    let entity_count = world.entities().len();
                    let pos = IVec2 {
                        x: rand::random::<i32>() % 30,
                        y: rand::random::<i32>() % 30,
                    };
                    let entity = world.spawn(
                        &format!("Companion_{}", entity_count),
                        pos,
                        Team { id: 0 }, // Team 0 = companion
                        100,            // HP
                        30,             // Ammo
                    );
                    println!("✅ Spawned companion #{} at ({}, {})", entity, pos.x, pos.y);
                }
            }

            if ui.button("➕ Spawn Enemy").clicked() {
                if let Some(state) = scene_state {
                    let world = state.world_mut();
                    let entity_count = world.entities().len();
                    let pos = IVec2 {
                        x: rand::random::<i32>() % 30,
                        y: rand::random::<i32>() % 30,
                    };
                    let entity = world.spawn(
                        &format!("Enemy_{}", entity_count),
                        pos,
                        Team { id: 1 }, // Team 1 = enemy
                        80,             // HP
                        20,             // Ammo
                    );
                    println!("✅ Spawned enemy #{} at ({}, {})", entity, pos.x, pos.y);
                }
            }

            if ui.button("🗑️ Clear All").clicked() {
                if let Some(state) = scene_state {
                    *state = EditorSceneState::new(World::new());
                    println!("🗑️ Cleared all entities");
                }
            }
        });

        ui.add_space(10.0);

        let mut component_edit = None;

        if let Some(entity) = selected_entity {
            if let Some(state) = scene_state {
                let world = state.world_mut();
                ui.group(|ui| {
                    ui.heading(format!("✏️ Entity #{}", entity));

                    if let Some(instance) = prefab_instance {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("💾 Prefab Instance:");
                            ui.monospace(
                                instance
                                    .source
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .as_ref(),
                            );
                        });

                        if instance.has_overrides(entity) {
                            ui.colored_label(
                                egui::Color32::from_rgb(100, 150, 255),
                                "⚠️ Modified components (blue text indicates overrides)",
                            );
                        }
                    }

                    ui.separator();

                    let components = self.component_registry.get_entity_components(world, entity);

                    if components.is_empty() {
                        ui.label("No components");
                    } else {
                        for component_type in components {
                            if let Some(edit) = component_type.show_ui(world, entity, ui) {
                                component_edit = Some(edit);
                            }
                        }
                    }
                });
            } else {
                ui.label("⚠️ No world initialized");
            }
        } else {
            ui.label("No entity selected");
            ui.label("Click an entity in the viewport to inspect it");
        }

        ui.separator();
        ui.add_space(10.0);

        // Display entity count
        if let Some(state) = scene_state {
            let world = state.world();
            let entity_count = world.entities().len();
            ui.label(format!("📊 Total Entities: {}", entity_count));

            ui.separator();

            // List all entities
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for entity in world.entities() {
                        if let Some(pose) = world.pose(entity) {
                            let team = world.team(entity).unwrap_or(Team { id: 0 });
                            let health = world.health(entity).unwrap_or(Health { hp: 0 });
                            let ammo = world.ammo(entity).unwrap_or(Ammo { rounds: 0 });

                            let team_name = if team.id == 0 { "Companion" } else { "Enemy" };
                            let team_color = if team.id == 0 {
                                egui::Color32::from_rgb(100, 150, 255)
                            } else {
                                egui::Color32::from_rgb(255, 100, 100)
                            };

                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("Entity #{}", entity)).strong(),
                                    );
                                    ui.label(egui::RichText::new(team_name).color(team_color));
                                });

                                ui.label(format!("📍 Position: ({}, {})", pose.pos.x, pose.pos.y));
                                ui.label(format!("❤️  Health: {}", health.hp));
                                ui.label(format!("🔫 Ammo: {}", ammo.rounds));
                            });
                        }
                    }
                });
        } else {
            ui.label("⚠️  No world initialized");
        }

        component_edit
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
