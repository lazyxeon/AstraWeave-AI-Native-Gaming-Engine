# AstraWeave Visual Editor: Roadmap to World-Class Status

**Current Date**: November 7, 2025  
**Analysis Basis**: ChatGPT reference + existing codebase assessment  
**Goal**: Transform aw_editor from functional prototype to production-grade visual editor

---

## 📊 Current State Analysis

### ✅ What We Have (Phase 1 Complete)

**Core Infrastructure**:
- ✅ 3D Viewport with wgpu rendering
- ✅ Orbit camera (left drag), pan (middle drag), zoom (scroll)
- ✅ Transform gizmos (Translate/Rotate/Scale with axis constraints)
- ✅ Entity selection system (click-to-select, raycast-based)
- ✅ ECS integration (astraweave-core World API)
- ✅ Material inspector (BRDF preview, PBR properties)
- ✅ Multiple rendering systems (Grid, Skybox, Entity, Gizmo)
- ✅ Real-time viewport rendering (@60 FPS target)

**Gizmo System (Recently Completed)**:
- ✅ G (Translate): FREE mode + X/Z constraints, raycast mouse-follow
- ✅ R (Rotate): XYZ axis rotation (pitch/yaw/roll)
- ✅ S (Scale): Scroll wheel mode, 1% sensitivity
- ✅ Camera orbit restoration after gizmo deselection
- ✅ Axis-constrained transformations with visual feedback
- ✅ Enter/Escape confirm/cancel workflow

**Panel System**:
- ✅ Scene hierarchy (entity tree view - basic)
- ✅ Material inspector (PBR preview, roughness/metallic sliders)
- ✅ Transform panel (position/rotation/scale inputs)
- ✅ Advanced widgets demo (color picker, range slider, tree view)

### ⚠️ What's Missing or Incomplete

**Critical Gaps** (Blocking "world-class" status):
1. **Scene Hierarchy**: No drag-drop parenting, no entity grouping
2. **Inspector**: Not component-based (hardcoded for Pose only)
3. **Asset Browser**: Non-existent (no file manager, no drag-drop)
4. **Undo/Redo**: Missing entirely
5. **Prefab System**: No prefab creation/instantiation
6. **Save/Load**: No scene serialization
7. **Play-in-Editor**: Can't run game logic in viewport
8. **Hot Reload**: No live script/asset updates

**Medium Priority** (Limits productivity):
- Multi-selection (select multiple entities, bulk edit)
- Copy/paste/duplicate entities
- Snapping (grid, angle, vertex)
- Camera bookmarks (save/recall viewpoints)
- Gizmo space toggle (world vs local space)
- Visual scripting or node editor
- Physics debug visualization
- AI navmesh visualization

**Low Priority** (Nice-to-haves):
- Dark/light theme toggle
- Custom layouts (save panel arrangements)
- Plugin system
- Performance profiler in-editor
- Build manager UI

---

## 🗺️ Phased Roadmap

### Phase 2: Foundation Layer (4-6 weeks)

**Priority**: Critical systems that unblock all other features

#### 2.1: Undo/Redo System (Week 1-2)
**Why First**: Every future feature depends on this (drag-drop, transform edits, etc.)

**Implementation**:
```rust
// Command pattern with undo stack
pub trait EditorCommand {
    fn execute(&mut self, world: &mut World) -> Result<()>;
    fn undo(&mut self, world: &mut World) -> Result<()>;
    fn describe(&self) -> String; // For undo menu
}

pub struct UndoStack {
    commands: Vec<Box<dyn EditorCommand>>,
    cursor: usize, // Current position in history
    max_size: usize, // Memory limit (e.g., 100 commands)
}

// Example commands:
struct MoveEntityCommand { entity_id, old_pos, new_pos }
struct RotateEntityCommand { entity_id, old_rot, new_rot }
struct DeleteEntityCommand { entity_id, snapshot: EntitySnapshot }
```

**Integration Points**:
- Gizmo transforms → wrap in commands
- Inspector edits → wrap in commands
- Hierarchy drag-drop → wrap in commands
- Hotkeys: Ctrl+Z (undo), Ctrl+Y (redo)

