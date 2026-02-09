---
layout: default
title: Crate Index
---

# Crate Index

AstraWeave consists of 49 crates organized into tiers.

## Tier 1: Core Engine

| Crate | Description | Tests |
|-------|-------------|-------|
| **astraweave-ecs** | Archetype-based ECS with deterministic stages | 386+ |
| **astraweave-core** | Shared types: WorldSnapshot, PlanIntent, ToolRegistry | 465+ |
| **astraweave-math** | SIMD-accelerated vector/matrix/quaternion ops | 109+ |
| **astraweave-physics** | Rapier3D wrapper: rigid bodies, spatial hash, ragdoll, cloth | 598+ |
| **astraweave-render** | wgpu 25 renderer: PBR, shadows, post-FX, animation | — |
| **astraweave-scene** | Scene graph, transforms, world partitioning | — |

## Tier 2: AI & Gameplay

| Crate | Description | Tests |
|-------|-------------|-------|
| **astraweave-ai** | AI orchestration: GOAP, BT, LLM, hybrid arbiter | 100+ |
| **astraweave-behavior** | Behavior trees and GOAP planner | — |
| **astraweave-gameplay** | Combat, crafting, quests, dialogue, items, biomes | — |
| **astraweave-nav** | Navmesh baking and A* pathfinding | — |
| **astraweave-sdk** | C ABI for non-Rust embedding (cbindgen) | 17+ |

## Tier 3: UI & Input

| Crate | Description | Tests |
|-------|-------------|-------|
| **astraweave-ui** | egui menus, HUD, accessibility, gamepad | 51+ |
| **astraweave-input** | Key/mouse/gamepad bindings and action mapping | — |
| **astraweave-audio** | rodio spatial audio, dialogue, TTS | — |
| **astraweave-cinematics** | Timeline sequencer, camera/audio/FX tracks | — |

## Tier 4: LLM & Intelligence

| Crate | Description | Tests |
|-------|-------------|-------|
| **astraweave-llm** | LLM client abstraction (Ollama, streaming) | 107+ |
| **astraweave-llm-eval** | LLM evaluation harness | — |
| **astraweave-prompts** | Prompt templates and engineering | — |
| **astraweave-embeddings** | Text embeddings client | — |
| **astraweave-rag** | Retrieval-augmented generation | — |
| **astraweave-memory** | Long-term memory for NPCs | — |
| **astraweave-context** | Context window management | — |
| **astraweave-persona** | NPC personality and dialogue personas | — |
| **astraweave-optimization** | LLM performance optimization | — |
| **astraweave-coordination** | Multi-agent coordination | — |

## Tier 5: World & Content

| Crate | Description | Tests |
|-------|-------------|-------|
| **astraweave-terrain** | Voxel/polygon hybrid, marching cubes | — |
| **astraweave-pcg** | Procedural content generation | — |
| **astraweave-weaving** | Fate-weaving system (Veilweaver mechanic) | — |
| **astraweave-fluids** | SPH fluid simulation | 2,404+ |
| **astraweave-materials** | Material definitions | — |

## Tier 6: Networking & Persistence

| Crate | Description | Tests |
|-------|-------------|-------|
| **astraweave-net** | Networking core | — |
| **astraweave-net-ecs** | ECS networking integration | — |
| **aw-net-client** | Network client | — |
| **aw-net-server** | Network server | — |
| **aw-net-proto** | Network protocol definitions | — |
| **aw-save** | Versioned, checksummed save files | — |
| **astraweave-persistence-ecs** | ECS persistence integration | — |
| **astraweave-persistence-player** | Player profile persistence | — |

## Tier 7: Tools & Infrastructure

| Crate | Description |
|-------|-------------|
| **aw_editor** | Level/encounter editor (GUI) |
| **aw_asset_cli** | Asset pipeline tooling |
| **aw_build** | Build automation |
| **aw_debug** | Debug utilities |
| **aw_headless** | Headless engine runner |
| **astraweave-profiling** | Tracy profiling integration |
| **astraweave-observability** | Metrics and logging |
| **astraweave-security** | Sandboxing and validation |
| **astraweave-scripting** | Rhai scripting integration |
| **astraweave-stress-test** | Stress testing framework |
| **astract** | React-style declarative UI widgets |

[← Back to Home](index.html)
