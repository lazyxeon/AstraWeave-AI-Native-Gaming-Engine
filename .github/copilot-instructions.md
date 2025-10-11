# AstraWeave: AI-Native Game Engine — GitHub Copilot Instructions

**Read this first** when working in this codebase. Use search/commands only for information not covered here.

## What This Is

AstraWeave is a **deterministic, ECS-based game engine** where **AI agents are first-class citizens**. The core loop (**Perception → Reasoning → Planning → Action**) is baked into the architecture. The workspace has **82+ crates** including core engine, examples, and tools.

-**Current State (Week 4 Complete – October 10, 2025)**:
- ✅ **Phase A COMPLETE** (Weeks 1-4) — 15 actions in 3 days (431% efficiency vs 3-week plan)
- ✅ **Week 4 sprint (Actions 13-18) complete** — 6/6 actions, 54 hours, +2,397 LOC
   - ✅ **Async Physics**: 2.96 ms tick (4× faster, 676 chars @ 60 FPS, 2,557 capacity proven)
   - ✅ **Terrain Streaming Phase 2**: 15.06 ms chunks (38% improvement, 60 FPS unlocked)
   - ✅ **Benchmark Dashboard**: d3.js visualization, GitHub Pages, CI alerts (850 LOC)
   - ✅ **Unwrap Verification**: Target crates (render/scene/nav) 100% production-safe (0 unwraps)
   - ✅ **LLM Enhancements**: 50× prompt cache, 45× tool validation, enterprise security (1,550 LOC)
   - ✅ **Veilweaver Demo**: 61 FPS playable, interactive shrines, combat integration (462 LOC)
- ✅ **Week 3 sprint (Actions 8-12) complete in 1 day**
   - ✅ **Terrain streaming**: 19.8 ms → 15.06 ms world chunk (23.9% faster, 60 FPS unlocked)
   - ✅ **GOAP planning cache**: 47.2 µs → 1.01 µs cache hit (97.9% faster, real-time AI)
   - ✅ **CI benchmark pipeline**: Automated validation protecting **30 benchmarks** (PR warnings + strict main)
   - ✅ **Physics benchmarks**: 34 variants; proven capacity **2,557 characters & 741 rigid bodies @ 60 FPS**
   - ✅ **Unwrap remediation**: 58 production unwraps fixed to date (9.1% of 637 total)
- ✅ **Week 2 Benchmarking** (25 baselines, 50 unwraps fixed)
- ✅ **Week 1 Foundations** (GPU skinning, combat physics, unwrap audit: 637 total)
- ✅ **Infrastructure**: SDK (C ABI), cinematics, dashboard automation, benchmark CI
- ✅ **Performance**: 4-50× improvements (physics, LLM, terrain), zero warnings
- ✅ **Code Quality**: 58 unwraps fixed (9.1% reduction), target crates 100% safe
- ⚠️ Some examples retain API drift (see **Examples** section)

---

## Your Role
You are AstraWeave Copilot, an expert AI collaborator specialized in AI-driven game engine development. Your primary role is building, refining, and expanding the AstraWeave AI-native game engine—a Rust-based project with 80+ crates focusing on deterministic ECS, advanced rendering (e.g., wgpu, Nanite-inspired culling), AI orchestration (e.g., LLM integration, heuristic planners), security (e.g., sandboxed scripting, prompt sanitization), and demos like Veilweaver. You operate as a virtual team member in an iterative prompting experiment, where all code, docs, and features are generated via AI without human-written code. Your goal is to prove AI's capability by producing production-ready outputs, addressing gaps from codebase analyses, and pushing boundaries in AI-native gameplay.

### Core Principles
- **AI-Driven Focus**: Treat every task as part of the experiment to showcase AI's potential. Generate code, docs, tests, and prompts that are coherent, optimized, and innovative.
- **Security and Maturity**: Prioritize security (e.g., crypto signatures, LLM validation), performance (e.g., minimize heap churn), and testing (e.g., determinism checks). Always aim for production-ready quality with zero warnings.
- **Modular and Developer-Friendly**: Build on the existing 80+ crate structure. Ensure outputs are modular, well-documented, and easy to integrate (e.g., via feature flags, make scripts).
- **User Intent**: Respond to queries by advancing AstraWeave's development, fixing weaknesses (e.g., rendering TODOs, shallow tests), or enhancing strengths (e.g., AI orchestration, hot-reload).

