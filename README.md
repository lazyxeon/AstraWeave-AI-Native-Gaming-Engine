<p align="center">
  <img src="assets/Astraweave_logo.jpg" alt="AstraWeave nebula logomark" width="360" />
</p>

<h1 align="center">AstraWeave — AI‑Native Game Engine</h1>

<p align="center">
  <a href="https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine/stargazers"><img src="https://img.shields.io/github/stars/lazyxeon/AstraWeave-AI-Native-Gaming-Engine?style=for-the-badge&logo=github" alt="GitHub stars" /></a>
  <a href="https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine/blob/main/LICENSE"><img src="https://img.shields.io/github/license/lazyxeon/AstraWeave-AI-Native-Gaming-Engine?style=for-the-badge" alt="License" /></a>
  <a href="https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine/blob/main/rust-toolchain.toml"><img src="https://img.shields.io/badge/rust-1.89.0-orange.svg?style=for-the-badge" alt="Rust toolchain" /></a>
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue.svg?style=for-the-badge" alt="Platforms" />
</p>

<div align="center">

Deterministic, ECS-based engine where AI agents are first‑class citizens. Built 100% by AI, validated with industry‑leading tests, and optimized for massive‑scale intelligent worlds.

📚 Docs: <a href="docs/">/docs</a> • 📈 Benchmarks: <a href="docs/current/MASTER_BENCHMARK_REPORT.md">Master</a> • 🗺️ Roadmap: <a href="docs/current/MASTER_ROADMAP.md">Master</a> • 🧪 Coverage: <a href="docs/current/MASTER_COVERAGE_REPORT.md">Master</a>

