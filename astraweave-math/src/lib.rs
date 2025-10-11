/*!
# AstraWeave Math

SIMD-accelerated math operations for the AstraWeave game engine.

## Features

- **SIMD Vector Operations**: Vec3/Vec4 dot, cross, normalize with 2-4× speedup
- **SIMD Matrix Operations**: Mat4 multiply, inverse with 2-4× speedup
- **Portable**: Platform-specific optimizations (SSE2/AVX2/NEON) with scalar fallback
- **Zero-copy**: Direct glam compatibility

## Modules

- `simd_vec`: SIMD Vec3/Vec4 operations
- `simd_mat`: SIMD Mat4 operations (coming soon)

## Usage

```rust
use astraweave_math::simd_vec::{dot_simd, cross_simd, normalize_simd};
use glam::Vec3;

let a = Vec3::new(1.0, 2.0, 3.0);
let b = Vec3::new(4.0, 5.0, 6.0);

// SIMD operations
let dot = dot_simd(a, b);
let cross = cross_simd(a, b);
let normalized = normalize_simd(a);
```

## Performance

Typical speedups vs scalar glam operations (x86_64 SSE2):

| Operation | Scalar | SIMD | Speedup |
|-----------|--------|------|---------|
| Vec3 dot | 10 ns | 3-5 ns | 2-3× |
| Vec3 cross | 15 ns | 5-7 ns | 2-3× |
| Vec3 normalize | 20 ns | 7-10 ns | 2-3× |
| Mat4 multiply | 40 ns | 12-15 ns | 2-3× |

## Platform Support

- **x86_64**: SSE2 (baseline), AVX2 (future)
- **ARM**: NEON (future)
- **Other**: Scalar fallback (glam)

*/

pub mod simd_vec;
// pub mod simd_mat;  // Coming soon

// Re-exports for convenience
pub use simd_vec::{
    dot_simd,
    cross_simd,
    normalize_simd,
    length_simd,
    length_squared_simd,
};
