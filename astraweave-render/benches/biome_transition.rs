//! Biome Transition Pipeline Benchmarks
//!
//! Measures performance of the biome transition system including:
//! - BiomeVisuals::for_biome() lookup
//! - BiomeVisuals::lerp() interpolation
//! - TransitionEffect update cycles
//! - SceneEnvironment weather application
//! - Full pipeline simulation
//!
//! These benchmarks validate that the biome transition system meets
//! 60 FPS frame budgets (16.67ms).

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::time::Duration;

use astraweave_render::{
    biome_transition::{BiomeVisuals, EasingFunction, TransitionConfig, TransitionEffect},
    effects::WeatherKind,
    scene_environment::SceneEnvironment,
};
use astraweave_terrain::biome::BiomeType;

const BIOMES: [BiomeType; 8] = [
    BiomeType::Forest,
    BiomeType::Desert,
    BiomeType::Grassland,
    BiomeType::Mountain,
    BiomeType::Tundra,
    BiomeType::Swamp,
    BiomeType::Beach,
    BiomeType::River,
];

const WEATHER_KINDS: [WeatherKind; 5] = [
    WeatherKind::None,
    WeatherKind::Rain,
    WeatherKind::Snow,
    WeatherKind::Sandstorm,
    WeatherKind::WindTrails,
];

// ============================================================================
// BiomeVisuals Benchmarks
// ============================================================================

fn bench_biome_visuals_for_biome(c: &mut Criterion) {
    let mut group = c.benchmark_group("biome_visuals");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    // Benchmark single lookup
    group.bench_function("for_biome_single", |b| {
        b.iter(|| {
            black_box(BiomeVisuals::for_biome(BiomeType::Forest))
        })
    });

    // Benchmark all 8 biomes
    group.bench_function("for_biome_all_8", |b| {
        b.iter(|| {
            for biome in &BIOMES {
                black_box(BiomeVisuals::for_biome(*biome));
            }
        })
    });

    // Benchmark lerp interpolation
    group.bench_function("lerp_single", |b| {
        let from = BiomeVisuals::for_biome(BiomeType::Forest);
        let to = BiomeVisuals::for_biome(BiomeType::Tundra);
        b.iter(|| {
            black_box(BiomeVisuals::lerp(&from, &to, 0.5))
        })
    });

    // Benchmark lerp at multiple t values (simulating transition)
    group.bench_function("lerp_full_transition_100_steps", |b| {
        let from = BiomeVisuals::for_biome(BiomeType::Forest);
        let to = BiomeVisuals::for_biome(BiomeType::Tundra);
        b.iter(|| {
            for i in 0..100 {
                let t = i as f32 / 99.0;
                black_box(BiomeVisuals::lerp(&from, &to, t));
            }
        })
    });

    group.finish();
}

// ============================================================================
// TransitionEffect Benchmarks
// ============================================================================

fn bench_transition_effect(c: &mut Criterion) {
    let mut group = c.benchmark_group("transition_effect");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    // Benchmark creation
    group.bench_function("new", |b| {
        let config = TransitionConfig {
            duration: 1.5,
            easing: EasingFunction::SmootherStep,
            blend_fog: true,
            blend_ambient: true,
            apply_tint: true,
            tint_alpha: 0.15,
        };
        b.iter(|| {
            black_box(TransitionEffect::new(config.clone()))
        })
    });

    // Benchmark starting a transition
    group.bench_function("start", |b| {
        let config = TransitionConfig::default();
        let mut effect = TransitionEffect::new(config);
        b.iter(|| {
            effect.start(Some(BiomeType::Forest), BiomeType::Tundra);
            black_box(&effect);
        })
    });

    // Benchmark update per frame (most important!)
    group.bench_function("update_per_frame", |b| {
        let config = TransitionConfig::default();
        let mut effect = TransitionEffect::new(config);
        effect.start(Some(BiomeType::Forest), BiomeType::Tundra);
        let dt = 1.0 / 60.0; // 60 FPS
        b.iter(|| {
            effect.update(dt);
            black_box(effect.current_visuals());
        })
    });

    // Benchmark full 1.5s transition (90 frames at 60 FPS)
    group.bench_function("full_transition_90_frames", |b| {
        let config = TransitionConfig {
            duration: 1.5,
            easing: EasingFunction::SmootherStep,
            ..Default::default()
        };
        let dt = 1.0 / 60.0;
        b.iter(|| {
            let mut effect = TransitionEffect::new(config.clone());
            effect.start(Some(BiomeType::Forest), BiomeType::Tundra);
            for _ in 0..90 {
                effect.update(dt);
                black_box(effect.current_visuals());
            }
        })
    });

    group.finish();
}

