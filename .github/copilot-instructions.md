# AstraWeave: AI-Native Game Engine — GitHub Copilot Instructions

**Read this first** when working in this codebase. Use search/commands only for information not covered here.

## What This Is

AstraWeave is a **deterministic, ECS-based game engine** where **AI agents are first-class citizens**. The core loop (**Perception → Reasoning → Planning → Action**) is baked into the architecture. The workspace has **50+ crates** including core engine, examples, and tools.

**Current State (Phase 4 Complete - October 2025)**:
- ✅ SDK with stable C ABI + auto-generated headers (validated in CI)
- ✅ Cinematics timeline/sequencer (UI integration, load/save)
- ✅ Core engine builds in 8-15s incremental
- ✅ Production-ready: astraweave-ecs, -ai, -physics, -render, -nav, -audio
- ⚠️ Some examples have API drift issues (see below)

---

## Quick Commands (Windows PowerShell)

**Setup & Build:**
```powershell
# Automated setup (handles Rust, dependencies, validation)
./scripts/bootstrap.sh       # Cross-platform (use Git Bash on Windows)
make setup                   # Alternative via Makefile

# Fast build (core components only - 2-5 min first time, 8-15s incremental)
make build
cargo build -p astraweave-ecs -p astraweave-ai -p astraweave-physics -p astraweave-nav -p astraweave-render -p hello_companion

# Workspace check (excludes broken crates - use task or alias)
# Task: "Phase1-check" in .vscode/tasks.json
# OR: cargo check-all (alias in .cargo/config.toml)
```

**Testing & Validation:**
```powershell
# Core tests (6-30 seconds)
cargo test -p astraweave-ecs -p astraweave-ai -p astraweave-physics -p astraweave-nav -p astraweave-audio
make test

# Working example (AI planning demo - expect LOS panic)
cargo run -p hello_companion --release
make example

# Code quality
cargo fmt --all; cargo clippy -p astraweave-ecs -p hello_companion --all-features -- -D warnings
make check    # Comprehensive (format, lint, test)
```

**Key Cargo Aliases** (in `.cargo/config.toml`):
- `cargo check-all` - Workspace check with exclusions
- `cargo build-core` - Core components only
- `cargo test-all` - Tests on working crates
- `cargo clippy-all` - Full linting with exclusions

---

## Architecture Essentials

### AI-First Loop (Core Pattern Everywhere)
```
Perception → Reasoning → Planning → Action
    ↓           ↓            ↓          ↓
WorldSnapshot  AI Model   PlanIntent  Tool Validation
```

**Key Concepts:**
- `WorldSnapshot`: Filtered world state for AI perception (see `astraweave-ai/src/core_loop.rs`)
- `PlanIntent` + `ActionStep`: AI decisions as validated action sequences
- `Orchestrator` trait: Abstracts AI planning (rule-based vs LLM)
- **Tool Sandbox**: All AI actions validated by engine (no cheating possible)

### ECS System Stages (astraweave-ecs)
Deterministic, ordered execution:
1. **PRE_SIMULATION** - Setup, initialization
2. **PERCEPTION** - Build WorldSnapshots, update AI sensors
3. **SIMULATION** - Game logic, cooldowns, state updates
4. **AI_PLANNING** - Generate PlanIntents from orchestrators
5. **PHYSICS** - Apply forces, resolve collisions
6. **POST_SIMULATION** - Cleanup, constraint resolution
7. **PRESENTATION** - Rendering, audio, UI updates

**Fixed 60Hz tick** with deterministic RNG and ordered entity iteration.

### Rendering & Materials (astraweave-render)
- **wgpu 25.0.2** backend (Vulkan/DX12/Metal via wgpu)
- **Material System**: TOML → GPU D2 array textures with stable indices
  - Pattern: `assets/materials/<biome>/{materials.toml, arrays.toml}`
  - WGSL bindings (group=1): albedo (0), sampler (1), normal (2), linear sampler (3), MRA (4)
- **Shared Utilities**: `MaterialManager`, `IblManager`, `MeshRegistry`
- **Feature Flags**: `textures`, `assets` gate loaders

---

## Workspace Structure

**Core Engine Crates** (production-ready):
```
astraweave-ecs/         # Archetype-based ECS, system stages, events
astraweave-ai/          # AI orchestrator, core loop, tool sandbox
astraweave-sdk/         # C ABI, header generation (SDK exports)
astraweave-render/      # wgpu 25 renderer, materials, IBL
astraweave-physics/     # Rapier3D wrapper, character controller
astraweave-nav/         # Navmesh, A*, portal graphs
astraweave-audio/       # Spatial audio, rodio backend
astraweave-scene/       # World partition, async cell streaming
astraweave-terrain/     # Voxel/polygon hybrid, marching cubes
astraweave-cinematics/  # Timeline, sequencer, camera/audio/FX tracks
```

