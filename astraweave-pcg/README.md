# astraweave-pcg

**Procedural Content Generation (PCG)** with deterministic seed reproducibility for AstraWeave.

## Overview

This crate provides seed-based procedural generation with the following guarantees:
- **Deterministic**: Same seed → same output, every time
- **Layered**: RNG can be forked into independent sublayers with derived seeds
- **Constraint-based**: Generation respects bounds, spacing, connectivity requirements

## Core Components

### SeedRng

Deterministic random number generator wrapper around `StdRng` with explicit seed tracking per layer.

```rust
use astraweave_pcg::SeedRng;

// Create RNG for a specific layer
let mut rng = SeedRng::new(42, "world_gen");

// Fork into sublayer (deterministic derived seed)
let mut encounter_rng = rng.fork("encounters");
let mut layout_rng = rng.fork("layout");

// Each sublayer has independent but reproducible sequences
```

**API**:
- `new(seed: u64, layer: &str)` - Create RNG with explicit seed
- `fork(&mut self, sublayer: &str)` - Derive child RNG deterministically
- `gen_range<T>(range)` - Generate value in range
- `choose<T>(slice)` - Pick random element
- `shuffle<T>(slice)` - Shuffle in place
- `gen_bool()`, `gen_f32()`, `gen_f64()` - Random primitives

### EncounterGenerator

Generates encounters (combat, loot, ambient events) with constraint satisfaction.

```rust
use astraweave_pcg::{EncounterGenerator, EncounterConstraints, SeedRng};
use glam::IVec2;

let constraints = EncounterConstraints {
    bounds: (IVec2::ZERO, IVec2::new(100, 100)),
    min_spacing: 10.0,
    difficulty_range: (1.0, 5.0),
};

let gen = EncounterGenerator::new(constraints);
let mut rng = SeedRng::new(42, "encounters");

let encounters = gen.generate(&mut rng, 10);
// All encounters satisfy spacing, bounds, difficulty constraints
```

**Constraints**:
- `bounds`: Min/max position rectangle
- `min_spacing`: Minimum distance between encounters (prevents clustering)
- `difficulty_range`: Min/max difficulty values

**Encounter Types**:
- `Combat { enemy_types, count }` - Enemy spawns
- `Loot { items }` - Item drops
- `Ambient { event_id }` - NPCs, events, etc.

### LayoutGenerator

Generates rooms with guaranteed connectivity (all rooms reachable).

```rust
use astraweave_pcg::{LayoutGenerator, SeedRng};
use glam::IVec2;

let gen = LayoutGenerator::new(IVec2::new(100, 100));
let mut rng = SeedRng::new(42, "layout");

let rooms = gen.generate_rooms(&mut rng, 10);
// All rooms are connected via graph traversal (BFS reachable)
```

**Features**:
- No room overlaps
- All rooms within grid bounds
- Chain + random connections (ensures connectivity + cycles)
- Configurable room size ranges

## Seed Policy

### Layer Naming Convention

Use descriptive hierarchical layer names:
```rust
let mut world_rng = SeedRng::new(seed, "world");
let mut biome_rng = world_rng.fork("biome");
let mut terrain_rng = biome_rng.fork("terrain");
let mut encounter_rng = world_rng.fork("encounters");
```

**Benefits**:
- Layer tracking for debugging (`rng.layer()` returns `"world::biome::terrain"`)
- Independent sublayers with derived seeds
- Reproducible on replay with same root seed

### Reproducibility Guarantees

**Same seed + same layer + same calls → same results**, guaranteed by:
1. **Deterministic RNG**: `StdRng` (rand 0.9 stable)
2. **Deterministic iteration**: `BTreeMap`/`BTreeSet` for stable ordering
3. **Explicit seeding**: No implicit global RNG, all seeds explicit
4. **Fork determinism**: Child seeds derived via `random::<u64>()` on parent

### Best Practices

1. **Use fixed seeds for tests**: `SeedRng::new(42, "test")`
2. **Fork for independent systems**: `terrain_rng`, `encounter_rng`, `loot_rng`
3. **Document seed sources**: Where does the root seed come from? (save file, server, config)
4. **Test reproducibility**: Same seed → same output in integration tests

## Testing

All generation is tested for determinism and constraint satisfaction:

```bash
cargo test -p astraweave-pcg
```

**Test Coverage**:
- SeedRng: 8 tests (determinism, forking, choosing, shuffling)
- Encounters: 4 tests (determinism, spacing, bounds, difficulty)
- Layout: 7 tests (determinism, overlaps, bounds, connectivity)

**Total**: 19/19 tests passing (100%)

## Performance Notes

- **StdRng**: Fast, cryptographically secure (PCG algorithm)
- **Constraint solving**: Placement uses rejection sampling (max attempts = count × 10)
- **Room connectivity**: Linear-time graph construction + BFS validation

For large-scale generation (10K+ entities), consider:
1. Spatial partitioning (chunk-based generation)
2. Caching generated content (don't regenerate on every frame)
3. Profiling with `criterion` benchmarks

## Integration Example

```rust
use astraweave_pcg::{SeedRng, EncounterGenerator, EncounterConstraints};
use glam::IVec2;

fn generate_world(seed: u64) {
    let mut rng = SeedRng::new(seed, "world");
    
    // Generate encounters
    let constraints = EncounterConstraints {
        bounds: (IVec2::ZERO, IVec2::new(200, 200)),
        min_spacing: 15.0,
        difficulty_range: (1.0, 10.0),
    };
    
    let gen = EncounterGenerator::new(constraints);
    let encounters = gen.generate(&mut rng.fork("encounters"), 50);
    
    // Spawn in ECS
    for enc in encounters {
        // spawn_encounter(world, enc);
    }
}
```

## Feature Flags

- `pcg` - Enable all PCG features (currently no effect, always enabled)

## Dependencies

- `rand` (0.9) - Core RNG traits and `StdRng`
- `glam` - Math types (`IVec2` for positions)
- `serde` - Serialization for generated content

## License

MIT OR Apache-2.0
