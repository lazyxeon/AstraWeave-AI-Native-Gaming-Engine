# AstraWeave: Master Strategic Roadmap

**Version**: 1.33  
**Last Updated**: December 5, 2025  
**Status**: Authoritative Source

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

### Current State (December 2025)

| Metric | Value |
|--------|-------|
| Workspace Packages | 121 (47 core + 74 examples/tools) |
| Total Tests | 1,349+ |
| Test Coverage (Overall) | ~76% |
| Frame Time (p95) | ~2.70ms |
| Agent Capacity | 12,700+ @ 60 FPS |

### What AstraWeave Is

- ✅ A **working prototype** with solid foundations
- ✅ The **world's first AI-native game engine** with unique architecture
- ✅ A **comprehensive experiment** proving AI can build complex systems
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
| **ECS** | Production | Archetype-based, deterministic, 7-stage pipeline |
| **AI** | Production | 6 planning modes, GOAP, Behavior Trees, LLM integration |
| **Rendering** | Production | wgpu 25, PBR/IBL, CSM shadows, post-processing |
| **Physics** | Production | Rapier3D, character controller, spatial hashing |
| **Audio** | Production | Spatial audio, 4-bus mixer, rodio backend |
| **Navigation** | Production | Navmesh, A*, portal graphs (66/66 tests) |

### Recent Additions (December 2025)

| Feature | Location | Tests |
|---------|----------|-------|
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
| 4 | Production Audio | 2-3 weeks | In Progress |

**Recent Additions (December 2025)**:
- `astraweave-ui/src/gamepad.rs` - PlayStation controller support (7 tests)
- `astraweave-ui/src/accessibility.rs` - Colorblind modes + high-contrast (12 tests)

### Phase 9: Distribution & Polish (Planned)

| Component | Timeline | Focus |
|-----------|----------|-------|
| Build & Packaging | 2 weeks | Desktop builds, installer |
| Asset Pipeline | 2 weeks | Texture baking, optimization |
| Profiling Tools | 1 week | Tracy integration, metrics |
| Scripting Runtime | 4 weeks | Rhai integration (Sprint 2 complete) |

### Phase 10: Advanced Features (Future)

| Component | Timeline | Focus |
|-----------|----------|-------|
| Multiplayer Networking | 4-6 weeks | Authoritative server, prediction |
| Global Illumination | 3-4 weeks | VXGI improvements |
| Console Support | 6-8 weeks | PlayStation, Xbox, Switch |

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
- API expansion (`apply_damage`, `set_position`)

---

## Revision History

| Version | Date | Summary |
|---------|------|---------|
| 1.33 | Dec 5, 2025 | Phase 8.3 Save/Load complete: Fixed debug test thresholds, 52 tests passing (aw-save + persistence-ecs) |
| 1.32 | Dec 5, 2025 | Phase 8.1 UI Framework complete: gamepad.rs (7 tests), accessibility.rs (12 tests), 153 total UI tests |
| 1.31 | Dec 5, 2025 | Documentation audit: Corrected crate count (82→121), added TLS/Security, LLM Schema, docs reorganization |
| 1.30 | Nov 23, 2025 | Phase 9.2 Sprint 2: Event system, API expansion |
| 1.29 | Nov 23, 2025 | Phase 9.2 started: astraweave-scripting crate |
| 1.28 | Nov 23, 2025 | Phase 8.7 Sprint 4: Full stack integration |
| 1.27 | Nov 22, 2025 | Phase 8.7 RAG integration complete |

---

*See `.github/copilot-instructions.md` for maintenance protocol and enforcement rules.*