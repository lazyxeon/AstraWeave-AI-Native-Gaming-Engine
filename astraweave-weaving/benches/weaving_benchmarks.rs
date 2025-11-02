//! Weaving System Benchmarks
//!
//! Measures performance of emergent behavior layer:
//! - Pattern detection (analyzing world state)
//! - Intent proposal (generating actions from patterns)
//! - Adjudication (budget enforcement, cooldown management, prioritization)
//! - Full weave pipeline (detect → propose → adjudicate)
//!
//! Performance targets:
//! - Pattern detection: <1 µs per detector (real-time pattern matching)
//! - Intent proposal: <500 ns per proposer (fast intent generation)
//! - Adjudication: <1 µs for 10 intents (efficient prioritization)
//! - Full pipeline: <5 µs (sub-frame budget @ 60 FPS)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::BTreeMap;
use std::hint::black_box as std_black_box;

use astraweave_weaving::{
    adjudicator::{WeaveAdjudicator, WeaveConfig},
    intents::{AidEventProposer, IntentProposer, SupplyDropProposer, WeaveIntent},
    patterns::{
        LowHealthClusterDetector, PatternDetector, PatternStrength, ResourceScarcityDetector,
        WorldMetrics,
    },
};

// ============================================================================
// Benchmark 1: Pattern Detection
// ============================================================================

