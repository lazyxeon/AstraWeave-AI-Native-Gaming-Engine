# AstraWeave Roadmap — From AI-Native Prototype to Production Engine (UE5-Class)

Goal: Evolve AstraWeave into a full, production-grade, AI-native game engine on par with Unreal Engine 5—where AI is first-class (not a plugin), with a modern renderer, robust tooling, editor, and scalable runtime. All milestones below are scoped to the existing Rust + wgpu + rapier + egui stack.

Principles
- AI-first: Perception → Reasoning → Planning → Action is built-in and deterministic; tools are validated, no AI “cheating.”
- Deterministic sim: Fixed-tick ECS with strong validation boundaries; multiplayer-friendly and replayable.
- Modular workspace: Isolated crates per system; clear public APIs; testable in isolation; minimal unsafe.
- Dev velocity: Hot-reload where safe (scripts, assets, materials), fast iteration loops, strong instrumentation.

Phase 0 (0–3 months): Stabilize Core and Ship a Visual Hello-World
Deliverables
- Rendering MVP: astraweave-render produces a lit scene (PBR, HDR, tonemapping, sky, shadows) via wgpu 0.20 (or latest compatible).
- Scene I/O: glTF 2.0 import (meshes, materials, textures); simple scene graph; GPU resource manager; frustum culling; basic instancing.
- Animation v0: Skeletal playback (simple blend, GPU skinning).
- Editor Shell: `tools/aw_editor` rebuilt using eframe/egui with dockable panels (Scene, Hierarchy, Inspector, Console, Profiler); live reload of material/shader params.
- AI Loop: Keep current `WorldSnapshot` / `PlanIntent`; unify `Orchestrator` API; CI example that runs end-to-end without panics.
- Fix/green core: make check, minimal lint set; working examples curated; broken demos quarantined.

Key Crates/Changes
- New: `astraweave-scene`, `astraweave-materials`, `astraweave-anim`, `astraweave-asset` (importers), `astraweave-shaders` (WGSL).
- Update: `astraweave-render` (PBR + shadows + post), `tools/aw_editor` (panelized), `examples/visual_3d` (MVP scene viewer).

Exit Criteria
- Run: `cargo run -p visual_3d` shows a PBR scene with a skinned character at 60 FPS on mid-tier GPU.
- Editor: can load a scene, tweak a material, toggle lights, and save.

Phase 1 (3–6 months): AI-Native Gameplay and Authoring Tools
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

Exit Criteria
- Example “hello_companion_3d”: companion perceives, plans, and executes in a small 3D level; voice line triggers; player orders via UI.
- Editor: author a simple quest + behavior graph and run it live.

Phase 2 (6–12 months): Modern Renderer and Asset Pipeline (UE5-Class Targets)
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

Phase 3 (12–18 months): Multiplayer, Determinism, and Scale
Deliverables
- Netcode: Server-authoritative with client prediction & reconciliation; snapshot delta encoding; interest management; region matchmaking; replay capture.
- Deterministic Sim: audited fixed-tick path; lockstep option; authoritative tool validation across clients.
- Save/Live Ops: robust save/versioning (`aw-save`), migration; asset patching and hotfix pipeline; crash reporting hooks.
- Security: anti-cheat hooks (server-side validation + telemetry); sandboxing for scripting.

Exit Criteria
- 4-player co-op demo in a medium-sized level with AI companions; 60 Hz sim; jitter-buffered voice/text; no desync under packet loss; stable replays.

Phase 4 (18–24 months): Production Polish and Extensibility
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
