# AstraWeave Roadmap — From AI-Native Prototype to Production Engine (UE5-Class)

Goal: Evolve AstraWeave into a full, production-grade, AI-native game engine on par with Unreal Engine 5—where AI is first-class (not a plugin), with a modern renderer, robust tooling, editor, and scalable runtime. All milestones below are scoped to the existing Rust + wgpu + rapier + egui stack.

Principles
- AI-first: Perception → Reasoning → Planning → Action is built-in and deterministic; tools are validated, no AI “cheating.”
- Deterministic sim: Fixed-tick ECS with strong validation boundaries; multiplayer-friendly and replayable.
- Modular workspace: Isolated crates per system; clear public APIs; testable in isolation; minimal unsafe.
- Dev velocity: Hot-reload where safe (scripts, assets, materials), fast iteration loops, strong instrumentation.

✅ Phase 0 (0–3 months): Stabilize Core and Ship a Visual Hello-World
Deliverables
- Rendering MVP: astraweave-render produces a lit scene (PBR, HDR, tonemapping, sky, shadows) via wgpu 0.20 (or latest compatible).
- Scene I/O: glTF 2.0 import (meshes, materials, textures); simple scene graph; GPU resource manager; frustum culling; basic instancing.
- Animation v0: Skeletal playback (simple blend, GPU skinning).
- Editor Shell: `tools/aw_editor` rebuilt using eframe/egui with dockable panels (Scene, Hierarchy, Inspector, Console, Profiler); live reload of material/shader params.
- AI Loop: Keep current `WorldSnapshot` / `PlanIntent`; unify `Orchestrator` API; CI example that runs end-to-end without panics.
- Fix/green core: make check, minimal lint set; working examples curated; broken demos quarantined.

DoD
- Build flags: wgpu backends guarded via cfgs; shader features togglable.
- Tests: snapshot tests for AI plans; renderer golden images for 2 scenes; importer round-trip tests.
- Benches: Criterion for world tick and render submit; baseline committed.
- Assets: sample PBR materials and a skinned character included under permissive license.
- Docs: `docs/Interfaces.md`, `docs/Perf Budgets.md`, and `docs/Asset IDs & Cache.md` linked from README.
- CI: `make ci` enforces fmt/clippy/tests/benches thresholds.

Key Crates/Changes
- New: `astraweave-scene`, `astraweave-materials`, `astraweave-anim`, `astraweave-asset` (importers), `astraweave-shaders` (WGSL).
- Update: `astraweave-render` (PBR + shadows + post), `tools/aw_editor` (panelized), `examples/visual_3d` (MVP scene viewer).

Exit Criteria
- Run: `cargo run -p visual_3d` shows a PBR scene with a skinned character at 60 FPS on mid-tier GPU.
- Editor: can load a scene, tweak a material, toggle lights, and save.

✅ Phase 1 (3–6 months): AI-Native Gameplay and Authoring Tools
Deliverables
- AI Orchestrator Suite: Rule-based + Utility/GOAP + LLM orchestrator behind `Orchestrator` trait; switchable per agent.
- Tool Sandbox v1: Formalized action verbs with validation categories (nav, physics, resources, visibility); comprehensive error taxonomy.
- Persona/Memory v1: `astraweave-persona` + `astraweave-memory` stabilized (profiles, episodic facts, skills); deterministic serialization and versioning.
- Nav & Physics: Rapier 3D character controller; navmesh baking (off-thread) + A*; cost layers; link with validation.
- Dialogue & Audio v1: `astraweave-audio` spatial audio; dialogue runner with branching; TTS integration hook; subtitle system.
- Authoring: Visual Behavior Graph (BT/HTN) panel in editor; Rhai hot-reload for behaviors; quest/dialogue graphs.

Key Crates/Changes
- New: `astraweave-behavior` (BT/HTN graphs), `astraweave-dialogue`, `astraweave-quests`.
- Update: `astraweave-ai`, `astraweave-nav`, `astraweave-physics`, `astraweave-audio`, `tools/aw_editor` (graph editors).

