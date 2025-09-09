use criterion::{criterion_group, criterion_main, Criterion};
use sha2::{Digest, Sha256};

fn bench_hash(c: &mut Criterion) {
    let data = vec![0u8; 8 * 1024 * 1024]; // 8MB
    c.bench_function("sha256 8MB", |b| {
        b.iter(|| {
            let mut h = Sha256::new();
            h.update(&data);
            let _ = h.finalize();
        })
    });
}
criterion_group!(benches, bench_hash);
criterion_main!(benches);
