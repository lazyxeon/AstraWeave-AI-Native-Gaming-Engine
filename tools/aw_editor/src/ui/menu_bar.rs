use egui::Ui;
use std::path::PathBuf;
use crate::panel_type::PanelType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignDirection {
    Left,
    Right,
    Top,
    Bottom,
    CenterX,
    CenterZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributeDirection {
    X,
    Z,
}

/// Handler for menu bar actions
pub trait MenuActionHandler {
    fn on_new(&mut self);
    fn on_open(&mut self);
    fn on_save(&mut self);
    fn on_save_json(&mut self);
    fn on_save_scene(&mut self);
    fn on_load_scene(&mut self);

    fn on_undo(&mut self);
    fn on_redo(&mut self);
    fn on_delete(&mut self);

    fn selection_count(&self) -> usize;
    
    fn on_apply_material(&mut self);
    fn on_group_selection(&mut self);
    fn on_ungroup_selection(&mut self);
    fn on_align_selection(&mut self, dir: AlignDirection);
    fn on_distribute_selection(&mut self, dir: DistributeDirection);
    fn on_select_all(&mut self);
    fn on_deselect_all(&mut self);

    // Recent Files
    fn get_recent_files(&self) -> Vec<PathBuf>;
    fn on_open_recent(&mut self, path: PathBuf);
    fn on_clear_recent(&mut self);

    // View
    fn is_view_hierarchy_open(&self) -> bool;
    fn toggle_view_hierarchy(&mut self);
    fn is_view_inspector_open(&self) -> bool;
    fn toggle_view_inspector(&mut self);
    fn is_view_console_open(&self) -> bool;
    fn toggle_view_console(&mut self);
    fn is_grid_visible(&self) -> bool;
    fn toggle_grid(&mut self);

    // Window
    fn is_docking_enabled(&self) -> bool;
    fn toggle_docking(&mut self);
    fn on_apply_layout_preset(&mut self, preset_name: &str); // "Default", "Wide", "Compact", "Modeling", "Animation", "Debug"
    fn is_dock_panel_visible(&self, panel: PanelType) -> bool;
    fn toggle_dock_panel(&mut self, panel: PanelType);

    // Settings
    fn on_open_settings(&mut self);

    // Debug
    fn on_scan_for_models(&mut self);
    fn on_load_test_model(&mut self, name: &str, path: PathBuf);
    fn on_toggle_engine_rendering(&mut self);
    fn on_show_engine_info(&mut self);
    
    fn on_debug_material(&mut self, name: &str); // "Red", "Green", "Blue", "White"
    fn on_debug_time_set(&mut self, time: f32);
    fn get_time_of_day(&self) -> f32; // return -1.0 or similar if unavailable? Or Option<f32>
    fn get_time_period(&self) -> String;
    
    fn is_shadows_enabled(&self) -> bool;
    fn set_shadows_enabled(&mut self, enabled: bool);
    
    fn on_diff_assets(&mut self);
    fn on_clear_console(&mut self);
}

/// Main Menu Bar Component
pub struct MenuBar;

impl MenuBar {
    pub fn show(ui: &mut Ui, handler: &mut dyn MenuActionHandler) {
        if ui.button("New").clicked() {
            handler.on_new();
        }
            if ui.button("Open").clicked() {
                handler.on_open();
            }
            if ui.button("Save").clicked() {
                handler.on_save();
            }
            if ui.button("Save JSON").clicked() {
                handler.on_save_json();
            }

            ui.separator();

            if ui.button("💾 Save Scene").clicked() {
                handler.on_save_scene();
            }
             if ui.button("📂 Load Scene").clicked() {
                handler.on_load_scene();
            }

            ui.separator();

            ui.menu_button("✏️ Edit", |ui| {
                if ui.button("↩️ Undo (Ctrl+Z)").clicked() {
                    handler.on_undo();
                    ui.close();
                }
                if ui.button("↪️ Redo (Ctrl+Y)").clicked() {
                    handler.on_redo();
                    ui.close();
                }

                if ui.button("🗑️ Delete (Del)").clicked() {
                    handler.on_delete();
                    ui.close();
                }

                ui.separator();

                let count = handler.selection_count();
                ui.label(format!("📦 {} selected", count));
                ui.separator();
                
                let has_multi = count > 1;

                if ui.add_enabled(has_multi, egui::Button::new("🎨 Apply Material to All")).clicked() {
                     handler.on_apply_material();
                     ui.close();
                }

                ui.separator();
                
                if ui.add_enabled(has_multi, egui::Button::new("📁 Group Selection (Ctrl+G)")).clicked() {
                    handler.on_group_selection();
                    ui.close();
                }
                
                if ui.button("📂 Ungroup (Ctrl+Shift+G)").clicked() {
                    handler.on_ungroup_selection();
                    ui.close();
                }
                
                ui.separator();
                ui.label("📐 Align Selection:");
                ui.horizontal(|ui| {
                     if ui.add_enabled(has_multi, egui::Button::new("⬅")).clicked() {
                         handler.on_align_selection(AlignDirection::Left);
                     }
                     if ui.add_enabled(has_multi, egui::Button::new("➡")).clicked() {
                         handler.on_align_selection(AlignDirection::Right);
                     }
                });
                
                ui.separator();
                
                let can_distribute = count >= 3;
                ui.label("📏 Distribute:");
                ui.horizontal(|ui| {
                    if ui.add_enabled(can_distribute, egui::Button::new("↔ X")).on_hover_text("Distribute evenly along X").clicked() {
                        handler.on_distribute_selection(DistributeDirection::X);
                    }
                    if ui.add_enabled(can_distribute, egui::Button::new("↕ Z")).on_hover_text("Distribute evenly along Z").clicked() {
                        handler.on_distribute_selection(DistributeDirection::Z);
                    }
                });
                
                ui.separator();
                
                if ui.button("📦 Select All (Ctrl+A)").clicked() {
                    handler.on_select_all();
                    ui.close();
                }
                if ui.button("🚫 Deselect All (Esc)").clicked() {
                    handler.on_deselect_all();
                    ui.close();
                }

            });

            // Recent Files
            ui.menu_button("📚 Recent Files", |ui| {
                let files = handler.get_recent_files();
                if files.is_empty() {
                    ui.label("No recent files");
                } else {
                    for path in files {
                         let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                         if ui.button(name).clicked() {
                             handler.on_open_recent(path);
                             ui.close();
                         }
                    }
                    ui.separator();
                    if ui.button("🗑️ Clear Recent Files").clicked() {
                        handler.on_clear_recent();
                        ui.close();
                    }
                }
            });

            // View
            ui.menu_button("👁 View", |ui| {
                let mut h = handler.is_view_hierarchy_open();
                if ui.checkbox(&mut h, "Hierarchy Panel").changed() {
                    handler.toggle_view_hierarchy();
                }
                let mut i = handler.is_view_inspector_open();
                if ui.checkbox(&mut i, "Inspector Panel").changed() {
                    handler.toggle_view_inspector();
                }
                let mut c = handler.is_view_console_open();
                if ui.checkbox(&mut c, "Console Panel").changed() {
                    handler.toggle_view_console();
                }
                ui.separator();
                let mut g = handler.is_grid_visible();
                if ui.checkbox(&mut g, "Grid").changed() {
                    handler.toggle_grid();
                }
            });

            // Window
            ui.menu_button("🪟 Window", |ui| {
                let mut docking = handler.is_docking_enabled();
                if ui.checkbox(&mut docking, "📐 Use Docking Layout").changed() {
                    handler.toggle_docking();
                }

                ui.separator();
                ui.label("Layout Presets:");

                if ui.button("📊 Default").clicked() { handler.on_apply_layout_preset("Default"); ui.close(); }
                if ui.button("📐 Wide").clicked() { handler.on_apply_layout_preset("Wide"); ui.close(); }
                if ui.button("📦 Compact").clicked() { handler.on_apply_layout_preset("Compact"); ui.close(); }
                if ui.button("🎨 Modeling").clicked() { handler.on_apply_layout_preset("Modeling"); ui.close(); }
                if ui.button("🎬 Animation").clicked() { handler.on_apply_layout_preset("Animation"); ui.close(); }
                if ui.button("🐛 Debug").clicked() { handler.on_apply_layout_preset("Debug"); ui.close(); }

                ui.separator();
                ui.label("Panels:");

                for &panel_type in PanelType::all() {
                    let is_visible = handler.is_dock_panel_visible(panel_type);
                    let label = format!("{} {}", panel_type.icon(), panel_type.title());
                    if ui.selectable_label(is_visible, label).clicked() {
                        handler.toggle_dock_panel(panel_type);
                    }
                }
            });

            // Settings
            if ui.button("⚙ Settings").clicked() {
                handler.on_open_settings();
            }

            // Debug
            ui.menu_button("🐛 Debug", |ui| {
                ui.label("🎨 Viewport Tests:");
                
                #[cfg(feature = "astraweave-render")]
                {
                     if ui.button("📦 Load Test Model (barrels.glb)").clicked() {
                         handler.on_load_test_model("test_barrels", PathBuf::from("assets/models/barrels.glb"));
                         ui.close();
                     }
                     if ui.button("🛏️ Load Test Model (bed.glb)").clicked() {
                         handler.on_load_test_model("test_bed", PathBuf::from("assets/models/bed.glb"));
                         ui.close();
                     }
                     if ui.button("🌲 Load Pine Tree").clicked() {
                         handler.on_load_test_model("pine_tree", PathBuf::from("PINE_TREE_AUTO")); 
                         ui.close();
                     }
                     
                     if ui.button("🔄 Toggle Engine Rendering").clicked() {
                         handler.on_toggle_engine_rendering();
                         ui.close();
                     }
                }
                
                ui.separator();
                ui.label("📊 Diagnostics:");
                if ui.button("📋 Show Engine Info").clicked() {
                    handler.on_show_engine_info();
                    ui.close();
                }
                
                ui.separator();
                ui.label("🎨 Material Testing:");
                if ui.button("🔴 Red Material").clicked() { handler.on_debug_material("Red"); ui.close(); }
                if ui.button("🟢 Green Metallic").clicked() { handler.on_debug_material("Green"); ui.close(); }
                if ui.button("🔵 Blue Rough").clicked() { handler.on_debug_material("Blue"); ui.close(); }
                if ui.button("⬜ White Default").clicked() { handler.on_debug_material("White"); ui.close(); }

                ui.separator();
                ui.label("☀️ Lighting / Time of Day:");
                ui.horizontal(|ui| {
                    if ui.button("🌅 Dawn (6:00)").clicked() { handler.on_debug_time_set(6.0); }
                    if ui.button("☀️ Noon (12:00)").clicked() { handler.on_debug_time_set(12.0); }
                });
                ui.horizontal(|ui| {
                    if ui.button("🌇 Sunset (18:00)").clicked() { handler.on_debug_time_set(18.0); }
                    if ui.button("🌙 Midnight (0:00)").clicked() { handler.on_debug_time_set(0.0); }
                });
                
                let time = handler.get_time_of_day();
                let hours = time.floor() as u32;
                let minutes = ((time - time.floor()) * 60.0) as u32;
                ui.label(format!("🕐 Current: {:02}:{:02} ({})", hours, minutes, handler.get_time_period()));

                ui.horizontal(|ui| {
                     let shadows = handler.is_shadows_enabled();
                     let label = if shadows { "🔦 Shadows: ON" } else { "🔦 Shadows: OFF" };
                     if ui.button(label).clicked() {
                         handler.set_shadows_enabled(!shadows);
                     }
                });

                ui.separator();
                ui.label("📁 Model Discovery:");
                
                if ui.button("📁 Scan For Models").clicked() {
                    handler.on_scan_for_models();
                    ui.close();
                }
                
                if ui.button("Diff Assets").clicked() {
                    handler.on_diff_assets();
                    ui.close();
                }
                
                if ui.button("🗑️ Clear Console").clicked() {
                    handler.on_clear_console();
                    ui.close();
                }
            });
    }
}