**📊 [Interactive Benchmark Dashboard](https://lazyxeon.github.io/AstraWeave-AI-Native-Gaming-Engine/)** • Local: `.\scripts\run_benchmark_dashboard.ps1`

</div>

---

## Snapshot (Nov 10, 2025)

- 12,700+ agents at 60 FPS • 100% deterministic replay
- Phase 8.1 Week 4 + Astract Gizmo Sprint COMPLETE (Oct 14–Nov 3, 2025)
  - Week 4: Animations & polish (health bars, damage numbers, quest notifications)
  - Astract Gizmo: Declarative UI framework (animation system, gallery, 5 tutorials, API docs, benchmarks)
- 166+ tests passing (Astract) + 42/42 tests (Phase 8.1 Week 3) + comprehensive integration tests
- Hermes 2 Pro LLM integrated; hybrid GOAP+LLM orchestration

For details, see the master reports linked above and the validation summaries in `docs/root-archive/`.

### At a glance

```mermaid
flowchart LR
  A[Perception] --> B[Reasoning]
  B --> C[Planning]
  C --> D[Action]
  subgraph Engine Validation
    D --> E[Tool Sandbox]
    E -->|Deterministic 60Hz| F[Simulation]
  end
```

```mermaid
pie showData
  title 60 FPS budget usage (typical scene)
  "AI Core" : 6
  "Physics" : 18
  "Render" : 28
  "Other" : 8
  "Headroom" : 40
```

```mermaid
gantt
  dateFormat  YYYY-MM-DD
  title Phase 8 – Game Engine Readiness
  section UI Framework (P1)
  Week 1–3 (menus, HUD)    :done,    2025-10-14, 2025-11-01
  Week 4 (polish)          :active,  2025-11-02, 2025-11-15
  Astract UI Framework     :done,    2025-11-02, 2025-11-03
  section Rendering (P2)
  Shadows, PostFX          :         2025-11-16, 2025-12-14
  section Save/Load (P3)
  ECS serialization        :         2025-12-15, 2025-12-29
  section Audio (P4)
  Mixer + dynamics         :         2025-12-30, 2026-01-13
```

---

## Key features

- AI‑first loop baked into ECS stages (Perception → Reasoning → Planning → Action)
- Deterministic 60 Hz simulation, capture/replay, RNG seeding (100% bit-identical replay)
- Modern renderer (wgpu 25.0.2): PBR materials, IBL, GPU skinning, mesh optimization, LOD
- Physics (Rapier3D) with spatial hash optimization (99.96% collision reduction)
- Navigation (navmesh + A*) and SIMD math (2.08× speedup @ 10k entities)
- Production tooling: Tracy 0.11.1 profiling, SDK C ABI, comprehensive testing

See detailed architecture and subsystem docs in `docs/` — this README stays concise.

---

## Quick start

```bash
git clone https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine.git
cd AstraWeave-AI-Native-Gaming-Engine
./scripts/bootstrap.sh    # or: make setup
```

Run a demo:

```bash
cargo run -p hello_companion --release
cargo run -p unified_showcase --release
```

More setup tips: `docs/supplemental-docs/DEVELOPMENT_SETUP.md`.

---

## Benchmarks (Week 8 validated)

- **Frame Time**: 2.70 ms @ 1,000 entities (370 FPS, 84% headroom vs 60 FPS budget)
- **AI Core Loop**: 184 ns – 2.10 µs (2,500× faster than 5 ms target)
- **Physics**: 114 ns character move, 2.97 µs rigid body step
- **Validation**: 6.48M checks/sec, 100% deterministic @ 60 Hz
- **AI-Native Capacity**: 12,700+ agents @ 60 FPS with full AI orchestration

Complete methodology, per-crate metrics, and historical runs: `docs/current/MASTER_BENCHMARK_REPORT.md` and `docs/root-archive/BASELINE_METRICS.md`.

---

## 📊 Benchmark Dashboard

**One-command launch**: View interactive performance charts, time series, and historical data.

```powershell
# Run benchmarks, export data, generate graphs, and open dashboard
.\scripts\run_benchmark_dashboard.ps1

# Or skip running benchmarks (use existing data)
.\scripts\run_benchmark_dashboard.ps1 -SkipBench
```

**Or double-click**: `Launch-Benchmark-Dashboard.bat` in the repository root.

**Live Dashboard**: [https://lazyxeon.github.io/AstraWeave-AI-Native-Gaming-Engine/](https://lazyxeon.github.io/AstraWeave-AI-Native-Gaming-Engine/) (updated nightly via CI)

The dashboard displays:
- 📈 Interactive time series charts (D3.js)
- 📊 Distribution histograms and sparklines
- 📉 Benchmark regression detection
- 🗂️ Sortable performance tables with percent changes
- 🖼️ Static PNG graphs for reports

See `tools/benchmark-dashboard/QUICK_START.md` for troubleshooting and configuration.

---

## Demos

```bash
# AI companion – all 6 planning modes (Phase 6)
cargo run -p hello_companion --release

# UI framework gallery – declarative widgets & animations (Astract Gizmo)
cargo run -p astract_gallery --release

# Profiling demo – Tracy integration with spatial hash optimization
cargo run -p profiling_demo --release -- --entities 1000

# Unified showcase – island with assets, rendering, physics
cargo run -p unified_showcase --release
```

**Status**: hello_companion, profiling_demo, unified_showcase, and astract_gallery are fully working. See `docs/root-archive/` for completion details.

---

## Documentation

- Start here: `docs/current/MASTER_ROADMAP.md`
- Performance: `docs/current/MASTER_BENCHMARK_REPORT.md`
- Coverage: `docs/current/MASTER_COVERAGE_REPORT.md`
- Attribution & Licenses: `docs/current/ATTRIBUTIONS.md`
- Deep dives & historical reports: `docs/root-archive/`

---

## Status & License

- **Version**: 0.4.0 • **Rust**: 1.89.0 • **Edition**: 2021
- **Phase 8** (Game Engine Readiness): UI Framework Week 4 ongoing (animations & polish), Astract Gizmo complete
- **Test Coverage**: 96.9% determinism validated, 166+ tests (Astract), 42/42 HUD tests
- **Licensed** under MIT — see `LICENSE`

---

<div align="center">

Building the future of AI‑native gaming. If this helps you, please ⭐ the repo.

</div>
