#![allow(dead_code)]

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
mod interaction; // Phase 8.1 Week 5 Day 3 - Gizmo interaction helpers (auto-tracking)
mod material_inspector;
mod panels;
mod prefab; // Phase 4.1 - Prefab System
mod recent_files; // Phase 3 - Recent files tracking
mod runtime; // Week 4 - Deterministic runtime integration
mod scene_serialization; // Phase 2.2 - Scene Save/Load
mod scene_state; // Week 1 - Canonical edit-mode world owner
mod ui; // Phase 3 - UI components (StatusBar, etc.)
mod viewport; // Phase 1.1 - 3D Viewport
              // mod voxel_tools;  // Temporarily disabled - missing astraweave-terrain dependency

use anyhow::Result;
use astraweave_asset::AssetDatabase;
use astraweave_core::{Entity, IVec2, Team, World};
use astraweave_dialogue::DialogueGraph;
use astraweave_nav::NavMesh;
use astraweave_quests::Quest;
use behavior_graph::{BehaviorGraphDocument, BehaviorGraphEditorUi};
use editor_mode::EditorMode;
use eframe::egui;
use entity_manager::MaterialSlot;
use entity_manager::{EntityManager, SelectionSet};
use gizmo::snapping::SnappingConfig;
use gizmo::state::GizmoMode;
use material_inspector::MaterialInspector;
use panels::{
    AdvancedWidgetsPanel, AnimationPanel, AssetAction, AssetBrowser, BuildManagerPanel,
    ChartsPanel, EntityPanel, GraphPanel, HierarchyPanel, Panel, PerformancePanel, PrefabAction,
    TextureType, ThemeManagerPanel, TransformPanel, WorldPanel,
};
mod plugin;
use prefab::PrefabManager;
use recent_files::RecentFilesManager;
use runtime::{EditorRuntime, RuntimeState};
use scene_state::{EditorSceneState, TransformableScene};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tracing::{debug, error, info, span, warn, Level};
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

#[derive(Clone, Debug)]
struct BehaviorGraphBinding {
    entity: Entity,
    name: String,
}

impl BehaviorGraphBinding {
    fn new(entity: Entity, name: impl Into<String>) -> Self {
        Self {
            entity,
            name: name.into(),
        }
    }
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
    behavior_graph_doc: BehaviorGraphDocument,
    behavior_graph_ui: BehaviorGraphEditorUi,
    behavior_graph_binding: Option<BehaviorGraphBinding>,
    dialogue_graph: DialogueGraph,
    quest_graph: Quest,
    console_logs: Vec<String>,
    profiler_data: Vec<String>,
    last_runtime_log: std::time::Instant,
    runtime: EditorRuntime,
    terrain_grid: Vec<Vec<String>>,
    selected_biome: String,
    nav_mesh: NavMesh,
    nav_max_step: f32,
    nav_max_slope_deg: f32,
    scene_state: Option<EditorSceneState>,
    material_inspector: MaterialInspector, // NEW - Phase PBR-G Task 2
    // Phase 1: Entity management
    entity_manager: EntityManager,
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
    // Phase 5.2: Build Manager
    build_manager_panel: BuildManagerPanel,
    // Phase 5.3: Plugin System
    plugin_manager: plugin::PluginManager,
    plugin_panel: plugin::PluginManagerPanel,
    // Phase 5.5: Theme & Layout Manager
    theme_manager: ThemeManagerPanel,
    // 3D Viewport (Phase 1.1 - Babylon.js-style editor)
    viewport: Option<ViewportWidget>,
    // Phase 3.5: StatusBar tracking
    current_gizmo_mode: GizmoMode,
    selection_set: SelectionSet,
    snapping_config: SnappingConfig,
    last_frame_time: std::time::Instant,
    current_fps: f32,
    recent_files: RecentFilesManager,
    // Phase 4.1: Prefab System
    prefab_manager: PrefabManager,
    // Phase 4.2: Play-in-Editor
    editor_mode: EditorMode,
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
            behavior_graph_doc: BehaviorGraphDocument::new_default(),
            behavior_graph_ui: BehaviorGraphEditorUi::default(),
            behavior_graph_binding: None,
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
            last_runtime_log: std::time::Instant::now(),
            runtime: EditorRuntime::new(),
            terrain_grid: vec![vec!["grass".into(); 10]; 10],
            selected_biome: "grass".into(),
            nav_mesh: NavMesh::bake(&[], 0.4, 60.0),
            nav_max_step: 0.4,
            nav_max_slope_deg: 60.0,
            scene_state: Some(EditorSceneState::new(Self::create_default_world())), // Initialize with sample entities
            material_inspector: MaterialInspector::new(), // NEW - Phase PBR-G Task 2
            // Phase 1: Entity management
            entity_manager: EntityManager::new(),
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
            // Phase 5.2: Build Manager
            build_manager_panel: BuildManagerPanel::new(),
            // Phase 5.3: Plugin System
            plugin_manager: plugin::PluginManager::default(),
            plugin_panel: plugin::PluginManagerPanel::default(),
            // Phase 5.5: Theme & Layout Manager
            theme_manager: ThemeManagerPanel::new(),
            // Viewport initialized in new() method (requires CreationContext)
            viewport: None,
            // Phase 3.5: StatusBar state
            current_gizmo_mode: GizmoMode::Inactive,
            selection_set: SelectionSet::new(),
            snapping_config: SnappingConfig::default(),
            last_frame_time: std::time::Instant::now(),
            current_fps: 60.0,
            recent_files: RecentFilesManager::load(),
            // Phase 4.1: Prefab System
            prefab_manager: PrefabManager::new("prefabs"),
            // Phase 4.2: Play-in-Editor
            editor_mode: EditorMode::default(),
        }
    }
}

impl EditorApp {
    fn edit_world(&self) -> Option<&World> {
        self.scene_state.as_ref().map(|state| state.world())
    }

    fn edit_world_mut(&mut self) -> Option<&mut World> {
        self.scene_state.as_mut().map(|state| state.world_mut())
    }

    fn active_world(&self) -> Option<&World> {
        if self.runtime.state() == RuntimeState::Editing {
            self.edit_world()
        } else {
            self.runtime.sim_world()
        }
    }

