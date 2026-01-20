use astraweave_memory::persona::{CompanionProfile, Episode, Fact, Persona, Skill};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test persona with typical fields
fn create_test_persona() -> Persona {
    Persona {
        name: "TestCompanion".to_string(),
        likes: vec!["strategy".to_string(), "tactics".to_string()],
        dislikes: vec!["chaos".to_string()],
        tone: "professional".to_string(),
        risk: "calculated".to_string(),
        humor: "dry".to_string(),
        voice: "tactical".to_string(),
        backstory: "A skilled tactical AI companion".to_string(),
        goals: vec!["assist player".to_string(), "optimize strategy".to_string()],
    }
}

/// Create a test fact
fn create_test_fact(key: &str, value: &str) -> Fact {
    Fact {
        k: key.to_string(),
        v: value.to_string(),
        t: "gameplay".to_string(),
    }
}

/// Create a test skill
fn create_test_skill(name: &str, level: u8) -> Skill {
    Skill {
        name: name.to_string(),
        level,
        notes: format!("Skilled in {}", name),
    }
}

/// Create a test episode
fn create_test_episode(title: &str) -> Episode {
    Episode {
        title: title.to_string(),
        summary: format!("Summary of {}", title),
        tags: vec!["action".to_string(), "gameplay".to_string()],
        ts: "2025-10-29T12:00:00Z".to_string(),
    }
}

/// Create a comprehensive companion profile
fn create_comprehensive_profile(
    num_facts: usize,
    num_skills: usize,
    num_episodes: usize,
) -> CompanionProfile {
    let mut profile = CompanionProfile::new_default();
    profile.persona = create_test_persona();

    // Add facts
    for i in 0..num_facts {
        profile.facts.push(create_test_fact(
            &format!("fact_{}", i),
            &format!("value_{}", i),
        ));
    }

    // Add skills
    for i in 0..num_skills {
        profile.skills.push(create_test_skill(
            &format!("skill_{}", i),
            ((i % 10) + 1) as u8,
        ));
    }

    // Add episodes
    for i in 0..num_episodes {
        profile
            .episodes
            .push(create_test_episode(&format!("Episode {}", i)));
    }

    // Add player preferences
    profile.player_prefs = serde_json::json!({
        "combat_style": "tactical",
        "difficulty": "hard",
        "hints_enabled": false
    });

    profile
}

// ============================================================================
// Benchmark 1: Persona Creation
// ============================================================================

fn bench_persona_creation(c: &mut Criterion) {
    c.bench_function("persona_creation", |b| {
        b.iter(|| {
            let persona = create_test_persona();
            black_box(persona)
        })
    });
}

fn bench_persona_default(c: &mut Criterion) {
    c.bench_function("persona_default", |b| {
        b.iter(|| {
            let persona = Persona::default();
            black_box(persona)
        })
    });
}

// ============================================================================
// Benchmark 2: Component Creation (Facts, Skills, Episodes)
// ============================================================================

fn bench_fact_creation(c: &mut Criterion) {
    c.bench_function("fact_creation", |b| {
        b.iter(|| {
            let fact = create_test_fact("test_key", "test_value");
            black_box(fact)
        })
    });
}

fn bench_skill_creation(c: &mut Criterion) {
    c.bench_function("skill_creation", |b| {
        b.iter(|| {
            let skill = create_test_skill("tactical_analysis", 8);
            black_box(skill)
        })
    });
}

fn bench_episode_creation(c: &mut Criterion) {
    c.bench_function("episode_creation", |b| {
        b.iter(|| {
            let episode = create_test_episode("First Mission");
            black_box(episode)
        })
    });
}

// ============================================================================
// Benchmark 3: Companion Profile Creation & Operations
// ============================================================================

fn bench_profile_creation_default(c: &mut Criterion) {
    c.bench_function("profile_creation_default", |b| {
        b.iter(|| {
            let profile = CompanionProfile::new_default();
            black_box(profile)
        })
    });
}

