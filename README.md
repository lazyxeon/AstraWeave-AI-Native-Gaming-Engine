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

<p align="center">
  <img src="https://img.shields.io/github/repo-size/lazyxeon/AstraWeave-AI-Native-Gaming-Engine?style=flat-square" alt="Repo Size" />
  <img src="https://img.shields.io/github/languages/code-size/lazyxeon/AstraWeave-AI-Native-Gaming-Engine?style=flat-square" alt="Code Size" />
  <img src="https://img.shields.io/github/commit-activity/m/lazyxeon/AstraWeave-AI-Native-Gaming-Engine?style=flat-square" alt="Commit Activity" />
  <img src="https://img.shields.io/github/issues/lazyxeon/AstraWeave-AI-Native-Gaming-Engine?style=flat-square" alt="Issues" />
  <img src="https://img.shields.io/github/issues-pr/lazyxeon/AstraWeave-AI-Native-Gaming-Engine?style=flat-square" alt="Pull Requests" />
</p>

<div align="center">

**The world's first AI-native game engine with deterministic ECS architecture where AI agents are first‑class citizens.**  
Built in Rust, designed for massive-scale intelligent worlds with production-grade performance.

📚 [Documentation](docs/) • 📊 [Benchmarks](docs/masters/MASTER_BENCHMARK_REPORT.md) • 🗺️ [Roadmap](docs/masters/MASTER_ROADMAP.md) • 🧪 [Coverage](docs/masters/MASTER_COVERAGE_REPORT.md)

