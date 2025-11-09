---
description: Repository Information Overview
alwaysApply: true
---

# AstraWeave AI-Native Gaming Engine

## Summary

**AstraWeave** is a deterministic, ECS-based game engine written entirely in Rust where AI agents are first-class citizens. The engine implements a **Perception → Reasoning → Planning → Action** core loop integrated at the architectural level. This 80+ crate mono-workspace demonstrates production-ready infrastructure for AI-native gameplay, featuring advanced rendering (wgpu 25.0.2), deterministic physics (Rapier3D), sophisticated AI orchestration (GOAP, behavior trees, LLM integration), and comprehensive procedural generation.

## Repository Structure

### Core Engine Architecture
- **astraweave-ecs**: Archetype-based ECS system with deterministic system stages (7 stages: PERCEPTION → AI_PLANNING → PHYSICS → PRESENTATION)
- **astraweave-ai**: AI orchestrator with tool sandbox, core loop implementation, and 6 planning modes (Classical, BehaviorTree, Utility, LLM, Hybrid, Ensemble)
- **astraweave-render**: wgpu 25.0.2 renderer with PBR materials, GPU skinning, mesh optimization, and IBL
- **astraweave-physics**: Rapier3D integration with character controller, spatial hash optimization, and physics queries
- **astraweave-nav**: Navmesh-based pathfinding with A* and portal graph navigation
- **astraweave-behavior**: Behavior trees and utility AI systems
- **astraweave-terrain**: Hybrid voxel/polygon terrain with marching cubes mesh generation

### Supporting Systems
- **astraweave-audio**: Spatial audio engine with rodio backend
- **astraweave-dialogue**: Branching dialogue system with audio mapping
- **astraweave-gameplay**: Combat physics, attack sweeps, damage calculation
- **astraweave-math**: SIMD-optimized vector/matrix operations (2.08× speedup)
- **astraweave-scene**: World partitioning and async cell streaming
- **astraweave-cinematics**: Timeline-based sequencing for camera/audio/FX
- **astraweave-persistence-ecs**: ECS world serialization and save/load

