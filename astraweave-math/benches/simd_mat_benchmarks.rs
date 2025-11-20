use astraweave_math::simd_mat::*;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use glam::{Mat4, Vec3};

fn bench_mat4_multiply(c: &mut Criterion) {
    let a = Mat4::from_scale(Vec3::splat(2.0));
    let b = Mat4::from_rotation_y(1.0);

    c.bench_function("mat4_multiply_scalar", |bencher| {
        bencher.iter(|| black_box(a * b));
    });

    c.bench_function("mat4_multiply_simd", |bencher| {
        bencher.iter(|| black_box(mul_simd(a, b)));
    });
}

fn bench_mat4_transpose(c: &mut Criterion) {
    let m = Mat4::from_cols_array(&[
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
    ]);

    c.bench_function("mat4_transpose_scalar", |bencher| {
        bencher.iter(|| black_box(m.transpose()));
    });

    c.bench_function("mat4_transpose_simd", |bencher| {
        bencher.iter(|| black_box(transpose_simd(m)));
    });
}

fn bench_mat4_inverse(c: &mut Criterion) {
    let m = Mat4::from_scale_rotation_translation(
        Vec3::splat(2.0),
        glam::Quat::from_rotation_y(1.0),
        Vec3::new(1.0, 2.0, 3.0),
    );

    c.bench_function("mat4_inverse_scalar", |bencher| {
        bencher.iter(|| black_box(m.inverse()));
    });

    c.bench_function("mat4_inverse_simd", |bencher| {
        bencher.iter(|| black_box(inverse_simd(m)));
    });
}

fn bench_transform_point(c: &mut Criterion) {
    let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let p = Vec3::new(4.0, 5.0, 6.0);

    c.bench_function("transform_point_scalar", |bencher| {
        bencher.iter(|| black_box(m.transform_point3(p)));
    });

    c.bench_function("transform_point_simd", |bencher| {
        bencher.iter(|| black_box(transform_point_simd(m, p)));
    });
}

fn bench_transform_points_batch(c: &mut Criterion) {
    let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let points: Vec<Vec3> = (0..16)
        .map(|i| Vec3::new(i as f32, i as f32 * 2.0, i as f32 * 3.0))
        .collect();

    c.bench_function("transform_points_batch_scalar", |bencher| {
        bencher.iter(|| {
            let result: Vec<Vec3> = points.iter().map(|&p| m.transform_point3(p)).collect();
            black_box(result)
        });
    });

    c.bench_function("transform_points_batch_simd", |bencher| {
        bencher.iter(|| black_box(transform_points_batch(m, &points)));
    });
}

criterion_group!(
    benches,
    bench_mat4_multiply,
    bench_mat4_transpose,
    bench_mat4_inverse,
    bench_transform_point,
    bench_transform_points_batch,
);
criterion_main!(benches);
