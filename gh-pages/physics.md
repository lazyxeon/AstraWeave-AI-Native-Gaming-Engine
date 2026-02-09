---
layout: default
title: Physics Subsystem
---

# Physics (astraweave-physics)

AstraWeave wraps **Rapier3D 0.22** with additional subsystems for game-ready physics simulation.

## Subsystems

| Subsystem | Grade | Tests | Description |
|-----------|-------|-------|-------------|
| **Core / Character Controller** | A | 110+ | Movement, gravity, NaN safety |
| **Spatial Hash** | A- | 41 | Grid broadphase, FxHashMap |
| **Async Scheduler** | B+ | 13+ | 3-stage parallel pipeline (rayon) |
| **Fluids (SPH)** | A+ | 2,404 | Reference implementation |
| **Environment** | A- | 55+ | Wind, buoyancy |
| **Vehicle** | B+ | 50+ | Wheel/suspension |
| **Gravity** | B+ | 30+ | Multi-body gravity |
| **Cloth** | B | 25+ | Verlet integration |
| **Ragdoll** | B | 33+ | Joint-based ragdoll |
| **Destruction** | C+ | 17 | Fracture, debris |
| **Projectile** | C+ | 21 | Ballistics, penetration |

**Total**: 598+ tests passing (`--features async-physics`)

## Spatial Hash

A grid-based spatial partitioning system for O(n log n) broadphase:

```rust
let mut grid = SpatialHash::new(2.0); // 2.0 unit cell size
grid.insert(entity_id, aabb);
let neighbors = grid.query(&query_aabb);
```

Key stats:
- **99.96% collision pair reduction** (499,500 → 180 checks)
- FxHashMap for fast integer hashing
- Cache-locality cascading improves all downstream systems by 9-17%

## Async Scheduler

A 3-stage parallel pipeline using rayon:

```
Broad Phase → Barrier → Narrow Phase → Barrier → Integration
   (rayon)                  (rayon)                  (rayon)
```

```rust
let (broad, narrow, integrated) = scheduler.step_parallel_staged(
    bodies.par_iter(),
    |body| compute_aabb(body),       // Stage 1: Broad phase
    |aabb| detect_collision(aabb),   // Stage 2: Narrow phase
    |col| resolve_contact(col),      // Stage 3: Integration
);
```

With timing statistics:

```rust
let (result, stats) = scheduler.step_parallel_staged_with_stats(
    bodies.par_iter(), broad_fn, narrow_fn, integrate_fn,
);
println!("Broad: {:?}, Narrow: {:?}, Integrate: {:?}",
    stats.broad_phase, stats.narrow_phase, stats.integration);
```

## Character Controller

```rust
let move_result = controller.move_character(
    &physics_world,
    character_entity,
    desired_velocity * dt,
);
// 114 ns per move operation
```

## Performance

| Operation | Latency |
|-----------|---------|
| Character move | 114 ns |
| Rigid body step | 2.97 µs |
| Full physics tick | 6.52 µs |
| Spatial hash broadphase | 3.77 ms |

[← Back to Home](index.html) · [Architecture](architecture.html) · [Navigation](navigation.html)
