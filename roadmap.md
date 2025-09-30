# AstraWeave Roadmap — Aligning with Leading Rust Game Engines

## Current Snapshot (Q2 2025)

### Foundations Already in Place
- ✅ Grid-based world, entity state, and deterministic tick scaffolding exist in `astraweave-core` (`World`, `Health`, `Team`, cooldown handling).【F:astraweave-core/src/world.rs†L1-L127】
- ✅ Shared AI data contracts (`WorldSnapshot`, `PlanIntent`, tool registry metadata) are codified and serializable for orchestration layers.【F:astraweave-core/src/schema.rs†L45-L193】
- ✅ A wgpu-based forward PBR prototype with cascaded shadows and normal mapping is implemented in `astraweave-render`, and the `visual_3d` example wires it to the current world state for interactive inspection.【F:astraweave-render/src/renderer.rs†L1-L200】【F:examples/visual_3d/src/main.rs†L1-L120】
- ✅ Initial asset ingestion stubs for glTF/GLB meshes and materials are present in `astraweave-asset`, providing a starting point for a structured asset pipeline.【F:astraweave-asset/src/lib.rs†L1-L200】
- ✅ Authoring/editor shell stubs (quest, dialogue, level docs) already exist in `tools/aw_editor`, anchoring future workflow tooling.【F:tools/aw_editor/src/main.rs†L1-L120】

### Gaps and Risks Blocking Engine Parity
- ⚠️ Core systems still rely on ad-hoc structs; there is no ECS schedule, component storage abstraction, or plugin boundary comparable to Bevy/Fyrox (e.g., `World` is a bespoke HashMap aggregate).【F:astraweave-core/src/world.rs†L29-L127】
- ⚠️ Critical gameplay/AI functionality was stubbed or duplicated: orchestrator implementations in `astraweave-ai` diverged between `lib.rs` and `orchestrator.rs`; the tool sandbox validator was unimplemented; capture/replay routines returned "Not yet implemented". These have been addressed in Phase 0 iteration 1: orchestrators unified, basic tool sandbox taxonomy in place, and minimal JSON capture/replay implemented with tests.【F:astraweave-ai/src/orchestrator.rs†L1-L200】【F:astraweave-ai/src/tool_sandbox.rs†L1-L120】【F:astraweave-core/src/capture_replay.rs†L1-L200】
- ⚠️ Observability and CI gates are aspirational: golden-image tests, deterministic replays, asset signing, and AI plan snapshot tests are only documented stubs with no automated enforcement.【F:astraweave-core/src/capture_replay.rs†L1-L16】【F:astraweave-ai/tests/plan_snapshot.rs†L1-L8】【F:tools/asset_signing.rs†L1-L16】
- ⚠️ Rendering, asset, and tooling crates are not yet unified under a render graph or asset database; there is no scheduler that integrates renderer, physics, AI, and networking in a deterministic frame loop.

---

## Phase 0 (0–1 months): Stabilize, Deduplicate, and Validate Baseline
**Objectives:** eliminate stubs, ensure repeatable builds/tests, and align nomenclature before layering new systems.

**Key Tasks**
1. Unify AI orchestrator interfaces: collapse duplicate implementations in `astraweave-ai/src/lib.rs` and `astraweave-ai/src/orchestrator.rs`, implement the rule/utility/GOAP planners, and cover them with deterministic tests. [Done]
2. Implement tool validation taxonomy in `astraweave-ai/src/tool_sandbox.rs`; add basic negative-path tests; wire deeper physics/nav checks in later phases. [Partial]
3. Deliver functional capture/replay for the core world state in `astraweave-core/src/capture_replay.rs` with JSON snapshots and replay stepping; add checksum/hashes later. [Done]
4. Replace placeholder returns in security/asset tooling with working signing/verification: introduced `tools/asset_signing` crate using Ed25519 over SHA-256 with a unit test. [Done]
5. Stand up continuous validation: `cargo check --all-targets`, `cargo fmt --check`, `cargo clippy --workspace`, unit tests, and golden image snapshots for the renderer gated via `make ci`.
6. Document and enforce workspace feature flags (renderer textures, asset import) to guarantee deterministic builds across platforms.

**Exit Criteria**
- All critical Phase 0 stubs replaced or tracked with tests: orchestrators [Done], capture/replay [Done], signing [Done], tool sandbox [Partial].
- CI pipeline blocks merges on format, lint, unit/integration tests, and renderer golden images. [Pending]
- Deterministic AI plan snapshots and world capture/replay succeed in automation. [Plan snapshots pending]

---

