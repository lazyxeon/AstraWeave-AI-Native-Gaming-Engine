# AW Editor – Recovery Roadmap

_Version 0.2.0 • Nov 16, 2025_

## Revision History
| Date | Version | Notes |
| --- | --- | --- |
| Nov 16, 2025 | 0.2.0 | Week 2 completion: snapping hub wiring, grid render gating, harness snapping tests, manual checklist logged |
| Nov 16, 2025 | 0.1.1 | Week 1 finishing touches: exported headless/telemetry APIs, new gizmo telemetry plumbing, validation evidence captured |
| Nov 16, 2025 | 0.1 | Initial recovery roadmap refresh aligned with Week 1 execution |

This roadmap sequences the work required to turn `tools/aw_editor` into a production-ready editor. It ties together the interaction, authoring, and simulation plans into five execution waves with explicit validation gates.

## Milestone Overview
| Week | Focus | Key Deliverables | Validation |
| --- | --- | --- | --- |
| 1 | Telemetry + Interaction Foundations | Known issues doc, headless gizmo harness, `EditorSceneState`, grid toggle plumbing | `cargo test -p aw_editor -- ui_gizmo_smoke` passes, grid toggle affects renderer |
| 2 | Gizmo/Grid Completion & Undo | Transform transactions, undo integration, tracing hooks, documentation update | Manual checklist (move, snap, undo/redo) signed off; tracing logs visible |
| 3 | Authoring Surface | Behavior graph editor MVP, asset drag/drop, prefab override UI | Round-trip behavior graphs; prefab apply/revert tests |
| 4 | Simulation Overhaul | `EditorRuntime`, deterministic snapshots, HUD + step controls | Play/Pause/Stop tests (`play_mode.rs`), manual UAT of level playback |
| 5 | Polish & Reporting | Performance metrics, README refresh, regression suite, master docs | Benchmark dashboard updated, Master Roadmap/Coverage revisions |

## Workstreams
1. **Interaction Track** (Weeks 1-2)
   - Implement `EditorSceneState` and retire the mock `EntityManager` for ECS data.
   - Refactor gizmo → undo pipeline per `AW_EDITOR_INTERACTION_PLAN.md`.
   - Connect toolbar toggles to renderer + gizmo snapping.
2. **Authoring Track** (Week 3)
   - Build new behavior-graph editor and asset ingestion pipeline (`AW_EDITOR_AUTHORING_PLAN.md`).
   - Prefab override visualization + apply/revert commands.
3. **Simulation Track** (Week 4)
   - Replace booleans with `EditorRuntime` per `AW_EDITOR_SIMULATION_PLAN.md`.
   - HUD + telemetry to observe deterministic tick loop.
4. **Quality & Docs** (Week 5)
   - UI smoke tests covering gizmo, asset drops, play mode.
   - Update `docs/current/MASTER_ROADMAP.md`, `MASTER_BENCHMARK_REPORT.md`, `MASTER_COVERAGE_REPORT.md` once features land.

## Phased Execution Plan (Weeks 1-5)

Each week focuses on a constrained set of issues. For every issue we define context, solution options, the recommended path, validation requirements, and the exact automated/manual tests that must pass before moving forward.

### Week 1 – Telemetry + Interaction Foundations

| Item | Details |
| --- | --- |
| **Issue** | Gizmo edits diverge from ECS world because `EntityManager` is a disconnected mock and no deterministic capture exists. |
| **Context** | `viewport/widget.rs` mutates `World`, while hierarchy/transform panels read only from `EntityManager`. Undo commands fail because inputs are inconsistent. |
| **Suggested Solutions** | (A) Introduce `EditorSceneState` as shared source of truth; (B) Patch `EntityManager` to mirror `World` every frame; (C) Gate gizmo editing behind play mode. |
| **Recommended** | Option A—implement `EditorSceneState` (struct + APIs) that wraps `World`, selection, prefab handles, and provides change notifications. Retire `init_sample_entities` once state is live. |
| **Implementation Steps** | 1. Define `EditorSceneState` module with `load_from_world`, `apply_gizmo_delta`, `selection_mut`. 2. Replace `entity_manager` references in hierarchy/transform panels with the new state. 3. Forward viewport gizmo updates through `EditorSceneState` so the same data drives rendering, ECS, and undo commands. 4. Re-export the headless harness + telemetry APIs from the crate so tests/benches can consume the canonical workflow. |
| **Success Criteria** | - Gizmo move/rotate/scale keeps entity positions after release. <br> - Hierarchy panel updates instantly when entities move. <br> - Undo/redo replay transforms without divergence. |
| **Testing** | - `cargo test -p aw_editor --test ui_gizmo_smoke` (updated to assert `EditorSceneState` sync). <br> - New unit test `tests/editor_scene_state.rs::sync_round_trip` verifying World⇄State round trips stay deterministic. <br> - Manual checklist: drag entity, undo, redo, ensure transforms persist. |