DoD
- Tests: unit tests for Tool Sandbox error taxonomy; persona/memory serialization versioned.
- Benches: AI fast-think ≤ 2 ms median; dialogue pipeline latency budget enforced.
- Examples: `hello_companion_3d` runs to completion without panics; audio/dialogue CLI demo produces VO or text barks offline.
- Docs: Behavior/quests pages, editor panel docs, error taxonomy referenced.

Exit Criteria
- Example “hello_companion_3d”: companion perceives, plans, and executes in a small 3D level; voice line triggers; player orders via UI.
- Editor: author a simple quest + behavior graph and run it live.

✅ Phase 2 (6–12 months): Modern Renderer and Asset Pipeline (UE5-Class Targets)
DoD
- Golden images updated for new features; ΔE thresholds respected.
- Benches: GPU frame ≤ 7 ms median on target GPU; shader compile cache hit-rate > 90%.
- Tools: material editor round-trip tests for WGSL outputs.
Rendering
- PBR 2.0: Clearcoat, anisotropy, transmission; clustered/forward+ lighting; cascaded shadow maps; SSR, SSAO/ASSAO, SSGI (probe-based); bloom/DoF/motion blur.
- GI: DDGI (probe volumes) + SDFGI fallback; lightmap baking path (GPU compute) for static scenes.
- Visibility: GPU-driven culled draws (compute binning) and meshlet/cluster culling; bindless resources where supported.
- Textures: Virtual Texturing (clipmap/mega-texture) for massive worlds; streaming & residency.
- VFX: GPU particle system; decals and screen-space effects.

Asset & Tooling
- Asset pipeline: deterministic importers (glTF, FBX via external tool, WAV/OGG, OTF/TTF), GUIDs, dependency graph, incremental builds, cache.
- Material Editor: node-based graphs compiled to WGSL; preview viewport; parameter collections; material variants.
- Animation 2.0: Retargeting, blend trees, IK (FABRIK/CCD), root motion; animation events.

Exit Criteria
- Visual benchmark scene parity: lighting/features comparable to a mid-level UE5 scene (no Nanite/Lumen yet) at 60 FPS 1080p on a mid-tier GPU.
- Editor: import a mid-complexity scene, edit materials, bake a light cache, profile frame breakdown.

✅ Phase 3 (12–18 months): Multiplayer, Determinism, and Scale
DoD
- Net sim soak test: 10-minute run without desync at 60 Hz; packet loss scenarios covered.
- Replays deterministic across platforms; snapshot delta size within budget.
Deliverables
- Netcode: Server-authoritative with client prediction & reconciliation; snapshot delta encoding; interest management; region matchmaking; replay capture.
- Deterministic Sim: audited fixed-tick path; lockstep option; authoritative tool validation across clients.
- Save/Live Ops: robust save/versioning (`aw-save`), migration; asset patching and hotfix pipeline; crash reporting hooks.
- Security: anti-cheat hooks (server-side validation + telemetry); sandboxing for scripting.

Exit Criteria
- 4-player co-op demo in a medium-sized level with AI companions; 60 Hz sim; jitter-buffered voice/text; no desync under packet loss; stable replays.

Phase 4 (18–24 months): Production Polish and Extensibility
DoD
- SDK headers generated and validated with a C harness; semantic versioning gates in CI.
- Cinematics editor saves/loads timelines; smoke test in CI.
Deliverables
- Cinematics: Timeline, tracks (anim, camera, audio, FX), cutscene editor; sequencer API.
- UI/UX: game UI framework (HUD, menus, inventory, map) with data-binding; accessibility options.
- Marketplace-Ready SDK: stable public APIs; versioning policy; plugin system for third parties; sample packs.
- Documentation & Samples: full docs site; 10+ polished sample projects (AI, networking, rendering, tools). 

Exit Criteria
- “Feature-complete” engine with editor; external team can build a vertical slice without engine dev support.

Cross-Cutting Workstreams (Continuous)
- Testing & CI: golden-image tests for renderer, snapshot tests for AI plans, physics regression, perf gates; `make ci` runs format, clippy (curated deny list), tests, benches.
- Performance: frame timing budgets; per-subsystem profilers; tracing to Chrome; capture & replay.
- Quality & Security: cargo-deny/audit clean; SBOM; reproducible builds; signed assets; defensively coded tools/LLM adapters.
- Content: exemplar assets (characters, terrain, materials) under permissive licenses for samples.

