use std::{fs, path::PathBuf, sync::mpsc::channel, time::Duration};
use eframe::egui;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Default)]
struct LevelDoc {
    title: String,
    biome: String,
    seed: u64,
    sky: Sky,
    biome_paints: Vec<BiomePaint>,
    obstacles: Vec<Obstacle>,
    npcs: Vec<NpcSpawn>,
    fate_threads: Vec<FateThread>,
    boss: BossCfg,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct Sky { time_of_day: String, weather: String }

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag="kind")]
enum BiomePaint {
    #[serde(rename="grass_dense")] GrassDense { area: Circle },
    #[serde(rename="moss_path")]  MossPath   { polyline: Vec<[i32;2]> },
}

#[derive(Clone, Serialize, Deserialize)]
struct Circle { cx:i32, cz:i32, radius:i32 }

#[derive(Clone, Serialize, Deserialize, Default)]
struct Obstacle { id:String, pos:[f32;3], yaw:f32, tags:Vec<String> }

#[derive(Clone, Serialize, Deserialize, Default)]
struct NpcSpawn { archetype:String, count:u32, spawn:Spawn, behavior:String }

#[derive(Clone, Serialize, Deserialize, Default)]
struct Spawn { pos:[f32;3], radius:f32 }

#[derive(Clone, Serialize, Deserialize, Default)]
struct FateThread { name:String, triggers:Vec<Trigger>, ops:Vec<DirectorOp> }

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag="kind")]
enum Trigger {
    #[serde(rename="enter_area")] EnterArea { center:[f32;3], radius:f32 }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag="op")]
enum DirectorOp {
    Fortify { area: FortRegion },
    Collapse { area: FortRegion },
    SpawnWave { archetype:String, count:u32, scatter:f32 },
}

#[derive(Clone, Serialize, Deserialize)]
struct FortRegion { cx:i32, cz:i32, r:i32 }

#[derive(Clone, Serialize, Deserialize, Default)]
struct BossCfg { director_budget_script:String, phase_script:String }

struct EditorApp {
    content_root: PathBuf,
    level: LevelDoc,
    status: String,
}

impl Default for EditorApp {
    fn default() -> Self {
        Self {
            content_root: PathBuf::from("content"),
            level: LevelDoc {
                title: "Untitled".into(),
                biome: "temperate_forest".into(),
                seed: 42,
                sky: Sky { time_of_day: "dawn".into(), weather: "clear".into() },
                ..Default::default()
            },
            status: "Ready".into(),
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("AstraWeave Level & Encounter Editor");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("New").clicked() { *self = Self::default(); }
                if ui.button("Open").clicked() {
                    // simple hardcoded example; integrate rfd/native dialog if desired
                    let p = self.content_root.join("levels/forest_breach.level.toml");
                    if let Ok(s) = fs::read_to_string(&p) {
                        match toml::from_str::<LevelDoc>(&s) {
                            Ok(ld) => { self.level = ld; self.status = format!("Opened {:?}", p); }
                            Err(e) => self.status = format!("Open failed: {e}")
                        }
                    }
                }
                if ui.button("Save").clicked() {
                    let dir = self.content_root.join("levels");
                    let _ = fs::create_dir_all(&dir);
                    let p = dir.join(format!("{}.level.toml",
                        self.level.title.replace(' ', "_").to_lowercase()));
                    match toml::to_string_pretty(&self.level) {
                        Ok(txt) => {
                            if let Err(e) = fs::write(&p, txt) {
                                self.status = format!("Save failed: {e}");
                            } else {
                                // Signal hot-reload to the runtime
                                let _ = fs::create_dir_all(&self.content_root);
                                let _ = fs::write(self.content_root.join("reload.signal"),
                                                  Uuid::new_v4().to_string());
                                self.status = format!("Saved {:?}", p);
                            }
                        }
                        Err(e) => self.status = format!("Serialize failed: {e}")
                    }
                }
            });
            ui.label(&self.status);
        });

        egui::SidePanel::left("left").show(ctx, |ui| {
            ui.heading("Level");
            ui.text_edit_singleline(&mut self.level.title);
            ui.text_edit_singleline(&mut self.level.biome);
            ui.add(egui::Slider::new(&mut self.level.seed, 0..=u64::MAX).text("seed"));
            ui.separator();
            ui.heading("Sky");
            ui.text_edit_singleline(&mut self.level.sky.time_of_day);
            ui.text_edit_singleline(&mut self.level.sky.weather);
            ui.separator();
            if ui.button("Add Rock").clicked() {
                self.level.obstacles.push(Obstacle {
                    id:"rock_big_01".into(), pos:[0.0,0.0,0.0], yaw:0.0, tags: vec!["cover".into()]
                });
            }
            if ui.button("Add Wolf Pack").clicked() {
                self.level.npcs.push(NpcSpawn {
                    archetype:"wolf_pack".into(), count:3,
                    spawn:Spawn{pos:[-5.0,0.0,5.0], radius:2.0}, behavior:"patrol".into()
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Canvas (schematic)");
            ui.label("→ Here you can render a simple 2.5D grid preview later.");
            ui.separator();
            ui.collapsing("Obstacles", |ui| {
                for (i, obstacle) in self.level.obstacles.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}: ", i+1));
                        ui.text_edit_singleline(&mut obstacle.id);
                        if ui.button("🗑").clicked() {
                            self.level.obstacles.remove(i);
                            break;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut obstacle.pos[0]).speed(0.1).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut obstacle.pos[1]).speed(0.1).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut obstacle.pos[2]).speed(0.1).prefix("z:"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Yaw:");
                        ui.add(egui::DragValue::new(&mut obstacle.yaw).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Tags:");
                        let mut tags_str = obstacle.tags.join(", ");
                        if ui.text_edit_singleline(&mut tags_str).changed() {
                            obstacle.tags = tags_str.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                    });
                    ui.separator();
                }
            });
            
            ui.collapsing("NPCs", |ui| {
                for (i, npc) in self.level.npcs.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}: ", i+1));
                        ui.text_edit_singleline(&mut npc.archetype);
                        if ui.button("🗑").clicked() {
                            self.level.npcs.remove(i);
                            break;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Count:");
                        ui.add(egui::DragValue::new(&mut npc.count).speed(1.0).clamp_range(1..=20));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut npc.spawn.pos[0]).speed(0.1).prefix("x:"));
                        ui.add(egui::DragValue::new(&mut npc.spawn.pos[1]).speed(0.1).prefix("y:"));
                        ui.add(egui::DragValue::new(&mut npc.spawn.pos[2]).speed(0.1).prefix("z:"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius:");
                        ui.add(egui::DragValue::new(&mut npc.spawn.radius).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Behavior:");
                        ui.text_edit_singleline(&mut npc.behavior);
                    });
                    ui.separator();
                }
            });
            
            ui.collapsing("Fate Threads", |ui| {
                for (i, ft) in self.level.fate_threads.iter_mut().enumerate() {
                    ui.horizontal(|ui| { 
                        ui.label(format!("Thread #{}: ", i+1)); 
                        ui.text_edit_singleline(&mut ft.name);
                        if ui.button("🗑").clicked() {
                            self.level.fate_threads.remove(i);
                            break;
                        }
                    });
                    
                    ui.collapsing("Triggers", |ui| {
                        for (j, trigger) in ft.triggers.iter_mut().enumerate() {
                            match trigger {
                                Trigger::EnterArea { center, radius } => {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Enter Area #{}", j+1));
                                        if ui.button("🗑").clicked() {
                                            ft.triggers.remove(j);
                                            break;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Center:");
                                        ui.add(egui::DragValue::new(&mut center[0]).speed(0.1).prefix("x:"));
                                        ui.add(egui::DragValue::new(&mut center[1]).speed(0.1).prefix("y:"));
                                        ui.add(egui::DragValue::new(&mut center[2]).speed(0.1).prefix("z:"));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        ui.add(egui::DragValue::new(radius).speed(0.1));
                                    });
                                }
                            }
                            ui.separator();
                        }
                        if ui.button("Add Enter Area Trigger").clicked() {
                            ft.triggers.push(Trigger::EnterArea { 
                                center: [0.0, 0.0, 0.0], 
                                radius: 5.0 
                            });
                        }
                    });
                    
                    ui.collapsing("Operations", |ui| {
                        for (j, op) in ft.ops.iter_mut().enumerate() {
                            match op {
                                DirectorOp::Fortify { area } => {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Fortify #{}", j+1));
                                        if ui.button("🗑").clicked() {
                                            ft.ops.remove(j);
                                            break;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Center:");
                                        ui.add(egui::DragValue::new(&mut area.cx).speed(1.0).prefix("x:"));
                                        ui.add(egui::DragValue::new(&mut area.cz).speed(1.0).prefix("z:"));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        ui.add(egui::DragValue::new(&mut area.r).speed(1.0));
                                    });
                                },
                                DirectorOp::Collapse { area } => {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Collapse #{}", j+1));
                                        if ui.button("🗑").clicked() {
                                            ft.ops.remove(j);
                                            break;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Center:");
                                        ui.add(egui::DragValue::new(&mut area.cx).speed(1.0).prefix("x:"));
                                        ui.add(egui::DragValue::new(&mut area.cz).speed(1.0).prefix("z:"));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Radius:");
                                        ui.add(egui::DragValue::new(&mut area.r).speed(1.0));
                                    });
                                },
                                DirectorOp::SpawnWave { archetype, count, scatter } => {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Spawn Wave #{}", j+1));
                                        if ui.button("🗑").clicked() {
                                            ft.ops.remove(j);
                                            break;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Archetype:");
                                        ui.text_edit_singleline(archetype);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Count:");
                                        ui.add(egui::DragValue::new(count).speed(1.0).clamp_range(1..=20));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Scatter:");
                                        ui.add(egui::DragValue::new(scatter).speed(0.1));
                                    });
                                }
                            }
                            ui.separator();
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Add Fortify").clicked() {
                                ft.ops.push(DirectorOp::Fortify { 
                                    area: FortRegion { cx: 0, cz: 0, r: 5 } 
                                });
                            }
                            if ui.button("Add Collapse").clicked() {
                                ft.ops.push(DirectorOp::Collapse { 
                                    area: FortRegion { cx: 0, cz: 0, r: 5 } 
                                });
                            }
                            if ui.button("Add Spawn Wave").clicked() {
                                ft.ops.push(DirectorOp::SpawnWave { 
                                    archetype: "wolf_pack".into(),
                                    count: 3,
                                    scatter: 2.5
                                });
                            }
                        });
                    });
                    ui.separator();
                }
                if ui.button("Add Fate Thread").clicked() {
                    self.level.fate_threads.push(FateThread{
                        name:"new_thread".into(), triggers: vec![], ops: vec![]
                    });
                }
            });
            
            ui.collapsing("Boss Scripts", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Budget Script:");
                    ui.text_edit_singleline(&mut self.level.boss.director_budget_script);
                });
                ui.horizontal(|ui| {
                    ui.label("Phase Script:");
                    ui.text_edit_singleline(&mut self.level.boss.phase_script);
                });
                
                if ui.button("Create Default Scripts").clicked() {
                    // Create default script files if they don't exist
                    let dir = self.content_root.join("encounters");
                    let _ = fs::create_dir_all(&dir);
                    
                    let level_name = self.level.title.replace(' ', "_").to_lowercase();
                    
                    let budget_path = dir.join(format!("{}.budget.rhai", level_name));
                    if !budget_path.exists() {
                        let budget_script = r#"// Return a budget object the engine understands
fn budget_for_tick(tick) {
  // Simple ramp: early defense, later aggression
  if tick < 1200 {
    #{ fortify: 8, collapse: 2, spawn: 4 }
  } else if tick < 2400 {
    #{ fortify: 5, collapse: 3, spawn: 6 }
  } else {
    #{ fortify: 3, collapse: 5, spawn: 8 }
  }
}"#;
                        let _ = fs::write(&budget_path, budget_script);
                    }
                    
                    let phase_path = dir.join(format!("{}.phases.rhai", level_name));
                    if !phase_path.exists() {
                        let phase_script = r#"// Phase transitions by boss HP/time; engine calls these hooks
fn phase_for(hp_pct, seconds) {
  if hp_pct > 0.7 { "phase_intro" }
  else if hp_pct > 0.35 { "phase_mid" }
  else { "phase_final" }
}"#;
                        let _ = fs::write(&phase_path, phase_script);
                    }
                    
                    // Update the paths in the level document
                    self.level.boss.director_budget_script = format!("content/encounters/{}.budget.rhai", level_name);
                    self.level.boss.phase_script = format!("content/encounters/{}.phases.rhai", level_name);
                    
                    self.status = "Created default boss scripts".into();
                }
            });
        });
    }
}

fn main() -> Result<()> {
    // Create content directory if it doesn't exist
    let content_dir = PathBuf::from("content");
    let _ = fs::create_dir_all(&content_dir);
    let _ = fs::create_dir_all(content_dir.join("levels"));
    let _ = fs::create_dir_all(content_dir.join("encounters"));
    
    let options = eframe::NativeOptions::default();
    eframe::run_native("AstraWeave Level & Encounter Editor", options, Box::new(|_| Box::<EditorApp>::default()))?;
    Ok(())
}