| Item | Details |
| --- | --- |
| **Issue** | Grid/snapping controls previously only affected UI; telemetry absent. |
| **Context** | While grid toggle wiring landed, there is no structured tracing to debug gizmo commits. |
| **Suggested Solutions** | (A) Add structured `tracing` spans for gizmo start/confirm/cancel; (B) Add console logging only; (C) Build a Tracy layer. |
| **Recommended** | Option A with optional Tracy export. Emit spans `aw_editor.gizmo.start`, `...commit`, `...cancel` including entity id, mode, constraint. |
| **Implementation Steps** | 1. Feed gizmo start/commit/cancel events into the telemetry buffer used by the headless harness. 2. Re-export telemetry helpers from `aw_editor` so smoke tests import a single module. 3. (Optional follow-up) Layer `tracing::span!` instrumentation inside `ViewportWidget::handle_input` for live sessions. |
| **Success Criteria** | - Headless gizmo harness captures start/commit/cancel events for every scripted drag. <br> - Grid toggle metrics appear in `aw_editor_trace.jsonl` (deferred to Week 2 tracing work). |
| **Testing** | - `ui_gizmo_smoke` parses the telemetry buffer to assert commit/cancel flows. <br> - `tests/editor_scene_state.rs` validates state/undo fidelity, ensuring telemetry events reflect actual world mutations. |

### Week 2 – Gizmo/Grid Completion & Undo

| Item | Details |
| --- | --- |
| **Issue** | Undo/redo stack lacks transform aggregation and spam commands per frame. |
| **Context** | `MoveEntityCommand` pushes new entries every mouse movement; there is no coalescing by drag session. |
| **Suggested Solutions** | (A) Add command coalescing with `peek_last_mut`; (B) Introduce high-level `TransformTransaction`; (C) Disable undo for gizmos. |
| **Recommended** | Option B—`TransformTransaction` capturing start snapshot, streaming deltas, and final commit. |
| **Implementation Steps** | 1. Add `TransformTransaction` helper storing `Entity`, `start_pose`, `pending_pose`. 2. Modify gizmo confirm/cancel to interact with the transaction API. 3. Update undo stack to support `TransformTransaction::into_command()`. |
| **Success Criteria** | - Single undo step per drag regardless of frame count. <br> - Cancelling a drag leaves no undo entry. |
| **Testing** | - New integration test `tests/undo_transactions.rs` simulating drag confirm/cancel. <br> - Update `ui_gizmo_smoke` to assert undo depth. |

| Item | Details |
| --- | --- |
| **Issue** | Grid snapping still diverges between renderer visuals and gizmo math in edge cases. |
| **Context** | Renderer pulls spacing from toolbar; gizmo uses internal `grid_snap_size`. |
| **Suggested Solutions** | (A) Central `SnappingConfig` resource broadcast via `EditorSceneState`; (B) Keep duplication but sync via polling; (C) Remove snapping. |
| **Recommended** | Option A. |
| **Implementation Steps** | 1. Move `SnappingConfig` into `EditorSceneState`. 2. Inject shared reference into toolbar + gizmo. 3. Add `EditorSceneState::subscribe_snap_changes` for renderer. |
| **Success Criteria** | - Disabling grid removes snap hints + GPU pass. <br> - Changing spacing updates renderer and gizmo within one frame. |
| **Testing** | - Extend harness to toggle snap and confirm serialized frame data matches expected layout. <br> - Add WGSL snapshot test `tests/grid_render.rs` to ensure grid buffer is skipped when disabled. |

### Week 3 – Authoring Surface

