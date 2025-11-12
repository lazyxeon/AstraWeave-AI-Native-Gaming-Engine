# Phase 4.3 Hot Reload + Comprehensive Editor Testing Plan

**Date**: November 11, 2025  
**Phase**: 4.3 Hot Reload Implementation + Full Editor Testing  
**Status**: Implementation Complete - Testing Pending

---

## Overview

This document outlines the comprehensive testing plan for:
1. **Phase 4.3**: Model Hot Reload (newly implemented)
2. **Full Editor Validation**: All features from Phases 1-5.1

---

## Phase 4.3: Hot Reload Implementation Summary

### What Was Implemented

**File Watcher Extensions**:
- ✅ Added `Model(PathBuf)` variant to `ReloadEvent` enum
- ✅ Extended file detection to recognize `.glb` and `.gltf` files
- ✅ Added `model_file_watcher` to `EditorApp` struct
- ✅ Initialized model watcher for `assets/models` directory
- ✅ Added model hot reload event processing in main update loop
- ✅ Updated `material_inspector.rs` to handle Model variant
- ✅ Updated documentation in `file_watcher.rs`

**Files Modified**:
1. `tools/aw_editor/src/file_watcher.rs` - Added Model variant, detection logic
2. `tools/aw_editor/src/main.rs` - Added model_file_watcher field and event processing
3. `tools/aw_editor/src/material_inspector.rs` - Added Model match arm

**Build Status**: ✅ `cargo check --package aw_editor` passes with 0 errors

---

## Comprehensive Testing Plan

### Test Category 1: Hot Reload System (Phase 4.3)

#### Test 1.1: Material Hot Reload
**Objective**: Verify material files auto-reload when modified

**Steps**:
1. Launch editor
2. Open Material Inspector
3. Load a material from `assets/materials/`
4. Modify the material's `.toml` file externally (e.g., change roughness value)
5. Save the file
6. Wait 500ms for debounce

**Expected Results**:
- ✅ Status bar shows: "🔄 Material file changed: {path}"
- ✅ Console log shows: "🔄 Detected material change: {path}"
- ✅ Material preview updates automatically
- ✅ Inspector shows new values

---

#### Test 1.2: Texture Hot Reload
**Objective**: Verify texture files auto-reload when modified

**Steps**:
1. Launch editor
2. Open Material Inspector with a loaded material
3. Modify one of the material's texture files (albedo, normal, ORM, MRA)
4. Save the modified texture
5. Wait 500ms for debounce

**Expected Results**:
- ✅ Status bar shows: "✅ Texture hot-reloaded: {filename}"
- ✅ Material preview updates with new texture
- ✅ No lag or crash

---

#### Test 1.3: Prefab Hot Reload
**Objective**: Verify prefab files auto-reload when modified

**Steps**:
1. Launch editor
2. Create a prefab from an entity (right-click → Create Prefab)
3. Note the prefab file path
4. Modify the `.prefab.ron` file externally (e.g., change position)
5. Save the file
6. Wait 500ms for debounce

**Expected Results**:
- ✅ Status bar shows: "🔄 Prefab file changed: {path}"
- ✅ Console log shows: "🔄 Detected prefab change: {path}"
- ✅ New instances use updated prefab data

---

#### Test 1.4: Model Hot Reload (NEW)
**Objective**: Verify model files auto-reload when modified

**Steps**:
1. Launch editor
2. Place a 3D model in the scene (from `assets/models/`)
3. Modify the model file externally (e.g., change geometry in Blender, re-export)
4. Save the modified `.glb` or `.gltf` file
5. Wait 500ms for debounce

**Expected Results**:
- ✅ Status bar shows: "🔄 Model file changed: {path}"
- ✅ Console log shows: "🔄 Detected model change: {path}"
- ✅ Model in viewport updates (if reload logic implemented)

**Note**: Current implementation detects changes but actual model reload may require additional integration.

---

### Test Category 2: Prefab System (Phase 4.1)

#### Test 2.1: Create Prefab
**Steps**:
1. Create an entity in the scene
2. Add components (Health, Pose, etc.)
3. Right-click entity in hierarchy
4. Select "💾 Create Prefab"
5. Enter filename "test_prefab"

**Expected Results**:
- ✅ File created at `prefabs/test_prefab.prefab.ron`
- ✅ Status bar shows: "✅ Prefab created: test_prefab.prefab.ron"
- ✅ File contains entity data in RON format

