use astraweave_math::simd_quat::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::Quat;

fn bench_quat_multiply(c: &mut Criterion) {
    let q1 = Quat::from_rotation_x(1.0);
    let q2 = Quat::from_rotation_y(1.5);

    c.bench_function("quat_multiply_scalar", |bencher| {
        bencher.iter(|| black_box(q1 * q2));
    });

    c.bench_function("quat_multiply_simd", |bencher| {
        bencher.iter(|| black_box(mul_quat_simd(q1, q2)));
    });
}

fn bench_quat_normalize(c: &mut Criterion) {
    let q = Quat::from_xyzw(1.0, 2.0, 3.0, 4.0);

    c.bench_function("quat_normalize_scalar", |bencher| {
        bencher.iter(|| black_box(q.normalize()));
    });

    c.bench_function("quat_normalize_simd", |bencher| {
        bencher.iter(|| black_box(normalize_quat_simd(q)));
    });
}

fn bench_quat_slerp(c: &mut Criterion) {
    let q1 = Quat::IDENTITY;
    let q2 = Quat::from_rotation_y(std::f32::consts::PI);

    c.bench_function("quat_slerp_scalar", |bencher| {
        bencher.iter(|| black_box(q1.slerp(q2, 0.5)));
    });

    c.bench_function("quat_slerp_simd", |bencher| {
        bencher.iter(|| black_box(slerp_simd(q1, q2, 0.5)));
    });
}

fn bench_quat_dot(c: &mut Criterion) {
    let q1 = Quat::from_rotation_x(1.0);
    let q2 = Quat::from_rotation_y(1.0);

    c.bench_function("quat_dot_scalar", |bencher| {
        bencher.iter(|| black_box(q1.dot(q2)));
    });

    c.bench_function("quat_dot_simd", |bencher| {
        bencher.iter(|| black_box(dot_quat_simd(q1, q2)));
    });
}

fn bench_quat_normalize_batch(c: &mut Criterion) {
    let quats: Vec<Quat> = (0..16)
        .map(|i| Quat::from_rotation_y(i as f32 * 0.1))
        .collect();

    c.bench_function("quat_normalize_batch_scalar", |bencher| {
        bencher.iter(|| {
            let result: Vec<Quat> = quats.iter().map(|&q| q.normalize()).collect();
            black_box(result)
        });
    });

    c.bench_function("quat_normalize_batch_simd", |bencher| {
        bencher.iter(|| black_box(normalize_batch(&quats)));
    });
}

fn bench_quat_slerp_batch(c: &mut Criterion) {
    let pairs: Vec<(Quat, Quat)> = (0..16)
        .map(|i| {
            let angle = i as f32 * 0.1;
            (Quat::IDENTITY, Quat::from_rotation_y(angle))
        })
        .collect();

    c.bench_function("quat_slerp_batch_scalar", |bencher| {
        bencher.iter(|| {
            let result: Vec<Quat> = pairs.iter().map(|&(a, b)| a.slerp(b, 0.5)).collect();
            black_box(result)
        });
    });

    c.bench_function("quat_slerp_batch_simd", |bencher| {
        bencher.iter(|| black_box(slerp_batch(&pairs, 0.5)));
    });
}

criterion_group!(
    benches,
    bench_quat_multiply,
    bench_quat_normalize,
    bench_quat_slerp,
    bench_quat_dot,
    bench_quat_normalize_batch,
    bench_quat_slerp_batch,
);
criterion_main!(benches);
