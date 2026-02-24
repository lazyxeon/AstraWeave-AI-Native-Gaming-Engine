# Veilweaver Vertical Slice Runtime

**Version**: 0.1.0 | **Rust**: 2021 Edition | **License**: MIT  
**Safety**: `#![forbid(unsafe_code)]` | **Status**: Phase 5 Complete ✅

---

## Overview

The Veilweaver vertical slice runtime is a **headless-safe** game runtime for AstraWeave's showcase vertical slice. It implements a complete 30-minute gameplay experience — zone traversal, dialogue, cinematics, storm choice branching, boss encounter, companion affinity, VFX/audio dispatch, and telemetry — **without any GPU, windowing, or audio dependencies**.

The presentation layer (egui, wgpu, rodio) reads the pure data models produced by this crate each frame.

## Architecture

```
┌───── Runtime (this crate) ─────────────────────────────────┐
│                                                             │
│  GameLoop ──► ZoneRegistry ──► DialogueRunner               │
│     │              │              │                          │
│     ├── CinematicPlayer ──────── StormChoiceState            │
│     │                                                       │
│  HUD Layer:  BossHealthBar · CompanionAffinityMeter         │
│              ThreadHud (stability + echoes) · DecisionUI     │
│              RecapPanel · TelemetryCollector                 │
│                                                             │
│  VFX/Audio:  VfxSpecs · AudioSpecs · Palette · VfxDispatch  │
│                                                             │
│  Validation: Determinism · PerfBudget · Checkpoint          │
│                                                             │
│  AI (opt):   CompanionAI · BossEncounter                    │
└─────────────────────────────────────────────────────────────┘
```

## Modules (23)

| Module | Phase | Purpose |
|--------|-------|---------|
| `game_loop` | 2 | Core tick loop, zone transitions, event dispatch |
| `zone_transitions` | 2 | Zone registry, 5 zones (Z0–Z4), trigger specs |
| `cinematic_player` | 2 | Cutscene sequencing (intro, storm, boss, finale) |
| `storm_choice` | 2 | Stabilize/Redirect branching with stat modifiers |
| `boss_hud` | 3 | 3-phase health bar with animated drain + flash |
| `companion_hud` | 3 | Affinity meter (5 ranks: Wary → Synced) |
| `decision_ui` | 3 | Storm decision prompt with timer + descriptions |
| `hud_state` | 3 | Thread stability bar + echo counter |
| `recap_panel` | 3 | End-of-run performance recap with S/A/B/C/D ratings |
| `telemetry` | 3 | Play session metrics (combat, exploration, timing) |
| `vfx_specs` | 4 | VFX descriptors (6 categories, 10+ spec types) |
| `audio_specs` | 4 | Audio cue descriptors (10+ types, spatial + mix) |
| `palette` | 4 | Zone-based color palettes + ambient lighting |
| `vfx_dispatch` | 4 | Per-frame VFX/audio scene builder from game state |
| `determinism` | 5 | Hash-based N-run consistency validation |
| `perf_budget` | 5 | Frame budget tracker (p50/p95/p99 percentiles) |
| `checkpoint` | 5 | Save/load with JSON serialization + proxy structs |
| `boss_encounter` | AI | Feature-gated boss AI director integration |
| `companion_ai` | AI | Feature-gated companion AI behaviors |
| `player_state` | 2 | Player HP, echoes, zone, anchors, respawn logic |
| `combat` | 2 | CombatEncounter with wave-based enemy spawning |
| `walkthrough` | 2 | SliceOrchestrator — top-level subsystem composer |
| `lib` | — | Crate root, ECS integration, trigger geometry |

## Tests

**402 tests** — 308 unit + 90 integration + 4 doc-tests, 0 failures, 0 unsafe.

| Suite | Tests | Focus |
|-------|-------|-------|
| Unit tests (in-module) | 308 | Per-module correctness, NaN guards, edge cases |
| Doc-tests | 4 | API usage examples (checkpoint, determinism, perf, vfx) |
| `e2e_content_validation` | 15 | Asset presence, RON parsing, navmesh stubs |
| `e2e_game_loop_smoke` | 3 | Zone traversal, event dispatch, storm resolve |
| `e2e_hud_pipeline` | 6 | HUD state updates, stability animation, echo feedback |
| `e2e_pacing_playthrough` | 6 | 30-min pacing simulation, telemetry milestones |
| `e2e_presentation_pipeline` | 8 | Full VFX/audio scene composition |
| `e2e_validation_smoke` | 11 | Determinism, perf budget, checkpoint round-trip |
| `e2e_vfx_audio_pipeline` | 8 | VFX dispatch, audio cue generation, scene builder |
| `e2e_walkthrough_integration` | 12 | Full Z0→Z4→Debrief orchestrator flow |
| `mutation_resistant` | 21 | Boundary conditions, comparison operators, path inversions |

## Quick Start

```powershell
# Check compilation
cargo check -p veilweaver_slice_runtime

# Run all tests (headless, no GPU required)
cargo test -p veilweaver_slice_runtime --features full-ai

# Run without optional AI features
cargo test -p veilweaver_slice_runtime

# Clippy lint
cargo clippy -p veilweaver_slice_runtime --lib -- -D warnings
```

## Features

| Feature | Dependencies | Purpose |
|---------|-------------|---------|
| `ai-companion` | `astraweave-ai` | Companion AI behaviors |
| `boss-director` | `astraweave-director` | Boss encounter AI director |
| `full-ai` | Both above | Enable all AI features |

## Edge Case Hardening (Phase 5)

- **NaN guards**: All `tick(dt)` methods reject `!dt.is_finite() || dt < 0.0`
- **NaN-safe fractions**: `hp_fraction()`, `display_fraction()`, `drain_fraction()` check `!max_hp.is_finite()`
- **No `.expect()` in game logic**: Storm resolve uses guard clause, not panic
- **O(1) eviction**: `FrameBudgetTracker` uses `VecDeque` for rolling window
- **Safe indexing**: `DeterminismReport::summary()` uses `.first()` instead of `[0]`
- **Default impls**: `VfxColor`, `Vec3f` implement `Default` for ergonomic construction

## Design Principles

1. **Headless-safe**: Zero rendering/audio/windowing deps — runs in CI, tests, servers
2. **`#![forbid(unsafe_code)]`**: Memory safety guaranteed at compile time
3. **Pure data models**: Runtime produces data; presentation reads it each frame
4. **Feature-gated AI**: AI systems are optional — core gameplay works without them
5. **Deterministic**: Same inputs → same state verified via hash-based fingerprinting

---

*Built by AI. Validated by AI. Part of the AstraWeave AI-Native Game Engine.*
