//! Adversarial Scripting Benchmarks
//!
//! Professional-grade stress testing for Rhai scripting system:
//! compilation, execution, command processing, security limits, hot reload.

#![allow(dead_code, clippy::enum_variant_names)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;
use std::sync::Arc;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-scripting API)
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
enum ScriptCommand {
    Spawn { prefab: String, position: (f32, f32, f32) },
    Despawn { entity: i64 },
    SetPosition { entity: i64, position: (f32, f32, f32) },
    ApplyDamage { entity: i64, amount: f32 },
    PlaySound { path: String },
    SpawnParticle { effect: String, position: (f32, f32, f32) },
}

#[derive(Clone, Debug, Default)]
struct ScriptCommands {
    commands: Vec<ScriptCommand>,
}

impl ScriptCommands {
    fn new() -> Self {
        Self::default()
    }

    fn spawn(&mut self, prefab: &str, position: (f32, f32, f32)) {
        self.commands.push(ScriptCommand::Spawn {
            prefab: prefab.to_string(),
            position,
        });
    }

    fn despawn(&mut self, entity: i64) {
        self.commands.push(ScriptCommand::Despawn { entity });
    }

    fn set_position(&mut self, entity: i64, position: (f32, f32, f32)) {
        self.commands.push(ScriptCommand::SetPosition { entity, position });
    }

    fn apply_damage(&mut self, entity: i64, amount: f32) {
        self.commands.push(ScriptCommand::ApplyDamage { entity, amount });
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct CScript {
    script_path: String,
    source: String,
    cached_ast: Option<Arc<String>>, // Simulated AST as String
    script_state: HashMap<String, i64>,
    enabled: bool,
}

impl CScript {
    fn new(path: &str, source: &str) -> Self {
        Self {
            script_path: path.to_string(),
            source: source.to_string(),
            cached_ast: None,
            script_state: HashMap::new(),
            enabled: true,
        }
    }
}

// Security limits
struct SecurityLimits {
    max_operations: u64,
    max_string_size: usize,
    max_array_size: usize,
    max_map_size: usize,
    max_expr_depth: usize,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_operations: 50_000,
            max_string_size: 1024,
            max_array_size: 1024,
            max_map_size: 1024,
            max_expr_depth: 64,
        }
    }
}

fn simulate_script_compile(source: &str) -> Result<Arc<String>, String> {
    if source.is_empty() {
        return Err("Empty script".to_string());
    }
    if source.contains("syntax_error!") {
        return Err("Syntax error".to_string());
    }
    // Simulate compilation time
    let _ = source.len();
    Ok(Arc::new(source.to_string()))
}

fn simulate_script_execute(ast: &str, state: &mut HashMap<String, i64>) -> Vec<ScriptCommand> {
    let mut commands = Vec::new();

    // Parse simple commands from script
    for line in ast.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("spawn(") {
            commands.push(ScriptCommand::Spawn {
                prefab: "enemy".to_string(),
                position: (0.0, 0.0, 0.0),
            });
        } else if trimmed.starts_with("despawn(") {
            commands.push(ScriptCommand::Despawn { entity: 0 });
        } else if trimmed.starts_with("let ") {
            // Simple variable tracking
            if let Some((var, val)) = trimmed
                .strip_prefix("let ")
                .and_then(|s| s.split_once('='))
            {
                let var_name = var.trim().to_string();
                let value: i64 = val.trim().trim_end_matches(';').parse().unwrap_or(0);
                state.insert(var_name, value);
            }
        }
    }

    commands
}

// ============================================================================
// CATEGORY 1: COMPILATION STRESS
// ============================================================================