| Item | Details |
| --- | --- |
| **Issue** | Behavior graph panel is a static demo and cannot author data. |
| **Context** | `behavior_graph` is a literal tree; no serialization to `astraweave_behavior::BehaviorGraph`. |
| **Suggested Solutions** | (A) Build new graph data model + persistence to `.behavior.ron`; (B) Embed existing CLI editor; (C) Hide feature. |
| **Recommended** | Option A with incremental features. |
| **Implementation Steps** | 1. Add `BehaviorGraphDocument` struct with nodes/edges referencing `astraweave_behavior`. 2. Implement add/delete/link UI backed by the document. 3. Support save/load (RON) and round-trip validation with `BehaviorGraph::from_document`. 4. Integrate with prefab toolbar so dropping a behavior asset binds it to selected entity. |
| **Success Criteria** | - Users can create nodes, connect them, save, reload, and see the same structure. <br> - Binding a behavior to an entity writes component data into `World`. |
| **Testing** | - New test `tests/behavior_editor.rs::round_trip` checking serialization. <br> - UI harness `cargo test -p aw_editor --test behavior_ui_smoke` driving add/delete actions. |

| Item | Details |
| --- | --- |
| **Issue** | Asset browser drag/drop never instantiates prefabs/assets. |
| **Context** | `dragged_prefab` is never consumed; `PrefabManager::instantiate_prefab` unused. |
| **Suggested Solutions** | (A) Handle drops inside viewport to spawn prefabs; (B) Convert to right-click menu; (C) Remove prefabs. |
| **Recommended** | Option A. |
| **Implementation Steps** | 1. When user drops an asset over the viewport, call `AssetBrowser::take_dragged_prefab`. 2. Pass path into `PrefabManager::instantiate_prefab`, insert into `EditorSceneState`. 3. Track overrides via `PrefabInstance::track_override` on edits, and surface apply/revert buttons in `EntityPanel`. |
| **Success Criteria** | - Dragging `.prefab.ron` spawns the prefab at cursor with undo support. <br> - Prefab overrides appear in inspector with Apply/Revert actions. |
| **Testing** | - Automated drag/drop scenario using egui input playback (`tests/prefab_drag_drop.rs`). <br> - Unit tests for `PrefabManager::instantiate_prefab` mocking `World`. |

### Week 4 – Simulation Overhaul

| Item | Details |
| --- | --- |
| **Issue** | Play/Pause/Stop merely toggle booleans and rebuild worlds, discarding edits. |
| **Context** | `simulation_playing` and `editor_mode` are flags; `sim_world` is recreated each tick. |
| **Suggested Solutions** | (A) Implement `EditorRuntime` with deterministic snapshots; (B) Move simulation to separate crate; (C) Disable play mode. |
| **Recommended** | Option A per `AW_EDITOR_SIMULATION_PLAN.md`. |
| **Implementation Steps** | 1. Create `EditorRuntime` struct handling `enter_play`, `tick`, `exit_play`, `step_frame`. 2. Capture snapshot via `SceneData` and feed into runtime on Play. 3. After Stop, merge runtime diff back into `EditorSceneState`. 4. Hook HUD controls (play/pause/step) into runtime APIs. |
| **Success Criteria** | - Play preserves current scene, runs deterministically for 200 ticks, and restores edits when stopped. <br> - Step button advances exactly one frame with matching hashes to auto-test harness. |
| **Testing** | - New integration test `tests/play_mode.rs` running runtime for N ticks and comparing hashes. <br> - Headless scenario `cargo test -p aw_editor --test runtime_determinism`. |

| Item | Details |
| --- | --- |
| **Issue** | HUD lacks telemetry and cannot display runtime metrics. |
| **Context** | Right panel shows static placeholders. |
| **Suggested Solutions** | (A) Feed runtime metrics (frame time, entity count) from `EditorRuntime`; (B) Bind to external Tracy session; (C) Remove panel. |
| **Recommended** | Option A. |
| **Implementation Steps** | 1. Expose `RuntimeStats` struct from `EditorRuntime`. 2. Update `PerformancePanel` to poll stats every frame. 3. Add timeline chart for CPU/GPU budgets. |
| **Success Criteria** | - Performance panel updates in real time with actual measurements. <br> - Budget warnings trigger when frame time > 16.7 ms. |
| **Testing** | - Unit test `performance_panel.rs::updates_from_stats`. <br> - Manual UAT: run profiling scenario, verify warnings appear. |

