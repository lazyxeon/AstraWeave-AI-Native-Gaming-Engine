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

#[derive(Clone, Copy, PartialEq)]
enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

struct Toast {
    message: String,
    level: ToastLevel,
    created: std::time::Instant,
}

impl Toast {
    fn new(message: impl Into<String>, level: ToastLevel) -> Self {
        Self {
            message: message.into(),
            level,
            created: std::time::Instant::now(),
        }
    }

    fn color(&self) -> egui::Color32 {
        match self.level {
            ToastLevel::Info => egui::Color32::from_rgb(100, 149, 237),
            ToastLevel::Success => egui::Color32::from_rgb(50, 205, 50),
            ToastLevel::Warning => egui::Color32::from_rgb(255, 165, 0),
            ToastLevel::Error => egui::Color32::from_rgb(220, 20, 60),
        }
    }

    fn is_expired(&self, duration_secs: f32) -> bool {
        self.created.elapsed().as_secs_f32() > duration_secs
    }
}

mod asset_pack;
mod behavior_graph;
mod brdf_preview;
mod clipboard; // Phase 3.4 - Copy/Paste/Duplicate
mod command; // Phase 2.1 - Undo/Redo system
mod component_ui; // Phase 2.3 - Component-based inspector
mod editor_mode; // Phase 4.2 - Play-in-Editor
mod editor_preferences; // Phase 9 - Editor preferences persistence
mod entity_manager;
mod file_watcher;
mod game_project; // Game project configuration (game.toml)
mod gizmo;
mod interaction; // Phase 8.1 Week 5 Day 3 - Gizmo interaction helpers (auto-tracking)
mod level_doc; // Level document types
mod material_inspector;
mod panels;
mod prefab; // Phase 4.1 - Prefab System
mod recent_files; // Phase 3 - Recent files tracking
mod runtime; // Week 4 - Deterministic runtime integration
mod scene_serialization; // Phase 2.2 - Scene Save/Load
mod scene_state; // Week 1 - Canonical edit-mode world owner
mod terrain_integration; // Terrain generation integration
mod ui; // Phase 3 - UI components (StatusBar, etc.)
mod viewport; // Phase 1.1 - 3D Viewport
mod voxel_tools; // Phase 10: Voxel editing tools // Phase 2: Asset packaging and compression

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
    ChartsPanel, ConsolePanel, EntityPanel, GraphPanel, HierarchyPanel, Panel, PerformancePanel,
    ProfilerPanel, SceneStats, SceneStatsPanel, TextureType, ThemeManagerPanel, TransformPanel,
    WorldPanel,
};
mod dock_layout;
mod dock_panels;
mod panel_type;
mod plugin;
mod tab_viewer;
use dock_layout::{DockLayout, LayoutPreset};
use panel_type::PanelType;
use prefab::PrefabManager;
use recent_files::RecentFilesManager;
use runtime::{EditorRuntime, RuntimeState};
use scene_state::{EditorSceneState, TransformableScene};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tab_viewer::{EditorDrawContext, EditorTabViewer, EntityInfo};
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

/// Simple asset registry for tracking loaded assets
#[derive(Default)]
struct AssetRegistry {
    count: usize,
}

impl AssetRegistry {
    fn count(&self) -> usize {
        self.count
    }

    #[allow(dead_code)]
    fn set_count(&mut self, count: usize) {
        self.count = count;
    }
}

/// Week 4 Day 5: Asset validation result for import operations
#[derive(Default)]
struct AssetValidation {
    is_valid: bool,
    warnings: Vec<String>,
    info: Vec<String>,
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
    // Enhanced Console Panel
    console_panel: ConsolePanel,
    // Scene Statistics Panel
    scene_stats_panel: SceneStatsPanel,
    // Performance Profiler Panel
    profiler_panel: ProfilerPanel,
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
    // Phase 6: Dirty flag for unsaved changes
    is_dirty: bool,
    show_quit_dialog: bool,
    // Phase 6: Toast notifications
    toasts: Vec<Toast>,
    // Phase 7: Help dialog
    show_help_dialog: bool,
    // Phase 8: Viewport settings
    show_grid: bool,
    // Phase 8: Auto-save
    auto_save_enabled: bool,
    auto_save_interval_secs: f32,
    last_auto_save: std::time::Instant,
    // Phase 9: Settings dialog
    show_settings_dialog: bool,
    // Phase 9: Panel visibility
    show_hierarchy_panel: bool,
    show_inspector_panel: bool,
    show_console_panel: bool,
    // Phase 10: Confirm dialog for new scene
    show_new_confirm_dialog: bool,
    // Phase 10: Voxel editing tools
    voxel_editor: voxel_tools::VoxelEditor,
    // Phase 11: Professional Docking System
    dock_layout: DockLayout,
    dock_tab_viewer: EditorTabViewer,
    use_docking: bool,
    /// Counter for generating unique entity IDs
    next_entity_id: u64,
    /// Track if scene has unsaved changes
    is_scene_modified: bool,
    /// Asset registry for counting loaded assets
    asset_registry: AssetRegistry,
    /// Last save timestamp
    last_save_time: Option<String>,
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

        let prefs = editor_preferences::EditorPreferences::load();

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
            // Enhanced Console Panel
            console_panel: ConsolePanel::new(),
            // Scene Statistics Panel
            scene_stats_panel: SceneStatsPanel::new(),
            // Performance Profiler Panel
            profiler_panel: ProfilerPanel::new(),
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
            // Phase 6: Dirty flag for unsaved changes
            is_dirty: false,
            show_quit_dialog: false,
            // Phase 6: Toast notifications
            toasts: Vec::new(),
            // Phase 7: Help dialog
            show_help_dialog: false,
            // Phase 8: Viewport settings
            show_grid: prefs.show_grid,
            // Phase 8: Auto-save
            auto_save_enabled: prefs.auto_save_enabled,
            auto_save_interval_secs: prefs.auto_save_interval_secs,
            last_auto_save: std::time::Instant::now(),
            // Phase 9: Settings dialog
            show_settings_dialog: false,
            // Phase 9: Panel visibility
            show_hierarchy_panel: prefs.show_hierarchy_panel,
            show_inspector_panel: prefs.show_inspector_panel,
            show_console_panel: prefs.show_console_panel,
            // Phase 10: Confirm dialog for new scene
            show_new_confirm_dialog: false,
            // Phase 10: Voxel editing tools
            voxel_editor: voxel_tools::VoxelEditor::new(),
            // Phase 11: Professional Docking System
            dock_layout: DockLayout::from_preset(LayoutPreset::Default),
            dock_tab_viewer: EditorTabViewer::new(),
            use_docking: true, // Re-enabled after fixing layout gap
            // Entity ID counter - start after sample entities (100+)
            next_entity_id: 100,
            // Scene modification tracking
            is_scene_modified: false,
            asset_registry: AssetRegistry::default(),
            last_save_time: None,
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

    fn toast(&mut self, message: impl Into<String>, level: ToastLevel) {
        self.toasts.push(Toast::new(message, level));
    }

    fn toast_success(&mut self, message: impl Into<String>) {
        self.toast(message, ToastLevel::Success);
    }

    fn toast_error(&mut self, message: impl Into<String>) {
        self.toast(message, ToastLevel::Error);
    }

    fn toast_info(&mut self, message: impl Into<String>) {
        self.toast(message, ToastLevel::Info);
    }

    fn save_preferences(&self) {
        let prefs = editor_preferences::EditorPreferences {
            show_grid: self.show_grid,
            auto_save_enabled: self.auto_save_enabled,
            auto_save_interval_secs: self.auto_save_interval_secs,
            show_hierarchy_panel: self.show_hierarchy_panel,
            show_inspector_panel: self.show_inspector_panel,
            show_console_panel: self.show_console_panel,
            camera: self.viewport.as_ref().map(|v| v.camera().clone()),
            snapping: Some(self.snapping_config),
        };
        prefs.save();
    }

    fn log(&mut self, message: impl Into<String>) {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs() % 86400;
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        let timestamp = format!("[{:02}:{:02}:{:02}]", hours, mins, secs);
        self.console_logs
            .push(format!("{} {}", timestamp, message.into()));
    }

    fn create_new_scene(&mut self) {
        let viewport = self.viewport.take();
        let prefs = editor_preferences::EditorPreferences {
            show_grid: self.show_grid,
            auto_save_enabled: self.auto_save_enabled,
            auto_save_interval_secs: self.auto_save_interval_secs,
            show_hierarchy_panel: self.show_hierarchy_panel,
            show_inspector_panel: self.show_inspector_panel,
            show_console_panel: self.show_console_panel,
            camera: viewport.as_ref().map(|v| v.camera().clone()),
            snapping: Some(self.snapping_config),
        };
        *self = Self::default();
        self.viewport = viewport;
        self.show_grid = prefs.show_grid;
        self.auto_save_enabled = prefs.auto_save_enabled;
        self.auto_save_interval_secs = prefs.auto_save_interval_secs;
        self.show_hierarchy_panel = prefs.show_hierarchy_panel;
        self.show_inspector_panel = prefs.show_inspector_panel;
        self.show_console_panel = prefs.show_console_panel;
        if let Some(snapping) = prefs.snapping {
            self.snapping_config = snapping;
            if let Some(v) = &mut self.viewport {
                v.set_snapping_config(snapping);
            }
        }
        self.scene_state = Some(scene_state::EditorSceneState::new(World::new()));
        self.console_logs.push("New scene created".into());
        self.status = "New scene created".into();
    }