fn bench_pattern_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_detection");

    // Setup world metrics with varying conditions
    let metrics_low_health = WorldMetrics {
        avg_health: 0.15,
        critical_health_count: 8,
        resource_scarcity: BTreeMap::new(),
        faction_tensions: BTreeMap::new(),
        recent_damage_events: 5,
        time_since_event: 2.0,
    };

    let mut metrics_resource_scarcity = WorldMetrics::default();
    metrics_resource_scarcity
        .resource_scarcity
        .insert("food".to_string(), 0.8);
    metrics_resource_scarcity
        .resource_scarcity
        .insert("water".to_string(), 0.6);

    // Benchmark: Low health cluster detection
    group.bench_function("low_health_cluster", |b| {
        let detector = LowHealthClusterDetector {
            threshold: 0.2,
            min_cluster_size: 3,
        };

        b.iter(|| {
            let patterns = detector.detect(&metrics_low_health);
            std_black_box(patterns)
        })
    });

    // Benchmark: Resource scarcity detection
    group.bench_function("resource_scarcity", |b| {
        let detector = ResourceScarcityDetector { threshold: 0.5 };

        b.iter(|| {
            let patterns = detector.detect(&metrics_resource_scarcity);
            std_black_box(patterns)
        })
    });

    // Benchmark: Multiple detectors (realistic scenario)
    group.bench_function("multiple_detectors", |b| {
        let detectors: Vec<Box<dyn PatternDetector>> = vec![
            Box::new(LowHealthClusterDetector {
                threshold: 0.2,
                min_cluster_size: 3,
            }),
            Box::new(ResourceScarcityDetector { threshold: 0.5 }),
        ];

        let mut metrics = metrics_low_health.clone();
        metrics
            .resource_scarcity
            .insert("food".to_string(), 0.7);

        b.iter(|| {
            let mut all_patterns = BTreeMap::new();
            for detector in &detectors {
                for (id, strength) in detector.detect(&metrics) {
                    all_patterns.insert(id, strength);
                }
            }
            std_black_box(all_patterns)
        })
    });

    // Benchmark: Pattern strength categorization
    group.bench_function("pattern_strength_categorization", |b| {
        b.iter(|| {
            let weak = PatternStrength::from_value(0.1);
            let moderate = PatternStrength::from_value(0.5);
            let strong = PatternStrength::from_value(0.9);
            std_black_box((weak, moderate, strong))
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 2: Intent Proposal
// ============================================================================

fn bench_intent_proposal(c: &mut Criterion) {
    let mut group = c.benchmark_group("intent_proposal");

    // Setup detected patterns
    let mut patterns = BTreeMap::new();
    patterns.insert("low_health_cluster".to_string(), 0.8);
    patterns.insert("resource_scarce_food".to_string(), 0.7);
    patterns.insert("resource_scarce_water".to_string(), 0.5);

    // Benchmark: Aid event proposal
    group.bench_function("aid_event_proposal", |b| {
        let proposer = AidEventProposer {
            strength_threshold: 0.5,
        };

        b.iter(|| {
            let intents = proposer.propose(&patterns, 12345);
            std_black_box(intents)
        })
    });

    // Benchmark: Supply drop proposal
    group.bench_function("supply_drop_proposal", |b| {
        let proposer = SupplyDropProposer {
            strength_threshold: 0.5,
        };

        b.iter(|| {
            let intents = proposer.propose(&patterns, 12345);
            std_black_box(intents)
        })
    });

    // Benchmark: Multiple proposers
    group.bench_function("multiple_proposers", |b| {
        let proposers: Vec<Box<dyn IntentProposer>> = vec![
            Box::new(AidEventProposer {
                strength_threshold: 0.5,
            }),
            Box::new(SupplyDropProposer {
                strength_threshold: 0.5,
            }),
        ];

        b.iter(|| {
            let mut all_intents = Vec::new();
            for proposer in &proposers {
                all_intents.extend(proposer.propose(&patterns, 12345));
            }
            std_black_box(all_intents)
        })
    });

    // Benchmark: Intent builder pattern
    group.bench_function("intent_builder", |b| {
        b.iter(|| {
            let intent = WeaveIntent::new("test_intent")
                .with_priority(0.8)
                .with_cost(10)
                .with_cooldown("test_cooldown")
                .with_payload("key1", "value1")
                .with_payload("key2", "value2");
            std_black_box(intent)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 3: Adjudication
// ============================================================================

fn bench_adjudication(c: &mut Criterion) {
    let mut group = c.benchmark_group("adjudication");

    // Setup config
    let config = WeaveConfig::default();

    // Benchmark: Cooldown check
    group.bench_function("cooldown_check", |b| {
        let mut adjudicator = WeaveAdjudicator::with_config(config.clone());
        adjudicator.begin_tick();

        b.iter(|| {
            let on_cooldown = adjudicator.is_on_cooldown("aid_event");
            std_black_box(on_cooldown)
        })
    });

    // Benchmark: Budget check
    group.bench_function("budget_check", |b| {
        let adjudicator = WeaveAdjudicator::with_config(config.clone());

        b.iter(|| {
            let has_budget = adjudicator.has_budget(10);
            std_black_box(has_budget)
        })
    });

    // Benchmark: Begin tick (cooldown decrements)
    group.bench_function("begin_tick", |b| {
        let mut adjudicator = WeaveAdjudicator::with_config(config.clone());

        // Add some active cooldowns
        adjudicator.begin_tick();
        let intent = WeaveIntent::new("spawn_aid_event")
            .with_cost(10)
            .with_cooldown("aid_event");
        adjudicator.adjudicate(vec![intent]);

        b.iter(|| {
            adjudicator.begin_tick();
        })
    });

    // Benchmark: Adjudicate small batch (5 intents)
    group.bench_function("adjudicate_5_intents", |b| {
        let mut adjudicator = WeaveAdjudicator::with_config(config.clone());

        b.iter(|| {
            adjudicator.begin_tick();
            let intents = vec![
                WeaveIntent::new("intent1")
                    .with_priority(0.9)
                    .with_cost(5),
                WeaveIntent::new("intent2")
                    .with_priority(0.7)
                    .with_cost(3),
                WeaveIntent::new("intent3")
                    .with_priority(0.5)
                    .with_cost(2),
                WeaveIntent::new("intent4")
                    .with_priority(0.3)
                    .with_cost(4),
                WeaveIntent::new("intent5")
                    .with_priority(0.2)
                    .with_cost(1),
            ];
            let approved = adjudicator.adjudicate(intents);
            std_black_box(approved)
        })
    });

    // Benchmark: Adjudicate medium batch (10 intents)
    group.bench_function("adjudicate_10_intents", |b| {
        let mut adjudicator = WeaveAdjudicator::with_config(config.clone());

        b.iter(|| {
            adjudicator.begin_tick();
            let mut intents = Vec::new();
            for i in 0..10 {
                intents.push(
                    WeaveIntent::new(format!("intent{}", i))
                        .with_priority(1.0 - (i as f32 * 0.1))
                        .with_cost(i as u32 + 1),
                );
            }
            let approved = adjudicator.adjudicate(intents);
            std_black_box(approved)
        })
    });

    // Benchmark: Adjudicate with cooldowns active
    group.bench_function("adjudicate_with_cooldowns", |b| {
        let mut adjudicator = WeaveAdjudicator::with_config(config.clone());

        // Pre-activate some cooldowns
        adjudicator.begin_tick();
        let intents = vec![
            WeaveIntent::new("aid_event")
                .with_cost(10)
                .with_cooldown("aid_event"),
            WeaveIntent::new("supply_drop")
                .with_cost(8)
                .with_cooldown("supply_drop_food"),
        ];
        adjudicator.adjudicate(intents);

        b.iter(|| {
            adjudicator.begin_tick();
            let intents = vec![
                WeaveIntent::new("aid_event")
                    .with_priority(0.9)
                    .with_cost(10)
                    .with_cooldown("aid_event"), // Should be blocked
                WeaveIntent::new("supply_drop")
                    .with_priority(0.8)
                    .with_cost(8)
                    .with_cooldown("supply_drop_food"), // Should be blocked
                WeaveIntent::new("mediator")
                    .with_priority(0.7)
                    .with_cost(5)
                    .with_cooldown("mediator"), // Should pass
            ];
            let approved = adjudicator.adjudicate(intents);
            std_black_box(approved)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 4: Configuration
// ============================================================================

fn bench_configuration(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration");

    // Benchmark: Config creation
    group.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = WeaveConfig::default();
            std_black_box(config)
        })
    });

    // Benchmark: Config TOML serialization
    group.bench_function("config_to_toml", |b| {
        let config = WeaveConfig::default();

        b.iter(|| {
            let toml_str = config.to_toml().unwrap();
            std_black_box(toml_str)
        })
    });

    // Benchmark: Config TOML deserialization
    group.bench_function("config_from_toml", |b| {
        let config = WeaveConfig::default();
        let toml_str = config.to_toml().unwrap();

        b.iter(|| {
            let parsed = WeaveConfig::from_toml(&toml_str).unwrap();
            std_black_box(parsed)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 5: Full Pipeline
// ============================================================================

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    // Benchmark: Complete weave cycle (detect → propose → adjudicate)
    group.bench_function("full_weave_cycle", |b| {
        // Setup components
        let detectors: Vec<Box<dyn PatternDetector>> = vec![
            Box::new(LowHealthClusterDetector {
                threshold: 0.2,
                min_cluster_size: 3,
            }),
            Box::new(ResourceScarcityDetector { threshold: 0.5 }),
        ];

        let proposers: Vec<Box<dyn IntentProposer>> = vec![
            Box::new(AidEventProposer {
                strength_threshold: 0.5,
            }),
            Box::new(SupplyDropProposer {
                strength_threshold: 0.5,
            }),
        ];

        let config = WeaveConfig::default();
        let mut adjudicator = WeaveAdjudicator::with_config(config);

        // Setup world state
        let mut metrics = WorldMetrics {
            avg_health: 0.15,
            critical_health_count: 8,
            resource_scarcity: BTreeMap::new(),
            faction_tensions: BTreeMap::new(),
            recent_damage_events: 5,
            time_since_event: 2.0,
        };
        metrics
            .resource_scarcity
            .insert("food".to_string(), 0.7);
        metrics
            .resource_scarcity
            .insert("water".to_string(), 0.5);

        b.iter(|| {
            // Step 1: Pattern detection
            let mut patterns = BTreeMap::new();
            for detector in &detectors {
                for (id, strength) in detector.detect(&metrics) {
                    patterns.insert(id, strength);
                }
            }

            // Step 2: Intent proposal
            let mut intents = Vec::new();
            for proposer in &proposers {
                intents.extend(proposer.propose(&patterns, 12345));
            }

            // Step 3: Adjudication
            adjudicator.begin_tick();
            let approved = adjudicator.adjudicate(intents);

            std_black_box(approved)
        })
    });

    // Benchmark: Scaling with world complexity
    for entity_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("pipeline_scaling", entity_count),
            entity_count,
            |b, &count| {
                let detectors: Vec<Box<dyn PatternDetector>> = vec![
                    Box::new(LowHealthClusterDetector {
                        threshold: 0.2,
                        min_cluster_size: 3,
                    }),
                    Box::new(ResourceScarcityDetector { threshold: 0.5 }),
                ];

                let proposers: Vec<Box<dyn IntentProposer>> = vec![
                    Box::new(AidEventProposer {
                        strength_threshold: 0.5,
                    }),
                    Box::new(SupplyDropProposer {
                        strength_threshold: 0.5,
                    }),
                ];

                let config = WeaveConfig::default();
                let mut adjudicator = WeaveAdjudicator::with_config(config);

                let mut metrics = WorldMetrics {
                    avg_health: 0.15,
                    critical_health_count: (count as f32 * 0.2) as usize,
                    resource_scarcity: BTreeMap::new(),
                    faction_tensions: BTreeMap::new(),
                    recent_damage_events: count / 10,
                    time_since_event: 2.0,
                };
                metrics
                    .resource_scarcity
                    .insert("food".to_string(), 0.7);

                b.iter(|| {
                    let mut patterns = BTreeMap::new();
                    for detector in &detectors {
                        for (id, strength) in detector.detect(&metrics) {
                            patterns.insert(id, strength);
                        }
                    }

                    let mut intents = Vec::new();
                    for proposer in &proposers {
                        intents.extend(proposer.propose(&patterns, 12345));
                    }

                    adjudicator.begin_tick();
                    let approved = adjudicator.adjudicate(intents);
                    std_black_box(approved)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_pattern_detection,
    bench_intent_proposal,
    bench_adjudication,
    bench_configuration,
    bench_full_pipeline,
);
criterion_main!(benches);
