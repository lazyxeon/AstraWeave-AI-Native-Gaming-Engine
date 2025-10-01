use criterion::{black_box, criterion_group, criterion_main, Criterion};
use astraweave_stress_test::{generate_stress_entities, run_stress_test, StressTestConfig};

fn persistence_stress_benchmark(c: &mut Criterion) {
    let config = StressTestConfig {
        entity_count: 2000,
        ai_entity_count: 200,
        network_entity_count: 100,
        duration_secs: 1,
    };

    c.bench_function("persistence_stress", |b| {
        b.iter(|| {
            let entities = generate_stress_entities(black_box(&config));
            black_box(entities);
        })
    });
}

criterion_group!(benches, persistence_stress_benchmark);
criterion_main!(benches);