use criterion::{black_box, criterion_group, criterion_main, Criterion};
use astraweave_stress_test::{generate_stress_entities, run_stress_test, StressTestConfig};

fn ecs_performance_benchmark(c: &mut Criterion) {
    let config = StressTestConfig {
        entity_count: 1000,
        ai_entity_count: 100,
        network_entity_count: 50,
        duration_secs: 1,
    };

    c.bench_function("ecs_performance", |b| {
        b.iter(|| {
            let entities = generate_stress_entities(black_box(&config));
            black_box(entities);
        })
    });
}

criterion_group!(benches, ecs_performance_benchmark);
criterion_main!(benches);