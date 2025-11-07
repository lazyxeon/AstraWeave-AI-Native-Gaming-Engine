# AW Editor Integration Roadmap

## Current State Analysis (November 6, 2025)

The aw_editor has **excellent architecture** with well-designed systems, but they are **not yet connected**. The codebase is ~80% complete - the hard work of building individual systems (gizmos, picking, rendering, panels) is done. What remains is the **integration glue** to wire everything together.

### What's Built (Excellent Quality)
✅ **3D Viewport** - wgpu rendering, camera controls, grid, skybox
✅ **Gizmo System** - Complete picking, state machine, translate/rotate/scale logic
✅ **Transform Panel** - UI for numeric transform editing
✅ **Material Inspector** - PBR texture preview and editing
✅ **Panels** - World, Entity, Performance, Charts, Animation
✅ **Camera** - OrbitCamera with pan/zoom/orbit
✅ **Rendering** - Grid, skybox, entities, gizmo visualization

### What's Missing (Integration Code ~1000-1500 LOC)
❌ **Gizmo ↔ Viewport Integration** - Picking not wired to click events
❌ **Transform ↔ Entity Integration** - Changes don't update entities
❌ **Panel ↔ Selection Integration** - Transform panel not synced
❌ **Entity Management** - No ECS integration for entity CRUD
❌ **Camera Helpers** - Frame entity, keyboard shortcuts
❌ **Advanced Features** - Multi-select, snap, local/world space

---

## Implementation Phases

### Phase 1: Core Gizmo Integration (CRITICAL - 3-5 hours)

**Goal**: Make entity selection and transformation functional

**Tasks**:

1. **Connect Gizmo Picker to Viewport** (`viewport/widget.rs`)
   ```rust
   // Replace simple entity cycling with proper picking
   impl ViewportWidget {
       fn handle_click(&mut self, screen_pos: egui::Pos2, inv_view_proj: Mat4) {
           let picker = GizmoPicker::default();
           
           // Pick gizmo first (if active)
           if let Some(selected) = self.selected_entity {
               let gizmo_pos = self.get_entity_position(selected);
               if let Some(handle) = picker.pick_handle(
                   to_ndc(screen_pos), inv_view_proj, gizmo_pos, self.gizmo_mode
               ) {
                   self.active_gizmo = Some(handle);
                   return;
               }
           }
           
           // Pick entity (ray-AABB test)
           let ray = Ray::from_screen(to_ndc(screen_pos), inv_view_proj);
           self.selected_entity = self.pick_entity(&ray);
       }
   }
   ```

2. **Wire GizmoState to Mouse Drag** (`viewport/widget.rs`)
   ```rust
   impl ViewportWidget {
       fn handle_drag(&mut self, delta: Vec2) {
           if let Some(handle) = self.active_gizmo {
               let constraint = handle.to_constraint();
               let world_delta = self.screen_to_world_delta(delta);
               
               // Apply constraint and update entity
               let transform_delta = apply_constraint(world_delta, constraint);
               self.apply_transform_to_selected(transform_delta);
           }
       }
   }
   ```

3. **Implement Entity Storage** (`main.rs` or new `entity_manager.rs`)
   ```rust
   struct EntityManager {
       entities: HashMap<u64, EditorEntity>,
       next_id: u64,
   }
   
   struct EditorEntity {
       id: u64,
       name: String,
       position: Vec3,
       rotation: Quat,
       scale: Vec3,
       mesh: Option<String>,
       components: HashMap<String, serde_json::Value>,
   }
   ```

4. **Sync Transform Panel** (`panels/transform_panel.rs` ↔ `main.rs`)
   ```rust
   // In EditorApp::update()
   if let Some(selected) = self.selected_entity {
       let entity = self.entity_manager.get(selected);
       self.transform_panel.set_selected(entity.transform());
       
       // Bidirectional sync
       if self.transform_panel.was_modified() {
           let new_transform = self.transform_panel.get_transform();
           self.entity_manager.update_transform(selected, new_transform);
       }
   }
   ```

**Estimated Time**: 3-5 hours  
**Impact**: 🔥 HIGH - Makes editor immediately usable

---

### Phase 2: Camera & Navigation (2-3 hours)

**Goal**: Complete camera controls for better workflow

**Tasks**:

1. **Implement `OrbitCamera::frame_entity()`** (`viewport/camera.rs`)
   ```rust
   pub fn frame_entity(&mut self, entity_pos: Vec3, entity_radius: f32) {
       self.focal_point = entity_pos;
       self.distance = entity_radius * 3.0; // Nice framing distance
       // Smooth transition (optional)
   }
   ```

2. **Add Keyboard Navigation** (`viewport/widget.rs`)
   ```rust
   // WASD for pan, QE for orbit, F for frame
   if i.key_down(egui::Key::W) { camera.pan(Vec2::new(0.0, 1.0), dt); }
   if i.key_down(egui::Key::F) { 
       if let Some(selected) = self.selected_entity {
           camera.frame_entity(get_entity_pos(selected), 2.0);
       }
   }
   ```

3. **Camera Right/Up Vectors** (`viewport/camera.rs`)
   ```rust
   pub fn right(&self) -> Vec3 {
       let forward = self.forward();
       forward.cross(Vec3::Y).normalize()
   }
   
   pub fn up(&self) -> Vec3 {
       let forward = self.forward();
       let right = self.right();
       right.cross(forward).normalize()
   }
   ```

**Estimated Time**: 2-3 hours  
**Impact**: ⚡ MEDIUM - Better UX, not blocking

---

### Phase 3: Advanced Features (3-4 hours)

**Goal**: Multi-selection, snap to grid, local vs world space

**Tasks**:

1. **Multi-Selection** (`viewport/widget.rs` + `entity_manager.rs`)
   - Ctrl+Click for additive selection
   - Box selection (drag without gizmo)
   - Transform multiple entities

2. **Snap to Grid** (`gizmo/translate.rs` integration)
   ```rust
   fn apply_snap(position: Vec3, snap_size: f32) -> Vec3 {
       Vec3::new(
           (position.x / snap_size).round() * snap_size,
           (position.y / snap_size).round() * snap_size,
           (position.z / snap_size).round() * snap_size,
       )
   }
   ```

3. **Local vs World Space** (`gizmo/state.rs` + rendering)
   - Toggle button in toolbar
   - Rotate gizmo with entity in local mode
   - Keep axes aligned to world in world mode

4. **Entity Outliner Panel**
   - Tree view of all entities
   - Name editing
   - Show/hide toggle
   - Parent/child hierarchy

**Estimated Time**: 3-4 hours  
**Impact**: 🌟 HIGH - Professional editor feel

---

### Phase 4: Polish & Production (2-3 hours)

**Goal**: Make editor production-ready

**Tasks**:

1. **Transform History/Undo**
   ```rust
   struct TransformHistory {
       stack: Vec<TransformSnapshot>,
       current: usize,
   }
   
   // Ctrl+Z, Ctrl+Y
   ```

2. **Keyboard Shortcuts Panel**
   - Help window showing all shortcuts
   - Customizable keybinds (optional)

3. **Entity Count in Stats** (`viewport/renderer.rs`)
   ```rust
   pub fn entity_count(&self) -> usize {
       self.entity_renderer.count()
   }
   
   pub fn triangle_count(&self) -> usize {
       self.entity_renderer.triangle_count()
   }
   ```

4. **Material Inspector Split View** (`material_inspector.rs`)
   - Side-by-side texture comparison
   - Before/after editing

**Estimated Time**: 2-3 hours  
**Impact**: 💎 POLISH - Nice to have

---

## Total Effort Estimate

| Phase | Time | Priority | Status |
|-------|------|----------|--------|
| Phase 1: Core Gizmo Integration | 3-5h | 🔥 CRITICAL | Not Started |
| Phase 2: Camera & Navigation | 2-3h | ⚡ MEDIUM | Not Started |
| Phase 3: Advanced Features | 3-4h | 🌟 HIGH | Not Started |
| Phase 4: Polish & Production | 2-3h | 💎 POLISH | Not Started |
| **Total** | **10-15 hours** | - | **0% Complete** |

---

## Quick Start Guide (For AI Implementation)

### Recommended Order

1. **Day 1** (3-5 hours): Phase 1 - Core Gizmo Integration
   - Most impactful, makes editor functional
   - Creates foundation for other phases
   - Users can see immediate results

