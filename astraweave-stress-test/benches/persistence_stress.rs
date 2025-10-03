use astraweave_stress_test::{generate_stress_entities, StressTestConfig};
use criterion::{criterion_group, criterion_main, Criterion};

fn persistence_stress_benchmark(c: &mut Criterion) {
    let config = StressTestConfig {
        entity_count: 2000,
        ai_entity_count: 200,
        network_entity_count: 100,
        test_duration_seconds: 60, // Set to desired duration
        max_memory_mb: 1024,       // Set to desired memory limit
    };

    c.bench_function("persistence_stress", |b| {
        b.iter(|| {
            let entities = generate_stress_entities(std::hint::black_box(&config));
            std::hint::black_box(entities);
        })
    });
}

criterion_group!(benches, persistence_stress_benchmark);
criterion_main!(benches);
