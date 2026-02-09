---
layout: default
title: Navigation Subsystem
---

# Navigation (astraweave-nav)

AstraWeave provides navmesh-based pathfinding with A* search and portal graph support.

## Features

- **NavMesh baking**: Automatic generation from world geometry
- **A* pathfinding**: Optimal path search on triangulated meshes
- **Portal graphs**: Fast hierarchical pathfinding between areas
- **Dynamic invalidation**: Rebuild portions of the navmesh as the world changes
- **Integration with AI**: WorldSnapshot includes accessible paths for AI planning

## Core Types

### NavMesh

```rust
use astraweave_nav::{NavMesh, Triangle, Aabb};

// Build a navmesh from triangles
let mesh = NavMesh::from_triangles(&triangles);

// Find a path from start to goal
let path = mesh.find_path(start_pos, goal_pos);
```

### Triangle

The fundamental navmesh primitive:

```rust
pub struct Triangle {
    pub vertices: [[f32; 3]; 3],
    pub neighbors: [Option<usize>; 3],
}
```

### Aabb

Axis-aligned bounding box for spatial queries:

```rust
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
```

## Integration with AI

AI agents use navigation through the WorldSnapshot:

```
PERCEPTION stage → Build WorldSnapshot (include nav data)
AI_PLANNING stage → Orchestrator queries navmesh for valid paths
PHYSICS stage → Character controller follows planned path
```

[← Back to Home](index.html) · [Architecture](architecture.html) · [Physics](physics.html)