    /// Initialize sample entities for viewport testing
    fn init_sample_entities(entity_manager: &mut EntityManager) {
        use glam::{Quat, Vec3};

        // Create a few test entities
        let cube1 = entity_manager.create("Cube_1".to_string());
        entity_manager.update_transform(cube1, Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE);

        let cube2 = entity_manager.create("Cube_2".to_string());
        entity_manager.update_transform(cube2, Vec3::new(3.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE);

        let cube3 = entity_manager.create("Cube_3".to_string());
        entity_manager.update_transform(cube3, Vec3::new(0.0, 0.0, 3.0), Quat::IDENTITY, Vec3::ONE);

        let sphere = entity_manager.create("Sphere_1".to_string());
        entity_manager.update_transform(
            sphere,
            Vec3::new(-3.0, 1.0, 0.0),
            Quat::IDENTITY,
            Vec3::splat(1.5),
        );
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

    fn request_play(&mut self) {
        let _span = span!(Level::INFO, "request_play", mode = ?self.editor_mode).entered();

        if self.editor_mode.is_editing() {
            if let Some(scene_state) = self.scene_state.as_ref() {
                match self.runtime.enter_play(scene_state.world()) {
                    Ok(()) => {
                        self.editor_mode = EditorMode::Play;
                        self.status = "▶️ Playing".into();
                        info!("Entered Play mode - snapshot captured");
                        self.console_logs
                            .push("▶️ Entered Play mode (F6 to pause, F7 to stop)".into());
                    }
                    Err(e) => {
                        error!("Failed to enter play mode: {}", e);
                        self.console_logs
                            .push(format!("❌ Failed to enter play mode: {}", e));
                        self.status = "❌ Failed to enter play".into();
                    }
                }
            } else {
                warn!("No world loaded - cannot enter play mode");
                self.console_logs
                    .push("⚠️  No world loaded – cannot enter play mode".into());
            }
        } else if self.editor_mode.is_paused() {
            self.runtime.resume();
            self.editor_mode = EditorMode::Play;
            self.status = "▶️ Playing".into();
            info!("Resumed playing from pause");
            self.console_logs.push("▶️ Resumed playing".into());
        }
    }

    fn request_pause(&mut self) {
        let _span = span!(Level::INFO, "request_pause").entered();

        if self.editor_mode.is_playing() {
            self.runtime.pause();
            self.editor_mode = EditorMode::Paused;
            self.status = "⏸️ Paused".into();
            info!(
                "Paused simulation at tick {}",
                self.runtime.stats().tick_count
            );
            self.console_logs
                .push("⏸️ Paused (F5 to resume, F7 to stop)".into());
        }
    }

    fn request_stop(&mut self) {
        let _span = span!(Level::INFO, "request_stop").entered();

        if !self.editor_mode.is_editing() {
            let final_tick = self.runtime.stats().tick_count;
            match self.runtime.exit_play() {
                Ok(restored) => {
                    if let Some(world) = restored {
                        self.scene_state = Some(EditorSceneState::new(world));
                    }
                    self.editor_mode = EditorMode::Edit;
                    self.status = "⏹️ Stopped (world restored)".into();
                    info!(
                        "Stopped simulation after {} ticks - snapshot restored",
                        final_tick
                    );
                    self.console_logs
                        .push("⏹️ Stopped play mode (world restored to snapshot)".into());
                    self.performance_panel.clear_runtime_stats();
                }
                Err(e) => {
                    error!("Failed to stop play mode: {}", e);
                    self.console_logs
                        .push(format!("❌ Failed to stop play mode: {}", e));
                    self.status = "❌ Failed to stop".into();
                }
            }
        }
    }

    fn request_step(&mut self) {
        let _span = span!(Level::DEBUG, "request_step").entered();

        if !self.editor_mode.is_editing() {
            if let Err(e) = self.runtime.step_frame() {
                error!("Step frame failed: {}", e);
                self.console_logs.push(format!("❌ Step failed: {}", e));
            } else {
                self.editor_mode = EditorMode::Paused;
                self.status = "⏭️ Stepped one frame".into();
                debug!(
                    "Stepped one frame to tick {}",
                    self.runtime.stats().tick_count
                );
                self.console_logs.push("⏭️ Advanced one frame".into());
            }
        }
    }

    fn show_play_controls(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let (mode_text, color) = match self.editor_mode {
                        EditorMode::Edit => ("🛠️ Edit", egui::Color32::LIGHT_GRAY),
                        EditorMode::Play => ("▶️ Playing", egui::Color32::from_rgb(80, 200, 120)),
                        EditorMode::Paused => ("⏸️ Paused", egui::Color32::from_rgb(255, 180, 50)),
                    };
                    ui.colored_label(color, mode_text);

                    ui.separator();

                    let play_enabled =
                        self.editor_mode.is_editing() || self.editor_mode.is_paused();
                    if ui
                        .add_enabled(play_enabled, egui::Button::new("▶️ Play (F5)"))
                        .clicked()
                    {
                        self.request_play();
                    }

                    let pause_enabled = self.editor_mode.is_playing();
                    if ui
                        .add_enabled(pause_enabled, egui::Button::new("⏸️ Pause (F6)"))
                        .clicked()
                    {
                        self.request_pause();
                    }

                    let stop_enabled = !self.editor_mode.is_editing();
                    if ui
                        .add_enabled(stop_enabled, egui::Button::new("⏹️ Stop (F7)"))
                        .clicked()
                    {
                        self.request_stop();
                    }

                    let step_enabled = self.editor_mode.is_paused();
                    if ui
                        .add_enabled(step_enabled, egui::Button::new("⏭️ Step (F8)"))
                        .clicked()
                    {
                        self.request_step();
                    }
                });

                ui.add_space(4.0);

                let stats = self.runtime.stats();
                ui.horizontal(|ui| {
                    if self.editor_mode.is_editing() {
                        ui.label("Not running – press ▶️ Play to preview the level");
                    } else {
                        ui.label(format!("Tick #{}", stats.tick_count));
                        ui.separator();
                        ui.label(format!("Entities: {}", stats.entity_count));
                        ui.separator();
                        ui.label(format!("{:.2} ms", stats.frame_time_ms));
                        ui.separator();
                        ui.label(format!("{:.0} FPS", stats.fps));
                    }
                });
            });
        });
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

        // Initialize sample entities for testing
        Self::init_sample_entities(&mut app.entity_manager);

        // Initialize viewport (requires wgpu render state from CreationContext)
        match ViewportWidget::new(cc) {
            Ok(viewport) => {
                app.viewport = Some(viewport);
                app.console_logs.push("✅ 3D Viewport initialized".into());
            }
            Err(e) => {
                app.console_logs
                    .push(format!("⚠️  Viewport init failed: {}", e));
                warn!("❌ Viewport initialization failed: {}", e);
                // Continue without viewport (fallback to 2D mode)
            }
        }

        Ok(app)
    }
}

impl EditorApp {
    fn show_scene_hierarchy(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scene Hierarchy");

        // Collect entity data first to avoid borrow issues
        let entity_data: Vec<(Entity, String, Option<_>, Option<_>, Option<_>)> =
            if let Some(world) = self.active_world() {
                world
                    .entities()
                    .iter()
                    .map(|&entity| {
                        let name = world
                            .name(entity)
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("Entity_{}", entity));
                        let pose = world.pose(entity);
                        let health = world.health(entity);
                        let team = world.team(entity);
                        (entity, name, pose, health, team)
                    })
                    .collect()
            } else {
                Vec::new()
            };

