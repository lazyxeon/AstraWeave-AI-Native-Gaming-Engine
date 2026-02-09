# astraweave-scene

Hierarchical scene graph with transform management and world partitioning.

## Overview

Provides a scene graph with parent-child transform hierarchy, spatial world partitioning for large open worlds, and async cell streaming.

## Key Types

| Type | Description |
|------|-------------|
| `Transform` | Translation/rotation/scale with lerp, compose, inverse |
| `Node` | Scene graph node with children and mesh/material refs |
| `Scene` | Top-level container managing the node hierarchy |

## Modules

- **`world_partition`** — Spatial partitioning for open worlds
- **`streaming`** — Async cell streaming with distance-based loading
- **`gpu_resource_manager`** — GPU resource lifecycle management
- **`partitioned_scene`** — Partitioned scene graph

## Feature Flags

| Feature | Description |
|---------|-------------|
| `ecs` | ECS system integration |
| `world-partition` | Spatial partitioning and streaming |

## License

MIT
