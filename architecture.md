---
layout: default
title: Architecture
---

# Architecture Overview

AstraWeave is an AI-native game engine organized as a Rust workspace of 49 crates with clear dependency boundaries.

## Dependency Graph

```
                    astraweave-sdk (C ABI)
                         │
                    astraweave-core (shared types)
                    ╱    │    ╲        ╲
          astraweave-ai  │  astraweave-gameplay
              │     astraweave-ecs     │
              │          │             │
    astraweave-behavior  │   astraweave-physics
                         │         │
                   astraweave-nav   │
                         │         │
                   astraweave-scene │
                    ╱         ╲    │
          astraweave-render  astraweave-audio
                    │
              astraweave-ui
                    │
              astraweave-input
```

## AI-Native Loop

Every NPC in AstraWeave runs through a validated 4-stage loop every frame:

### 1. Perception (SystemStage::PERCEPTION)

Build a `WorldSnapshot` for each AI agent containing filtered world state: player position, enemy positions, points of interest, obstacles, and companion status.

### 2. Reasoning (SystemStage::AI_PLANNING)

The `Orchestrator` trait dispatches to the configured AI backend:

| Mode | Backend | Latency | Use Case |
|------|---------|---------|----------|
| Classical | Behavior Trees | 57–253 ns | Patrol, combat AI |
| GOAP | Goal-Oriented Planner | 1–47 µs | Strategic planning |
| LLM | Hermes 2 Pro (Ollama) | 2–8 s | Creative, emergent behavior |
| Hybrid | GOAP + LLM Arbiter | 314 ns–8 s | Best of both (GOAP for tactics, LLM for strategy) |

### 3. Planning

The orchestrator produces a `PlanIntent` containing a sequence of `ActionStep` values. Each step references a registered tool (`move_to`, `attack`, `take_cover`, etc.).

### 4. Action (Tool Sandbox)

The `ToolRegistry` validates every `ActionStep` against permitted actions, constraints, and cooldowns. Invalid actions are rejected — **no AI can "cheat"**.

## ECS System Stages

Deterministic execution order, 60 Hz fixed tick:

1. **PRE_SIMULATION** — Setup, initialization
2. **PERCEPTION** — Build WorldSnapshots, update sensors
3. **SIMULATION** — Game logic, cooldowns, state updates
4. **AI_PLANNING** — Generate PlanIntents
5. **PHYSICS** — Forces, collision resolution
6. **POST_SIMULATION** — Cleanup, constraints
7. **PRESENTATION** — Rendering, audio, UI

## Memory Safety

All unsafe code has been Miri-validated:

| Crate | Tests | Unsafe Patterns |
|-------|-------|-----------------|
| astraweave-ecs | 386 | BlobVec, SparseSet, EntityAllocator, SystemParam |
| astraweave-math | 109 | SIMD intrinsics (SSE2), scalar fallback |
| astraweave-core | 465 | Entity::from_raw, capture/replay |
| astraweave-sdk | 17 | C ABI FFI, raw pointer handling |
| **Total** | **977** | **0 undefined behavior** |

## Determinism

AstraWeave achieves industry-leading determinism:

- Bit-identical replay across runs
- Position tolerance: < 0.0001
- 100-frame replay validated (1.67 seconds @ 60 FPS)
- 5-run consistency validated (exceeds 3-run target)
- 100 seeds tested (comprehensive RNG validation)

[← Back to Home](index.html)
