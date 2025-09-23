use egui::{self, Color32, RichText};
use glam::Vec3;

use astraweave_gameplay::quests::QuestLog;
use astraweave_gameplay::stats::Stats;
use astraweave_gameplay::{Inventory, RecipeBook};

use crate::{Accessibility, UiFlags};
use astraweave_cinematics as awc;

#[derive(Default)]
pub struct UiResult {
    pub crafted: Option<String>, // name
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ui(
    ctx: &egui::Context,
    flags: &mut UiFlags,
    acc: &mut Accessibility,
    player_stats: &Stats,
    player_pos: Vec3,
    inventory: &mut Inventory,
    recipes: Option<&RecipeBook>,
    quests: Option<&mut QuestLog>,
) -> UiResult {
    let mut out = UiResult::default();

    // Top bar – Menu toggles
    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Menu").clicked() {
                flags.show_menu = !flags.show_menu;
            }
            if ui.button("Inventory (I)").clicked() {
                flags.show_inventory = !flags.show_inventory;
            }
            if ui.button("Crafting (C)").clicked() {
                flags.show_crafting = !flags.show_crafting;
            }
            if ui.button("Map (M)").clicked() {
                flags.show_map = !flags.show_map;
            }
            if ui.button("Quests (J)").clicked() {
                flags.show_quests = !flags.show_quests;
            }
            if ui.button("Settings").clicked() {
                flags.show_settings = !flags.show_settings;
            }
            ui.separator();
            ui.label(
                RichText::new(format!("HP: {}", player_stats.hp))
                    .color(Color32::from_rgb(220, 80, 80)),
            );
            ui.label(
                RichText::new(format!("STA: {}", player_stats.stamina))
                    .color(Color32::from_rgb(80, 180, 220)),
            );
            ui.label(format!(
                "Pos: {:.1}, {:.1}, {:.1}",
                player_pos.x, player_pos.y, player_pos.z
            ));
        });
    });

    // HUD (bottom-left small)
    egui::TopBottomPanel::bottom("hud").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Ability1 [Q]  Ability2 [E]");
            if acc.high_contrast_ui {
                ui.colored_label(Color32::YELLOW, "High Contrast");
            }
            if let Some(mode) = &acc.colorblind_mode {
                ui.label(format!("CB: {}", mode));
            }
        });
    });

    if flags.show_menu {
        egui::Window::new("Main Menu")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Pause / Save / Exit – (placeholder)");
                if ui.button("Close").clicked() {
                    flags.show_menu = false;
                }
            });
    }

    if flags.show_inventory {
        egui::Window::new("Inventory")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Resources");
                for (k, c) in &inventory.resources {
                    ui.label(format!("{:?}: {}", k, c));
                }
                ui.separator();
                ui.heading("Items");
                for it in &inventory.items {
                    ui.label(format!("{} {:?}", it.name, it.kind));
                }
            });
    }

    if flags.show_crafting {
        egui::Window::new("Crafting")
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(book) = recipes {
                    for r in &book.recipes {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} -> {:?}", r.name, r.output_item));
                            if ui.button("Craft").clicked() {
                                if let Some(it) = book.craft(&r.name, inventory) {
                                    // push crafted to inventory
                                    let new_it = it.clone();
                                    inventory.items.push(new_it);
                                    out.crafted = Some(r.name.clone());
                                }
                            }
                        });
                    }
                } else {
                    ui.label("No recipes loaded.");
                }
            });
    }

    if flags.show_map {
        egui::Window::new("Map").resizable(true).show(ctx, |ui| {
            ui.label("World Map (placeholder).");
            ui.add(egui::widgets::Label::new(
                "MiniMap: TODO: render navmesh texture or world quads",
            ));
        });
    }

    if flags.show_quests {
        egui::Window::new("Quest Log")
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(q) = quests {
                    for (id, quest) in &q.quests {
                        ui.heading(format!("{} - {}", id, quest.title));
                        for t in &quest.tasks {
                            ui.label(format!(
                                "{:?} {} {}",
                                t.kind,
                                if t.done { "[done]" } else { "[todo]" },
                                t.id
                            ));
                        }
                        ui.separator();
                    }
                } else {
                    ui.label("No quests.");
                }
            });
    }

    if flags.show_settings {
        egui::Window::new("Settings / Accessibility").resizable(true).show(ctx, |ui| {
            ui.checkbox(&mut acc.high_contrast_ui, "High contrast UI");
            ui.checkbox(&mut acc.reduce_motion, "Reduce motion");
            ui.checkbox(&mut acc.subtitles, "Subtitles");
            ui.add(egui::Slider::new(&mut acc.subtitle_scale, 0.6..=1.8).text("Subtitle scale"));
            let mut cb_idx: usize = match acc.colorblind_mode.as_deref() {
                Some("protanopia") => 1,
                Some("deuteranopia") => 2,
                Some("tritanopia") => 3,
                _ => 0,
            };
            egui::ComboBox::from_label("Colorblind mode").selected_text(match cb_idx {
                1 => "Protanopia",
                2 => "Deuteranopia",
                3 => "Tritanopia",
                _ => "None",
            }).show_ui(ui, |ui| {
                ui.selectable_value(&mut cb_idx, 0, "None");
                ui.selectable_value(&mut cb_idx, 1, "Protanopia");
                ui.selectable_value(&mut cb_idx, 2, "Deuteranopia");
                ui.selectable_value(&mut cb_idx, 3, "Tritanopia");
            });
            acc.colorblind_mode = match cb_idx {1=>Some("protanopia".into()),2=>Some("deuteranopia".into()),3=>Some("tritanopia".into()), _=>None};

            ui.separator();
            ui.label("Input Remapping (press the desired key/gamepad button after selecting an action):");
            ui.label("This demo shows the UX portion; wire into the input manager's remap call.");
        });
    }

    // Simple Cinematics panel (dev-only)
    egui::Window::new("Cinematics").resizable(true).show(ctx, |ui| {
        static mut TL: Option<awc::Timeline> = None;
        static mut SEQ: Option<awc::Sequencer> = None;
        static mut FILENAME: Option<String> = None;
        let mut load_demo = false;
        ui.horizontal(|ui| {
            if ui.button("Load Demo").clicked() { load_demo = true; }
            if ui.button("Save JSON").clicked() {
                unsafe {
                    if let Some(ref tl) = TL { let s = serde_json::to_string_pretty(tl).unwrap(); ui.output_mut(|o| o.copied_text = s); }
                }
            }
            // Load/Save to assets
            unsafe {
                if FILENAME.is_none() { FILENAME = Some("assets/cinematics/cutscene.json".into()); }
                if let Some(name) = FILENAME.as_mut() {
                    ui.text_edit_singleline(name);
                    if ui.button("Load File").clicked() {
                        match std::fs::read_to_string(&*name) {
                            Ok(s) => {
                                match serde_json::from_str::<awc::Timeline>(&s) {
                                    Ok(tl) => { TL = Some(tl); SEQ = Some(awc::Sequencer::new()); }
                                    Err(e) => { ui.label(format!("Parse error: {}", e)); }
                                }
                            }
                            Err(e) => { ui.label(format!("IO error: {}", e)); }
                        }
                    }
                    if ui.button("Save File").clicked() {
                        if let Some(ref tl) = TL {
                            match serde_json::to_string_pretty(tl) {
                                Ok(s) => { let _ = std::fs::create_dir_all("assets/cinematics"); let path = name.clone(); let _ = std::fs::write(path, s); }
                                Err(e) => { ui.label(format!("Serialize error: {}", e)); }
                            }
                        }
                    }
                }
            }
            if ui.button("Play").clicked() {
                unsafe { if SEQ.is_none() { SEQ = Some(awc::Sequencer::new()); } }
            }
            if ui.button("Step 0.5s").clicked() {
                unsafe {
                    if let (Some(seq), Some(tl)) = (SEQ.as_mut(), TL.as_ref()) {
                        if let Ok(evs) = seq.step(0.5, tl) {
                            for e in evs { ui.label(format!("{:4.1}s: {:?}", seq.t.0, e)); }
                        }
                    }
                }
            }
        });
        if load_demo {
            unsafe {
                let mut tl = awc::Timeline::new("cutscene", 5.0);
                tl.tracks.push(awc::Track::Camera { keyframes: vec![
                    awc::CameraKey { t: awc::Time(1.0), pos:(0.0,1.5,3.0), look_at:(0.0,1.0,0.0), fov_deg:60.0 },
                    awc::CameraKey { t: awc::Time(3.0), pos:(2.0,2.0,3.0), look_at:(0.0,1.0,0.0), fov_deg:55.0 },
                ]});
                tl.tracks.push(awc::Track::Audio { clip: "music:start".into(), start: awc::Time(0.2), volume: 0.7 });
                tl.tracks.push(awc::Track::Fx { name: "fade-in".into(), start: awc::Time(0.0), params: serde_json::json!({"duration": 0.5}) });
                TL = Some(tl);
                SEQ = Some(awc::Sequencer::new());
            }
        }
    });

    out
}
