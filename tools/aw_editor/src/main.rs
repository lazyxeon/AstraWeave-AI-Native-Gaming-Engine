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

mod brdf_preview;
mod file_watcher; // Task 3: Hot-reload support
mod material_inspector;

use anyhow::Result;
use astraweave_asset::AssetDatabase;
use astraweave_behavior::{BehaviorGraph, BehaviorNode};
use astraweave_core::{IVec2, Team, World};
use astraweave_dialogue::DialogueGraph;
use astraweave_nav::NavMesh;
use astraweave_quests::Quest;
use eframe::egui;
use material_inspector::MaterialInspector;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
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
    #[allow(dead_code)]
    dialogue: DialogueDoc,
    #[allow(dead_code)]
    quest: QuestDoc,
    asset_db: AssetDatabase,
    behavior_graph: BehaviorGraph,
    dialogue_graph: DialogueGraph,
    quest_graph: Quest,
    console_logs: Vec<String>,
    profiler_data: Vec<String>,
    simulation_playing: bool,
    terrain_grid: Vec<Vec<String>>,
    selected_biome: String,
    last_sim_tick: std::time::Instant,
    nav_mesh: NavMesh,
    nav_max_step: f32,
    nav_max_slope_deg: f32,
    sim_world: Option<World>,
    sim_tick_count: u64,
    material_inspector: MaterialInspector,  // NEW - Phase PBR-G Task 2
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
            behavior_graph: BehaviorGraph {
                root: BehaviorNode::Action("idle".into()),
            },
            dialogue_graph: DialogueGraph {
                nodes: vec![astraweave_dialogue::DialogueNode {
                    id: "start".into(),
                    text: "Hello!".into(),
                    responses: vec![astraweave_dialogue::DialogueResponse {
                        text: "Hi!".into(),
                        next_id: None,
                    }],
                }],
            },
            quest_graph: Quest {
                title: "Sample Quest".into(),
                steps: vec![astraweave_quests::QuestStep {
                    description: "Talk to elder.".into(),
                    completed: false,
                }],
            },
            console_logs: vec!["Editor started.".into()],
            profiler_data: vec![],
            simulation_playing: false,
            terrain_grid: vec![vec!["grass".into(); 10]; 10],
            selected_biome: "grass".into(),
            last_sim_tick: std::time::Instant::now(),
            nav_mesh: NavMesh {
                tris: vec![],
                max_step: 0.4,
                max_slope_deg: 60.0,
            },
            nav_max_step: 0.4,
            nav_max_slope_deg: 60.0,
            sim_world: None,
            sim_tick_count: 0,
            material_inspector: MaterialInspector::new(),  // NEW - Phase PBR-G Task 2
        }
    }
}

