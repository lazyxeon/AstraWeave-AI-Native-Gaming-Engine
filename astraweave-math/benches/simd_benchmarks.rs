/*!
# SIMD Math Benchmarks

Benchmarks comparing SIMD vs scalar math operations.

Run with:
```bash
cargo bench -p astraweave-math --bench simd_benchmarks
```

Quick mode:
```bash
cargo bench -p astraweave-math --bench simd_benchmarks -- --quick
```
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use glam::Vec3;
use astraweave_math::simd_vec::{dot_simd, cross_simd, normalize_simd, length_simd};

// ============================================================================
// Vec3 Dot Product Benchmarks
// ============================================================================

fn bench_dot_scalar(c: &mut Criterion) {
    let a = Vec3::new(1.5, 2.5, 3.5);
    let b = Vec3::new(4.5, 5.5, 6.5);

    c.bench_function("vec3_dot/scalar", |bencher| {
        bencher.iter(|| {
            let result = black_box(a).dot(black_box(b));
            black_box(result)
        })
    });
}

fn bench_dot_simd(c: &mut Criterion) {
    let a = Vec3::new(1.5, 2.5, 3.5);
    let b = Vec3::new(4.5, 5.5, 6.5);

    c.bench_function("vec3_dot/simd", |bencher| {
        bencher.iter(|| {
            let result = dot_simd(black_box(a), black_box(b));
            black_box(result)
        })
    });
}

fn bench_dot_throughput(c: &mut Criterion) {
    let vectors_a: Vec<Vec3> = (0..1000)
        .map(|i| Vec3::new(i as f32, (i + 1) as f32, (i + 2) as f32))
        .collect();
    let vectors_b: Vec<Vec3> = (0..1000)
        .map(|i| Vec3::new((i + 3) as f32, (i + 4) as f32, (i + 5) as f32))
        .collect();

    let mut group = c.benchmark_group("vec3_dot_throughput");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("scalar", |bencher| {
        bencher.iter(|| {
            let mut sum = 0.0;
            for (&a, &b) in vectors_a.iter().zip(vectors_b.iter()) {
                sum += black_box(a).dot(black_box(b));
            }
            black_box(sum)
        })
    });

    group.bench_function("simd", |bencher| {
        bencher.iter(|| {
            let mut sum = 0.0;
            for (&a, &b) in vectors_a.iter().zip(vectors_b.iter()) {
                sum += dot_simd(black_box(a), black_box(b));
            }
            black_box(sum)
        })
    });

    group.finish();
}

// ============================================================================
// Vec3 Cross Product Benchmarks
// ============================================================================

fn bench_cross_scalar(c: &mut Criterion) {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    c.bench_function("vec3_cross/scalar", |bencher| {
        bencher.iter(|| {
            let result = black_box(a).cross(black_box(b));
            black_box(result)
        })
    });
}

fn bench_cross_simd(c: &mut Criterion) {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    c.bench_function("vec3_cross/simd", |bencher| {
        bencher.iter(|| {
            let result = cross_simd(black_box(a), black_box(b));
            black_box(result)
        })
    });
}

// ============================================================================
// Vec3 Normalize Benchmarks
// ============================================================================

fn bench_normalize_scalar(c: &mut Criterion) {
    let v = Vec3::new(3.5, 4.5, 5.5);

    c.bench_function("vec3_normalize/scalar", |bencher| {
        bencher.iter(|| {
            let result = black_box(v).normalize();
            black_box(result)
        })
    });
}

fn bench_normalize_simd(c: &mut Criterion) {
    let v = Vec3::new(3.5, 4.5, 5.5);

    c.bench_function("vec3_normalize/simd", |bencher| {
        bencher.iter(|| {
            let result = normalize_simd(black_box(v));
            black_box(result)
        })
    });
}

// ============================================================================
// Vec3 Length Benchmarks
// ============================================================================

fn bench_length_scalar(c: &mut Criterion) {
    let v = Vec3::new(3.0, 4.0, 5.0);

    c.bench_function("vec3_length/scalar", |bencher| {
        bencher.iter(|| {
            let result = black_box(v).length();
            black_box(result)
        })
    });
}

fn bench_length_simd(c: &mut Criterion) {
    let v = Vec3::new(3.0, 4.0, 5.0);

    c.bench_function("vec3_length/simd", |bencher| {
        bencher.iter(|| {
            let result = length_simd(black_box(v));
            black_box(result)
        })
    });
}

// ============================================================================
// Integrated Benchmarks (Realistic Physics/Rendering Scenarios)
// ============================================================================

fn bench_physics_tick_scalar(c: &mut Criterion) {
    // Simulate physics tick: 100 rigid bodies with forces/torques
    let positions: Vec<Vec3> = (0..100)
        .map(|i| Vec3::new(i as f32, (i + 1) as f32, (i + 2) as f32))
        .collect();
    let velocities: Vec<Vec3> = (0..100)
        .map(|i| Vec3::new((i + 3) as f32 * 0.1, (i + 4) as f32 * 0.1, (i + 5) as f32 * 0.1))
        .collect();
    let forces: Vec<Vec3> = (0..100)
        .map(|i| Vec3::new((i + 6) as f32 * 0.01, (i + 7) as f32 * 0.01, (i + 8) as f32 * 0.01))
        .collect();

    c.bench_function("physics_tick/scalar", |bencher| {
        bencher.iter(|| {
            let mut total_energy = 0.0;
            for i in 0..100 {
                let pos = black_box(positions[i]);
                let vel = black_box(velocities[i]);
                let force = black_box(forces[i]);

                // Normalize direction
                let dir = pos.normalize();
                
                // Kinetic energy: 0.5 * m * v² (using dot product)
                let kinetic = 0.5 * vel.dot(vel);

                // Torque (cross product)
                let torque = pos.cross(force);
                let angular_energy = torque.length();

                total_energy += kinetic + angular_energy + dir.length();
            }
            black_box(total_energy)
        })
    });
}

fn bench_physics_tick_simd(c: &mut Criterion) {
    // Same setup as scalar version
    let positions: Vec<Vec3> = (0..100)
        .map(|i| Vec3::new(i as f32, (i + 1) as f32, (i + 2) as f32))
        .collect();
    let velocities: Vec<Vec3> = (0..100)
        .map(|i| Vec3::new((i + 3) as f32 * 0.1, (i + 4) as f32 * 0.1, (i + 5) as f32 * 0.1))
        .collect();
    let forces: Vec<Vec3> = (0..100)
        .map(|i| Vec3::new((i + 6) as f32 * 0.01, (i + 7) as f32 * 0.01, (i + 8) as f32 * 0.01))
        .collect();

    c.bench_function("physics_tick/simd", |bencher| {
        bencher.iter(|| {
            let mut total_energy = 0.0;
            for i in 0..100 {
                let pos = black_box(positions[i]);
                let vel = black_box(velocities[i]);
                let force = black_box(forces[i]);

                // Normalize direction (SIMD)
                let dir = normalize_simd(pos);
                
                // Kinetic energy: 0.5 * m * v² (SIMD dot)
                let kinetic = 0.5 * dot_simd(vel, vel);

                // Torque (SIMD cross)
                let torque = cross_simd(pos, force);
                let angular_energy = length_simd(torque);

                total_energy += kinetic + angular_energy + length_simd(dir);
            }
            black_box(total_energy)
        })
    });
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    vec3_dot,
    bench_dot_scalar,
    bench_dot_simd,
    bench_dot_throughput
);

criterion_group!(
    vec3_cross,
    bench_cross_scalar,
    bench_cross_simd
);

criterion_group!(
    vec3_normalize,
    bench_normalize_scalar,
    bench_normalize_simd
);

criterion_group!(
    vec3_length,
    bench_length_scalar,
    bench_length_simd
);

criterion_group!(
    integrated,
    bench_physics_tick_scalar,
    bench_physics_tick_simd
);

criterion_main!(
    vec3_dot,
    vec3_cross,
    vec3_normalize,
    vec3_length,
    integrated
);
