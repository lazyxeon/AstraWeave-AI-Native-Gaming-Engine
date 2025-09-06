# Veilweaver: Threads of Eternity - GitHub Copilot Instructions

**ALWAYS** reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

Veilweaver is an AI-native action RPG engine built with Rust. It uses the AstraWeave engine core with modules for AI companions, physics, rendering, audio, and networking. The codebase is a Rust workspace with 44+ crates and 23 demo examples.

## Working Effectively

### Bootstrap and Build Process
- **System Dependencies (Linux)**: Run these commands to install required packages:
  ```bash
  sudo apt-get update
  sudo apt-get install -y build-essential pkg-config cmake ninja-build \
    libx11-dev libxi-dev libxcursor-dev libxrandr-dev libxinerama-dev \
    libxkbcommon-dev libxkbcommon-x11-dev libx11-xcb-dev libxcb1-dev \
    libxcb-randr0-dev libxcb-xfixes0-dev libxcb-shape0-dev libxcb-xkb-dev \
    libgl1-mesa-dev libegl1-mesa-dev wayland-protocols libwayland-dev \
    libasound2-dev libpulse-dev libudev-dev mesa-vulkan-drivers vulkan-tools
  ```

- **Rust Version**: Uses stable Rust (tested with 1.89.0). The repository includes `rust-toolchain.toml`.

- **Core Build (Working Components)**: Build the functioning core components:
  ```bash
  cargo build -p astraweave-core -p astraweave-ai -p astraweave-physics \
              -p astraweave-nav -p astraweave-render -p hello_companion
  ```
  **Timing: 8-15 seconds after initial dependency download. NEVER CANCEL - set timeout to 30+ minutes.**

- **Full Workspace Build (HAS ISSUES)**: DO NOT use `cargo build --workspace` as some crates have compilation errors:
  - `astraweave-author` - rhai sync/send trait issues  
  - `visual_3d` and graphics examples - API mismatches with egui/winit
  - Many examples missing `serde_json` dependency

### Testing
- **Core Tests**: Run unit tests on working crates:
  ```bash
  cargo test -p astraweave-input
  ```
  **Timing: 6 seconds. NEVER CANCEL - set timeout to 15+ minutes.**

- **Most crates have no tests**: The project is demo-heavy rather than test-heavy.

### Code Quality
- **Format Check**: 
  ```bash
  cargo fmt --all -- --check
  ```
  **CURRENTLY FAILS**: Many formatting violations exist in the codebase.

- **Linting**:
  ```bash
  cargo clippy -p astraweave-core -p hello_companion --all-features -- -D warnings
  ```
  **CURRENTLY FAILS**: Multiple clippy warnings including unused imports and identity operations.

- **Security Audit**: 
  ```bash
  cargo audit
  cargo deny check
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

Fixes #12.