# AstraWeave Developer Documentation

**AstraWeave** is an AI-Native Gaming Engine built in Rust, featuring real-time AI companion systems, procedural content generation, and modern rendering capabilities.

---

## Table of Contents

| Section | Description |
|---------|-------------|
| [Quick Start](#quick-start) | Get started in 5 minutes |
| [Guides](#guides) | How-to guides and tutorials |
| [Reference](#reference) | API and technical reference |
| [Architecture](#architecture) | System design documentation |

---

## Quick Start

**[→ QUICKSTART.md](QUICKSTART.md)** - Complete setup and first steps guide

```bash
# Clone and build
git clone https://github.com/your-org/AstraWeave-AI-Native-Gaming-Engine.git
cd AstraWeave-AI-Native-Gaming-Engine
cargo build --release

# Run example
cargo run --example simple_scene
```

---

## Documentation Structure

```
docs/
├── README.md              # This file
├── QUICKSTART.md          # Getting started guide
├── masters/               # ⭐ AUTHORITATIVE MASTER REPORTS
│   ├── MASTER_ROADMAP.md
│   ├── MASTER_BENCHMARK_REPORT.md
│   └── MASTER_COVERAGE_REPORT.md
├── guides/                # How-to guides and tutorials
│   ├── BUILD_QUICK_REFERENCE.md
│   ├── RUST_TOOLCHAIN_GUIDE.md
│   ├── SECURITY_AUDIT_GUIDE.md
│   ├── assets_pipeline.md
│   └── ...
├── reference/             # Technical reference
│   ├── Interfaces.md
│   ├── error_codes.md
│   ├── authoring_schemas.md
│   └── ...
├── current/               # Active planning docs
├── audits/                # Codebase audit reports
├── lessons/               # Lessons learned
├── pbr/                   # PBR rendering docs
├── src/                   # mdBook source
└── archive/               # Historical documentation
```

---

## Guides

Developer guides for common tasks and workflows.

### Getting Started
- **[QUICKSTART.md](QUICKSTART.md)** - Complete setup guide
- **[BUILD_QUICK_REFERENCE.md](guides/BUILD_QUICK_REFERENCE.md)** - Build commands
- **[RUST_TOOLCHAIN_GUIDE.md](guides/RUST_TOOLCHAIN_GUIDE.md)** - Rust setup

### Asset Pipeline
- **[assets_pipeline.md](guides/assets_pipeline.md)** - Asset workflow
- **[POLYHAVEN_QUICK_START.md](guides/POLYHAVEN_QUICK_START.md)** - PBR assets
- **[ASSET_AND_TEXTURE_INDEX.md](guides/ASSET_AND_TEXTURE_INDEX.md)** - Asset reference

### AI & Scripting
- **[ai-scripting.md](guides/ai-scripting.md)** - Rhai scripting
- **[behavior_and_quests.md](guides/behavior_and_quests.md)** - Behavior trees
- **[hierarchical_goals_designer_guide.md](guides/hierarchical_goals_designer_guide.md)** - Goal systems

### Security
- **[SECURITY_AUDIT_GUIDE.md](guides/SECURITY_AUDIT_GUIDE.md)** - Security procedures
- **[SECURITY_QUICK_REFERENCE.md](guides/SECURITY_QUICK_REFERENCE.md)** - Quick reference

### Performance
- **[BENCHMARKING_GUIDE.md](guides/BENCHMARKING_GUIDE.md)** - Benchmarking
- **[build-optimization.md](guides/build-optimization.md)** - Build optimization

---

## Reference

Technical reference documentation.

### Engine Core
- **[Interfaces.md](reference/Interfaces.md)** - Core interfaces
- **[engine-api.md](reference/engine-api.md)** - Engine API
- **[error_codes.md](reference/error_codes.md)** - Error codes

### Standards
- **[naming_and_style.md](reference/naming_and_style.md)** - Code conventions
- **[platform_matrix.md](reference/platform_matrix.md)** - Platform support
- **[perf_budgets.md](reference/perf_budgets.md)** - Performance budgets

### Schemas
- **[authoring_schemas.md](reference/authoring_schemas.md)** - Data schemas
- **[asset_ids_and_cache.md](reference/asset_ids_and_cache.md)** - Asset IDs

---

## Architecture

For architectural documentation, see:

- **[current/](current/)** - Active development plans
- **[pbr/](pbr/)** - PBR rendering architecture
- **[audits/](audits/)** - Codebase audits

---

## Crate Documentation

Each crate contains its own README with specific documentation:

| Crate | Description |
|-------|-------------|
| `astraweave-core` | Core ECS and game loop |
| `astraweave-render` | Rendering engine |
| `astraweave-ai` | AI companion system |
| `astraweave-llm` | LLM integration |
| `astraweave-behavior` | Behavior trees |
| `astraweave-security` | Security and validation |
| `astraweave-net` | Networking |
| `astraweave-ui` | UI framework |

---

## Contributing

See the repository root `CONTRIBUTING.md` for contribution guidelines.

---

*Last Updated: December 2025*