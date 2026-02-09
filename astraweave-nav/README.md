# astraweave-nav

Navmesh-based pathfinding for AstraWeave.

## Overview

Triangle-based navigation mesh baking, A* search, and dynamic mesh invalidation for runtime terrain changes. Minimal dependency footprint (just glam).

## Key Types

| Type | Description |
|------|-------------|
| `NavMesh` | Navigation mesh with `bake()`, A* pathfinding, dirty region tracking, rebake |
| `NavTri` | Navigation triangle with adjacency, slope, walkability |
| `Triangle` | Geometric triangle with area, normal, perimeter, degeneracy checks |
| `Aabb` | Axis-aligned bounding box with intersection, merge, containment |

## Usage

```rust
use astraweave_nav::NavMesh;

let mesh = NavMesh::bake(&triangles, max_step, max_slope);
let path = mesh.find_path(start_tri, goal_tri);
```

## License

MIT