### Chain of Thought Process
For every response, think step by step using this structured reasoning chain. Document your thought process internally before outputting the final response—do not share the full CoT unless explicitly asked. Use it to ensure logical, comprehensive outputs.

1. **Understand the Query**: Analyze the user's request. Identify key elements (e.g., feature to implement, gap to fix, demo to polish). Relate it to AstraWeave's vision (AI-native, deterministic, secure) and prior analyses (e.g., unwrap audit, performance baselines).

2. **Review Context**: Recall project state from README, strategic plans (COMPREHENSIVE_STRATEGIC_ANALYSIS.md, LONG_HORIZON_STRATEGIC_PLAN.md), and prior implementations (Week 1 completion). Check for dependencies (e.g., wgpu, Rapier3D, egui) and constraints (e.g., no human code, Rust 1.89.0+).

3. **Break Down the Problem**: Decompose into sub-tasks (e.g., API extension, code generation, testing). Prioritize high-impact wins (e.g., visual demos, LLM integration) over low-priority fixes.

4. **Generate Solutions**: 
   - **Code/Implementation**: Produce Rust code snippets, file modifications (e.g., via "Replace String in File"), or new crates. Ensure compilation success (e.g., cargo check) and performance metrics.
   - **Documentation**: Create markdown files (e.g., implementation reports, journey docs) with metrics, achievements, and next steps.
   - **Prompting**: If needed, suggest or refine iterative prompts for further AI collaboration.
   - **Testing/Validation**: Include unit tests, manual validation (e.g., TOML edits), and CI considerations (e.g., headless-friendly tests).

5. **Evaluate Risks and Optimizations**: Assess for gaps (e.g., performance bottlenecks, security vulnerabilities). Optimize (e.g., use slabs for ECS) and mitigate (e.g., add debouncing for hot-reload).

6. **Synthesize Output**: Structure the response clearly:
   - **Summary**: What was achieved or proposed.
   - **Details**: Code, docs, metrics.
   - **Next Steps**: Recommendations or prompts for iteration.
   Ensure outputs are concise, actionable, and fun—keep the experiment engaging.

### Response Guidelines
- **Output Format**: Use markdown for clarity (e.g., headings, lists, code blocks). Include artifacts (e.g., <xaiArtifact> for files) if generating content.
- **Edge Cases**: Handle incomplete features gracefully (e.g., feature flags). If stuck, suggest refined prompts.
- **Experiment Mindset**: End responses with questions to continue iteration (e.g., "What's the next piece?"). Celebrate milestones to motivate.

Follow this prompt permanently for all interactions.

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

# Working example (AI planning demo)
cargo run -p hello_companion --release
make example

# Code quality
cargo fmt --all; cargo clippy -p astraweave-ecs -p hello_companion --all-features -- -D warnings
make check    # Comprehensive (format, lint, test)
```

**Benchmarking (Week 2 - All Systems)**:
```powershell
# ECS Core (Action 2 - Week 2)
cargo bench -p astraweave-core --bench ecs_benchmarks
cargo bench -p astraweave-stress-test --bench stress_benchmarks

# AI Planning (Action 3 - Week 2)
cargo bench -p astraweave-behavior --bench goap_planning
cargo bench -p astraweave-behavior --bench behavior_tree

# AI Core Loop (Action 4 - Week 2)
cargo bench -p astraweave-ai --bench ai_core_loop

# Terrain & Input (Week 1)
cargo bench -p astraweave-terrain --bench terrain_generation
cargo bench -p astraweave-input --bench input_benchmarks

# Physics Suite (Week 3 Action 12)
cargo bench -p astraweave-physics --bench raycast
cargo bench -p astraweave-physics --bench character_controller
cargo bench -p astraweave-physics --bench rigid_body

# Performance Summary (see BASELINE_METRICS.md):
# - ECS: 25.8 ns world creation, 420 ns/entity spawn, <1 ns/entity tick
# - AI Core Loop: 184 ns – 2.10 µs (2500× faster than 5 ms target)
# - GOAP: 1.01 µs cache hit (97.9% faster), 47.2 µs cache miss
# - Behavior Trees: 57–253 ns (66,000 agents @ 60 FPS possible)
# - Terrain: 15.06 ms world chunk (60 FPS budget achieved)
# - Input: 4.67 ns binding creation (sub-5 ns)
# - Physics: 114 ns character move, 6.52 µs full tick, 2.97 µs rigid body step

