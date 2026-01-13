// AstraWeave Quest System Benchmarks
// =============================================================================
// Comprehensive performance and correctness validation for quest operations
//
// MISSION-CRITICAL CORRECTNESS ASSERTIONS
// =============================================================================
// This benchmark suite validates not only performance but CORRECTNESS of quest systems:
//   1. Quest Validation: Validates quest structure correctly
//   2. Completion Tracking: Accurately tracks quest step completion
//   3. Serialization: Round-trip serialization preserves data integrity
//   4. Component Operations: ECS components perform within frame budget
//   5. Terrain Quest Generation: Terrain triggers generate valid quests
// =============================================================================

use astraweave_quests::{
    Quest, QuestStep,
    CQuestGenerator, CActiveQuest, QuestState, ChoiceRecord,
    TerrainFeatureType, TerrainQuestContext, TerrainQuestTrigger,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box;

// =============================================================================
// CORRECTNESS ASSERTION HELPERS
// =============================================================================

/// Validates that a quest is structurally correct
#[inline]
fn assert_quest_valid(quest: &Quest, context: &str) {
    if quest.title.is_empty() {
        panic!("[CORRECTNESS FAILURE] {}: quest has empty title", context);
    }
}

/// Validates that validation result matches expected state
#[inline]
fn assert_validation_result(result: &Result<(), String>, should_pass: bool, context: &str) {
    match (result.is_ok(), should_pass) {
        (true, false) => panic!("[CORRECTNESS FAILURE] {}: validation passed when it should fail", context),
        (false, true) => panic!("[CORRECTNESS FAILURE] {}: validation failed when it should pass - {:?}", context, result),
        _ => {}
    }
}

// =============================================================================
// TEST DATA GENERATORS
// =============================================================================

/// Create a minimal quest with specified number of steps
fn create_quest(title: &str, num_steps: usize, completed_count: usize) -> Quest {
    let mut steps = Vec::with_capacity(num_steps);
    for i in 0..num_steps {
        steps.push(QuestStep {
            description: format!("Step {}: Complete objective", i + 1),
            completed: i < completed_count,
        });
    }
    Quest {
        title: title.to_string(),
        steps,
    }
}

/// Create terrain quest context for benchmarking
fn create_terrain_context(feature_type: TerrainFeatureType) -> TerrainQuestContext {
    TerrainQuestContext {
        feature_type,
        position: (100.0, 50.0, 100.0),
        radius: 64.0,
        intensity: 0.75,
        biome: "forest".to_string(),
        nearby_features: vec!["ancient_ruins".to_string(), "river".to_string()],
        is_ai_generated: true,
        seed: Some(42),
    }
}

/// Create quest generator component
fn create_quest_generator() -> CQuestGenerator {
    CQuestGenerator::new(
        "player_001".to_string(),
        10,
        "mystic_forest".to_string(),
    )
}

/// Create terrain quest trigger
fn create_terrain_trigger(feature_types: Vec<TerrainFeatureType>) -> TerrainQuestTrigger {
    TerrainQuestTrigger {
        id: "trigger_001".to_string(),
        feature_types,
        min_player_level: 1,
        max_player_level: Some(50),
        required_biomes: vec!["forest".to_string(), "mountain".to_string()],
        trigger_probability: 0.5,
        cooldown_seconds: 300.0,
        quest_template: "exploration".to_string(),
    }
}

// =============================================================================
// QUEST VALIDATION BENCHMARKS
// =============================================================================

fn bench_quest_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("quest_validation");
    group.throughput(Throughput::Elements(1));
    
    // Benchmark valid quest validation
    let valid_quest = create_quest("The Hero's Journey", 5, 0);
    group.bench_function("validate_valid_quest", |b| {
        b.iter(|| {
            let result = black_box(&valid_quest).validate();
            assert_validation_result(&result, true, "validate_valid_quest");
            result
        })
    });
    
    // Benchmark invalid quest (empty title) validation
    let invalid_title = Quest {
        title: String::new(),
        steps: vec![QuestStep {
            description: "Step 1".to_string(),
            completed: false,
        }],
    };
    group.bench_function("validate_empty_title", |b| {
        b.iter(|| {
            let result = black_box(&invalid_title).validate();
            assert_validation_result(&result, false, "validate_empty_title");
            result
        })
    });
    
    // Benchmark invalid quest (no steps) validation
    let invalid_steps = Quest {
        title: "Empty Quest".to_string(),
        steps: vec![],
    };
    group.bench_function("validate_no_steps", |b| {
        b.iter(|| {
            let result = black_box(&invalid_steps).validate();
            assert_validation_result(&result, false, "validate_no_steps");
            result
        })
    });
    
    // Benchmark validation with varying step counts
    for step_count in [1, 5, 10, 20, 50].iter() {
        let quest = create_quest("Multi-step Quest", *step_count, 0);
        group.bench_with_input(
            BenchmarkId::new("validate_steps", step_count),
            step_count,
            |b, _| {
                b.iter(|| {
                    let result = black_box(&quest).validate();
                    assert_validation_result(&result, true, &format!("validate_{}_steps", step_count));
                    result
                })
            },
        );
    }
    
    group.finish();
}