### Week 5 – Polish, Reporting, and Regression Net

| Item | Details |
| --- | --- |
| **Issue** | Editor has no automated regression coverage spanning gizmo, prefab, play mode, and authoring surfaces. |
| **Context** | Only ad-hoc smoke tests exist. |
| **Suggested Solutions** | (A) Build consolidated regression suite triggered via `cargo test -p aw_editor --all-features`; (B) Rely on manual QA. |
| **Recommended** | Option A. |
| **Implementation Steps** | 1. Create `tests/regression_suite.rs` orchestrating scripted egui sessions (move entity, spawn prefab, edit behavior). 2. Add screenshot diff harness leveraging `aw_editor_headless`. 3. Integrate into CI with `cargo test -p aw_editor --tests`. |
| **Success Criteria** | - Regression suite passes headless. <br> - Visual diffs < 1% pixel delta for canonical scenes. |
| **Testing** | - `cargo test -p aw_editor --test regression_suite`. <br> - Visual regression script `scripts/run_editor_vr.ps1`. |

| Item | Details |
| --- | --- |
| **Issue** | Master documentation and benchmark reports lag behind new editor capabilities. |
| **Context** | Reporting policy requires updates when workstreams land. |
| **Suggested Solutions** | (A) Automate report generation; (B) Manual updates; (C) Skip. |
| **Recommended** | Option A/B hybrid: script outlines and manually edit final text. |
| **Implementation Steps** | 1. Extend `scripts/update_master_docs.rs` to ingest editor metrics/tests. 2. After each week, regenerate doc scaffolds and fill deltas before committing. 3. Track revisions in `docs/current/MASTER_ROADMAP.md`/`MASTER_BENCHMARK_REPORT.md`/`MASTER_COVERAGE_REPORT.md`. |
| **Success Criteria** | - All three master docs show new version numbers referencing aw_editor milestones. <br> - Benchmarks include editor frame time + interaction throughput. |
| **Testing** | - Run `cargo run -p tools::doc_linter` (if available) or `scripts/validate_docs.ps1`. <br> - Manual review checklist verifying version bumps + revision table entries. |

## Testing Strategy
- **Unit / Integration**: add target-specific tests mentioned in each plan (`ui_gizmo`, `asset_ingest`, `play_mode`).
- **Headless Harness**: new CLI scenarios scripted via egui input playback to ensure regressions are reproducible.
- **Manual UAT**: curated checklist stored in `tools/aw_editor/README.md` (or `docs/current/AW_EDITOR_UAT.md`) run at the end of each week.

## Risks & Mitigations
- **Large Refactors**: Interaction changes touch viewport, gizmo, entity storage. Mitigate by staging behind feature flags and landing incremental PRs.
- **Asset Pipeline Complexity**: rely on existing `asset_database` crates and scripts to avoid reinventing importers.
- **Deterministic Snapshots**: reuse `SceneData` + `WorldSnapshot` utilities validated in Phase 6/7 to reduce serialization bugs.

## Reporting
- After each week, log progress in `docs/journey/weekly/PHASE_8_UI_WEEK_[n]_REPORT.md` (or similar) to keep the experiment traceable.
- Update benchmark/history JSONL files when editor metrics improve so dashboards stay accurate.

This roadmap is the execution contract for bringing aw_editor up to world-class standards. Each referenced plan provides the detailed design for its workstream; this document keeps the sequence and validation visible across the project.

## Week 1 Progress – Nov 16, 2025

- ✅ Telemetry capture, structured gizmo events, and the headless gizmo harness are live (`cargo test -p aw_editor --test ui_gizmo_smoke`).
- ✅ Grid toggle + snap slider drive both renderer and snapping math; disabling the grid skips the GPU pass entirely.
- ✅ `EditorSceneState` now drives hierarchy, inspector, transform panel, viewport, and undo. Legacy `EntityManager` data is no longer mutated, and `tests/editor_scene_state.rs` validates round-trip fidelity.
- ✅ Known Issues digest (`docs/current/AW_EDITOR_KNOWN_ISSUES.md`) refreshed Nov 16 with Issue #1 marked resolved and circulated for sign-off.
- ⏭️ Ready to enter Week 2 scope (transform transactions, snap unification, tracing hooks) now that Week 1 deliverables are closed.

