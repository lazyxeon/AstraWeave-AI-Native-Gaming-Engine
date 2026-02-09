# astraweave-math

SIMD-accelerated math operations for AstraWeave.

## Overview

Platform-optimized vector, matrix, quaternion, and batch movement operations built on **glam**, with SSE2 baseline and scalar fallback.

## Modules

| Module | Description |
|--------|-------------|
| `simd_vec` | Vector ops: `dot_simd`, `cross_simd`, `normalize_simd`, `length_simd` |
| `simd_mat` | Matrix ops: `mul_simd`, `transpose_simd`, `inverse_simd`, `transform_points_batch` |
| `simd_quat` | Quaternion ops: `mul_quat_simd`, `slerp_simd`, `slerp_batch`, `normalize_batch` |
| `simd_movement` | Batch position updates (2.08× speedup vs scalar) |

## Performance

| Operation | Speedup vs scalar |
|-----------|-------------------|
| Vec3 dot | 2.1× |
| Mat4 mul | 2.5× |
| Quat slerp | 1.75× |
| Batch movement | 2.08× |

## Feature Flags

| Feature | Description |
|---------|-------------|
| `simd` | Platform-specific SIMD paths (SSE2/AVX2) |

## Usage

```rust
use astraweave_math::simd_movement::update_positions_simd;

update_positions_simd(&mut positions, &velocities, dt);
```

## License

MIT