### Tools & Development
- **tools/aw_editor**: 14-panel level/encounter editor with egui-wgpu
- **tools/aw_asset_cli**: Asset pipeline and material management
- **tools/ollama_probe**: LLM connection validator for Hermes 2 Pro integration
- **examples/**: 40+ working examples including hello_companion (Phase 6, all 6 AI modes)

## Language & Runtime

**Language**: Rust  
**Rust Version**: 1.89.0 (pinned via rust-toolchain.toml)  
**Edition**: 2021  
**Build System**: Cargo with workspace resolver v2  
**Targets Supported**: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin  
**Components**: rustfmt, clippy, rust-src, rust-analyzer, llvm-tools-preview

## Dependencies

**Core Graphics & Rendering**:
- wgpu 25.0.2 (GPU backend: Vulkan/DX12/Metal)
- egui 0.32 + egui-wgpu 0.32 (UI framework)
- winit 0.30 (window/input handling)
- glam 0.30 (math library with SIMD support)

**Physics & Simulation**:
- rapier3d 0.22 (physics engine)
- rand/rand_chacha 0.9 (deterministic RNG)

**AI & Language Models**:
- astraweave-llm (Hermes 2 Pro via Ollama, 37-tool vocabulary)
- rhai 1.23 (scripting language)

**Async Runtime & Networking**:
- tokio 1.x (async executor, full features)
- tungstenite 0.28 + tokio-tungstenite (WebSocket)
- reqwest 0.12 (HTTP client)

**Serialization & Storage**:
- serde 1 + serde_json 1 (serialization)
- toml 0.9 (config files)
- zip 6.0 (archive support)

**Audio**:
- rodio 0.17 (spatial audio backend)

**Security & Signing**:
- ed25519-dalek 2 (cryptographic signatures)
- sha2 0.10 (hashing)
- hex 0.4 (hex encoding)

**Development & Testing**:
- criterion 0.7 (benchmarking)
- proptest + custom fuzzing (ECS fuzz targets)
- tracing 0.1 (observability)

## Build & Installation

```bash
# Setup development environment
make setup
# OR: ./scripts/bootstrap.sh (cross-platform)

# Quick build (core components only)
make build
cargo build -p astraweave-ecs -p astraweave-ai -p astraweave-physics

# Build all working components
make build-all
cargo build-working

# Development checks (format + lint + test)
make dev
cargo fmt --all && cargo clippy-all && cargo test-all
```

**Build Profiles**:
- `dev`: Fast iteration (debug symbols, ~15-45 min first build, 8-15s incremental)
- `release`: Optimized runtime (`--release` required for examples)

**Build Aliases** (in `.cargo/config.toml`):
- `cargo check-all`: Workspace check excluding broken crates
- `cargo build-core`: Core components only
- `cargo build-working`: All working crates
- `cargo test-all`: Tests on working crates
- `cargo clippy-all`: Full linting with exclusions

## Testing

**Framework**: Native Rust cargo test + criterion benchmarks  
**Test Locations**: `crates/*/tests/`, `crates/*/benches/`  
**Naming Convention**: `*_tests.rs` for integration tests, `test_*` functions

**Key Test Suites**:
- **astraweave-ecs**: 15/15 passing (archetype operations, system dispatch, events)
- **astraweave-ai**: 4/5 passing (core loop, orchestrator, tool sandbox, LLM)
- **astraweave-physics**: 5/5 passing (character controller, raycasts, spatial hash)
- **astraweave-nav**: Navigation pathfinding validation
- **Integration Tests**: 9 cross-module tests (ECS→AI→Physics→Navigation pipeline)

**Run Tests**:
```bash
make test                    # All working crates
cargo test -p astraweave-ecs  # Specific crate
cargo test --release        # Optimized test run
```

**Benchmarking**:
```bash
# Full benchmark suite
cargo bench -p astraweave-ecs --bench ecs_benchmarks
cargo bench -p astraweave-ai --bench ai_core_loop
cargo bench -p astraweave-physics --bench raycast
cargo bench -p astraweave-math --bench simd_movement
```

**Performance Baselines** (Week 8 validated):
- ECS: 25.8 ns world creation, 420 ns/entity spawn
- AI Core Loop: 184 ns – 2.10 µs (2500× faster than 5ms target)
- Physics: 114 ns character move, 2.97 µs rigid body step
- SIMD Math: 2.08× speedup (20.588 µs → 9.879 µs @ 10k entities)
- Frame Time: 2.70 ms @ 1,000 entities (370 FPS, 84% headroom vs 60 FPS budget)
- AI-Native Capacity: **12,700+ agents @ 60 FPS** with 100% determinism

## Main Entry Points

**Working Examples**:
- `examples/hello_companion` - Phase 6 complete: all 6 AI modes + Hermes 2 Pro LLM, metrics export
- `examples/unified_showcase` - Comprehensive feature demonstration
- `examples/profiling_demo` - Week 8 Tracy profiling integration
- `examples/ui_menu_demo` - Phase 8.1 in-game UI framework (pause, settings, HUD)
- `examples/astract_gallery` - Astract declarative UI widget gallery (Day 10 complete)

**Tools**:
- `tools/aw_editor` - Level editor with 14 UI panels
- `tools/aw_asset_cli` - Asset pipeline management
- `tools/ollama_probe` - Verify Hermes 2 Pro LLM connectivity

**Build Commands**:
```bash
# Run primary example (all 6 AI modes)
cargo run -p hello_companion --release

# Run UI demo (Phase 8.1 menus and HUD)
cargo run -p ui_menu_demo --release

# Run profiler with Tracy integration
cargo run -p profiling_demo --release -- --entities 1000

# Run editor
cargo run -p aw_editor
```

## Docker Configuration

**Status**: No official Docker images maintained. The engine supports cross-compilation to Linux targets via rust-toolchain.toml configuration.

**Build for Linux**:
```bash
cargo build --target x86_64-unknown-linux-gnu --release
```

## Project Metadata

**Version**: 0.4.0  
**License**: MIT  
**Status**: Production-ready infrastructure (Phase 6 complete, Phase 8 in progress)  
**AI-Generated**: 100% AI-generated codebase (80+ crates, ~150K LOC) via iterative GitHub Copilot prompting  
**Last Updated**: November 1, 2025 (Option 3 Determinism Validation complete, 31/32 tests passing @ 96.9%)

## Strategic Roadmap

- **Phase 6** ✅ COMPLETE: Hermes 2 Pro LLM integration, 54 compilation errors → 0, 6 AI modes validated
- **Phase 7** ✅ COMPLETE: AI Arbiter system (GOAP + LLM hybrid), 37-tool vocabulary, 4-tier fallback system
- **Phase 8** 🔄 IN PROGRESS: Game Engine Readiness (UI framework, rendering completion, save/load, audio)
  - Week 1-3 ✅: UI framework (menu, HUD, dialogue) - 1,535 LOC, 42/42 tests
  - Week 4 🔄: Animations & polish (18-day zero-warning streak)
  - Priority 2-4: Rendering, save/load, production audio (9-12 weeks)

**See**: `docs/current/MASTER_ROADMAP.md`, `docs/root-archive/GAME_ENGINE_READINESS_ROADMAP.md`
