//! Adversarial NPC Benchmarks
//!
//! Stress testing for NPC behavior, LLM integration, profiles, runtime, and sense systems.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-npc API)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NpcId(u64);

#[derive(Clone, Debug)]
struct NpcProfile {
    id: NpcId,
    name: String,
    personality: Personality,
    schedule: Vec<ScheduleEntry>,
    relationships: HashMap<NpcId, RelationshipLevel>,
    dialogue_history: Vec<DialogueEntry>,
}

#[derive(Clone, Copy, Debug)]
struct Personality {
    openness: f32,
    conscientiousness: f32,
    extraversion: f32,
    agreeableness: f32,
    neuroticism: f32,
}

impl Personality {
    fn ocean(o: f32, c: f32, e: f32, a: f32, n: f32) -> Self {
        Self {
            openness: o,
            conscientiousness: c,
            extraversion: e,
            agreeableness: a,
            neuroticism: n,
        }
    }
    
    fn compatibility(&self, other: &Self) -> f32 {
        let diff = (self.openness - other.openness).abs()
            + (self.conscientiousness - other.conscientiousness).abs()
            + (self.extraversion - other.extraversion).abs()
            + (self.agreeableness - other.agreeableness).abs()
            + (self.neuroticism - other.neuroticism).abs();
        
        1.0 - (diff / 5.0)
    }
}

#[derive(Clone, Copy, Debug)]
enum RelationshipLevel {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    Rival,
    Enemy,
}

impl RelationshipLevel {
    fn trust_multiplier(&self) -> f32 {
        match self {
            Self::Stranger => 0.0,
            Self::Acquaintance => 0.3,
            Self::Friend => 0.6,
            Self::CloseFriend => 1.0,
            Self::Rival => -0.3,
            Self::Enemy => -1.0,
        }
    }
}

#[derive(Clone, Debug)]
struct ScheduleEntry {
    time_start: u32, // Minutes from midnight
    time_end: u32,
    activity: Activity,
    location: String,
}

#[derive(Clone, Copy, Debug)]
enum Activity {
    Sleep,
    Work,
    Eat,
    Socialize,
    Patrol,
    Rest,
    Custom(u32),
}

#[derive(Clone, Debug)]
struct DialogueEntry {
    timestamp: u64,
    speaker: NpcId,
    text: String,
    emotion: Emotion,
}

#[derive(Clone, Copy, Debug)]
enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Fearful,
    Surprised,
}

#[derive(Clone, Debug)]
struct SenseData {
    visible_entities: Vec<(u64, [f32; 3], EntityType)>,
    audible_sounds: Vec<(SoundType, f32, [f32; 3])>,
    smell_traces: Vec<(SmellType, f32)>,
    danger_level: f32,
}

#[derive(Clone, Copy, Debug)]
enum EntityType {
    Player,
    FriendlyNpc,
    HostileNpc,
    Animal,
    Object,
}

#[derive(Clone, Copy, Debug)]
enum SoundType {
    Footsteps,
    Combat,
    Speech,
    Ambient,
    Alert,
}

#[derive(Clone, Copy, Debug)]
enum SmellType {
    Blood,
    Fire,
    Food,
    Creature,
}