### Week 1 Validation Evidence (Nov 16, 2025)

- `cargo test -p aw_editor --test editor_scene_state` – PASS (covers `apply_transform_updates_world_and_cache`, `sync_entity_reflects_direct_world_edits`, `snapshot_round_trip_restores_pose`).
- `cargo test -p aw_editor --test ui_gizmo_smoke` – PASS after the crate exported `headless`, `interaction`, and `telemetry` modules and `GizmoState` began persisting the last active operation for accurate telemetry.
- Telemetry events now route through `aw_editor::record_telemetry`, and the headless harness’ selection/drag helpers mirror the UI path, ensuring Week 1’s determinism goals are validated outside of egui sessions.

## Week 2 Progress – Nov 16, 2025

- ✅ Transform transactions now aggregate entire drags into single undo entries with cancellation safeguards. The regression suite in `tools/aw_editor/tests/undo_transactions.rs` plus the updated `ui_gizmo_smoke` cases ensure we never spam the undo stack and that cancels leave history untouched.
- ✅ Snap hub instrumentation broadcasts every grid/snap change through `EditorSceneState::update_snapping`, emitting both telemetry events and structured tracing spans (`aw_editor.grid.update`). The new `tests/grid_render.rs` harness scenarios drive snapping toggles through the same headless workflow to prove parity with the viewport.
- ✅ Gizmo commit/cancel paths now emit `tracing` spans (`aw_editor.gizmo.*`) with entity/op metadata, the viewport renderer gates the grid pass using the shared `SnappingConfig`, and the `GridRenderer` shader now consumes the live grid size so visuals and math finally stay in lockstep.
- ✅ Manual checklist captured in `docs/current/AW_EDITOR_UAT.md` (Nov 16 run) covers move/snap/cancel/undo scenarios with tracing artifacts attached, matching the success criteria for Week 2 validation gates.
- ✅ Validation: `cargo test -p aw_editor` (Nov 16) exercises the full suite, including new `tests/grid_render.rs`, `tests/undo_transactions.rs`, and the refreshed smoke tests. Zero failures keep the Week 2 scope green while we set up Week 3 authoring work.

### Week 2 Validation Evidence – Nov 16, 2025

- `cargo test -p aw_editor` – PASS (lib/bin/tests + `tests/grid_render.rs`, `tests/ui_gizmo_smoke.rs`, `tests/undo_transactions.rs`, `tests/editor_scene_state.rs`).
- Renderer gating verified by the new `grid_pass_enabled` unit test plus harness coverage in `tests/grid_render.rs` that toggles snapping and confirms world poses snap vs. free-move cases.
- Telemetry + manual confirmation logged in `docs/current/AW_EDITOR_UAT.md`, including `aw_editor.gizmo.*` span captures for move/snap/cancel scenarios.
- Grid snap hub change notifications validated through `scene_state::tests::snapping_updates_emit_grid_event`, ensuring UI, renderer, and telemetry surfaces observe identical state transitions.

## Week 3 Progress – Nov 16, 2025

- ✅ Prefab drag/drop is now fully wired: viewport drops snap to the shared snapping hub, respect play-mode guard rails, emit undo/redo entries through the new `spawn_prefab_with_undo` helper, and immediately select/highlight the spawned instance.
- ✅ Prefab override tracking is automatic—viewport gizmo commits, inspector component edits, and transform-panel tweaks now call into `PrefabManager::track_override_snapshot`, keeping Apply/Revert buttons and inspector messaging in sync with real edits.
- ✅ Viewport exposes frame events (gizmo commit metadata) so higher-level UI can respond without polling internal gizmo state, completing the “selection sync + override tracking” acceptance criteria for Week 3.
- ✅ Regression coverage expanded with `prefab_spawn_helper_records_undo_entry` and `prefab_manager_tracks_override_from_snapshot`, proving prefab drops feed the undo stack and that override badges update without reopening inspectors.

### Week 3 Validation Evidence – Nov 16, 2025

- `cargo test -p aw_editor` (Nov 16) – PASS, including the new prefab workflow cases and the viewport drop helper coverage.
- Manual drop checklist appended to `docs/current/AW_EDITOR_UAT.md`, capturing edit-mode vs. play-mode drops, undo/redo of spawned prefabs, and inspector Apply/Revert state transitions using the new override tracking hooks.