// ============================================================================
// SceneEnvironment Benchmarks
// ============================================================================

fn bench_scene_environment(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_environment");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    // Benchmark default creation
    group.bench_function("default", |b| {
        b.iter(|| {
            black_box(SceneEnvironment::default())
        })
    });

    // Benchmark set_biome
    group.bench_function("set_biome", |b| {
        let mut env = SceneEnvironment::default();
        b.iter(|| {
            env.set_biome(BiomeType::Forest);
            black_box(&env);
        })
    });

    // Benchmark apply_weather for each type
    for kind in &WEATHER_KINDS {
        let kind_name = format!("{:?}", kind);
        group.bench_function(BenchmarkId::new("apply_weather", &kind_name), |b| {
            let mut env = SceneEnvironment::default();
            b.iter(|| {
                env.apply_weather(*kind);
                black_box(&env);
            })
        });
    }

    // Benchmark to_ubo conversion
    group.bench_function("to_ubo", |b| {
        let env = SceneEnvironment::default();
        b.iter(|| {
            black_box(env.to_ubo())
        })
    });

    group.finish();
}

// ============================================================================
// Full Pipeline Benchmarks
// ============================================================================

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("biome_pipeline");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.throughput(Throughput::Elements(1)); // 1 frame

    // Benchmark per-frame update (steady state, no transition)
    group.bench_function("per_frame_steady_state", |b| {
        let mut env = SceneEnvironment::default();
        env.set_biome(BiomeType::Forest);
        env.apply_weather(WeatherKind::Rain);
        b.iter(|| {
            // Simulate per-frame operations
            let ubo = env.to_ubo();
            black_box(ubo);
        })
    });

    // Benchmark per-frame update during active transition
    group.bench_function("per_frame_during_transition", |b| {
        let config = TransitionConfig::default();
        let mut effect = TransitionEffect::new(config);
        effect.start(Some(BiomeType::Forest), BiomeType::Tundra);
        
        let mut env = SceneEnvironment::default();
        let dt = 1.0 / 60.0;
        
        b.iter(|| {
            // Update transition
            effect.update(dt);
            // Update scene environment
            env.visuals = effect.current_visuals();
            env.blend_factor = effect.blend_factor();
            env.tint_alpha = effect.tint_alpha();
            // Apply weather
            env.apply_weather(WeatherKind::Rain);
            // Convert to GPU format
            let ubo = env.to_ubo();
            black_box(ubo);
        })
    });

    // Benchmark biome change + new weather
    group.bench_function("biome_change_with_weather", |b| {
        let config = TransitionConfig::default();
        b.iter(|| {
            let mut effect = TransitionEffect::new(config.clone());
            effect.start(Some(BiomeType::Forest), BiomeType::Desert);
            
            let mut env = SceneEnvironment::default();
            env.visuals = effect.current_visuals();
            env.apply_weather(WeatherKind::Sandstorm);
            let ubo = env.to_ubo();
            black_box(ubo);
        })
    });

    // Benchmark 1000 biome lookups (stress test)
    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_biome_lookups", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                for biome in &BIOMES {
                    black_box(BiomeVisuals::for_biome(*biome));
                }
            }
        })
    });

    group.finish();
}

// ============================================================================
// Scaling Benchmarks (entity counts)
// ============================================================================

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("biome_scaling");
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(2));

    // Benchmark supporting multiple simultaneous biome zones
    for count in [1, 4, 16, 64] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("parallel_zones", count),
            &count,
            |b, &count| {
                let dt = 1.0 / 60.0;
                
                b.iter(|| {
                    // Create effects fresh each iteration (measures creation + update cost)
                    for _ in 0..count {
                        let config = TransitionConfig::default();
                        let mut effect = TransitionEffect::new(config);
                        effect.start(Some(BiomeType::Forest), BiomeType::Tundra);
                        effect.update(dt);
                        
                        let mut env = SceneEnvironment::default();
                        env.visuals = effect.current_visuals();
                        black_box(env.to_ubo());
                    }
                })
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_biome_visuals_for_biome,
    bench_transition_effect,
    bench_scene_environment,
    bench_full_pipeline,
    bench_scaling,
);

criterion_main!(benches);