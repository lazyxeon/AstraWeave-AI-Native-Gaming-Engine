---
layout: default
title: Development Setup
---

# Development Setup

## Prerequisites

| Requirement | Version | Purpose |
|-------------|---------|---------|
| **Rust** | 1.89.0+ | Compiler and cargo |
| **Git** | Any | Version control |
| **Ollama** | Latest (optional) | LLM features |
| **Tracy** | 0.11+ (optional) | Performance profiling |

## Initial Setup

```bash
# Clone the repository
git clone https://github.com/YOUR_ORG/AstraWeave-AI-Native-Gaming-Engine.git
cd AstraWeave-AI-Native-Gaming-Engine

# First build (15-45 min — wgpu + graphics dependencies)
cargo build

# Verify with workspace check
cargo check-all
```

## Cargo Aliases

The workspace defines convenient aliases in `.cargo/config.toml`:

```bash
cargo check-all    # Workspace check (excludes broken crates)
cargo build-core   # Core components only
cargo test-all     # Tests on all working crates
cargo clippy-all   # Full linting with -D warnings
```

## Running Tests

```bash
# All tests
cargo test-all

# Specific crate
cargo test -p astraweave-ecs
cargo test -p astraweave-physics --lib
cargo test -p astraweave-fluids --lib

# With async-physics feature
cargo test -p astraweave-physics --features async-physics --lib

# AI demo (requires release mode for performance)
cargo run -p hello_companion --release
```

## Running Benchmarks

```bash
cargo bench -p astraweave-core
cargo bench -p astraweave-physics
cargo bench -p astraweave-ai
```

## LLM Setup (Optional)

To use LLM-powered AI features:

1. Install [Ollama](https://ollama.ai)
2. Pull the Hermes 2 Pro model:
   ```bash
   ollama pull adrienbrault/nous-hermes2pro:Q4_K_M
   ```
3. Run with LLM features:
   ```bash
   cargo run -p hello_companion --release --features llm_orchestrator
   ```

## Project Structure

```
AstraWeave-AI-Native-Gaming-Engine/
├── astraweave-ecs/         # Entity Component System
├── astraweave-core/        # Shared types and schema
├── astraweave-ai/          # AI orchestration
├── astraweave-render/      # GPU rendering (wgpu 25)
├── astraweave-physics/     # Physics (Rapier3D)
├── astraweave-math/        # SIMD math operations
├── astraweave-gameplay/    # Game systems
├── astraweave-nav/         # Navmesh pathfinding
├── astraweave-audio/       # Spatial audio
├── astraweave-scene/       # Scene graph
├── astraweave-ui/          # In-game UI (egui)
├── astraweave-input/       # Input handling
├── astraweave-sdk/         # C ABI
├── examples/               # 59 runnable examples
├── tools/                  # Editor and CLI tools
├── docs/                   # Internal documentation
└── gh-pages/               # This documentation site
```

## Development Workflow

1. Make changes in one crate at a time
2. **Quick check**: `cargo check -p <crate>` (mandatory after every change)
3. **Fix errors**: All compilation errors must be resolved immediately
4. **Test**: `cargo test -p <crate>`
5. **Format**: `cargo fmt --all`
6. **Lint**: `cargo clippy -p <crate> --all-features -- -D warnings`
7. **Integration**: Run `hello_companion` or `unified_showcase`

## Build Timings

| Build Type | Duration |
|------------|----------|
| First build (cold) | 15–45 min |
| Core incremental | 8–15 sec |
| Full workspace check | 2–4 min |

[← Back to Home](index.html)
