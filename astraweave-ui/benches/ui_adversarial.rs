//! Adversarial UI Benchmarks
//!
//! Professional-grade stress testing for UI system:
//! menu state, HUD rendering, health animations, damage numbers, settings.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-ui API)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuState {
    MainMenu,
    PauseMenu,
    SettingsMenu,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone)]
struct GraphicsSettings {
    resolution: (u32, u32),
    quality: QualityPreset,
    fullscreen: bool,
    vsync: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            quality: QualityPreset::High,
            fullscreen: false,
            vsync: true,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AudioSettings {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    voice_volume: f32,
    master_mute: bool,
    music_mute: bool,
    sfx_mute: bool,
    voice_mute: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 100.0,
            music_volume: 80.0,
            sfx_volume: 90.0,
            voice_volume: 100.0,
            master_mute: false,
            music_mute: false,
            sfx_mute: false,
            voice_mute: false,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ControlsSettings {
    move_forward: String,
    move_backward: String,
    move_left: String,
    move_right: String,
    jump: String,
    crouch: String,
    sprint: String,
    attack: String,
    interact: String,
    inventory: String,
    mouse_sensitivity: f32,
    invert_y: bool,
}

impl Default for ControlsSettings {
    fn default() -> Self {
        Self {
            move_forward: "W".to_string(),
            move_backward: "S".to_string(),
            move_left: "A".to_string(),
            move_right: "D".to_string(),
            jump: "Space".to_string(),
            crouch: "LControl".to_string(),
            sprint: "LShift".to_string(),
            attack: "Mouse0".to_string(),
            interact: "E".to_string(),
            inventory: "Tab".to_string(),
            mouse_sensitivity: 1.0,
            invert_y: false,
        }
    }
}

// Health animation
mod easing {
    pub fn ease_out_cubic(t: f32) -> f32 {
        let t = t - 1.0;
        t * t * t + 1.0
    }

    pub fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
}

#[derive(Clone, Debug)]
struct HealthAnimation {
    current_visual: f32,
    target: f32,
    animation_time: f32,
    duration: f32,
    flash_timer: f32,
    flash_duration: f32,
}

impl HealthAnimation {
    fn new(health: f32) -> Self {
        Self {
            current_visual: health,
            target: health,
            animation_time: 0.0,
            duration: 0.4,
            flash_timer: 0.0,
            flash_duration: 0.2,
        }
    }

    fn set_target(&mut self, new_health: f32) {
        self.target = new_health;
        self.animation_time = 0.0;
        if new_health < self.current_visual {
            self.flash_timer = self.flash_duration;
        }
    }

    fn update(&mut self, dt: f32) {
        if self.flash_timer > 0.0 {
            self.flash_timer = (self.flash_timer - dt).max(0.0);
        }

        if (self.current_visual - self.target).abs() > 0.01 {
            self.animation_time += dt;
            let t = (self.animation_time / self.duration).min(1.0);

            let eased_t = if self.target > self.current_visual {
                easing::ease_in_out_quad(t)
            } else {
                easing::ease_out_cubic(t)
            };

            self.current_visual =
                self.current_visual + (self.target - self.current_visual) * eased_t;

            if t >= 1.0 {
                self.current_visual = self.target;
            }
        }
    }

    fn visual_health(&self) -> f32 {
        self.current_visual
    }

    fn flash_alpha(&self) -> f32 {
        if self.flash_timer > 0.0 {
            (self.flash_timer / self.flash_duration) * 0.6
        } else {
            0.0
        }
    }
}

// Damage numbers
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
enum DamageType {
    Normal,
    Critical,
    SelfDamage,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct DamageNumber {
    value: i32,
    spawn_time: f32,
    world_pos: (f32, f32, f32),
    damage_type: DamageType,
    velocity_x: f32,
    velocity_y: f32,
    gravity: f32,
    shake_amplitude: f32,
    shake_frequency: f32,
}

impl DamageNumber {
    fn new(value: i32, spawn_time: f32, world_pos: (f32, f32, f32), damage_type: DamageType) -> Self {
        let hash = ((spawn_time * 1000.0) as u32).wrapping_mul(2654435761);
        let random_val = (hash as f32 / u32::MAX as f32) - 0.5;
        let velocity_x = random_val * 60.0;
        let velocity_y = -80.0;
        let gravity = 150.0;

        let shake_amplitude = match damage_type {
            DamageType::Critical => 0.175,
            _ => 0.087,
        };
        let shake_frequency = 15.0;

        Self {
            value,
            spawn_time,
            world_pos,
            damage_type,
            velocity_x,
            velocity_y,
            gravity,
            shake_amplitude,
            shake_frequency,
        }
    }

    fn calculate_offset(&self, age: f32) -> (f32, f32) {
        let offset_x = self.velocity_x * age;
        let offset_y = self.velocity_y * age + 0.5 * self.gravity * age * age;
        (offset_x, offset_y)
    }

    fn calculate_shake(&self, age: f32) -> f32 {
        let damping = (-age * 5.0).exp();
        self.shake_amplitude * (age * self.shake_frequency * std::f32::consts::TAU).sin() * damping
    }
}

// Quest system
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Objective {
    id: u32,
    description: String,
    completed: bool,
    progress: Option<(u32, u32)>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Quest {
    id: u32,
    title: String,
    description: String,
    objectives: Vec<Objective>,
}

impl Quest {
    fn completion(&self) -> f32 {
        if self.objectives.is_empty() {
            return 0.0;
        }
        let completed = self.objectives.iter().filter(|o| o.completed).count();
        completed as f32 / self.objectives.len() as f32
    }
}

// ============================================================================
// CATEGORY 1: MENU STATE TRANSITIONS
// ============================================================================

fn bench_menu_state_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_adversarial/menu_state_transitions");

    // Test 1: Rapid state changes
    group.bench_function("rapid_state_changes_100", |bencher| {
        let mut current_state = MenuState::None;

        bencher.iter(|| {
            for _ in 0..100 {
                current_state = match current_state {
                    MenuState::None => MenuState::MainMenu,
                    MenuState::MainMenu => MenuState::PauseMenu,
                    MenuState::PauseMenu => MenuState::SettingsMenu,
                    MenuState::SettingsMenu => MenuState::None,
                };
            }
            std_black_box(current_state)
        });
    });

    // Test 2: State validation
    group.bench_function("state_validation_checks", |bencher| {
        let states = [
            MenuState::None,
            MenuState::MainMenu,
            MenuState::PauseMenu,
            MenuState::SettingsMenu,
        ];

        bencher.iter(|| {
            let mut valid_transitions = 0;
            for state in &states {
                // Check valid transitions from each state
                let can_pause = matches!(state, MenuState::None);
                let can_settings =
                    matches!(state, MenuState::MainMenu | MenuState::PauseMenu);
                let can_resume = matches!(state, MenuState::PauseMenu);

                if can_pause || can_settings || can_resume {
                    valid_transitions += 1;
                }
            }
            std_black_box(valid_transitions)
        });
    });

    // Test 3: State history tracking
    group.bench_function("state_history_50", |bencher| {
        let mut history: Vec<MenuState> = Vec::with_capacity(50);
        let mut current = MenuState::MainMenu;

        bencher.iter(|| {
            history.clear();
            for i in 0..50 {
                history.push(current);
                current = match i % 4 {
                    0 => MenuState::MainMenu,
                    1 => MenuState::PauseMenu,
                    2 => MenuState::SettingsMenu,
                    _ => MenuState::None,
                };
            }
            std_black_box(history.len())
        });
    });

    // Test 4: State equality checks
    group.bench_function("equality_checks_1000", |bencher| {
        let current = MenuState::PauseMenu;
        let targets = [
            MenuState::None,
            MenuState::MainMenu,
            MenuState::PauseMenu,
            MenuState::SettingsMenu,
        ];

        bencher.iter(|| {
            let mut matches = 0;
            for _ in 0..250 {
                for target in &targets {
                    if current == *target {
                        matches += 1;
                    }
                }
            }
            std_black_box(matches)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: HEALTH ANIMATION STRESS
// ============================================================================

fn bench_health_animation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_adversarial/health_animation");

    // Test 1: Single animation update
    group.bench_function("single_update", |bencher| {
        let mut animation = HealthAnimation::new(100.0);
        animation.set_target(50.0);

        bencher.iter(|| {
            animation.update(0.016); // 60 FPS delta
            std_black_box(animation.visual_health())
        });
    });

    // Test 2: Rapid target changes
    group.bench_function("rapid_target_changes_100", |bencher| {
        let mut animation = HealthAnimation::new(100.0);

        bencher.iter(|| {
            for i in 0..100 {
                animation.set_target((i % 100) as f32);
                animation.update(0.016);
            }
            std_black_box(animation.visual_health())
        });
    });

    // Test 3: Multiple animations (enemy health bars)
    for count in [10, 50, 100] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("multiple_animations", count),
            &count,
            |bencher, &count| {
                let mut animations: Vec<HealthAnimation> =
                    (0..count).map(|_| HealthAnimation::new(100.0)).collect();

                for (i, anim) in animations.iter_mut().enumerate() {
                    anim.set_target((50 + i as i32 % 50) as f32);
                }

                bencher.iter(|| {
                    for anim in &mut animations {
                        anim.update(0.016);
                    }
                    std_black_box(animations.len())
                });
            },
        );
    }

    // Test 4: Flash alpha calculation
    group.bench_function("flash_alpha_100", |bencher| {
        let mut animations: Vec<HealthAnimation> =
            (0..100).map(|_| HealthAnimation::new(100.0)).collect();

        for anim in &mut animations {
            anim.set_target(50.0); // Trigger flash
        }

        bencher.iter(|| {
            let total_flash: f32 = animations.iter().map(|a| a.flash_alpha()).sum();
            std_black_box(total_flash)
        });
    });

    // Test 5: Easing function stress
    group.bench_function("easing_calculations_1000", |bencher| {
        bencher.iter(|| {
            let mut sum = 0.0f32;
            for i in 0..1000 {
                let t = i as f32 / 1000.0;
                sum += easing::ease_out_cubic(t);
                sum += easing::ease_in_out_quad(t);
            }
            std_black_box(sum)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: DAMAGE NUMBERS
// ============================================================================

fn bench_damage_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_adversarial/damage_numbers");

    // Test 1: Spawn single damage number
    group.bench_function("spawn_single", |bencher| {
        let mut spawn_time = 0.0f32;

        bencher.iter(|| {
            spawn_time += 0.1;
            let damage = DamageNumber::new(100, spawn_time, (0.0, 0.0, 0.0), DamageType::Normal);
            std_black_box(damage.value)
        });
    });

    // Test 2: Spawn burst (many at once)
    group.bench_function("spawn_burst_50", |bencher| {
        let base_time = 1.0f32;

        bencher.iter(|| {
            let numbers: Vec<DamageNumber> = (0..50)
                .map(|i| {
                    let pos = (i as f32 * 0.1, 0.0, 0.0);
                    DamageNumber::new(
                        100 + i as i32 * 10,
                        base_time + i as f32 * 0.001,
                        pos,
                        if i % 5 == 0 {
                            DamageType::Critical
                        } else {
                            DamageType::Normal
                        },
                    )
                })
                .collect();

            std_black_box(numbers.len())
        });
    });

    // Test 3: Arc motion calculation
    group.bench_function("arc_motion_100", |bencher| {
        let numbers: Vec<DamageNumber> = (0..100)
            .map(|i| {
                DamageNumber::new(100, i as f32 * 0.1, (0.0, 0.0, 0.0), DamageType::Normal)
            })
            .collect();

        bencher.iter(|| {
            let offsets: Vec<(f32, f32)> =
                numbers.iter().map(|n| n.calculate_offset(0.5)).collect();
            std_black_box(offsets.len())
        });
    });

    // Test 4: Shake calculation
    group.bench_function("shake_calculation_100", |bencher| {
        let numbers: Vec<DamageNumber> = (0..100)
            .map(|i| {
                DamageNumber::new(100, i as f32 * 0.1, (0.0, 0.0, 0.0), DamageType::Critical)
            })
            .collect();

        bencher.iter(|| {
            let shakes: Vec<f32> = numbers.iter().map(|n| n.calculate_shake(0.3)).collect();
            std_black_box(shakes.len())
        });
    });

    // Test 5: Lifetime filtering (remove expired)
    group.bench_function("lifetime_filter_200", |bencher| {
        let current_time = 5.0f32;
        let lifetime = 2.0f32;

        let numbers: Vec<DamageNumber> = (0..200)
            .map(|i| {
                DamageNumber::new(
                    100,
                    i as f32 * 0.05, // Spread spawn times
                    (0.0, 0.0, 0.0),
                    DamageType::Normal,
                )
            })
            .collect();

        bencher.iter(|| {
            let active: Vec<_> = numbers
                .iter()
                .filter(|n| (current_time - n.spawn_time) < lifetime)
                .collect();

            std_black_box(active.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: SETTINGS VALIDATION
// ============================================================================

fn bench_settings_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_adversarial/settings_validation");

    // Test 1: Graphics settings validation
    group.bench_function("graphics_validation", |bencher| {
        let settings = GraphicsSettings::default();
        let valid_resolutions = [(1920, 1080), (2560, 1440), (3840, 2160), (1280, 720)];

        bencher.iter(|| {
            let is_valid = valid_resolutions.contains(&settings.resolution)
                && settings.quality == QualityPreset::High;
            std_black_box(is_valid)
        });
    });

    // Test 2: Audio settings validation
    group.bench_function("audio_validation", |bencher| {
        let settings = AudioSettings::default();

        bencher.iter(|| {
            let is_valid = (0.0..=100.0).contains(&settings.master_volume)
                && (0.0..=100.0).contains(&settings.music_volume)
                && (0.0..=100.0).contains(&settings.sfx_volume)
                && (0.0..=100.0).contains(&settings.voice_volume);
            std_black_box(is_valid)
        });
    });

    // Test 3: Controls validation
    group.bench_function("controls_validation", |bencher| {
        let settings = ControlsSettings::default();
        let valid_keys = [
            "W", "A", "S", "D", "E", "F", "Q", "R", "T", "Space", "LControl", "LShift", "Tab",
            "Mouse0", "Mouse1", "Mouse2",
        ];

        bencher.iter(|| {
            let all_valid = valid_keys.contains(&settings.move_forward.as_str())
                && valid_keys.contains(&settings.move_backward.as_str())
                && valid_keys.contains(&settings.move_left.as_str())
                && valid_keys.contains(&settings.move_right.as_str())
                && (0.1..=5.0).contains(&settings.mouse_sensitivity);
            std_black_box(all_valid)
        });
    });

    // Test 4: Duplicate key detection
    group.bench_function("duplicate_key_detection", |bencher| {
        let settings = ControlsSettings::default();

        bencher.iter(|| {
            let bindings = [
                &settings.move_forward,
                &settings.move_backward,
                &settings.move_left,
                &settings.move_right,
                &settings.jump,
                &settings.crouch,
                &settings.sprint,
                &settings.attack,
                &settings.interact,
                &settings.inventory,
            ];

            let mut unique = std::collections::HashSet::new();
            let has_duplicates = !bindings.iter().all(|b| unique.insert(*b));
            std_black_box(has_duplicates)
        });
    });

    // Test 5: Settings serialization (simulated)
    group.bench_function("settings_serialization_sim", |bencher| {
        let graphics = GraphicsSettings::default();
        let audio = AudioSettings::default();
        let controls = ControlsSettings::default();

        bencher.iter(|| {
            // Simulate serialization work
            let combined = format!(
                "{{resolution:{:?},quality:{:?},fullscreen:{},vsync:{},master:{},music:{},sfx:{},voice:{},forward:{},backward:{},sensitivity:{}}}",
                graphics.resolution,
                graphics.quality,
                graphics.fullscreen,
                graphics.vsync,
                audio.master_volume,
                audio.music_volume,
                audio.sfx_volume,
                audio.voice_volume,
                controls.move_forward,
                controls.move_backward,
                controls.mouse_sensitivity
            );
            std_black_box(combined.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: QUEST TRACKER
// ============================================================================

fn bench_quest_tracker(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_adversarial/quest_tracker");

    // Test 1: Quest completion calculation
    group.bench_function("completion_calculation_10", |bencher| {
        let quests: Vec<Quest> = (0..10)
            .map(|q| Quest {
                id: q,
                title: format!("Quest {}", q),
                description: format!("Description for quest {}", q),
                objectives: (0..5)
                    .map(|o| Objective {
                        id: o,
                        description: format!("Objective {}.{}", q, o),
                        completed: o < 3, // 3/5 completed
                        progress: Some((o.min(3), 5)),
                    })
                    .collect(),
            })
            .collect();

        bencher.iter(|| {
            let completions: Vec<f32> = quests.iter().map(|q| q.completion()).collect();
            std_black_box(completions.iter().sum::<f32>())
        });
    });

    // Test 2: Active quests filtering
    group.bench_function("active_quest_filter_50", |bencher| {
        let quests: Vec<Quest> = (0..50)
            .map(|q| Quest {
                id: q,
                title: format!("Quest {}", q),
                description: "".to_string(),
                objectives: vec![Objective {
                    id: 0,
                    description: "".to_string(),
                    completed: q % 2 == 0, // Half completed
                    progress: None,
                }],
            })
            .collect();

        bencher.iter(|| {
            let active: Vec<_> = quests
                .iter()
                .filter(|q| q.completion() < 1.0)
                .collect();
            std_black_box(active.len())
        });
    });

    // Test 3: Objective progress update
    group.bench_function("objective_progress_update_100", |bencher| {
        let mut quests: Vec<Quest> = (0..20)
            .map(|q| Quest {
                id: q,
                title: format!("Quest {}", q),
                description: "".to_string(),
                objectives: (0..5)
                    .map(|o| Objective {
                        id: o,
                        description: format!("Collect {} items", 5),
                        completed: false,
                        progress: Some((0, 5)),
                    })
                    .collect(),
            })
            .collect();

        bencher.iter(|| {
            let mut updates = 0;
            for quest in &mut quests {
                for obj in &mut quest.objectives {
                    if let Some((current, total)) = &mut obj.progress {
                        if *current < *total {
                            *current += 1;
                            updates += 1;
                            if *current >= *total {
                                obj.completed = true;
                            }
                        }
                    }
                }
            }
            std_black_box(updates)
        });

        // Reset for next iteration
        for quest in &mut quests {
            for obj in &mut quest.objectives {
                if let Some((current, _)) = &mut obj.progress {
                    *current = 0;
                }
                obj.completed = false;
            }
        }
    });

    // Test 4: Quest lookup by ID
    group.bench_function("quest_lookup_100", |bencher| {
        let quests: HashMap<u32, Quest> = (0..100)
            .map(|q| {
                (
                    q,
                    Quest {
                        id: q,
                        title: format!("Quest {}", q),
                        description: "".to_string(),
                        objectives: vec![],
                    },
                )
            })
            .collect();

        bencher.iter(|| {
            let mut found = 0;
            for id in [5, 25, 50, 75, 95] {
                if quests.contains_key(&id) {
                    found += 1;
                }
            }
            std_black_box(found)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: UI LAYOUT CALCULATIONS
// ============================================================================

fn bench_layout_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_adversarial/layout_calculations");

    // Test 1: Screen-space position calculation
    group.bench_function("screen_space_position_100", |bencher| {
        let world_positions: Vec<(f32, f32, f32)> =
            (0..100).map(|i| (i as f32, 0.0, i as f32 * 2.0)).collect();

        let screen_size = (1920.0f32, 1080.0f32);
        let view_proj = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]; // Identity for simplicity

        bencher.iter(|| {
            let screen_positions: Vec<(f32, f32)> = world_positions
                .iter()
                .map(|(x, y, z)| {
                    // Simplified world-to-screen
                    let clip_x = x * view_proj[0][0];
                    let clip_y = y * view_proj[1][1];
                    let ndc_x = clip_x / (z.max(0.1));
                    let ndc_y = clip_y / (z.max(0.1));
                    let screen_x = (ndc_x + 1.0) * 0.5 * screen_size.0;
                    let screen_y = (1.0 - ndc_y) * 0.5 * screen_size.1;
                    (screen_x, screen_y)
                })
                .collect();

            std_black_box(screen_positions.len())
        });
    });

    // Test 2: Health bar width calculation
    group.bench_function("health_bar_width_100", |bencher| {
        let health_data: Vec<(f32, f32)> = (0..100)
            .map(|i| ((i % 100) as f32, 100.0)) // (current, max)
            .collect();

        let bar_max_width = 200.0f32;

        bencher.iter(|| {
            let widths: Vec<f32> = health_data
                .iter()
                .map(|(current, max)| (current / max) * bar_max_width)
                .collect();

            std_black_box(widths.iter().sum::<f32>())
        });
    });

    // Test 3: Element visibility culling
    group.bench_function("visibility_culling_200", |bencher| {
        let elements: Vec<(f32, f32, f32, f32)> = (0..200)
            .map(|i| {
                let x = (i % 20) as f32 * 100.0;
                let y = (i / 20) as f32 * 100.0;
                (x, y, 50.0, 20.0) // x, y, width, height
            })
            .collect();

        let viewport = (0.0f32, 0.0f32, 1920.0f32, 1080.0f32); // x, y, w, h

        bencher.iter(|| {
            let visible: Vec<_> = elements
                .iter()
                .filter(|(x, y, w, h)| {
                    x + w > viewport.0
                        && *x < viewport.0 + viewport.2
                        && y + h > viewport.1
                        && *y < viewport.1 + viewport.3
                })
                .collect();

            std_black_box(visible.len())
        });
    });

    // Test 4: Text size estimation
    group.bench_function("text_size_estimation_50", |bencher| {
        let texts: Vec<&str> = (0..50)
            .map(|i| match i % 5 {
                0 => "Short",
                1 => "Medium length text",
                2 => "This is a longer piece of text for the UI",
                3 => "Quest: Defeat the dragon and save the village",
                _ => "A",
            })
            .collect();

        let char_width = 8.0f32;
        let line_height = 16.0f32;

        bencher.iter(|| {
            let sizes: Vec<(f32, f32)> = texts
                .iter()
                .map(|text| {
                    let width = text.len() as f32 * char_width;
                    let height = line_height;
                    (width, height)
                })
                .collect();

            std_black_box(sizes.len())
        });
    });

    // Test 5: Minimap coordinate transformation
    group.bench_function("minimap_transform_100", |bencher| {
        let world_positions: Vec<(f32, f32)> = (0..100)
            .map(|i| ((i % 50) as f32 * 10.0, (i / 50) as f32 * 10.0))
            .collect();

        let minimap_size = 150.0f32;
        let world_range = 500.0f32;
        let player_pos = (250.0f32, 250.0f32);

        bencher.iter(|| {
            let minimap_positions: Vec<(f32, f32)> = world_positions
                .iter()
                .map(|(wx, wy)| {
                    let relative_x = wx - player_pos.0;
                    let relative_y = wy - player_pos.1;
                    let normalized_x = relative_x / world_range;
                    let normalized_y = relative_y / world_range;
                    let map_x = minimap_size / 2.0 + normalized_x * minimap_size / 2.0;
                    let map_y = minimap_size / 2.0 + normalized_y * minimap_size / 2.0;
                    (map_x, map_y)
                })
                .collect();

            std_black_box(minimap_positions.len())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_menu_state_transitions,
    bench_health_animation,
    bench_damage_numbers,
    bench_settings_validation,
    bench_quest_tracker,
    bench_layout_calculations,
);

criterion_main!(benches);