---

#### Test 2.2: Drag-Drop Prefab from Asset Browser
**Steps**:
1. Open Asset Browser panel
2. Navigate to prefabs folder
3. Locate a `.prefab.ron` file (displays with 💾 icon, blue color)
4. Click and drag the prefab file
5. Release mouse button

**Expected Results**:
- ✅ Prefab instantiates at position (0, 0)
- ✅ Status bar shows: "✅ Instantiated prefab at (0, 0): root={entity_id}"
- ✅ Console log shows: "✅ Instantiated prefab: {path} -> root entity #{id}"
- ✅ New entity appears in hierarchy

---

#### Test 2.3: Prefab Override Indicators
**Steps**:
1. Instantiate a prefab
2. Select the prefab instance in hierarchy
3. Check Entity Inspector panel
4. Look for prefab info display

**Expected Results**:
- ✅ Inspector shows: "💾 Prefab Instance: {filename}"
- ✅ If no overrides: No warning text
- ✅ If entity modified: Blue warning text appears
- ✅ Warning text: "⚠️ Modified components (blue text indicates overrides)"

---

#### Test 2.4: Nested Prefabs (Foundation)
**Steps**:
1. Open a `.prefab.ron` file
2. Check for `prefab_reference` field in entity data

**Expected Results**:
- ✅ Field exists but is `None` for regular entities
- ✅ Entities with `prefab_reference: Some("path")` are skipped during instantiation
- ✅ No crashes when loading prefabs

**Note**: Full nested prefab support is not yet implemented (foundation only).

---

### Test Category 3: Gizmo System (Phase 1)

#### Test 3.1: Translate Gizmo (G Key)
**Steps**:
1. Select an entity
2. Press `G` key
3. Move mouse
4. Observe entity movement

**Constraint Tests**:
- Press `G` → `X` → Move: Entity moves only on X axis (red axis)
- Press `G` → `Z` → Move: Entity moves only on Z axis (blue axis)
- Press `Escape`: Cancel movement, restore original position
- Press `Enter`: Confirm movement

**Expected Results**:
- ✅ FREE mode: Entity follows mouse in viewport plane
- ✅ X constraint: Red axis highlights, movement locked to X
- ✅ Z constraint: Blue axis highlights, movement locked to Z
- ✅ Raycast mode: Entity snaps to ground plane intersection
- ✅ Status bar shows: "🎯 Translate Mode | FREE" or "🎯 Translate Mode | X-Axis"

---

#### Test 3.2: Rotate Gizmo (R Key)
**Steps**:
1. Select an entity
2. Press `R` key
3. Move mouse horizontally

**Rotation Tests**:
- Press `R` → Move: Rotates around Y axis (yaw)
- Press `R` → `X`: Rotates around X axis (pitch)
- Press `R` → `Y`: Rotates around Y axis (yaw)
- Press `R` → `Z`: Rotates around Z axis (roll)

**Expected Results**:
- ✅ Entity rotates smoothly
- ✅ Visual feedback shows rotation amount
- ✅ Status bar shows: "🔄 Rotate Mode | Y-Axis"
- ✅ Escape cancels, Enter confirms

---

#### Test 3.3: Scale Gizmo (S Key)
**Steps**:
1. Select an entity
2. Press `S` key
3. Scroll mouse wheel up/down

**Expected Results**:
- ✅ Scroll up: Entity scales larger (1% per scroll tick)
- ✅ Scroll down: Entity scales smaller
- ✅ Status bar shows: "📏 Scale Mode | Uniform"
- ✅ Minimum scale clamped (no negative/zero)
- ✅ Escape cancels, Enter confirms

---

### Test Category 4: Undo/Redo System (Phase 2.1)

#### Test 4.1: Undo Transform Operations
**Steps**:
1. Select entity at position (5, 0)
2. Press `G`, move to (10, 0), press Enter
3. Press `Ctrl+Z`

**Expected Results**:
- ✅ Entity returns to (5, 0)
- ✅ Status bar shows: "↩️ Undo: Move Entity"

---

#### Test 4.2: Redo Transform Operations
**Steps**:
1. Perform undo (Test 4.1)
2. Press `Ctrl+Y` (or `Ctrl+Shift+Z`)

**Expected Results**:
- ✅ Entity moves back to (10, 0)
- ✅ Status bar shows: "↪️ Redo: Move Entity"

---

