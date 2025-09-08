
# Veilweaver: Threads of Eternity

<div align="center">

## Build & Platform Support
[![CI](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/ci.yml/badge.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/ci.yml)
[![Rust](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/rust.yml/badge.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/rust.yml)
[![Cross Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/ci.yml)
[![Rust Version](https://img.shields.io/badge/rust-1.89.0-orange.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/blob/main/rust-toolchain.toml)

## Security & Quality Assurance
[![Security Audit](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/security-audit.yml/badge.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/security-audit.yml)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/lazyxeon/Veilweaver-Threads-of-Eternity/badge)](https://scorecard.dev/viewer/?uri=github.com/lazyxeon/Veilweaver-Threads-of-Eternity)
[![CodeQL Analysis](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/codeql-analysis.yml/badge.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/codeql-analysis.yml)
[![Benchmarks](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/benchmark.yml/badge.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/benchmark.yml)

## Project Information
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/blob/main/LICENSE)
[![Version](https://img.shields.io/badge/version-0.4.0-blue.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/blob/main/Cargo.toml)
[![Rust Edition](https://img.shields.io/badge/edition-2021-red.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)

## Documentation & Community  
[![Documentation](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/docs.yml/badge.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/actions/workflows/docs.yml)
[![GitHub Issues](https://img.shields.io/github/issues/lazyxeon/Veilweaver-Threads-of-Eternity.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/lazyxeon/Veilweaver-Threads-of-Eternity.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/pulls)
[![Last Commit](https://img.shields.io/github/last-commit/lazyxeon/Veilweaver-Threads-of-Eternity.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/commits/main)
[![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/blob/main/CONTRIBUTING.md)

## Project Focus
[![AI-Native](https://img.shields.io/badge/AI--Native-Game%20Engine-purple.svg)](#project-focus)
[![Game Engine](https://img.shields.io/badge/Game%20Engine-3D%20RPG-darkgreen.svg)](#directory-structure)
[![Research Platform](https://img.shields.io/badge/Research-Platform-orange.svg)](#veilweaver-threads-of-eternity)
[![Rust Ecosystem](https://img.shields.io/badge/ecosystem-wgpu%20%7C%20rapier3d%20%7C%20rodio-brown.svg)](#development-environment)
[![GitHub Stars](https://img.shields.io/github/stars/lazyxeon/Veilweaver-Threads-of-Eternity?style=social)](https://github.com/lazyxeon/Veilweaver-Threads-of-Eternity/stargazers)

</div>

---
Veilweaver: Threads of Eternity is a research platform and playable vertical slice for
building AI‑native games. It delivers a complete engine and toolkit for creating rich,
emergent RPG experiences where companions learn, bosses adapt, and the world itself can
be altered through Fate‑Weaving. In addition to the core engine, the repo contains modular
gameplay systems, authoring tools, and demos showing how to stitch everything together.
 Highlights

## AI‑Native Engine (AstraWeave) – Deterministic ECS‑style simulation with strict tool‑sandboxing for
AI planners. Companions propose actions; the engine validates line‑of‑sight, cooldowns and
navmesh before execution. Includes 3D rendering (via wgpu), custom shaders, depth/lighting,
weather VFX and letterbox/fade overlays.

## Robust Physics & Navmesh – Rapier3D integration for rigid bodies, character controller (grounded/
climb/swim), ragdolls, destructible objects, buoyancy, wind and dynamic weather. Lightweight
navmesh baking and A* pathfinding with funnel/string‑pull refinement.

## Adaptive AI Systems – Persistent companion AI profiles ( .cprof ) that learn your tactics and
preferences. Boss director with multi‑phase behaviour, budgeted fortify/collapse/spawn operations,
portal graphs and telegraphed phase shifts.

## AI NPC system - all npc's have ai dialogue, actions, reactions.

## Gameplay Modules – Fate‑Weaving (terrain/weather manipulation with consequences), crafting and
echo infusion, combo‑based combat, resource harvesting, procedurally generated biomes,
branching dialogue with variables, quests and cutscenes.

## Audio & Dialogue – Background music with crossfades, spatial sound effects, voice‑over playback
and fallback text‑to‑speech (adapter), automatic music ducking during dialogue. A voice‑bank loader
and dialogue runtime map nodes to audio files.

## Authoring & Modding Tools – Rhai scripts for encounter tuning, persona packs for companions,
voice bank definitions, TOML‑based dialogue and quest files. Example demos show how to assemble
levels, dialogues, AI behaviour and physics into playable scenes.

## Network & IPC – WebSocket‑based intent replication for co‑op/multiplayer, with server‑authoritative
validation. IPC layer for swapping local AI with edge/cloud models.

---

## Getting Started
Clone the repository and install dependencies with Rust's nightly toolchain. Examples can be run directly
with Cargo:
# run a 3D world with basic companion planning and rendering
cargo run -p weaving_playground
# craft an item, infuse an echo and test a combat combo
cargo run -p crafting_combat_demo
# hear dialogue lines with voice‑over and subtitles
cargo run -p dialogue_voice_demo
# explore ragdolls, destructible objects, climbing and swimming in physics
playground
cargo run -p physics_demo3d
# observe an adaptive boss using multi‑phase director
cargo run -p phase_director
To see the navmesh bake and pathfinding in action:
cargo run -p navmesh_demo
For a cutscene with camera control and letterbox fade:
cargo run -p cutscene_render_demo
Development Environment
The project uses the following crates and tools:
wgpu for cross‑platform GPU rendering.
Rapier3D for physics simulation and kinematic character control.
rodio for audio playback (music, SFX, voice).
serde/toml for data‑driven content (recipes, quests, dialogue, voice maps).
rhai for scripting encounter budgets and hints.
Crossbeam/tokio (in the network examples) for async WebSocket servers.
Ensure you have Rust 1.73+ installed with cargo. Some demos require the --features ollama flag if you
want to integrate your own LLM backend.

---

## 🗂 Directory Structure
astraweave-core/ # engine core: ECS world, tool‑sandbox, validator,
snapshots/intents
astraweave-render/ # rendering backend (wgpu), camera controller, weather &
overlay effects
astraweave-physics/ # physics wrapper (Rapier3D) with character controller,
ragdolls, destructibles
astraweave-nav/ # navmesh baking and A* pathfinding with portal graphs
astraweave-gameplay/ # gameplay systems: weaving, crafting, combat, quests,
dialogue, cutscenes
astraweave-audio/ # audio engine: BGM, spatial SFX, voice playback, TTS
adapter
examples/ # runnable demos covering each subsystem and integrated
experiences
assets/ # sample data (dialogue, quests, recipes, voices)

---

## 🔒 Security & Quality Assurance

Veilweaver implements several security and quality assurance workflows to ensure code reliability and safety:

### OpenSSF Scorecard
We use the [OpenSSF Scorecard](https://securityscorecards.dev/) to continuously monitor our security posture and best practices compliance.

### Security Auditing
- **Dependency Scanning**: Automated checks for vulnerable dependencies using cargo-audit
- **License Compliance**: Verification of dependency licenses using cargo-deny
- **Vulnerability Reporting**: Clear process for reporting security issues (see [SECURITY.md](SECURITY.md))

### Code Quality
- **Static Analysis**: Rust's Clippy for code quality and potential bug detection
- **Formatting Standards**: Enforced code style with rustfmt
- **CodeQL Analysis**: Advanced static analysis for security vulnerabilities

### Continuous Integration
- **Cross-platform Testing**: Automated testing on Linux, Windows, and macOS
- **Performance Benchmarking**: Tracking performance metrics over time
- **Documentation Generation**: Automatic API documentation updates

For more information on contributing securely to this project, please see our [Contributing Guidelines](CONTRIBUTING.md).

---

# LICENSE
MIT License

---

# Contributing
Contributions are welcome! If you'd like to add new demos, improve performance, integrate a new AI
backend or expand gameplay mechanics, please open an issue or pull request. When contributing code,
please format with rustfmt and include unit tests or example scenes.

For detailed contribution guidelines, please see [CONTRIBUTING.md](CONTRIBUTING.md).

---

## Acknowledgements
Veilweaver was inspired by the desire to explore AI‑native games where companions and enemies adapt to
the player. Thanks to the open‑source community for wonderful crates like wgpu , Rapier3D and
rodio , without which this project would not exist.
