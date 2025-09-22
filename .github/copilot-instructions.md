# AstraWeave: AI-Native Game Engine - GitHub Copilot Instructions

**ALWAYS** reference these instructions first and fallback to search or commands only when you encounter unexpected information that does not match the info here.

AstraWeave is a deterministic, ECS-based game engine where **AI agents are first-class citizens**. Unlike traditional engines where AI is bolted on, AstraWeave implements the core AI loop (**Perception → Reasoning → Planning → Action**) directly into the simulation architecture. The codebase is a Rust workspace with 40+ crates and 23+ examples.

## Working Effectively

### Bootstrap and Build Process
- **Cross-Platform**: Supports Linux, macOS, Windows (PowerShell). Use `./scripts/bootstrap.sh` on Unix or review `DEVELOPMENT_SETUP.md` for Windows.

- **Rust Version**: Uses stable Rust 1.89.0+. The repository includes `rust-toolchain.toml` for automatic toolchain management.

- **Quick Setup**: Use convenience commands:
  ```bash
  make setup          # or ./scripts/bootstrap.sh  
  make build          # Build core components only
  make build-all      # Build all working components  
  make example        # Run hello_companion demo
  ```

- **Manual Core Build**: Build the functioning core components:
  ```bash
  cargo build -p astraweave-core -p astraweave-ai -p astraweave-physics \
              -p astraweave-nav -p astraweave-render -p hello_companion
  ```
  **Timing: 2-5 minutes depending on system. NEVER CANCEL - dependency compilation can take 15+ minutes initially.**

- **Workspace Check with Exclusions**: Many compilation issues have been fixed, but some crates still have issues:
  ```bash
  cargo check --workspace --exclude astraweave-author --exclude visual_3d \
              --exclude ui_controls_demo --exclude npc_town_demo --exclude rhai_authoring \
              --exclude debug_overlay --exclude cutscene_render_demo --exclude weaving_playground \
              --exclude combat_physics_demo --exclude navmesh_demo --exclude physics_demo3d \
              --exclude debug_toolkit_demo --exclude aw_debug --exclude aw_editor \
              --exclude aw_asset_cli --exclude astraweave-llm --exclude llm_toolcall \
              --exclude llm_integration
  ```

### Testing
- **Core Tests**: Run unit tests on working crates:
  ```bash
  cargo test -p astraweave-input  # Has actual unit tests
  make test                       # Runs tests on all working components
  ```
  **Timing: 6-30 seconds. Most crates are demo-heavy rather than test-heavy.**

- **Integration Testing**: Use the working hello_companion example to validate AI systems:
  ```bash
  cargo run -p hello_companion --release
  make example
  ```

### Code Quality
- **Format Check**: 
  ```bash
  cargo fmt --all --check  # Check formatting
  make format              # Apply formatting
  ```

- **Linting**:
  ```bash
  cargo clippy -p astraweave-core -p hello_companion --all-features -- -D warnings
  make lint                # Run comprehensive linting
  ```

- **Development Workflow**: Use convenience commands:
  ```bash
  make check               # Run comprehensive checks (format, lint, test)
  make dev                 # Quick development cycle (format, lint, test)
  make ci                  # Full CI-style validation
  ```

- **Security Audit**: 
  ```bash
  cargo audit              # Check for known vulnerabilities
  cargo deny check         # License and dependency validation
  make audit               # Combined security checks
  ```

## Validation

### Working Examples
- **hello_companion**: Builds and runs (panics on LOS logic but demonstrates AI planning):
  ```bash
  cargo run -p hello_companion --release
  ```
  Expected output: Shows AI plan generation, then panics with "LosBlocked" error.

### Manual Testing Scenarios
- **CANNOT fully validate graphics examples** due to compilation errors in visual_3d and UI demos
- **Basic AI Logic**: hello_companion demonstrates AI companion planning and intent validation
- **Test Infrastructure**: astraweave-input has working unit tests

### **CRITICAL LIMITATIONS**
- **Graphics Examples Don't Work**: visual_3d, debug_overlay, ui_controls_demo have API mismatches  
- **Many Examples Missing Dependencies**: Need manual `serde_json` additions
- **No End-to-End Validation Possible**: Cannot test complete user scenarios due to build issues

## Repository Structure

```
astraweave-core/        # ECS world, validation, intent system  
astraweave-ai/          # AI orchestrator and planning
astraweave-render/      # wgpu-based 3D rendering
astraweave-physics/     # Rapier3D wrapper with character controller
astraweave-nav/         # Navmesh baking and A* pathfinding  
astraweave-gameplay/    # Weaving, crafting, combat, dialogue
astraweave-audio/       # Audio engine with spatial effects
examples/               # 23 demos (MANY BROKEN)
assets/                 # Sample data files
```

## Key Architecture Patterns

### AI-First Design Philosophy
- **Perception → Reasoning → Planning → Action** loop is core to all AI systems
- `WorldSnapshot` structs provide structured, filtered world state to AI agents
- `PlanIntent` with `ActionStep` sequences define all AI behavior
- **Tool validation** ensures AI actions are physically/logically valid
- `Orchestrator` trait abstracts AI planning (rule-based vs LLM-based)

### ECS and Data Flow
- Entities have core components: Position (`IVec2`), Health, Team, Cooldowns
- `World` struct manages entities, obstacles, and simulation state
- Fixed-tick simulation (60Hz target) with deterministic time progression
- `build_snapshot()` filters world state for specific agent perception
- `validate_and_execute()` processes AI plans through engine validation

### Component Communication Patterns
- Core crates export clear public APIs (see `lib.rs` files)
- Heavy use of `anyhow::Result` for error handling
- Workspace dependencies centralized in root `Cargo.toml`
- Examples demonstrate integration patterns, not production code

## Important Build Information

### Working Dependencies
- **Graphics**: wgpu 0.20, winit 0.29, egui 0.28
- **Physics**: rapier3d 0.22
- **Audio**: rodio 0.17  
- **AI/Scripting**: rhai 1.22 (HAS SYNC ISSUES in some crates)

### Known Compilation Issues
- **astraweave-author**: rhai trait sync errors
- **rhai_authoring**: Depends on broken astraweave-author
- **npc_town_demo**: API mismatches
- **debug_overlay**: egui API changes
- **visual_3d**: winit Arc<Window> mismatch

### Performance Notes
- **Initial Build**: 15-45+ minutes (estimate based on partial builds)
- **Incremental Build**: 8-15 seconds for core components
- **Release Build**: Faster, use for testing examples

## Critical Warnings

- **DO NOT** attempt to build full workspace without excluding broken crates
- **DO NOT** try to run graphics examples - they won't compile
- **ALWAYS** use long timeouts (30+ minutes) for builds
- **NEVER CANCEL** long-running builds - they are normal for Rust graphics projects
- **EXPECT** runtime panics in examples - they demonstrate concepts but have logic issues

## When Working with This Codebase

1. **Start with core libraries**: Focus on astraweave-core, astraweave-ai, astraweave-physics
2. **Fix one crate at a time**: Don't attempt workspace-wide fixes
3. **Use release builds** for faster iteration when testing examples
4. **Check individual crate dependencies** before building examples
5. **ALWAYS validate changes** with the working core components first