**Success Criteria**:
- 100-command history
- Undo/redo works for all transform operations
- Status bar shows "Undo: Move Entity (G)" tooltip
- Memory-safe (old snapshots cleaned up)

---

#### 2.2: Scene Serialization (Week 2-3)
**Why Second**: Enables save/load, prefabs, and copy/paste

**File Format** (RON - Rusty Object Notation):
```ron
// scene.ron
Scene(
    entities: [
        Entity(
            id: 42,
            name: "Player",
            components: {
                Pose: (pos: (10, 5), rotation: 0.0, scale: 1.0),
                Health: (current: 100, max: 100),
                Collider: Box((width: 2, height: 2)),
            },
            children: [123, 456], // Hierarchical parenting
        ),
        // ... more entities
    ],
    metadata: (
        version: "0.1.0",
        created: "2025-11-07T12:00:00Z",
    ),
)
```

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SceneData {
    pub entities: Vec<EntityData>,
    pub metadata: SceneMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct EntityData {
    pub id: Entity,
    pub name: String,
    pub components: HashMap<String, ComponentData>, // Dynamic components
    pub children: Vec<Entity>,
}

// Integrate with astraweave-core World
impl World {
    pub fn save_to_ron(&self, path: &Path) -> Result<()> {
        let scene_data = self.to_scene_data()?;
        let ron_string = ron::to_string(&scene_data)?;
        std::fs::write(path, ron_string)?;
        Ok(())
    }
    
    pub fn load_from_ron(&mut self, path: &Path) -> Result<()> {
        let ron_string = std::fs::read_to_string(path)?;
        let scene_data: SceneData = ron::from_str(&ron_string)?;
        self.from_scene_data(scene_data)?;
        Ok(())
    }
}
```

**Integration Points**:
- File menu: File → Save Scene (Ctrl+S), Load Scene (Ctrl+O)
- Autosave every 5 minutes → `.autosave/` folder
- Recent files list (last 10 scenes)

**Success Criteria**:
- Save/load preserves all entity data (pos, rot, scale, components)
- Hierarchical relationships preserved
- Versioning system (warn on old file formats)
- Crash-safe (atomic writes with temp files)

---

#### 2.3: Component-Based Inspector (Week 3-4)
**Why Third**: Unlock extensibility for any component type

**Current Problem**: Inspector hardcoded for `Pose` only
**Solution**: Reflection system or trait-based component UI

**Option A: Trait-Based (Simple, Rust-native)**:
```rust
pub trait InspectorUI {
    fn ui(&mut self, ui: &mut egui::Ui, label: &str) -> bool; // Returns true if changed
}

impl InspectorUI for Pose {
    fn ui(&mut self, ui: &mut egui::Ui, label: &str) -> bool {
        let mut changed = false;
        ui.label(label);
        changed |= ui.add(egui::DragValue::new(&mut self.pos.x).prefix("X: ")).changed();
        changed |= ui.add(egui::DragValue::new(&mut self.pos.y).prefix("Z: ")).changed();
        changed |= ui.add(egui::DragValue::new(&mut self.rotation).prefix("Rotation: ")).changed();
        changed |= ui.add(egui::DragValue::new(&mut self.scale).prefix("Scale: ")).changed();
        changed
    }
}

impl InspectorUI for Health {
    fn ui(&mut self, ui: &mut egui::Ui, label: &str) -> bool {
        ui.label(label);
        let mut changed = false;
        changed |= ui.add(egui::DragValue::new(&mut self.current).prefix("HP: ")).changed();
        changed |= ui.add(egui::DragValue::new(&mut self.max).prefix("Max: ")).changed();
        changed
    }
}
```

**Option B: Reflection (Advanced, like bevy_inspector_egui)**:
- Requires `#[derive(Reflect)]` on all components
- Auto-generate UI from type metadata
- More complex but fully dynamic

**Recommendation**: Start with **Option A** (trait-based) for Phase 2, migrate to reflection in Phase 4

**Integration Points**:
- Inspector panel: Loop over entity components, call `.ui()` for each
- Undo integration: Wrap component edits in `EditComponentCommand`
- Add Component button: Dropdown list of registered component types

**Success Criteria**:
- Inspector shows all components on selected entity
- Editing any component creates undo command
- "Add Component" button works with type registry
- Component headers collapsible (like Unity)

---

### Phase 3: Productivity Layer (4-6 weeks)

#### 3.1: Asset Browser (Week 1-2)
**Implementation**:
- File tree view (egui `Tree` widget)
- Thumbnail previews (textures, models)
- Drag-drop assets into scene
- Import settings dialog (scale, pivot, collider)

#### 3.2: Hierarchy Enhancements (Week 2-3)
**Implementation**:
- Drag-drop entity parenting
- Multi-selection (Ctrl+click, Shift+click)
- Right-click context menu (Duplicate, Delete, Rename)
- Entity grouping ("Empty" nodes)

#### 3.3: Snapping & Grid (Week 3-4)
**Implementation**:
- Grid snapping (Ctrl held → snap to 1.0 units)
- Angle snapping (rotate in 15° increments)
- Vertex snapping (V key → snap to nearest vertex)
- Grid size controls (0.5, 1.0, 2.0 units)

#### 3.4: Copy/Paste/Duplicate (Week 4)
**Implementation**:
- Ctrl+C → serialize selected entities to clipboard (JSON)
- Ctrl+V → deserialize and spawn at mouse cursor
- Ctrl+D → duplicate in place (offset by 1 unit)

---

### Phase 4: Advanced Features (6-8 weeks)

#### 4.1: Prefab System (Week 1-3)
**What**: Reusable entity templates with override system

**Example Workflow**:
1. Create "Enemy Goblin" entity (mesh, health, AI)
2. Right-click → Create Prefab → save as `goblin.prefab.ron`
3. Drag `goblin.prefab.ron` into scene → instantiates goblin
4. Edit prefab instance → mark as override (blue text)
5. "Apply to Prefab" button → updates all instances
6. "Revert to Prefab" button → discard local edits

**Implementation**:
```rust
pub struct PrefabInstance {
    pub source: PathBuf, // Path to .prefab.ron file
    pub overrides: HashMap<String, ComponentData>, // Modified components
}

// Detect changes from prefab
fn is_component_overridden(instance: &PrefabInstance, component_name: &str) -> bool {
    instance.overrides.contains_key(component_name)
}

// Apply instance changes back to prefab file
fn apply_to_prefab(instance: &PrefabInstance, world: &World) -> Result<()> {
    let prefab_data = world.entity_to_prefab_data(instance.entity_id)?;
    std::fs::write(&instance.source, ron::to_string(&prefab_data)?)?;
    Ok(())
}
```

**Success Criteria**:
- Drag prefab from asset browser → instantiates entity
- Editing instance marks component as overridden (visual indicator)
- "Apply to Prefab" updates all instances globally
- Nested prefabs supported (prefab contains prefab)

---

#### 4.2: Play-in-Editor (Week 3-5)
**What**: Run game logic while editor is open (like Unity Play button)

**Architecture**:
```rust
pub enum EditorMode {
    Edit,   // Normal editing
    Play,   // Game running
    Paused, // Game paused (can inspect state)
}

impl EditorApp {
    fn enter_play_mode(&mut self) -> Result<()> {
        // 1. Save current scene state (for Revert on Stop)
        self.play_mode_snapshot = self.world.clone();
        
        // 2. Switch to Play mode
        self.mode = EditorMode::Play;
        
        // 3. Start game systems (physics, AI, scripts)
        self.game_systems.start(&mut self.world)?;
        
        Ok(())
    }
    
    fn exit_play_mode(&mut self) -> Result<()> {
        // 1. Stop game systems
        self.game_systems.stop();
        
        // 2. Restore pre-play state
        self.world = self.play_mode_snapshot.take().unwrap();
        
        // 3. Switch back to Edit mode
        self.mode = EditorMode::Edit;
        
        Ok(())
    }
}
```

**UI Changes**:
- Top toolbar: ▶️ Play, ⏸️ Pause, ⏹️ Stop buttons
- Viewport tint (orange border when playing)
- Disable editing during play (inspector read-only)

**Success Criteria**:
- Play button starts game loop (physics, AI, scripts run)
- Stop button reverts to pre-play state (no data loss)
- Pause button freezes game, allows inspection
- Frame-by-frame stepping (Step button)

---

#### 4.3: Hot Reload (Week 5-6)
**What**: Update code/assets without restarting editor

**Implementation**:
```rust
use notify::{Watcher, RecursiveMode};

pub struct HotReloadSystem {
    watcher: RecommendedWatcher,
    modified_files: Arc<Mutex<Vec<PathBuf>>>,
}

impl HotReloadSystem {
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        Ok(())
    }
    
    pub fn poll_changes(&mut self) -> Vec<PathBuf> {
        let mut files = self.modified_files.lock().unwrap();
        std::mem::take(&mut *files) // Drain the list
    }
}

// In editor update loop:
fn update(&mut self) {
    for changed_file in self.hot_reload.poll_changes() {
        match changed_file.extension() {
            Some("png") | Some("jpg") => self.reload_texture(&changed_file),
            Some("glb") | Some("gltf") => self.reload_model(&changed_file),
            Some("rs") => self.reload_script(&changed_file), // Requires dynamic linking
            _ => {}
        }
    }
}
```

**Success Criteria**:
- Editing texture in Photoshop → auto-reloads in editor
- Modifying .glb model → updates in viewport
- Script hot-reload (requires `libloading` or WASM)

---

#### 4.4: Visual Scripting (Week 7-8) [OPTIONAL]
**What**: Node-based logic editor (like Unreal Blueprints)

**Implementation**:
- Use `egui_node_graph` crate
- Define node types (GetPosition, SetVelocity, OnCollision)
- Connect to astraweave-ai behavior trees
- Compile to Rust code or interpret at runtime

**Success Criteria**:
- Create simple behavior (OnKeyPress → MoveForward) via nodes
- Save/load node graphs as JSON
- Execute in Play mode

---

### Phase 5: Polish & Ecosystem (4-6 weeks)

#### 5.1: Advanced Viewport (Week 1-2)
- Multiple viewports (split screen: top/front/side views)
- View modes (Wireframe, Lighting-only, Collision-only)
- Camera bookmarks (F1-F12 to save/recall)
- Screenshot tool (capture viewport to PNG)

#### 5.2: Build Manager (Week 2-3)
- Target platform selection (Windows, Linux, Web)
- Asset bundling (compress textures, strip debug symbols)
- One-click build button
- Output logs and error reporting

#### 5.3: Plugin System (Week 3-4)
- Define plugin API (trait `EditorPlugin`)
- Hot-load plugins from `plugins/` folder
- Example: Custom inspector, custom gizmo, custom importer

#### 5.4: Profiler Integration (Week 4-5)
- Frame time graph (CPU/GPU)
- Memory usage tracker
- Draw call counter
- Entity count / system timing breakdown

#### 5.5: Dark Theme & Layouts (Week 5-6)
- Dark/light theme toggle (egui `Visuals`)
- Save panel layouts (serialize `egui::DockState`)
- Layout presets (Modeling, Animation, Scripting)

---

## 🎯 Success Metrics

**World-Class Status Checklist** (All must be ✅):

### Core Functionality
- [ ] Undo/redo for ALL operations
- [ ] Save/load scenes with full fidelity
- [ ] Component-based inspector (extensible)
- [ ] Drag-drop asset import
- [ ] Multi-selection and bulk editing
- [ ] Hierarchical entity parenting

### Workflow Essentials
- [ ] Copy/paste/duplicate entities
- [ ] Prefab system with override tracking
- [ ] Play-in-editor mode
- [ ] Hot reload (assets + scripts)
- [ ] Snapping (grid, angle, vertex)

### Advanced Features
- [ ] Visual scripting (optional but recommended)
- [ ] Physics debug visualization
- [ ] Build manager with packaging
- [ ] Plugin system for extensions
- [ ] Performance profiler in-editor

### UX Polish
- [ ] Dark/light theme support
- [ ] Customizable layouts (save/load)
- [ ] Context menus everywhere (right-click)
- [ ] Tooltips on all buttons
- [ ] Keyboard shortcut consistency (Ctrl+S, Ctrl+Z, etc.)

---

## 📈 Timeline Summary

| Phase | Duration | Focus | Deliverables |
|-------|----------|-------|--------------|
| Phase 1 | ✅ **COMPLETE** | Gizmos & Viewport | Transform tools, camera controls |
| Phase 2 | 4-6 weeks | Foundation | Undo/redo, save/load, inspector |
| Phase 3 | 4-6 weeks | Productivity | Asset browser, hierarchy, snapping |
| Phase 4 | 6-8 weeks | Advanced | Prefabs, play-in-editor, hot reload |
| Phase 5 | 4-6 weeks | Polish | Profiler, plugins, themes, layouts |
| **Total** | **18-26 weeks** | **(4.5-6.5 months)** | World-class editor |

---

## 🚀 Next Immediate Steps

**Recommendation: Start Phase 2.1 (Undo/Redo)**

1. **Week 1**: Implement `EditorCommand` trait and `UndoStack`
2. **Week 2**: Wrap gizmo transforms in commands, test with Ctrl+Z
3. **Week 3**: Extend to inspector edits and entity creation/deletion
4. **Week 4**: Add UI indicators (status bar, undo menu)

**Why This Order**:
- Undo/redo is foundational (every future feature depends on it)
- Relatively self-contained (doesn't require save/load or asset system)
- High user impact (immediately improves editing experience)
- Establishes command pattern for future features

---

## 🔧 Technical Recommendations

### Architecture Patterns
1. **Command Pattern**: All editor actions as reversible commands
2. **Event-Driven**: Use `egui::Context::request_repaint()` for async updates
3. **Separation of Concerns**: Keep editor code separate from game code
4. **Hot-Reload Friendly**: Design for dynamic reloading from day 1

### Dependencies to Add
```toml
[dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
ron = "0.8" # Rusty Object Notation

# File watching
notify = "6.0"

# Clipboard
arboard = "3.2" # Cross-platform clipboard

# Visual scripting (optional)
egui_node_graph = "0.6"

# Reflection (Phase 4)
bevy_reflect = { version = "0.12", optional = true }
```

### Performance Targets
- **Editor FPS**: 60 FPS minimum (even with 1000+ entities)
- **Undo Stack**: 100 commands without lag
- **Save Time**: <1 second for typical scenes (<10,000 entities)
- **Hot Reload**: <500ms for texture updates

---

## 💡 Inspiration & References

**Study These Editors**:
- **Unity**: Gold standard for inspector, prefabs, play-in-editor
- **Unreal**: Blueprint visual scripting, multi-viewport, material editor
- **Godot**: Lightweight, fast iteration, integrated script editor
- **Blender**: Custom layout system, extensibility, hotkeys

**Key Takeaways**:
- **Unity**: Copy the inspector component workflow (collapsible headers, Add Component button)
- **Unreal**: Study the command pattern for undo (every action is a transaction)
- **Godot**: Emulate the fast startup time and lightweight feel
- **Blender**: Learn from the keyboard shortcut system (G/R/S for transform)

---

## 📝 Notes

**Current Strengths to Preserve**:
- ✅ Smooth gizmo interactions (Phase 1 nailed this)
- ✅ Clean separation between viewport and panels
- ✅ wgpu rendering pipeline (future-proof)

**Potential Pitfalls**:
- ⚠️ Don't over-engineer reflection system (start simple with traits)
- ⚠️ Undo/redo memory management (limit stack size, don't leak)
- ⚠️ Hot reload can cause crashes (robust error handling required)

**User Testing Goals**:
- By end of Phase 2: "I can build a simple scene and save it"
- By end of Phase 3: "I can work as fast as in Unity for basic tasks"
- By end of Phase 4: "I can prototype gameplay without leaving editor"
- By end of Phase 5: "This feels professional and polished"

---

**Last Updated**: November 11, 2025  
**Document Owner**: AstraWeave AI Development Team  
**Status**: Phase 1 Complete ✅ | Phase 2 Complete ✅ | Phase 3 Complete ✅ | Phase 4.1 Complete ✅ | Phase 4.2 Complete ✅ | Phase 4.3 Complete ✅ | Phase 5.1 Complete ✅
