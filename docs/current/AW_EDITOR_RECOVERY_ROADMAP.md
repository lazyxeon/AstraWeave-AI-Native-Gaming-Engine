# AW Editor – Recovery Roadmap

_Version 0.1 • Nov 15, 2025_

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

## Week 1 Progress – Nov 15, 2025

- ✅ Telemetry capture, structured gizmo events, and the new headless harness are live (`cargo test -p aw_editor --test ui_gizmo_smoke`).
- ✅ Grid toggle + snap slider now directly drive the renderer: disabling the grid skips the GPU pass entirely, and spacing matches the toolbar/snapping configuration.
- 🔄 Remaining Week 1 tasks: land `EditorSceneState` + retire the mock `EntityManager`, then circulate the Known Issues digest for sign-off.