Technical Parity Mapping vs UE5 (Pragmatic Targets)
- Nanite → GPU-driven cluster culling + meshlet path, LOD streaming, VT for textures; no hardware mesh shaders requirement.
- Lumen → DDGI + SDFGI hybrid; baked fallback; SSR/SSGI augment; editor GI volumes.
- Blueprints → Visual Behavior/Graph editors backed by Rust/Rhai; node-based materials & animation graphs.
- Sequencer → Timeline/cinematic tracks with camera rig and event binding.
- Editor → eframe/egui-based multi-dock editor; later optional migration to winit multi-window if needed.

Initial Backlog (actionable next sprints)
- Render MVP: PBR BRDF (Disney-ish), CSM shadows, ACES tonemap; import glTF; draw submission with bindless-like structs.
- Editor: scene tree, inspector, console/logger, live material param editing; file watcher & hot-reload for WGSL/materials.
- AI: unify `Orchestrator` returning `PlanIntent`; validation error types; sample tool verbs + tests.
- Nav/Physics: Rapier CC; bake navmesh from scene collider mesh; runtime A*; validation integration tests.
- Audio: rodio 3D spatialization; mixer; event-driven cues.
- CI: workspace check with curated excludes; renderer headless test renders; input unit tests.

Resourcing & Risks
- Team: 1 rendering, 1 tools/editor, 1 AI/gameplay, 1 networking/systems, 1 content TD (part-time). Grow as needed.
- Risks: wgpu feature gaps (mesh shaders); mitigate with compute-based culling. LLM latency; mitigate with on-device 7B quantized + short-horizon planner.

KPIs & Success Metrics
- Editor cold start < 3s; hot reload < 500ms.
- Visual demo 1080p60 on 3060-class GPU.
- Co-op demo stable at 60Hz with < 120ms RTT; no desync over 10 min.
- CI: green on core + examples; perf regression < 5% week-over-week.

How To Track
- Create GitHub Projects board with swimlanes matching phases/workstreams.
- Each milestone produces: demo video, example crate, docs page, and CI validation.

## Subsystem Design Plans (MVP)

This section distills user requirements and aligns them with AstraWeave’s AI-first architecture. It defines per-subsystem goals, core structures, open decisions, and immediate next steps. Where relevant, see: `docs/engine-api.md`, `docs/ai-scripting.md`, `docs/networking.md`, `docs/persona-packs.md`, `docs/BUILD_QUICK_REFERENCE.md`, `SECURITY.md`, `SECURITY_AUDIT_GUIDE.md`, and crate-level READMEs.

### 1. AI Cognition Stack
Purpose
- Agents perceive, reason, plan, and act via validated engine verbs. Companions should feel human, learn player preferences, and persist memory across sessions. No “cheats”: all actions are validated by the engine.

Architecture
- Perception Bus: broadcast filtered `WorldSnapshot`s to agents at a cadence. MVP target: ~15 Hz in combat (10–20 Hz bound), lower in exploration. Redact out-of-LOS info and inject noise for fairness. See `astraweave-core` world snapshot docs.
- Planner: hybrid Utility/GOAP/HTN that decomposes goals into verb sequences; goals (assist, revive, cover fire, hide, rally, etc.) are scored by safety, progress, and player orders. “Deep-think” via LLM at low cadence (e.g., 2 Hz out of combat, once per large encounter) and “fast-think” via utility scoring each tick. See `astraweave-ai`.
- Tool Sandbox: registry of validated verbs and constraints: `move_to`, `cover_fire`, `dodge`, `converse`, `interact`, `use_item`, `stay`, `wander_freely`, `hide`, `rally_on_player`. Engine validates nav/physics/resources/LOS. See `docs/engine-api.md` Tool Validation.
- Behaviour Controller: merges planner output with cooldowns and player orders into atomic actions; enforces constraints and reparents failures to re-plan or fallbacks.
- Memory Fabric: persona traits, semantic facts, episodic memories, skills. Persist playstyle preferences (aggression/stealth bias), encountered facts (safehouses, tactics), episodes (clutch revives), skill progression. See `astraweave-memory`, `astraweave-persona`.
- Personas & Filters: presets (class clown, gritty outlaw, scholar, explorer) that shape tone/risk; apply rating filters (PG, PG-13, R) and toxicity/profanity filters to LLM outputs. See `docs/persona-packs.md`.

