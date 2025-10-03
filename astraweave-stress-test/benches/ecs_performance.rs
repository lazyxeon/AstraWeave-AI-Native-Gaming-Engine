use astraweave_stress_test::{generate_stress_entities, StressTestConfig};
use criterion::{criterion_group, criterion_main, Criterion};

fn ecs_performance_benchmark(c: &mut Criterion) {
    let config = StressTestConfig {
        entity_count: 1000,
        ai_entity_count: 100,
        network_entity_count: 50,
        test_duration_seconds: 1,
        max_memory_mb: 512,
    };

    c.bench_function("ecs_performance", |b| {
        b.iter(|| {
            let entities = generate_stress_entities(std::hint::black_box(&config));
            std::hint::black_box(entities);
        })
    });
}

criterion_group!(benches, ecs_performance_benchmark);
criterion_main!(benches);