**[Interactive Benchmark Dashboard](https://lazyxeon.github.io/AstraWeave-AI-Native-Gaming-Engine/)** • Local: `.\scripts\run_benchmark_dashboard.ps1`

</div>

---

## 🚀 Quick Start

```bash
git clone https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine.git
cd AstraWeave-AI-Native-Gaming-Engine

# Build core engine (Note: Editor currently undergoing remediation)
cargo build --release -p astraweave-core

# Run the flagship AI companion demo (6 planning modes)
cargo run -p hello_companion --release

# Run the rendering showcase (Island scene)
cargo run -p unified_showcase --release
```

**Note**: Full workspace build is currently blocked by the editor; individual crates and examples build successfully. See `docs/supplemental-docs/DEVELOPMENT_SETUP.md` for detailed setup.

---

## 🌌 Why AstraWeave?

Traditional game engines bolt AI onto simulation. **AstraWeave weaves AI into the core.**

In AstraWeave, the "Game Loop" is an **Intelligence Loop**:
1.  **Perception**: Agents "see" the world through a snapshot system.
2.  **Reasoning**: LLMs and Utility systems analyze the state.
3.  **Planning**: GOAP and Behavior Trees formulate plans.
4.  **Action**: Plans execute via deterministic ECS commands.

This architecture enables **12,700+ intelligent agents** running at **60 FPS** with complex reasoning, not just simple state machines.

---

## 🏗️ Architecture

```mermaid
flowchart TB
    A[Perception] --> B[Reasoning]
    B --> C[Planning]
    C --> D[Action]
    D --> E[Validation]
    E --> F[Simulation]
    F --> A
    
    style A fill:#4a90e2
    style B fill:#7b68ee
    style C fill:#50c878
    style D fill:#ffa500
    style E fill:#ff6b6b
    style F fill:#45b7d1
```

**7-Stage Execution Pipeline:**
1. Pre-Simulation → 2. Perception → 3. Simulation → 4. AI Planning → 5. Physics → 6. Post-Simulation → 7. Presentation

---

## ✨ Key Features

### 🧠 AI & Agents
-   **Multi-Modal Intelligence**: 6 validated AI modes including LLM (Hermes 2 Pro), GOAP, Behavior Trees, and Hybrid ensembles.
-   **Massive Scale**: Orchestrates 12,700+ agents @ 60 FPS.
-   **LLM Integration**: Streaming API, batch executor, and response caching.
-   **Dynamic Terrain**: ✅ **Production** AI-orchestrated terrain generation with LLM integration.
-   **Scripting**: **Active/Alpha** Rhai-based scripting system for behavior logic (`astraweave-scripting`).
-   **Generative AI**: **Experimental** Asset generation pipeline (`astraweave-ai-gen`).

### ⚙️ Core Engine
-   **Deterministic ECS**: 100% bit-identical replay validation.
-   **Performance**: Fixed 60Hz simulation, SIMD acceleration (glam), cache-friendly archetype storage.
-   **Networking**: Client-server architecture with delta encoding and state synchronization.
-   **Persistence**: ECS world save/load with version migration.

### 🎨 Rendering (wgpu)
-   **AAA Pipeline**: Cook-Torrance PBR, IBL, and clustered forward lighting (100k+ lights).
-   **Advanced Effects**: VXGI, Volumetric Fog, SSAO, SSR, Bloom, DOF, Motion Blur.
-   **Optimization**: Nanite-inspired virtualized geometry, GPU occlusion culling.
-   **Materials**: Advanced shaders (Clearcoat, SSS, Anisotropy).

### 🍎 Physics & Simulation
-   **Rapier3D Integration**: Rigid bodies, character controllers, and spatial queries.
-   **Navigation**: Navmesh generation (Delaunay) + A* pathfinding (142k queries/sec).
-   **Terrain**: Voxel-based terrain with AI-orchestrated dynamic modification.
-   **Audio**: Spatial audio with occlusion and dialogue runtime.

---

## 📊 Project Status

**Overall Status**: Phase 8 (Game Engine Readiness) - ~85% Production Ready.

| Component | Status | Notes |
| :--- | :--- | :--- |
| **Core ECS** | ✅ Production Ready | 96.67% coverage, rock solid. |
| **Rendering** | ✅ Production Ready | Feature complete AAA pipeline. |
| **Physics/Nav** | ✅ Production Ready | Highly optimized integration. |
| **AI Orchestration** | ✅ Production Ready | Validated at scale. |
| **Scripting** | ⚠️ Alpha | Functional Rhai integration, expanding API. |
| **Editor** | ❌ Remediation | Currently broken, undergoing 4-6 week fix. |
| **UI Framework** | ⚠️ Beta | Functional but low test coverage. |
| **LLM Support** | ⚠️ Beta | Working but needs higher test coverage. |
| **AI Generation** | 🧪 Experimental | Prototype asset generation pipeline. |

### 🏆 Quality Metrics
-   **Test Coverage**: 71.37% (94.71% in Core Systems)
-   **Performance**: 60 FPS @ 12,700 agents
-   **Security**: A- (92/100)

---

## 📦 Crate Ecosystem

AstraWeave is a workspace of **80+ crates**. Here are the highlights:

### Core & Infrastructure
-   `astraweave-core`: The heart of the engine (ECS, World, Time).
-   `astraweave-ecs`: Custom high-performance ECS.
-   `astraweave-math`: SIMD math utilities.
-   `astraweave-profiling`: Tracy integration.

### AI & Behavior
-   `astraweave-ai`: AI orchestration and planning systems.
-   `astraweave-llm`: LLM inference and management.
-   `astraweave-behavior`: Behavior trees and GOAP.
-   `astraweave-scripting`: Rhai scripting integration.
-   `astraweave-ai-gen`: **[Experimental]** AI asset generation.

### Rendering & Assets
-   `astraweave-render`: WGPU-based renderer.
-   `astraweave-materials`: PBR material system.
-   `astraweave-asset`: Asset management and streaming.

### Gameplay & Tools
-   `astraweave-physics`: Physics integration.
-   `astraweave-nav`: Navigation and pathfinding.
-   `astraweave-terrain`: Voxel terrain with AI-orchestrated dynamic generation.
-   `astraweave-quests`: LLM-powered quest generation with terrain integration.
-   `astraweave-ui`: Egui-based UI framework.
-   `tools/aw_editor`: The editor (currently in remediation).

---

## 🤝 Contributing

AstraWeave is an experimental project built **100% by AI** to prove AI's capability to create production-grade systems.

**Current Focus:**
1.  Fixing Editor compilation errors.
2.  Improving UI and LLM test coverage.
3.  Stabilizing the Scripting and AI Generation crates.

See `CONTRIBUTING.md` and `docs/current/MASTER_ROADMAP.md` for details.

---

<div align="center">

**Building the future of AI‑native gaming.**  
If this experiment interests you, please ⭐ the repo.

</div>