#[derive(Clone, Debug)]
struct BehaviorState {
    current: BehaviorType,
    substate: u32,
    timer: f32,
    memory: HashMap<String, f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum BehaviorType {
    Idle,
    Patrol,
    Combat,
    Flee,
    Interact,
    FollowSchedule,
    Custom(u32),
}

#[derive(Clone, Debug)]
struct LlmContext {
    npc_id: NpcId,
    personality_summary: String,
    recent_events: Vec<String>,
    world_state: HashMap<String, String>,
    conversation_history: Vec<String>,
}

// ============================================================================
// CATEGORY 1: BEHAVIOR SYSTEMS
// ============================================================================

fn bench_behavior_systems(c: &mut Criterion) {
    let mut group = c.benchmark_group("npc_adversarial/behavior_systems");
    
    // Test 1: Behavior state transitions
    group.bench_function("state_transitions_5000", |bencher| {
        let mut states: Vec<BehaviorState> = (0..5000)
            .map(|i| BehaviorState {
                current: match i % 6 {
                    0 => BehaviorType::Idle,
                    1 => BehaviorType::Patrol,
                    2 => BehaviorType::Combat,
                    3 => BehaviorType::Flee,
                    4 => BehaviorType::Interact,
                    _ => BehaviorType::FollowSchedule,
                },
                substate: i as u32 % 10,
                timer: i as f32 * 0.1,
                memory: HashMap::new(),
            })
            .collect();
        
        let transition_conditions: Vec<(BehaviorType, BehaviorType, f32)> = vec![
            (BehaviorType::Idle, BehaviorType::Patrol, 5.0),
            (BehaviorType::Patrol, BehaviorType::Combat, 0.0),
            (BehaviorType::Combat, BehaviorType::Flee, 0.0),
            (BehaviorType::Flee, BehaviorType::Idle, 10.0),
        ];
        
        let dt = 0.016f32;
        
        bencher.iter(|| {
            let mut transitions = 0;
            
            for state in states.iter_mut() {
                state.timer += dt;
                
                for (from, to, min_time) in &transition_conditions {
                    if state.current == *from && state.timer > *min_time {
                        state.current = *to;
                        state.timer = 0.0;
                        transitions += 1;
                        break;
                    }
                }
            }
            
            std_black_box(transitions)
        });
    });
    
    // Test 2: Behavior tree evaluation
    group.bench_function("behavior_tree_eval_1000", |bencher| {
        // Simulate behavior tree nodes
        let trees: Vec<Vec<(u8, bool)>> = (0..1000)
            .map(|i| {
                (0..20)
                    .map(|j| {
                        let node_type = (j % 4) as u8; // 0=sequence, 1=selector, 2=action, 3=condition
                        let success = (i + j) % 3 != 0;
                        (node_type, success)
                    })
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            let results: Vec<bool> = trees
                .iter()
                .map(|tree| {
                    let mut stack: Vec<(usize, bool)> = vec![(0, true)];
                    
                    while let Some((idx, running)) = stack.pop() {
                        if idx >= tree.len() || !running {
                            return running;
                        }
                        
                        let (node_type, success) = tree[idx];
                        match node_type {
                            0 => {
                                // Sequence: all must succeed
                                if success && idx + 1 < tree.len() {
                                    stack.push((idx + 1, true));
                                }
                            }
                            1 => {
                                // Selector: first success wins
                                if !success && idx + 1 < tree.len() {
                                    stack.push((idx + 1, true));
                                }
                            }
                            _ => {
                                // Action/condition
                                return success;
                            }
                        }
                    }
                    
                    true
                })
                .collect();
            
            let successes = results.iter().filter(|&&r| r).count();
            std_black_box(successes)
        });
    });
    
    // Test 3: Utility AI scoring
    group.bench_function("utility_scoring_500", |bencher| {
        let npcs: Vec<(f32, f32, f32, f32)> = (0..500)
            .map(|i| {
                let health = (50 + i % 50) as f32;
                let danger = (i % 100) as f32;
                let hunger = (i % 80) as f32;
                let social = (i % 60) as f32;
                (health, danger, hunger, social)
            })
            .collect();
        
        bencher.iter(|| {
            let decisions: Vec<&str> = npcs
                .iter()
                .map(|(health, danger, hunger, social)| {
                    // Calculate utility scores
                    let fight_score = 100.0 - danger + health * 0.5;
                    let flee_score = *danger - health * 0.3;
                    let eat_score = *hunger * 1.5;
                    let talk_score = *social * 0.8;
                    
                    // Pick highest utility
                    let max = fight_score.max(flee_score).max(eat_score).max(talk_score);
                    
                    if (max - fight_score).abs() < 0.01 {
                        "fight"
                    } else if (max - flee_score).abs() < 0.01 {
                        "flee"
                    } else if (max - eat_score).abs() < 0.01 {
                        "eat"
                    } else {
                        "talk"
                    }
                })
                .collect();
            
            let fight_count = decisions.iter().filter(|&&d| d == "fight").count();
            std_black_box(fight_count)
        });
    });
    
    // Test 4: Goal-oriented behavior
    group.bench_function("goap_planning_200", |bencher| {
        let npcs: Vec<HashMap<String, i32>> = (0..200)
            .map(|i| {
                let mut state = HashMap::new();
                state.insert("has_weapon".to_string(), (i % 2) as i32);
                state.insert("enemy_visible".to_string(), (i % 3 == 0) as i32);
                state.insert("health".to_string(), 50 + (i % 50) as i32);
                state.insert("ammo".to_string(), (i % 30) as i32);
                state
            })
            .collect();
        
        // Available actions
        let actions: Vec<(&str, HashMap<String, i32>, HashMap<String, i32>, i32)> = vec![
            ("attack", [("has_weapon".into(), 1), ("enemy_visible".into(), 1)].into(), [("enemy_visible".into(), 0)].into(), 10),
            ("reload", [("has_weapon".into(), 1)].into(), [("ammo".into(), 30)].into(), 5),
            ("find_weapon", [("has_weapon".into(), 0)].into(), [("has_weapon".into(), 1)].into(), 15),
            ("heal", [("health".into(), -50)].into(), [("health".into(), 100)].into(), 20), // health < 50
        ];
        
        bencher.iter(|| {
            let plans: Vec<Vec<&str>> = npcs
                .iter()
                .map(|state| {
                    let mut plan = Vec::new();
                    let mut current = state.clone();
                    
                    // Simple forward planning
                    for _ in 0..5 {
                        for (name, precond, effects, _cost) in &actions {
                            let mut can_execute = true;
                            
                            for (key, &required) in precond {
                                if let Some(&value) = current.get(key) {
                                    if required >= 0 && value != required {
                                        can_execute = false;
                                    } else if required < 0 && value >= -required {
                                        can_execute = false;
                                    }
                                }
                            }
                            
                            if can_execute {
                                plan.push(*name);
                                for (key, &value) in effects {
                                    current.insert(key.clone(), value);
                                }
                                break;
                            }
                        }
                    }
                    
                    plan
                })
                .collect();
            
            let total_actions: usize = plans.iter().map(|p| p.len()).sum();
            std_black_box(total_actions)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 2: PROFILE MANAGEMENT
// ============================================================================

fn bench_profile_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("npc_adversarial/profile_management");
    
    // Test 1: Profile creation
    group.bench_function("profile_creation_500", |bencher| {
        bencher.iter(|| {
            let profiles: Vec<NpcProfile> = (0..500)
                .map(|i| {
                    NpcProfile {
                        id: NpcId(i as u64),
                        name: format!("NPC_{}", i),
                        personality: Personality::ocean(
                            (i as f32 * 0.002) % 1.0,
                            (i as f32 * 0.003) % 1.0,
                            (i as f32 * 0.004) % 1.0,
                            (i as f32 * 0.005) % 1.0,
                            (i as f32 * 0.006) % 1.0,
                        ),
                        schedule: (0..8)
                            .map(|j| ScheduleEntry {
                                time_start: j * 180,
                                time_end: (j + 1) * 180,
                                activity: match j % 6 {
                                    0 => Activity::Sleep,
                                    1 => Activity::Work,
                                    2 => Activity::Eat,
                                    3 => Activity::Socialize,
                                    4 => Activity::Patrol,
                                    _ => Activity::Rest,
                                },
                                location: format!("location_{}", j),
                            })
                            .collect(),
                        relationships: HashMap::new(),
                        dialogue_history: Vec::new(),
                    }
                })
                .collect();
            
            std_black_box(profiles.len())
        });
    });
    
    // Test 2: Personality compatibility calculation
    group.bench_function("personality_compatibility_1000", |bencher| {
        let personalities: Vec<Personality> = (0..1000)
            .map(|i| {
                Personality::ocean(
                    (i as f32 * 0.001) % 1.0,
                    (i as f32 * 0.002) % 1.0,
                    (i as f32 * 0.003) % 1.0,
                    (i as f32 * 0.004) % 1.0,
                    (i as f32 * 0.005) % 1.0,
                )
            })
            .collect();
        
        bencher.iter(|| {
            // Calculate all-pairs compatibility (sample)
            let mut total = 0.0f32;
            let mut count = 0;
            
            for i in 0..personalities.len().min(100) {
                for j in (i + 1)..personalities.len().min(100) {
                    total += personalities[i].compatibility(&personalities[j]);
                    count += 1;
                }
            }
            
            std_black_box(total / count as f32)
        });
    });
    
    // Test 3: Relationship updates
    group.bench_function("relationship_updates_5000", |bencher| {
        let mut profiles: Vec<NpcProfile> = (0..100)
            .map(|i| {
                let mut p = NpcProfile {
                    id: NpcId(i as u64),
                    name: format!("NPC_{}", i),
                    personality: Personality::ocean(0.5, 0.5, 0.5, 0.5, 0.5),
                    schedule: Vec::new(),
                    relationships: HashMap::new(),
                    dialogue_history: Vec::new(),
                };
                
                // Initialize some relationships
                for j in 0..10 {
                    p.relationships.insert(
                        NpcId(((i + j + 1) % 100) as u64),
                        match j % 4 {
                            0 => RelationshipLevel::Acquaintance,
                            1 => RelationshipLevel::Friend,
                            2 => RelationshipLevel::Rival,
                            _ => RelationshipLevel::Stranger,
                        },
                    );
                }
                p
            })
            .collect();
        
        let updates: Vec<(usize, NpcId, RelationshipLevel)> = (0..5000)
            .map(|i| {
                let profile_idx = i % 100;
                let target = NpcId(((i + 50) % 100) as u64);
                let level = match i % 6 {
                    0 => RelationshipLevel::Stranger,
                    1 => RelationshipLevel::Acquaintance,
                    2 => RelationshipLevel::Friend,
                    3 => RelationshipLevel::CloseFriend,
                    4 => RelationshipLevel::Rival,
                    _ => RelationshipLevel::Enemy,
                };
                (profile_idx, target, level)
            })
            .collect();
        
        bencher.iter(|| {
            for (idx, target, level) in &updates {
                profiles[*idx].relationships.insert(*target, *level);
            }
            
            let total_relationships: usize = profiles.iter().map(|p| p.relationships.len()).sum();
            std_black_box(total_relationships)
        });
    });
    
    // Test 4: Schedule lookup
    group.bench_function("schedule_lookup_10000", |bencher| {
        let profiles: Vec<NpcProfile> = (0..100)
            .map(|i| {
                NpcProfile {
                    id: NpcId(i as u64),
                    name: format!("NPC_{}", i),
                    personality: Personality::ocean(0.5, 0.5, 0.5, 0.5, 0.5),
                    schedule: (0..16)
                        .map(|j| ScheduleEntry {
                            time_start: j * 90,
                            time_end: (j + 1) * 90,
                            activity: Activity::Work,
                            location: format!("loc_{}", j),
                        })
                        .collect(),
                    relationships: HashMap::new(),
                    dialogue_history: Vec::new(),
                }
            })
            .collect();
        
        let lookups: Vec<(usize, u32)> = (0..10000)
            .map(|i| (i % 100, (i % 1440) as u32))
            .collect();
        
        bencher.iter(|| {
            let activities: Vec<Option<Activity>> = lookups
                .iter()
                .map(|(profile_idx, time)| {
                    profiles[*profile_idx]
                        .schedule
                        .iter()
                        .find(|e| *time >= e.time_start && *time < e.time_end)
                        .map(|e| e.activity)
                })
                .collect();
            
            let found = activities.iter().filter(|a| a.is_some()).count();
            std_black_box(found)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 3: SENSE SYSTEMS
// ============================================================================

fn bench_sense_systems(c: &mut Criterion) {
    let mut group = c.benchmark_group("npc_adversarial/sense_systems");
    
    // Test 1: Vision cone checks
    group.bench_function("vision_cone_5000", |bencher| {
        let npcs: Vec<([f32; 3], [f32; 3], f32, f32)> = (0..100)
            .map(|i| {
                let pos = [(i % 100) as f32, 0.0, ((i / 100) % 100) as f32];
                let forward = [1.0f32, 0.0, 0.0];
                let fov = 60.0f32.to_radians();
                let range = 50.0f32;
                (pos, forward, fov, range)
            })
            .collect();
        
        let targets: Vec<[f32; 3]> = (0..5000)
            .map(|i| {
                [
                    (i % 200) as f32 - 50.0,
                    0.0,
                    ((i / 200) % 100) as f32 - 50.0,
                ]
            })
            .collect();
        
        bencher.iter(|| {
            let mut visible_count = 0;
            
            for (npc_pos, forward, fov, range) in &npcs {
                let half_fov = fov / 2.0;
                let cos_half_fov = half_fov.cos();
                
                for target in &targets {
                    let to_target = [
                        target[0] - npc_pos[0],
                        target[1] - npc_pos[1],
                        target[2] - npc_pos[2],
                    ];
                    
                    let dist_sq = to_target[0] * to_target[0]
                        + to_target[1] * to_target[1]
                        + to_target[2] * to_target[2];
                    
                    if dist_sq > range * range {
                        continue;
                    }
                    
                    let dist = dist_sq.sqrt();
                    if dist < 0.001 {
                        continue;
                    }
                    
                    let normalized = [
                        to_target[0] / dist,
                        to_target[1] / dist,
                        to_target[2] / dist,
                    ];
                    
                    let dot = forward[0] * normalized[0]
                        + forward[1] * normalized[1]
                        + forward[2] * normalized[2];
                    
                    if dot >= cos_half_fov {
                        visible_count += 1;
                    }
                }
            }
            
            std_black_box(visible_count)
        });
    });
    
    // Test 2: Sound propagation
    group.bench_function("sound_propagation_1000", |bencher| {
        let listeners: Vec<[f32; 3]> = (0..100)
            .map(|i| [(i % 10) as f32 * 10.0, 0.0, (i / 10) as f32 * 10.0])
            .collect();
        
        let sounds: Vec<([f32; 3], SoundType, f32)> = (0..1000)
            .map(|i| {
                let pos = [
                    (i % 100) as f32,
                    0.0,
                    (i / 100) as f32,
                ];
                let sound_type = match i % 5 {
                    0 => SoundType::Footsteps,
                    1 => SoundType::Combat,
                    2 => SoundType::Speech,
                    3 => SoundType::Ambient,
                    _ => SoundType::Alert,
                };
                let volume = 50.0 + (i % 50) as f32;
                (pos, sound_type, volume)
            })
            .collect();
        
        bencher.iter(|| {
            let mut heard_count = 0;
            
            for listener in &listeners {
                for (sound_pos, _sound_type, volume) in &sounds {
                    let dist_sq = (listener[0] - sound_pos[0]).powi(2)
                        + (listener[1] - sound_pos[1]).powi(2)
                        + (listener[2] - sound_pos[2]).powi(2);
                    
                    // Inverse square law
                    let attenuation = 1.0 / (1.0 + dist_sq * 0.01);
                    let perceived_volume = volume * attenuation;
                    
                    if perceived_volume > 10.0 {
                        heard_count += 1;
                    }
                }
            }
            
            std_black_box(heard_count)
        });
    });
    
    // Test 3: Threat assessment
    group.bench_function("threat_assessment_500", |bencher| {
        let npcs: Vec<(f32, f32, usize, usize)> = (0..500)
            .map(|i| {
                let health = 50.0 + (i % 50) as f32;
                let weapon_power = 10.0 + (i % 30) as f32;
                let allies_nearby = i % 5;
                let enemies_nearby = i % 8;
                (health, weapon_power, allies_nearby, enemies_nearby)
            })
            .collect();
        
        bencher.iter(|| {
            let threats: Vec<f32> = npcs
                .iter()
                .map(|(health, weapon, allies, enemies)| {
                    let health_factor = 100.0 / health.max(1.0);
                    let weapon_factor = weapon / 50.0;
                    let ally_factor = 1.0 + *allies as f32 * 0.2;
                    let enemy_factor = 1.0 + *enemies as f32 * 0.5;
                    
                    (health_factor * enemy_factor) / (weapon_factor * ally_factor)
                })
                .collect();
            
            let high_threat = threats.iter().filter(|&&t| t > 2.0).count();
            std_black_box(high_threat)
        });
    });
    
    // Test 4: Memory decay
    group.bench_function("memory_decay_2000", |bencher| {
        let mut memories: Vec<Vec<(u64, f32, f32)>> = (0..200)
            .map(|_| {
                (0..100)
                    .map(|j| {
                        let entity_id = j as u64;
                        let last_seen = (100 - j) as f32;
                        let importance = 0.5 + (j % 10) as f32 * 0.05;
                        (entity_id, last_seen, importance)
                    })
                    .collect()
            })
            .collect();
        
        let decay_rate = 0.01f32;
        let forget_threshold = 0.1f32;
        
        bencher.iter(|| {
            let mut forgotten = 0;
            
            for npc_memories in memories.iter_mut() {
                // Decay all memories
                for (_, _, importance) in npc_memories.iter_mut() {
                    *importance *= 1.0 - decay_rate;
                }
                
                // Count and remove forgotten
                let before = npc_memories.len();
                npc_memories.retain(|(_, _, imp)| *imp > forget_threshold);
                forgotten += before - npc_memories.len();
            }
            
            std_black_box(forgotten)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 4: LLM INTEGRATION
// ============================================================================

fn bench_llm_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("npc_adversarial/llm_integration");
    
    // Test 1: Context building
    group.bench_function("context_building_200", |bencher| {
        let npcs: Vec<(NpcId, String, Personality, Vec<String>, HashMap<String, String>)> = (0..200)
            .map(|i| {
                let id = NpcId(i as u64);
                let summary = format!(
                    "NPC {} is {} and {}",
                    i,
                    if i % 2 == 0 { "friendly" } else { "hostile" },
                    if i % 3 == 0 { "cautious" } else { "bold" }
                );
                let personality = Personality::ocean(
                    (i as f32 * 0.005) % 1.0,
                    (i as f32 * 0.007) % 1.0,
                    (i as f32 * 0.003) % 1.0,
                    (i as f32 * 0.009) % 1.0,
                    (i as f32 * 0.004) % 1.0,
                );
                let events: Vec<String> = (0..5)
                    .map(|j| format!("Event {} for NPC {}", j, i))
                    .collect();
                let mut world = HashMap::new();
                world.insert("time".to_string(), "noon".to_string());
                world.insert("weather".to_string(), "clear".to_string());
                (id, summary, personality, events, world)
            })
            .collect();
        
        bencher.iter(|| {
            let contexts: Vec<LlmContext> = npcs
                .iter()
                .map(|(id, summary, _personality, events, world)| {
                    LlmContext {
                        npc_id: *id,
                        personality_summary: summary.clone(),
                        recent_events: events.clone(),
                        world_state: world.clone(),
                        conversation_history: Vec::new(),
                    }
                })
                .collect();
            
            std_black_box(contexts.len())
        });
    });
    
    // Test 2: Prompt formatting
    group.bench_function("prompt_formatting_500", |bencher| {
        let contexts: Vec<LlmContext> = (0..500)
            .map(|i| {
                let mut world = HashMap::new();
                world.insert("location".to_string(), format!("area_{}", i % 10));
                world.insert("threat_level".to_string(), format!("{}", i % 5));
                
                LlmContext {
                    npc_id: NpcId(i as u64),
                    personality_summary: format!("Personality summary for NPC {}", i),
                    recent_events: vec![
                        format!("Event 1: {}", i),
                        format!("Event 2: {}", i + 1),
                    ],
                    world_state: world,
                    conversation_history: vec![
                        format!("Player: Hello NPC {}", i),
                        format!("NPC: Greetings, traveler."),
                    ],
                }
            })
            .collect();
        
        bencher.iter(|| {
            let prompts: Vec<String> = contexts
                .iter()
                .map(|ctx| {
                    let mut prompt = String::with_capacity(1000);
                    
                    prompt.push_str("You are an NPC in a game world.\n");
                    prompt.push_str("Personality: ");
                    prompt.push_str(&ctx.personality_summary);
                    prompt.push_str("\n\nRecent events:\n");
                    
                    for event in &ctx.recent_events {
                        prompt.push_str("- ");
                        prompt.push_str(event);
                        prompt.push('\n');
                    }
                    
                    prompt.push_str("\nWorld state:\n");
                    for (key, value) in &ctx.world_state {
                        prompt.push_str("- ");
                        prompt.push_str(key);
                        prompt.push_str(": ");
                        prompt.push_str(value);
                        prompt.push('\n');
                    }
                    
                    prompt.push_str("\nConversation:\n");
                    for line in &ctx.conversation_history {
                        prompt.push_str(line);
                        prompt.push('\n');
                    }
                    
                    prompt
                })
                .collect();
            
            let total_len: usize = prompts.iter().map(|p| p.len()).sum();
            std_black_box(total_len)
        });
    });
    
    // Test 3: Response parsing
    group.bench_function("response_parsing_1000", |bencher| {
        let responses: Vec<String> = (0..1000)
            .map(|i| {
                format!(
                    "{{\"action\": \"{}\", \"target\": \"entity_{}\", \"dialogue\": \"Response {}\"}}",
                    match i % 4 {
                        0 => "speak",
                        1 => "move",
                        2 => "attack",
                        _ => "wait",
                    },
                    i % 50,
                    i
                )
            })
            .collect();
        
        bencher.iter(|| {
            let parsed: Vec<(String, String, String)> = responses
                .iter()
                .filter_map(|r| {
                    let action = r.split("\"action\": \"")
                        .nth(1)?
                        .split('"')
                        .next()?
                        .to_string();
                    
                    let target = r.split("\"target\": \"")
                        .nth(1)?
                        .split('"')
                        .next()?
                        .to_string();
                    
                    let dialogue = r.split("\"dialogue\": \"")
                        .nth(1)?
                        .split('"')
                        .next()?
                        .to_string();
                    
                    Some((action, target, dialogue))
                })
                .collect();
            
            std_black_box(parsed.len())
        });
    });
    
    // Test 4: Conversation history management
    group.bench_function("conversation_history_500", |bencher| {
        let mut histories: Vec<Vec<String>> = (0..500)
            .map(|_| Vec::with_capacity(100))
            .collect();
        
        let max_history = 50;
        let messages: Vec<String> = (0..5000)
            .map(|i| format!("Message {}", i))
            .collect();
        
        bencher.iter(|| {
            for (i, msg) in messages.iter().enumerate() {
                let history_idx = i % histories.len();
                let history = &mut histories[history_idx];
                
                history.push(msg.clone());
                
                if history.len() > max_history {
                    history.remove(0);
                }
            }
            
            let total_msgs: usize = histories.iter().map(|h| h.len()).sum();
            std_black_box(total_msgs)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 5: RUNTIME SYSTEMS
// ============================================================================

fn bench_runtime_systems(c: &mut Criterion) {
    let mut group = c.benchmark_group("npc_adversarial/runtime_systems");
    
    // Test 1: NPC tick update
    for npc_count in [500, 1000, 2000] {
        group.throughput(Throughput::Elements(npc_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("tick_update", npc_count),
            &npc_count,
            |bencher, &count| {
                let mut npcs: Vec<(BehaviorState, SenseData, f32)> = (0..count)
                    .map(|i| {
                        let state = BehaviorState {
                            current: BehaviorType::Idle,
                            substate: 0,
                            timer: 0.0,
                            memory: HashMap::new(),
                        };
                        let sense = SenseData {
                            visible_entities: Vec::new(),
                            audible_sounds: Vec::new(),
                            smell_traces: Vec::new(),
                            danger_level: (i % 10) as f32 * 0.1,
                        };
                        let cooldown = i as f32 * 0.1;
                        (state, sense, cooldown)
                    })
                    .collect();
                
                let dt = 0.016f32;
                
                bencher.iter(|| {
                    let mut state_changes = 0;
                    
                    for (state, sense, cooldown) in npcs.iter_mut() {
                        *cooldown = (*cooldown - dt).max(0.0);
                        state.timer += dt;
                        
                        // Simple state machine
                        if sense.danger_level > 0.5 && state.current != BehaviorType::Combat {
                            state.current = BehaviorType::Combat;
                            state.timer = 0.0;
                            state_changes += 1;
                        } else if sense.danger_level < 0.2 && state.current == BehaviorType::Combat {
                            state.current = BehaviorType::Idle;
                            state.timer = 0.0;
                            state_changes += 1;
                        }
                    }
                    
                    std_black_box(state_changes)
                });
            },
        );
    }
    
    // Test 2: Action queue processing
    group.bench_function("action_queue_processing_1000", |bencher| {
        let mut queues: Vec<Vec<(String, u64, f32)>> = (0..100)
            .map(|i| {
                (0..10)
                    .map(|j| {
                        let action = format!("action_{}", j);
                        let target = (i * 10 + j) as u64;
                        let duration = 0.5 + j as f32 * 0.1;
                        (action, target, duration)
                    })
                    .collect()
            })
            .collect();
        
        let dt = 0.016f32;
        
        bencher.iter(|| {
            let mut completed = 0;
            
            for queue in queues.iter_mut() {
                if let Some((_, _, duration)) = queue.first_mut() {
                    *duration -= dt;
                    if *duration <= 0.0 {
                        queue.remove(0);
                        completed += 1;
                    }
                }
            }
            
            std_black_box(completed)
        });
    });
    
    // Test 3: Path following
    group.bench_function("path_following_500", |bencher| {
        let mut npcs: Vec<([f32; 3], Vec<[f32; 3]>, f32)> = (0..500)
            .map(|i| {
                let pos = [i as f32 % 100.0, 0.0, (i / 100) as f32];
                let path: Vec<[f32; 3]> = (0..10)
                    .map(|j| [pos[0] + j as f32 * 2.0, 0.0, pos[2] + j as f32])
                    .collect();
                let speed = 5.0 + (i % 10) as f32;
                (pos, path, speed)
            })
            .collect();
        
        let dt = 0.016f32;
        
        bencher.iter(|| {
            let mut reached_waypoints = 0;
            
            for (pos, path, speed) in npcs.iter_mut() {
                if path.is_empty() {
                    continue;
                }
                
                let target = path[0];
                let to_target = [
                    target[0] - pos[0],
                    target[1] - pos[1],
                    target[2] - pos[2],
                ];
                
                let dist_sq = to_target[0] * to_target[0]
                    + to_target[1] * to_target[1]
                    + to_target[2] * to_target[2];
                
                if dist_sq < 1.0 {
                    path.remove(0);
                    reached_waypoints += 1;
                } else {
                    let dist = dist_sq.sqrt();
                    let move_dist = *speed * dt;
                    let factor = move_dist / dist;
                    
                    pos[0] += to_target[0] * factor;
                    pos[1] += to_target[1] * factor;
                    pos[2] += to_target[2] * factor;
                }
            }
            
            std_black_box(reached_waypoints)
        });
    });
    
    // Test 4: LOD management
    group.bench_function("lod_management_2000", |bencher| {
        let camera_pos = [500.0f32, 0.0, 500.0];
        let npcs: Vec<([f32; 3], u8)> = (0..2000)
            .map(|i| {
                let pos = [
                    (i % 100) as f32 * 10.0,
                    0.0,
                    (i / 100) as f32 * 10.0,
                ];
                let current_lod = (i % 4) as u8;
                (pos, current_lod)
            })
            .collect();
        
        let lod_distances = [50.0f32, 100.0, 200.0, 500.0]; // LOD 0-3
        
        bencher.iter(|| {
            let lods: Vec<u8> = npcs
                .iter()
                .map(|(pos, _)| {
                    let dist_sq = (pos[0] - camera_pos[0]).powi(2)
                        + (pos[1] - camera_pos[1]).powi(2)
                        + (pos[2] - camera_pos[2]).powi(2);
                    
                    let dist = dist_sq.sqrt();
                    
                    lod_distances
                        .iter()
                        .position(|&d| dist < d)
                        .unwrap_or(lod_distances.len()) as u8
                })
                .collect();
            
            let lod0_count = lods.iter().filter(|&&l| l == 0).count();
            std_black_box(lod0_count)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 6: DIALOGUE SYSTEMS
// ============================================================================

fn bench_dialogue_systems(c: &mut Criterion) {
    let mut group = c.benchmark_group("npc_adversarial/dialogue_systems");
    
    // Test 1: Emotion blending
    group.bench_function("emotion_blending_1000", |bencher| {
        let emotions: Vec<(Emotion, f32, Emotion, f32)> = (0..1000)
            .map(|i| {
                let e1 = match i % 6 {
                    0 => Emotion::Neutral,
                    1 => Emotion::Happy,
                    2 => Emotion::Sad,
                    3 => Emotion::Angry,
                    4 => Emotion::Fearful,
                    _ => Emotion::Surprised,
                };
                let w1 = (i % 10) as f32 * 0.1;
                let e2 = match (i + 1) % 6 {
                    0 => Emotion::Neutral,
                    1 => Emotion::Happy,
                    2 => Emotion::Sad,
                    3 => Emotion::Angry,
                    4 => Emotion::Fearful,
                    _ => Emotion::Surprised,
                };
                let w2 = 1.0 - w1;
                (e1, w1, e2, w2)
            })
            .collect();
        
        bencher.iter(|| {
            let blended: Vec<[f32; 6]> = emotions
                .iter()
                .map(|(e1, w1, e2, w2)| {
                    let mut result = [0.0f32; 6];
                    
                    // Convert emotions to vectors and blend
                    let idx1 = match e1 {
                        Emotion::Neutral => 0,
                        Emotion::Happy => 1,
                        Emotion::Sad => 2,
                        Emotion::Angry => 3,
                        Emotion::Fearful => 4,
                        Emotion::Surprised => 5,
                    };
                    let idx2 = match e2 {
                        Emotion::Neutral => 0,
                        Emotion::Happy => 1,
                        Emotion::Sad => 2,
                        Emotion::Angry => 3,
                        Emotion::Fearful => 4,
                        Emotion::Surprised => 5,
                    };
                    
                    result[idx1] = *w1;
                    result[idx2] += *w2;
                    
                    result
                })
                .collect();
            
            std_black_box(blended.len())
        });
    });
    
    // Test 2: Dialogue tree traversal
    group.bench_function("dialogue_tree_traversal_500", |bencher| {
        // Simple dialogue tree: (node_id, text, options: [(choice_text, next_node)])
        let trees: Vec<Vec<(u32, String, Vec<(String, u32)>)>> = (0..500)
            .map(|i| {
                (0..10)
                    .map(|j| {
                        let node_id = j as u32;
                        let text = format!("Node {} text for tree {}", j, i);
                        let options: Vec<(String, u32)> = if j < 9 {
                            vec![
                                (format!("Choice A"), (j + 1) as u32),
                                (format!("Choice B"), (j + 1) as u32),
                            ]
                        } else {
                            vec![]
                        };
                        (node_id, text, options)
                    })
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            let final_nodes: Vec<u32> = trees
                .iter()
                .map(|tree| {
                    let mut current_node = 0u32;
                    
                    loop {
                        let node = tree.iter().find(|(id, _, _)| *id == current_node);
                        
                        if let Some((_, _, options)) = node {
                            if options.is_empty() {
                                break;
                            }
                            // Always pick first option for determinism
                            current_node = options[0].1;
                        } else {
                            break;
                        }
                    }
                    
                    current_node
                })
                .collect();
            
            std_black_box(final_nodes.len())
        });
    });
    
    // Test 3: Keyword matching
    group.bench_function("keyword_matching_2000", |bencher| {
        let keywords = vec![
            "quest", "help", "gold", "weapon", "armor", "potion",
            "merchant", "guard", "king", "dragon", "treasure", "map",
        ];
        
        let dialogues: Vec<String> = (0..2000)
            .map(|i| {
                format!(
                    "The {} mentioned something about a {} near the {} castle",
                    keywords[i % keywords.len()],
                    keywords[(i + 1) % keywords.len()],
                    keywords[(i + 2) % keywords.len()]
                )
            })
            .collect();
        
        bencher.iter(|| {
            let matches: Vec<Vec<&str>> = dialogues
                .iter()
                .map(|d| {
                    let lower = d.to_lowercase();
                    keywords
                        .iter()
                        .filter(|&k| lower.contains(k))
                        .copied()
                        .collect()
                })
                .collect();
            
            let total_matches: usize = matches.iter().map(|m| m.len()).sum();
            std_black_box(total_matches)
        });
    });
    
    // Test 4: Response selection
    group.bench_function("response_selection_500", |bencher| {
        let response_pools: Vec<Vec<(String, f32, Vec<String>)>> = (0..500)
            .map(|i| {
                (0..20)
                    .map(|j| {
                        let text = format!("Response {} for pool {}", j, i);
                        let base_score = j as f32 * 0.05;
                        let tags: Vec<String> = vec![
                            format!("tag_{}", j % 5),
                            format!("mood_{}", j % 3),
                        ];
                        (text, base_score, tags)
                    })
                    .collect()
            })
            .collect();
        
        let context_tags: Vec<String> = vec!["tag_2".to_string(), "mood_1".to_string()];
        
        bencher.iter(|| {
            let selected: Vec<&String> = response_pools
                .iter()
                .map(|pool| {
                    pool.iter()
                        .max_by(|(_, score_a, tags_a), (_, score_b, tags_b)| {
                            let bonus_a = tags_a.iter().filter(|t| context_tags.contains(t)).count() as f32 * 0.5;
                            let bonus_b = tags_b.iter().filter(|t| context_tags.contains(t)).count() as f32 * 0.5;
                            
                            (score_a + bonus_a)
                                .partial_cmp(&(score_b + bonus_b))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(text, _, _)| text)
                        .unwrap_or(&String::new())
                })
                .collect();
            
            std_black_box(selected.len())
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_behavior_systems,
    bench_profile_management,
    bench_sense_systems,
    bench_llm_integration,
    bench_runtime_systems,
    bench_dialogue_systems,
);

criterion_main!(benches);
