# API Documentation

Browse the auto-generated Rust API documentation for all AstraWeave crates.

## Quick Links

| Crate | Description | Status |
|-------|-------------|--------|
| [astraweave_core](astraweave_core/index.html) | Core ECS, simulation, and world systems | Production |
| [astraweave_ecs](astraweave_ecs/index.html) | Entity Component System implementation | Production |
| [astraweave_ai](astraweave_ai/index.html) | AI orchestration and planning | Production |
| [astraweave_behavior](astraweave_behavior/index.html) | Behavior trees and GOAP | Production |
| [astraweave_llm](astraweave_llm/index.html) | LLM integration layer | Beta |
| [astraweave_render](astraweave_render/index.html) | wgpu-based renderer | Production |
| [astraweave_physics](astraweave_physics/index.html) | Rapier3D integration | Production |
| [astraweave_nav](astraweave_nav/index.html) | Navigation and pathfinding | Production |
| [astraweave_gameplay](astraweave_gameplay/index.html) | Combat, crafting, dialogue | Production |
| [astraweave_terrain](astraweave_terrain/index.html) | Voxel terrain generation | Production |

## Core Engine Crates

### astraweave-core

The heart of AstraWeave. Contains:

- **World** - The game world container
- **Simulation** - Fixed-tick deterministic simulation loop
- **Time** - Frame timing and delta management
- **Tools** - AI tool vocabulary and validation
- **Perception** - AI perception bus system

```rust
use astraweave_core::{World, Simulation, Time};
```

[View Full Documentation](astraweave_core/index.html)

### astraweave-ecs

High-performance Entity Component System:

- **Entity** - Lightweight entity handles with generational indices
- **Component** - Trait for component data
- **System** - Query-based system execution
- **Archetype Storage** - Cache-friendly component storage

```rust
use astraweave_ecs::{Entity, World, Query};
```

[View Full Documentation](astraweave_ecs/index.html)

## AI & Behavior Crates

### astraweave-ai

AI orchestration layer:

- **Arbiter** - Tool validation and sandboxing
- **Orchestrator** - Multi-agent coordination
- **CoreLoop** - Perception-Reasoning-Planning-Action loop

```rust
use astraweave_ai::{AiArbiter, Orchestrator};
```

[View Full Documentation](astraweave_ai/index.html)

### astraweave-behavior

Behavior systems:

- **BehaviorTree** - Hierarchical behavior nodes
- **GOAP** - Goal-Oriented Action Planning
- **Blackboard** - Shared data for AI agents

```rust
use astraweave_behavior::{BehaviorTree, GoapPlanner};
```

[View Full Documentation](astraweave_behavior/index.html)

### astraweave-llm

LLM integration:

- **LlmAdapter** - Provider-agnostic LLM interface
- **BatchExecutor** - Batched inference for efficiency
- **StreamingParser** - Real-time response parsing
- **PromptTemplate** - Type-safe prompt construction

```rust
use astraweave_llm::{LlmAdapter, BatchExecutor};
```

[View Full Documentation](astraweave_llm/index.html)

## Rendering & Graphics Crates

### astraweave-render

wgpu-based rendering:

- **Renderer** - Main render pipeline
- **PBR Materials** - Physically-based rendering
- **Clustered Lighting** - Efficient many-light rendering
- **Nanite-style LOD** - Virtualized geometry

```rust
use astraweave_render::{Renderer, Material, Mesh};
```

[View Full Documentation](astraweave_render/index.html)

## Physics & Navigation Crates

### astraweave-physics

Rapier3D integration:

- **RigidBody** - Physics bodies
- **Collider** - Collision shapes
- **CharacterController** - Player movement
- **SpatialHash** - Broad-phase acceleration

```rust
use astraweave_physics::{RigidBody, Collider};
```

[View Full Documentation](astraweave_physics/index.html)

### astraweave-nav

Navigation and pathfinding:

- **Navmesh** - Navigation mesh generation
- **Pathfinding** - A* and hierarchical path planning
- **Agent** - Navigation agent component

```rust
use astraweave_nav::{Navmesh, PathQuery};
```

[View Full Documentation](astraweave_nav/index.html)

## Gameplay Crates

### astraweave-gameplay

Game systems:

- **Combat** - Damage, effects, abilities
- **Crafting** - Recipe and item systems
- **Dialogue** - Conversation management
- **Quests** - Quest state machine

```rust
use astraweave_gameplay::{Combat, CraftingSystem};
```

[View Full Documentation](astraweave_gameplay/index.html)

### astraweave-terrain

Procedural terrain:

- **VoxelTerrain** - Voxel-based terrain
- **BiomeSystem** - Biome distribution
- **TerrainModifier** - Runtime terrain modification

```rust
use astraweave_terrain::{VoxelTerrain, Biome};
```

[View Full Documentation](astraweave_terrain/index.html)

## External Documentation

- **[docs.rs](https://docs.rs)** - Published crate documentation (when published)
- **[crates.io](https://crates.io)** - Package registry (when published)

## Building API Docs Locally

```bash
# Generate all API documentation
cargo doc --workspace --no-deps --open

# Generate for specific crate
cargo doc --package astraweave-core --no-deps --open
```

## See Also

- [Crate Documentation](../reference/crates.md) - High-level crate overview
- [Architecture Overview](../architecture/overview.md) - System design
- [Examples](../examples/index.md) - Working code examples
