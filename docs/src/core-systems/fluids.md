# Fluids System

> **Status**: Production Ready (A+ Grade)  
> **Coverage**: 94.2% (2,404 tests)  
> **Crate**: `astraweave-fluids`

AstraWeave's fluid simulation system provides production-quality water effects for games, featuring Position-Based Dynamics (PBD) particle simulation, volumetric grids, and comprehensive visual effects.

## Overview

### Core Systems

| System | Description |
|--------|-------------|
| **Position-Based Dynamics (PBD)** | GPU-accelerated particle simulation |
| **Volumetric Grid** | Voxel-based water for building/terrain interaction |
| **Terrain Integration** | Automatic river, lake, and waterfall detection |
| **SPH Kernels** | Smoothed Particle Hydrodynamics (poly6, spiky, viscosity) |

### Visual Effects

| Effect | Description |
|--------|-------------|
| **Caustics** | Underwater light refraction patterns |
| **God Rays** | Volumetric light shafts through water |
| **Reflections** | Screen-space and planar water reflections |
| **Foam** | Dynamic whitecaps, wakes, and shore foam |
| **Particles** | Waterfalls, bubbles, debris, and spray |

---

## Quick Start

```rust
use astraweave_fluids::{WaterEffectsManager, WaterQualityPreset};

// Create water effects manager with quality preset
let manager = WaterEffectsManager::from_preset(WaterQualityPreset::High)?;

// Game loop
loop {
    // Update water simulation
    manager.update(delta_time, camera_pos, water_height);
    
    // Render water
    manager.render(&render_context);
}
```

---

## Architecture

### Module Overview

```
astraweave-fluids/
├── Core Simulation
│   ├── pcisph_system.rs      # PCISPH pressure solver
│   ├── unified_solver.rs     # Combined solver
│   ├── viscosity.rs          # Viscosity handling
│   ├── boundary.rs           # Domain boundaries
│   └── simd_ops.rs           # SIMD optimizations
├── Visual Effects
│   ├── caustics.rs           # Light caustics
│   ├── god_rays.rs           # Volumetric rays
│   ├── foam.rs               # Foam generation
│   ├── water_reflections.rs  # Reflection system
│   └── underwater.rs         # Underwater effects
├── Terrain Integration
│   ├── terrain_integration.rs
│   ├── volume_grid.rs
│   └── waterfall.rs
└── Editor/Tools
    ├── editor.rs             # Editor integration
    ├── profiling.rs          # Performance stats
    └── serialization.rs      # Save/load
```

---

## SPH Simulation

### Kernel Functions

AstraWeave implements standard SPH kernels:

```rust
// Poly6 kernel for density calculation
fn poly6_kernel(r_sq: f32, h: f32) -> f32 {
    let h2 = h * h;
    if r_sq > h2 { return 0.0; }
    let diff = h2 - r_sq;
    POLY6_FACTOR / h.powi(9) * diff * diff * diff
}

// Spiky kernel for pressure gradient
fn spiky_gradient(r: Vec3, r_len: f32, h: f32) -> Vec3 {
    if r_len > h || r_len < 1e-6 { return Vec3::ZERO; }
    let diff = h - r_len;
    SPIKY_FACTOR / h.powi(6) * diff * diff * (-r / r_len)
}

// Viscosity kernel for velocity diffusion
fn viscosity_laplacian(r_len: f32, h: f32) -> f32 {
    if r_len > h { return 0.0; }
    VISCOSITY_FACTOR / h.powi(3) * (h - r_len)
}
```

### Performance

| Operation | 1K Particles | 10K Particles | 100K Particles |
|-----------|--------------|---------------|----------------|
| Density calc | 5.3 µs | 53 µs | 530 µs |
| Pressure solve | 12 µs | 120 µs | 1.2 ms |
| SPH kernels | — | — | 171-223 µs |
| Full step | 1.8-3.0 ms | — | — |

---

## Volume Grid

For voxel-based water simulation:

```rust
use astraweave_fluids::{VolumeGrid, FlowDirection};

// Create volume grid
let mut grid = VolumeGrid::new(
    dimensions: (64, 32, 64),  // Grid size
    cell_size: 0.5,            // World units per cell
);

// Fill cells
grid.set_water_level(x, y, z, 1.0);

// Simulate flow
grid.simulate_step(delta_time);

// Query water at position
let water_level = grid.sample_at(world_pos);
```

---

## Water Effects Manager

The `WaterEffectsManager` coordinates all water visual effects:

```rust
use astraweave_fluids::{
    WaterEffectsManager,
    WaterQualityPreset,
    CausticsConfig,
    GodRaysConfig,
    FoamConfig,
};

// Create with custom configuration
let manager = WaterEffectsManager::new()
    .with_caustics(CausticsConfig {
        intensity: 0.8,
        scale: 2.0,
        speed: 0.5,
    })
    .with_god_rays(GodRaysConfig {
        density: 0.3,
        decay: 0.95,
        samples: 64,
    })
    .with_foam(FoamConfig {
        threshold: 0.7,
        persistence: 2.0,
    });
```

### Quality Presets

