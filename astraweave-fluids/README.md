# astraweave-fluids

A production-grade GPU-accelerated fluid simulation system for the AstraWeave game engine.

## Features

### Core Simulation
- **Position-Based Dynamics (PBD)** solver with adaptive iterations
- **Temperature & Heat Transfer** with Boussinesq buoyancy
- **Multi-Phase Fluids** (water, oil, custom phases)
- **Surface Tension** (Akinci kernel)
- **Vorticity Confinement** for turbulent flow

### Rendering (Screen-Space Fluid Rendering)
- **Depth Smoothing** for smooth surfaces
- **Beer-Lambert Absorption** for volumetric lighting
- **Procedural Caustics** via Voronoi patterns
- **Temporal Reprojection** for anti-aliasing
- **Whitewater & Foam** secondary particles

### Infrastructure
- **Dynamic Particle Management** (spawn/despawn at runtime)
- **Mesh-Based Emitters** (Point, Sphere, Box, Mesh shapes)
- **State Serialization** (save/load with bincode)
- **GPU Frustum Culling** for efficient rendering
- **LOD System** for distance-based optimization
- **Jump Flood Algorithm (JFA)** for SDF generation

## Quick Start

```rust
use astraweave_fluids::{FluidSystem, FluidRenderer, FluidLodManager};

// Create system with 100k particles
let mut fluid_system = FluidSystem::new(&device, 100_000);

// Configure simulation
fluid_system.smoothing_radius = 1.0;
fluid_system.viscosity = 10.0;
fluid_system.surface_tension = 0.02;

// Step simulation each frame
fluid_system.step(&device, &mut encoder, &queue, dt);

// Render
renderer.render(encoder, view, /* ... */);
```

## Modules

| Module | Description |
|--------|-------------|
| `lib.rs` | Core `FluidSystem`, `Particle`, `SimParams` |
| `renderer.rs` | `FluidRenderer` with SSFR pipeline |
| `emitter.rs` | `FluidEmitter`, `FluidDrain` types |
| `lod.rs` | `FluidLodManager` for distance-based LOD |
| `profiling.rs` | `FluidProfiler`, `FluidTimingStats` |
| `serialization.rs` | `FluidSnapshot` for state persistence |
| `sdf.rs` | `SdfSystem` with Jump Flood Algorithm |

## Shaders

| Shader | Purpose |
|--------|---------|
| `fluid.wgsl` | Core PBD solver kernels |
| `ssfr_depth.wgsl` | Particle depth rendering |
| `ssfr_smooth.wgsl` | Bilateral depth smoothing |
| `ssfr_shade.wgsl` | Final shading with SSS |
| `ssfr_temporal.wgsl` | Temporal reprojection |
| `cull.wgsl` | GPU frustum culling |
| `sdf_gen.wgsl` | SDF generation via JFA |
| `secondary.wgsl` | Whitewater billboard rendering |

## Performance Tips

1. **Use LOD** - Enable `FluidLodManager` to skip simulation when far from camera
2. **Adaptive Iterations** - PBD iterations auto-adjust based on density error
3. **Frustum Culling** - `cull.wgsl` reduces draw calls for offscreen particles
4. **Async Compute** - Staging buffers avoid GPU stalls

## Demo

```powershell
cargo run -p fluids_demo
```

Press **SPACE** to switch scenarios. Press **ESC** to exit.

## License

MIT License - See LICENSE file for details.