Open Decisions & Next Steps
- Finalize MVP verb registry (above). Define JSON/Protobuf schemas for snapshots and intents; ensure determinism and versioning.
- Cadence policy: deep-think opportunistic in exploration and per large encounter; fast-think every tick; document budgets.
- Implement persona presets and rating filters within the LLM adapter; add moderation gates in `astraweave-llm` when re-enabled.

### 2. Simulation Kernel & ECS
Purpose
- Deterministic fixed-tick simulation driving physics, navigation, AI, animation, and audio; decoupled from rendering.

Architecture
- Tick Model: run sim at 60 Hz; variable-rate renderer. Job order: physics → navigation → perception → cognition → behaviour → animation/audio. See `docs/engine-api.md`.
- Component Taxonomy (MVP):
	- Transform (model), Camera (view), Projection
	- Physics (velocity, mass, collider, layer/mask)
	- Health, Faction, Threat
	- Inventory & Abilities (items, cooldowns, stamina/resources)
	- AIState (goal stack, behaviour node, timers)
	- NavAgent (radius, height, path)
- Determinism: session RNG seed; fixed time steps; record/replay hooks for reproduction.

Open Decisions & Next Steps
- Lock job ordering above; prototype internal concurrency and benchmark.
- Use floating-point for positions/velocities with fixed-step determinism; revisit fixed-point if precision issues emerge.

### 3. Rendering & Camera
Purpose
- Realistic visuals with first-/third-person cameras, dynamic lighting, post-effects, and debug overlays.

Architecture
- Renderer: wgpu-based forward-plus/clustered forward; PBR, dynamic shadows, tone mapping, bloom; linear workflow with gamma-correct output. See `astraweave-render` and `docs/engine-api.md`.
- Camera Modes: free-fly (FPS), orbit (third-person), smooth transitions; camera shake; optional target cycling.
- Debug Overlays: egui overlays for AI plan trees, utility heatmaps, navmesh visualization, LOS rays, perf graphs; toggle via hotkeys.

Open Decisions & Next Steps
- Confirm tonemapping/HDR path (ACES) and lighting pipeline details; keep compatibility with current wgpu targets.
- Implement smooth camera mode switching and default keybinds; wire overlay toggles.

### 4. Physics & Interaction
Purpose
- Collision, movement, and damage with layers/masks and a robust character controller.

Architecture
- Collision Layers/Masks: Player, AI, World (static), Triggers, Projectiles. Matrix: Player ↔ World/AI/Projectiles; AI ↔ World/Player/AI/Projectiles; World ↔ Player/AI/Projectiles; Triggers overlap only; Projectiles ↔ Player/AI/World.
- Character Controller: walking, running, jumping, crouch/prone, step height ≈ 0.3 m, slope limit ≈ 45°. Capsule colliders; nav integration.
- Damage System: hitscan (raycast) and projectiles; fall damage by impact velocity; friendly fire toggle (off by default). Ragdolls deferred.

Open Decisions & Next Steps
- Include crouch/prone for cover/stealth in MVP; defer climbing/swimming.
- Define starter weapon categories and damage values; integrate with Tool Sandbox constraints.

### 5. Navigation
Purpose
- AI and players traverse large, varied environments efficiently.

Architecture
- Navmesh Baking: tiled navmesh from world geometry (Recast/Detour style); parameters for cell size, agent radius/height; off-mesh links (ladders, jumps); runtime updates for dynamic obstacles (doors, destructibles).
- Pathfinding: A* with optional hierarchical flow fields for large areas; cost modifiers for cover/hazard avoidance; path smoothing before feeding AI.
- Local Steering: basic avoidance/arrival behaviors to reduce agent overlap.