## Phase 1 (1–3 months): ECS & Simulation Core Parity
**Objectives:** evolve the bespoke world into a modular ECS with scheduling, reflecting the architecture of Bevy/Fyrox/Amethyst.

**Key Tasks**
1. Introduce an `astraweave-ecs` crate providing archetype-based storage, command buffers, and deterministic scheduling (or adopt `bevy_ecs` via a compatibility layer) and migrate `astraweave-core::World` onto it.
2. Define system stages (Perception → Simulation → AI Planning → Physics → Presentation) with explicit schedules and frame boundaries.
3. Implement resource injection, events, and fixed-timestep drivers comparable to Bevy's `App`/`Schedule` API, exposing plugin registration points across crates.
4. Port existing world interactions (spawning, cooldown ticks, LOS helpers) into ECS components/systems with coverage tests and benchmarks.
5. Provide migration utilities bridging legacy HashMap-backed saves to the new ECS layout.

**Exit Criteria**
- Simulation runs through an ECS-driven schedule with deterministic hash-locked component order.
- Plugins for AI, physics, rendering, and input register via a unified application builder.
- Benchmarks demonstrate stable frame times comparable to baseline HashMap implementation.

---

### Phase 1 progress update (Sep 2025)

What’s landed in this iteration:

- Deterministic ECS crate (`astraweave-ecs`)
	- Component storage backed by BTreeMap for stable iteration order
	- Fixed stages: perception → simulation → ai_planning → physics → presentation
	- Minimal `App` and `Plugin` APIs and a fixed-timestep driver, with unit tests

- Core ECS adapter (`astraweave-core::ecs_adapter`)
	- Bridges legacy `World` into the ECS schedule; ticks world time and mirrors cooldown decay
	- Simple movement system: moves entities toward `CDesiredPos` deterministically
	- LOS refresh placeholder system (hooks into legacy obstacles for now)
	- Added deterministic events resource (`ecs_events::Events<T>`) and emitted `MovedEvent`s from movement

- AI planning plugin (no core/AI cycles)
	- Introduced `astraweave-ai::AiPlanningPlugin` that registers a planner system into the `ai_planning` stage
	- Builds minimal snapshots from ECS components and uses the Rule orchestrator to set `CDesiredPos`
	- Convenience builder: `astraweave_ai::ecs_ai_plugin::build_app_with_ai(World, dt)` composes core schedule + AI plugin
	- All associated unit tests pass across `astraweave-core`, `astraweave-ecs`, and `astraweave-ai`

How to try it locally:

```powershell
cargo test -p astraweave-ecs -p astraweave-core -p astraweave-ai
```

Minimal integration pattern (code): use `ecs_adapter::build_app(world, dt)` and add the AI plugin, or call the helper:

- Add plugin explicitly: `build_app(world, dt).add_plugin(astraweave_ai::AiPlanningPlugin)`
- Or use helper: `astraweave_ai::ecs_ai_plugin::build_app_with_ai(world, dt)`

Remaining Phase 1 work (next iterations):

- Parity tests: compare ECS-driven movement/cooldowns vs legacy over N ticks; add golden replays
- Perception: switch planner snapshots to use `core::perception::build_snapshot` for richer inputs
- ECS ergonomics: small query helpers/macros; multi-component queries and filtered iteration
- Wire examples: integrate the AI plugin into one working example and capture smoke tests
- Events: expand event usage (e.g., `AiPlannedEvent`) and route into validation/telemetry
- Docs: expand developer docs around the ECS schedule, plugin patterns, and testing guidance

## Phase 2 (3–6 months): Rendering & Scene Graph Modernization
**Objectives:** harden the wgpu renderer into a modular render graph integrated with ECS, matching Bevy/Fyrox capabilities.

**Key Tasks**
1. Build a render graph abstraction in `astraweave-render` (graph nodes, resource handles, graph compiler) and integrate clustered lighting, shadowing, and post pipelines as graph passes.
2. Implement GPU resource lifetime management (bindless-like material/mesh registries, streaming textures) tied into ECS resource events.
3. Expand terrain, sky, and weather systems to operate through ECS components and renderer plugins.
4. Add render doc tests: golden images for static scenes, shader compile caches, and automated shader validation.
5. Wire ECS scene graph (`astraweave-scene`) into renderer instances, including hierarchical transforms and skinning uploads.

**Exit Criteria**
- Renderer runs headless golden-image tests with stable outputs across platforms.
- ECS-driven scene graph produces renderable instances via the render graph.
- Profiling captures (wgpu trace) integrated into CI for regression detection.

---

