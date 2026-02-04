# Fluids API Reference

> **Crate**: `astraweave-fluids`  
> **Coverage**: ~91%  
> **Tests**: 2,404

Production-ready SPH fluid simulation with real-time rendering, terrain integration, and editor support.

## Quick Links

- [rustdoc](https://docs.rs/astraweave-fluids) (when published)
- [Source](https://github.com/astraweave/astraweave/tree/main/astraweave-fluids)
- [Fluids System Guide](../core-systems/fluids.md)

---

## Core Types

### WaterEffectsManager

Central manager for all water visual effects.

```rust
use astraweave_fluids::{WaterEffectsManager, WaterQualityPreset};

let mut manager = WaterEffectsManager::new(WaterQualityPreset::High);

// Configure effects
manager.enable_caustics(true);
manager.enable_god_rays(true);
manager.enable_foam(true);

// Update each frame
manager.update(delta_time, &water_state);

// Render
manager.render(&mut render_context);
```

---

### WaterQualityPreset

Quality presets for different hardware targets.

```rust
use astraweave_fluids::WaterQualityPreset;

// Available presets
let ultra = WaterQualityPreset::Ultra;    // 64 rays, 1024 particles
let high = WaterQualityPreset::High;      // 32 rays, 512 particles
let medium = WaterQualityPreset::Medium;  // 16 rays, 256 particles
let low = WaterQualityPreset::Low;        // 8 rays, 128 particles

// Apply preset
let manager = WaterEffectsManager::new(high);
```

---

### PcisphSystem

Predictive-Corrective Incompressible SPH solver.

```rust
use astraweave_fluids::pcisph_system::{PcisphSystem, PcisphConfig};

let config = PcisphConfig {
    particle_count: 10000,
    particle_radius: 0.1,
    rest_density: 1000.0,
    viscosity: 0.01,
    surface_tension: 0.0728,
    ..Default::default()
};

let mut solver = PcisphSystem::new(config);

// Add particles
solver.spawn_particles(position, count);

// Step simulation
solver.step(delta_time);

// Get particle data for rendering
let positions = solver.positions();
let velocities = solver.velocities();
```

---

### UnifiedSolver

Unified particle solver handling fluids, foam, and spray.

```rust
use astraweave_fluids::unified_solver::{UnifiedSolver, ParticleType};

let mut solver = UnifiedSolver::new(config);

// Spawn different particle types
solver.spawn(ParticleType::Water, position, count);
solver.spawn(ParticleType::Foam, position, count);
solver.spawn(ParticleType::Spray, position, count);

// Step all particles together
solver.step(delta_time);
```

---

## Visual Effects

### Caustics

Underwater light caustic patterns.

```rust
use astraweave_fluids::caustics::{CausticRenderer, CausticConfig};

let config = CausticConfig {
    resolution: 512,
    intensity: 1.0,
    speed: 0.5,
    scale: 10.0,
    ..Default::default()
};

let mut caustics = CausticRenderer::new(config);
caustics.update(time, water_surface);

// Apply to underwater objects
caustics.apply_to_surface(&mut material);
```

---

### God Rays

Volumetric underwater light shafts.

```rust
use astraweave_fluids::god_rays::{GodRayRenderer, GodRayConfig};

let config = GodRayConfig {
    ray_count: 32,
    ray_length: 50.0,
    intensity: 0.8,
    decay: 0.95,
    ..Default::default()
};

let mut god_rays = GodRayRenderer::new(config);
god_rays.render(sun_direction, water_surface);
```

---

### Foam

Surface foam simulation.

```rust
use astraweave_fluids::foam::{FoamSystem, FoamConfig};

let config = FoamConfig {
    spawn_threshold: 2.0,    // Velocity threshold
    lifetime: 3.0,           // Seconds
    size: 0.1,
    ..Default::default()
};

let mut foam = FoamSystem::new(config);
foam.update(delta_time, &water_particles);
foam.render(&mut render_context);
```

---

## Terrain Integration

### WaterTerrainIntegration

Seamless water-terrain interaction.

```rust
use astraweave_fluids::terrain_integration::{WaterTerrainIntegration, WaterBodyConfig};

let integration = WaterTerrainIntegration::new();

// Define water body
let lake = WaterBodyConfig {
    center: Vec3::new(100.0, 50.0, 100.0),
    radius: 50.0,
    depth: 10.0,
    flow_direction: Vec3::ZERO,
    ..Default::default()
};

// Query water at position
let water_info = integration.sample_water(position, &terrain);
if water_info.is_submerged {
    apply_buoyancy(water_info.depth);
}
```

---

### Building Integration

Water interaction with structures.

```rust
use astraweave_fluids::building::{WaterBuildingIntegration, FloodConfig};

let mut building_water = WaterBuildingIntegration::new();

// Configure flooding
building_water.set_flood_config(FloodConfig {
    water_level: 45.0,
    rise_rate: 0.1,
    ..Default::default()
});

// Update with building geometry
building_water.update(&buildings, delta_time);

// Query flood state
let flood = building_water.get_flood_level(building_id);
```

---

## Volume Grid

### GpuVolumeGrid

GPU-accelerated 3D volume grid for water simulation.

```rust
use astraweave_fluids::gpu_volume::{GpuVolumeGrid, VolumeConfig};

let config = VolumeConfig {
    resolution: [128, 64, 128],
    cell_size: 0.5,
    ..Default::default()
};

let mut volume = GpuVolumeGrid::new(&device, config);

// Update from particles
volume.splat_particles(&particles);

// Read density
let density = volume.sample(position);
```

---

## Editor Support

### Undo/Redo

Full undo/redo support for editor integration.

```rust
use astraweave_fluids::editor::{WaterEditor, EditorCommand};

let mut editor = WaterEditor::new();

// Make changes
editor.execute(EditorCommand::SetWaterLevel(50.0));
editor.execute(EditorCommand::AddEmitter(emitter));

// Undo
editor.undo();

// Redo
editor.redo();
```

---

### Validation

Real-time validation of water configurations.

```rust
use astraweave_fluids::editor::validation::{validate_water_body, ValidationResult};

let result = validate_water_body(&config);
match result {
    ValidationResult::Valid => { /* OK */ }
    ValidationResult::Warning(msg) => {
        ui.show_warning(&msg);
    }
    ValidationResult::Error(msg) => {
        ui.show_error(&msg);
        return;
    }
}
```

---

## Serialization

### Save/Load

Full water state serialization.

```rust
use astraweave_fluids::serialization::{save_water_state, load_water_state};

// Save
let data = save_water_state(&manager)?;
std::fs::write("water.bin", &data)?;

// Load
let data = std::fs::read("water.bin")?;
let manager = load_water_state(&data)?;
```

---

## LOD System

### Adaptive Quality

Distance-based quality adjustment.

```rust
use astraweave_fluids::lod::{WaterLod, LodConfig};

let lod = WaterLod::new(LodConfig {
    lod0_distance: 50.0,   // Full quality
    lod1_distance: 100.0,  // Medium
    lod2_distance: 200.0,  // Low
    ..Default::default()
});

// Update based on camera
lod.update(camera_position);

// Get quality for water body
let quality = lod.get_quality(water_position);
```

---

## Performance

| Operation | Latency | Notes |
|-----------|---------|-------|
| PCISPH step (10K particles) | ~2 ms | GPU accelerated |
| Caustic update | ~0.5 ms | 512×512 texture |
| God ray render | ~0.8 ms | 32 rays |
| Foam update | ~0.3 ms | 1K particles |
| Volume splat | ~0.2 ms | 128³ grid |

---

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `gpu` | GPU acceleration | ✅ |
| `editor` | Editor integration | ❌ |
| `serialize` | State serialization | ✅ |
| `simd` | SIMD optimization | ✅ |

---

## See Also

- [Fluids System Guide](../core-systems/fluids.md)
- [Terrain Integration](../core-systems/terrain.md#water-integration)
- [Physics System](./physics.md)
- [Performance Benchmarks](../performance/benchmarks.md)
