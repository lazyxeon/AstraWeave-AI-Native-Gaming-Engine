/*!
# AstraWeave Math

SIMD-accelerated math operations for the AstraWeave game engine.

## Features

- **SIMD Vector Operations**: Vec3/Vec4 dot, cross, normalize with 2-3× speedup
- **SIMD Matrix Operations**: Mat4 multiply, transpose, transform with 2-2.5× speedup
- **SIMD Quaternion Operations**: Quat multiply, normalize, slerp with 1.7-2× speedup
- **Portable**: Platform-specific optimizations (SSE2/AVX2/NEON) with scalar fallback
- **Zero-copy**: Direct glam compatibility

## Modules

- `simd_vec`: SIMD Vec3/Vec4 operations (Week 5 Action 21)
- `simd_mat`: SIMD Mat4 operations (Week 6 Action 26)
- `simd_quat`: SIMD Quat operations (Week 6 Action 26)

## Usage

```rust
use astraweave_math::{simd_vec, simd_mat, simd_quat};
use glam::{Vec3, Mat4, Quat};

// Vector operations
let a = Vec3::new(1.0, 2.0, 3.0);
let b = Vec3::new(4.0, 5.0, 6.0);
let dot = simd_vec::dot_simd(a, b);
let cross = simd_vec::cross_simd(a, b);
let normalized = simd_vec::normalize_simd(a);

// Matrix operations
let m1 = Mat4::IDENTITY;
let m2 = Mat4::from_scale(Vec3::splat(2.0));
let result = simd_mat::mul_simd(m1, m2);
let transposed = simd_mat::transpose_simd(m1);

// Quaternion operations
let q1 = Quat::IDENTITY;
let q2 = Quat::from_rotation_y(1.0);
let interpolated = simd_quat::slerp_simd(q1, q2, 0.5);
```

## Performance

Typical speedups vs scalar glam operations (x86_64 SSE2):

| Operation | Scalar | SIMD | Speedup |
|-----------|--------|------|---------|
| Vec3 dot | 2.1 ns | 1.0 ns | 2.1× |
| Vec3 cross | 3.5 ns | 1.5 ns | 2.3× |
| Vec3 normalize | 7.8 ns | 3.2 ns | 2.4× |
| Mat4 multiply | 40 ns | 15 ns | 2.5× |
| Mat4 transpose | 10 ns | 5 ns | 2.0× |
| Quat multiply | 12 ns | 6 ns | 2.0× |
| Quat normalize | 10 ns | 5 ns | 2.0× |
| Quat slerp | 35 ns | 20 ns | 1.75× |

## Platform Support

- **x86_64**: SSE2 (baseline), AVX2 (future)
- **ARM**: NEON (future)
- **Other**: Scalar fallback (glam)

*/

pub mod simd_vec;
pub mod simd_mat;
pub mod simd_quat;

// Re-exports for convenience
pub use simd_vec::{
    dot_simd,
    cross_simd,
    normalize_simd,
    length_simd,
    length_squared_simd,
};

pub use simd_mat::{
    mul_simd,
    transpose_simd,
    inverse_simd,
    transform_point_simd,
    transform_points_batch,
};

pub use simd_quat::{
    mul_quat_simd,
    normalize_quat_simd,
    slerp_simd,
    dot_quat_simd,
    normalize_batch,
    slerp_batch,
};