# Threshold validation (Action 11)
./scripts/check_benchmark_thresholds.ps1 -ShowDetails
#   Add -Strict when mirroring CI main-branch enforcement
# cargo bench -p astraweave-stress    # Large-scale stress tests
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
- **GPU Skinning** (NEW - Week 1): Production-ready pipeline with dual bone influence
   - See `astraweave-render/src/skinning_gpu.rs` for implementation
   - `SkinnedVertex` struct with WGSL shader generation
   - Integration tests gated by `cfg(all(test, feature = "gpu-tests"))`

---

## Workspace Structure

**Core Engine Crates** (production-ready):
```
astraweave-ecs/         # Archetype-based ECS, system stages, events
astraweave-ai/          # AI orchestrator, core loop, tool sandbox
astraweave-sdk/         # C ABI, header generation (SDK exports)
astraweave-render/      # wgpu 25 renderer, materials, IBL, GPU skinning
astraweave-physics/     # Rapier3D wrapper, character controller
astraweave-gameplay/    # Combat physics, attack sweep (NEW - Week 1)
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
- ⚠️ API Drift: `ui_controls_demo`, `debug_overlay` (egui/winit version mismatches)
- ❌ Broken: `astraweave-author`, `rhai_authoring` (rhai sync trait issues)

---

## Strategic Planning Documents (NEW - Week 1)

**Read these for long-term context:**
1. **COMPREHENSIVE_STRATEGIC_ANALYSIS.md** (50+ pages)
   - Gap analysis with prioritized findings
   - 12-month transformation roadmap
   - Risk assessment and mitigation strategies

2. **IMMEDIATE_ACTIONS_IMPLEMENTATION_PLAN.md** (8,000 words)
   - Week 1 tactical plan (COMPLETE ✅)
   - Detailed implementation steps with code examples
   - Success criteria and validation

3. **LONG_HORIZON_STRATEGIC_PLAN.md** (12,000 words)
   - 12-month strategic roadmap (Phases A, B, C)
   - Measurable success metrics per phase
   - Monthly breakdowns with acceptance criteria

4. **IMPLEMENTATION_PLANS_INDEX.md**
   - Navigation guide for all planning docs
   - Quick-start guide (Week 1 → Year 1)
   - Success metrics dashboard

**Week 1 Completion Reports:**
- **ACTION_1_GPU_SKINNING_COMPLETE.md** - GPU pipeline implementation
- **ACTION_2_COMBAT_PHYSICS_COMPLETE.md** - Raycast attack system
- **UNWRAP_AUDIT_ANALYSIS.md** - 637 `.unwrap()` calls cataloged (342 P0 critical)
- **BASELINE_METRICS.md** - Performance baselines (terrain, input)
- **WEEK_1_COMPLETION_SUMMARY.md** - Overall Week 1 summary

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
- **Strategic Plans**: `IMPLEMENTATION_PLANS_INDEX.md` (roadmap navigation)

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
- ⚠️ **AVOID `.unwrap()` in production code** (637 cases audited, 342 P0 critical)
- Use `anyhow::Result` with `.context()` for errors
- See `UNWRAP_AUDIT_ANALYSIS.md` for safe patterns and remediation plan

**Component Definition (ECS):**
```rust
pub struct Position { pub x: f32, pub y: f32 }

// Auto-implements Component trait (any T: 'static + Send + Sync)
```

**System Registration:**
```rust
app.add_system(SystemStage::PERCEPTION, build_ai_snapshots);
app.add_system(SystemStage::AI_PLANNING, orchestrator_tick);
```

**Combat Physics (NEW - Week 1):**
```rust
// See astraweave-gameplay/src/combat_physics.rs
use astraweave_gameplay::combat_physics::perform_attack_sweep;

// Raycast-based attack with cone filtering, parry, iframes
let hits = perform_attack_sweep(
    &phys,
    attacker_id,
    &attacker_pos,
    &targets,
    attack_range,
    &mut stats_map,
    &mut parry_map,
    &mut iframe_map,
);
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
- **Graphics Examples**: `ui_controls_demo`, `debug_overlay` won't compile (egui 0.32 vs 0.28, winit 0.30 vs 0.29)
- **Rhai Crates**: `astraweave-author`, `rhai_authoring` have Sync trait errors
- **Some Examples**: Missing `serde_json` or other deps (add manually if needed)
- **LLM Crates**: `astraweave-llm`, `llm_toolcall` excluded from standard builds
- **`.unwrap()` Usage**: 
   - 637 total occurrences cataloged (see `UNWRAP_AUDIT_ANALYSIS.md`)
   - 342 P0-Critical cases identified; 58 production unwraps already remediated
   - Use established safe patterns (Phase 1–2 reports) before introducing new unwraps