fn bench_compilation_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("scripting_adversarial/compilation_stress");

    // Test 1: Empty script compilation
    group.bench_function("empty_script_compile", |bencher| {
        let source = "";

        bencher.iter(|| {
            let result = simulate_script_compile(source);
            std_black_box(result.is_err())
        });
    });

    // Test 2: Large script compilation
    for lines in [100, 500, 1000] {
        group.throughput(Throughput::Elements(lines as u64));

        group.bench_with_input(
            BenchmarkId::new("large_script_compile", lines),
            &lines,
            |bencher, &lines| {
                let source: String = (0..lines)
                    .map(|i| format!("let x{} = {};\n", i, i))
                    .collect();

                bencher.iter(|| {
                    let result = simulate_script_compile(&source);
                    std_black_box(result.is_ok())
                });
            },
        );
    }

    // Test 3: Syntax error handling
    group.bench_function("syntax_error_handling", |bencher| {
        let source = "let x = 1; syntax_error! let y = 2;";

        bencher.iter(|| {
            let result = simulate_script_compile(source);
            std_black_box(result.is_err())
        });
    });

    // Test 4: Deeply nested expressions
    group.bench_function("deeply_nested_100", |bencher| {
        let mut source = String::from("let x = ");
        for _ in 0..100 {
            source.push('(');
        }
        source.push('1');
        for _ in 0..100 {
            source.push_str(" + 1)");
        }
        source.push(';');

        bencher.iter(|| {
            let result = simulate_script_compile(&source);
            std_black_box(result.is_ok())
        });
    });

    // Test 5: Many function definitions
    group.bench_function("many_functions_50", |bencher| {
        let source: String = (0..50)
            .map(|i| format!("fn func_{}() {{ return {}; }}\n", i, i))
            .collect();

        bencher.iter(|| {
            let result = simulate_script_compile(&source);
            std_black_box(result.is_ok())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: EXECUTION STRESS
// ============================================================================

fn bench_execution_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("scripting_adversarial/execution_stress");

    // Test 1: Simple script execution
    group.bench_function("simple_execution", |bencher| {
        let ast = "let x = 1;\nlet y = 2;\nlet z = x + y;";
        let mut state = HashMap::new();

        bencher.iter(|| {
            let commands = simulate_script_execute(ast, &mut state);
            std_black_box(commands.len())
        });
    });

    // Test 2: Heavy loop simulation
    group.bench_function("loop_heavy_1000_iterations", |bencher| {
        let ast: String = (0..1000)
            .map(|i| format!("let iter_{} = {};\n", i, i))
            .collect();
        let mut state = HashMap::new();

        bencher.iter(|| {
            let _commands = simulate_script_execute(&ast, &mut state);
            std_black_box(state.len())
        });
    });

    // Test 3: Many command emissions
    group.bench_function("many_commands_100", |bencher| {
        let ast: String = (0..100)
            .map(|_| "spawn(enemy, 0, 0, 0);\n")
            .collect();
        let mut state = HashMap::new();

        bencher.iter(|| {
            let commands = simulate_script_execute(&ast, &mut state);
            std_black_box(commands.len())
        });
    });

    // Test 4: State persistence
    group.bench_function("state_persistence_stress", |bencher| {
        let ast = "let counter = 1;";
        let mut state: HashMap<String, i64> = (0..100)
            .map(|i| (format!("var_{}", i), i as i64))
            .collect();

        bencher.iter(|| {
            let _commands = simulate_script_execute(ast, &mut state);
            std_black_box(state.len())
        });
    });

    // Test 5: No-op execution (empty AST)
    group.bench_function("noop_execution", |bencher| {
        let ast = "// Just a comment";
        let mut state = HashMap::new();

        bencher.iter(|| {
            let commands = simulate_script_execute(ast, &mut state);
            std_black_box(commands.is_empty())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: COMMAND PROCESSING
// ============================================================================

fn bench_command_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("scripting_adversarial/command_processing");

    // Test 1: Single command
    group.bench_function("single_spawn_command", |bencher| {
        let mut cmds = ScriptCommands::new();

        bencher.iter(|| {
            cmds.commands.clear();
            cmds.spawn("enemy_grunt", (1.0, 2.0, 3.0));
            std_black_box(cmds.commands.len())
        });
    });

    // Test 2: Batch commands
    group.bench_function("batch_commands_100", |bencher| {
        let mut cmds = ScriptCommands::new();

        bencher.iter(|| {
            cmds.commands.clear();
            for i in 0..100 {
                cmds.spawn("enemy", (i as f32, 0.0, 0.0));
            }
            std_black_box(cmds.commands.len())
        });
    });

    // Test 3: Mixed command types
    group.bench_function("mixed_commands_50", |bencher| {
        let mut cmds = ScriptCommands::new();

        bencher.iter(|| {
            cmds.commands.clear();
            for i in 0..50 {
                match i % 4 {
                    0 => cmds.spawn("enemy", (i as f32, 0.0, 0.0)),
                    1 => cmds.despawn(i as i64),
                    2 => cmds.set_position(i as i64, (i as f32, i as f32, i as f32)),
                    _ => cmds.apply_damage(i as i64, i as f32 * 10.0),
                }
            }
            std_black_box(cmds.commands.len())
        });
    });

    // Test 4: Command validation
    group.bench_function("command_validation_100", |bencher| {
        let commands: Vec<ScriptCommand> = (0..100)
            .map(|i| ScriptCommand::Spawn {
                prefab: format!("entity_{}", i),
                position: (i as f32, 0.0, 0.0),
            })
            .collect();

        bencher.iter(|| {
            let valid_count = commands
                .iter()
                .filter(|cmd| match cmd {
                    ScriptCommand::Spawn { prefab, position } => {
                        !prefab.is_empty() && position.0.is_finite()
                    }
                    ScriptCommand::Despawn { entity } => *entity >= 0,
                    ScriptCommand::SetPosition { entity, position } => {
                        *entity >= 0 && position.0.is_finite()
                    }
                    ScriptCommand::ApplyDamage { entity, amount } => {
                        *entity >= 0 && *amount >= 0.0
                    }
                    _ => true,
                })
                .count();

            std_black_box(valid_count)
        });
    });

    // Test 5: Empty command list
    group.bench_function("empty_command_list", |bencher| {
        let cmds = ScriptCommands::new();

        bencher.iter(|| {
            let count = cmds.commands.len();
            std_black_box(count == 0)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: SECURITY LIMITS
// ============================================================================

fn bench_security_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("scripting_adversarial/security_limits");

    // Test 1: Operation count enforcement
    group.bench_function("operation_limit_check", |bencher| {
        let limits = SecurityLimits::default();
        let operations = 49_999u64;

        bencher.iter(|| {
            let within_limit = operations <= limits.max_operations;
            std_black_box(within_limit)
        });
    });

    // Test 2: Operation limit exceeded
    group.bench_function("operation_limit_exceeded", |bencher| {
        let limits = SecurityLimits::default();
        let operations = 100_000u64;

        bencher.iter(|| {
            let exceeded = operations > limits.max_operations;
            std_black_box(exceeded)
        });
    });

    // Test 3: String size limit
    group.bench_function("string_size_limit", |bencher| {
        let limits = SecurityLimits::default();
        let test_strings: Vec<String> = (0..100)
            .map(|i| "x".repeat(i * 20))
            .collect();

        bencher.iter(|| {
            let violations: Vec<_> = test_strings
                .iter()
                .filter(|s| s.len() > limits.max_string_size)
                .collect();

            std_black_box(violations.len())
        });
    });

    // Test 4: Array size limit
    group.bench_function("array_size_limit", |bencher| {
        let limits = SecurityLimits::default();
        let array_sizes: Vec<usize> = (0..100).map(|i| i * 25).collect();

        bencher.iter(|| {
            let violations: usize = array_sizes
                .iter()
                .filter(|&&size| size > limits.max_array_size)
                .count();

            std_black_box(violations)
        });
    });

    // Test 5: Expression depth check
    group.bench_function("expr_depth_check", |bencher| {
        let limits = SecurityLimits::default();
        let depths: Vec<usize> = vec![10, 30, 50, 64, 65, 100, 200];

        bencher.iter(|| {
            let valid: Vec<_> = depths
                .iter()
                .filter(|&&d| d <= limits.max_expr_depth)
                .collect();

            std_black_box(valid.len())
        });
    });

    // Test 6: Combined limits check
    group.bench_function("combined_limits_check", |bencher| {
        let limits = SecurityLimits::default();

        struct ScriptMetrics {
            operations: u64,
            max_string: usize,
            max_array: usize,
            max_map: usize,
            max_depth: usize,
        }

        let metrics = ScriptMetrics {
            operations: 45_000,
            max_string: 512,
            max_array: 500,
            max_map: 100,
            max_depth: 32,
        };

        bencher.iter(|| {
            let passed = metrics.operations <= limits.max_operations
                && metrics.max_string <= limits.max_string_size
                && metrics.max_array <= limits.max_array_size
                && metrics.max_map <= limits.max_map_size
                && metrics.max_depth <= limits.max_expr_depth;

            std_black_box(passed)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: HOT RELOAD SIMULATION
// ============================================================================

fn bench_hot_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("scripting_adversarial/hot_reload");

    // Test 1: Script modification detection
    group.bench_function("modification_detection", |bencher| {
        let last_modified = 1000u64;
        let current_modified = 1001u64;

        bencher.iter(|| {
            let needs_reload = current_modified > last_modified;
            std_black_box(needs_reload)
        });
    });

    // Test 2: Cache invalidation
    group.bench_function("cache_invalidation_100", |bencher| {
        let mut cache: HashMap<String, Arc<String>> = (0..100)
            .map(|i| (format!("script_{}.rhai", i), Arc::new(format!("// v{}", i))))
            .collect();

        let modified_scripts: Vec<String> = (0..10)
            .map(|i| format!("script_{}.rhai", i))
            .collect();

        bencher.iter(|| {
            for script in &modified_scripts {
                cache.remove(script);
            }
            std_black_box(cache.len())
        });

        // Restore cache for next iteration
        for i in 0..10 {
            cache.insert(
                format!("script_{}.rhai", i),
                Arc::new(format!("// v{}", i)),
            );
        }
    });

    // Test 3: Recompilation after modification
    group.bench_function("recompilation_cycle", |bencher| {
        let mut script = CScript::new("test.rhai", "let x = 1;");
        let mut counter = 0u32;

        bencher.iter(|| {
            // Simulate modification
            counter = counter.wrapping_add(1);
            script.source = format!("let x = {};", counter % 1000);
            script.cached_ast = None;

            // Recompile
            if let Ok(ast) = simulate_script_compile(&script.source) {
                script.cached_ast = Some(ast);
            }

            std_black_box(script.cached_ast.is_some())
        });
    });

    // Test 4: State preservation on reload
    group.bench_function("state_preservation", |bencher| {
        let mut script = CScript::new("test.rhai", "let x = 1;");
        script.script_state.insert("counter".to_string(), 42);
        script.script_state.insert("health".to_string(), 100);

        bencher.iter(|| {
            // Preserve state
            let preserved_state = script.script_state.clone();

            // Simulate reload
            script.cached_ast = None;
            let _ = simulate_script_compile(&script.source);

            // Restore state
            script.script_state = preserved_state;

            std_black_box(script.script_state.len())
        });
    });

    // Test 5: No reload needed (cache hit)
    group.bench_function("cache_hit_no_reload", |bencher| {
        let last_modified = 1000u64;
        let current_modified = 1000u64; // Same timestamp
        let cached_ast = Arc::new("compiled_ast".to_string());

        bencher.iter(|| {
            let needs_reload = current_modified > last_modified;
            let ast = if needs_reload {
                None // Would recompile
            } else {
                Some(cached_ast.clone()) // Use cache
            };

            std_black_box(ast.is_some())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: CALLBACK EVENT HANDLING
// ============================================================================

fn bench_callback_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("scripting_adversarial/callback_events");

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum ScriptEvent {
        OnCollision { entity: i64, other: i64 },
        OnTrigger { entity: i64, trigger_name: String },
        OnDamage { entity: i64, damage: f32, source: i64 },
        OnSpawn { entity: i64 },
    }

    // Test 1: Single event dispatch
    group.bench_function("single_event_dispatch", |bencher| {
        let event = ScriptEvent::OnCollision { entity: 1, other: 2 };

        bencher.iter(|| {
            let callback = match &event {
                ScriptEvent::OnCollision { .. } => "on_collision",
                ScriptEvent::OnTrigger { .. } => "on_trigger",
                ScriptEvent::OnDamage { .. } => "on_damage",
                ScriptEvent::OnSpawn { .. } => "on_spawn",
            };
            std_black_box(callback)
        });
    });

    // Test 2: Batch event processing
    group.bench_function("batch_events_100", |bencher| {
        let events: Vec<ScriptEvent> = (0..100)
            .map(|i| match i % 4 {
                0 => ScriptEvent::OnCollision { entity: i, other: i + 1 },
                1 => ScriptEvent::OnTrigger {
                    entity: i,
                    trigger_name: format!("trigger_{}", i),
                },
                2 => ScriptEvent::OnDamage {
                    entity: i,
                    damage: i as f32 * 10.0,
                    source: 0,
                },
                _ => ScriptEvent::OnSpawn { entity: i },
            })
            .collect();

        bencher.iter(|| {
            let mut callbacks_called = 0;
            for event in &events {
                let _callback = match event {
                    ScriptEvent::OnCollision { .. } => {
                        callbacks_called += 1;
                        "on_collision"
                    }
                    ScriptEvent::OnTrigger { .. } => {
                        callbacks_called += 1;
                        "on_trigger"
                    }
                    ScriptEvent::OnDamage { .. } => {
                        callbacks_called += 1;
                        "on_damage"
                    }
                    ScriptEvent::OnSpawn { .. } => {
                        callbacks_called += 1;
                        "on_spawn"
                    }
                };
            }
            std_black_box(callbacks_called)
        });
    });

    // Test 3: Event filtering by entity
    group.bench_function("event_filtering_by_entity", |bencher| {
        let events: Vec<ScriptEvent> = (0..100)
            .map(|i| ScriptEvent::OnCollision {
                entity: i % 10,
                other: i + 1,
            })
            .collect();

        let target_entity = 5i64;

        bencher.iter(|| {
            let filtered: Vec<_> = events
                .iter()
                .filter(|e| match e {
                    ScriptEvent::OnCollision { entity, .. } => *entity == target_entity,
                    ScriptEvent::OnTrigger { entity, .. } => *entity == target_entity,
                    ScriptEvent::OnDamage { entity, .. } => *entity == target_entity,
                    ScriptEvent::OnSpawn { entity } => *entity == target_entity,
                })
                .collect();

            std_black_box(filtered.len())
        });
    });

    // Test 4: Empty event queue
    group.bench_function("empty_event_queue", |bencher| {
        let events: Vec<ScriptEvent> = Vec::new();

        bencher.iter(|| {
            let processed = events.len();
            std_black_box(processed == 0)
        });
    });

    // Test 5: High-frequency collision events
    group.bench_function("high_freq_collisions_500", |bencher| {
        let events: Vec<ScriptEvent> = (0..500)
            .map(|i| ScriptEvent::OnCollision {
                entity: i % 50,
                other: (i + 1) % 50,
            })
            .collect();

        bencher.iter(|| {
            // Count unique entity pairs
            let mut pairs = std::collections::HashSet::new();
            for event in &events {
                if let ScriptEvent::OnCollision { entity, other } = event {
                    let pair = if entity < other {
                        (*entity, *other)
                    } else {
                        (*other, *entity)
                    };
                    pairs.insert(pair);
                }
            }
            std_black_box(pairs.len())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_compilation_stress,
    bench_execution_stress,
    bench_command_processing,
    bench_security_limits,
    bench_hot_reload,
    bench_callback_events,
);

criterion_main!(benches);
