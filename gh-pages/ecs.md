---
layout: default
title: ECS Subsystem
---

# Entity Component System (astraweave-ecs)

AstraWeave's ECS is an archetype-based system with deterministic execution ordering and 7 fixed system stages.

## Key Features

- **Archetype storage**: Components grouped by type-set for cache-friendly iteration
- **Deterministic ordering**: Fixed 60 Hz tick with ordered entity iteration
- **System stages**: 7 stages executed in strict sequence every frame
- **Events**: Typed event channels for decoupled communication
- **Miri-validated**: 386 tests, zero undefined behavior

## Core Types

### World

```rust
let mut world = World::new();
let entity = world.spawn();
world.insert(entity, Position { x: 0.0, y: 0.0 });
let pos = world.get::<Position>(entity);
```

### Components

Any `'static + Send + Sync` type is automatically a component:

```rust
pub struct Position { pub x: f32, pub y: f32 }
pub struct Velocity { pub dx: f32, pub dy: f32 }
pub struct Health(pub f32);
```

### System Stages

Systems execute in this deterministic order every frame:

| Stage | Purpose |
|-------|---------|
| `PRE_SIMULATION` | Setup, initialization |
| `PERCEPTION` | Build WorldSnapshots |
| `SIMULATION` | Game logic, cooldowns |
| `AI_PLANNING` | Generate PlanIntents |
| `PHYSICS` | Forces, collisions |
| `POST_SIMULATION` | Cleanup, constraints |
| `PRESENTATION` | Rendering, audio, UI |

### Registration

```rust
app.add_system(SystemStage::PERCEPTION, build_ai_snapshots);
app.add_system(SystemStage::AI_PLANNING, orchestrator_tick);
app.add_system(SystemStage::PHYSICS, physics_step);
```

## Performance

| Operation | Latency |
|-----------|---------|
| World creation | 25.8 ns |
| Entity spawn | 420 ns |
| Per-entity tick | <1 ns |
| Component add | ~100 ns |

## Memory Safety

All unsafe code in the ECS has been validated with Miri (386 tests):

- **BlobVec**: Untyped heap storage for archetype columns
- **SparseSet**: Entity-indexed sparse storage
- **EntityAllocator**: Generation-based entity recycling
- **SystemParam**: Type-erased system parameter access

[← Back to Home](index.html) · [Architecture](architecture.html) · [Crate Index](crates.html)