    /// Week 4 Day 5: Validate model file before import
    fn validate_model_file(&self, path: &std::path::Path) -> AssetValidation {
        let mut result = AssetValidation {
            is_valid: true,
            warnings: Vec::new(),
            info: Vec::new(),
        };

        // Check file exists
        if !path.exists() {
            result.is_valid = false;
            result.warnings.push("File does not exist".to_string());
            return result;
        }

        // Get file metadata
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                result.is_valid = false;
                result.warnings.push(format!("Cannot read file metadata: {}", e));
                return result;
            }
        };

        let file_size = metadata.len();
        let file_size_mb = file_size as f64 / (1024.0 * 1024.0);

        // Info: File size
        result.info.push(format!("File size: {:.2} MB", file_size_mb));

        // Warning: Very large files
        if file_size_mb > 100.0 {
            result.warnings.push(format!(
                "Very large model ({:.1} MB) - may cause slow loading",
                file_size_mb
            ));
        } else if file_size_mb > 50.0 {
            result.warnings.push(format!(
                "Large model ({:.1} MB) - consider using LODs",
                file_size_mb
            ));
        }

        // Check extension
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match extension.to_lowercase().as_str() {
            "glb" => {
                result.info.push("Format: GLB (binary glTF)".to_string());
            }
            "gltf" => {
                result.info.push("Format: glTF (JSON)".to_string());
                // Check for external files
                if let Some(parent) = path.parent() {
                    let bin_path = parent.join(format!(
                        "{}.bin",
                        path.file_stem().and_then(|s| s.to_str()).unwrap_or("")
                    ));
                    if bin_path.exists() {
                        result.info.push("External .bin file found".to_string());
                    }
                }
            }
            _ => {
                result.is_valid = false;
                result.warnings.push(format!("Unsupported format: .{}", extension));
            }
        }

        // Basic GLB header validation for binary files
        if extension.to_lowercase() == "glb" {
            if let Ok(file) = std::fs::File::open(path) {
                use std::io::Read;
                let mut reader = std::io::BufReader::new(file);
                let mut magic = [0u8; 4];
                if reader.read_exact(&mut magic).is_ok() {
                    // GLB magic number: "glTF" (0x676C5446)
                    if &magic == b"glTF" {
                        result.info.push("Valid GLB magic header".to_string());
                    } else {
                        result.is_valid = false;
                        result.warnings.push("Invalid GLB header - file may be corrupted".to_string());
                    }
                }
            }
        }

        result
    }

    /// Week 4 Day 3-4: Import texture with BC7 compression
    fn import_texture_with_compression(&mut self, path: &std::path::Path, file_name: &str) {
        use astraweave_asset_pipeline::compress_bc7;
        
        self.log(format!("🖼️ Importing texture: {}", file_name));
        
        // Load the image
        let image = match image::open(path) {
            Ok(img) => img,
            Err(e) => {
                self.log(format!("❌ Failed to open image: {}", e));
                self.toast_error(format!("Failed to open: {}", file_name));
                return;
            }
        };
        
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        let original_size = rgba.len();
        
        self.log(format!("📊 Image dimensions: {}×{}, {} bytes", width, height, original_size));
        
        // Check if dimensions are divisible by 4 (required for BC7)
        if width % 4 != 0 || height % 4 != 0 {
            self.log(format!("⚠️ BC7 requires dimensions divisible by 4 (got {}×{})", width, height));
            self.log("💡 Consider resizing to nearest power of 2 (e.g., 512×512, 1024×1024, 2048×2048)".to_string());
            self.toast_info(format!("{}: Needs resize for compression ({}×{})", file_name, width, height));
            return;
        }
        
        // Compress to BC7
        let start_time = std::time::Instant::now();
        match compress_bc7(&rgba) {
            Ok(compressed) => {
                let elapsed = start_time.elapsed();
                let compressed_size = compressed.len();
                let ratio = original_size as f32 / compressed_size as f32;
                let reduction = 100.0 * (1.0 - compressed_size as f32 / original_size as f32);
                
                self.log(format!(
                    "✅ BC7 compression complete: {} → {} bytes ({:.1}:1, {:.1}% reduction) in {:.2}s",
                    original_size, compressed_size, ratio, reduction, elapsed.as_secs_f32()
                ));
                
                // Save compressed texture to project folder
                let output_path = self.content_root.join("textures").join(format!(
                    "{}.bc7.bin",
                    path.file_stem().and_then(|s| s.to_str()).unwrap_or("texture")
                ));
                
                // Create textures directory if needed
                if let Some(parent) = output_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                
                match std::fs::write(&output_path, &compressed) {
                    Ok(_) => {
                        self.log(format!("💾 Saved BC7 texture: {}", output_path.display()));
                        self.toast_success(format!(
                            "Compressed {}: {:.1}× smaller",
                            file_name, ratio
                        ));
                    }
                    Err(e) => {
                        self.log(format!("❌ Failed to save compressed texture: {}", e));
                        self.toast_error(format!("Failed to save: {}", file_name));
                    }
                }
            }
            Err(e) => {
                self.log(format!("❌ BC7 compression failed: {}", e));
                self.toast_error(format!("Compression failed: {}", file_name));
            }
        }
    }

    /// Week 4: Handle files dropped onto the editor window
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        // Show drop overlay when files are being dragged over
        let hovered_files = ctx.input(|i| i.raw.hovered_files.clone());
        if !hovered_files.is_empty() {
            self.show_drop_overlay(ctx, &hovered_files);
        }

        // Check for dropped files
        let dropped_files: Vec<std::path::PathBuf> = ctx.input(|i| {
            i.raw.dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });

        if dropped_files.is_empty() {
            return;
        }

        for path in dropped_files {
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

            match extension.to_lowercase().as_str() {
                // 3D Models - Load to viewport with validation
                "glb" | "gltf" => {
                    self.log(format!("📦 Importing model: {}", file_name));
                    
                    // Week 4 Day 5: Validate asset before import
                    let validation = self.validate_model_file(&path);
                    for warning in &validation.warnings {
                        self.log(format!("⚠️ {}", warning));
                    }
                    
                    if !validation.is_valid {
                        self.toast_error(format!("Invalid model: {}", validation.warnings.first().unwrap_or(&"Unknown error".to_string())));
                        self.log(format!("❌ Model validation failed: {}", file_name));
                        continue;
                    }
                    
                    // Log validation stats
                    if !validation.info.is_empty() {
                        for info in &validation.info {
                            self.log(format!("ℹ️ {}", info));
                        }
                    }
                    
                    if let Some(viewport) = &self.viewport {
                        match viewport.load_gltf_model(file_name, &path) {
                            Ok(_) => {
                                self.toast_success(format!("Loaded model: {}", file_name));
                                self.log(format!("✅ Model loaded: {}", file_name));
                            }
                            Err(e) => {
                                self.toast_error(format!("Failed to load {}: {}", file_name, e));
                                self.log(format!("❌ Model load failed: {} - {}", file_name, e));
                            }
                        }
                    } else {
                        self.toast_error("No viewport available for model preview");
                        self.log("⚠️ Cannot import model: viewport not initialized");
                    }
                }

                // Scene files - Load scene
                "ron" => {
                    if path.to_string_lossy().contains("scene") {
                        self.log(format!("📂 Loading scene: {}", file_name));
                        match scene_serialization::load_scene(&path) {
                            Ok(world) => {
                                self.scene_state = Some(scene_state::EditorSceneState::new(world));
                                self.current_scene_path = Some(path.clone());
                                self.recent_files.add_file(path.clone());
                                self.is_dirty = false;
                                self.toast_success(format!("Loaded scene: {}", file_name));
                                self.log(format!("✅ Scene loaded: {}", file_name));
                            }
                            Err(e) => {
                                self.toast_error(format!("Failed to load scene: {}", e));
                                self.log(format!("❌ Scene load failed: {}", e));
                            }
                        }
                    } else {
                        self.log(format!("ℹ️ RON file dropped (not a scene): {}", file_name));
                    }
                }

                // Textures - Import with BC7 compression
                "png" | "jpg" | "jpeg" => {
                    self.import_texture_with_compression(&path, file_name);
                }
                
                // KTX2 textures - Already compressed, just register
                "ktx2" => {
                    self.log(format!("🖼️ KTX2 texture imported: {} (already compressed)", file_name));
                    self.toast_success(format!("Imported: {} (KTX2)", file_name));
                }

                // Materials - Log for future implementation
                "toml" => {
                    if path.to_string_lossy().contains("material") {
                        self.log(format!("🎨 Material definition: {} (load material - TODO)", file_name));
                        self.toast_info(format!("Material: {} (material loading coming soon)", file_name));
                    } else {
                        self.log(format!("📄 TOML file dropped: {}", file_name));
                    }
                }

                // Audio files - Log for future implementation
                "ogg" | "wav" | "mp3" => {
                    self.log(format!("🔊 Audio file: {} (audio import - TODO)", file_name));
                    self.toast_info(format!("Audio: {} (audio import coming soon)", file_name));
                }

                // Unknown file types
                _ => {
                    self.log(format!("❓ Unknown file type dropped: {} ({})", file_name, extension));
                    self.toast_info(format!("Unknown file: .{} not supported", extension));
                }
            }
        }
    }

    /// Week 4: Show visual overlay when dragging files over editor
    fn show_drop_overlay(&self, ctx: &egui::Context, hovered_files: &[egui::HoveredFile]) {
        let screen_rect = ctx.screen_rect();
        
        // Count file types
        let mut model_count = 0;
        let mut scene_count = 0;
        let mut texture_count = 0;
        let mut other_count = 0;

        for file in hovered_files {
            if let Some(path) = &file.path {
                match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
                    "glb" | "gltf" => model_count += 1,
                    "ron" => scene_count += 1,
                    "png" | "jpg" | "jpeg" | "ktx2" => texture_count += 1,
                    _ => other_count += 1,
                }
            } else {
                other_count += 1;
            }
        }

        // Build description text
        let mut parts = Vec::new();
        if model_count > 0 {
            parts.push(format!("📦 {} model{}", model_count, if model_count > 1 { "s" } else { "" }));
        }
        if scene_count > 0 {
            parts.push(format!("📂 {} scene{}", scene_count, if scene_count > 1 { "s" } else { "" }));
        }
        if texture_count > 0 {
            parts.push(format!("🖼️ {} texture{}", texture_count, if texture_count > 1 { "s" } else { "" }));
        }
        if other_count > 0 {
            parts.push(format!("📄 {} file{}", other_count, if other_count > 1 { "s" } else { "" }));
        }

        let description = if parts.is_empty() {
            "Drop files here".to_string()
        } else {
            parts.join(", ")
        };

        // Paint semi-transparent overlay
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("drop_overlay"),
        ));

        // Background dimming
        painter.rect_filled(
            screen_rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
        );

        // Center box with info
        let box_size = egui::vec2(400.0, 150.0);
        let box_rect = egui::Rect::from_center_size(screen_rect.center(), box_size);

        painter.rect_filled(
            box_rect,
            12.0,
            egui::Color32::from_rgb(40, 60, 80),
        );

        painter.rect_stroke(
            box_rect,
            12.0,
            egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 180, 255)),
            egui::StrokeKind::Outside,
        );

        // Icon
        painter.text(
            box_rect.center() - egui::vec2(0.0, 30.0),
            egui::Align2::CENTER_CENTER,
            "📥",
            egui::FontId::proportional(48.0),
            egui::Color32::WHITE,
        );

        // "Drop files here" text
        painter.text(
            box_rect.center() + egui::vec2(0.0, 20.0),
            egui::Align2::CENTER_CENTER,
            "Drop to import",
            egui::FontId::proportional(20.0),
            egui::Color32::from_rgb(200, 200, 200),
        );

        // File type description
        painter.text(
            box_rect.center() + egui::vec2(0.0, 45.0),
            egui::Align2::CENTER_CENTER,
            &description,
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(150, 200, 255),
        );
    }

    fn render_toasts(&mut self, ctx: &egui::Context) {
        const TOAST_DURATION: f32 = 4.0;
        const TOAST_WIDTH: f32 = 300.0;
        const TOAST_PADDING: f32 = 10.0;

        self.toasts.retain(|t| !t.is_expired(TOAST_DURATION));

        let screen_rect = ctx.screen_rect();
        let mut y_offset = screen_rect.max.y - TOAST_PADDING;

        for toast in self.toasts.iter().rev() {
            let age = toast.created.elapsed().as_secs_f32();
            let alpha = if age < 0.3 {
                age / 0.3
            } else if age > TOAST_DURATION - 0.5 {
                (TOAST_DURATION - age) / 0.5
            } else {
                1.0
            };

            let color = toast.color();
            let bg_color = egui::Color32::from_rgba_unmultiplied(
                color.r(),
                color.g(),
                color.b(),
                (200.0 * alpha) as u8,
            );

            let toast_rect = egui::Rect::from_min_size(
                egui::pos2(
                    screen_rect.max.x - TOAST_WIDTH - TOAST_PADDING,
                    y_offset - 40.0,
                ),
                egui::vec2(TOAST_WIDTH, 35.0),
            );

            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("toasts"),
            ));

            painter.rect_filled(toast_rect, 5.0, bg_color);
            painter.text(
                toast_rect.center(),
                egui::Align2::CENTER_CENTER,
                &toast.message,
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, (255.0 * alpha) as u8),
            );

            y_offset -= 45.0;
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

        // Load preferences again to ensure we have the latest
        let prefs = editor_preferences::EditorPreferences::load();

        // Initialize sample entities for testing
        Self::init_sample_entities(&mut app.entity_manager);

        // Initialize viewport (requires wgpu render state from CreationContext)
        match ViewportWidget::new(cc) {
            Ok(mut viewport) => {
                // Apply persisted camera and snapping settings
                if let Some(camera) = prefs.camera {
                    viewport.set_camera(camera);
                }
                if let Some(snapping) = prefs.snapping {
                    viewport.set_snapping_config(snapping);
                    app.snapping_config = snapping;
                }

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

        // Initialize default scene so asset imports work immediately
        let default_world = astraweave_core::World::new();
        app.scene_state = Some(EditorSceneState::new(default_world));
        app.console_logs.push("✅ Default scene created".into());

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
        self.console_panel
            .show_with_logs(ui, &mut self.console_logs);
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
                            self.console_logs
                                .push(format!("⚠️ glTF loading failed: {}", e));
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

                self.status = format!("Loading scene {}...", scene_name);
                self.log(format!("Loading scene: {}...", scene_name));

                match scene_serialization::load_scene(&path) {
                    Ok(loaded_world) => {
                        // Clear old scene state and prefab instances to prevent memory leaks
                        self.prefab_manager.clear_instances();
                        self.undo_stack.clear();

                        self.scene_state = Some(EditorSceneState::new(loaded_world));
                        self.current_scene_path = Some(path.clone());
                        self.is_dirty = false;

                        info!("Loaded scene: {}", scene_name);
                        self.toast_success(format!("Loaded scene: {}", scene_name));
                        self.log(format!("✅ Loaded scene: {}", scene_name));
                        self.status = format!("Loaded: {}", scene_name);

                        // Add to recent files
                        self.recent_files.add_file(path);
                    }
                    Err(err) => {
                        error!("Failed to load scene '{}': {}", scene_name, err);
                        self.toast_error(format!("Failed to load scene: {}", err));
                        self.log(format!("❌ Failed to load scene '{}': {}", scene_name, err));
                        self.status = "Error loading scene".into();
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

            AssetAction::LoadToViewport { path } => {
                // Load model directly to viewport for preview (no entity created)
                let model_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("preview_model")
                    .to_string();

                #[cfg(feature = "astraweave-render")]
                if let Some(viewport) = &self.viewport {
                    match viewport.load_gltf_model(&model_name, &path) {
                        Ok(()) => {
                            info!("Loaded '{}' to viewport for preview", model_name);
                            self.console_logs
                                .push(format!("👁️ Loaded '{}' to viewport", model_name));
                            self.status = format!("Viewing: {}", model_name);
                        }
                        Err(e) => {
                            warn!("Failed to load '{}' to viewport: {}", model_name, e);
                            self.console_logs
                                .push(format!("⚠️ Failed to load '{}': {}", model_name, e));
                        }
                    }
                } else {
                    warn!("No viewport available for model preview");
                    self.console_logs
                        .push("⚠️ Viewport not available for preview".into());
                }

                #[cfg(not(feature = "astraweave-render"))]
                {
                    warn!("astraweave-render feature not enabled - cannot preview model");
                    self.console_logs
                        .push("⚠️ Render feature not enabled - cannot preview model".into());
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
        ui.label("Live material editing - synced with 3D viewport");

        // Color sliders with live preview
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("🎨 Base Color:");
        });

        if ui.add(egui::Slider::new(&mut self.mat_doc.base_color[0], 0.0..=1.0).text("R")).changed() {
            changed = true;
        }
        if ui.add(egui::Slider::new(&mut self.mat_doc.base_color[1], 0.0..=1.0).text("G")).changed() {
            changed = true;
        }
        if ui.add(egui::Slider::new(&mut self.mat_doc.base_color[2], 0.0..=1.0).text("B")).changed() {
            changed = true;
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("⚙️ PBR Properties:");
        });

        if ui.add(egui::Slider::new(&mut self.mat_doc.metallic, 0.0..=1.0).text("Metallic")).changed() {
            changed = true;
        }
        if ui.add(egui::Slider::new(&mut self.mat_doc.roughness, 0.04..=1.0).text("Roughness")).changed() {
            changed = true;
        }

        // Apply changes to 3D viewport in real-time
        if changed {
            if let Some(viewport) = &self.viewport {
                let base_color = [
                    self.mat_doc.base_color[0],
                    self.mat_doc.base_color[1],
                    self.mat_doc.base_color[2],
                    1.0, // Alpha
                ];
                let _ = viewport.set_material_params(base_color, self.mat_doc.metallic, self.mat_doc.roughness);
            }
        }

        // Color preview swatch
        ui.add_space(8.0);
        let preview_color = egui::Color32::from_rgb(
            (self.mat_doc.base_color[0] * 255.0) as u8,
            (self.mat_doc.base_color[1] * 255.0) as u8,
            (self.mat_doc.base_color[2] * 255.0) as u8,
        );
        ui.horizontal(|ui| {
            ui.label("Preview:");
            let (rect, _response) = ui.allocate_exact_size(egui::vec2(40.0, 20.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 4.0, preview_color);
            ui.label(format!(
                "M:{:.2} R:{:.2}",
                self.mat_doc.metallic, self.mat_doc.roughness
            ));
        });

        ui.add_space(8.0);

        // Manual apply button (in case auto-sync didn't work)
        ui.horizontal(|ui| {
            if ui.button("🔄 Apply to Viewport").on_hover_text("Manually apply material to 3D viewport").clicked() {
                if let Some(viewport) = &self.viewport {
                    let base_color = [
                        self.mat_doc.base_color[0],
                        self.mat_doc.base_color[1],
                        self.mat_doc.base_color[2],
                        1.0,
                    ];
                    match viewport.set_material_params(base_color, self.mat_doc.metallic, self.mat_doc.roughness) {
                        Ok(_) => {
                            self.console_logs.push("🎨 Material applied to viewport".into());
                        }
                        Err(e) => {
                            self.console_logs.push(format!("⚠️ Material error: {}", e));
                        }
                    }
                }
            }

            // Sync status indicator
            if self.viewport.is_some() {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "🔗 Viewport synced");
            } else {
                ui.colored_label(egui::Color32::from_rgb(200, 150, 100), "⚠️ No viewport");
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Texture path
        let tex_ref = self.mat_doc.texture_path.get_or_insert(String::new());
        ui.horizontal(|ui| {
            ui.label("📁 Texture:");
            ui.text_edit_singleline(tex_ref);
        });

        ui.add_space(8.0);
        if ui.button("💾 Save & Reload Material").clicked() {
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

    fn show_voxel_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Voxel Editor");
        ui.label("Interactive terrain sculpting tools");

        let mut brush = *self.voxel_editor.brush();

        ui.group(|ui| {
            ui.label("Brush Settings");
            egui::ComboBox::from_label("Shape")
                .selected_text(format!("{:?}", brush.shape))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut brush.shape,
                        voxel_tools::BrushShape::Sphere,
                        "Sphere",
                    );
                    ui.selectable_value(&mut brush.shape, voxel_tools::BrushShape::Cube, "Cube");
                    ui.selectable_value(
                        &mut brush.shape,
                        voxel_tools::BrushShape::Cylinder,
                        "Cylinder",
                    );
                });

            egui::ComboBox::from_label("Mode")
                .selected_text(format!("{:?}", brush.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut brush.mode, voxel_tools::BrushMode::Add, "Add");
                    ui.selectable_value(&mut brush.mode, voxel_tools::BrushMode::Remove, "Remove");
                    ui.selectable_value(&mut brush.mode, voxel_tools::BrushMode::Paint, "Paint");
                });

            ui.add(egui::Slider::new(&mut brush.radius, 0.1..=20.0).text("Radius"));
            ui.add(egui::Slider::new(&mut brush.strength, 0.0..=1.0).text("Strength"));
            ui.checkbox(&mut brush.smooth, "Smooth Edges");
        });

        self.voxel_editor.set_brush(brush);

        ui.separator();

        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.voxel_editor.can_undo(), egui::Button::new("⏪ Undo"))
                .clicked()
            {
                self.log("Voxel undo requested (integration pending)");
            }
            if ui
                .add_enabled(self.voxel_editor.can_redo(), egui::Button::new("⏩ Redo"))
                .clicked()
            {
                self.log("Voxel redo requested (integration pending)");
            }
        });

        if ui.button("🗑 Clear History").clicked() {
            self.voxel_editor.clear_history();
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
        // FORCE zero spacing for all panels to eliminate unclaimed gaps
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(0.0, 0.0);
        style.spacing.window_margin = egui::Margin::same(0);
        ctx.set_style(style);

        if ctx.input(|i| i.viewport().close_requested()) && self.is_dirty {
            self.show_quit_dialog = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // Week 4: Handle drag-drop file imports
        self.handle_dropped_files(ctx);

        let now = std::time::Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.current_fps = if frame_time > 0.0 {
            1.0 / frame_time
        } else {
            60.0
        };

        self.profiler_panel.push_frame_time(frame_time * 1000.0);

        let selected_count = self.selection_set.entities.len();
        let scene_entity_count = self
            .scene_state
            .as_ref()
            .map(|s| s.world().entities().len())
            .unwrap_or(0);
        self.scene_stats_panel.update_stats(SceneStats {
            entity_count: scene_entity_count,
            selected_count,
            component_count: scene_entity_count * 3,
            prefab_count: self.prefab_manager.instance_count(),
            undo_stack_size: self.undo_stack.undo_count(),
            redo_stack_size: self.undo_stack.redo_count(),
            memory_estimate_kb: scene_entity_count * 2,
            scene_path: self
                .current_scene_path
                .as_ref()
                .map(|p| p.display().to_string()),
            is_dirty: self.is_dirty,
        });

        // Phase 7: Dynamic window title with file name and dirty state
        let file_name = self
            .current_scene_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");
        let dirty_marker = if self.is_dirty { "*" } else { "" };
        let entity_count = self
            .scene_state
            .as_ref()
            .map(|s| s.world().entities().len())
            .unwrap_or(0);
        let title = format!(
            "AstraWeave Editor - {}{} ({} entities)",
            file_name, dirty_marker, entity_count
        );
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));

        // Phase 8: Auto-save logic
        if self.auto_save_enabled
            && self.is_dirty
            && self.last_auto_save.elapsed().as_secs_f32() > self.auto_save_interval_secs
        {
            if let Some(world) = self.edit_world() {
                let path = self
                    .current_scene_path
                    .clone()
                    .unwrap_or_else(|| self.content_root.join("scenes/autosave.scene.ron"));
                if scene_serialization::save_scene(world, &path).is_ok() {
                    self.last_auto_save = std::time::Instant::now();
                    self.toast_info(format!(
                        "Auto-saved to {:?}",
                        path.file_name().unwrap_or_default()
                    ));
                }
            }
        }

        // Phase 6: Quit confirmation dialog
        if self.show_quit_dialog {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes. What would you like to do?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save & Quit").clicked() {
                            if let Some(world) = self.edit_world() {
                                let path = self.current_scene_path.clone().unwrap_or_else(|| {
                                    self.content_root.join("scenes/untitled.scene.ron")
                                });
                                let _ = scene_serialization::save_scene(world, &path);
                            }
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Quit Without Saving").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_quit_dialog = false;
                        }
                    });
                });
        }

        // Phase 7: Keyboard shortcuts help dialog
        if self.show_help_dialog {
            egui::Window::new("Keyboard Shortcuts")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("File");
                    ui.label("Ctrl+N          New Scene");
                    ui.label("Ctrl+Shift+N    New Entity");
                    ui.label("Ctrl+S          Save Scene");
                    ui.label("Ctrl+Shift+S    Save As");
                    ui.label("Ctrl+O          Open Scene");
                    ui.add_space(8.0);

                    ui.heading("Edit");
                    ui.label("Ctrl+Z          Undo");
                    ui.label("Ctrl+Shift+Z    Redo");
                    ui.label("Ctrl+A          Select All");
                    ui.label("Ctrl+D          Duplicate");
                    ui.label("Delete          Delete Selected");
                    ui.label("Escape          Deselect All");
                    ui.add_space(8.0);

                    ui.heading("Camera");
                    ui.label("F               Focus on Selected");
                    ui.label("Home            Reset Camera");
                    ui.label("Alt+1           Front View");
                    ui.label("Alt+3           Right View");
                    ui.label("Alt+7           Top View");
                    ui.label("Alt+0           Perspective View");
                    ui.add_space(8.0);

                    ui.heading("Gizmo");
                    ui.label("W               Translate Mode");
                    ui.label("E               Rotate Mode");
                    ui.label("R               Scale Mode");
                    ui.add_space(8.0);

                    ui.heading("View");
                    ui.label("F1              Show This Help");
                    ui.label("G               Toggle Grid");
                    ui.label("Escape          Close Dialogs");
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                self.show_help_dialog = false;
                            }
                        });
                    });
                });
        }

        // Phase 9: Settings/Preferences dialog
        if self.show_settings_dialog {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(true)
                .default_width(450.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("General");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Grid Visible:");
                        ui.checkbox(&mut self.show_grid, "");
                    });

                    ui.add_space(16.0);
                    ui.heading("Auto-Save");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Enable Auto-Save:");
                        ui.checkbox(&mut self.auto_save_enabled, "");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Interval (seconds):");
                        ui.add(
                            egui::DragValue::new(&mut self.auto_save_interval_secs)
                                .range(30.0..=3600.0)
                                .speed(10.0),
                        );
                    });

                    ui.add_space(16.0);
                    ui.heading("Panels");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Show Hierarchy:");
                        ui.checkbox(&mut self.show_hierarchy_panel, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Show Inspector:");
                        ui.checkbox(&mut self.show_inspector_panel, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Show Console:");
                        ui.checkbox(&mut self.show_console_panel, "");
                    });

                    ui.add_space(16.0);
                    ui.heading("Snapping");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Grid Snap:");
                        ui.checkbox(&mut self.snapping_config.grid_enabled, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Grid Size:");
                        ui.add(
                            egui::DragValue::new(&mut self.snapping_config.grid_size)
                                .range(0.1..=10.0)
                                .speed(0.1)
                                .suffix(" units"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Angle Snap:");
                        ui.checkbox(&mut self.snapping_config.angle_enabled, "");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Angle Increment:");
                        ui.add(
                            egui::DragValue::new(&mut self.snapping_config.angle_increment)
                                .range(1.0..=90.0)
                                .speed(1.0)
                                .suffix("°"),
                        );
                    });

                    ui.add_space(16.0);
                    ui.collapsing("Keyboard Shortcuts", |ui| {
                        egui::Grid::new("shortcuts_grid")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Ctrl+S");
                                ui.label("Save scene");
                                ui.end_row();
                                ui.label("Ctrl+Z");
                                ui.label("Undo");
                                ui.end_row();
                                ui.label("Ctrl+Y");
                                ui.label("Redo");
                                ui.end_row();
                                ui.label("Ctrl+C");
                                ui.label("Copy");
                                ui.end_row();
                                ui.label("Ctrl+V");
                                ui.label("Paste");
                                ui.end_row();
                                ui.label("Ctrl+D");
                                ui.label("Duplicate");
                                ui.end_row();
                                ui.label("Delete");
                                ui.label("Delete entity");
                                ui.end_row();
                                ui.label("F5");
                                ui.label("Play");
                                ui.end_row();
                                ui.label("F6");
                                ui.label("Pause");
                                ui.end_row();
                                ui.label("F7");
                                ui.label("Stop");
                                ui.end_row();
                                ui.label("F8");
                                ui.label("Step frame");
                                ui.end_row();
                                ui.label("G");
                                ui.label("Translate mode");
                                ui.end_row();
                                ui.label("R");
                                ui.label("Rotate mode");
                                ui.end_row();
                                ui.label("S");
                                ui.label("Scale mode");
                                ui.end_row();
                                ui.label("Escape");
                                ui.label("Deselect / cancel");
                                ui.end_row();
                            });
                    });

                    ui.add_space(12.0);
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                self.save_preferences();
                                self.show_settings_dialog = false;
                            }
                        });
                    });
                });
        }

        // Phase 10: Confirm dialog for new scene when dirty
        if self.show_new_confirm_dialog {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes. Create new scene anyway?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Discard Changes").clicked() {
                            self.show_new_confirm_dialog = false;
                            self.create_new_scene();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_confirm_dialog = false;
                        }
                    });
                });
        }

        // Phase 10: Confirm dialog for new scene when dirty
        if self.show_new_confirm_dialog {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes. Create new scene anyway?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Discard Changes").clicked() {
                            self.show_new_confirm_dialog = false;
                            self.create_new_scene();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_confirm_dialog = false;
                        }
                    });
                });
        }

        // Check for close request
        ctx.input(|i| {
            if i.viewport().close_requested() && self.is_dirty && !self.show_quit_dialog {
                self.show_quit_dialog = true;
            }
        });

        // Phase 2.1 & 2.2: Global hotkeys for undo/redo and scene save/load
        ctx.input(|i| {
            // Ctrl+Z: Undo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) && !i.modifiers.shift {
                if let Some(scene_state) = self.scene_state.as_mut() {
                    let undo_error = self.undo_stack.undo(scene_state.world_mut()).err();

                    if let Some(e) = undo_error {
                        self.console_logs.push(format!("Undo failed: {}", e));
                    } else if let Some(desc) = self.undo_stack.redo_description() {
                        self.status = format!("Undid: {}", desc);
                        self.console_logs.push(format!("Undo: {}", desc));
                        self.is_dirty = true;
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
                        self.console_logs.push(format!("Redo failed: {}", e));
                    } else if let Some(desc) = self.undo_stack.undo_description() {
                        self.status = format!("Redid: {}", desc);
                        self.console_logs.push(format!("Redo: {}", desc));
                        self.is_dirty = true;
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
                            self.status = format!("Saved scene to {:?}", path);
                            self.console_logs.push(format!("Scene saved: {:?}", path));
                            self.last_auto_save = std::time::Instant::now();
                            self.is_dirty = false;
                            self.toasts
                                .push(Toast::new("Scene saved successfully", ToastLevel::Success));
                        }
                        Err(e) => {
                            self.status = format!("Scene save failed: {}", e);
                            self.console_logs
                                .push(format!("Failed to save scene: {}", e));
                            self.toasts
                                .push(Toast::new(format!("Save failed: {}", e), ToastLevel::Error));
                        }
                    }
                } else {
                    self.console_logs.push("No world to save".into());
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
            if i.key_pressed(egui::Key::Delete) && self.editor_mode.can_edit() {
                if let Some(scene_state) = self.scene_state.as_mut() {
                    let selected = self.hierarchy_panel.get_all_selected();
                    if !selected.is_empty() {
                        let cmd = command::DeleteEntitiesCommand::new(selected.clone());
                        let delete_result = self.undo_stack.execute(cmd, scene_state.world_mut());

                        match delete_result {
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

            // Ctrl+N: New Scene
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) && !i.modifiers.shift {
                if self.is_dirty {
                    self.show_new_confirm_dialog = true;
                } else {
                    self.create_new_scene();
                }
            }

            // Ctrl+Shift+N: New Entity
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::N) {
                if let Some(scene_state) = self.scene_state.as_mut() {
                    let world = scene_state.world_mut();
                    let entity_id = world.spawn(
                        "New Entity",
                        astraweave_core::IVec2 { x: 0, y: 0 },
                        astraweave_core::Team { id: 0 },
                        0,
                        0,
                    );
                    self.selected_entity = Some(entity_id as u64);
                    self.hierarchy_panel.set_selected(Some(entity_id));
                    self.is_dirty = true;
                    self.status = format!("Created entity {}", entity_id);
                    self.toast_success("New entity created");
                }
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
                            self.console_logs
                                .push(format!("✅ Scene saved as: {:?}", path));
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

            // F1: Show keyboard shortcuts help
            if i.key_pressed(egui::Key::F1) {
                self.show_help_dialog = !self.show_help_dialog;
            }

            // G: Toggle grid visibility
            if i.key_pressed(egui::Key::G) && !i.modifiers.ctrl {
                self.show_grid = !self.show_grid;
                self.status = if self.show_grid {
                    "Grid enabled".to_string()
                } else {
                    "Grid disabled".to_string()
                };
            }

            // Escape: Close dialogs
            if i.key_pressed(egui::Key::Escape) {
                if self.show_new_confirm_dialog {
                    self.show_new_confirm_dialog = false;
                } else if self.show_settings_dialog {
                    self.save_preferences();
                    self.show_settings_dialog = false;
                } else if self.show_help_dialog {
                    self.show_help_dialog = false;
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
                        self.last_auto_save = std::time::Instant::now();
                    }
                    Err(e) => {
                        self.console_logs
                            .push(format!("⚠️  Autosave failed: {}", e));
                        self.last_auto_save = std::time::Instant::now();
                    }
                }
            }
        }

        let stats = self.runtime.stats().clone();
        self.performance_panel.set_frame_time(frame_time * 1000.0);
        if !self.editor_mode.is_editing() {
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
        self.world_panel.update();
        self.animation_panel.update(frame_time);

        egui::TopBottomPanel::top("top")
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.heading("AstraWeave Level & Encounter Editor");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    if self.is_dirty {
                        self.show_new_confirm_dialog = true;
                    } else {
                        self.create_new_scene();
                    }
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
                                self.is_dirty = false;
                                self.status = format!("💾 Saved scene to {:?}", path);
                                self.toast_success(format!("Saved scene: {:?}", path.file_name().unwrap_or_default()));
                                self.console_logs
                                    .push(format!("✅ Scene saved: {:?}", path));
                                self.last_auto_save = std::time::Instant::now();
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

                ui.menu_button("👁 View", |ui| {
                    if ui.checkbox(&mut self.show_hierarchy_panel, "Hierarchy Panel").changed() {
                        let state = if self.show_hierarchy_panel { "shown" } else { "hidden" };
                        self.status = format!("Hierarchy panel {}", state);
                    }
                    if ui.checkbox(&mut self.show_inspector_panel, "Inspector Panel").changed() {
                        let state = if self.show_inspector_panel { "shown" } else { "hidden" };
                        self.status = format!("Inspector panel {}", state);
                    }
                    if ui.checkbox(&mut self.show_console_panel, "Console Panel").changed() {
                        let state = if self.show_console_panel { "shown" } else { "hidden" };
                        self.status = format!("Console panel {}", state);
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.show_grid, "Grid").changed() {
                        let state = if self.show_grid { "enabled" } else { "disabled" };
                        self.status = format!("Grid {}", state);
                    }
                });

                // Window menu for docking layouts
                ui.menu_button("🪟 Window", |ui| {
                    // Toggle between legacy and docking UI
                    if ui.checkbox(&mut self.use_docking, "📐 Use Docking Layout").changed() {
                        let mode = if self.use_docking { "Docking" } else { "Legacy" };
                        self.status = format!("Switched to {} layout mode", mode);
                    }

                    ui.separator();
                    ui.label("Layout Presets:");

                    if ui.button("📊 Default").clicked() {
                        self.dock_layout = DockLayout::from_preset(LayoutPreset::Default);
                        self.status = "Applied Default layout".to_string();
                        ui.close();
                    }
                    if ui.button("📐 Wide").clicked() {
                        self.dock_layout = DockLayout::from_preset(LayoutPreset::Wide);
                        self.status = "Applied Wide layout".to_string();
                        ui.close();
                    }
                    if ui.button("📦 Compact").clicked() {
                        self.dock_layout = DockLayout::from_preset(LayoutPreset::Compact);
                        self.status = "Applied Compact layout".to_string();
                        ui.close();
                    }
                    if ui.button("🎨 Modeling").clicked() {
                        self.dock_layout = DockLayout::from_preset(LayoutPreset::Modeling);
                        self.status = "Applied Modeling layout".to_string();
                        ui.close();
                    }
                    if ui.button("🎬 Animation").clicked() {
                        self.dock_layout = DockLayout::from_preset(LayoutPreset::Animation);
                        self.status = "Applied Animation layout".to_string();
                        ui.close();
                    }
                    if ui.button("🐛 Debug").clicked() {
                        self.dock_layout = DockLayout::from_preset(LayoutPreset::Debug);
                        self.status = "Applied Debug layout".to_string();
                        ui.close();
                    }

                    ui.separator();
                    ui.label("Panels:");

                    // Show available panels to add
                    for &panel_type in PanelType::all() {
                        let is_visible = self.dock_layout.has_panel(&panel_type);
                        let label = format!("{} {}", panel_type.icon(), panel_type.title());
                        if ui.selectable_label(is_visible, label).clicked() {
                            if is_visible {
                                self.dock_layout.remove_panel(&panel_type);
                                self.status = format!("Closed {} panel", panel_type.title());
                            } else {
                                self.dock_layout.add_panel(panel_type);
                                self.status = format!("Opened {} panel", panel_type.title());
                            }
                        }
                    }
                });

                if ui.button("⚙ Settings").clicked() {
                    self.show_settings_dialog = true;
                }

                // Debug menu for testing engine features
                ui.menu_button("🐛 Debug", |ui| {
                    ui.label("🎨 Viewport Tests:");

                    #[cfg(feature = "astraweave-render")]
                    {
                        // Test glTF loading with a sample model
                        if ui.button("📦 Load Test Model (barrels.glb)").clicked() {
                            let test_path = PathBuf::from("assets/models/barrels.glb");
                            if test_path.exists() {
                                if let Some(viewport) = &self.viewport {
                                    match viewport.load_gltf_model("test_barrels", &test_path) {
                                        Ok(()) => {
                                            self.toast_success("Test model loaded successfully!");
                                            self.console_logs.push("✅ Loaded test model: barrels.glb".into());
                                            self.status = "Test model loaded - PBR rendering enabled".into();
                                        }
                                        Err(e) => {
                                            self.toast_error(format!("Model load failed: {}", e));
                                            self.console_logs.push(format!("❌ Test model failed: {}", e));
                                            self.status = format!("Model load error: {}", e);
                                        }
                                    }
                                } else {
                                    self.console_logs.push("⚠️ Viewport not initialized".into());
                                }
                            } else {
                                self.console_logs.push(format!("⚠️ Test model not found: {:?}", test_path));
                            }
                            ui.close();
                        }

                        // Load bed model (another simple test)
                        if ui.button("🛏️ Load Test Model (bed.glb)").clicked() {
                            let test_path = PathBuf::from("assets/models/bed.glb");
                            if test_path.exists() {
                                if let Some(viewport) = &self.viewport {
                                    match viewport.load_gltf_model("test_bed", &test_path) {
                                        Ok(()) => {
                                            self.toast_success("Bed model loaded!");
                                            self.console_logs.push("✅ Loaded test model: bed.glb".into());
                                        }
                                        Err(e) => {
                                            self.console_logs.push(format!("❌ bed.glb failed: {}", e));
                                        }
                                    }
                                }
                            } else {
                                self.console_logs.push("⚠️ bed.glb not found".into());
                            }
                            ui.close();
                        }

                        // Load pine tree - check local first, then external locations
                        if ui.button("🌲 Load Pine Tree").clicked() {
                            // Try multiple possible paths - local first, then external
                            let possible_paths = [
                                PathBuf::from("assets/models/pine_tree_01_1k.glb"),  // Local copy
                                PathBuf::from("../pine_forest/pine_tree_01_1k.glb"),
                                PathBuf::from("../../Downloads/pine_forest/pine_tree_01_1k.glb"),
                            ];
                            let mut loaded = false;
                            for test_path in &possible_paths {
                                if test_path.exists() {
                                    if let Some(viewport) = &self.viewport {
                                        match viewport.load_gltf_model("pine_tree", test_path) {
                                            Ok(()) => {
                                                self.toast_success("Pine tree loaded!");
                                                self.console_logs.push(format!(
                                                    "✅ Loaded pine tree from: {:?}",
                                                    test_path
                                                ));
                                                loaded = true;
                                                break;
                                            }
                                            Err(e) => {
                                                self.console_logs.push(format!(
                                                    "❌ Pine tree failed: {} (from {:?})",
                                                    e, test_path
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                            if !loaded {
                                self.console_logs.push("⚠️ Pine tree not found in any expected location".into());
                            }
                            ui.close();
                        }

                        // Toggle engine rendering
                        if ui.button("🔄 Toggle Engine Rendering").clicked() {
                            if let Some(viewport) = &self.viewport {
                                if let Ok(mut renderer) = viewport.renderer().lock() {
                                    let current = renderer.use_engine_rendering();
                                    renderer.set_use_engine_rendering(!current);
                                    let state = if !current { "enabled" } else { "disabled" };
                                    self.console_logs.push(format!("🎨 Engine rendering {}", state));
                                    self.status = format!("Engine rendering {}", state);
                                }
                            }
                            ui.close();
                        }
                    }

                    #[cfg(not(feature = "astraweave-render"))]
                    {
                        ui.label("⚠️ astraweave-render feature not enabled");
                    }

                    ui.separator();
                    ui.label("📊 Diagnostics:");

                    if ui.button("📋 Show Engine Info").clicked() {
                        if let Some(viewport) = &self.viewport {
                            if let Ok(renderer) = viewport.renderer().lock() {
                                let engine_active = renderer.use_engine_rendering();
                                let adapter_init = renderer.engine_adapter_initialized();
                                self.console_logs.push(format!(
                                    "🎮 Engine Status:\n  - Engine Rendering: {}\n  - Adapter Initialized: {}",
                                    engine_active, adapter_init
                                ));
                            }
                        }
                        ui.close();
                    }

                    ui.separator();
                    ui.label("🎨 Material Testing:");

                    if ui.button("🔴 Red Material").clicked() {
                        if let Some(viewport) = &self.viewport {
                            if let Err(e) = viewport.set_material_params([1.0, 0.2, 0.2, 1.0], 0.0, 0.5) {
                                self.console_logs.push(format!("⚠️ Material error: {}", e));
                            } else {
                                self.console_logs.push("🔴 Applied red material".into());
                            }
                        }
                        ui.close();
                    }

                    if ui.button("🟢 Green Metallic").clicked() {
                        if let Some(viewport) = &self.viewport {
                            if let Err(e) = viewport.set_material_params([0.2, 0.8, 0.2, 1.0], 0.9, 0.3) {
                                self.console_logs.push(format!("⚠️ Material error: {}", e));
                            } else {
                                self.console_logs.push("🟢 Applied green metallic".into());
                            }
                        }
                        ui.close();
                    }

                    if ui.button("🔵 Blue Rough").clicked() {
                        if let Some(viewport) = &self.viewport {
                            if let Err(e) = viewport.set_material_params([0.2, 0.3, 0.9, 1.0], 0.1, 0.9) {
                                self.console_logs.push(format!("⚠️ Material error: {}", e));
                            } else {
                                self.console_logs.push("🔵 Applied blue rough".into());
                            }
                        }
                        ui.close();
                    }

                    if ui.button("⬜ White Default").clicked() {
                        if let Some(viewport) = &self.viewport {
                            if let Err(e) = viewport.set_material_params([1.0, 1.0, 1.0, 1.0], 0.0, 0.5) {
                                self.console_logs.push(format!("⚠️ Material error: {}", e));
                            } else {
                                self.console_logs.push("⬜ Applied white default".into());
                            }
                        }
                        ui.close();
                    }

                    ui.separator();
                    ui.label("☀️ Lighting / Time of Day:");

                    // Time presets row
                    ui.horizontal(|ui| {
                        if ui.button("🌅 Dawn (6:00)").clicked() {
                            if let Some(viewport) = &self.viewport {
                                if let Err(e) = viewport.set_time_of_day(6.0) {
                                    self.console_logs.push(format!("⚠️ Lighting error: {}", e));
                                } else {
                                    self.console_logs.push("🌅 Set time to dawn (6:00)".into());
                                }
                            }
                        }
                        if ui.button("☀️ Noon (12:00)").clicked() {
                            if let Some(viewport) = &self.viewport {
                                if let Err(e) = viewport.set_time_of_day(12.0) {
                                    self.console_logs.push(format!("⚠️ Lighting error: {}", e));
                                } else {
                                    self.console_logs.push("☀️ Set time to noon (12:00)".into());
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("🌇 Sunset (18:00)").clicked() {
                            if let Some(viewport) = &self.viewport {
                                if let Err(e) = viewport.set_time_of_day(18.0) {
                                    self.console_logs.push(format!("⚠️ Lighting error: {}", e));
                                } else {
                                    self.console_logs.push("🌇 Set time to sunset (18:00)".into());
                                }
                            }
                        }
                        if ui.button("🌙 Midnight (0:00)").clicked() {
                            if let Some(viewport) = &self.viewport {
                                if let Err(e) = viewport.set_time_of_day(0.0) {
                                    self.console_logs.push(format!("⚠️ Lighting error: {}", e));
                                } else {
                                    self.console_logs.push("🌙 Set time to midnight (0:00)".into());
                                }
                            }
                        }
                    });

                    // Show current time
                    if let Some(viewport) = &self.viewport {
                        if let Ok(time) = viewport.get_time_of_day() {
                            let period = viewport.get_time_period().unwrap_or("Unknown");
                            let hours = time.floor() as u32;
                            let minutes = ((time - time.floor()) * 60.0) as u32;
                            ui.label(format!("🕐 Current: {:02}:{:02} ({})", hours, minutes, period));
                        }
                    }

                    // Shadow toggle
                    ui.horizontal(|ui| {
                        let shadows_on = self.viewport.as_ref()
                            .and_then(|v| v.shadows_enabled().ok())
                            .unwrap_or(true);
                        let shadow_label = if shadows_on { "🔦 Shadows: ON" } else { "🔦 Shadows: OFF" };
                        if ui.button(shadow_label).clicked() {
                            if let Some(viewport) = &self.viewport {
                                if let Err(e) = viewport.set_shadows_enabled(!shadows_on) {
                                    self.console_logs.push(format!("⚠️ Shadow error: {}", e));
                                } else {
                                    let status = if !shadows_on { "enabled" } else { "disabled" };
                                    self.console_logs.push(format!("🔦 Shadows {}", status));
                                }
                            }
                        }
                    });

                    ui.separator();
                    ui.label("📁 Model Discovery:");

                    if ui.button("📁 Scan For Models").clicked() {
                        let scan_dirs = [
                            ("Local", PathBuf::from("assets/models")),
                            ("Pine Forest", PathBuf::from("../pine_forest")),
                            ("Downloads PF", PathBuf::from("../../Downloads/pine_forest")),
                        ];
                        let mut found_any = false;
                        for (name, dir) in &scan_dirs {
                            if dir.exists() {
                                if let Ok(entries) = std::fs::read_dir(dir) {
                                    let glb_files: Vec<_> = entries
                                        .filter_map(|e| e.ok())
                                        .filter(|e| {
                                            e.path().extension().map_or(false, |ext| {
                                                ext == "glb" || ext == "gltf"
                                            })
                                        })
                                        .take(8)
                                        .collect();
                                    if !glb_files.is_empty() {
                                        found_any = true;
                                        self.console_logs.push(format!("📁 {} ({}):", name, glb_files.len()));
                                        for entry in glb_files {
                                            self.console_logs.push(format!("  • {}", entry.file_name().to_string_lossy()));
                                        }
                                    }
                                }
                            }
                        }
                        if !found_any {
                            self.console_logs.push("⚠️ No glTF/glb models found in any scanned directory".into());
                        }
                        ui.close();
                    }

                    if ui.button("🗑️ Clear Console").clicked() {
                        self.console_logs.clear();
                        ui.close();
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

            // Compact performance indicator in header
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let frame_time = self.runtime.stats().frame_time_ms;
                let fps_color = if self.current_fps >= 55.0 {
                    egui::Color32::from_rgb(100, 255, 100) // Green
                } else if self.current_fps >= 30.0 {
                    egui::Color32::from_rgb(255, 200, 100) // Yellow
                } else {
                    egui::Color32::from_rgb(255, 100, 100) // Red
                };

                ui.label(egui::RichText::new(format!("FPS: {:.0}", self.current_fps))
                    .color(fps_color)
                    .strong());
                ui.separator();
                ui.label(egui::RichText::new(format!("{:.1}ms", frame_time))
                    .color(egui::Color32::from_gray(180)));
                ui.separator();
                ui.label(egui::RichText::new("⚡")
                    .color(fps_color));
            });
        });

        // BOTTOM PANEL - StatusBar (Phase 3.5 & 4) - Moved before SidePanel for standard layout order
        let bottom_entity_count = self
            .scene_state
            .as_ref()
            .map(|s| s.world().entities().len())
            .unwrap_or(0);

        let bottom_scene_path_str = self.current_scene_path.as_ref().and_then(|p| p.to_str());

        egui::TopBottomPanel::bottom("status_bar")
            .min_height(24.0)
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                StatusBar::show(
                    ui,
                    &self.editor_mode,
                    &self.current_gizmo_mode,
                    &self.selection_set,
                    &self.undo_stack,
                    &self.snapping_config,
                    self.current_fps,
                    self.is_dirty,
                    bottom_entity_count,
                    bottom_scene_path_str,
                );
            });

        // LEFT PANEL - Only show in legacy mode (pruned for docking)
        if !self.use_docking {
            egui::SidePanel::left("astract_left_panel")
                .resizable(true)
                .min_width(250.0)
                .frame(egui::Frame::NONE.inner_margin(0.0))
                .show(ctx, |ui| {
                    ui.set_min_size(ui.available_size());

                    ui.vertical(|ui| {
                        ui.set_min_size(ui.available_size());

                        ui.heading("📋 Hierarchy");
                        ui.add_space(4.0);

                        // Search bar
                        ui.horizontal(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label("🔍");
                            ui.text_edit_singleline(&mut self.level.title);
                        });
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_min_size(ui.available_size());
                                self.show_scene_hierarchy(ui);
                            });
                    });
                });
        }

        // Performance indicator was moved to header - see show_play_controls

        // Render main content area - either docking layout or legacy panels
        if self.use_docking {
            // Phase 11: Professional Docking System
            // Sync selected entity to tab viewer
            self.dock_tab_viewer
                .set_selected_entity(self.selected_entity);
            self.dock_tab_viewer
                .set_is_playing(!self.editor_mode.is_editing());
            self.dock_tab_viewer.begin_frame();

            // Sync entity list for hierarchy panel
            let entity_list: Vec<EntityInfo> = self
                .entity_manager
                .entities()
                .iter()
                .map(|(id, entity)| EntityInfo {
                    id: *id,
                    name: entity.name.clone(),
                    components: entity.components.keys().cloned().collect(),
                    entity_type: if entity.components.contains_key("Camera") {
                        "Camera".to_string()
                    } else if entity.components.contains_key("Light") {
                        "Light".to_string()
                    } else if entity.components.contains_key("Mesh") {
                        "Mesh".to_string()
                    } else {
                        "Entity".to_string()
                    },
                })
                .collect();
            self.dock_tab_viewer.set_entity_list(entity_list);

            // Sync selected entity transform for inspector
            if let Some(entity_id) = self.selected_entity {
                if let Some(entity) = self.entity_manager.get(entity_id) {
                    let (pos, rot, scale) = entity.transform();
                    // Convert to 2D-like format: x, y, rotation_z, scale_x, scale_y
                    let angle = rot.to_euler(glam::EulerRot::ZXY).0;
                    self.dock_tab_viewer
                        .set_selected_transform(Some((pos.x, pos.y, angle, scale.x, scale.y)));
                    let entity_type = if entity.components.contains_key("Camera") {
                        "Camera".to_string()
                    } else if entity.components.contains_key("Light") {
                        "Light".to_string()
                    } else if entity.components.contains_key("Mesh") {
                        "Mesh".to_string()
                    } else {
                        "Entity".to_string()
                    };
                    self.dock_tab_viewer
                        .set_selected_entity_info(Some(EntityInfo {
                            id: entity_id,
                            name: entity.name.clone(),
                            components: entity.components.keys().cloned().collect(),
                            entity_type,
                        }));
                } else {
                    self.dock_tab_viewer.set_selected_transform(None);
                    self.dock_tab_viewer.set_selected_entity_info(None);
                }
            } else {
                self.dock_tab_viewer.set_selected_transform(None);
                self.dock_tab_viewer.set_selected_entity_info(None);
            }

            // Sync console logs
            self.dock_tab_viewer
                .set_console_logs(self.console_logs.clone());

            // Sync runtime stats for profiler panel
            let runtime_stats = tab_viewer::RuntimeStatsInfo {
                frame_time_ms: self.runtime.stats().frame_time_ms,
                fps: self.current_fps,
                entity_count: self.entity_manager.entities().len(),
                tick_count: self.runtime.stats().tick_count,
                is_playing: self.runtime.is_playing(),
                is_paused: self.runtime.is_paused(),
                // Subsystem timing (placeholder values - would come from actual profiling)
                render_time_ms: self.runtime.stats().frame_time_ms * 0.4, // ~40% for render
                physics_time_ms: self.runtime.stats().frame_time_ms * 0.15, // ~15% for physics
                ai_time_ms: self.runtime.stats().frame_time_ms * 0.1,     // ~10% for AI
                script_time_ms: self.runtime.stats().frame_time_ms * 0.05, // ~5% for scripts
                audio_time_ms: self.runtime.stats().frame_time_ms * 0.02, // ~2% for audio
                draw_calls: 150 + (self.entity_manager.entities().len() * 2), // Estimate
                triangles: 50000 + (self.entity_manager.entities().len() * 1000), // Estimate
                gpu_memory_bytes: 256 * 1024 * 1024,                      // 256 MB placeholder
            };
            self.dock_tab_viewer.set_runtime_stats(runtime_stats);

            // Sync scene stats
            let total_components: usize = self
                .entity_manager
                .entities()
                .values()
                .map(|e| e.components.len())
                .sum();
            let entity_count = self.entity_manager.entities().len();
            let scene_stats = tab_viewer::SceneStatsInfo {
                total_entities: entity_count,
                total_components,
                prefab_instances: 0, // Would count prefab instances
                selected_count: self.selection_set.count(),
                memory_usage_bytes: entity_count * 1024 + total_components * 256, // Rough estimate
                active_systems: 12, // Typical system count
                loaded_assets: self.asset_registry.count(),
                light_count: entity_count / 10, // Estimate ~10% are lights
                mesh_count: entity_count / 2,   // Estimate ~50% have meshes
                physics_bodies: entity_count / 4, // Estimate ~25% have physics
                is_modified: self.is_scene_modified,
                audio_sources: entity_count / 20, // Estimate ~5% have audio
                particle_systems: entity_count / 30, // Estimate ~3% are particles
                camera_count: 1 + (entity_count / 50), // At least 1 camera
                collider_count: entity_count / 3, // Estimate ~33% have colliders
                script_count: entity_count / 2,   // Estimate ~50% have scripts
                ui_element_count: 0,              // Would count UI elements
                scene_path: self
                    .current_scene_path
                    .as_ref()
                    .map(|p| p.display().to_string()),
                last_save_time: self.last_save_time.clone(),
            };
            self.dock_tab_viewer.set_scene_stats(scene_stats);

            // Sync undo/redo counts
            self.dock_tab_viewer.set_undo_redo_counts(
                self.undo_stack.len(),
                0, // Redo count would come from redo stack
            );

            // Update frame time history for profiler graph
            self.dock_tab_viewer
                .push_frame_time(self.runtime.stats().frame_time_ms);

            // Render the docking layout with EditorDrawContext for viewport integration
            // We need to carefully structure borrows to avoid conflicts
            // Use CentralPanel with no frame to render dock in remaining space (after side panels)
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(0.0))
                .show(ctx, |ui| {
                    ui.set_min_size(ui.available_size());

                    // Get mutable world from scene state
                    let world_opt = self.scene_state.as_mut().map(|s| s.world_mut());

                    // Unified context rendering to avoid type-switching issues
                    let mut context = EditorDrawContext::new(&mut self.dock_tab_viewer);

                    if let (Some(world), Some(viewport)) = (world_opt, self.viewport.as_mut()) {
                        context = context
                            .with_viewport(viewport)
                            .with_world(world)
                            .with_entity_manager(&mut self.entity_manager)
                            .with_undo_stack(&mut self.undo_stack)
                            .with_prefab_manager(&mut self.prefab_manager);
                    }

                    self.dock_layout.show_inside(ui, &mut context);
                });

            // Check for transform changes and emit events
            self.dock_tab_viewer.check_transform_changes();

            // Handle panel close events (separate from PanelEvent for backward compatibility)
            for panel in self.dock_tab_viewer.take_closed_panels() {
                self.status = format!("Closed {} panel", panel.title());
            }
            for panel in self.dock_tab_viewer.take_panels_to_add() {
                self.dock_layout.add_panel(panel);
                self.status = format!("Added {} panel", panel.title());
            }
            self.dock_tab_viewer.check_transform_changes();

            // Handle panel events from the tab viewer
            for event in self.dock_tab_viewer.take_events() {
                match event {
                    tab_viewer::PanelEvent::EntitySelected(entity_id) => {
                        self.selected_entity = Some(entity_id);
                        self.selection_set.primary = Some(entity_id);
                        self.status = format!("Selected entity {}", entity_id);
                    }
                    tab_viewer::PanelEvent::EntityDeselected => {
                        self.selected_entity = None;
                        self.selection_set.primary = None;
                        self.status = "Deselected entity".to_string();
                    }
                    tab_viewer::PanelEvent::TransformPositionChanged { entity_id, x, y } => {
                        // Update entity transform in scene
                        self.status =
                            format!("Entity {} position: ({:.2}, {:.2})", entity_id, x, y);
                        // TODO: Update actual entity transform in scene graph when available
                    }
                    tab_viewer::PanelEvent::TransformRotationChanged {
                        entity_id,
                        rotation,
                    } => {
                        self.status = format!("Entity {} rotation: {:.1}°", entity_id, rotation);
                        // TODO: Update actual entity rotation in scene graph when available
                    }
                    tab_viewer::PanelEvent::TransformScaleChanged {
                        entity_id,
                        scale_x,
                        scale_y,
                    } => {
                        self.status = format!(
                            "Entity {} scale: ({:.2}, {:.2})",
                            entity_id, scale_x, scale_y
                        );
                        // TODO: Update actual entity scale in scene graph when available
                    }
                    tab_viewer::PanelEvent::CreateEntity => {
                        // Create a new entity with a unique ID
                        let new_id = self.next_entity_id;
                        self.next_entity_id += 1;
                        let new_entity = tab_viewer::EntityInfo {
                            id: new_id,
                            name: format!("Entity_{}", new_id),
                            entity_type: "Empty".to_string(),
                            components: vec![],
                        };
                        self.dock_tab_viewer.add_entity(new_entity);
                        self.selected_entity = Some(new_id);
                        self.selection_set.primary = Some(new_id);
                        self.status = format!("Created entity {}", new_id);
                    }
                    tab_viewer::PanelEvent::DeleteEntity(entity_id) => {
                        // Remove entity from list
                        self.dock_tab_viewer.remove_entity(entity_id);
                        if self.selected_entity == Some(entity_id) {
                            self.selected_entity = None;
                            self.selection_set.primary = None;
                        }
                        self.status = format!("Deleted entity {}", entity_id);
                    }
                    tab_viewer::PanelEvent::DuplicateEntity(entity_id) => {
                        // Find and duplicate the entity
                        let source_info = self.dock_tab_viewer.find_entity(entity_id).cloned();
                        if let Some(source) = source_info {
                            let new_id = self.next_entity_id;
                            self.next_entity_id += 1;
                            let new_entity = tab_viewer::EntityInfo {
                                id: new_id,
                                name: format!("{}_copy", source.name),
                                entity_type: source.entity_type.clone(),
                                components: source.components.clone(),
                            };
                            self.dock_tab_viewer.add_entity(new_entity);
                            self.selected_entity = Some(new_id);
                            self.selection_set.primary = Some(new_id);
                            self.status = format!("Duplicated entity {} as {}", entity_id, new_id);
                        }
                    }
                    tab_viewer::PanelEvent::PanelClosed(panel) => {
                        self.status = format!("Closed {} panel", panel.title());
                    }
                    tab_viewer::PanelEvent::PanelFocused(panel) => {
                        self.status = format!("Focused {} panel", panel.title());
                    }
                    tab_viewer::PanelEvent::AddPanel(panel) => {
                        self.dock_layout.add_panel(panel);
                        self.status = format!("Added {} panel", panel.title());
                    }
                    tab_viewer::PanelEvent::MaterialChanged {
                        name,
                        property,
                        value,
                    } => {
                        self.status = format!("Material '{}': {} = {:.2}", name, property, value);
                    }
                    tab_viewer::PanelEvent::AnimationPlayStateChanged { is_playing } => {
                        if is_playing {
                            self.status = "Animation playing".to_string();
                        } else {
                            self.status = "Animation paused".to_string();
                        }
                    }
                    tab_viewer::PanelEvent::AnimationFrameChanged { frame } => {
                        self.status = format!("Animation frame: {}", frame);
                    }
                    tab_viewer::PanelEvent::AnimationKeyframeAdded {
                        track_index,
                        frame,
                        value,
                    } => {
                        self.status = format!(
                            "Added keyframe at frame {} (track {}, value {:.2})",
                            frame, track_index, value
                        );
                    }
                    tab_viewer::PanelEvent::ThemeChanged(theme) => {
                        self.status = format!("Theme changed to {:?}", theme);
                    }
                    tab_viewer::PanelEvent::BuildRequested { target, profile } => {
                        self.status = format!("Build requested: {} ({})", target, profile);
                    }
                    tab_viewer::PanelEvent::ConsoleCleared => {
                        self.status = "Console cleared".to_string();
                    }
                    tab_viewer::PanelEvent::AssetSelected(asset) => {
                        self.status = format!("Selected asset: {}", asset);
                    }
                    tab_viewer::PanelEvent::BehaviorNodeSelected(node_id) => {
                        self.status = format!("Selected behavior node: {}", node_id);
                    }
                    tab_viewer::PanelEvent::GraphNodeSelected(node_id) => {
                        self.status = format!("Selected graph node: {}", node_id);
                    }
                    tab_viewer::PanelEvent::HierarchySearchChanged(search) => {
                        self.status = format!("Hierarchy search: {}", search);
                    }
                    tab_viewer::PanelEvent::ConsoleSearchChanged(search) => {
                        self.status = format!("Console search: {}", search);
                    }
                    tab_viewer::PanelEvent::RefreshSceneStats => {
                        self.status = "Refreshing scene statistics...".to_string();
                    }
                    tab_viewer::PanelEvent::AddComponent {
                        entity_id,
                        component_type,
                    } => {
                        self.status = format!("Adding {} to entity {}", component_type, entity_id);
                        // Would add component to entity in actual implementation
                    }
                    tab_viewer::PanelEvent::RemoveComponent {
                        entity_id,
                        component_type,
                    } => {
                        self.status =
                            format!("Removing {} from entity {}", component_type, entity_id);
                        // Would remove component from entity in actual implementation
                    }
                    tab_viewer::PanelEvent::ViewportViewModeChanged(mode) => {
                        let mode_names = ["Shaded", "Wireframe", "Unlit", "Normals", "UVs"];
                        self.status = format!(
                            "Viewport view mode: {}",
                            mode_names.get(mode).unwrap_or(&"Unknown")
                        );
                    }
                    tab_viewer::PanelEvent::ViewportGizmoModeChanged(mode) => {
                        let mode_names = ["Translate", "Rotate", "Scale"];
                        self.status =
                            format!("Gizmo mode: {}", mode_names.get(mode).unwrap_or(&"Unknown"));
                    }
                    tab_viewer::PanelEvent::ViewportGizmoSpaceChanged(space) => {
                        self.status = format!(
                            "Gizmo space: {}",
                            if space == 0 { "Local" } else { "World" }
                        );
                    }
                    tab_viewer::PanelEvent::ViewportOverlayToggled { overlay, enabled } => {
                        self.status = format!(
                            "Viewport overlay '{}': {}",
                            overlay,
                            if enabled { "enabled" } else { "disabled" }
                        );
                    }
                    tab_viewer::PanelEvent::ViewportCameraChanged {
                        fov,
                        near,
                        far,
                        speed,
                    } => {
                        self.status = format!(
                            "Camera: FOV={:.0}°, Clip={:.2}-{:.0}, Speed={:.1}",
                            fov, near, far, speed
                        );
                    }
                    tab_viewer::PanelEvent::ViewportFocusOnSelection => {
                        self.status = "Focusing on selection...".to_string();
                    }
                    tab_viewer::PanelEvent::ViewportResetCamera => {
                        self.status = "Camera reset to default position".to_string();
                    }
                    tab_viewer::PanelEvent::ViewportCameraPreset(preset) => {
                        self.status = format!("Camera preset applied: {}", preset);
                    }
                    tab_viewer::PanelEvent::ResetLayout => {
                        // Reset dock state to default layout
                        self.dock_layout = DockLayout::from_preset(LayoutPreset::Default);
                        self.status = "Layout reset to default".to_string();
                    }
                }
            }
        } else {
            // Legacy layout - original CentralPanel rendering
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(0.0).fill(egui::Color32::from_rgb(0, 255, 0))) // GREEN for legacy
                .show(ctx, |ui| {
                // 3D Viewport (Phase 1.1 - Babylon.js-style editor)
                if let Some(viewport) = &mut self.viewport {
                    // Phase 14: Update viewport HUD with selection count
                    viewport.set_selection_count(self.selection_set.count());

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

                        ui.separator();

                        // Engine PBR Rendering toggle
                        let mut use_pbr = viewport.renderer().lock().map(|r| r.use_engine_rendering()).unwrap_or(false);
                        if ui.checkbox(&mut use_pbr, "🚀 Engine PBR").on_hover_text("Enable full PBR mesh rendering instead of cube placeholders").changed() {
                            if let Ok(mut renderer) = viewport.renderer().lock() {
                                renderer.set_use_engine_rendering(use_pbr);
                            }
                        }
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

                let scene_entity_count = self.active_world().map(|w| w.entities().len()).unwrap_or(0);
                let scene_hier_header = format!("Scene Hierarchy ({} entities)", scene_entity_count);
                ui.collapsing(scene_hier_header, |ui| self.show_scene_hierarchy(ui));
                if self.show_inspector_panel {
                    let inspector_header = if let Some(entity_id) = self.selection_set.primary {
                        if let Ok(entity) = u32::try_from(entity_id) {
                            if let Some(world) = self.active_world() {
                                let name = world.name(entity)
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| format!("Entity_{}", entity));
                                format!("Inspector - {}", name)
                            } else {
                                "Inspector".to_string()
                            }
                        } else {
                            "Inspector".to_string()
                        }
                    } else {
                        "Inspector (no selection)".to_string()
                    };
                    ui.collapsing(inspector_header, |ui| self.show_inspector(ui));
                }

                if self.show_console_panel {
                // Console section with auto-expand when active
                let console_header = format!("Console ({} messages)", self.console_logs.len());
                egui::CollapsingHeader::new(console_header)
                    .default_open(console_open)
                    .show(ui, |ui| self.show_console(ui));
                }

                ui.collapsing("Scene Statistics", |ui| {
                    self.scene_stats_panel.show_inline(ui);
                });

                ui.collapsing("Performance Profiler", |ui| {
                    self.profiler_panel.show(ui);
                });

                let runtime_state = self.runtime.state();
                let tick_count = self.runtime.stats().tick_count;
                let profiler_header = match runtime_state {
                    RuntimeState::Editing => "Profiler [Editing]".to_string(),
                    RuntimeState::Playing => format!("Profiler [Playing - Tick {}]", tick_count),
                    RuntimeState::Paused => format!("Profiler [Paused - Tick {}]", tick_count),
                    RuntimeState::SteppingOneFrame => format!("Profiler [Step - Tick {}]", tick_count),
                };
                ui.collapsing(profiler_header, |ui| self.show_profiler(ui));
                let graph_node_count = self.graph_panel.total_node_count();
                let graph_header = format!("Behavior Graph Editor ({} nodes)", graph_node_count);
                ui.collapsing(graph_header, |ui| {
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
                ui.collapsing("Voxel Editor", |ui| self.show_voxel_editor(ui));
                ui.collapsing("Asset Inspector", |ui| self.show_asset_inspector(ui));
            });
        });
        } // End of else block for legacy layout

        self.render_toasts(ctx);

        if let Err(e) = self.runtime.tick(frame_time) {
            self.console_logs
                .push(format!("Runtime tick failed: {}", e));
        }
    }
}

fn main() -> Result<()> {
    // Initialize observability
    if let Err(e) = astraweave_observability::init_observability(Default::default()) {
        eprintln!("⚠️ Warning: Failed to initialize observability: {}", e);
    }

    // Create content directory if it doesn't exist
    let content_dir = PathBuf::from("content");
    let _ = fs::create_dir_all(&content_dir);
    let _ = fs::create_dir_all(content_dir.join("levels"));
    let _ = fs::create_dir_all(content_dir.join("encounters"));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_title("AstraWeave Level & Encounter Editor"),
        ..Default::default()
    };
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
