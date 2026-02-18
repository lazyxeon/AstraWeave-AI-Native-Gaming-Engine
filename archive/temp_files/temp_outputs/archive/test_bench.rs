// Quick test to see if criterion benchmarks are working

use criterion::{criterion_group, criterion_main, Criterion};

fn test_bench(c: &mut Criterion) {
    c.bench_function("simple_test", |b| {
        b.iter(|| {
            std::hint::black_box(42 + 42)
        })
    });
}

criterion_group!(benches, test_bench);
criterion_main!(benches);