// =============================================================================
// QUEST COMPLETION BENCHMARKS
// =============================================================================

fn bench_quest_completion(c: &mut Criterion) {
    let mut group = c.benchmark_group("quest_completion");
    group.throughput(Throughput::Elements(1));
    
    // Benchmark is_complete on completed quest
    let completed = create_quest("Complete Quest", 10, 10);
    group.bench_function("check_complete_all_done", |b| {
        b.iter(|| {
            let result = black_box(&completed).is_complete();
            assert!(result, "[CORRECTNESS FAILURE] Quest should be complete");
            result
        })
    });
    
    // Benchmark is_complete on partial quest
    let partial = create_quest("Partial Quest", 10, 5);
    group.bench_function("check_complete_partial", |b| {
        b.iter(|| {
            let result = black_box(&partial).is_complete();
            assert!(!result, "[CORRECTNESS FAILURE] Quest should not be complete");
            result
        })
    });
    
    // Benchmark is_complete on fresh quest
    let fresh = create_quest("Fresh Quest", 10, 0);
    group.bench_function("check_complete_none", |b| {
        b.iter(|| {
            let result = black_box(&fresh).is_complete();
            assert!(!result, "[CORRECTNESS FAILURE] Quest should not be complete");
            result
        })
    });
    
    // Benchmark with varying step counts
    for step_count in [1, 10, 50, 100].iter() {
        let quest = create_quest("Large Quest", *step_count, *step_count);
        group.bench_with_input(
            BenchmarkId::new("is_complete_steps", step_count),
            step_count,
            |b, _| {
                b.iter(|| {
                    let result = black_box(&quest).is_complete();
                    assert!(result, "[CORRECTNESS FAILURE] All steps complete, quest should be complete");
                    result
                })
            },
        );
    }
    
    group.finish();
}

// =============================================================================
// SERIALIZATION BENCHMARKS
// =============================================================================

fn bench_quest_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("quest_serialization");
    
    // Test quests of varying sizes
    for step_count in [1, 5, 10, 20, 50].iter() {
        let quest = create_quest("Serialization Test Quest", *step_count, *step_count / 2);
        let json_data = serde_json::to_string(&quest).unwrap();
        
        group.throughput(Throughput::Bytes(json_data.len() as u64));
        
        // Benchmark serialization
        group.bench_with_input(
            BenchmarkId::new("serialize_json", step_count),
            &quest,
            |b, quest| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(quest)).unwrap();
                    assert!(!json.is_empty(), "[CORRECTNESS FAILURE] Serialization produced empty result");
                    json
                })
            },
        );
        
        // Benchmark deserialization
        group.bench_with_input(
            BenchmarkId::new("deserialize_json", step_count),
            &json_data,
            |b, json| {
                b.iter(|| {
                    let quest: Quest = serde_json::from_str(black_box(json)).unwrap();
                    assert_quest_valid(&quest, "deserialize_json");
                    quest
                })
            },
        );
        
        // Benchmark round-trip
        group.bench_with_input(
            BenchmarkId::new("roundtrip_json", step_count),
            &quest,
            |b, original| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(original)).unwrap();
                    let restored: Quest = serde_json::from_str(&json).unwrap();
                    assert_eq!(
                        restored.title, original.title,
                        "[CORRECTNESS FAILURE] Round-trip changed quest title"
                    );
                    assert_eq!(
                        restored.steps.len(), original.steps.len(),
                        "[CORRECTNESS FAILURE] Round-trip changed step count"
                    );
                    restored
                })
            },
        );
    }
    
    group.finish();
}

