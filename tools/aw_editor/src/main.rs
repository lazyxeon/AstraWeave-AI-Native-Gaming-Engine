#![allow(dead_code)] // Large portions of the legacy editor are scaffolded but not yet reconnected

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

mod behavior_graph;
mod brdf_preview;
mod clipboard; // Phase 3.4 - Copy/Paste/Duplicate
mod command; // Phase 2.1 - Undo/Redo system
mod component_ui; // Phase 2.3 - Component-based inspector
mod editor_mode; // Phase 4.2 - Play-in-Editor
mod entity_manager;
mod file_watcher;
mod gizmo;
mod interaction;
mod material_inspector;
mod panels;
mod prefab; // Phase 4.1 - Prefab System
mod recent_files; // Phase 3 - Recent files tracking
mod scene_serialization; // Phase 2.2 - Scene Save/Load
mod scene_state;
mod telemetry;
mod ui; // Phase 3 - UI components (StatusBar, etc.)
mod viewport; // Phase 1.1 - 3D Viewport
              // mod voxel_tools;  // Temporarily disabled - missing astraweave-terrain dependency

use crate::command::{
    spawn_prefab_with_undo, EditAmmoCommand, EditHealthCommand, EditTeamCommand,
    PrefabApplyOverridesCommand, PrefabRevertOverridesCommand,
};
use anyhow::{anyhow, Result};
use astraweave_asset::AssetDatabase;
use astraweave_core::{Entity, IVec2, Team, World};
use astraweave_dialogue::DialogueGraph;
use astraweave_nav::NavMesh;
use astraweave_quests::Quest;
use behavior_graph::{BehaviorGraphDocument, BehaviorGraphEditorUi};
use editor_mode::EditorMode;
use eframe::egui;
use entity_manager::SelectionSet;
use gizmo::snapping::SnappingConfig;
use gizmo::state::GizmoMode;
use material_inspector::MaterialInspector;
use panels::{
    AdvancedWidgetsPanel, AnimationPanel, AssetBrowser, ChartsPanel, EntityPanel, GraphPanel,
    HierarchyPanel, Panel, PerformancePanel, TransformPanel, WorldPanel,
};
use prefab::{PrefabData, PrefabManager, PrefabManagerHandle};
use recent_files::RecentFilesManager;
use scene_serialization::SceneData;
use scene_state::EditorSceneState;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use ui::StatusBar;
use uuid::Uuid;
use viewport::ViewportWidget; // Phase 1.1

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
    behavior_graph: BehaviorGraphDocument,
    behavior_graph_ui: BehaviorGraphEditorUi,
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
    scene_state: EditorSceneState,
    runtime_world: Option<World>,
    sim_tick_count: u64,
    material_inspector: MaterialInspector, // NEW - Phase PBR-G Task 2
    selected_entity: Option<u64>,
    // Phase 2.1: Undo/Redo system
    undo_stack: command::UndoStack,
    // Phase 2.2: Scene Save/Load
    current_scene_path: Option<PathBuf>,
    last_autosave: std::time::Instant,
    // Phase 3.4: Copy/Paste/Duplicate
    clipboard: Option<clipboard::ClipboardData>,
    // Astract panels
    world_panel: WorldPanel,
    entity_panel: EntityPanel,
    performance_panel: PerformancePanel,
    charts_panel: ChartsPanel,
    advanced_widgets_panel: AdvancedWidgetsPanel,
    graph_panel: GraphPanel,
    animation_panel: AnimationPanel,
    transform_panel: TransformPanel,
    asset_browser: AssetBrowser,
    hierarchy_panel: HierarchyPanel,
    // 3D Viewport (Phase 1.1 - Babylon.js-style editor)
    viewport: Option<ViewportWidget>,
    // Phase 3.5: StatusBar tracking
    current_gizmo_mode: GizmoMode,
    selection_set: SelectionSet,
    last_frame_time: std::time::Instant,
    current_fps: f32,
    recent_files: RecentFilesManager,
    // Phase 4.1: Prefab System
    prefab_manager: PrefabManagerHandle,
    // Phase 4.2: Play-in-Editor
    editor_mode: EditorMode,
    world_snapshot: Option<SceneData>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let scene_state = EditorSceneState::new(Self::create_default_world());
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
            behavior_graph: BehaviorGraphDocument::new_default(),
            behavior_graph_ui: BehaviorGraphEditorUi::default(),
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
            scene_state,
            runtime_world: None,
            sim_tick_count: 0,
            material_inspector: MaterialInspector::new(), // NEW - Phase PBR-G Task 2
            selected_entity: None,
            // Phase 2.1: Undo/Redo system
            undo_stack: command::UndoStack::new(100), // Store last 100 commands
            // Phase 2.2: Scene Save/Load
            current_scene_path: None,
            last_autosave: std::time::Instant::now(),
            // Phase 3.4: Copy/Paste/Duplicate
            clipboard: None,
            // Initialize Astract panels
            world_panel: WorldPanel::new(),
            entity_panel: EntityPanel::new(),
            performance_panel: PerformancePanel::new(),
            charts_panel: ChartsPanel::new(),
            advanced_widgets_panel: AdvancedWidgetsPanel::new(),
            graph_panel: GraphPanel::new(),
            animation_panel: AnimationPanel::default(),
            transform_panel: TransformPanel::new(),
            asset_browser: AssetBrowser::new(PathBuf::from("assets")),
            hierarchy_panel: HierarchyPanel::new(),
            // Viewport initialized in new() method (requires CreationContext)
            viewport: None,
            // Phase 3.5: StatusBar state
            current_gizmo_mode: GizmoMode::Inactive,
            selection_set: SelectionSet::new(),
            last_frame_time: std::time::Instant::now(),
            current_fps: 60.0,
            recent_files: RecentFilesManager::load(),
            // Phase 4.1: Prefab System
            prefab_manager: PrefabManager::shared("prefabs"),
            // Phase 4.2: Play-in-Editor
            editor_mode: EditorMode::default(),
            world_snapshot: None,
        }
    }
}

impl EditorApp {
    fn edit_world(&self) -> &World {
        self.scene_state.world()
    }

    fn edit_world_mut(&mut self) -> &mut World {
        self.scene_state.world_mut()
    }