impl EditorApp {
    fn show_scene_hierarchy(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scene Hierarchy");
        ui.label("ECS entities and components (stub)");
        // TODO: Integrate with ECS world snapshot
    }

    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");
        ui.label("Selected entity properties (stub)");
        // TODO: Show selected entity's components
    }

    fn show_console(&mut self, ui: &mut egui::Ui) {
        ui.heading("Console");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for log in &self.console_logs {
                ui.label(log);
            }
        });
    }

    fn show_profiler(&mut self, ui: &mut egui::Ui) {
        ui.heading("Profiler");
        ui.label("Performance metrics (stub)");
        for data in &self.profiler_data {
            ui.label(data);
        }
    }

    fn show_behavior_graph_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Behavior Graph Editor");
        ui.label("Node-based BT/HTN editor");

        // Simple tree editor for behavior graph
        fn show_node(ui: &mut egui::Ui, node: &mut BehaviorNode) {
            match node {
                BehaviorNode::Action(ref mut s) => {
                    ui.horizontal(|ui| {
                        ui.label("Action:");
                        ui.text_edit_singleline(s);
                    });
                }
                BehaviorNode::Condition(ref mut s) => {
                    ui.horizontal(|ui| {
                        ui.label("Condition:");
                        ui.text_edit_singleline(s);
                    });
                }
                BehaviorNode::Sequence(ref mut children) => {
                    ui.collapsing("Sequence", |ui| {
                        for child in children.iter_mut() {
                            show_node(ui, child);
                        }
                        if ui.button("Add Action").clicked() {
                            children.push(BehaviorNode::Action("new action".into()));
                        }
                    });
                }
                BehaviorNode::Selector(ref mut children) => {
                    ui.collapsing("Selector", |ui| {
                        for child in children.iter_mut() {
                            show_node(ui, child);
                        }
                        if ui.button("Add Action").clicked() {
                            children.push(BehaviorNode::Action("new action".into()));
                        }
                    });
                }
                BehaviorNode::Decorator(_, ref mut child) => {
                    ui.collapsing("Decorator", |ui| {
                        show_node(ui, child);
                    });
                }
                BehaviorNode::Parallel(ref mut children, _) => {
                    ui.collapsing("Parallel", |ui| {
                        for child in children.iter_mut() {
                            show_node(ui, child);
                        }
                        if ui.button("Add Action").clicked() {
                            children.push(BehaviorNode::Action("new action".into()));
                        }
                    });
                }
            }
        }

        show_node(ui, &mut self.behavior_graph.root);

        if ui.button("Validate Graph").clicked() {
            // TODO: Implement validation
            self.console_logs
                .push("Behavior graph validation stub.".into());
        }
    }

    fn show_dialogue_graph_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Dialogue Graph Editor");
        ui.label("Node-based dialogue editor with live validation and editing");

        ui.horizontal(|ui| {
            if ui.button("Add Node").clicked() {
                let new_id = format!("node_{}", self.dialogue_graph.nodes.len());
                self.dialogue_graph
                    .nodes
                    .push(astraweave_dialogue::DialogueNode {
                        id: new_id,
                        text: "New dialogue text".into(),
                        responses: vec![astraweave_dialogue::DialogueResponse {
                            text: "Response".into(),
                            next_id: None,
                        }],
                    });
            }
            if ui.button("Validate Dialogue").clicked() {
                if let Err(e) = self.dialogue_graph.validate() {
                    self.console_logs
                        .push(format!("Dialogue validation error: {}", e));
                } else {
                    self.console_logs.push("Dialogue validated.".into());
                }
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, node) in self.dialogue_graph.nodes.iter_mut().enumerate() {
                ui.collapsing(format!("Node {}: {}", i, node.id), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        ui.text_edit_singleline(&mut node.id);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Text:");
                        ui.text_edit_multiline(&mut node.text);
                    });
                    ui.label("Responses:");
                    let mut to_remove = vec![];
                    for (j, resp) in node.responses.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", j));
                            ui.text_edit_singleline(&mut resp.text);
                            ui.label("Next ID:");
                            let next_id = resp.next_id.get_or_insert(String::new());
                            ui.text_edit_singleline(next_id);
                            if ui.button("Remove").clicked() {
                                to_remove.push(j);
                            }
                        });
                    }
                    // Remove in reverse order to avoid index invalidation
                    for &idx in to_remove.iter().rev() {
                        node.responses.remove(idx);
                    }
                    if ui.button("Add Response").clicked() {
                        node.responses.push(astraweave_dialogue::DialogueResponse {
                            text: "New response".into(),
                            next_id: None,
                        });
                    }
                });
            }
        });
    }

    fn show_quest_graph_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Quest Graph Editor");
        ui.label("Node-based quest editor");

        ui.horizontal(|ui| {
            if ui.button("Add Step").clicked() {
                self.quest_graph.steps.push(astraweave_quests::QuestStep {
                    description: "New quest step".into(),
                    completed: false,
                });
            }
            if ui.button("Validate Quest").clicked() {
                if let Err(e) = self.quest_graph.validate() {
                    self.console_logs
                        .push(format!("Quest validation error: {}", e));
                } else {
                    self.console_logs.push("Quest validated.".into());
                }
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, step) in self.quest_graph.steps.iter_mut().enumerate() {
                ui.collapsing(format!("Step {}: {}", i, step.description), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.text_edit_singleline(&mut step.description);
                    });
                    ui.checkbox(&mut step.completed, "Completed");
                });
            }
        });
    }

    fn show_material_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Material Editor");
        ui.label("Live material editing with hot reload");
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
        if ui.button("Save & Reload Material").clicked() {
            let _ = fs::create_dir_all("assets");
            match serde_json::to_string_pretty(&self.mat_doc) {
                Ok(s) => {
                    if fs::write("assets/material_live.json", s).is_ok() {
                        self.status = "Saved assets/material_live.json".into();
                        // TODO: Trigger hot reload
                    } else {
                        self.status = "Failed to write material_live.json".into();
                    }
                }
                Err(e) => {
                    self.status = format!("Serialize error: {e}");
                }
            }
        }
    }

    fn show_terrain_painter(&mut self, ui: &mut egui::Ui) {
        ui.heading("Terrain Painter");
        ui.label("Click cells to paint biomes");

        ui.horizontal(|ui| {
            ui.label("Selected Biome:");
            egui::ComboBox::from_label("")
                .selected_text(&self.selected_biome)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_biome, "grass".into(), "Grass");
                    ui.selectable_value(&mut self.selected_biome, "forest".into(), "Forest");
                    ui.selectable_value(&mut self.selected_biome, "mountain".into(), "Mountain");
                    ui.selectable_value(&mut self.selected_biome, "water".into(), "Water");
                });
        });

        ui.separator();

        egui::Grid::new("terrain_grid").show(ui, |ui| {
            for (y, row) in self.terrain_grid.iter_mut().enumerate() {
                for (x, cell) in row.iter_mut().enumerate() {
                    let color = match cell.as_str() {
                        "grass" => egui::Color32::GREEN,
                        "forest" => egui::Color32::DARK_GREEN,
                        "mountain" => egui::Color32::GRAY,
                        "water" => egui::Color32::BLUE,
                        _ => egui::Color32::WHITE,
                    };
                    let response = ui.add(
                        egui::Button::new("")
                            .fill(color)
                            .min_size(egui::Vec2::new(20.0, 20.0)),
                    );
                    if response.clicked() {
                        *cell = self.selected_biome.clone();
                    }
                    ui.label(format!("({}, {})", x, y));
                }
                ui.end_row();
            }
        });

        if ui.button("Save Terrain").clicked() {
            let _ = fs::create_dir_all("assets");
            match serde_json::to_string_pretty(&self.terrain_grid) {
                Ok(s) => {
                    if fs::write("assets/terrain_grid.json", s).is_ok() {
                        self.status = "Saved terrain grid".into();
                    } else {
                        self.status = "Failed to save terrain grid".into();
                    }
                }
                Err(e) => self.status = format!("Serialize terrain error: {}", e),
            }
        }

        if ui.button("Load Terrain").clicked() {
            match fs::read_to_string("assets/terrain_grid.json") {
                Ok(s) => match serde_json::from_str::<Vec<Vec<String>>>(&s) {
                    Ok(grid) => {
                        if grid.len() == 10 && grid.iter().all(|r| r.len() == 10) {
                            self.terrain_grid = grid;
                            self.status = "Loaded terrain grid".into();
                        } else {
                            self.status = "Invalid terrain grid format".into();
                        }
                    }
                    Err(e) => self.status = format!("Deserialize terrain error: {}", e),
                },
                Err(e) => self.status = format!("Read terrain error: {}", e),
            }
        }

        if ui.button("Sync with Level").clicked() {
            // Convert grid to biome_paints
            self.level.biome_paints.clear();
            for (y, row) in self.terrain_grid.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    if *cell == "grass" {
                        self.level.biome_paints.push(BiomePaint::GrassDense {
                            area: Circle {
                                cx: x as i32 * 10,
                                cz: y as i32 * 10,
                                radius: 5,
                            },
                        });
                    }
                    // Add others if needed
                }
            }
            self.status = "Synced terrain with level".into();
        }
    }

    fn show_navmesh_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Navmesh Controls");
        ui.label("Baking and visualization controls");

        ui.horizontal(|ui| {
            ui.label("Max Step:");
            ui.add(egui::DragValue::new(&mut self.nav_max_step).speed(0.1));
            ui.label("Max Slope Deg:");
            ui.add(egui::DragValue::new(&mut self.nav_max_slope_deg).speed(1.0));
        });

        if ui.button("Bake Navmesh").clicked() {
            // Generate triangles from level obstacles
            let mut tris = vec![];
            for obs in &self.level.obstacles {
                // Assume obstacle is a 1x1 square on XZ plane at Y=0
                let x = obs.pos[0] as i32;
                let z = obs.pos[2] as i32;
                // Generate two triangles for the square
                tris.push(astraweave_nav::Triangle {
                    a: glam::Vec3::new(x as f32, 0.0, z as f32),
                    b: glam::Vec3::new(x as f32 + 1.0, 0.0, z as f32),
                    c: glam::Vec3::new(x as f32, 0.0, z as f32 + 1.0),
                });
                tris.push(astraweave_nav::Triangle {
                    a: glam::Vec3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0),
                    b: glam::Vec3::new(x as f32, 0.0, z as f32 + 1.0),
                    c: glam::Vec3::new(x as f32 + 1.0, 0.0, z as f32),
                });
            }
            if tris.is_empty() {
                // Fallback to dummy
                for x in 0..9 {
                    for z in 0..9 {
                        tris.push(astraweave_nav::Triangle {
                            a: glam::Vec3::new(x as f32, 0.0, z as f32),
                            b: glam::Vec3::new(x as f32 + 1.0, 0.0, z as f32),
                            c: glam::Vec3::new(x as f32, 0.0, z as f32 + 1.0),
                        });
                        tris.push(astraweave_nav::Triangle {
                            a: glam::Vec3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0),
                            b: glam::Vec3::new(x as f32, 0.0, z as f32 + 1.0),
                            c: glam::Vec3::new(x as f32 + 1.0, 0.0, z as f32),
                        });
                    }
                }
            }
            self.nav_mesh =
                astraweave_nav::NavMesh::bake(&tris, self.nav_max_step, self.nav_max_slope_deg);
            self.console_logs.push(format!(
                "Navmesh baked with {} triangles from level.",
                self.nav_mesh.tris.len()
            ));
        }

        ui.label(format!("Triangles: {}", self.nav_mesh.tris.len()));
    }

    fn show_asset_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Asset Inspector");
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
            if let Ok(()) = self
                .asset_db
                .load_manifest(&PathBuf::from("assets/assets.json"))
            {
                self.status = "Reloaded assets from manifest".into();
            } else {
                let _ = self.asset_db.scan_directory(&PathBuf::from("assets"));
                self.status = "Scanned assets directory".into();
            }
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
                if ui.button("Save JSON").clicked() {
                    let dir = self.content_root.join("levels");
                    let _ = fs::create_dir_all(&dir);
                    let p = dir.join(format!(
                        "{}.level.json",
                        self.level.title.replace(' ', "_").to_lowercase()
                    ));
                    match serde_json::to_string_pretty(&self.level) {
                        Ok(txt) => {
                            if let Err(e) = fs::write(&p, txt) {
                                self.status = format!("Save JSON failed: {e}");
                            } else {
                                self.status = format!("Saved JSON {:?}", p);
                            }
                        }
                        Err(e) => self.status = format!("Serialize JSON failed: {e}"),
                    }
                }
                ui.checkbox(&mut self.simulation_playing, "Play Simulation");
                if self.simulation_playing {
                    ui.label("(Simulating...)");
                }
                if ui.button("Diff Assets").clicked() {
                    match std::process::Command::new("git")
                        .args(&["diff", "assets"])
                        .output()
                    {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            if stdout.is_empty() && stderr.is_empty() {
                                self.console_logs.push("No asset changes.".into());
                            } else {
                                self.console_logs.push(format!("Asset diff:\n{}", stdout));
                                if !stderr.is_empty() {
                                    self.console_logs.push(format!("Diff stderr: {}", stderr));
                                }
                            }
                        }
                        Err(e) => self.console_logs.push(format!("Git diff failed: {}", e)),
                    }
                }
            });
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.collapsing("Scene Hierarchy", |ui| self.show_scene_hierarchy(ui));
                ui.collapsing("Inspector", |ui| self.show_inspector(ui));
                ui.collapsing("Console", |ui| self.show_console(ui));
                ui.collapsing("Profiler", |ui| self.show_profiler(ui));
                ui.collapsing("Behavior Graph Editor", |ui| {
                    self.show_behavior_graph_editor(ui)
                });
                ui.collapsing("Dialogue Graph Editor", |ui| {
                    self.show_dialogue_graph_editor(ui)
                });
                ui.collapsing("Quest Graph Editor", |ui| self.show_quest_graph_editor(ui));
                ui.collapsing("Material Editor", |ui| self.show_material_editor(ui));
                ui.collapsing("Material Inspector", |ui| {
                    self.material_inspector.show(ui, ctx)
                });
                ui.collapsing("Terrain Painter", |ui| self.show_terrain_painter(ui));
                ui.collapsing("Navmesh Controls", |ui| self.show_navmesh_controls(ui));
                ui.collapsing("Asset Inspector", |ui| self.show_asset_inspector(ui));
            });
        });

        if self.simulation_playing {
            // Initialize world if needed
            if self.sim_world.is_none() {
                let mut world = World::new();
                // Add entities from level
                for obs in &self.level.obstacles {
                    let pos = IVec2 {
                        x: obs.pos[0] as i32,
                        y: obs.pos[2] as i32,
                    };
                    let _entity = world.spawn("obstacle", pos, Team { id: 2 }, 100, 0);
                    world.obstacles.insert((pos.x, pos.y));
                }
                for npc in &self.level.npcs {
                    for _ in 0..npc.count {
                        let pos = IVec2 {
                            x: npc.spawn.pos[0] as i32,
                            y: npc.spawn.pos[2] as i32,
                        };
                        world.spawn(&npc.archetype, pos, Team { id: 1 }, 50, 10);
                    }
                }
                self.sim_world = Some(world);
                self.sim_tick_count = 0;
                self.console_logs
                    .push("Simulation started with entities from level.".into());
            }
            // Tick simulation
            let _now = std::time::Instant::now();
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(self.last_sim_tick);
            const TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

            if elapsed >= TICK_INTERVAL {
                let ticks = (elapsed.as_millis() / 100) as u64;
                for _ in 0..ticks {
                    if let Some(world) = &mut self.sim_world {
                        world.tick(0.1); // dt = 0.1s
                                         // Simple behavior: regenerate health
                        for entity in world.entities() {
                            if let Some(health) = world.health_mut(entity) {
                                health.hp = (health.hp + 1).min(100);
                            }
                        }
                        self.sim_tick_count += 1;
                        if self.sim_tick_count % 10 == 0 {
                            self.console_logs.push(format!(
                                "Simulation tick {}: {} entities, time {:.1}s",
                                self.sim_tick_count,
                                world.entities().len(),
                                world.t
                            ));
                        }
                    }
                }
                self.last_sim_tick += TICK_INTERVAL * ticks as u32;
            }
        } else {
            // Stop simulation
            if self.sim_world.is_some() {
                self.sim_world = None;
                self.console_logs.push("Simulation stopped.".into());
            }
        }
    }
}

fn main() -> Result<()> {
    // Initialize observability
    astraweave_observability::init_observability(Default::default())
        .expect("Failed to initialize observability");

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