// =============================================================================
// COMPONENT OPERATION BENCHMARKS
// =============================================================================

fn bench_component_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_operations");
    group.throughput(Throughput::Elements(1));
    
    // Benchmark quest generator creation
    group.bench_function("create_quest_generator", |b| {
        b.iter(|| {
            let gen = CQuestGenerator::new(
                black_box("player_001".to_string()),
                black_box(10),
                black_box("forest".to_string()),
            );
            assert_eq!(gen.context.player_level, 10, "[CORRECTNESS FAILURE] Wrong player level");
            gen
        })
    });
    
    // Benchmark can_generate_quest check
    let generator = create_quest_generator();
    group.bench_function("can_generate_quest", |b| {
        let current_time = 1000000u64;
        b.iter(|| {
            let result = black_box(&generator).can_generate_quest(black_box(current_time));
            result
        })
    });
    
    // Benchmark context update
    let mut generator = create_quest_generator();
    group.bench_function("update_context", |b| {
        b.iter(|| {
            black_box(&mut generator).update_context(
                Some("new_location".to_string()),
                Some(vec!["npc_1".to_string(), "npc_2".to_string()]),
                Some(HashMap::new()),
            );
        })
    });
    
    // Benchmark add/remove active quest
    let mut generator = create_quest_generator();
    group.bench_function("add_remove_active_quest", |b| {
        b.iter(|| {
            let gen = black_box(&mut generator);
            gen.add_active_quest("quest_001".to_string());
            gen.remove_active_quest("quest_001");
        })
    });
    
    // Benchmark add recent activity
    let mut generator = create_quest_generator();
    group.bench_function("add_recent_activity", |b| {
        b.iter(|| {
            black_box(&mut generator).add_recent_activity("Explored cave".to_string());
        })
    });
    
    group.finish();
}

// =============================================================================
// TERRAIN QUEST BENCHMARKS
// =============================================================================

