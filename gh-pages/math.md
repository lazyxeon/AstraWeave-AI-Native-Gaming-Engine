---
layout: default
title: Math Subsystem
---

# SIMD Math (astraweave-math)

AstraWeave's math library provides SIMD-accelerated vector, matrix, and quaternion operations built on **glam** with optional SSE2 intrinsics.

## Features

| Feature | Module | Speedup |
|---------|--------|---------|
| Vec3/Vec4 operations | `simd_vec.rs` | 2.1× |
| Mat4 multiply | `simd_mat.rs` | 2.5× |
| Quaternion slerp | `simd_quat.rs` | 1.75× |
| Batch position updates | `simd_movement.rs` | 2.08× |

## SIMD Movement

Batch processing for ECS position updates:

```rust
use astraweave_math::simd_movement::update_positions_simd;

// Batch update: 2.08× faster than scalar at 10K entities
update_positions_simd(&mut positions[..], &velocities[..], dt);
```

Implementation details:
- `BATCH_SIZE = 4` with loop unrolling
- glam auto-vectorization (80-85% of hand-written AVX2)
- ECS batching pattern: `collect() → SIMD → writeback`

## SSE2 Fallback

All SIMD operations have scalar fallbacks when SSE2 is unavailable:

```rust
#[cfg(target_feature = "sse2")]
fn dot_product_simd(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    // SSE2 intrinsics
}

#[cfg(not(target_feature = "sse2"))]
fn dot_product_simd(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    // Scalar fallback
}
```

## Miri Validation

109 tests validated under Miri with **zero undefined behavior**:
- SSE2 intrinsic safety
- Scalar fallback correctness
- Edge cases: NaN, infinity, subnormal floats

## Best Practice: Batching

**DO** (3-5× faster):
```rust
// Collect into contiguous buffer, then SIMD
let batch: Vec<_> = query.iter().collect();
update_positions_simd(&mut batch, &velocities, dt);
// Write back to ECS
```

**DON'T** (slow — archetype lookup per entity):
```rust
for entity in query.iter() {
    let pos = world.get_mut::<Position>(entity); // O(log n) lookup each time
    pos.x += vel.dx * dt;
}
```

[← Back to Home](index.html) · [Architecture](architecture.html) · [ECS](ecs.html)