        if entity_data.is_empty() {
            if self.active_world().is_none() {
                ui.label("No scene loaded");
            } else {
                ui.label("No entities in scene");
            }
        } else {
            ui.label(format!("{} entities:", entity_data.len()));
            ui.separator();

            let mut new_selection: Option<u64> = None;
            let current_primary = self.selection_set.primary;

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (entity, name, pose, health, team) in &entity_data {
                        let is_selected = current_primary == Some(*entity as u64);

                        let response = ui.selectable_label(is_selected, format!("📦 {}", name));

                        if response.clicked() {
                            new_selection = Some(*entity as u64);
                        }

                        // Show entity info on hover
                        response.on_hover_ui(|ui| {
                            ui.label(format!("ID: {}", entity));
                            if let Some(pose) = pose {
                                ui.label(format!("Position: ({}, {})", pose.pos.x, pose.pos.y));
                                ui.label(format!("Scale: {:.2}", pose.scale));
                            }
                            if let Some(health) = health {
                                ui.label(format!("Health: {}", health.hp));
                            }
                            if let Some(team) = team {
                                ui.label(format!("Team: {}", team.id));
                            }
                        });
                    }
                });

            // Apply selection change after the scroll area
            if let Some(sel) = new_selection {
                self.selection_set.primary = Some(sel);
            }
        }
    }

    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");

        // Show selected entity's components
        if let Some(entity_id) = self.selection_set.primary {
            if let Ok(entity) = u32::try_from(entity_id) {
                if let Some(world) = self.active_world() {
                    let name = world
                        .name(entity)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("Entity_{}", entity));

                    ui.label(format!("Selected: {} (ID: {})", name, entity));
                    ui.separator();

                    // Transform section
                    ui.collapsing("Transform", |ui| {
                        if let Some(pose) = world.pose(entity) {
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui.label(format!("({}, {})", pose.pos.x, pose.pos.y));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Rotation:");
                                ui.label(format!("{:.1}°", pose.rotation.to_degrees()));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Scale:");
                                ui.label(format!("{:.2}", pose.scale));
                            });
                        } else {
                            ui.label("No transform component");
                        }
                    });

                    // Health section
                    ui.collapsing("Health", |ui| {
                        if let Some(health) = world.health(entity) {
                            ui.horizontal(|ui| {
                                ui.label("HP:");
                                let hp_color = if health.hp > 50 {
                                    egui::Color32::GREEN
                                } else if health.hp > 20 {
                                    egui::Color32::YELLOW
                                } else {
                                    egui::Color32::RED
                                };
                                ui.colored_label(hp_color, format!("{}", health.hp));
                            });
                        } else {
                            ui.label("No health component");
                        }
                    });

                    // Team section
                    ui.collapsing("Team", |ui| {
                        if let Some(team) = world.team(entity) {
                            ui.horizontal(|ui| {
                                ui.label("Team ID:");
                                ui.label(format!("{}", team.id));
                            });
                        } else {
                            ui.label("No team component");
                        }
                    });

                    // Ammo section
                    ui.collapsing("Ammo", |ui| {
                        if let Some(ammo) = world.ammo(entity) {
                            ui.horizontal(|ui| {
                                ui.label("Rounds:");
                                ui.label(format!("{}", ammo.rounds));
                            });
                        } else {
                            ui.label("No ammo component");
                        }
                    });
                } else {
                    ui.label("No scene loaded");
                }
            } else {
                ui.label("Invalid entity ID");
            }
        } else {
            ui.label("No entity selected");
            ui.label("Select an entity in the Scene Hierarchy or viewport");
        }
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
        if self.profiler_data.is_empty() {
            ui.label("No runtime telemetry yet – press ▶️ Play to sample frame data.");
        } else {
            egui::ScrollArea::vertical()
                .max_height(160.0)
                .show(ui, |ui| {
                    for data in self.profiler_data.iter().rev() {
                        ui.label(data);
                    }
                });
        }
    }

    fn selected_entity_handle(&self) -> Option<Entity> {
        self.selection_set
            .primary
            .and_then(|id| u32::try_from(id).ok())
    }

    fn resolve_entity_label(&self, entity: Entity) -> String {
        self.scene_state
            .as_ref()
            .and_then(|state| state.world().name(entity).map(|s| s.to_string()))
            .unwrap_or_else(|| format!("Entity_{}", entity))
    }

    fn load_behavior_graph_from_selection(&mut self) {
        let Some(entity) = self.selected_entity_handle() else {
            self.console_logs
                .push("⚠️ Select an entity before loading its behavior graph.".into());
            return;
        };

        let Some(scene_state) = self.scene_state.as_ref() else {
            self.console_logs
                .push("⚠️ No scene loaded – cannot pull behavior graphs.".into());
            return;
        };

        let entity_name = scene_state
            .world()
            .name(entity)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Entity_{}", entity));
        let current_graph = scene_state.world().behavior_graph(entity).cloned();

        match current_graph {
            Some(graph) => {
                self.behavior_graph_doc = BehaviorGraphDocument::from_runtime(&graph);
                self.behavior_graph_doc.rebuild_next_id();
                self.behavior_graph_binding =
                    Some(BehaviorGraphBinding::new(entity, entity_name.clone()));
                self.console_logs.push(format!(
                    "📥 Loaded behavior graph from {} (#{}) into the editor.",
                    entity_name, entity
                ));
            }
            None => {
                self.behavior_graph_doc = BehaviorGraphDocument::new_default();
                self.behavior_graph_binding =
                    Some(BehaviorGraphBinding::new(entity, entity_name.clone()));
                self.console_logs.push(format!(
                    "🆕 {} had no behavior graph; starting from the default template.",
                    entity_name
                ));
            }
        }
    }

    fn apply_behavior_graph_to_selection(&mut self) {
        let Some(entity) = self.selected_entity_handle() else {
            self.console_logs
                .push("⚠️ Select an entity before applying a behavior graph.".into());
            return;
        };

        let runtime_graph = match self.behavior_graph_doc.to_runtime() {
            Ok(graph) => graph,
            Err(err) => {
                self.console_logs
                    .push(format!("❌ Behavior graph is invalid: {}", err));
                return;
            }
        };

        let Some(scene_state) = self.scene_state.as_mut() else {
            self.console_logs
                .push("⚠️ No scene loaded – cannot apply behavior graphs.".into());
            return;
        };

        let entity_name = scene_state
            .world()
            .name(entity)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Entity_{}", entity));
        scene_state
            .world_mut()
            .set_behavior_graph(entity, runtime_graph);
        scene_state.sync_entity(entity);
        self.behavior_graph_binding = Some(BehaviorGraphBinding::new(entity, entity_name.clone()));
        self.console_logs.push(format!(
            "✅ Applied behavior graph to {} (#{}) and synced the scene state.",
            entity_name, entity
        ));
    }

    fn spawn_prefab_from_drag(&mut self, prefab_path: PathBuf, spawn_pos: (i32, i32)) {
        let _span = span!(Level::INFO, "spawn_prefab", path = %prefab_path.display(), pos = ?(spawn_pos.0, spawn_pos.1)).entered();

        let Some(scene_state) = self.scene_state.as_mut() else {
            warn!("No scene loaded - cannot instantiate prefabs");
            self.console_logs
                .push("⚠️ No scene loaded – cannot instantiate prefabs.".into());
            return;
        };

        let prefab_name = prefab_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown");

        match self.prefab_manager.instantiate_prefab(
            &prefab_path,
            scene_state.world_mut(),
            spawn_pos,
        ) {
            Ok(root_entity) => {
                scene_state.sync_entity(root_entity);
                self.selected_entity = Some(root_entity as u64);
                info!(
                    "Instantiated prefab '{}' at ({}, {}) - root entity #{}",
                    prefab_name, spawn_pos.0, spawn_pos.1, root_entity
                );
                self.console_logs.push(format!(
                    "✅ Instantiated prefab '{}' at ({}, {}). Root entity: #{}",
                    prefab_name, spawn_pos.0, spawn_pos.1, root_entity
                ));
                self.status = format!("Spawned prefab: {}", prefab_name);
            }
            Err(err) => {
                error!("Failed to instantiate prefab '{}': {}", prefab_name, err);
                self.console_logs.push(format!(
                    "❌ Failed to instantiate prefab '{}': {}",
                    prefab_name, err
                ));
                self.status = format!("Failed to spawn prefab: {}", prefab_name);
            }
        }
    }

    /// Handle asset actions from the asset browser
    fn handle_asset_action(&mut self, action: AssetAction) {
        match action {
            AssetAction::ImportModel { path } => {
                let model_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("imported_model")
                    .to_string();

                if let Some(scene_state) = self.scene_state.as_mut() {
                    // Create a new entity with the model
                    let entity = scene_state.world_mut().spawn(
                        &model_name,
                        astraweave_core::IVec2 { x: 0, y: 0 },
                        astraweave_core::Team { id: 0 },
                        100,
                        0,
                    );
                    scene_state.sync_entity(entity);

                    // Get the editor entity and set its mesh
                    if let Some(editor_entity) = scene_state.get_editor_entity_mut(entity) {
                        editor_entity.set_mesh(path.display().to_string());
                    }

                    // Load the glTF model into the engine renderer
                    #[cfg(feature = "astraweave-render")]
                    if let Some(viewport) = &self.viewport {
                        if let Err(e) = viewport.load_gltf_model(&model_name, &path) {
                            warn!("Failed to load glTF model into renderer: {}", e);
                            self.console_logs.push(format!(
                                "⚠️ glTF loading failed: {}",
                                e
                            ));
                        } else {
                            debug!("Loaded glTF model '{}' into engine renderer", model_name);
                        }
                    }

                    self.selected_entity = Some(entity as u64);
                    info!("Imported model '{}' as entity #{}", model_name, entity);
                    self.console_logs.push(format!(
                        "✅ Imported model '{}' as entity #{}",
                        model_name, entity
                    ));
                    self.status = format!("Imported: {}", model_name);
                } else {
                    warn!("No scene loaded - cannot import model");
                    self.console_logs
                        .push("⚠️ No scene loaded – cannot import model.".into());
                }
            }

            AssetAction::ApplyTexture { path, texture_type } => {
                // Convert TextureType to MaterialSlot
                let slot = match texture_type {
                    TextureType::Albedo => MaterialSlot::Albedo,
                    TextureType::Normal => MaterialSlot::Normal,
                    TextureType::ORM => MaterialSlot::ORM,
                    TextureType::MRA => MaterialSlot::ORM, // Map MRA to ORM slot
                    TextureType::Roughness => MaterialSlot::Roughness,
                    TextureType::Metallic => MaterialSlot::Metallic,
                    TextureType::AO => MaterialSlot::AO,
                    TextureType::Emission => MaterialSlot::Emission,
                    TextureType::Height => MaterialSlot::Height,
                    TextureType::Unknown => MaterialSlot::Albedo, // Default to albedo
                };

                if let Some(selected_id) = self.selected_entity {
                    if let Some(scene_state) = self.scene_state.as_mut() {
                        if let Some(editor_entity) = scene_state
                            .get_editor_entity_mut(selected_id as astraweave_core::Entity)
                        {
                            editor_entity.set_texture(slot, path.clone());
                            info!(
                                "Applied {:?} texture '{}' to entity #{}",
                                slot,
                                path.display(),
                                selected_id
                            );
                            self.console_logs.push(format!(
                                "✅ Applied {:?} texture '{}' to entity #{}",
                                slot,
                                path.file_name().unwrap_or_default().to_string_lossy(),
                                selected_id
                            ));
                            self.status = format!(
                                "Applied texture: {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                        }
                    }
                } else {
                    warn!("No entity selected - cannot apply texture");
                    self.console_logs
                        .push("⚠️ Select an entity first to apply textures.".into());
                }
            }

            AssetAction::ApplyMaterial { path } => {
                let material_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("material")
                    .to_string();

                if let Some(selected_id) = self.selected_entity {
                    if let Some(scene_state) = self.scene_state.as_mut() {
                        if let Some(editor_entity) = scene_state
                            .get_editor_entity_mut(selected_id as astraweave_core::Entity)
                        {
                            // Create a new material with the name from the path
                            let mut material = entity_manager::EntityMaterial::new();
                            material.name = material_name.clone();
                            editor_entity.set_material(material);
                            info!(
                                "Applied material '{}' to entity #{}",
                                material_name, selected_id
                            );
                            self.console_logs.push(format!(
                                "✅ Applied material '{}' to entity #{}",
                                material_name, selected_id
                            ));
                            self.status = format!("Applied material: {}", material_name);
                        }
                    }
                } else {
                    warn!("No entity selected - cannot apply material");
                    self.console_logs
                        .push("⚠️ Select an entity first to apply materials.".into());
                }
            }

            AssetAction::LoadScene { path } => {
                let scene_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("scene")
                    .to_string();

                match scene_serialization::load_scene(&path) {
                    Ok(loaded_world) => {
                        self.scene_state = Some(EditorSceneState::new(loaded_world));
                        self.current_scene_path = Some(path.clone());
                        info!("Loaded scene: {}", scene_name);
                        self.console_logs
                            .push(format!("✅ Loaded scene: {}", scene_name));
                        self.status = format!("Loaded: {}", scene_name);
                    }
                    Err(err) => {
                        error!("Failed to load scene '{}': {}", scene_name, err);
                        self.console_logs
                            .push(format!("❌ Failed to load scene '{}': {}", scene_name, err));
                    }
                }
            }

            AssetAction::SpawnPrefab { path } => {
                self.spawn_prefab_from_drag(path, (0, 0));
            }

            AssetAction::OpenExternal { path } => {
                // Use std::process::Command to open files with default application
                #[cfg(target_os = "windows")]
                {
                    if let Err(err) = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &path.display().to_string()])
                        .spawn()
                    {
                        error!("Failed to open external: {}", err);
                        self.console_logs
                            .push(format!("❌ Failed to open: {}", err));
                    } else {
                        info!("Opened external: {}", path.display());
                    }
                }
                #[cfg(target_os = "macos")]
                {
                    if let Err(err) = std::process::Command::new("open").arg(&path).spawn() {
                        error!("Failed to open external: {}", err);
                        self.console_logs
                            .push(format!("❌ Failed to open: {}", err));
                    } else {
                        info!("Opened external: {}", path.display());
                    }
                }
                #[cfg(target_os = "linux")]
                {
                    if let Err(err) = std::process::Command::new("xdg-open").arg(&path).spawn() {
                        error!("Failed to open external: {}", err);
                        self.console_logs
                            .push(format!("❌ Failed to open: {}", err));
                    } else {
                        info!("Opened external: {}", path.display());
                    }
                }
            }

            AssetAction::InspectAsset { path } => {
                // Log for material inspector (future expansion)
                info!("Inspecting asset: {}", path.display());
                self.console_logs
                    .push(format!("🔍 Inspecting: {}", path.display()));
                self.status = format!(
                    "Inspecting: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
    }

    fn show_behavior_graph_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Behavior Graph Editor");
        let selected_entity = self.selected_entity_handle();

        ui.horizontal(|ui| match (selected_entity, self.scene_state.as_ref()) {
            (Some(entity), Some(state)) => {
                let label = state.world().name(entity).unwrap_or("Unnamed");
                ui.label(format!("Selected entity: {} (#{})", label, entity));
            }
            _ => {
                ui.label("Select an entity to load/apply behavior graphs.");
            }
        });

        ui.horizontal(|ui| {
            let has_selection = selected_entity.is_some() && self.scene_state.is_some();
            if ui
                .add_enabled(has_selection, egui::Button::new("Load From Selection"))
                .clicked()
            {
                self.load_behavior_graph_from_selection();
            }
            if ui
                .add_enabled(has_selection, egui::Button::new("Apply To Selection"))
                .clicked()
            {
                self.apply_behavior_graph_to_selection();
            }
            if ui
                .add_enabled(
                    self.behavior_graph_binding.is_some(),
                    egui::Button::new("Detach"),
                )
                .clicked()
            {
                self.behavior_graph_binding = None;
                self.console_logs
                    .push("📤 Behavior graph document detached from entity binding.".into());
            }
        });

        if let Some(binding) = &self.behavior_graph_binding {
            ui.label(format!(
                "Document bound to {} (#{}) – changes can be applied directly.",
                binding.name, binding.entity
            ));
        } else {
            ui.label("Document is unbound. Load from an entity or file to bind.");
        }

        ui.separator();
        self.behavior_graph_ui
            .show(ui, &mut self.behavior_graph_doc, |entry| {
                self.console_logs.push(entry);
            });
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
                    let save_path = std::path::Path::new("assets/material_live.json");
                    if fs::write(save_path, s).is_ok() {
                        self.status = "Saved assets/material_live.json".into();
                        self.console_logs
                            .push("✅ Material saved to assets/material_live.json".into());
                        // Trigger hot reload by reloading the material in the inspector
                        // The file watcher will also detect this change automatically
                        self.console_logs.push("🔄 Hot reload triggered".into());
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
                if let Some(scene_state) = self.scene_state.as_mut() {
                    let undo_error = self.undo_stack.undo(scene_state.world_mut()).err();

                    if let Some(e) = undo_error {
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
                if let Some(scene_state) = self.scene_state.as_mut() {
                    let redo_error = self.undo_stack.redo(scene_state.world_mut()).err();

                    if let Some(e) = redo_error {
                        self.console_logs.push(format!("❌ Redo failed: {}", e));
                    } else if let Some(desc) = self.undo_stack.undo_description() {
                        self.status = format!("⏭️  Redid: {}", desc);
                        self.console_logs.push(format!("⏭️  Redo: {}", desc));
                    }
                }
            }

            // Ctrl+S: Save Scene
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                if let Some(world) = self.edit_world() {
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
                } else {
                    self.console_logs.push("⚠️  No world to save".into());
                }
            }

            // Ctrl+O: Load Scene
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                let path = self.content_root.join("scenes/untitled.scene.ron");
                match scene_serialization::load_scene(&path) {
                    Ok(world) => {
                        self.scene_state = Some(EditorSceneState::new(world));
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
                if let Some(world) = self.edit_world() {
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
            }

            // Ctrl+V: Paste entities
            if i.modifiers.ctrl && i.key_pressed(egui::Key::V) {
                if let Some(clipboard) = &self.clipboard {
                    if let Some(scene_state) = self.scene_state.as_mut() {
                        let clipboard_data = clipboard.clone();
                        let offset = IVec2 { x: 1, y: 1 };
                        let cmd =
                            command::SpawnEntitiesCommand::new(clipboard_data.clone(), offset);
                        let paste_result = self.undo_stack.execute(cmd, scene_state.world_mut());

                        match paste_result {
                            Ok(()) => {
                                let count = clipboard_data.entities.len();
                                self.status = format!("📋 Pasted {} entities", count);
                                self.console_logs
                                    .push(format!("✅ Pasted {} entities", count));
                            }
                            Err(e) => {
                                self.status = format!("❌ Paste failed: {}", e);
                                self.console_logs.push(format!("❌ Paste failed: {}", e));
                            }
                        }
                    }
                } else {
                    self.console_logs.push("⚠️  Clipboard is empty".into());
                }
            }

            // Ctrl+D: Duplicate selected entities
            if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
                if let Some(scene_state) = self.scene_state.as_mut() {
                    let selected = self.hierarchy_panel.get_all_selected();
                    if !selected.is_empty() {
                        let offset = IVec2 { x: 1, y: 1 };
                        let cmd = command::DuplicateEntitiesCommand::new(selected.clone(), offset);
                        let duplicate_result =
                            self.undo_stack.execute(cmd, scene_state.world_mut());

                        match duplicate_result {
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
            }

            // F5: Play / Resume
            if i.key_pressed(egui::Key::F5) {
                self.request_play();
            }

            // F6: Pause/Unpause
            if i.key_pressed(egui::Key::F6) {
                if self.editor_mode.is_playing() {
                    self.request_pause();
                } else if self.editor_mode.is_paused() {
                    self.request_play();
                }
            }

            // F7: Stop (restore snapshot)
            if i.key_pressed(egui::Key::F7) {
                self.request_stop();
            }

            // F8: Step one frame
            if i.key_pressed(egui::Key::F8) {
                self.request_step();
            }

            // Delete: Delete selected entities
            if i.key_pressed(egui::Key::Delete)
                && self.editor_mode.can_edit() {
                    if let Some(scene_state) = self.scene_state.as_mut() {
                        let selected = self.hierarchy_panel.get_all_selected();
                        if !selected.is_empty() {
                            let cmd = command::DeleteEntitiesCommand::new(selected.clone());
                            let delete_result =
                                self.undo_stack.execute(cmd, scene_state.world_mut());

                            match delete_result {
                                Ok(()) => {
                                    self.hierarchy_panel.set_selected(None);
                                    self.selected_entity = None;
                                    self.status =
                                        format!("🗑️  Deleted {} entities", selected.len());
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

            // Ctrl+N: New Scene
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) && !i.modifiers.shift {
                self.scene_state = Some(EditorSceneState::new(World::new()));
                self.current_scene_path = None;
                self.undo_stack.clear();
                self.hierarchy_panel.set_selected(None);
                self.selected_entity = None;
                self.status = "📄 New scene created".to_string();
                self.console_logs.push("✅ Created new scene".into());
            }

            // Ctrl+Shift+S: Save As
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::S) {
                if let Some(world) = self.edit_world() {
                    let dir = self.content_root.join("scenes");
                    let _ = fs::create_dir_all(&dir);
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let path = dir.join(format!("scene_{}.scene.ron", timestamp));

                    match scene_serialization::save_scene(world, &path) {
                        Ok(()) => {
                            self.current_scene_path = Some(path.clone());
                            self.recent_files.add_file(path.clone());
                            self.status = format!("💾 Saved scene as {:?}", path);
                            self.console_logs.push(format!("✅ Scene saved as: {:?}", path));
                        }
                        Err(e) => {
                            self.status = format!("❌ Save As failed: {}", e);
                            self.console_logs.push(format!("❌ Save As failed: {}", e));
                        }
                    }
                }
            }

            // Ctrl+A: Select All entities
            if i.modifiers.ctrl && i.key_pressed(egui::Key::A) && !i.modifiers.shift {
                if let Some(world) = self.edit_world() {
                    let all_entities = world.entities();
                    if !all_entities.is_empty() {
                        self.hierarchy_panel.set_selected_multiple(&all_entities);
                        self.status = format!("Selected {} entities", all_entities.len());
                    }
                }
            }

            // Escape: Deselect all (when not in gizmo mode)
            if i.key_pressed(egui::Key::Escape) && self.editor_mode.can_edit() {
                self.hierarchy_panel.set_selected(None);
                self.selected_entity = None;
                self.status = "Selection cleared".to_string();
            }

            // F: Focus camera on selected entity
            if i.key_pressed(egui::Key::F) && !i.modifiers.ctrl {
                if let Some(selected_id) = self.selected_entity {
                    if let Some(entity) = self.entity_manager.get(selected_id) {
                        if let Some(viewport) = &mut self.viewport {
                            let entity_pos = glam::Vec3::new(
                                entity.position.x,
                                entity.position.y,
                                entity.position.z,
                            );
                            viewport.camera_mut().frame_entity(entity_pos, 2.0);
                            self.status = format!("Focused on entity {}", selected_id);
                        }
                    }
                } else {
                    self.status = "No entity selected to focus".to_string();
                }
            }

            // Home: Reset camera to origin
            if i.key_pressed(egui::Key::Home) {
                if let Some(viewport) = &mut self.viewport {
                    viewport.camera_mut().reset_to_origin();
                    self.status = "Camera reset to origin".to_string();
                }
            }

            // Numpad 1: Front view
            if i.key_pressed(egui::Key::Num1) && i.modifiers.alt {
                if let Some(viewport) = &mut self.viewport {
                    viewport.camera_mut().set_view_front();
                    self.status = "Front view".to_string();
                }
            }

            // Numpad 3: Right view
            if i.key_pressed(egui::Key::Num3) && i.modifiers.alt {
                if let Some(viewport) = &mut self.viewport {
                    viewport.camera_mut().set_view_right();
                    self.status = "Right view".to_string();
                }
            }

            // Numpad 7: Top view
            if i.key_pressed(egui::Key::Num7) && i.modifiers.alt {
                if let Some(viewport) = &mut self.viewport {
                    viewport.camera_mut().set_view_top();
                    self.status = "Top view".to_string();
                }
            }

            // Numpad 0 / Alt+0: Perspective view
            if i.key_pressed(egui::Key::Num0) && i.modifiers.alt {
                if let Some(viewport) = &mut self.viewport {
                    viewport.camera_mut().set_view_perspective();
                    self.status = "Perspective view".to_string();
                }
            }

            // Ctrl+D: Duplicate selected entities
            if i.modifiers.ctrl && i.key_pressed(egui::Key::D) {
                if let Some(selected_id) = self.selected_entity {
                    if let Some(entity) = self.entity_manager.get(selected_id) {
                        let new_name = format!("{}_copy", entity.name.clone());
                        let new_pos = glam::Vec3::new(
                            entity.position.x + 1.0,
                            entity.position.y,
                            entity.position.z + 1.0,
                        );
                        let rotation = entity.rotation;
                        let scale = entity.scale;
                        let new_id = self.entity_manager.create(new_name);
                        if let Some(new_entity) = self.entity_manager.get_mut(new_id) {
                            new_entity.set_transform(new_pos, rotation, scale);
                        }
                        self.selected_entity = Some(new_id);
                        self.hierarchy_panel.set_selected(Some(new_id as u32));
                        self.status = format!("Duplicated entity {} -> {}", selected_id, new_id);
                    }
                } else {
                    self.status = "No entity selected to duplicate".to_string();
                }
            }
        });

        // Phase 2.2: Autosave every 5 minutes
        if self.last_autosave.elapsed().as_secs() >= 300 {
            if let Some(world) = self.edit_world() {
                let autosave_dir = self.content_root.join(".autosave");
                let _ = fs::create_dir_all(&autosave_dir);
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
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
        }

        // Update Astract panels
        if self.editor_mode.is_editing() {
            self.performance_panel.clear_runtime_stats();
        } else {
            let stats = self.runtime.stats().clone();
            self.performance_panel.push_runtime_stats(&stats);

            if self.last_runtime_log.elapsed().as_millis() >= 500 {
                self.profiler_data.push(format!(
                    "Tick {:05} | {:>4} ents | {:>5.2} ms | {:>3.0} FPS",
                    stats.tick_count, stats.entity_count, stats.frame_time_ms, stats.fps
                ));
                if self.profiler_data.len() > 60 {
                    self.profiler_data.remove(0);
                }
                self.last_runtime_log = std::time::Instant::now();
            }
        }
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
                    if let Some(world) = self.edit_world() {
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
                    } else {
                        self.console_logs.push("⚠️  No world to save".into());
                    }
                }

                if ui.button("📂 Load Scene").clicked() {
                    let path = self.content_root.join("scenes/untitled.scene.ron");
                    match scene_serialization::load_scene(&path) {
                        Ok(world) => {
                            self.scene_state = Some(EditorSceneState::new(world));
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
                    ui.add_space(6.0);
                    self.show_play_controls(ui);
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
                                        self.scene_state = Some(EditorSceneState::new(world));
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
                        self.request_play();
                    }

                    let pause_enabled = self.editor_mode.is_playing();
                    if ui
                        .add_enabled(pause_enabled, egui::Button::new("⏸️ Pause (F6)"))
                        .clicked()
                    {
                        self.request_pause();
                    }

                    let step_enabled = self.editor_mode.is_paused();
                    if ui
                        .add_enabled(step_enabled, egui::Button::new("⏭️ Step (F8)"))
                        .clicked()
                    {
                        self.request_step();
                    }

                    let stop_enabled = !self.editor_mode.is_editing();
                    if ui
                        .add_enabled(stop_enabled, egui::Button::new("⏹️ Stop (F7)"))
                        .clicked()
                    {
                        self.request_stop();
                    }

                    ui.separator();

                    let status_label = egui::RichText::new(self.editor_mode.status_text())
                        .color(self.editor_mode.status_color());
                    ui.label(status_label);

                    if !self.editor_mode.is_editing() {
                        let stats = self.runtime.stats();
                        ui.label(format!(
                            "| tick {} | entities {} | {:.1} ms ({:.0} FPS)",
                            stats.tick_count, stats.entity_count, stats.frame_time_ms, stats.fps
                        ));
                    } else if let Some(world) = self.active_world() {
                        ui.label(format!("| {} entities", world.entities().len()));
                    }
                });
                if ui.button("Diff Assets").clicked() {
                    match std::process::Command::new("git")
                        .args(["diff", "assets"])
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

            // Show play controls in toolbar
            ui.separator();
            self.show_play_controls(ui);
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
                            
                            // Process dragged prefabs after UI rendering
                            if let Some(dragged_path) = self.asset_browser.take_dragged_prefab() {
                                // Default spawn position: center of viewport (0, 0 in grid coordinates)
                                let spawn_pos = (0, 0);
                                self.spawn_prefab_from_drag(dragged_path, spawn_pos);
                            }
                        });

                        // Process asset actions outside the collapsing closure
                        let actions = self.asset_browser.take_pending_actions();
                        for action in actions {
                            self.handle_asset_action(action);
                        }

                        ui.add_space(10.0);

                        ui.collapsing("🌲 Hierarchy", |ui| {
                            let runtime_state = self.runtime.state();
                            let world_opt = if runtime_state == RuntimeState::Editing {
                                self.scene_state
                                    .as_mut()
                                    .map(|state| state.world_mut())
                            } else {
                                self.runtime.sim_world_mut()
                            };

                            if let Some(world) = world_opt {
                                self.hierarchy_panel.sync_with_world(world);
                                if let Some(selected) =
                                    self.hierarchy_panel.show_with_world(ui, world)
                                {
                                    self.selected_entity = Some(selected as u64);
                                }
                            }
                        });

                        let all_selected = self.hierarchy_panel.get_all_selected();
                        self.selection_set.clear();
                        for &entity_id in &all_selected {
                            self.selection_set.add(entity_id as u64, false);
                        }
                        if let Some(primary) = all_selected.last() {
                            self.selection_set.primary = Some(*primary as u64);
                        }

                        ui.add_space(10.0);

                        ui.collapsing("🔧 Transform", |ui| {
                            // Sync selected entity to transform panel
                            if let Some(selected_id) = self.selected_entity {
                                if let Some(entity) = self.entity_manager.get(selected_id) {
                                    // Update panel with entity transform
                                    let transform = crate::gizmo::Transform {
                                        position: entity.position,
                                        rotation: entity.rotation,
                                        scale: entity.scale,
                                    };
                                    self.transform_panel.set_selected(transform);
                                }
                            } else {
                                self.transform_panel.clear_selection();
                            }

                            self.transform_panel.show(ui);

                            // Apply changes back to entity if transform was modified
                            if let Some(selected_id) = self.selected_entity {
                                if let Some(new_transform) = self.transform_panel.get_transform() {
                                    self.entity_manager.update_transform(
                                        selected_id,
                                        new_transform.position,
                                        new_transform.rotation,
                                        new_transform.scale,
                                    );
                                }
                            }
                        });

                        ui.add_space(10.0);

                        ui.collapsing("📊 Charts", |ui| {
                            self.charts_panel.show(ui);
                        });

                        ui.add_space(10.0);

                        ui.collapsing("🎮 Entities", |ui| {
                            let selected_u32 = self.selected_entity.map(|e| e as u32);
                            let has_scene = self.scene_state.is_some();
                            
                            // Look up prefab instance if entity is selected
                            let prefab_instance = selected_u32.and_then(|entity| {
                                self.prefab_manager.find_instance(entity)
                            });
                            
                            let (component_edit, prefab_action) = {
                                let scene_handle = self.scene_state.as_mut();
                                self.entity_panel.show_with_scene_state(
                                    ui,
                                    scene_handle,
                                    selected_u32,
                                    prefab_instance,
                                )
                            };

                            // Handle prefab actions (Apply/Revert)
                            if let Some(action) = prefab_action {
                                let _span = span!(Level::INFO, "prefab_action", action = ?action).entered();
                                
                                match action {
                                    PrefabAction::RevertToOriginal(entity) => {
                                        if let Some(instance) = self.prefab_manager.find_instance_mut(entity) {
                                            let source = instance.source.display().to_string();
                                            if let Some(world) = self.scene_state.as_mut().map(|s| s.world_mut()) {
                                                match instance.revert_to_prefab(world) {
                                                    Ok(()) => {
                                                        info!("Reverted entity #{} to prefab: {}", entity, source);
                                                        self.console_logs.push(format!(
                                                            "🔄 Reverted entity #{} to prefab original",
                                                            entity
                                                        ));
                                                        self.status = "🔄 Reverted to prefab".into();
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to revert entity #{} to {}: {}", entity, source, e);
                                                        self.console_logs.push(format!(
                                                            "❌ Failed to revert entity #{}: {}",
                                                            entity, e
                                                        ));
                                                        self.status = format!("❌ Revert failed: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    PrefabAction::ApplyChangesToFile(entity) => {
                                        if let Some(instance) = self.prefab_manager.find_instance_mut(entity) {
                                            let source = instance.source.display().to_string();
                                            if let Some(world) = self.scene_state.as_ref().map(|s| s.world()) {
                                                match instance.apply_to_prefab(world) {
                                                    Ok(()) => {
                                                        info!("Applied entity #{} changes to prefab: {}", entity, source);
                                                        self.console_logs.push(format!(
                                                            "💾 Applied entity #{} changes to prefab file",
                                                            entity
                                                        ));
                                                        self.status = "💾 Applied to prefab".into();
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to apply entity #{} to {}: {}", entity, source, e);
                                                        self.console_logs.push(format!(
                                                            "❌ Failed to apply entity #{}: {}",
                                                            entity, e
                                                        ));
                                                        self.status = format!("❌ Apply failed: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    PrefabAction::RevertAllToOriginal(entity) => {
                                        if let Some(instance) = self.prefab_manager.find_instance_mut(entity) {
                                            let source = instance.source.display().to_string();
                                            let entity_count = instance.entity_mapping.len();
                                            if let Some(world) = self.scene_state.as_mut().map(|s| s.world_mut()) {
                                                match instance.revert_all_to_prefab(world) {
                                                    Ok(()) => {
                                                        info!("Reverted {} entities to prefab: {}", entity_count, source);
                                                        self.console_logs.push(format!(
                                                            "🔄 Reverted {} entities to prefab original",
                                                            entity_count
                                                        ));
                                                        self.status = format!("🔄 Reverted {} entities to prefab", entity_count);
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to revert {} entities to {}: {}", entity_count, source, e);
                                                        self.console_logs.push(format!(
                                                            "❌ Failed to revert {} entities: {}",
                                                            entity_count, e
                                                        ));
                                                        self.status = format!("❌ Revert all failed: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    PrefabAction::ApplyAllChangesToFile(entity) => {
                                        if let Some(instance) = self.prefab_manager.find_instance_mut(entity) {
                                            let source = instance.source.display().to_string();
                                            let entity_count = instance.entity_mapping.len();
                                            if let Some(world) = self.scene_state.as_ref().map(|s| s.world()) {
                                                match instance.apply_all_to_prefab(world) {
                                                    Ok(()) => {
                                                        info!("Applied {} entities to prefab: {}", entity_count, source);
                                                        self.console_logs.push(format!(
                                                            "💾 Applied {} entities to prefab file",
                                                            entity_count
                                                        ));
                                                        self.status = format!("💾 Applied {} entities to prefab", entity_count);
                                                    }
                                                    Err(e) => {
                                                        error!("Failed to apply {} entities to {}: {}", entity_count, source, e);
                                                        self.console_logs.push(format!(
                                                            "❌ Failed to apply {} entities: {}",
                                                            entity_count, e
                                                        ));
                                                        self.status = format!("❌ Apply all failed: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(component_edit) = component_edit {
                                use crate::command::{
                                    EditAmmoCommand, EditHealthCommand, EditTeamCommand,
                                };
                                use crate::component_ui::ComponentEdit;

                                let cmd: Box<dyn crate::command::EditorCommand> =
                                    match component_edit {
                                        ComponentEdit::Health {
                                            entity,
                                            old_hp,
                                            new_hp,
                                        } => EditHealthCommand::new(entity, old_hp, new_hp),
                                        ComponentEdit::Team {
                                            entity,
                                            old_id,
                                            new_id,
                                        } => EditTeamCommand::new(
                                            entity,
                                            Team { id: old_id },
                                            Team { id: new_id },
                                        ),
                                        ComponentEdit::Ammo {
                                            entity,
                                            old_rounds,
                                            new_rounds,
                                        } => EditAmmoCommand::new(entity, old_rounds, new_rounds),
                                    };

                                if has_scene {
                                    self.undo_stack.push_executed(cmd);
                                }
                            }
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

                        ui.add_space(10.0);

                        // Phase 5.2: Build Manager
                        ui.collapsing("🔨 Build Manager", |ui| {
                            self.build_manager_panel.show(ui);
                        });

                        ui.add_space(10.0);

                        // Phase 5.3: Plugin System
                        ui.collapsing("🔌 Plugins", |ui| {
                            self.plugin_panel.show(ui, &mut self.plugin_manager, ctx);
                        });

                        ui.add_space(10.0);

                        // Phase 5.5: Theme & Layout
                        ui.collapsing("🎨 Theme & Layout", |ui| {
                            self.theme_manager.show(ui);
                            // Apply theme changes immediately
                            self.theme_manager.apply_theme(ctx);
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
                StatusBar::show(
                    ui,
                    &self.editor_mode,
                    &self.current_gizmo_mode,
                    &self.selection_set,
                    &self.undo_stack,
                    &self.snapping_config,
                    self.current_fps,
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // 3D Viewport (Phase 1.1 - Babylon.js-style editor)
            if let Some(viewport) = &mut self.viewport {
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

                    ui.horizontal(|ui| {
                        ui.label("⚡ Snapping:");

                        ui.checkbox(&mut self.snapping_config.grid_enabled, "Grid");

                        ui.label("Size:");
                        let mut grid_size_idx = match self.snapping_config.grid_size {
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
                            self.snapping_config.grid_size = match grid_size_idx {
                                0 => 0.5,
                                1 => 1.0,
                                2 => 2.0,
                                _ => 1.0,
                            };
                        }

                        ui.separator();
                        ui.checkbox(&mut self.snapping_config.angle_enabled, "Angle");
                        ui.label(format!("{}°", self.snapping_config.angle_increment));
                    });

                    ui.separator();

                    // Render viewport (takes 70% width, full available height)
                    let runtime_state = self.runtime.state();
                    if runtime_state == RuntimeState::Editing && self.scene_state.is_none() {
                        self.scene_state =
                            Some(EditorSceneState::new(Self::create_default_world()));
                    }

                    let mut edited_world = false;
                    let world_to_render = if runtime_state == RuntimeState::Editing {
                        self.scene_state.as_mut().map(|state| state.world_mut())
                    } else {
                        self.runtime.sim_world_mut()
                    };

                    if let Some(world) = world_to_render {
                        if runtime_state == RuntimeState::Editing {
                            edited_world = true;
                        }
                        if let Err(e) = viewport.ui(
                            ui,
                            world,
                            &mut self.entity_manager,
                            &mut self.undo_stack,
                            Some(&mut self.prefab_manager),
                        ) {
                            self.console_logs.push(format!("❌ Viewport error: {}", e));
                            warn!("❌ Viewport error: {}", e);
                        }
                    } else {
                        ui.label("⚠️ No world available for rendering");
                    }

                    if edited_world {
                        if let Some(scene_state) = self.scene_state.as_mut() {
                            scene_state.sync_all();
                        }
                    }

                    // Sync selected entity from viewport to app state
                    if let Some(selected) = viewport.selected_entity() {
                        self.selected_entity = Some(selected as u64);
                    }

                    // Sync snapping settings from viewport toolbar to EditorApp
                    self.snapping_config.grid_enabled = viewport.toolbar().snap_enabled;
                    self.snapping_config.grid_size = viewport.toolbar().snap_size;
                    self.snapping_config.angle_enabled = viewport.toolbar().angle_snap_enabled;
                    self.snapping_config.angle_increment = viewport.toolbar().angle_snap_degrees;

                    ui.add_space(10.0);
                });

                ui.separator();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Auto-expand Console when simulation is running (so users see feedback)
                let console_open = self.runtime.is_playing() || !self.console_logs.is_empty();

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
        if let Err(e) = self.runtime.tick(frame_time) {
            self.console_logs
                .push(format!("❌ Runtime tick failed: {}", e));
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