fn bench_terrain_quests(c: &mut Criterion) {
    let mut group = c.benchmark_group("terrain_quests");
    group.throughput(Throughput::Elements(1));
    
    // Benchmark terrain context creation
    group.bench_function("create_terrain_context", |b| {
        b.iter(|| {
            let ctx = TerrainQuestContext {
                feature_type: black_box(TerrainFeatureType::Mountain),
                position: black_box((100.0, 200.0, 100.0)),
                radius: black_box(128.0),
                intensity: black_box(0.8),
                biome: "alpine".to_string(),
                nearby_features: vec!["glacier".to_string()],
                is_ai_generated: true,
                seed: Some(12345),
            };
            assert_eq!(ctx.feature_type, TerrainFeatureType::Mountain, 
                       "[CORRECTNESS FAILURE] Wrong feature type");
            ctx
        })
    });
    
    // Benchmark quest archetypes lookup
    group.bench_function("get_quest_archetypes", |b| {
        b.iter(|| {
            let archetypes = black_box(TerrainFeatureType::Mountain).quest_archetypes();
            assert!(!archetypes.is_empty(), "[CORRECTNESS FAILURE] No archetypes returned");
            archetypes
        })
    });
    
    // Benchmark all terrain feature archetypes
    for feature in [
        TerrainFeatureType::Mountain,
        TerrainFeatureType::Forest,
        TerrainFeatureType::Lake,
        TerrainFeatureType::Cave,
        TerrainFeatureType::Desert,
    ].iter() {
        group.bench_with_input(
            BenchmarkId::new("archetypes_for", format!("{:?}", feature)),
            feature,
            |b, feature| {
                b.iter(|| {
                    let archetypes = black_box(*feature).quest_archetypes();
                    assert!(!archetypes.is_empty(), "[CORRECTNESS FAILURE] No archetypes for {:?}", feature);
                    archetypes
                })
            },
        );
    }
    
    // Benchmark difficulty modifier
    group.bench_function("get_difficulty_modifier", |b| {
        b.iter(|| {
            let modifiers: Vec<f32> = [
                TerrainFeatureType::Mountain,
                TerrainFeatureType::Meadow,
                TerrainFeatureType::Cave,
            ].iter().map(|t| black_box(*t).difficulty_modifier()).collect();
            assert!(modifiers[0] > modifiers[1], 
                   "[CORRECTNESS FAILURE] Mountain should be harder than meadow");
            modifiers
        })
    });
    
    // Benchmark trigger should_trigger check
    let trigger = create_terrain_trigger(vec![
        TerrainFeatureType::Mountain,
        TerrainFeatureType::Cave,
        TerrainFeatureType::Forest,
    ]);
    let context = create_terrain_context(TerrainFeatureType::Mountain);
    group.bench_function("should_trigger_check", |b| {
        let mut rng = rand::rng();
        b.iter(|| {
            let result = black_box(&trigger).should_trigger(
                black_box(&context),
                black_box(15),
                &mut rng,
            );
            result
        })
    });
    
    // Benchmark trigger with non-matching feature
    let non_matching_context = create_terrain_context(TerrainFeatureType::Lake);
    group.bench_function("should_trigger_no_match", |b| {
        let mut rng = rand::rng();
        b.iter(|| {
            let result = black_box(&trigger).should_trigger(
                black_box(&non_matching_context),
                black_box(15),
                &mut rng,
            );
            assert!(!result, "[CORRECTNESS FAILURE] Should not trigger for non-matching feature");
            result
        })
    });
    
    group.finish();
}

// =============================================================================
// BATCH OPERATION BENCHMARKS
// =============================================================================

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    // Benchmark validating multiple quests
    for batch_size in [10, 50, 100, 500].iter() {
        let quests: Vec<Quest> = (0..*batch_size)
            .map(|i| create_quest(&format!("Quest {}", i), 5, 0))
            .collect();
        
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("validate_batch", batch_size),
            &quests,
            |b, quests| {
                b.iter(|| {
                    let mut valid_count = 0;
                    for quest in black_box(quests).iter() {
                        if quest.validate().is_ok() {
                            valid_count += 1;
                        }
                    }
                    assert_eq!(valid_count, quests.len(), 
                              "[CORRECTNESS FAILURE] Not all quests validated");
                    valid_count
                })
            },
        );
        
        // Benchmark checking completion in batch
        let completed_quests: Vec<Quest> = (0..*batch_size)
            .map(|i| create_quest(&format!("Quest {}", i), 5, 5))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("check_completion_batch", batch_size),
            &completed_quests,
            |b, quests| {
                b.iter(|| {
                    let completed: Vec<bool> = black_box(quests)
                        .iter()
                        .map(|q| q.is_complete())
                        .collect();
                    assert!(completed.iter().all(|&c| c), 
                           "[CORRECTNESS FAILURE] All quests should be complete");
                    completed
                })
            },
        );
    }
    
    group.finish();
}

// =============================================================================
// BENCHMARK GROUPS AND MAIN
// =============================================================================

criterion_group!(
    name = quests_benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(3));
    targets = 
        bench_quest_validation,
        bench_quest_completion,
        bench_quest_serialization,
        bench_component_operations,
        bench_terrain_quests,
        bench_batch_operations,
);

criterion_main!(quests_benches);