    fn runtime_world_mut(&mut self) -> Option<&mut World> {
        self.runtime_world.as_mut()
    }

    fn lock_prefab_manager(&self) -> Result<std::sync::MutexGuard<'_, PrefabManager>> {
        self.prefab_manager
            .lock()
            .map_err(|_| anyhow!("Prefab manager lock poisoned"))
    }

    fn with_world_and_undo_stack<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut command::UndoStack, &mut World) -> R,
    {
        let mut undo_stack = std::mem::take(&mut self.undo_stack);
        let result = {
            let world = self.scene_state.world_mut();
            f(&mut undo_stack, world)
        };
        self.undo_stack = undo_stack;
        result
    }

    fn set_snapping_config(&self, new_config: SnappingConfig) {
        let current = self.scene_state.snapping_config();
        if current != new_config {
            self.scene_state.update_snapping(|cfg| *cfg = new_config);
        }
    }

    fn prefab_spawn_coords(&self, world_point: glam::Vec3) -> (i32, i32) {
        let cfg = self.scene_state.snapping_config();
        let mut x = world_point.x;
        let mut z = world_point.z;

        if cfg.grid_enabled && cfg.grid_size > 0.0 {
            x = (x / cfg.grid_size).round() * cfg.grid_size;
            z = (z / cfg.grid_size).round() * cfg.grid_size;
        }

        (x.round() as i32, z.round() as i32)
    }

    fn handle_prefab_drop(
        &mut self,
        prefab_path: PathBuf,
        world_point: glam::Vec3,
        viewport: &mut ViewportWidget,
    ) {
        let spawn_coords = self.prefab_spawn_coords(world_point);
        match self.spawn_prefab_for_drop(prefab_path.clone(), spawn_coords) {
            Ok(entity) => {
                viewport.set_selected_entity(Some(entity));
                self.selected_entity = Some(entity as u64);
                self.selection_set.clear();
                self.selection_set.add(entity as u64, true);
                self.scene_state.sync_all();
                let message = format!(
                    "Spawned prefab {} at ({}, {})",
                    prefab_path.display(),
                    spawn_coords.0,
                    spawn_coords.1
                );
                self.status = message.clone();
                self.console_logs.push(format!("✅ {}", message));
            }
            Err(err) => {
                let message = format!("Failed to spawn prefab {}: {}", prefab_path.display(), err);
                self.console_logs.push(format!("❌ {}", message));
                self.status = message;
            }
        }
    }

    fn spawn_prefab_for_drop(
        &mut self,
        prefab_path: PathBuf,
        spawn_coords: (i32, i32),
    ) -> Result<Entity> {
        let mut undo_stack = std::mem::take(&mut self.undo_stack);
        let entity = spawn_prefab_with_undo(
            self.prefab_manager.clone(),
            prefab_path,
            spawn_coords,
            self.scene_state.world_mut(),
            &mut undo_stack,
        )?;
        self.undo_stack = undo_stack;
        Ok(entity)
    }

    fn notify_prefab_override(&mut self, entity: Entity) {
        let pose = self.scene_state.world().pose(entity);
        let health = self.scene_state.world().health(entity);

        if pose.is_none() && health.is_none() {
            return;
        }

        if let Ok(mut manager) = self.prefab_manager.lock() {
            manager.track_override_snapshot(entity, pose, health);
        }
    }

    fn process_viewport_events(&mut self, viewport: &mut ViewportWidget) {
        let events = viewport.take_pending_events();
        if let Some(metadata) = events.gizmo_commit {
            self.notify_prefab_override(metadata.entity);
        }
    }

    /// Create a default world with sample entities for viewport testing
    ///
    /// Spawns:
    /// - 10 companions (Team 0, blue) in a line at Y=0
    /// - 10 enemies (Team 1, red) in a line at Y=20
    fn create_default_world() -> World {
        let mut world = World::new();

        // Spawn 10 companion entities (blue team)
        for i in 0..10 {
            let pos = IVec2 { x: i * 3, y: 0 }; // Spread along X axis
            world.spawn(
                &format!("Companion_{}", i),
                pos,
                Team { id: 0 }, // Team 0 = companion
                100,            // HP
                30,             // Ammo
            );
        }

        // Spawn 10 enemy entities (red team)
        for i in 0..10 {
            let pos = IVec2 { x: i * 3, y: 20 }; // Spread along X axis, offset in Z
            world.spawn(
                &format!("Enemy_{}", i),
                pos,
                Team { id: 1 }, // Team 1 = enemy
                80,             // HP
                20,             // Ammo
            );
        }

        world
    }

    /// Create editor with CreationContext (for wgpu access)
    ///
    /// This method initializes the 3D viewport, which requires access to
    /// eframe's wgpu render state.
    ///
    /// # Errors
    ///
    /// Returns error if viewport initialization fails (missing wgpu support).
    fn new(cc: &eframe::CreationContext) -> Result<Self> {
        let mut app = Self::default();

        // Initialize viewport (requires wgpu render state from CreationContext)
        match ViewportWidget::new(cc) {
            Ok(viewport) => {
                app.viewport = Some(viewport);
                app.console_logs.push("✅ 3D Viewport initialized".into());
            }
            Err(e) => {
                app.console_logs
                    .push(format!("⚠️  Viewport init failed: {}", e));
                eprintln!("❌ Viewport initialization failed: {}", e);
                // Continue without viewport (fallback to 2D mode)
            }
        }

        Ok(app)
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
        ui.label("Node-based BT/HTN editor with RON round-tripping");
        let logs = &mut self.console_logs;
        self.behavior_graph_ui
            .show(ui, &mut self.behavior_graph, |msg| logs.push(msg));
        ui.separator();
        self.draw_behavior_binding_controls(ui);
    }

    fn draw_behavior_binding_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Entity Binding");
        let Some(selected_raw) = self.selected_entity else {
            ui.label("Select an entity in the viewport to bind this graph.");
            return;
        };
        let entity = selected_raw as Entity;
        if self.scene_state.world().pose(entity).is_none() {
            ui.label("Selected entity is no longer present in the scene.");
            return;
        }

        let entity_name = self.scene_state.world().name(entity).unwrap_or("Unnamed");
        ui.label(format!(
            "Currently editing entity #{entity} ({entity_name})"
        ));

        if ui.button("Assign Graph To Entity").clicked() {
            match self.behavior_graph.to_runtime() {
                Ok(graph) => {
                    self.scene_state
                        .world_mut()
                        .set_behavior_graph(entity, graph);
                    self.console_logs
                        .push(format!("🔗 Assigned behavior graph to entity #{entity}"));
                }
                Err(err) => {
                    self.console_logs
                        .push(format!("❌ Failed to assign behavior graph: {err}"));
                }
            }
        }

        let existing_graph = self.scene_state.world().behavior_graph(entity).cloned();

        if let Some(graph) = existing_graph {
            ui.colored_label(
                egui::Color32::from_rgb(120, 200, 255),
                "Entity has a behavior graph bound",
            );
            ui.horizontal(|ui| {
                if ui.button("Load Graph From Entity").clicked() {
                    self.behavior_graph = BehaviorGraphDocument::from_runtime(&graph);
                    self.behavior_graph.mark_clean();
                    self.console_logs
                        .push(format!("📥 Loaded behavior graph from entity #{entity}"));
                }
                if ui.button("Clear Binding").clicked() {
                    self.scene_state.world_mut().remove_behavior_graph(entity);
                    self.console_logs
                        .push(format!("🧹 Cleared behavior graph from entity #{entity}"));
                }
            });
        } else {
            ui.label("Entity has no behavior graph assigned.");
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
                        self.console_logs
                            .push("✅ Material saved to assets/material_live.json".into());
                        // TODO: Trigger hot reload
                    } else {
                        self.status = "Failed to write material_live.json".into();
                        self.console_logs
                            .push("❌ Failed to write material file".into());
                    }
                }
                Err(e) => {
                    self.status = format!("Serialize error: {e}");
                    self.console_logs
                        .push(format!("❌ Material serialization error: {}", e));
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
                        self.console_logs
                            .push("✅ Terrain grid saved to assets/terrain_grid.json".into());
                    } else {
                        self.status = "Failed to save terrain grid".into();
                        self.console_logs
                            .push("❌ Failed to write terrain grid file".into());
                    }
                }
                Err(e) => {
                    self.status = format!("Serialize terrain error: {}", e);
                    self.console_logs
                        .push(format!("❌ Terrain serialization error: {}", e));
                }
            }
        }

        if ui.button("Load Terrain").clicked() {
            match fs::read_to_string("assets/terrain_grid.json") {
                Ok(s) => match serde_json::from_str::<Vec<Vec<String>>>(&s) {
                    Ok(grid) => {
                        if grid.len() == 10 && grid.iter().all(|r| r.len() == 10) {
                            self.terrain_grid = grid;
                            self.status = "Loaded terrain grid".into();
                            self.console_logs.push(
                                "✅ Terrain grid loaded from assets/terrain_grid.json".into(),
                            );
                        } else {
                            self.status = "Invalid terrain grid format".into();
                            self.console_logs
                                .push("❌ Invalid terrain grid format (must be 10x10)".into());
                        }
                    }
                    Err(e) => {
                        self.status = format!("Deserialize terrain error: {}", e);
                        self.console_logs
                            .push(format!("❌ Failed to parse terrain file: {}", e));
                    }
                },
                Err(e) => {
                    self.status = format!("Read terrain error: {}", e);
                    self.console_logs
                        .push(format!("❌ Failed to read terrain file: {}", e));
                }
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
            let tri_count = self.nav_mesh.tris.len();
            self.console_logs.push(format!(
                "✅ Navmesh baked: {} triangles, max_step={}, max_slope={}°",
                tri_count, self.nav_max_step, self.nav_max_slope_deg
            ));
            self.status = format!("Navmesh baked ({} triangles)", tri_count);
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
                self.console_logs.push(format!(
                    "✅ Assets reloaded from manifest: {} total",
                    self.asset_db.assets.len()
                ));
            } else {
                let _ = self.asset_db.scan_directory(&PathBuf::from("assets"));
                self.status = "Rescanned assets directory".into();
                self.console_logs.push(format!(
                    "✅ Assets rescanned from directory: {} total",
                    self.asset_db.assets.len()
                ));
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
        let now = std::time::Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.current_fps = if frame_time > 0.0 {
            1.0 / frame_time
        } else {
            60.0
        };

        // Phase 2.1 & 2.2: Global hotkeys for undo/redo and scene save/load
        ctx.input(|i| {
            // Ctrl+Z: Undo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) && !i.modifiers.shift {
                if self.editor_mode.can_edit() {
                    let result =
                        self.with_world_and_undo_stack(|undo_stack, world| undo_stack.undo(world));
                    if let Err(e) = result {
                        self.console_logs.push(format!("❌ Undo failed: {}", e));
                    } else if let Some(desc) = self.undo_stack.redo_description() {
                        self.status = format!("⏮️  Undid: {}", desc);
                        self.console_logs.push(format!("⏮️  Undo: {}", desc));
                    }
                }
            }

            // Ctrl+Y or Ctrl+Shift+Z: Redo
            if (i.modifiers.ctrl && i.key_pressed(egui::Key::Y))
                || (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z))
            {
                if self.editor_mode.can_edit() {
                    let result =
                        self.with_world_and_undo_stack(|undo_stack, world| undo_stack.redo(world));
                    if let Err(e) = result {
                        self.console_logs.push(format!("❌ Redo failed: {}", e));
                    } else if let Some(desc) = self.undo_stack.undo_description() {
                        self.status = format!("⏭️  Redid: {}", desc);
                        self.console_logs.push(format!("⏭️  Redo: {}", desc));
                    }
                }
            }

            // Ctrl+S: Save Scene
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                let world = self.edit_world();
                let path = if let Some(p) = &self.current_scene_path {
                    p.clone()
                } else {
                    let dir = self.content_root.join("scenes");
                    let _ = fs::create_dir_all(&dir);
                    dir.join("untitled.scene.ron")
                };

                match scene_serialization::save_scene(world, &path) {
                    Ok(()) => {
                        self.current_scene_path = Some(path.clone());
                        self.recent_files.add_file(path.clone());
                        self.status = format!("💾 Saved scene to {:?}", path);
                        self.console_logs
                            .push(format!("✅ Scene saved: {:?}", path));
                        self.last_autosave = std::time::Instant::now();
                    }
                    Err(e) => {
                        self.status = format!("❌ Scene save failed: {}", e);
                        self.console_logs
                            .push(format!("❌ Failed to save scene: {}", e));
                    }
                }
            }

            // Ctrl+O: Load Scene
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                let path = self.content_root.join("scenes/untitled.scene.ron");
                match scene_serialization::load_scene(&path) {
                    Ok(world) => {
                        self.scene_state = EditorSceneState::new(world);
                        self.current_scene_path = Some(path.clone());
                        self.recent_files.add_file(path.clone());
                        self.status = format!("📂 Loaded scene from {:?}", path);
                        self.console_logs
                            .push(format!("✅ Scene loaded: {:?}", path));
                        self.undo_stack.clear();
                    }
                    Err(e) => {
                        self.status = format!("❌ Scene load failed: {}", e);
                        self.console_logs
                            .push(format!("❌ Failed to load scene: {}", e));
                    }
                }
            }

            // Ctrl+C: Copy selected entities
            if i.modifiers.ctrl && i.key_pressed(egui::Key::C) && !i.modifiers.shift {
                let world = self.edit_world();
                let selected = self.hierarchy_panel.get_all_selected();
                if !selected.is_empty() {
                    self.clipboard =
                        Some(clipboard::ClipboardData::from_entities(world, &selected));
                    self.status = format!("📋 Copied {} entities", selected.len());
                    self.console_logs.push(format!(
                        "📋 Copied {} entities to clipboard",
                        selected.len()
                    ));
                } else {
                    self.console_logs
                        .push("⚠️  No entities selected to copy".into());
                }
            }

            // Ctrl+V: Paste entities
            if i.modifiers.ctrl && i.key_pressed(egui::Key::V) {
                if let Some(clipboard) = self.clipboard.clone() {
                    let entity_count = clipboard.entities.len();
                    let offset = IVec2 { x: 1, y: 1 };
                    let result = self.with_world_and_undo_stack(|undo_stack, world| {
                        let cmd = command::SpawnEntitiesCommand::new(clipboard.clone(), offset);
                        undo_stack.execute(cmd, world)
                    });

                    match result {
                        Ok(()) => {
                            self.status = format!("📋 Pasted {} entities", entity_count);
                            self.console_logs
                                .push(format!("✅ Pasted {} entities", entity_count));
                        }
                        Err(e) => {
                            self.status = format!("❌ Paste failed: {}", e);
                            self.console_logs.push(format!("❌ Paste failed: {}", e));
                        }
                    }
                } else {
                    self.console_logs.push("⚠️  Clipboard is empty".into());
                }
            }

            // Ctrl+D: Duplicate selected entities
            if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
                let selected = self.hierarchy_panel.get_all_selected();
                if !selected.is_empty() {
                    let offset = IVec2 { x: 1, y: 1 };
                    let selected_clone = selected.clone();
                    let result = self.with_world_and_undo_stack(|undo_stack, world| {
                        let cmd =
                            command::DuplicateEntitiesCommand::new(selected_clone.clone(), offset);
                        undo_stack.execute(cmd, world)
                    });
                    match result {
                        Ok(()) => {
                            self.status = format!("📋 Duplicated {} entities", selected.len());
                            self.console_logs
                                .push(format!("✅ Duplicated {} entities", selected.len()));
                        }
                        Err(e) => {
                            self.status = format!("❌ Duplicate failed: {}", e);
                            self.console_logs
                                .push(format!("❌ Duplicate failed: {}", e));
                        }
                    }
                } else {
                    self.console_logs
                        .push("⚠️  No entities selected to duplicate".into());
                }
            }

            // F5: Play
            if i.key_pressed(egui::Key::F5) {
                if self.editor_mode.is_editing() {
                    let snapshot = {
                        let world = self.edit_world();
                        SceneData::from_world(world)
                    };
                    self.world_snapshot = Some(snapshot);
                    self.editor_mode = EditorMode::Play;
                    self.simulation_playing = true;
                    self.status = "▶️ Playing".into();
                    self.console_logs
                        .push("▶️ Entered Play mode (F6 to pause, F7 to stop)".into());
                }
            }

            // F6: Pause/Unpause
            if i.key_pressed(egui::Key::F6) {
                if self.editor_mode.is_playing() {
                    self.editor_mode = EditorMode::Paused;
                    self.simulation_playing = false;
                    self.status = "⏸️ Paused".into();
                    self.console_logs
                        .push("⏸️ Paused (F5 to resume, F7 to stop)".into());
                } else if self.editor_mode.is_paused() {
                    self.editor_mode = EditorMode::Play;
                    self.simulation_playing = true;
                    self.status = "▶️ Playing".into();
                    self.console_logs.push("▶️ Resumed playing".into());
                }
            }

            // F7: Stop (restore snapshot)
            if i.key_pressed(egui::Key::F7) {
                if !self.editor_mode.is_editing() {
                    if let Some(snapshot) = self.world_snapshot.take() {
                        self.scene_state = EditorSceneState::new(snapshot.to_world());
                        self.editor_mode = EditorMode::Edit;
                        self.simulation_playing = false;
                        self.status = "⏹️ Stopped (world restored)".into();
                        self.console_logs
                            .push("⏹️ Stopped play mode (world restored to snapshot)".into());
                    } else {
                        self.editor_mode = EditorMode::Edit;
                        self.simulation_playing = false;
                        self.status = "⏹️ Stopped".into();
                        self.console_logs.push("⏹️ Stopped play mode".into());
                    }
                }
            }

            // Delete: Delete selected entities
            if i.key_pressed(egui::Key::Delete) {
                if self.editor_mode.can_edit() {
                    let selected = self.hierarchy_panel.get_all_selected();
                    if !selected.is_empty() {
                        let selected_clone = selected.clone();
                        let result = self.with_world_and_undo_stack(|undo_stack, world| {
                            let cmd = command::DeleteEntitiesCommand::new(selected_clone.clone());
                            undo_stack.execute(cmd, world)
                        });
                        match result {
                            Ok(()) => {
                                self.hierarchy_panel.set_selected(None);
                                self.selected_entity = None;
                                self.status = format!("🗑️  Deleted {} entities", selected.len());
                                self.console_logs
                                    .push(format!("✅ Deleted {} entities", selected.len()));
                            }
                            Err(e) => {
                                self.status = format!("❌ Delete failed: {}", e);
                                self.console_logs.push(format!("❌ Delete failed: {}", e));
                            }
                        }
                    } else {
                        self.console_logs
                            .push("⚠️  No entities selected to delete".into());
                    }
                }
            }
        });

        // Phase 2.2: Autosave every 5 minutes
        if self.last_autosave.elapsed().as_secs() >= 300 {
            let world = self.edit_world();
            let autosave_dir = self.content_root.join(".autosave");
            let _ = fs::create_dir_all(&autosave_dir);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let autosave_path = autosave_dir.join(format!("autosave_{}.scene.ron", timestamp));

            match scene_serialization::save_scene(world, &autosave_path) {
                Ok(()) => {
                    self.console_logs
                        .push(format!("💾 Autosaved to {:?}", autosave_path));
                    self.last_autosave = std::time::Instant::now();
                }
                Err(e) => {
                    self.console_logs
                        .push(format!("⚠️  Autosave failed: {}", e));
                    self.last_autosave = std::time::Instant::now();
                }
            }
        }

        // Update Astract panels
        self.performance_panel.update();
        self.charts_panel.update();

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("AstraWeave Level & Encounter Editor");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    // Preserve viewport when creating new level (viewport requires CreationContext)
                    let viewport = self.viewport.take();
                    *self = Self::default();
                    self.viewport = viewport;
                    self.console_logs
                        .push("✅ New level created (reset to defaults)".into());
                    self.status = "New level created".into();
                }
                if ui.button("Open").clicked() {
                    // simple hardcoded example; integrate rfd/native dialog if desired
                    let p = self.content_root.join("levels/forest_breach.level.toml");
                    if let Ok(s) = fs::read_to_string(&p) {
                        match toml::from_str::<LevelDoc>(&s) {
                            Ok(ld) => {
                                self.level = ld;
                                self.status = format!("Opened {:?}", p);
                                self.console_logs.push(format!("✅ Opened level: {:?}", p));
                            }
                            Err(e) => {
                                self.status = format!("Open failed: {e}");
                                self.console_logs
                                    .push(format!("❌ Failed to open level: {}", e));
                            }
                        }
                    } else {
                        self.console_logs
                            .push(format!("❌ File not found: {:?}", p));
                        self.status = "File not found".into();
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
                                self.console_logs.push(format!("❌ Failed to save: {}", e));
                            } else {
                                // Signal hot-reload to the runtime
                                let _ = fs::create_dir_all(&self.content_root);
                                let _ = fs::write(
                                    self.content_root.join("reload.signal"),
                                    Uuid::new_v4().to_string(),
                                );
                                self.status = format!("Saved {:?}", p);
                                self.console_logs.push(format!("✅ Saved level: {:?}", p));
                            }
                        }
                        Err(e) => {
                            self.status = format!("Serialize failed: {e}");
                            self.console_logs
                                .push(format!("❌ Serialization failed: {}", e));
                        }
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
                                self.console_logs
                                    .push(format!("❌ Failed to save JSON: {}", e));
                            } else {
                                self.status = format!("Saved JSON {:?}", p);
                                self.console_logs.push(format!("✅ Saved JSON: {:?}", p));
                            }
                        }
                        Err(e) => {
                            self.status = format!("Serialize JSON failed: {e}");
                            self.console_logs
                                .push(format!("❌ JSON serialization failed: {}", e));
                        }
                    }
                }

                ui.separator();

                if ui.button("💾 Save Scene").clicked() {
                    let world = self.edit_world();
                    let path = if let Some(p) = &self.current_scene_path {
                        p.clone()
                    } else {
                        let dir = self.content_root.join("scenes");
                        let _ = fs::create_dir_all(&dir);
                        dir.join("untitled.scene.ron")
                    };

                    match scene_serialization::save_scene(world, &path) {
                        Ok(()) => {
                            self.current_scene_path = Some(path.clone());
                            self.recent_files.add_file(path.clone());
                            self.status = format!("💾 Saved scene to {:?}", path);
                            self.console_logs
                                .push(format!("✅ Scene saved: {:?}", path));
                            self.last_autosave = std::time::Instant::now();
                        }
                        Err(e) => {
                            self.status = format!("❌ Scene save failed: {}", e);
                            self.console_logs
                                .push(format!("❌ Failed to save scene: {}", e));
                        }
                    }
                }

                if ui.button("📂 Load Scene").clicked() {
                    let path = self.content_root.join("scenes/untitled.scene.ron");
                    match scene_serialization::load_scene(&path) {
                        Ok(world) => {
                            self.scene_state = EditorSceneState::new(world);
                            self.current_scene_path = Some(path.clone());
                            self.recent_files.add_file(path.clone());
                            self.status = format!("📂 Loaded scene from {:?}", path);
                            self.console_logs
                                .push(format!("✅ Scene loaded: {:?}", path));
                            self.undo_stack.clear();
                        }
                        Err(e) => {
                            self.status = format!("❌ Scene load failed: {}", e);
                            self.console_logs
                                .push(format!("❌ Failed to load scene: {}", e));
                        }
                    }
                }

                ui.menu_button("📚 Recent Files", |ui| {
                    let recent_files = self.recent_files.get_files().to_vec();

                    if recent_files.is_empty() {
                        ui.label("No recent files");
                    } else {
                        for path in recent_files {
                            let file_name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown");

                            if ui.button(file_name).clicked() {
                                match scene_serialization::load_scene(&path) {
                                    Ok(world) => {
                                        self.scene_state = EditorSceneState::new(world);
                                        self.current_scene_path = Some(path.clone());
                                        self.recent_files.add_file(path.clone());
                                        self.status = format!("📂 Loaded scene from {:?}", path);
                                        self.console_logs
                                            .push(format!("✅ Scene loaded: {:?}", path));
                                        self.undo_stack.clear();
                                        ui.close();
                                    }
                                    Err(e) => {
                                        self.status = format!("❌ Scene load failed: {}", e);
                                        self.console_logs
                                            .push(format!("❌ Failed to load scene: {}", e));
                                    }
                                }
                            }
                        }

                        ui.separator();

                        if ui.button("🗑️ Clear Recent Files").clicked() {
                            self.recent_files.clear();
                            ui.close();
                        }
                    }
                });

                ui.separator();

                // Phase 4: Play-in-Editor controls
                ui.horizontal(|ui| {
                    ui.label("Play:");

                    let play_enabled =
                        self.editor_mode.is_editing() || self.editor_mode.is_paused();
                    if ui
                        .add_enabled(play_enabled, egui::Button::new("▶️ Play (F5)"))
                        .clicked()
                    {
                        let scene_data = {
                            let world = self.edit_world();
                            SceneData::from_world(world)
                        };
                        if self.editor_mode.is_editing() {
                            self.world_snapshot = Some(scene_data.clone());
                        }
                        self.runtime_world = Some(scene_data.to_world());
                        self.editor_mode = EditorMode::Play;
                        self.simulation_playing = true;
                        self.status = "▶️ Playing".into();
                        self.console_logs.push("▶️ Entered Play mode".into());
                    }

                    let pause_enabled = !self.editor_mode.is_editing();
                    if ui
                        .add_enabled(pause_enabled, egui::Button::new("⏸️ Pause (F6)"))
                        .clicked()
                    {
                        if self.editor_mode.is_playing() {
                            self.editor_mode = EditorMode::Paused;
                            self.simulation_playing = false;
                            self.status = "⏸️ Paused".into();
                            self.console_logs.push("⏸️ Paused".into());
                        }
                    }

                    let stop_enabled = !self.editor_mode.is_editing();
                    if ui
                        .add_enabled(stop_enabled, egui::Button::new("⏹️ Stop (F7)"))
                        .clicked()
                    {
                        if let Some(snapshot) = self.world_snapshot.take() {
                            self.scene_state = EditorSceneState::new(snapshot.to_world());
                            self.status = "⏹️ Stopped (world restored)".into();
                            self.console_logs.push("⏹️ Stopped (world restored)".into());
                        } else {
                            self.status = "⏹️ Stopped".into();
                        }
                        self.editor_mode = EditorMode::Edit;
                        self.simulation_playing = false;
                        self.runtime_world = None;
                    }

                    ui.separator();

                    // Status indicator with color
                    let status_label = egui::RichText::new(self.editor_mode.status_text())
                        .color(self.editor_mode.status_color());
                    ui.label(status_label);

                    // Show simulation info when playing
                    if self.editor_mode.is_playing() {
                        if let Some(world) = &self.runtime_world {
                            ui.label(format!(
                                "| {} entities, tick {}, {:.1}s",
                                world.entities().len(),
                                self.sim_tick_count,
                                world.t
                            ));
                        }
                    }
                });
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

        // LEFT PANEL - Astract World & Entity panels
        egui::SidePanel::left("astract_left_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("🎨 Astract Panels");
                ui.separator();

                // Add ScrollArea to handle expanded menus
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.collapsing("🌍 World", |ui| {
                            self.world_panel.show(ui);
                        });

                        ui.add_space(10.0);

                        ui.collapsing("📦 Assets", |ui| {
                            self.asset_browser.show(ui);
                        });

                        ui.add_space(10.0);

                        let mut hierarchy_panel = std::mem::take(&mut self.hierarchy_panel);

                        ui.collapsing("🌲 Hierarchy", |ui| {
                            {
                                let world = self.edit_world_mut();
                                hierarchy_panel.sync_with_world(world);
                            }

                            let selected = {
                                let world = self.edit_world_mut();
                                hierarchy_panel.show_with_world(ui, world)
                            };

                            if let Some(selected) = selected {
                                self.selected_entity = Some(selected as u64);
                            }
                        });

                        let all_selected = hierarchy_panel.get_all_selected();
                        self.selection_set.clear();
                        for &entity_id in &all_selected {
                            self.selection_set.add(entity_id as u64, false);
                        }
                        if let Some(primary) = all_selected.last() {
                            self.selection_set.primary = Some(*primary as u64);
                        }

                        self.hierarchy_panel = hierarchy_panel;

                        ui.add_space(10.0);

                        let mut entity_panel = std::mem::take(&mut self.entity_panel);

                        ui.collapsing("🎮 Entities", |ui| {
                            use panels::entity_panel::PrefabAction;

                            let selected_u32 = self.selected_entity.map(|e| e as u32);
                            let prefab_info = {
                                let world = self.scene_state.world();
                                selected_u32.and_then(|entity| {
                                    self.lock_prefab_manager()
                                        .ok()
                                        .and_then(|mut mgr| mgr.describe_instance(entity, world))
                                })
                            };

                            let panel_result = {
                                let world = self.edit_world_mut();
                                entity_panel.show_with_world(
                                    ui,
                                    world,
                                    selected_u32,
                                    prefab_info.as_ref(),
                                )
                            };

                            if let Some(action) = panel_result.prefab_action {
                                match action {
                                    PrefabAction::Apply { entity } => {
                                        let prefab_path = self
                                            .lock_prefab_manager()
                                            .ok()
                                            .and_then(|mgr| mgr.instance_path(entity));

                                        match prefab_path {
                                            Some(prefab_path) => match PrefabData::load_from_file(&prefab_path) {
                                                Ok(prev_data) => {
                                                    let mut undo_stack =
                                                        std::mem::take(&mut self.undo_stack);
                                                    let handle = self.prefab_manager.clone();
                                                    let cmd = PrefabApplyOverridesCommand::new(
                                                        handle,
                                                        entity,
                                                        prefab_path.clone(),
                                                        prev_data,
                                                    );
                                                    let exec_result = {
                                                        let world = self.scene_state.world_mut();
                                                        undo_stack.execute(cmd, world)
                                                    };

                                                    if let Err(err) = exec_result {
                                                        self.console_logs.push(format!(
                                                            "Failed to apply prefab overrides: {err}"
                                                        ));
                                                    } else {
                                                        self.status =
                                                            "Prefab saved with overrides".into();
                                                    }
                                                    self.undo_stack = undo_stack;
                                                }
                                                Err(err) => {
                                                    self.console_logs.push(format!(
                                                        "Failed to snapshot prefab before apply: {err}"
                                                    ));
                                                }
                                            },
                                            None => self
                                                .console_logs
                                                .push("No prefab path found for selection".into()),
                                        }
                                    }
                                    PrefabAction::Revert { entity } => {
                                        let snapshot = self
                                            .lock_prefab_manager()
                                            .ok()
                                            .and_then(|mgr| {
                                                let world = self.scene_state.world();
                                                mgr.capture_snapshot(world, entity)
                                            });

                                        match snapshot {
                                            Some(snapshot) => {
                                                let mut undo_stack =
                                                    std::mem::take(&mut self.undo_stack);
                                                let handle = self.prefab_manager.clone();
                                                let cmd = PrefabRevertOverridesCommand::new(
                                                    handle,
                                                    entity,
                                                    snapshot,
                                                );
                                                let exec_result = {
                                                    let world = self.scene_state.world_mut();
                                                    undo_stack.execute(cmd, world)
                                                };

                                                if let Err(err) = exec_result {
                                                    self.console_logs.push(format!(
                                                        "Failed to revert prefab overrides: {err}"
                                                    ));
                                                } else {
                                                    self.scene_state.sync_all();
                                                    self.status =
                                                        "Overrides reverted from prefab".into();
                                                }
                                                self.undo_stack = undo_stack;
                                            }
                                            None => self
                                                .console_logs
                                                .push("Failed to capture prefab snapshot".into()),
                                        }
                                    }
                                }
                            }

                            if let Some(component_edit) = panel_result.component_edit {
                                use crate::component_ui::ComponentEdit;

                                let (entity, cmd): (Entity, Box<dyn crate::command::EditorCommand>) =
                                    match component_edit {
                                        ComponentEdit::Health {
                                            entity,
                                            old_hp,
                                            new_hp,
                                        } => (
                                            entity,
                                            EditHealthCommand::new(entity, old_hp, new_hp),
                                        ),
                                        ComponentEdit::Team {
                                            entity,
                                            old_id,
                                            new_id,
                                        } => (
                                            entity,
                                            EditTeamCommand::new(
                                                entity,
                                                Team { id: old_id },
                                                Team { id: new_id },
                                            ),
                                        ),
                                        ComponentEdit::Ammo {
                                            entity,
                                            old_rounds,
                                            new_rounds,
                                        } => (
                                            entity,
                                            EditAmmoCommand::new(
                                                entity,
                                                old_rounds,
                                                new_rounds,
                                            ),
                                        ),
                                    };

                                self.undo_stack.push_executed(cmd);
                                self.notify_prefab_override(entity);
                            }
                        });

                        self.entity_panel = entity_panel;

                        ui.add_space(10.0);

                        ui.collapsing("🔧 Transform", |ui| {
                            // Sync selected entity to transform panel
                            if let Some(selected_id) = self.selected_entity {
                                let entity = selected_id as u32;
                                if let Some(transform) = self.scene_state.transform_for(entity) {
                                    self.transform_panel.set_selected(transform);
                                }
                            } else {
                                self.transform_panel.clear_selection();
                            }

                            self.transform_panel.show(ui);

                            // Apply changes back to entity if transform was modified
                            if let Some(selected_id) = self.selected_entity {
                                if let Some(new_transform) = self.transform_panel.get_transform() {
                                    self.scene_state
                                        .apply_transform(selected_id as u32, &new_transform);
                                }
                            }
                        });

                        ui.add_space(10.0);

                        ui.collapsing("📊 Charts", |ui| {
                            self.charts_panel.show(ui);
                        });

                        ui.add_space(10.0);

                        ui.collapsing("🎨 Advanced Widgets", |ui| {
                            self.advanced_widgets_panel.show(ui);
                        });

                        ui.add_space(10.0);

                        ui.collapsing("🕸️ Graph Visualization", |ui| {
                            self.graph_panel.show(ui);
                        });

                        ui.add_space(10.0);

                        ui.collapsing("🎬 Animation", |_ui| {
                            self.animation_panel.show(ctx);
                        });
                    });
            });

        // RIGHT PANEL - Astract Performance panel
        egui::SidePanel::right("astract_right_panel")
            .default_width(350.0)
            .show(ctx, |ui| {
                self.performance_panel.show(ui);
            });

        // BOTTOM PANEL - StatusBar (Phase 3.5 & 4)
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(24.0)
            .show(ctx, |ui| {
                let snap_config = self.scene_state.snapping_config();
                StatusBar::show(
                    ui,
                    &self.editor_mode,
                    &self.current_gizmo_mode,
                    &self.selection_set,
                    &self.undo_stack,
                    &snap_config,
                    self.current_fps,
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // 3D Viewport (Phase 1.1 - Babylon.js-style editor)
            if let Some(mut viewport) = self.viewport.take() {
                // Phase 4: Visual indicator for play mode
                let viewport_frame = if !self.editor_mode.is_editing() {
                    let border_color = if self.editor_mode.is_playing() {
                        egui::Color32::from_rgb(100, 200, 100)
                    } else {
                        egui::Color32::from_rgb(255, 180, 50)
                    };

                    egui::Frame::NONE
                        .stroke(egui::Stroke::new(3.0, border_color))
                        .inner_margin(4.0)
                } else {
                    egui::Frame::NONE
                };

                viewport_frame.show(ui, |ui| {
                    ui.heading("🎮 3D Viewport");
                    ui.label(
                        "Phase 1.1 Complete: Grid rendering active, texture display in progress",
                    );

                    let mut snap_config = self.scene_state.snapping_config();
                    let mut snap_changed = false;
                    ui.horizontal(|ui| {
                        ui.label("⚡ Snapping:");

                        if ui.checkbox(&mut snap_config.grid_enabled, "Grid").changed() {
                            snap_changed = true;
                        }

                        ui.label("Size:");
                        let mut grid_size_idx = match snap_config.grid_size {
                            s if (s - 0.5).abs() < 0.01 => 0,
                            s if (s - 1.0).abs() < 0.01 => 1,
                            s if (s - 2.0).abs() < 0.01 => 2,
                            _ => 1,
                        };

                        if ui
                            .add(
                                egui::Slider::new(&mut grid_size_idx, 0..=2)
                                    .show_value(false)
                                    .custom_formatter(|n, _| match n as usize {
                                        0 => "0.5".to_string(),
                                        1 => "1.0".to_string(),
                                        2 => "2.0".to_string(),
                                        _ => "1.0".to_string(),
                                    }),
                            )
                            .changed()
                        {
                            snap_config.grid_size = match grid_size_idx {
                                0 => 0.5,
                                1 => 1.0,
                                2 => 2.0,
                                _ => 1.0,
                            };
                            snap_changed = true;
                        }

                        ui.separator();
                        if ui
                            .checkbox(&mut snap_config.angle_enabled, "Angle")
                            .changed()
                        {
                            snap_changed = true;
                        }
                        ui.label(format!("{}°", snap_config.angle_increment));
                    });

                    if snap_changed {
                        self.set_snapping_config(snap_config);
                    }

                    ui.separator();

                    // Render viewport (takes 70% width, full available height)
                    // Use edit scene while authoring; switch to runtime world when simulating
                    let mut undo_stack = std::mem::take(&mut self.undo_stack);
                    let snap_config = self.scene_state.snapping_config();
                    let viewport_result = if self.editor_mode.is_editing() {
                        let world = self.edit_world_mut();
                        viewport.ui(ui, world, &mut undo_stack, snap_config)
                    } else if let Some(world) = self.runtime_world_mut() {
                        viewport.ui(ui, world, &mut undo_stack, snap_config)
                    } else {
                        let world = self.edit_world_mut();
                        viewport.ui(ui, world, &mut undo_stack, snap_config)
                    };

                    if let Err(e) = viewport_result {
                        self.console_logs.push(format!("❌ Viewport error: {}", e));
                        eprintln!("❌ Viewport error: {}", e);
                    }

                    self.undo_stack = undo_stack;

                    // Sync selected entity from viewport to app state
                    if let Some(selected) = viewport.selected_entity() {
                        self.selected_entity = Some(selected as u64);
                    }

                    // Sync snapping settings from viewport toolbar to shared hub
                    let toolbar = viewport.toolbar();
                    let toolbar_config = SnappingConfig {
                        grid_size: toolbar.snap_size,
                        angle_increment: toolbar.angle_snap_degrees,
                        grid_enabled: toolbar.snap_enabled,
                        angle_enabled: toolbar.angle_snap_enabled,
                    };
                    self.set_snapping_config(toolbar_config);

                    let pointer_released = ui
                        .ctx()
                        .input(|i| i.pointer.button_released(egui::PointerButton::Primary));

                    if pointer_released && self.asset_browser.is_dragging_prefab() {
                        let pointer_pos = ui.ctx().input(|i| i.pointer.latest_pos());
                        match pointer_pos {
                            Some(pos) => {
                                match viewport.world_pos_from_pointer(pos) {
                                    Some(world_point) => {
                                        if self.editor_mode.is_editing() {
                                            if let Some(prefab_path) =
                                                self.asset_browser.take_dragged_prefab()
                                            {
                                                self.handle_prefab_drop(
                                                    prefab_path,
                                                    world_point,
                                                    &mut viewport,
                                                );
                                                self.asset_browser.cancel_prefab_drag();
                                            }
                                        } else {
                                            self.console_logs.push(
                                                "Prefab drops are disabled while the simulation is running.".into(),
                                            );
                                            self.asset_browser.cancel_prefab_drag();
                                        }
                                    }
                                    None => {
                                        self.console_logs.push(
                                            "Ignored prefab drop outside of the viewport bounds.".into(),
                                        );
                                        self.asset_browser.cancel_prefab_drag();
                                    }
                                }
                            }
                            None => {
                                self.console_logs
                                    .push("No pointer data available for prefab drop.".into());
                                self.asset_browser.cancel_prefab_drag();
                            }
                        }
                    } else {
                        // No prefab drag active - make sure stale drags are cleared when pointer stays idle
                        if pointer_released {
                            self.asset_browser.cancel_prefab_drag();
                        }
                    }

                    self.process_viewport_events(&mut viewport);

                    ui.add_space(10.0);
                });

                ui.separator();

                self.viewport = Some(viewport);
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Auto-expand Console when simulation is running (so users see feedback)
                let console_open = self.simulation_playing || !self.console_logs.is_empty();

                ui.collapsing("Scene Hierarchy", |ui| self.show_scene_hierarchy(ui));
                ui.collapsing("Inspector", |ui| self.show_inspector(ui));

                // Console section with auto-expand when active
                egui::CollapsingHeader::new("Console")
                    .default_open(console_open)
                    .show(ui, |ui| self.show_console(ui));

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
            // Initialize runtime world if Play was triggered via keyboard shortcut without UI snapshot
            if self.runtime_world.is_none() {
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
                self.runtime_world = Some(world);
                self.sim_tick_count = 0;
                self.console_logs
                    .push("Simulation started with entities from level.".into());
            }
            // Tick simulation
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(self.last_sim_tick);
            const TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

            if elapsed >= TICK_INTERVAL {
                let ticks = (elapsed.as_millis() / 100) as u64;
                for _ in 0..ticks {
                    if let Some(world) = self.runtime_world.as_mut() {
                        world.tick(0.1); // dt = 0.1s
                                         // Simple behavior: regenerate health
                        for entity in world.entities() {
                            if let Some(health) = world.health_mut(entity) {
                                health.hp = (health.hp + 1).min(100);
                            }
                        }
                    }

                    self.sim_tick_count += 1;

                    if self.sim_tick_count % 10 == 0 {
                        if let Some(world) = self.runtime_world.as_ref() {
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
            if self.runtime_world.is_some() {
                self.runtime_world = None;
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
        Box::new(|cc| {
            // Use EditorApp::new() to initialize viewport with CreationContext
            match EditorApp::new(cc) {
                Ok(app) => Ok(Box::new(app) as Box<dyn eframe::App>),
                Err(e) => Err(format!("{:?}", e).into()),
            }
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e))?;
    Ok(())
}
