#[derive(Clone, Serialize, Deserialize, Default)]
struct DialogueDoc {
    title: String,
    nodes: Vec<DialogueNode>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct DialogueNode {
    id: String,
    text: String,
    responses: Vec<DialogueResponse>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct DialogueResponse {
    text: String,
    next_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct QuestDoc {
    title: String,
    steps: Vec<QuestStep>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct QuestStep {
    description: String,
    completed: bool,
}
use anyhow::Result;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use astraweave_asset::AssetDatabase;
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
struct Sky {
    time_of_day: String,
    weather: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum BiomePaint {
    #[serde(rename = "grass_dense")]
    GrassDense { area: Circle },
    #[serde(rename = "moss_path")]
    MossPath { polyline: Vec<[i32; 2]> },
}

#[derive(Clone, Serialize, Deserialize)]
struct Circle {
    cx: i32,
    cz: i32,
    radius: i32,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct Obstacle {
    id: String,
    pos: [f32; 3],
    yaw: f32,
    tags: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct NpcSpawn {
    archetype: String,
    count: u32,
    spawn: Spawn,
    behavior: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct Spawn {
    pos: [f32; 3],
    radius: f32,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct FateThread {
    name: String,
    triggers: Vec<Trigger>,
    ops: Vec<DirectorOp>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Trigger {
    #[serde(rename = "enter_area")]
    EnterArea { center: [f32; 3], radius: f32 },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
enum DirectorOp {
    Fortify {
        area: FortRegion,
    },
    Collapse {
        area: FortRegion,
    },
    SpawnWave {
        archetype: String,
        count: u32,
        scatter: f32,
    },
}

#[derive(Clone, Serialize, Deserialize)]
struct FortRegion {
    cx: i32,
    cz: i32,
    r: i32,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct BossCfg {
    director_budget_script: String,
    phase_script: String,
}

struct EditorApp {
    content_root: PathBuf,
    level: LevelDoc,
    status: String,
    mat_doc: MaterialLiveDoc,
    dialogue: DialogueDoc,
    quest: QuestDoc,
    asset_db: AssetDatabase,
}

impl Default for EditorApp {
    fn default() -> Self {
        let mut asset_db = AssetDatabase::new();
        // Try to load from assets.json
        if let Ok(()) = asset_db.load_manifest(&PathBuf::from("assets/assets.json")) {
            // Loaded
        } else {
            // Scan assets directory
            let _ = asset_db.scan_directory(&PathBuf::from("assets"));
        }
        Self {
            content_root: PathBuf::from("content"),
            level: LevelDoc {
                title: "Untitled".into(),
                biome: "temperate_forest".into(),
                seed: 42,
                sky: Sky {
                    time_of_day: "dawn".into(),
                    weather: "clear".into(),
                },
                ..Default::default()
            },
            status: "Ready".into(),
            mat_doc: MaterialLiveDoc {
                base_color: [1.0, 1.0, 1.0, 1.0],
                metallic: 0.1,
                roughness: 0.6,
                texture_path: None,
            },
            dialogue: DialogueDoc {
                title: "Sample Dialogue".into(),
                nodes: vec![DialogueNode {
                    id: "start".into(),
                    text: "Hello, traveler!".into(),
                    responses: vec![DialogueResponse {
                        text: "Hi!".into(),
                        next_id: None,
                    }],
                }],
            },
            quest: QuestDoc {
                title: "Sample Quest".into(),
                steps: vec![QuestStep {
                    description: "Talk to the elder.".into(),
                    completed: false,
                }],
            },
            asset_db,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaterialLiveDoc {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    texture_path: Option<String>,
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("AstraWeave Level & Encounter Editor");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    *self = Self::default();
                }
                if ui.button("Open").clicked() {
                    // simple hardcoded example; integrate rfd/native dialog if desired
                    let p = self.content_root.join("levels/forest_breach.level.toml");
                    if let Ok(s) = fs::read_to_string(&p) {
                        match toml::from_str::<LevelDoc>(&s) {
                            Ok(ld) => {
                                self.level = ld;
                                self.status = format!("Opened {:?}", p);
                            }
                            Err(e) => self.status = format!("Open failed: {e}"),
                        }
                    }
                }
                if ui.button("Save").clicked() {
                    let dir = self.content_root.join("levels");
                    let _ = fs::create_dir_all(&dir);
                    let p = dir.join(format!(
                        "{}.level.toml",
                        self.level.title.replace(' ', "_").to_lowercase()
                    ));
                    match toml::to_string_pretty(&self.level) {
                        Ok(txt) => {
                            if let Err(e) = fs::write(&p, txt) {
                                self.status = format!("Save failed: {e}");
                            } else {
                                // Signal hot-reload to the runtime
                                let _ = fs::create_dir_all(&self.content_root);
                                let _ = fs::write(
                                    self.content_root.join("reload.signal"),
                                    Uuid::new_v4().to_string(),
                                );
                                self.status = format!("Saved {:?}", p);
                            }
                        }
                        Err(e) => self.status = format!("Serialize failed: {e}"),
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
                    id: "rock_big_01".into(),
                    pos: [0.0, 0.0, 0.0],
                    yaw: 0.0,
                    tags: vec!["cover".into()],
                });
            }
            if ui.button("Add Wolf Pack").clicked() {
                self.level.npcs.push(NpcSpawn {
                    archetype: "wolf_pack".into(),
                    count: 3,
                    spawn: Spawn {
                        pos: [-5.0, 0.0, 5.0],
                        radius: 2.0,
                    },
                    behavior: "patrol".into(),
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Canvas (schematic)");
            ui.label("→ Here you can render a simple 2.5D grid preview later.");
            ui.separator();

            ui.collapsing("Dialogue Editor", |ui| {
                ui.text_edit_singleline(&mut self.dialogue.title);
                for node in &mut self.dialogue.nodes {
                    ui.group(|ui| {
                        ui.text_edit_singleline(&mut node.id);
                        ui.text_edit_multiline(&mut node.text);
                        for resp in &mut node.responses {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut resp.text);
                                ui.text_edit_singleline(resp.next_id.get_or_insert(String::new()));
                            });
                        }
                        if ui.button("Add Response").clicked() {
                            node.responses.push(DialogueResponse::default());
                        }
                    });
                }
                if ui.button("Add Node").clicked() {
                    self.dialogue.nodes.push(DialogueNode::default());
                }
                if ui.button("Save Dialogue").clicked() {
                    let dir = self.content_root.join("dialogue");
                    let _ = fs::create_dir_all(&dir);
                    let p = dir.join(format!("{}.dialogue.toml", self.dialogue.title.replace(' ', "_").to_lowercase()));
                    match toml::to_string_pretty(&self.dialogue) {
                        Ok(txt) => {
                            if let Err(e) = fs::write(&p, txt) {
                                self.status = format!("Save dialogue failed: {e}");
                            } else {
                                let _ = fs::write(self.content_root.join("reload.signal"), Uuid::new_v4().to_string());
                                self.status = format!("Saved dialogue to {:?}", p);
                            }
                        }
                        Err(e) => self.status = format!("Serialize dialogue failed: {e}"),
                    }
                }
            });

            ui.collapsing("Quest Editor", |ui| {
                ui.text_edit_singleline(&mut self.quest.title);
                for step in &mut self.quest.steps {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut step.description);
                        ui.checkbox(&mut step.completed, "Completed");
                    });
                }
                if ui.button("Add Step").clicked() {
                    self.quest.steps.push(QuestStep::default());
                }
                if ui.button("Save Quest").clicked() {
                    let dir = self.content_root.join("quests");
                    let _ = fs::create_dir_all(&dir);
                    let p = dir.join(format!("{}.quest.toml", self.quest.title.replace(' ', "_").to_lowercase()));
                    match toml::to_string_pretty(&self.quest) {
                        Ok(txt) => {
                            if let Err(e) = fs::write(&p, txt) {
                                self.status = format!("Save quest failed: {e}");
                            } else {
                                let _ = fs::write(self.content_root.join("reload.signal"), Uuid::new_v4().to_string());
                                self.status = format!("Saved quest to {:?}", p);
                            }
                        }
                        Err(e) => self.status = format!("Serialize quest failed: {e}"),
                    }
                }
            });
            ui.collapsing("Material Inspector (Live)", |ui| {
                ui.label("Adjust PBR params and write to assets/material_live.json");
                ui.add(egui::Slider::new(&mut self.mat_doc.base_color[0], 0.0..=1.0).text("Base R"));
                ui.add(egui::Slider::new(&mut self.mat_doc.base_color[1], 0.0..=1.0).text("Base G"));
                ui.add(egui::Slider::new(&mut self.mat_doc.base_color[2], 0.0..=1.0).text("Base B"));
                ui.add(egui::Slider::new(&mut self.mat_doc.metallic, 0.0..=1.0).text("Metallic"));
                ui.add(egui::Slider::new(&mut self.mat_doc.roughness, 0.04..=1.0).text("Roughness"));
                let tex_ref = self.mat_doc.texture_path.get_or_insert(String::new());
                ui.horizontal(|ui| {
                    ui.label("Texture path:");
                    ui.text_edit_singleline(tex_ref);
                });
                if ui.button("Save Live Material").clicked() {
                    let _ = fs::create_dir_all("assets");
                    match serde_json::to_string_pretty(&self.mat_doc) {
                        Ok(s) => {
                            if fs::write("assets/material_live.json", s).is_ok() {
                                self.status = "Saved assets/material_live.json".into();
                            } else {
                                self.status = "Failed to write material_live.json".into();
                            }
                        }
                        Err(e) => { self.status = format!("Serialize error: {e}"); }
                    }
                }
            });
            ui.collapsing("Obstacles", |ui| {
                let mut to_remove = None;
                for (i, obstacle) in self.level.obstacles.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}: ", i+1));
                        ui.text_edit_singleline(&mut obstacle.id);
                        if ui.button("🗑").clicked() {
                            to_remove = Some(i);
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
                    if to_remove.is_some() {
                        break;
                    }
                }
                if let Some(index) = to_remove {
                    self.level.obstacles.remove(index);
                }
            });

            ui.collapsing("NPCs", |ui| {
                let mut to_remove = None;
                for (i, npc) in self.level.npcs.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}: ", i+1));
                        ui.text_edit_singleline(&mut npc.archetype);
                        if ui.button("🗑").clicked() {
                            to_remove = Some(i);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Count:");
                        ui.add(egui::DragValue::new(&mut npc.count).speed(1.0).range(1..=20));
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
                    if to_remove.is_some() {
                        break;
                    }
                }
                if let Some(index) = to_remove {
                    self.level.npcs.remove(index);
                }
            });

            ui.collapsing("Fate Threads", |ui| {
                let mut to_remove = None;
                for (i, ft) in self.level.fate_threads.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Thread #{}: ", i+1));
                        ui.text_edit_singleline(&mut ft.name);
                        if ui.button("🗑").clicked() {
                            to_remove = Some(i);
                        }
                    });

                    ui.collapsing("Triggers", |ui| {
                        let mut trigger_to_remove = None;
                        for (j, trigger) in ft.triggers.iter_mut().enumerate() {
                            match trigger {
                                Trigger::EnterArea { center, radius } => {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Enter Area #{}", j+1));
                                        if ui.button("🗑").clicked() {
                                            trigger_to_remove = Some(j);
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
                            if trigger_to_remove.is_some() {
                                break;
                            }
                        }
                        if let Some(index) = trigger_to_remove {
                            ft.triggers.remove(index);
                        }
                        if ui.button("Add Enter Area Trigger").clicked() {
                            ft.triggers.push(Trigger::EnterArea {
                                center: [0.0, 0.0, 0.0],
                                radius: 5.0
                            });
                        }
                    });

                    ui.collapsing("Operations", |ui| {
                        let mut op_to_remove = None;
                        for (j, op) in ft.ops.iter_mut().enumerate() {
                            match op {
                                DirectorOp::Fortify { area } => {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Fortify #{}", j+1));
                                        if ui.button("🗑").clicked() {
                                            op_to_remove = Some(j);
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
                                            op_to_remove = Some(j);
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
                                            op_to_remove = Some(j);
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Archetype:");
                                        ui.text_edit_singleline(archetype);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Count:");
                                        ui.add(egui::DragValue::new(count).speed(1.0).range(1..=20));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Scatter:");
                                        ui.add(egui::DragValue::new(scatter).speed(0.1));
                                    });
                                }
                            }
                            ui.separator();
                            if op_to_remove.is_some() {
                                break;
                            }
                        }
                        if let Some(index) = op_to_remove {
                            ft.ops.remove(index);
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
                    if to_remove.is_some() {
                        break;
                    }
                }
                if let Some(index) = to_remove {
                    self.level.fate_threads.remove(index);
                }
                if ui.button("Add Fate Thread").clicked() {
                    self.level.fate_threads.push(FateThread{
                        name:"new_thread".into(), triggers: vec![], ops: vec![]
                    });
                }
            });

            ui.collapsing("Asset Inspector", |ui| {
                ui.label(format!("Total assets: {}", self.asset_db.assets.len()));
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (guid, meta) in &self.asset_db.assets {
                        ui.collapsing(format!("{} ({})", meta.path, guid), |ui| {
                            ui.label(format!("Kind: {:?}", meta.kind));
                            ui.label(format!("Size: {} bytes", meta.size_bytes));
                            ui.label(format!("Hash: {}", &meta.hash[..16]));
                            ui.label(format!("Modified: {}", meta.last_modified));
                            if !meta.dependencies.is_empty() {
                                ui.label("Dependencies:");
                                for dep in &meta.dependencies {
                                    ui.label(format!("  {}", dep));
                                }
                            }
                        });
                    }
                });
                if ui.button("Reload Assets").clicked() {
                    self.asset_db = AssetDatabase::new();
                    if let Ok(()) = self.asset_db.load_manifest(&PathBuf::from("assets/assets.json")) {
                        self.status = "Reloaded assets from manifest".into();
                    } else {
                        let _ = self.asset_db.scan_directory(&PathBuf::from("assets"));
                        self.status = "Scanned assets directory".into();
                    }
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
    eframe::run_native(
        "AstraWeave Level & Encounter Editor",
        options,
        Box::new(|_| Ok(Box::<EditorApp>::default())),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e))?;
    Ok(())
}