#### Test 4.3: Undo Component Edits
**Steps**:
1. Select entity with Health component
2. Change health from 100 to 50 in inspector
3. Press `Ctrl+Z`

**Expected Results**:
- ✅ Health returns to 100
- ✅ Status bar shows: "↩️ Undo: Edit Health"

---

#### Test 4.4: Undo History Limit
**Steps**:
1. Perform 150 transform operations (exceeds 100-command limit)
2. Press `Ctrl+Z` repeatedly

**Expected Results**:
- ✅ Can undo up to 100 commands
- ✅ Oldest commands beyond 100 are discarded
- ✅ No memory leak or crash

---

### Test Category 5: Scene Save/Load (Phase 2.2)

#### Test 5.1: Save Scene
**Steps**:
1. Create several entities with components
2. Press `Ctrl+S`
3. Enter filename "test_scene"

**Expected Results**:
- ✅ File saved to `scenes/test_scene.ron`
- ✅ Status bar shows: "💾 Scene saved: test_scene.ron"
- ✅ File contains all entity data in RON format

---

#### Test 5.2: Load Scene
**Steps**:
1. Clear current scene
2. Press `Ctrl+O`
3. Select "test_scene.ron" from recent files

**Expected Results**:
- ✅ All entities restored with correct positions
- ✅ All components restored (Health, Team, etc.)
- ✅ Status bar shows: "✅ Scene loaded: test_scene.ron"
- ✅ Hierarchy panel shows all entities

---

#### Test 5.3: Autosave
**Steps**:
1. Make changes to scene
2. Wait 5 minutes (autosave interval)

**Expected Results**:
- ✅ File created in `.autosave/` folder
- ✅ Console log shows: "💾 Autosaved scene"
- ✅ No lag during autosave

---

### Test Category 6: Asset Browser (Phase 3.1)

#### Test 6.1: Navigate Folders
**Steps**:
1. Open Asset Browser
2. Click folder icons to navigate

**Expected Results**:
- ✅ Folder contents update
- ✅ Breadcrumb shows current path
- ✅ "Up" button works

---

#### Test 6.2: View Modes (List/Grid)
**Steps**:
1. Click "List" button
2. Click "Grid" button

**Expected Results**:
- ✅ List view: Files shown in vertical list with icons
- ✅ Grid view: Files shown in grid with larger previews
- ✅ Both modes support drag-drop

---

#### Test 6.3: File Type Recognition
**Steps**:
1. Navigate to folders with different file types

**Expected Results**:
- ✅ `.prefab.ron`: 💾 icon, blue color (RGB 150, 200, 255)
- ✅ `.glb`, `.gltf`: 🎲 icon, green color (RGB 150, 255, 150)
- ✅ `.png`, `.jpg`: 🖼️ icon, purple color
- ✅ `.toml`: ⚙️ icon, gray color
- ✅ Unknown: 📄 icon, white text

---

### Test Category 7: Play-in-Editor (Phase 4.2)

#### Test 7.1: Enter Play Mode
**Steps**:
1. Set up scene with entities
2. Click ▶️ Play button (or press F5)

**Expected Results**:
- ✅ Viewport border turns orange
- ✅ Inspector becomes read-only
- ✅ Physics/AI systems start running
- ✅ Status bar shows: "▶️ Play Mode"

---

#### Test 7.2: Pause/Resume
**Steps**:
1. Enter play mode
2. Click ⏸️ Pause button
3. Click ▶️ Resume

**Expected Results**:
- ✅ Game logic freezes during pause
- ✅ Can inspect entity state while paused
- ✅ Resumes smoothly

---

#### Test 7.3: Stop Play Mode
**Steps**:
1. Make changes during play mode
2. Click ⏹️ Stop button

**Expected Results**:
- ✅ Scene reverts to pre-play state
- ✅ All changes discarded
- ✅ Inspector becomes editable again
- ✅ Status bar shows: "✏️ Edit Mode"

---

### Test Category 8: Advanced Viewport (Phase 5.1)

#### Test 8.1: Camera Controls
**Camera Orbit**:
- Left drag: Orbit around focal point
- Middle drag: Pan camera
- Scroll wheel: Zoom in/out

**Expected Results**:
- ✅ Smooth camera movement
- ✅ No camera jitter or jumping
- ✅ Zoom respects min/max distance

---

#### Test 8.2: Grid & Skybox
**Steps**:
1. Observe viewport rendering