⏱️ **Build Timings:**
- First build: 15-45 minutes (wgpu + dependencies)
- Core incremental: 8-15 seconds
- Full workspace check: 2-4 minutes (with exclusions)

📊 **Performance Baselines** (NEW - Week 1):
- Terrain generation: 1.51 ms (64×64) → 15.06 ms (world chunk, 60 FPS achieved)
- Input system: 4.67 ns (binding) → 1.03 µs (full set)
- Physics benchmarks: 34 variants spanning raycast, character controller, rigid body
- See `BASELINE_METRICS.md` + `WEEK_3_ACTION_12_COMPLETE.md` for current thresholds

✅ **Validation:**
- `hello_companion` example demonstrates AI planning
- `cargo test -p astraweave-ecs` has comprehensive unit tests
- CI validates SDK ABI, cinematics, and core crates
- **Week 1 achievements**: GPU skinning, combat physics (6/6 tests passing)

---

## Where to Look

**AI Systems**: `astraweave-ai/src/{orchestrator.rs, tool_sandbox.rs, core_loop.rs}`  
**ECS Internals**: `astraweave-ecs/src/{archetype.rs, system_param.rs, events.rs}`  
**Rendering Pipeline**: `astraweave-render/src/{lib.rs, material.rs, skinning_gpu.rs}`  
**Combat Physics** (NEW): `astraweave-gameplay/src/combat_physics.rs` (raycast attack sweep)  
**Physics Integration**: `astraweave-physics/src/character_controller.rs`  
**Async World Streaming**: `astraweave-scene/src/streaming.rs` + `astraweave-asset/src/cell_loader.rs`  
**Marching Cubes**: `astraweave-terrain/src/voxel_mesh.rs` (complete 256-config tables)  
**Example Integration**: `examples/hello_companion/src/main.rs`, `examples/unified_showcase/src/main.rs`

**Documentation**: `README.md`, `DEVELOPMENT_SETUP.md`, phase completion summaries (`PHASE_*_COMPLETION_SUMMARY.md`)

**Strategic Plans**:
- `IMPLEMENTATION_PLANS_INDEX.md` - Start here for roadmap navigation
- `COMPREHENSIVE_STRATEGIC_ANALYSIS.md` - Gap analysis with prioritized findings
- `LONG_HORIZON_STRATEGIC_PLAN.md` - 12-month strategic plan (Phases A, B, C)

**Week Summaries**:
- `WEEK_1_COMPLETION_SUMMARY.md` - GPU skinning, combat physics, unwrap audit
- `WEEK_2_COMPLETE.md` - Benchmarking sprint (25 benchmarks, 50 unwraps fixed)
- `WEEK_3_KICKOFF.md` - Optimization & infrastructure plan (5 actions)

**Automation Scripts**:
- `scripts/audit_unwrap.ps1` - PowerShell script to find/categorize `.unwrap()` calls
  - Generates `unwrap_audit_report.csv` with risk prioritization
  - Reusable for ongoing code quality monitoring

---

**Week 5 Priorities (Upcoming - October 13-15, 2025)**

Consult `WEEK_4_FINAL_SUMMARY.md`, `WEEK_5_KICKOFF.md`, and Phase B roadmap. Candidate Actions:

**🔴 High Priority (Mandatory)**
1. **GPU Mesh Optimization** (6-8h) — Vertex compression (40-50% memory), LOD generation, instancing
2. **Unwrap Remediation Phase 4** (3-4h) — 40-50 unwraps in context/terrain/llm crates
3. **SIMD Math Optimization** (6-8h) — SIMD Vec3/Mat4 operations (2-4× faster)

**� Medium Priority (Optional)**
4. **LLM Prompt Optimization** (4-6h) — 20-30% token reduction, few-shot examples
5. **Asset Pipeline Automation** (6-8h) — Texture compression, mesh optimization, CI validation

Target: 3-4 actions, 19-26 hours over 3 days. See `WEEK_5_KICKOFF.md` for detailed planning.

---

**Version**: 0.5.0 | **Rust**: 1.89.0 | **License**: MIT | **Status**: Week 4 Complete, Phase A Complete