| Preset | Particles | Grid Resolution | Effects |
|--------|-----------|-----------------|---------|
| **Low** | 1,000 | 32³ | Basic reflections |
| **Medium** | 5,000 | 64³ | + Caustics |
| **High** | 10,000 | 128³ | + God rays, foam |
| **Ultra** | 25,000 | 256³ | All effects |

---

## Terrain Integration

Automatic water body detection:

```rust
use astraweave_fluids::{TerrainWaterIntegration, WaterBodyType};

let integration = TerrainWaterIntegration::new(&terrain);

// Detect water bodies
let water_bodies = integration.detect_water_bodies();

for body in water_bodies {
    match body.body_type {
        WaterBodyType::Lake { depth, .. } => {
            println!("Lake detected: depth = {}", depth);
        }
        WaterBodyType::River { flow_speed, .. } => {
            println!("River detected: flow = {}", flow_speed);
        }
        WaterBodyType::Waterfall { height, .. } => {
            println!("Waterfall detected: height = {}", height);
        }
    }
}
```

---

## Building Integration

Interactive water elements for building systems:

```rust
use astraweave_fluids::{
    WaterBuildingManager,
    WaterDispenser,
    WaterDrain,
    WaterGate,
    WaterWheel,
};

let mut water_manager = WaterBuildingManager::new(&volume_grid);

// Water source
let dispenser = WaterDispenser::new(position, flow_rate: 5.0);
water_manager.add_dispenser(dispenser);

// Water sink
let drain = WaterDrain::new(position, capacity: 10.0);
water_manager.add_drain(drain);

// Player-controlled gate
let gate = WaterGate::new(position, width: 2.0);
water_manager.add_gate(gate);

// Power generator
let wheel = WaterWheel::new(position, radius: 1.5);
water_manager.add_wheel(wheel);

// Query power generation
let power = water_manager.get_wheel_power(&wheel);
```

---

## Editor Integration

Full editor support with undo/redo:

```rust
use astraweave_fluids::editor::{
    FluidEditorConfig,
    ConfigHistory,
    ValidationIssue,
};

let mut config = FluidEditorConfig::default();
let mut history = ConfigHistory::new();

// Make changes with history tracking
history.push(config.clone());
config.physics.viscosity = 0.5;

// Undo
if let Some(prev) = history.undo() {
    config = prev;
}

// Validate configuration
let issues: Vec<ValidationIssue> = config.validate();
for issue in issues {
    eprintln!("{}: {}", issue.severity, issue.message);
}
```

---

## Serialization

Save and load fluid state:

```rust
use astraweave_fluids::serialization::{FluidSnapshot, save_snapshot, load_snapshot};

// Save current state
let snapshot = FluidSnapshot::capture(&simulation);
save_snapshot(&snapshot, "saves/water_state.bin")?;

// Load state
let loaded = load_snapshot("saves/water_state.bin")?;
simulation.restore(&loaded);
```

---

## Performance Optimization

### SIMD Operations

The fluids system uses SIMD for critical operations:

```rust
use astraweave_fluids::simd_ops;

// Batch particle updates (SIMD accelerated)
simd_ops::update_positions_batch(&mut particles, delta_time);
simd_ops::compute_densities_batch(&particles, &neighbors, kernel_h);
```

### LOD System

Distance-based quality reduction:

```rust
use astraweave_fluids::lod::{FluidLodConfig, LodLevel};

let lod_config = FluidLodConfig {
    levels: vec![
        LodLevel { distance: 0.0, particle_skip: 1 },   // Full detail
        LodLevel { distance: 50.0, particle_skip: 2 },  // 50% particles
        LodLevel { distance: 100.0, particle_skip: 4 }, // 25% particles
    ],
    cull_distance: 200.0,
};
```

### Profiling

Built-in performance monitoring:

```rust
use astraweave_fluids::profiling::FluidProfiler;

let profiler = FluidProfiler::new();

// Wrap simulation step
profiler.begin("simulation_step");
simulation.step(dt);
profiler.end("simulation_step");

// Get statistics
let stats = profiler.get_stats();
println!("Avg step time: {:?}", stats.avg_step_time);
println!("Particle count: {}", stats.active_particles);
```

---

## Benchmarks

From [Master Benchmark Report](../performance/benchmarks.md):

| Benchmark | Result | Notes |
|-----------|--------|-------|
| Particle operations (1K) | 5.3 µs | 100-322 Melem/s |
| Spatial hashing | 163 µs - 5.6 ms | 38-62% improvement |
| SPH kernels (100K) | 171-223 µs | poly6/spiky/viscosity |
| Simulation step (1K) | 1.8-3.0 ms | — |
| Multi-step | 450-500 µs | 45-57% faster |
| GPU data prep | 0.9-2.6 ns | Sub-nanosecond |

---

## Examples

### Basic Water Pool

```bash
cargo run --example fluids_demo -- --preset pool
```

### Interactive Waterfall

```bash
cargo run --example fluids_demo -- --preset waterfall
```

### Building Water System

```bash
cargo run --example fluids_demo -- --preset building
```

---

## See Also

- [API Reference](../api/fluids.md) - Detailed API documentation
- [Benchmarks](../performance/benchmarks.md) - Performance data
- [Physics System](./physics.md) - Physics integration
- [Terrain System](./terrain.md) - Terrain integration