**Expected Results**:
- ✅ Grid visible at Y=0 plane
- ✅ Skybox renders behind all objects
- ✅ Grid fades at distance

---

#### Test 8.3: Entity Picking
**Steps**:
1. Click on entity in viewport
2. Verify selection

**Expected Results**:
- ✅ Entity highlights
- ✅ Hierarchy panel syncs
- ✅ Inspector shows components
- ✅ Gizmo appears at entity position

---

## Testing Execution Checklist

### Pre-Test Setup
- [ ] Build editor: `cargo build --package aw_editor --release`
- [ ] Create test assets in `assets/models/` (copy sample .glb files)
- [ ] Create test materials in `assets/materials/`
- [ ] Create test prefabs in `prefabs/`
- [ ] Prepare external editor (Blender, GIMP) for file modification tests

### Test Execution Order
1. ✅ Category 6: Asset Browser (verify UI works)
2. ✅ Category 3: Gizmo System (basic interaction)
3. ✅ Category 4: Undo/Redo (verify history works)
4. ✅ Category 5: Scene Save/Load (persistence)
5. ✅ Category 2: Prefab System (drag-drop, overrides)
6. ✅ Category 1: Hot Reload (materials, textures, prefabs, models)
7. ✅ Category 7: Play-in-Editor (game logic)
8. ✅ Category 8: Advanced Viewport (rendering, picking)

### Post-Test Documentation
- [ ] Record test results in this document
- [ ] Take screenshots of key features
- [ ] Note any bugs or issues discovered
- [ ] Create bug report if critical issues found
- [ ] Update roadmap with completion status

---

## Known Limitations

### Phase 4.3 (Hot Reload)
- ⚠️ Model hot reload detects file changes but may not trigger automatic re-import
- ⚠️ Requires manual refresh or entity re-selection to see updated model
- ⚠️ Full model reload integration requires renderer pipeline updates

### Phase 4.1 (Prefabs)
- ⚠️ Nested prefabs foundation exists but full recursive loading not implemented
- ⚠️ Override tracking works but "Apply to Prefab" / "Revert" buttons not implemented
- ⚠️ Override indicators show warning text but don't highlight specific fields

### General
- ⚠️ Editor assumes `prefabs/` and `assets/models/` directories exist
- ⚠️ File watcher requires `.ok()` (non-existent directories silently fail)

---

## Success Criteria

**Phase 4.3 Complete** if:
- ✅ All 4 hot reload tests pass (materials, textures, prefabs, models)
- ✅ No crashes during file modification
- ✅ Debouncing works correctly (500ms delay)
- ✅ Status messages display correctly

**Full Editor Validation Complete** if:
- ✅ All 8 test categories pass (38 individual tests)
- ✅ No critical bugs discovered
- ✅ Performance is acceptable (60 FPS in viewport)
- ✅ All features from Phases 1-5.1 functional

---

## Test Results

**Testing Date**: _To be filled after testing_  
**Tester**: _AI/Human_  
**Build Version**: _To be filled_

### Results Summary Table

| Category | Tests Passed | Tests Failed | Notes |
|----------|--------------|--------------|-------|
| 1. Hot Reload | - / 4 | - | |
| 2. Prefab System | - / 4 | - | |
| 3. Gizmo System | - / 3 | - | |
| 4. Undo/Redo | - / 4 | - | |
| 5. Save/Load | - / 3 | - | |
| 6. Asset Browser | - / 3 | - | |
| 7. Play-in-Editor | - / 3 | - | |
| 8. Advanced Viewport | - / 3 | - | |
| **TOTAL** | **- / 38** | **-** | |

---

## Next Steps After Testing

1. **If All Tests Pass**:
   - Update `EDITOR_ROADMAP_TO_WORLD_CLASS.md` status
   - Mark Phase 4.3 as ✅ Complete
   - Proceed to Phase 5.2 (Build Manager) or Phase 4.4 (Visual Scripting)

2. **If Critical Bugs Found**:
   - Create detailed bug report
   - Prioritize fixes
   - Re-test after fixes

3. **Future Enhancements**:
   - Implement actual model reload (trigger re-import from GPU pipeline)
   - Add "Apply to Prefab" / "Revert" UI buttons
   - Implement nested prefab recursive loading
   - Add blue text highlighting for individual overridden fields

---

**Document Version**: 1.0  
**Last Updated**: November 11, 2025  
**Status**: Testing Plan Ready - Awaiting Execution
