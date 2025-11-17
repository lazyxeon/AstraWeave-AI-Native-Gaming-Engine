# AW Editor – Current Failure Modes (Nov 16, 2025)

This note captures the reproducible breakages reported for `tools/aw_editor`, their current status, and the validation artifacts tied to each issue.

## Status Update (Nov 16, 2025)
- ✅ Grid toolbar plumbing is live: `ViewportWidget` forwards snap sliders into `GridRenderSettings`, and `ViewportRenderer` skips the GPU pass entirely when `show_grid` is disabled (validated via `cargo test -p aw_editor --test ui_gizmo_smoke`).
- ✅ Telemetry capture + headless harness landed, giving us deterministic gizmo repros and assertions for commit/cancel flows.
- ✅ `EditorSceneState` now owns the authoritative ECS world; hierarchy, inspector, transform panel, viewport, and undo stack all operate on the same data. The new `tests/editor_scene_state.rs` suite covers transform application, cache sync, and snapshot restoration.
- ✅ This Known Issues digest has been refreshed and circulated for Week 1 sign-off.
- ⏭️ Next focus (Week 2): transform transactions + undo coalescing, snap unification, and tracing hooks per roadmap.

## 1. Gizmo Translation Snaps Back — **Resolved (Nov 16, 2025)**
- **Symptoms (historic)**: Dragging via `G` appeared to move entities, but releasing the mouse reset them because edits never reached the ECS world.
- **Fix**: `EditorSceneState` became the single source of truth. Viewport, hierarchy, inspector, transform panel, clipboard, and undo all borrow the same `World` instance via `EditorApp::with_world_and_undo_stack`. Legacy `EntityManager` data is no longer mutated. Integration tests in `tests/editor_scene_state.rs` verify round-trip fidelity (`apply_transform_updates_world_and_cache`, `sync_entity_reflects_direct_world_edits`, `snapshot_round_trip_restores_pose`).
- **Validation**: `cargo test -p aw_editor --test editor_scene_state` and `cargo test -p aw_editor --test ui_gizmo_smoke` both pass, demonstrating that gizmo commits persist and telemetry still captures the interaction timeline.

## 2. Grid Toggle and Snapping — **Resolved (Nov 15, 2025)**
- `GridRenderSettings` flows from the toolbar → widget → renderer → WGSL uniforms. Disabling the grid bypasses the render pass, and the spacing slider now mirrors the snap configuration.
- Remaining follow-ups feed into Week 2: expose major-line density + ensure gizmo snap math pulls directly from the shared `SnappingConfig` resource so visuals and math stay in lockstep.

## 3. Behavior Node Editor Is Static
- `EditorApp::show_behavior_graph_editor` renders a pre-canned `BehaviorNode` tree with no editable data model, node creation workflow, or serialization hooks into `astraweave_behavior::BehaviorGraph`.
- **Impact**: Designers cannot author AI behaviors inside the editor. Week 3 workstream introduces `BehaviorGraphDocument` plus save/load support.

## 4. Asset / Texture Import Missing
- `AssetBrowser` advertises drag-and-drop but never hands data off to the rest of the editor: `dragged_prefab` is only set inside `asset_browser.rs`.
- `PrefabManager::instantiate_prefab` remains unused outside tests, so there is no way to place assets into the live world. This blocks prefab workflows and scene assembly.

## 5. Play/Pause/Stop Does Not Drive Simulation
- `EditorMode` toggles booleans without deterministic snapshots. `simulation_playing` still rebuilds ad-hoc worlds instead of using the edited scene, and `runtime_world` drops user changes when exiting Play.
- Week 4 introduces `EditorRuntime` to own snapshots, deterministic ticks, and diff-merging back into `EditorSceneState`.

## 6. Prefab / Entity Sync Gaps
- Prefab overrides are neither detected nor visualized in the hierarchy. `EntityPanel` can show a prefab instance if one is manually looked up, but there is no Apply/Revert UX.
- Combined with Issue #4, prefab workflows remain blocked until asset drops spawn instances and overrides are tracked.

## 7. Regression Coverage Gaps
- We now have gizmo telemetry + the headless harness, yet broader regression coverage is still missing (prefab drag/drop, behavior authoring, play mode, HUD telemetry).
- Week 5 work targets a consolidated regression suite plus screenshot diffs to prevent future interaction regressions.

These findings seed the follow-up design documents (interaction fixes, authoring upgrades, simulation overhaul) and provide the reproducible evidence needed for each execution wave.