fn bench_profile_creation_comprehensive(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_creation_comprehensive");

    for (facts, skills, episodes) in [(10, 5, 5), (50, 10, 10), (100, 20, 20)] {
        group.bench_with_input(
            BenchmarkId::new("profile", format!("f{}_s{}_e{}", facts, skills, episodes)),
            &(facts, skills, episodes),
            |b, &(num_facts, num_skills, num_episodes)| {
                b.iter(|| {
                    let profile = create_comprehensive_profile(num_facts, num_skills, num_episodes);
                    black_box(profile)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 4: Profile Operations
// ============================================================================

fn bench_profile_clone(c: &mut Criterion) {
    let profile = create_comprehensive_profile(50, 10, 10);

    c.bench_function("profile_clone", |b| {
        b.iter(|| {
            let cloned = profile.clone();
            black_box(cloned)
        })
    });
}

fn bench_profile_sign(c: &mut Criterion) {
    c.bench_function("profile_sign", |b| {
        b.iter_with_setup(
            || create_comprehensive_profile(20, 5, 5),
            |mut profile| {
                profile.sign();
                black_box(profile)
            },
        )
    });
}

fn bench_profile_verify(c: &mut Criterion) {
    let mut profile = create_comprehensive_profile(20, 5, 5);
    profile.sign();

    c.bench_function("profile_verify", |b| {
        b.iter(|| {
            let valid = profile.verify();
            black_box(valid)
        })
    });
}

fn bench_profile_distill(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_distill");

    for num_episodes in [10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_episodes),
            &num_episodes,
            |b, &num_episodes| {
                b.iter_with_setup(
                    || create_comprehensive_profile(0, 0, num_episodes),
                    |mut profile| {
                        profile.distill();
                        black_box(profile)
                    },
                )
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 5: Serialization (JSON)
// ============================================================================

fn bench_profile_serialize_json(c: &mut Criterion) {
    let profile = create_comprehensive_profile(50, 10, 10);

    c.bench_function("profile_serialize_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&profile).unwrap();
            black_box(json)
        })
    });
}

fn bench_profile_deserialize_json(c: &mut Criterion) {
    let profile = create_comprehensive_profile(50, 10, 10);
    let json = serde_json::to_string(&profile).unwrap();

    c.bench_function("profile_deserialize_json", |b| {
        b.iter(|| {
            let profile: CompanionProfile = serde_json::from_str(&json).unwrap();
            black_box(profile)
        })
    });
}

// ============================================================================
// Benchmark 6: Profile Modifications
// ============================================================================

fn bench_profile_add_facts(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_add_facts");

    for count in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_with_setup(
                CompanionProfile::new_default,
                |mut profile| {
                    for i in 0..count {
                        profile.facts.push(create_test_fact(
                            &format!("key_{}", i),
                            &format!("value_{}", i),
                        ));
                    }
                    black_box(profile)
                },
            )
        });
    }

    group.finish();
}

fn bench_profile_add_skills(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_add_skills");

    for count in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_with_setup(
                CompanionProfile::new_default,
                |mut profile| {
                    for i in 0..count {
                        profile
                            .skills
                            .push(create_test_skill(&format!("skill_{}", i), (i % 10) as u8));
                    }
                    black_box(profile)
                },
            )
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark Registration
// ============================================================================

criterion_group!(
    benches,
    bench_persona_creation,
    bench_persona_default,
    bench_fact_creation,
    bench_skill_creation,
    bench_episode_creation,
    bench_profile_creation_default,
    bench_profile_creation_comprehensive,
    bench_profile_clone,
    bench_profile_sign,
    bench_profile_verify,
    bench_profile_distill,
    bench_profile_serialize_json,
    bench_profile_deserialize_json,
    bench_profile_add_facts,
    bench_profile_add_skills,
);

criterion_main!(benches);
