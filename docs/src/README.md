# Veilweaver: Threads of Eternity Documentation

Welcome to the documentation for Veilweaver: Threads of Eternity, an AI-native action RPG engine built with Rust.

## Quick Navigation

- [Overview](./overview.md) - Executive summary and feature overview
- [API Documentation](./api/index.html) - Generated Rust API documentation

## About This Project

Veilweaver is an AI-native action RPG engine that demonstrates:

- **AI Companions**: Persistent learning profiles that adapt to player tactics
- **Adaptive Bosses**: Multi-phase boss directors with budgeted operations  
- **Dynamic World**: Fate-Weaving system for terrain and weather changes
- **Server Authority**: Multiplayer co-op with validation and intent replication
- **Modular Architecture**: Clean separation between rendering, physics, AI, and networking

## Getting Started

The engine is built as a Rust workspace with multiple crates:

- `astraweave-core` - ECS world model and AI tool sandbox
- `astraweave-render` - wgpu-based 3D rendering
- `astraweave-physics` - Rapier3D integration with character controller
- `astraweave-ai` - AI orchestrator and planning systems
- `astraweave-nav` - Navmesh baking and A* pathfinding

See the [Overview](./overview.md) for a complete feature breakdown and the API documentation for implementation details.