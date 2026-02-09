---
layout: default
title: Scene Subsystem
---

# Scene Management (astraweave-scene)

AstraWeave's scene system provides a transform hierarchy, node tree, and world partitioning for large open worlds.

## Core Types

### Transform

Position, rotation, and scale applied hierarchically:

```rust
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

### Node

Scene graph nodes with parent-child relationships:

```rust
pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub children: Vec<NodeId>,
    pub components: Vec<ComponentId>,
}
```

### Scene

A complete scene definition (serializable):

```rust
pub struct Scene {
    pub name: String,
    pub root: NodeId,
    pub nodes: Vec<Node>,
}
```

## World Partitioning

For large open worlds, the scene system divides the world into cells that stream in/out asynchronously:

```
World Grid
┌────┬────┬────┐
│ A1 │ A2 │ A3 │  ← Each cell loaded independently
├────┼────┼────┤
│ B1 │ B2 │ B3 │  ← Streaming based on player position
├────┼────┼────┤
│ C1 │ C2 │ C3 │
└────┴────┴────┘
```

### Cell Loading

```rust
use astraweave_scene::streaming::CellLoader;

// Async cell loading from RON files
let cell = CellLoader::load_cell("world/cell_b2.ron").await?;
```

## Integration

- **ECS**: Nodes map to entities with Transform components
- **Physics**: Cell boundaries define physics simulation regions
- **Rendering**: Only loaded cells are submitted to the render queue
- **AI**: WorldSnapshot filters entities by loaded cells

[← Back to Home](index.html) · [Architecture](architecture.html) · [Rendering](rendering.html)
