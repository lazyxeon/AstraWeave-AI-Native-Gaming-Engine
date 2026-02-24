# AstraWeave: Master Strategic Roadmap

**Version**: 1.46  
**Last Updated**: February 10, 2026  
**Status**: Authoritative Source  
**Validation**: ✅ PASS — [Full Report](../current/ENGINE_VALIDATION_2026_01_13.md)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Project Status](#project-status)
3. [Architecture Overview](#architecture-overview)
4. [Strategic Roadmap](#strategic-roadmap)
5. [Success Metrics](#success-metrics)
6. [Completed Phases](#completed-phases)
7. [Revision History](#revision-history)

---

## Executive Summary

### Mission

AstraWeave is the **world's first AI-native game engine** where AI agents are first-class citizens, not bolted-on features. The entire codebase is developed iteratively by AI Agents (GitHub Copilot/Antigravity) with zero human-written code.

### Current State (February 3, 2026)

| Metric | Value |
|--------|-------|
| **Validation Status** | ✅ **A+ Grade (95/100)** |
| **Tests Passing** | **5,372/5,383 (99.8%)** |
| **Compilation** | **17/17 core crates** (0 errors, 0 warnings) |
| **Miri Validation** | ✅ **977 tests, 0 UB** |
| Workspace Packages | 121 (47 core + 74 examples/tools) |
| Total Tests | 7,600+ |
| Test Coverage (Overall) | ~78% |
| Frame Time (p95) | ~2.70ms |
| Agent Capacity | 12,700+ @ 60 FPS |
| **Determinism** | **100% validated** |
| **Memory Safety** | ✅ **Miri-verified** |

**Recent Achievement** (Feb 3, 2026): Miri memory safety validation complete — 4 crates, 977 tests, zero undefined behavior — [Details](../current/MIRI_VALIDATION_REPORT.md)

### What AstraWeave Is

- ✅ A **working prototype** with solid foundations
- ✅ The **world's first AI-native game engine** with unique architecture
- ✅ A **comprehensive experiment** proving AI can build complex systems
- ✅ **Memory-safe**: All unsafe code Miri-validated
- ✅ Approximately **3-12 months away** from production readiness

### What AstraWeave Is Not

- ❌ A fully production-ready game engine
- ❌ A feature-complete Unity/Unreal alternative
- ❌ Ready to ship commercial games today

---

## Project Status

### Core Capabilities ✅

| System | Status | Key Features |
|--------|--------|--------------|
| **ECS** | Production | Archetype-based, BlobVec storage, deterministic, 7-stage pipeline, 386 tests, **Miri-validated** |
| **AI** | Production | 6 planning modes, GOAP, Behavior Trees, LLM integration |
| **Rendering** | Production | wgpu 25, PBR/IBL, CSM shadows, post-processing, **Headless Support** |
| **Physics** | Production | Rapier3D, character controller, spatial hashing |
| **Audio** | Production | Spatial audio, 4-bus mixer, rodio backend |
| **Navigation** | Production | Navmesh, A*, portal graphs (66/66 tests) |
| **SDK** | Production | C ABI FFI, **Miri-validated** |

### Recent Additions (February 2026)

| Feature | Location | Tests |
|---------|----------|-------|
| **✅ Miri Memory Safety Validation** | ecs, math, core, sdk | **977** |
| ECS BlobVec Optimization | `astraweave-ecs/src/archetype.rs` | 386 |
| Headless Renderer & GPU Verification | `astraweave-render/src/renderer.rs` | 369 |
| AI-Orchestrated Dynamic Terrain | Multiple crates | 320+ |
| TLS/SSL Security | `astraweave-net/src/tls.rs` | 3 |
| LLM Schema Enforcement | `astraweave-llm/src/schema.rs` | 7 |
| Path Traversal Protection | `astraweave-security/src/path.rs` | 15 |
| Serde Size Limits | `astraweave-security/src/deserialization.rs` | 6 |

### Test Coverage by Priority Tier

| Tier | Description | Average Coverage |
|------|-------------|------------------|
| P0 | Core Engine (Math, Physics, Behavior, Nav, Audio) | 94.71% |
| P1-A | Infrastructure (AI, ECS, Core) | 96.43% |
| P1-B | Game Systems (Gameplay, Terrain, Render, Scene) | 71.06% |
| P1-C | Support Features (PCG, Weaving, Input, Cinematics) | 86.32% |

---

## Architecture Overview

### Intelligence Loop

```
Perception → Reasoning → Planning → Action → Validation → Simulation
```

The core game loop is an **Intelligence Loop** where:

1. **Perception**: Agents see the world through WorldSnapshot
2. **Reasoning**: LLMs and Utility systems analyze state
3. **Planning**: GOAP and Behavior Trees formulate plans
4. **Action**: Plans execute via deterministic ECS commands
5. **Validation**: Tool sandbox validates all actions
6. **Simulation**: ECS updates world state

### ECS System Stages

| Stage | Order | Purpose |
|-------|-------|---------|
| PRE_SIMULATION | 1 | Setup, initialization |
| PERCEPTION | 2 | Build WorldSnapshots, update sensors |
| SIMULATION | 3 | Game logic, cooldowns, state |
| AI_PLANNING | 4 | Generate PlanIntents |
| PHYSICS | 5 | Apply forces, collisions |
| POST_SIMULATION | 6 | Cleanup, constraints |
| PRESENTATION | 7 | Rendering, audio, UI |

### AI Arbiter System

The **Arbiter** (`astraweave-ai/src/ai_arbiter.rs`) orchestrates the handshake between LLM strategic planning and GOAP tactical execution:

```
┌─────────────────────────────────────────────────────────────┐
│                      AI Arbiter                              │
├─────────────────────────────────────────────────────────────┤
│  WorldSnapshot → [LLM Strategic] ↔ [GOAP Tactical] → Action │
│                        ↓                  ↓                  │
│              Long-term Goals      Short-term Plans           │
│              (3-10 seconds)       (per-frame)                │
└─────────────────────────────────────────────────────────────┘
```

| Mode | Description | Latency |
|------|-------------|---------|
| **Classical** | Rule-based fallback | ~0.20ms |
| **BehaviorTree** | Hierarchical state machines | ~0.17ms |
| **Utility** | Weighted action scoring | ~0.46ms |
| **LLM** | Hermes 2 Pro strategic planning | ~3.5s |
| **Hybrid** | LLM strategy + GOAP execution | ~2.2s |
| **Ensemble** | Multi-mode weighted voting | ~2.4s |

Key features:
- **Async LLM polling**: Non-blocking (<10 µs overhead)
- **Automatic fallback**: LLM → GOAP → Utility → Classical
- **Metrics tracking**: Transition success rates, execution times

**Core Crates** (47):
- `astraweave-ecs`, `astraweave-ai`, `astraweave-core`
- `astraweave-render`, `astraweave-physics`, `astraweave-audio`
- `astraweave-nav`, `astraweave-behavior`, `astraweave-llm`
- `astraweave-security`, `astraweave-net`, `astraweave-scripting`
- ...and 35 more

**Examples/Tools** (74):
- `hello_companion`, `unified_showcase`, `aw_editor`
- Integration demos, test harnesses, CLI tools

---

## Strategic Roadmap

### Phase 8: Game Engine Readiness (Current)

**Objective**: Transform from "production-ready infrastructure" to "ship a game on it"

| Priority | Component | Timeline | Status |
|----------|-----------|----------|--------|
| 1 | In-Game UI Framework | 5 weeks | ✅ Complete (153 tests) |
| 2 | Complete Rendering Pipeline | 4-5 weeks | ✅ Complete |
| 3 | Save/Load System | 2-3 weeks | ✅ Complete (52 tests) |
| 4 | Production Audio | 2-3 weeks | ✅ Complete (147 tests) |

**Recent Additions (December 2025)**:
- `astraweave-ui/src/gamepad.rs` - PlayStation controller support (7 tests)
- `astraweave-ui/src/accessibility.rs` - Colorblind modes + high-contrast (12 tests)

### Phase 9: Distribution & Polish ✅

| Component | Timeline | Status |
|-----------|----------|--------|
| Scripting Runtime | 4 weeks | ✅ Complete (12 tests) |
| Asset Pipeline | 2 weeks | ✅ Complete (48 tests) |
| Build & Packaging | 2 weeks | ✅ Complete (aw_build + aw_release) |
| Profiling Tools | 1 week | ✅ Tracy integrated |

### Phase 10: Advanced Features ✅

| Component | Timeline | Status |
|-----------|----------|--------|
| Multiplayer Networking | 4-6 weeks | ✅ Complete (11 tests) |
| Global Illumination | 3-4 weeks | ✅ Complete (355 render tests, VXGI) |
| Steam Platform | 2 weeks | ✅ Complete (8 tests) |
| **AI-Orchestrated Terrain** | 2-3 weeks | ✅ Complete (320+ tests) |

---

## Success Metrics

### Current vs Targets

| Metric | Current | 12-Month Target | Status |
|--------|---------|-----------------|--------|
| Test Coverage (Overall) | ~76% | 85%+ | 🟢 On Track |
| Test Coverage (P0) | 94.71% | 90%+ | ✅ Exceeded |
| Total Tests | 1,349 | 1,500+ | 🟢 On Track |
| Integration Tests | 215 | 100+ | ✅ Exceeded |
| `.unwrap()` in Core | 0 | 0 | ✅ Complete |
| Frame Time (p95) | ~2.70ms | <10ms | ✅ Exceeded |
| Agent Capacity | 12,700+ | 10,000+ | ✅ Exceeded |
| LLM Quality Score | 75-85% | 95%+ | 🟠 In Progress |

### Performance Baselines

| Subsystem | Metric | Value |
|-----------|--------|-------|
| ECS | World creation | 25.8 ns |
| ECS | Entity spawn | 420 ns |
| AI Core Loop | Tick | 184 ns - 2.10 µs |
| GOAP | Cache hit | 1.01 µs |
| Behavior Trees | Tick | 57-253 ns |
| Physics | Full tick | 6.52 µs |
| Frame Time | Overall | 2.70 ms @ 1,000 entities |

---

## Completed Phases

### Phase 1-8: Rendering System ✅

**Completed**: November 2025 (~15 hours, 36/36 tasks)

Key achievements:
- PBR with Cook-Torrance BRDF
- Image-Based Lighting with environment maps
- Cascaded Shadow Maps (4 cascades, PCF)
- Post-processing (Bloom, SSAO, SSR, Motion Blur, DoF)
- GPU particle system with compute shaders
- Back-face culling (~40% performance improvement)

### Phase 2: AI Hardening ✅

**Completed**: November 2025

Key achievements:
- GOAP infinite loop prevention
- LLM client stabilization (history tracking)
- Streaming API (44.3× time-to-first-chunk improvement)
- Batch executor (6-8× throughput improvement)

### Phase 8.7: LLM Testing ✅

**Completed**: November 2025

Key achievements:
- 107+ tests added (100% pass rate)
- RAG integration with lifecycle management
- Persona system stabilization

### Phase 9.2: Scripting Foundation ✅

**Completed**: November 2025 (Sprint 2)

Key achievements:
- `astraweave-scripting` crate with Rhai v1.23
- `CScript` component for ECS integration
- Event system with `ScriptEvent` enum

### Phase 10.4: AI-Orchestrated Dynamic Terrain ✅

**Completed**: December 8, 2025

Key achievements:
- TerrainSolver for LLM-to-world coordinate translation
- TerrainModifier with time-sliced voxel operations
- NavMesh invalidation and partial rebaking
- Terrain persistence with compression
- LLM prompt templates for terrain generation
- Terrain-driven quest generation (20 feature types)
- API expansion (`apply_damage`, `set_position`)

---

## Revision History

> **IEEE/ACM-Compliant Format** (restructured 2026-01-18)

### Executive Summary

| Metric | Value |
|--------|-------|
| **Total Versions** | 18 |
| **Timeline** | Nov 22, 2025 → Jan 2026 (~2 months) |
| **Primary Authors** | AI Team |
| **Average Frequency** | ~3 days/version |

### Version Type Legend

| Symbol | Type | Description |
|--------|------|-------------|
| 🔷 | MAJOR | Phase completion, strategic milestones |
| 🔹 | MINOR | Feature additions, sprint completion |
| 🔸 | PATCH | Bug fixes, small updates |
| 🔍 | AUDIT | Validation, verification sessions |
| ⚠️ | FIX | Critical bug fixes, regressions |
| 📋 | PLAN | Documentation, planning updates |

### Impact Grade Legend

| Grade | Symbol | Criteria |
|-------|--------|----------|
| 🔴 | CRITICAL | Phase completion, >100 tests, strategic change |
| 🟡 | SIGNIFICANT | Sprint complete, >20 tests, milestone |
| 🟢 | INCREMENTAL | <20 tests, documentation, minor updates |
| ⚪ | ADMINISTRATIVE | Planning, documentation-only |

---

### Primary Revision Table

| Ver | Date | Type | Impact | Summary (≤80 chars) |
|-----|------|------|--------|---------------------|
| **1.46** | Feb 10 | 🔷 | 🔴 | Veilweaver vertical slice: 5 phases, 20 modules, 320 tests, zero unsafe |
| **1.44** | Jan 26 | 📋 | 🟢 | Revised Validation Plan v2.0: Sanitizers, CI workflow, validate.ps1 |
| **1.43** | Jan 26 | ⚠️ | 🔴 | ECS REGRESSION FIXED: BlobVec 52-68% faster, 10K+ entities restored |
| **1.42** | Dec 21 | 🔍 | 🟢 | Renderer production validation: Headless verification complete |
| **1.41** | Dec 20 | 🔹 | 🟡 | Renderer Headless: +365 tests, 42% coverage, CI-safe testing |
| **1.40** | Dec 8 | 🔷 | 🔴 | AI Terrain: 8 phases, TerrainSolver, NavMesh, 320+ tests |
| **1.39** | Dec 6 | 🔹 | 🟡 | Steam Platform: astraweave-steam, Steamworks SDK, achievements |
| **1.38** | Dec 6 | 🔍 | 🟡 | Phase 10 GI: VXGI complete, VxgiRenderer, 355 render tests |
| **1.37** | Dec 6 | 🔷 | 🟡 | Phase 10 Networking: astraweave-net/net-ecs, TLS/WebSocket/HMAC |
| **1.36** | Dec 6 | 🔷 | 🟡 | Phase 9 complete: Build/Packaging, release.yml Win/Mac/Linux |
| **1.35** | Dec 6 | 🔍 | 🟢 | Phase 9 audit: Scripting 12 tests, Asset Pipeline 48 tests |
| **1.34** | Dec 6 | 🔷 | 🟡 | Phase 8.4 Audio complete: #[serial] fix, 147 tests passing |
| **1.33** | Dec 5 | 🔷 | 🟡 | Phase 8.3 Save/Load: Debug thresholds fixed, 52 tests |
| **1.32** | Dec 5 | 🔷 | 🟡 | Phase 8.1 UI Framework: gamepad/accessibility, 153 tests |
| **1.31** | Dec 5 | 📋 | 🟢 | Docs audit: 82→121 crates, TLS/Security, docs reorg |
| **1.30** | Nov 23 | 🔹 | 🟢 | Phase 9.2 Sprint 2: Event system, API expansion |
| **1.29** | Nov 23 | 🔹 | 🟢 | Phase 9.2 started: astraweave-scripting crate |
| **1.28** | Nov 23 | 🔹 | 🟢 | Phase 8.7 Sprint 4: Full stack integration |
| **1.27** | Nov 22 | 🔹 | 🟢 | Phase 8.7 RAG integration complete |

---

### Statistical Summary

**By Type:**
| Type | Count | % |
|------|-------|---|
| 🔷 MAJOR | 6 | 33.3% |
| 🔹 MINOR | 6 | 33.3% |
| 📋 PLAN | 2 | 11.1% |
| 🔍 AUDIT | 3 | 16.7% |
| ⚠️ FIX | 1 | 5.6% |

**By Impact:**
| Impact | Count | % |
|--------|-------|---|
| 🔴 CRITICAL | 2 | 11.1% |
| 🟡 SIGNIFICANT | 8 | 44.4% |
| 🟢 INCREMENTAL | 8 | 44.4% |

---

### Key Milestones Timeline

```
Nov 22 ─── v1.27: Phase 8.7 RAG Integration
    │
Nov 23 ─── v1.28-1.30: Sprint 4, Scripting Foundation
    │
Dec 5 ─┬─ v1.32: Phase 8.1 UI Framework (153 tests)
       ├─ v1.33: Phase 8.3 Save/Load (52 tests)
       └─ v1.34: Phase 8.4 Audio (147 tests)
    │
Dec 6 ─┬─ v1.36: Phase 9 Build/Packaging
       ├─ v1.37: Phase 10 Networking (TLS/WS)
       └─ v1.39: Steam Platform Integration
    │
Dec 8 ─── v1.40: AI-Orchestrated Dynamic Terrain (320+ tests)
    │
Dec 20-21 ─ v1.41-1.42: Renderer Headless (+365 tests)
    │
Jan 2026 ─ v1.43-1.44: ECS Regression Fix, Validation Plan v2.0
    │
Feb 2026 ─ v1.46: Veilweaver Vertical Slice (320 tests, 20 modules)
```

---

### Detailed Changelog (Critical Versions)

<details>
<summary><b>v1.46 (Feb 2026) - VEILWEAVER VERTICAL SLICE</b></summary>

**Impact**: 🔴 CRITICAL  
**Type**: 🔷 MAJOR  
**Author**: AI Team

**Changes**:
- Complete 5-phase Veilweaver vertical slice (`veilweaver_slice_runtime` crate)
- `#![forbid(unsafe_code)]` — zero unsafe, headless-safe, no wgpu/egui deps
- **Phase 1**: Core game loop, zone registry (5 zones), ECS integration
- **Phase 2**: Dialogue system, cinematics player, storm choice branching
- **Phase 3**: Boss HUD (3-phase health bar), companion affinity meter, telemetry
- **Phase 4**: VFX descriptors (6 categories), audio specs (10+ cue types), palette
- **Phase 5**: Determinism validation (3-run hash consistency), perf budget tracker (p50/p95/p99), save/checkpoint, 30-min pacing simulation, edge case hardening
- **Edge case hardening**: NaN guards on all animation ticks, `.expect()` → guard clause, VecDeque for O(1) eviction, Default impls, `.first()` vs direct-index safety
- **20 source modules**, 6 integration test suites, 320 tests (265 unit + 55 integration)
- Clippy clean, fmt clean, zero warnings in crate
- New crate: `veilweaver_slice_runtime v0.1.0`

</details>

<details>
<summary><b>v1.43 (Jan 2026) - ECS REGRESSION FIX</b></summary>

**Impact**: 🔴 CRITICAL  
**Type**: ⚠️ FIX  
**Author**: AI Team

**Changes**:
- Fixed critical BlobVec performance regression from NASA-grade audit v5.52
- Root cause: HashMap allocation in Archetype::new() even in legacy mode
- Fix: Lazy initialization with Option<HashMap>
- Results: entity_spawn 52% faster, entity_despawn 68% faster
- 10K+ entities @ 60 FPS restored, all 220 ECS tests passing
- Grade: B- → A+

</details>

<details>
<summary><b>v1.40 (Dec 8, 2025) - AI-ORCHESTRATED DYNAMIC TERRAIN</b></summary>

**Impact**: 🔴 CRITICAL  
**Type**: 🔷 MAJOR  
**Author**: AI Team

**Changes**:
- Complete 8-phase implementation:
  - TerrainSolver for LLM-to-world coordinate translation
  - TerrainModifier with time-sliced voxel operations
  - NavMesh invalidation and partial rebaking
  - Terrain persistence with compression
  - LLM prompt templates for terrain generation
  - Terrain-driven quest generation (20 feature types)
  - API expansion (`apply_damage`, `set_position`)
- 320+ tests added

</details>

<details>
<summary>v1.45 (2026-02-03) — Miri Memory Safety Validation</summary>

**Impact**: 🟢 ENHANCEMENT  
**Type**: 🔷 MAJOR  
**Author**: AI Team

**Changes**:
- Comprehensive Miri validation across 4 crates with unsafe code
- 977 tests validated with zero undefined behavior detected
- Crates validated: astraweave-ecs (386 tests), astraweave-math (109 tests), astraweave-core (465 tests), astraweave-sdk (17 tests)
- Memory safety certification for: BlobVec, SparseSet, EntityAllocator, SIMD intrinsics, C ABI FFI
- Documentation: [MIRI_VALIDATION_REPORT.md](../current/MIRI_VALIDATION_REPORT.md), [ECS_MIRI_VALIDATION_REPORT.md](../current/ECS_MIRI_VALIDATION_REPORT.md)

</details>

---

### Issues Resolved During Restructuring

| ID | Issue | Resolution |
|----|-------|------------|
| VERBOSE-001 | Entries 50-200 words each | Summarized to ≤80 chars |
| FORMAT-001 | Missing type/impact classification | Added 6 types + 4 impact grades |
| STATS-001 | No statistical summary | Added type/impact distribution |
| TIMELINE-001 | No visual timeline | Added ASCII milestone visualization |

---

**Next Review Date**: 2026-03-03 (monthly cadence)  
**Revision History Format Version**: 2.0.0 (IEEE/ACM-compliant)  
**Last Restructured**: 2026-01-18

---

*See `.github/copilot-instructions.md` for maintenance protocol and enforcement rules.*