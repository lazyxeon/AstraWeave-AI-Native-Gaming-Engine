use astraweave_stress_test::{generate_stress_entities, StressTestConfig};
use criterion::{criterion_group, criterion_main, Criterion};

fn network_stress_benchmark(c: &mut Criterion) {
    let config = StressTestConfig {
        entity_count: 500,
        ai_entity_count: 50,
        network_entity_count: 200,
        test_duration_seconds: 1, // Set to desired value
        max_memory_mb: 1024,      // Set to desired value
    };

    c.bench_function("network_stress", |b| {
        b.iter(|| {
            let entities = generate_stress_entities(std::hint::black_box(&config));
            std::hint::black_box(entities);
        })
    });
}

criterion_group!(benches, network_stress_benchmark);
criterion_main!(benches);