**Gameplay & Tools**:
```
astraweave-behavior/    # Behavior trees, utility AI
astraweave-weaving/     # Fate-weaving system (Veilweaver game mechanic)
astraweave-pcg/         # Procedural content generation
tools/aw_editor/        # Level/encounter editor (GUI)
tools/aw_asset_cli/     # Asset pipeline tooling
```

**Examples** (`examples/`):
- ✅ Working: `hello_companion`, `unified_showcase`, `core_loop_bt_demo`, `core_loop_goap_demo`, `weaving_pcg_demo`
- ⚠️ API Drift: `visual_3d`, `ui_controls_demo`, `debug_overlay` (egui/winit version mismatches)
- ❌ Broken: `astraweave-author`, `rhai_authoring` (rhai sync trait issues)

---

## Working Effectively

### Build Strategy
**DO:**
- Build incrementally (`-p` flag for single crates)
- Use cargo aliases (`check-all`, `build-core`) or VS Code tasks
- Let initial builds complete (15-45 min first time - normal for Rust graphics projects)
- Use `--release` for examples (much faster runtime)

**DON'T:**
- Attempt full workspace builds without exclusions (broken crates will fail)
- Cancel long-running builds (dependency compilation takes time)
- Try to fix broken examples without checking API versions first

### Development Workflow
1. **Make changes** in one crate at a time
2. **Quick check**: `cargo check -p <crate>` (fast feedback)
3. **Test**: `cargo test -p <crate>` (if tests exist)
4. **Format**: `cargo fmt --all` (before commit)
5. **Lint**: `cargo clippy -p <crate> --all-features -- -D warnings`
6. **Integration**: Run `hello_companion` or `unified_showcase` to validate

### Key Files to Check
- **Public APIs**: Each crate's `src/lib.rs` (exports)
- **Workspace Deps**: Root `Cargo.toml` (centralized versions)
- **Build Config**: `.cargo/config.toml` (aliases, profiles, sccache)
- **CI Tasks**: `.vscode/tasks.json` (Phase1-check, Phase1-tests)
- **Exclusions**: See `check-all` alias for crates to skip

---

## Common Patterns & Conventions

**Error Handling:**
```rust
use anyhow::{Context, Result};
fn do_work() -> Result<()> {
    something().context("Failed to do work")?;
    Ok(())
}
```
- Use `anyhow::Result` with `.context()` for errors
- Avoid panics in core crates (examples can panic for demo purposes)

**Component Definition (ECS):**
```rust
#[derive(Clone, Copy)]
pub struct Position { pub x: f32, pub y: f32 }

// Auto-implements Component trait (any T: 'static + Send + Sync)
```

**System Registration:**
```rust
app.add_system(SystemStage::PERCEPTION, build_ai_snapshots);
app.add_system(SystemStage::AI_PLANNING, orchestrator_tick);
```

**Asset Loading (async pattern):**
```rust
// See astraweave-asset/src/cell_loader.rs
use tokio::fs;
pub async fn load_cell_from_ron(path: &Path) -> Result<CellData> {
    let content = fs::read_to_string(path).await?;
    Ok(ron::from_str(&content)?)
}
```

---

## Critical Warnings

⚠️ **Known Issues:**
- **Graphics Examples**: `visual_3d`, `debug_overlay` won't compile (egui 0.32 vs 0.28, winit 0.30 vs 0.29)
- **Rhai Crates**: `astraweave-author`, `rhai_authoring` have Sync trait errors
- **Some Examples**: Missing `serde_json` or other deps (add manually if needed)
- **LLM Crates**: `astraweave-llm`, `llm_toolcall` excluded from standard builds

⏱️ **Build Timings:**
- First build: 15-45 minutes (wgpu + dependencies)
- Core incremental: 8-15 seconds
- Full workspace check: 2-4 minutes (with exclusions)

✅ **Validation:**
- `hello_companion` example demonstrates AI planning (expect LOS panic - this is intentional)
- `cargo test -p astraweave-ecs` has comprehensive unit tests
- CI validates SDK ABI, cinematics, and core crates

---

## Where to Look

**AI Systems**: `astraweave-ai/src/{orchestrator.rs, tool_sandbox.rs, core_loop.rs}`  
**ECS Internals**: `astraweave-ecs/src/{archetype.rs, system_param.rs, events.rs}`  
**Rendering Pipeline**: `astraweave-render/src/{lib.rs, material.rs}`  
**Physics Integration**: `astraweave-physics/src/character_controller.rs`  
**Async World Streaming**: `astraweave-scene/src/streaming.rs` + `astraweave-asset/src/cell_loader.rs`  
**Marching Cubes**: `astraweave-terrain/src/voxel_mesh.rs` (complete 256-config tables)  
**Example Integration**: `examples/hello_companion/src/main.rs`, `examples/unified_showcase/src/main.rs`

**Documentation**: `README.md`, `DEVELOPMENT_SETUP.md`, phase completion summaries (`PHASE_*_COMPLETION_SUMMARY.md`)

---

**Version**: 0.4.0 | **Rust**: 1.89.0 | **License**: MIT | **Status**: Production-ready core (Phase 4 complete)