Open Decisions & Next Steps
- Choose dynamic navmesh granularity for doors/collapsing bridges/movable obstacles.
- Evaluate streaming/tiling for large worlds; implement only if needed.

### 6. Audio & Dialogue
Purpose
- Immersive audio (weapons, footsteps, ambience, VO) and LLM-driven dialogue with tone/rating filters.

Architecture
- Audio Categories: channels for weapons, footsteps, ambience, water/FX, voice; spatialize 3D sounds; low-pass behind obstacles.
- Music System: state machine crossfading between exploration, tension, and combat.
- Dialogue: TTS integration (open-source or commercial), with LLM text generation; respect rating filters; provide offline text bark fallback. See `astraweave-audio` and `docs/ai-scripting.md`.

Open Decisions & Next Steps
- Evaluate open-source TTS (e.g., Coqui/VITS) for offline synthesis; confirm licensing/quality.
- Decide on adaptive music stems vs simple crossfades post MVP shakeout.

### 7. Input & UI
Purpose
- KB/M and gamepad input with a clean HUD and accessibility options.

Architecture
- Input: cross-platform abstraction; default bindings for KB/M and Xbox/PlayStation; rebinding + profile save. Map A/B/X/Y (Cross/Circle/Square/Triangle) to interact/use/dodge; triggers aim/fire; bumpers companion commands.
- HUD: health, stamina, minimap, quest objectives, current item, inventory.
- Accessibility: subtitles toggle, text scaling, color-blind palettes, remappable controls.

Open Decisions & Next Steps
- Finalize KB/M and controller mappings; user test early.
- Consider radial menus or quick slots for inventory if usability improves.

### 8. Networking & IPC
Purpose
- Co-op multiplayer with server authority and low-latency AI; model routing via IPC.

Architecture
- Authoritative Server: server runs simulation; clients send inputs/AI intents; server validates/broadcasts. Transport via WebSockets (default) or UDP/QUIC if latency warrants. Target <120 ms RTT for actions; if deep-think exceeds budget, fallback to micro-policies. See `docs/networking.md`.
- Co-op Limits: MVP supports up to two human players and two companions each.
- Model IPC: run LLM locally by default; optional cloud for deep reasoning; timeouts + fallbacks; message bus/shared memory for snapshot → plan intent.

Open Decisions & Next Steps
- Benchmark latency; set deep-think timeouts and fallback behaviors (e.g., take cover if delayed).
- Confirm protocol default (WebSockets) and keep QUIC/UDP as an optimization path.

### 9. Persistence & Profiles
Purpose
- Save player progress, companion memories, and world changes with integrity and privacy.

Architecture
- Save Format: encrypted, signed `.cprof` for companion profiles and separate world saves; include versioning and migration hooks. Use AES-GCM with per-user keys. See `SECURITY.md`.
- What Persists: player stats, inventory, quests, world state (e.g., destroyed bridges), companion profiles; small vector index for fast memory lookup.
- Data Quotas: soft limits (e.g., 10 MB per companion profile) with pruning beyond threshold.

Open Decisions & Next Steps
- Key management: OS secure storage (Keychain, Windows Credential Vault); plan key rotation/revocation; see `SECURITY_AUDIT_GUIDE.md`.
- Compression: enable zstd for save files to reduce footprint.

### 10. Editor & Tooling
Purpose
- Real-time insight into AI decisions, memory, and determinism.

Architecture
- Live Plan View: visualize plan tree, goal stack, utility scores; heatmaps/bars; timing and rationale.
- Memory Inspector: persona attributes, facts, episodes, skills; manual pruning/distillation.
- Deterministic Replays: record/replay at frame granularity; diff tools for run comparison.
- Encounter Fuzzer: vary encounter layouts/enemy composition; record KPIs (time-to-clear, damage taken, assist rate).

Open Decisions & Next Steps
- Build plan view, memory inspector, deterministic replays, and fuzzer in egui; define logging for replay fidelity vs file size.

### 11. SDK & Bindings
Purpose
- Expose engine via a stable C ABI and bindings for integration with other frameworks.

