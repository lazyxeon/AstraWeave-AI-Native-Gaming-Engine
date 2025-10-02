# World Partition System

## Overview

The World Partition system enables AstraWeave to handle large open worlds by dividing the game world into a grid of cells that are streamed in and out based on camera position. This keeps memory usage bounded while allowing seamless exploration of vast environments.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    WorldPartition                            │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Grid (HashMap<GridCoord, Cell>)           │ │
│  │                                                         │ │
│  │  Cell(0,0)    Cell(1,0)    Cell(2,0)                  │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                │ │
│  │  │Entities │  │Entities │  │Entities │                │ │
│  │  │Assets   │  │Assets   │  │Assets   │                │ │
│  │  │State    │  │State    │  │State    │                │ │
│  │  └─────────┘  └─────────┘  └─────────┘                │ │
│  │                                                         │ │
│  │  Cell(0,1)    Cell(1,1)    Cell(2,1)                  │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                │ │
│  │  │Entities │  │Entities │  │Entities │                │ │
│  │  │Assets   │  │Assets   │  │Assets   │                │ │
│  │  │State    │  │State    │  │State    │                │ │
│  │  └─────────┘  └─────────┘  └─────────┘                │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              WorldPartitionManager                           │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Active Cells (based on camera frustum)               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐            │ │
│  │  │ Cell(1,1)│  │ Cell(2,1)│  │ Cell(1,2)│            │ │
│  │  └──────────┘  └──────────┘  └──────────┘            │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  LRU Cache (recently unloaded cells)                  │ │
│  │  [Cell(0,0)] → [Cell(0,1)] → [Cell(3,3)]             │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Async Loader (tokio tasks)                           │ │
│  │  Loading: Cell(2,2), Cell(3,2)                        │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. WorldPartition

The core data structure that manages the grid of cells.

- **GridCoord**: 3D integer coordinates for cells (x, y, z)
- **Cell**: Contains entities, assets, and state for a grid cell
- **AABB**: Axis-aligned bounding box for spatial queries
- **GridConfig**: Configuration for cell size and world bounds

### 2. WorldPartitionManager

Handles async streaming of cells based on camera position.

- **Streaming Logic**: Determines which cells to load/unload
- **LRU Cache**: Keeps recently unloaded cells for quick reload
- **Event System**: Emits events for cell load/unload operations
- **Metrics**: Tracks performance and memory usage

### 3. PartitionedScene

Integration layer between Scene and WorldPartition.

- Wraps Scene with partitioning capabilities
- Provides convenient API for streaming updates
- Manages lifecycle of partitioned content

## Usage

### Basic Setup

```rust
use astraweave_scene::world_partition::{GridConfig, WorldPartition};
use astraweave_scene::streaming::{StreamingConfig, WorldPartitionManager};
use astraweave_scene::partitioned_scene::PartitionedScene;
use glam::Vec3;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    // Configure grid
    let grid_config = GridConfig {
        cell_size: 100.0,  // 100m cells
        world_bounds: (-5000.0, 5000.0, -5000.0, 5000.0),  // 10km x 10km
    };

    // Configure streaming
    let streaming_config = StreamingConfig {
        max_active_cells: 25,
        lru_cache_size: 5,
        streaming_radius: 500.0,  // 500m
        max_concurrent_loads: 4,
    };

    // Create partitioned scene
    let mut scene = PartitionedScene::new(grid_config, streaming_config);

    // Update streaming based on camera position
    let camera_pos = Vec3::new(0.0, 0.0, 0.0);
    scene.update_streaming(camera_pos).await.unwrap();

    // Check metrics
    let metrics = scene.metrics();
    println!("Active cells: {}", metrics.active_cells);
    println!("Memory usage: {} bytes", metrics.memory_usage_bytes);
}
```

### Adding Entities to Cells

```rust
use astraweave_scene::world_partition::{GridCoord, WorldPartition};
use glam::Vec3;

let mut partition = WorldPartition::new(grid_config);

// Assign entity by position
#[cfg(feature = "ecs")]
partition.assign_entity_to_cell(entity_id, Vec3::new(150.0, 0.0, 250.0));

// Assign entity by bounding box (can span multiple cells)
#[cfg(feature = "ecs")]
{
    use astraweave_scene::world_partition::AABB;
    let bounds = AABB::new(
        Vec3::new(50.0, 0.0, 50.0),
        Vec3::new(250.0, 10.0, 250.0)
    );
    partition.assign_entity_to_cells_by_bounds(entity_id, bounds);
}
```

### Event Handling

```rust
scene.manager.add_event_listener(|event| {
    match event {
        StreamingEvent::CellLoaded(coord) => {
            println!("Cell loaded: {:?}", coord);
        }
        StreamingEvent::CellUnloaded(coord) => {
            println!("Cell unloaded: {:?}", coord);
        }
        StreamingEvent::CellLoadFailed(coord, error) => {
            eprintln!("Cell load failed: {:?} - {}", coord, error);
        }
        _ => {}
    }
});
```

### Frustum Culling

```rust
use astraweave_scene::world_partition::Frustum;
use glam::Mat4;

// Create frustum from view-projection matrix
let view_proj = camera.view_matrix() * camera.projection_matrix();
let frustum = Frustum::from_view_projection(view_proj);

// Get cells in frustum
let camera_pos = Vec3::new(0.0, 50.0, 0.0);
let visible_cells = frustum.cells_in_frustum(camera_pos, 100.0, 500.0);
```

## Configuration

### GridConfig

- `cell_size`: Size of each cell in world units (default: 100.0m)
- `world_bounds`: (min_x, max_x, min_z, max_z) for the world

### StreamingConfig

- `max_active_cells`: Maximum cells to keep loaded (default: 25)
- `lru_cache_size`: Number of cells to cache (default: 5)
- `streaming_radius`: Radius around camera to load (default: 500.0m)
- `max_concurrent_loads`: Max parallel loading tasks (default: 4)

## Performance Considerations

### Memory Usage

The system tracks memory usage through metrics:

```rust
let metrics = scene.metrics();
println!("Memory: {:.2} MB", metrics.memory_usage_bytes as f64 / 1_048_576.0);
```

Target: < 500MB for 10km² world with 100m cells

### Loading Performance

- Cells are loaded asynchronously using tokio
- LRU cache prevents immediate reload of recently unloaded cells
- Concurrent loading is limited to prevent resource exhaustion

### Frame Time

- Streaming updates should complete in < 100ms
- Use metrics to monitor performance:

```rust
let metrics = scene.metrics();
println!("Active: {} | Loading: {} | Cached: {}",
    metrics.active_cells,
    metrics.loading_cells,
    metrics.cached_cells
);
```

## Testing

Run tests with:

```bash
cargo test -p astraweave-scene --features world-partition
```

Run the demo:

```bash
cargo run --example world_partition_demo
```

## Future Enhancements

- [ ] HLOD (Hierarchical Level of Detail) support
- [ ] Data layers for different content types
- [ ] Persistent world state
- [ ] Network replication for multiplayer
- [ ] Editor integration for visual cell management
- [ ] Occlusion culling integration
- [ ] Dynamic cell size based on content density

## References

- UE5 World Partition: https://docs.unrealengine.com/5.0/en-US/world-partition-in-unreal-engine/
- Spatial Partitioning: https://en.wikipedia.org/wiki/Space_partitioning
- Level Streaming: https://docs.unrealengine.com/4.27/en-US/BuildingWorlds/LevelStreaming/