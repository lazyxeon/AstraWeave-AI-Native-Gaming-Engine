# AW Editor – Current Failure Modes (Nov 15, 2025)

This note captures the reproducible breakages reported for `tools/aw_editor` and the code-level evidence gathered after inspecting the latest sources.

## Status Update (Nov 15, 2025)
- ✅ Grid toolbar plumbing is now live: `ViewportWidget` forwards snap sliders into `GridRenderSettings`, and `ViewportRenderer` skips the GPU pass entirely when `show_grid` is disabled. Verified via `cargo test -p aw_editor --test ui_gizmo_smoke`.
- ✅ Telemetry capture + headless harness landed, giving us deterministic gizmo repros and assertions for commit/cancel flows.
- 🔄 Still outstanding for Week 1: migrate viewport selection/state to `EditorSceneState` and circulate this Known Issues doc for sign-off.

## 1. Gizmo Translation Snaps Back
- **Symptoms**: Pressing `G` to move an entity shows live movement while dragging, but the entity jumps back (often to the origin) as soon as the mouse is released.
- **Root causes**:
  - Gizmo operations mutate `EntityManager` state only. The authoritative ECS world (`sim_world`) never receives the transform delta, so the next frame repopulates the gizmo from the untouched `EntityManager` entry. `TransformPanel` also reads exclusively from `entity_manager` (see `main.rs` lines ~1500).
  - `entity_manager` itself is populated with sample primitives (`EditorApp::init_sample_entities`) and never synchronized with actual ECS entities; drag operations therefore decouple visuals from the underlying simulation data.
  - No undo command is issued at drag release, so any manual synchronization with the ECS world would still lack persistence/history.

## 2. Grid Toggle and Snapping — **Resolved (Nov 15)**
- `GridRenderSettings` now flows from the toolbar → widget → renderer → WGSL uniforms. Disabling the grid bypasses the entire render pass, and spacing mirrors the snap slider (clamped to ≥0.1m).
- Remaining follow-ups: surface major-line density in the UI and ensure gizmo snapping math consumes the same `SnappingConfig` used by the renderer, so visual + interaction layers stay in lockstep.

## 3. Behavior Node Editor Is Static
- `EditorApp::show_behavior_graph_editor` (lines ~470-560) renders a pre-canned `BehaviorNode` tree. There is no editable data-model, no node creation UI, and no serialization hooks into `astraweave_behavior::BehaviorGraph`.
- The egui panel calls `show_node` recursively on an in-memory literal, so the “editor” is effectively documentation rather than an authoring surface.

## 4. Asset / Texture Import Missing
- `AssetBrowser` advertises drag-and-drop but never hands data off to the rest of the editor: `dragged_prefab` is only set within `asset_browser.rs` and never consumed elsewhere.
- There is no pathway from the file picker to instantiate prefabs or meshes inside the current world (`PrefabManager::instantiate_prefab` is defined but never called outside prefab tests). Consequently designers cannot add new meshes, materials, or textures into a scene.

## 5. Play/Pause/Stop Does Not Drive Simulation
- `EditorMode` buttons only toggle booleans; no deterministic snapshot or state isolation happens beyond storing `SceneData::from_world` when Play is pressed once (see `main.rs` lines 1150-1200).
- While `simulation_playing` is `false` (normal Edit mode), the main loop forcibly sets `self.sim_world = None` every frame (lines 1845-1860). Stopping simulation therefore destroys the edited world and replaces it with a default template when the viewport repaints.
- When Play is active, `sim_world` is lazily built from `level.obstacles`/`npcs`, ignoring the entities the user was editing. There is no bridge back to the ECS data after simulation, so edited content never participates in gameplay and user changes disappear on the next tick.

## 6. Prefab/Entity Sync Gaps
- `PrefabManager::find_instance` exists, but prefab overrides are neither detected nor visualized in the hierarchy. `EntityPanel` references prefab instances only after a manual lookup triggered in `main.rs`, and there is no way to apply/revert overrides.
- Related to issue #4: new prefabs cannot be instantiated from the asset browser, so prefab workflows stall.

## 7. Telemetry & Testing Absent
- No tracing/logging exists around gizmo drags, grid toggles, or play-state transitions, making regressions hard to diagnose.
- There are zero automated UI/regression tests for aw_editor; breakages go unnoticed until manual testing.

These findings will seed the follow-up design documents (interaction fixes, authoring upgrades, simulation overhaul) and give us concrete repro steps for validation harnesses.