2. **Day 2** (2-3 hours): Phase 2 - Camera Controls
   - Improves workflow significantly
   - Keyboard shortcuts feel professional
   - Frame entity is highly requested

3. **Day 3** (3-4 hours): Phase 3 - Advanced Features
   - Multi-selection and snap are game-changers
   - Entity outliner makes large scenes manageable
   - Local/world space is essential for rotation

4. **Day 4** (2-3 hours): Phase 4 - Polish
   - Undo/redo is crucial for production
   - Shortcuts panel helps discoverability
   - Material inspector improvements

### Files to Modify

**Phase 1 (Core Integration)**:
- `tools/aw_editor/src/viewport/widget.rs` - Main integration point
- `tools/aw_editor/src/main.rs` - Add EntityManager, sync panels
- `tools/aw_editor/src/panels/transform_panel.rs` - Wire to selection
- `tools/aw_editor/src/gizmo/mod.rs` - Export picker/state
- `tools/aw_editor/src/gizmo/constraints.rs` - Export apply_constraint

**Phase 2 (Camera)**:
- `tools/aw_editor/src/viewport/camera.rs` - Implement missing methods
- `tools/aw_editor/src/viewport/widget.rs` - Add keyboard handlers

**Phase 3 (Advanced)**:
- `tools/aw_editor/src/main.rs` - Multi-selection state
- `tools/aw_editor/src/viewport/widget.rs` - Box selection
- `tools/aw_editor/src/panels/outliner.rs` - NEW FILE
- `tools/aw_editor/src/gizmo/state.rs` - Local/world toggle

**Phase 4 (Polish)**:
- `tools/aw_editor/src/history.rs` - NEW FILE (undo system)
- `tools/aw_editor/src/panels/shortcuts.rs` - NEW FILE
- `tools/aw_editor/src/material_inspector.rs` - Split view
- `tools/aw_editor/src/viewport/renderer.rs` - Entity counts

---

## Success Criteria

### Phase 1 Complete ✅
- [ ] Click to select entities
- [ ] Drag gizmo to translate entities
- [ ] Transform panel shows selected entity
- [ ] Numeric input in transform panel works
- [ ] Changes sync bidirectionally

### Phase 2 Complete ✅
- [ ] F key frames selected entity
- [ ] WASD keys pan camera
- [ ] QE keys orbit camera
- [ ] Smooth camera transitions

### Phase 3 Complete ✅
- [ ] Ctrl+Click multi-selects
- [ ] Drag box selects multiple entities
- [ ] Snap to grid works (with toggle)
- [ ] Local/world space switch works
- [ ] Entity outliner shows hierarchy

### Phase 4 Complete ✅
- [ ] Ctrl+Z/Ctrl+Y undo/redo works
- [ ] F1 shows keyboard shortcuts
- [ ] Material split view implemented
- [ ] Entity/triangle counts accurate

---

## Architecture Decisions

### Entity Storage
**Decision**: Use `HashMap<u64, EditorEntity>` instead of full ECS integration  
**Rationale**: 
- Simpler for editor use case
- Easy serialization to level files
- Can bridge to ECS for simulation later
- Faster iteration during development

### Transform Sync
**Decision**: Bidirectional sync (Viewport ↔ Transform Panel)  
**Rationale**:
- User can edit numerically or visually
- Both views stay in sync automatically
- Clear source of truth (EntityManager)

### Gizmo Rendering
**Decision**: Keep gizmo rendering separate from entity rendering  
**Rationale**:
- Different rendering requirements
- Gizmos always on top (depth testing)
- Easier to toggle visibility

### Undo System
**Decision**: Command pattern with snapshot-based undo  
**Rationale**:
- Simple to implement
- Works with any transform
- Memory efficient for editor scale

---

## Notes for Future Development

1. **Performance**: Current design handles 1000s of entities efficiently
2. **Extensibility**: Easy to add new gizmo modes (e.g., vertex editing)
3. **Serialization**: Entity format designed for RON/JSON export
4. **Multiplayer**: Can extend to collaborative editing later
5. **Scripting**: Could expose EntityManager via Lua/Rhai

---

## Contact

For questions or implementation help, see:
- Gizmo implementation: `tools/aw_editor/src/gizmo/`
- Viewport implementation: `tools/aw_editor/src/viewport/`
- Panel implementation: `tools/aw_editor/src/panels/`