Architecture
- API Surface: C ABI for world creation, system registration, snapshots in, intents out, and state queries; callback hooks for scripts/UI.
- Bindings: Rust primary; generate C headers (cbindgen); experimental Godot GDExtension; evaluate Unity/Unreal via C++ plugin later.
- Schemas: Protobuf for snapshots/intents/profiles with backward compatibility.

Open Decisions & Next Steps
- FFI & scripting: adopt cbindgen; integrate Rhai for quest logic in MVP.

### 12. Security, Privacy & Safety
Purpose
- Ensure fair play, protect data, and filter inappropriate content.

Architecture
- Action Validation: every AI action is validated against physics, LOS, cooldowns, and nav; invalid actions auto-repaired or rejected. Players and AI share systems. See `docs/engine-api.md`.
- LLM Guardrails: prompt sanitization and response filters; align to chosen rating (PG, PG-13, R); moderation override to mute/adjust persona if boundaries are crossed.
- Encrypted Saves & Profiles: encrypted and signed profiles with per-user keys; optional cloud sync.

Open Decisions & Next Steps
- Content moderation: select/build rating-aware classifier.
- Key policies: define key rotation/revocation and audit trails.

### 13. Performance, Benchmarks & CI
Purpose
- Meet performance targets and ensure cross-platform stability.

Architecture
- Perf Budgets per tick: physics + nav ≤ 1.5 ms; AI planner ≤ 2 ms (fast think) and ≤ 150 ms (deep think with fallback); rendering ≤ 7 ms for 60 FPS.
- Benchmark Suites: Criterion.rs for world creation, entity spawning, world tick, and input ops; track in CI and alert on >200% regressions. See `docs/BENCHMARKING_GUIDE.md`.
- CI Matrix: Linux/macOS/Windows; use `sccache` and multi-level caching to speed builds; see `docs/BUILD_QUICK_REFERENCE.md`.

Open Decisions & Next Steps
- AI & LLM benchmarks: add planning and LLM inference benchmarks; integrate into CI.
- Runtime monitoring: choose profilers (Tracy, OpenTelemetry) for CPU/GPU metrics.

### 14. Examples & Demo Roadmap
Purpose
- Polished examples to showcase features and serve as integration tests.

Proposed MVP Demos
- `hello_companion`: AI planning + validation; bring to green status.
- `adaptive_boss`: multi-phase boss with dynamic tactics; validate nav + planner.
- `navmesh_demo`: open-world pathfinding with dynamic obstacles; integrate camera + overlays.
- `unified_showcase`: combined camera, physics, AI, navmesh, audio; includes plan viewer and memory inspector.

Next Steps
- Fix broken examples and update dependencies; ensure curated workspace checks pass.
- Create a demo illustrating open-world manipulation with LLM-driven companions.

### 15. Asset Pipeline & Modding (Post-MVP)
Purpose
- Enable artists/modders to import content and script behaviors.

Guidelines
- Importers: glTF/FBX for models, Ogg/Flac for audio, PNG/JPEG for textures; leverage offline tools (Assimp) for conversions.
- Data-Driven Config: verbs, items, NPC stats, and quest logic in YAML/JSON; hot-reload in editor.
- Modding: post-MVP, expose content packs with signed manifests to prevent cheating; provide safe LLM prompt/persona guidelines.

Summary
- These subsystem plans emphasize determinism, AI fairness, persistent companions, and cross-platform support. Open items include encryption, protocol choices, FFI, and editor UX; the proposals align with current engine docs and architecture.

Appendix: Proposed Crates/Dirs
- `astraweave-scene/` (scene graph, transforms, cameras)
- `astraweave-asset/` (importers, GUIDs, dependency graph, cache)
- `astraweave-shaders/` (WGSL modules, shaderlib)
- `astraweave-materials/` (PBR, node graphs → WGSL compiler)
- `astraweave-anim/` (skeletal/IK, blend trees)
- `astraweave-behavior/` (BT/HTN graphs, runtime)
- `astraweave-dialogue/`, `astraweave-quests/`
- `examples/visual_3d/`, `examples/hello_companion_3d/`

This roadmap is intentionally staged: ship a viewable, editable, and AI-playable core quickly; iterate toward UE5-class rendering, tools, and multiplayer scale while preserving AstraWeave’s AI-native edge.