## Phase 3 (6–8 months): Asset Pipeline & Data Management
**Objectives:** deliver a deterministic asset database akin to Godot/Bevy asset servers.

**Key Tasks**
1. Extend `astraweave-asset` with dependency graph tracking, GUID assignment, hot-reload watchers, and import pipelines for glTF, textures, audio, and dialogue.
2. Introduce asset cooking/build steps (`tools/aw_asset_cli`) for offline processing, compression, and validation.
3. Integrate asset streaming into renderer/material subsystems with residency tracking.
4. Store asset metadata and hashes for reproducible builds; integrate signing/verification pipeline.
5. Provide asset inspection UI in `tools/aw_editor` and command-line status reports.

**Exit Criteria**
- Assets load through a central database with hot reload and dependency invalidation.
- CI verifies asset hashes, metadata completeness, and importer round-trip tests.
- Editor displays asset metadata and previews via ECS-powered viewers.

---

## Phase 4 (8–11 months): Authoring Tools & Workflow Integration
**Objectives:** evolve `tools/aw_editor` into a multi-dock authoring environment comparable to Godot/Bevy editors.

**Key Tasks**
1. Implement docking, scene hierarchy, inspector, console, and profiler panels in `aw_editor`, fed by ECS state snapshots.
2. Embed graph editors for behavior trees, dialogue, and quests with live validation hooks into `astraweave-behavior`, `astraweave-dialogue`, and `astraweave-quests`.
3. Enable live material/shader editing with hot reload via the asset pipeline and renderer graph.
4. Integrate terrain/biome painting, navmesh baking controls, and simulation playback.
5. Provide collaborative-friendly save formats and diff tooling for authored assets.

**Exit Criteria**
- Editor sessions can author a scene, save assets, adjust materials, and trigger AI validation without restarting.
- Automated UI smoke tests (via wgpu headless backend) run in CI.

---

## Phase 5 (11–14 months): AI, Gameplay, and Systems Depth
**Objectives:** achieve AI/gameplay feature parity with precedent engines' gameplay modules.

**Key Tasks**
1. Implement full tool validation categories (nav, physics, resources, visibility) and integrate with Rapier/navmesh data.
2. Flesh out behavior trees/HTN in `astraweave-behavior`, hooking into ECS events and orchestrators.
3. Expand persona/memory persistence with deterministic serialization and versioning.
4. Integrate LLM planning with guardrails (schema validation, sandboxing) and fallback heuristics.
5. Deliver gameplay modules (combat, crafting, quests) as ECS plugins with deterministic tests.

**Exit Criteria**
- AI agents operate through validated plans with deterministic outcomes across runs.
- Tool sandbox enforces safety constraints and logs telemetry for debugging.
- Gameplay feature tests (combat, quests, dialogue) pass in CI.

---

## Phase 6 (14–18 months): Networking, Persistence, and Scale
**Objectives:** reach multiplayer-ready fidelity similar to Amethyst/Godot networking stacks.

**Key Tasks**
1. Finalize `aw-net` crates with server-authoritative snapshot/rollback, interest management, and secure serialization.
2. Implement deterministic replay + save/load integration via `persistence/aw-save` tied to ECS state snapshots.
3. Harden security: sandbox scripting, enforce anti-cheat hooks, and integrate telemetry exporters.
4. Stress-test large scenes/AI loads; add automated soak tests for netcode and save systems.

**Exit Criteria**
- 4-player deterministic demo with AI companions runs without desync in CI soak tests.
- Save/replay flows validated across platform targets with checksum verification.

---

## Phase 7 (18–24 months): Observability, Packaging, and Ecosystem
**Objectives:** polish for production adoption and third-party extensibility.

**Key Tasks**
1. Establish observability stack (tracing, metrics, crash reporting) integrated into editor and runtime builds.
2. Publish SDK artifacts (C ABI via `astraweave-sdk`), plugin templates, and documentation site.
3. Provide sample projects demonstrating vertical slices, automation to build distributable demos, and marketing assets.
4. Formalize semantic versioning, release automation, and long-term support cadence.

**Exit Criteria**
- External teams can author content using published SDK/docs without engine modifications.
- Release pipelines produce signed binaries, documentation, and sample content automatically.

---

## Continuous Workstreams
- **Quality & Security:** Maintain cargo-audit/deny, SBOM generation, secret scanning, and hardened LLM adapters.
- **Performance & Observability:** Track ECS/renderer benchmarks, integrate tracing and frame capture tooling, and enforce performance budgets in CI.
- **Documentation & Developer Experience:** Keep docs aligned with roadmap phases, publish migration guides, and provide reproducible setup scripts.
