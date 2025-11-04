//! UI Benchmarks
//!
//! Measures performance of UI rendering and widget operations:
//! - Menu rendering (main menu, pause, settings)
//! - HUD rendering (health bars, damage numbers, minimap)
//! - Widget construction and layout
//! - Settings persistence (save/load TOML)
//!
//! Performance targets (from Phase 8.1):
//! - Menu render: <2 ms per frame (120 FPS capability)
//! - HUD render: <1 ms per frame (minimal overhead)
//! - Widget construction: <100 µs (instant responsiveness)
//! - Settings save/load: <10 ms (non-blocking)

use astraweave_ui::{
    load_settings, save_settings, AudioSettings, ControlsSettings, DamageNumber, DamageType,
    DialogueChoice, DialogueNode, EnemyData, EnemyFaction, GraphicsSettings, HudManager, HudState,
    MenuAction, MenuManager, MenuState, Objective, PlayerStats, PoiMarker, PoiType, Quest,
    TooltipData,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;

// ============================================================================
// Benchmark 1: Menu Manager Operations
// ============================================================================

fn bench_menu_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("menu_manager");

    // Benchmark: Menu creation
    group.bench_function("menu_creation", |b| {
        b.iter(|| black_box(MenuManager::new()))
    });

    // Benchmark: Menu state transitions
    group.bench_function("state_transitions", |b| {
        let mut manager = MenuManager::new();
        let mut state_idx = 0;
        let states = [
            MenuState::MainMenu,
            MenuState::Paused,
            MenuState::Settings,
            MenuState::InGame,
        ];

        b.iter(|| {
            manager.set_state(black_box(states[state_idx % 4]));
            state_idx += 1;
        })
    });

    // Benchmark: Settings menu navigation
    group.bench_function("settings_navigation", |b| {
        let mut manager = MenuManager::new();
        manager.set_state(MenuState::Settings);

        b.iter(|| {
            black_box(manager.get_settings_state());
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 2: Graphics Settings Operations
// ============================================================================

fn bench_graphics_settings(c: &mut Criterion) {
    let mut group = c.benchmark_group("graphics_settings");

    // Benchmark: Settings creation
    group.bench_function("settings_creation", |b| {
        b.iter(|| black_box(GraphicsSettings::default()))
    });

    // Benchmark: Resolution updates
    group.bench_function("resolution_update", |b| {
        let mut settings = GraphicsSettings::default();
        let mut toggle = true;

        b.iter(|| {
            toggle = !toggle;
            settings.resolution_x = if toggle { 1920 } else { 2560 };
            settings.resolution_y = if toggle { 1080 } else { 1440 };
            black_box(&settings)
        })
    });

    // Benchmark: Quality preset changes
    group.bench_function("quality_preset_change", |b| {
        let mut settings = GraphicsSettings::default();
        let mut idx = 0;

        b.iter(|| {
            idx = (idx + 1) % 4; // Cycle through presets
            black_box(&mut settings);
            black_box(idx)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 3: Audio Settings Operations
// ============================================================================

fn bench_audio_settings(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_settings");

    // Benchmark: Settings creation
    group.bench_function("settings_creation", |b| {
        b.iter(|| black_box(AudioSettings::default()))
    });

    // Benchmark: Volume adjustments
    group.bench_function("volume_adjustment", |b| {
        let mut settings = AudioSettings::default();
        let mut volume = 0.0f32;

        b.iter(|| {
            volume = (volume + 0.05) % 1.0;
            settings.master_volume = volume;
            settings.music_volume = volume * 0.8;
            settings.sfx_volume = volume * 0.9;
            settings.voice_volume = volume * 1.0;
            black_box(&settings)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 4: Controls Settings Operations
// ============================================================================

fn bench_controls_settings(c: &mut Criterion) {
    let mut group = c.benchmark_group("controls_settings");

    // Benchmark: Settings creation
    group.bench_function("settings_creation", |b| {
        b.iter(|| black_box(ControlsSettings::default()))
    });

    // Benchmark: Key binding updates
    group.bench_function("key_binding_update", |b| {
        let mut settings = ControlsSettings::default();
        let mut toggle = true;

        b.iter(|| {
            toggle = !toggle;
            settings.key_forward = if toggle {
                "W".to_string()
            } else {
                "Z".to_string()
            };
            black_box(&settings)
        })
    });

    // Benchmark: Mouse sensitivity adjustment
    group.bench_function("mouse_sensitivity_adjustment", |b| {
        let mut settings = ControlsSettings::default();
        let mut sensitivity = 0.5f32;

        b.iter(|| {
            sensitivity = (sensitivity + 0.1) % 2.0;
            settings.mouse_sensitivity = sensitivity;
            black_box(&settings)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 5: HUD Manager Operations
// ============================================================================

fn bench_hud_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("hud_manager");

    // Benchmark: HUD creation
    group.bench_function("hud_creation", |b| b.iter(|| black_box(HudManager::new())));

    // Benchmark: Player stats update
    group.bench_function("player_stats_update", |b| {
        let mut hud = HudManager::new();
        let mut hp = 100;

        b.iter(|| {
            hp = (hp + 10) % 100;
            let stats = PlayerStats {
                health: hp,
                max_health: 100,
                mana: 50,
                max_mana: 100,
                stamina: 75,
                max_stamina: 100,
            };
            hud.update_player_stats(stats);
            black_box(&hud)
        })
    });

    // Benchmark: Enemy data updates (scalable)
    for enemy_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("enemy_data_update", enemy_count),
            enemy_count,
            |b, &count| {
                let mut hud = HudManager::new();

                b.iter(|| {
                    let mut enemies = vec![];
                    for i in 0..count {
                        enemies.push(EnemyData {
                            id: i as u64,
                            position: (100.0 + i as f32 * 10.0, 50.0, 0.0).into(),
                            health: 80,
                            max_health: 100,
                            faction: if i % 2 == 0 {
                                EnemyFaction::Hostile
                            } else {
                                EnemyFaction::Neutral
                            },
                            is_elite: i % 5 == 0,
                        });
                    }
                    hud.update_enemies(enemies);
                    black_box(&hud)
                })
            },
        );
    }

    // Benchmark: Damage number spawning
    group.bench_function("damage_number_spawn", |b| {
        let mut hud = HudManager::new();
        let mut dmg = 10;

        b.iter(|| {
            dmg = (dmg + 5) % 100;
            let damage_num = DamageNumber {
                value: dmg,
                damage_type: if dmg % 20 == 0 {
                    DamageType::Critical
                } else {
                    DamageType::Normal
                },
                position: (100.0, 100.0).into(),
                time_alive: 0.0,
            };
            hud.spawn_damage_number(damage_num);
            black_box(&hud)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 6: Quest System Operations
// ============================================================================

fn bench_quest_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("quest_system");

    // Benchmark: Quest creation
    group.bench_function("quest_creation", |b| {
        b.iter(|| {
            black_box(Quest {
                id: 1,
                title: "Test Quest".to_string(),
                description: "Complete the benchmark".to_string(),
                objectives: vec![Objective {
                    description: "Run benchmark".to_string(),
                    current: 0,
                    required: 100,
                    completed: false,
                }],
            })
        })
    });

    // Benchmark: Quest progress update
    group.bench_function("quest_progress_update", |b| {
        let mut quest = Quest {
            id: 1,
            title: "Test Quest".to_string(),
            description: "Complete the benchmark".to_string(),
            objectives: vec![Objective {
                description: "Run benchmark".to_string(),
                current: 0,
                required: 100,
                completed: false,
            }],
        };

        b.iter(|| {
            quest.objectives[0].current = (quest.objectives[0].current + 1) % 100;
            quest.objectives[0].completed =
                quest.objectives[0].current >= quest.objectives[0].required;
            black_box(&quest)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 7: POI Marker Operations
// ============================================================================

fn bench_poi_markers(c: &mut Criterion) {
    let mut group = c.benchmark_group("poi_markers");

    // Benchmark: POI creation (scalable)
    for poi_count in [1, 5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("poi_creation", poi_count),
            poi_count,
            |b, &count| {
                b.iter(|| {
                    let mut pois = vec![];
                    for i in 0..count {
                        pois.push(PoiMarker {
                            id: i as u64,
                            world_position: (i as f32 * 10.0, 0.0, 0.0).into(),
                            poi_type: if i % 3 == 0 {
                                PoiType::Objective
                            } else {
                                PoiType::Collectible
                            },
                            label: format!("POI {}", i),
                        });
                    }
                    black_box(pois)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 8: Dialogue System Operations
// ============================================================================

fn bench_dialogue_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("dialogue_system");

    // Benchmark: Dialogue node creation
    group.bench_function("dialogue_node_creation", |b| {
        b.iter(|| {
            black_box(DialogueNode {
                id: 1,
                speaker: "NPC".to_string(),
                text: "Hello, traveler!".to_string(),
                choices: vec![
                    DialogueChoice {
                        text: "Hello".to_string(),
                        next_node_id: Some(2),
                    },
                    DialogueChoice {
                        text: "Goodbye".to_string(),
                        next_node_id: None,
                    },
                ],
            })
        })
    });

    // Benchmark: Dialogue tree traversal (4-node tree)
    group.bench_function("dialogue_tree_traversal", |b| {
        let nodes = vec![
            DialogueNode {
                id: 1,
                speaker: "NPC".to_string(),
                text: "Hello!".to_string(),
                choices: vec![DialogueChoice {
                    text: "Hi".to_string(),
                    next_node_id: Some(2),
                }],
            },
            DialogueNode {
                id: 2,
                speaker: "NPC".to_string(),
                text: "How are you?".to_string(),
                choices: vec![DialogueChoice {
                    text: "Good".to_string(),
                    next_node_id: Some(3),
                }],
            },
            DialogueNode {
                id: 3,
                speaker: "NPC".to_string(),
                text: "Great!".to_string(),
                choices: vec![DialogueChoice {
                    text: "Bye".to_string(),
                    next_node_id: None,
                }],
            },
        ];

        b.iter(|| {
            let mut current_id = 1;
            for _ in 0..3 {
                if let Some(node) = nodes.iter().find(|n| n.id == current_id) {
                    if let Some(choice) = node.choices.first() {
                        if let Some(next_id) = choice.next_node_id {
                            current_id = next_id;
                        }
                    }
                }
            }
            black_box(current_id)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 9: Tooltip Operations
// ============================================================================

fn bench_tooltip_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tooltip_operations");

    // Benchmark: Tooltip creation
    group.bench_function("tooltip_creation", |b| {
        b.iter(|| {
            black_box(TooltipData {
                title: "Item Name".to_string(),
                description: "A powerful item".to_string(),
                stats: vec![
                    ("Damage".to_string(), "+10".to_string()),
                    ("Speed".to_string(), "+5".to_string()),
                ],
            })
        })
    });

    // Benchmark: Tooltip with many stats (scalable)
    for stat_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("tooltip_with_stats", stat_count),
            stat_count,
            |b, &count| {
                b.iter(|| {
                    let mut stats = vec![];
                    for i in 0..count {
                        stats.push((format!("Stat {}", i), format!("+{}", i * 5)));
                    }
                    black_box(TooltipData {
                        title: "Item".to_string(),
                        description: "Description".to_string(),
                        stats,
                    })
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 10: Settings Persistence (Save/Load)
// ============================================================================

fn bench_settings_persistence(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings_persistence");

    // Benchmark: Settings save
    group.bench_function("settings_save", |b| {
        let graphics = GraphicsSettings::default();
        let audio = AudioSettings::default();
        let controls = ControlsSettings::default();

        b.iter(|| {
            black_box(
                save_settings(
                    black_box(&graphics),
                    black_box(&audio),
                    black_box(&controls),
                )
                .ok(),
            )
        })
    });

    // Benchmark: Settings load
    group.bench_function("settings_load", |b| {
        // Ensure settings file exists
        let graphics = GraphicsSettings::default();
        let audio = AudioSettings::default();
        let controls = ControlsSettings::default();
        save_settings(&graphics, &audio, &controls).ok();

        b.iter(|| black_box(load_settings().ok()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_menu_manager,
    bench_graphics_settings,
    bench_audio_settings,
    bench_controls_settings,
    bench_hud_manager,
    bench_quest_system,
    bench_poi_markers,
    bench_dialogue_system,
    bench_tooltip_operations,
    bench_settings_persistence,
);

criterion_main!(benches);
