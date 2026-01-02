//! Adversarial Author/Scripting Benchmarks
//!
//! Stress testing for Rhai scripting, map generation, and configuration parsing.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-author API)
// ============================================================================

#[derive(Clone, Debug)]
struct MapMeta {
    width: u32,
    height: u32,
    enemy_count: u32,
    difficulty: f32,
    biome: String,
    spawn_zones: Vec<SpawnZone>,
}

#[derive(Clone, Debug)]
struct SpawnZone {
    x: f32,
    y: f32,
    radius: f32,
    enemy_types: Vec<String>,
    max_spawns: u32,
}

#[derive(Clone, Debug)]
struct DirectorBudget {
    total_enemies: u32,
    total_loot_value: u32,
    boss_encounters: u32,
    miniboss_count: u32,
    difficulty_curve: Vec<f32>,
}

#[derive(Clone, Debug)]
struct ScriptContext {
    variables: HashMap<String, ScriptValue>,
    functions: Vec<String>,
    execution_time_ms: f32,
}

#[derive(Clone, Debug)]
enum ScriptValue {
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Map(HashMap<String, ScriptValue>),
}

fn generate_map_meta(width: u32, height: u32) -> MapMeta {
    MapMeta {
        width,
        height,
        enemy_count: width * height / 100,
        difficulty: 0.5,
        biome: "forest".to_string(),
        spawn_zones: (0..10)
            .map(|i| SpawnZone {
                x: (i * 10) as f32,
                y: (i * 10) as f32,
                radius: 5.0,
                enemy_types: vec!["goblin".to_string(), "orc".to_string()],
                max_spawns: 5,
            })
            .collect(),
    }
}

fn parse_script_expression(expr: &str) -> Result<ScriptValue, String> {
    // Simplified expression parser
    let trimmed = expr.trim();
    if let Ok(i) = trimmed.parse::<i64>() {
        Ok(ScriptValue::Integer(i))
    } else if let Ok(f) = trimmed.parse::<f64>() {
        Ok(ScriptValue::Float(f))
    } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
        Ok(ScriptValue::String(trimmed[1..trimmed.len() - 1].to_string()))
    } else {
        Err(format!("Cannot parse: {}", expr))
    }
}

fn validate_script(script: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Check for dangerous patterns
    if script.contains("eval(") {
        errors.push("Dangerous: eval() not allowed".to_string());
    }
    if script.contains("import") {
        errors.push("Dangerous: import not allowed in sandbox".to_string());
    }
    if script.contains("while true") || script.contains("loop {") {
        errors.push("Warning: Potential infinite loop".to_string());
    }

    // Check brace matching
    let open_braces = script.chars().filter(|&c| c == '{').count();
    let close_braces = script.chars().filter(|&c| c == '}').count();
    if open_braces != close_braces {
        errors.push(format!(
            "Mismatched braces: {} open, {} close",
            open_braces, close_braces
        ));
    }

    errors
}

fn execute_script_simulation(script: &str, context: &mut ScriptContext) -> Result<ScriptValue, String> {
    // Simulate script execution with some basic operations
    let lines: Vec<&str> = script.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        // Simulate variable assignment
        if let Some(eq_pos) = trimmed.find('=') {
            let var_name = trimmed[..eq_pos].trim().to_string();
            let value_str = trimmed[eq_pos + 1..].trim();
            if let Ok(value) = parse_script_expression(value_str) {
                context.variables.insert(var_name, value);
            }
        }
    }

    Ok(ScriptValue::Integer(0))
}

// ============================================================================
// CATEGORY 1: SCRIPT PARSING
// ============================================================================

fn bench_script_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("author_adversarial/script_parsing");

    // Test 1: Simple expressions
    group.bench_function("parse_expressions_1000", |bencher| {
        let expressions: Vec<String> = (0..1000)
            .map(|i| match i % 4 {
                0 => format!("{}", i),
                1 => format!("{}.5", i),
                2 => format!("\"string_{}\"", i),
                _ => format!("{} + {}", i, i + 1),
            })
            .collect();

        bencher.iter(|| {
            let results: Vec<_> = expressions
                .iter()
                .map(|e| parse_script_expression(e))
                .collect();
            std_black_box(results.len())
        });
    });

    // Test 2: Script validation
    group.bench_function("validate_scripts_100", |bencher| {
        let scripts: Vec<String> = (0..100)
            .map(|i| {
                format!(
                    r#"
                    let x = {};
                    let y = x * 2;
                    fn calculate() {{
                        return x + y;
                    }}
                    "#,
                    i
                )
            })
            .collect();

        bencher.iter(|| {
            let errors: Vec<_> = scripts.iter().map(|s| validate_script(s)).collect();
            let total_errors: usize = errors.iter().map(|e| e.len()).sum();
            std_black_box(total_errors)
        });
    });

    // Test 3: Malicious script detection
    group.bench_function("malicious_detection_50", |bencher| {
        let scripts: Vec<String> = (0..50)
            .map(|i| {
                match i % 5 {
                    0 => "let x = 1; let y = 2;".to_string(), // Safe
                    1 => "eval(user_input);".to_string(), // Dangerous
                    2 => "import std::fs;".to_string(), // Dangerous
                    3 => "while true { }".to_string(), // Dangerous
                    _ => "fn safe() { return 42; }".to_string(), // Safe
                }
            })
            .collect();

        bencher.iter(|| {
            let dangerous_count = scripts
                .iter()
                .filter(|s| !validate_script(s).is_empty())
                .count();
            std_black_box(dangerous_count)
        });
    });

    // Test 4: Token counting
    group.bench_function("token_counting_large_script", |bencher| {
        let script = r#"
            fn generate_map(width, height) {
                let tiles = [];
                for y in 0..height {
                    for x in 0..width {
                        let tile = create_tile(x, y);
                        tiles.push(tile);
                    }
                }
                return tiles;
            }
            
            fn create_tile(x, y) {
                let height = noise(x * 0.1, y * 0.1);
                return #{
                    x: x,
                    y: y,
                    height: height,
                    type: if height > 0.5 { "mountain" } else { "plains" }
                };
            }
        "#
        .repeat(10);

        bencher.iter(|| {
            let tokens: Vec<_> = script
                .split(|c: char| c.is_whitespace() || "{}()[];,".contains(c))
                .filter(|s| !s.is_empty())
                .collect();
            std_black_box(tokens.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: MAP GENERATION
// ============================================================================

fn bench_map_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("author_adversarial/map_generation");

    // Test 1: Map metadata creation
    for size in [64, 128, 256] {
        group.throughput(Throughput::Elements((size * size) as u64));

        group.bench_with_input(
            BenchmarkId::new("map_meta_creation", format!("{}x{}", size, size)),
            &size,
            |bencher, &size| {
                bencher.iter(|| {
                    let meta = generate_map_meta(size, size);
                    std_black_box(meta.spawn_zones.len())
                });
            },
        );
    }

    // Test 2: Spawn zone generation
    group.bench_function("spawn_zone_generation_100", |bencher| {
        bencher.iter(|| {
            let zones: Vec<SpawnZone> = (0..100)
                .map(|i| SpawnZone {
                    x: (i % 10) as f32 * 10.0,
                    y: (i / 10) as f32 * 10.0,
                    radius: 5.0 + (i % 5) as f32,
                    enemy_types: vec![
                        "goblin".to_string(),
                        "orc".to_string(),
                        "troll".to_string(),
                    ],
                    max_spawns: (i % 10 + 1) as u32,
                })
                .collect();

            let total_spawns: u32 = zones.iter().map(|z| z.max_spawns).sum();
            std_black_box(total_spawns)
        });
    });

    // Test 3: Director budget calculation
    group.bench_function("director_budget_calculation", |bencher| {
        let map = generate_map_meta(256, 256);

        bencher.iter(|| {
            let budget = DirectorBudget {
                total_enemies: map.enemy_count * 2,
                total_loot_value: map.enemy_count * 100,
                boss_encounters: (map.difficulty * 3.0) as u32,
                miniboss_count: (map.difficulty * 10.0) as u32,
                difficulty_curve: (0..100)
                    .map(|i| {
                        let t = i as f32 / 100.0;
                        t * t * map.difficulty // Quadratic curve
                    })
                    .collect(),
            };

            std_black_box(budget.difficulty_curve.len())
        });
    });

    // Test 4: Biome distribution
    group.bench_function("biome_distribution_1024", |bencher| {
        let biomes = ["forest", "desert", "mountain", "swamp", "tundra"];

        bencher.iter(|| {
            let distribution: Vec<&str> = (0..1024)
                .map(|i| {
                    let noise = ((i as f32 * 0.1).sin() + 1.0) / 2.0;
                    let idx = (noise * biomes.len() as f32) as usize;
                    biomes[idx.min(biomes.len() - 1)]
                })
                .collect();

            let forest_count = distribution.iter().filter(|&&b| b == "forest").count();
            std_black_box(forest_count)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: SCRIPT EXECUTION
// ============================================================================

fn bench_script_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("author_adversarial/script_execution");

    // Test 1: Variable operations
    group.bench_function("variable_operations_1000", |bencher| {
        let script = r#"
            x = 1
            y = 2
            z = 3
        "#
        .repeat(333);

        bencher.iter(|| {
            let mut context = ScriptContext {
                variables: HashMap::new(),
                functions: Vec::new(),
                execution_time_ms: 0.0,
            };

            let _ = execute_script_simulation(&script, &mut context);
            std_black_box(context.variables.len())
        });
    });

    // Test 2: Function call simulation
    group.bench_function("function_calls_500", |bencher| {
        let functions: HashMap<String, fn(i64, i64) -> i64> = [
            ("add".to_string(), (|a, b| a + b) as fn(i64, i64) -> i64),
            ("sub".to_string(), (|a, b| a - b) as fn(i64, i64) -> i64),
            ("mul".to_string(), (|a, b| a * b) as fn(i64, i64) -> i64),
            ("div".to_string(), (|a, b| if b != 0 { a / b } else { 0 }) as fn(i64, i64) -> i64),
        ]
        .into_iter()
        .collect();

        let calls: Vec<(&str, i64, i64)> = (0..500)
            .map(|i| {
                let func = match i % 4 {
                    0 => "add",
                    1 => "sub",
                    2 => "mul",
                    _ => "div",
                };
                (func, i as i64, (i + 1) as i64)
            })
            .collect();

        bencher.iter(|| {
            let results: Vec<i64> = calls
                .iter()
                .filter_map(|(name, a, b)| functions.get(*name).map(|f| f(*a, *b)))
                .collect();
            std_black_box(results.iter().sum::<i64>())
        });
    });

    // Test 3: Loop execution
    group.bench_function("loop_execution_10000_iterations", |bencher| {
        bencher.iter(|| {
            let mut sum = 0i64;
            let mut context = ScriptContext {
                variables: HashMap::new(),
                functions: Vec::new(),
                execution_time_ms: 0.0,
            };

            // Simulate a for loop
            for i in 0..10000 {
                context.variables.insert(
                    "i".to_string(),
                    ScriptValue::Integer(i),
                );
                sum += i;
            }

            std_black_box(sum)
        });
    });

    // Test 4: Context switching
    group.bench_function("context_switching_100", |bencher| {
        let contexts: Vec<ScriptContext> = (0..100)
            .map(|i| {
                let mut vars = HashMap::new();
                vars.insert("id".to_string(), ScriptValue::Integer(i));
                vars.insert("name".to_string(), ScriptValue::String(format!("context_{}", i)));
                ScriptContext {
                    variables: vars,
                    functions: vec!["update".to_string(), "render".to_string()],
                    execution_time_ms: 0.0,
                }
            })
            .collect();

        bencher.iter(|| {
            let mut active_context = &contexts[0];

            for i in 0..100 {
                active_context = &contexts[i % contexts.len()];
            }

            std_black_box(active_context.variables.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: JSON/TOML CONVERSION
// ============================================================================

fn bench_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("author_adversarial/conversion");

    // Test 1: Map meta to JSON-like structure
    group.bench_function("map_to_json_100", |bencher| {
        let maps: Vec<MapMeta> = (0..100)
            .map(|i| generate_map_meta(64 + i * 4, 64 + i * 4))
            .collect();

        bencher.iter(|| {
            let json_strings: Vec<String> = maps
                .iter()
                .map(|m| {
                    format!(
                        r#"{{"width":{},"height":{},"enemy_count":{},"difficulty":{},"biome":"{}","spawn_zones":{}}}"#,
                        m.width, m.height, m.enemy_count, m.difficulty, m.biome, m.spawn_zones.len()
                    )
                })
                .collect();

            let total_len: usize = json_strings.iter().map(|s| s.len()).sum();
            std_black_box(total_len)
        });
    });

    // Test 2: Script value serialization
    group.bench_function("value_serialization_500", |bencher| {
        let values: Vec<ScriptValue> = (0..500)
            .map(|i| match i % 5 {
                0 => ScriptValue::Integer(i as i64),
                1 => ScriptValue::Float(i as f64 * 0.5),
                2 => ScriptValue::String(format!("string_{}", i)),
                3 => ScriptValue::Array(vec![
                    ScriptValue::Integer(i as i64),
                    ScriptValue::Integer((i + 1) as i64),
                ]),
                _ => {
                    let mut map = HashMap::new();
                    map.insert("key".to_string(), ScriptValue::Integer(i as i64));
                    ScriptValue::Map(map)
                }
            })
            .collect();

        bencher.iter(|| {
            let serialized: Vec<String> = values
                .iter()
                .map(|v| match v {
                    ScriptValue::Integer(i) => i.to_string(),
                    ScriptValue::Float(f) => f.to_string(),
                    ScriptValue::String(s) => format!("\"{}\"", s),
                    ScriptValue::Array(arr) => format!("[{}]", arr.len()),
                    ScriptValue::Map(map) => format!("{{{}}}", map.len()),
                })
                .collect();

            let total_len: usize = serialized.iter().map(|s| s.len()).sum();
            std_black_box(total_len)
        });
    });

    // Test 3: TOML-like key-value parsing
    group.bench_function("toml_parsing_200_lines", |bencher| {
        let toml_content: String = (0..200)
            .map(|i| format!("key_{} = {}\n", i, i * 10))
            .collect();

        bencher.iter(|| {
            let parsed: HashMap<String, i64> = toml_content
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim().to_string();
                        let value = parts[1].trim().parse::<i64>().ok()?;
                        Some((key, value))
                    } else {
                        None
                    }
                })
                .collect();

            std_black_box(parsed.len())
        });
    });

    // Test 4: Nested structure conversion
    group.bench_function("nested_structure_50", |bencher| {
        let nested: Vec<(String, Vec<(String, i64)>)> = (0..50)
            .map(|i| {
                let inner: Vec<(String, i64)> = (0..10)
                    .map(|j| (format!("inner_{}", j), j as i64))
                    .collect();
                (format!("outer_{}", i), inner)
            })
            .collect();

        bencher.iter(|| {
            let flattened: Vec<(String, i64)> = nested
                .iter()
                .flat_map(|(outer, inners)| {
                    inners
                        .iter()
                        .map(|(inner, val)| (format!("{}.{}", outer, inner), *val))
                        .collect::<Vec<_>>()
                })
                .collect();

            std_black_box(flattened.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: SANDBOXING
// ============================================================================

fn bench_sandboxing(c: &mut Criterion) {
    let mut group = c.benchmark_group("author_adversarial/sandboxing");

    // Test 1: Operation counting
    group.bench_function("operation_counting_10000", |bencher| {
        let max_operations = 100000u64;

        bencher.iter(|| {
            let mut op_count = 0u64;

            // Simulate counting operations
            for _ in 0..10000 {
                op_count += 1;
                if op_count > max_operations {
                    break;
                }
            }

            std_black_box(op_count)
        });
    });

    // Test 2: Memory tracking
    group.bench_function("memory_tracking_allocations", |bencher| {
        let max_memory = 1024 * 1024usize; // 1MB

        bencher.iter(|| {
            let mut allocated = 0usize;
            let mut allocations: Vec<Vec<u8>> = Vec::new();

            for i in 0..100 {
                let size = (i + 1) * 1024;
                if allocated + size <= max_memory {
                    allocations.push(vec![0u8; size]);
                    allocated += size;
                }
            }

            std_black_box(allocations.len())
        });
    });

    // Test 3: Time limit checking
    group.bench_function("time_limit_check_1000", |bencher| {
        let start_time = 0u64; // Simulated
        let time_limit_ns = 1_000_000u64; // 1ms

        bencher.iter(|| {
            let mut exceeded = false;
            let mut iterations = 0u64;

            for i in 0..1000u64 {
                // Simulate time check every 100 iterations
                if i % 100 == 0 {
                    let current_time = i * 1000; // Simulated elapsed time
                    if current_time - start_time > time_limit_ns {
                        exceeded = true;
                        break;
                    }
                }
                iterations += 1;
            }

            std_black_box((iterations, exceeded))
        });
    });

    // Test 4: Stack depth limiting
    group.bench_function("stack_depth_limit_recursive", |bencher| {
        let max_depth = 100usize;

        fn recursive_with_limit(depth: usize, max: usize) -> usize {
            if depth >= max {
                return depth;
            }
            recursive_with_limit(depth + 1, max)
        }

        bencher.iter(|| {
            let final_depth = recursive_with_limit(0, max_depth);
            std_black_box(final_depth)
        });
    });

    // Test 5: Forbidden API detection
    group.bench_function("forbidden_api_detection", |bencher| {
        let calls: Vec<String> = (0..200)
            .map(|i| match i % 10 {
                0 => "file_read".to_string(),
                1 => "file_write".to_string(),
                2 => "network_connect".to_string(),
                3 => "process_spawn".to_string(),
                4 => "env_get".to_string(),
                _ => "safe_function".to_string(),
            })
            .collect();

        let forbidden = ["file_read", "file_write", "network_connect", "process_spawn", "env_get"];
        let forbidden_set: std::collections::HashSet<&str> = forbidden.iter().copied().collect();

        bencher.iter(|| {
            let blocked: Vec<_> = calls
                .iter()
                .filter(|c| forbidden_set.contains(c.as_str()))
                .collect();

            std_black_box(blocked.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: INTEGRATION
// ============================================================================

fn bench_integration(c: &mut Criterion) {
    let mut group = c.benchmark_group("author_adversarial/integration");

    // Test 1: Full map generation pipeline
    group.bench_function("full_map_pipeline", |bencher| {
        bencher.iter(|| {
            // Step 1: Parse configuration
            let config = r#"
                width = 128
                height = 128
                difficulty = 0.7
            "#;

            let mut settings: HashMap<String, f64> = HashMap::new();
            for line in config.lines() {
                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim().to_string();
                    if let Ok(val) = line[eq_pos + 1..].trim().parse::<f64>() {
                        settings.insert(key, val);
                    }
                }
            }

            // Step 2: Generate map
            let width = settings.get("width").copied().unwrap_or(64.0) as u32;
            let height = settings.get("height").copied().unwrap_or(64.0) as u32;
            let meta = generate_map_meta(width, height);

            // Step 3: Generate spawn zones
            let zones: Vec<SpawnZone> = (0..20)
                .map(|i| SpawnZone {
                    x: (i % 10) as f32 * (width as f32 / 10.0),
                    y: (i / 10) as f32 * (height as f32 / 2.0),
                    radius: 10.0,
                    enemy_types: vec!["goblin".to_string()],
                    max_spawns: 5,
                })
                .collect();

            // Step 4: Calculate budget
            let budget = DirectorBudget {
                total_enemies: zones.iter().map(|z| z.max_spawns).sum(),
                total_loot_value: meta.enemy_count * 50,
                boss_encounters: 1,
                miniboss_count: 3,
                difficulty_curve: vec![0.0, 0.3, 0.5, 0.7, 1.0],
            };

            std_black_box((meta.spawn_zones.len(), budget.total_enemies))
        });
    });

    // Test 2: Script-driven entity spawning
    group.bench_function("script_entity_spawning_100", |bencher| {
        let spawn_commands: Vec<(String, f32, f32)> = (0..100)
            .map(|i| {
                let enemy_type = match i % 4 {
                    0 => "goblin",
                    1 => "orc",
                    2 => "troll",
                    _ => "dragon",
                };
                (enemy_type.to_string(), i as f32 * 5.0, (i % 10) as f32 * 10.0)
            })
            .collect();

        bencher.iter(|| {
            let entities: Vec<(u64, String, (f32, f32))> = spawn_commands
                .iter()
                .enumerate()
                .map(|(id, (enemy_type, x, y))| {
                    (id as u64, enemy_type.clone(), (*x, *y))
                })
                .collect();

            std_black_box(entities.len())
        });
    });

    // Test 3: Event dispatching
    group.bench_function("event_dispatching_500", |bencher| {
        let events: Vec<(String, HashMap<String, ScriptValue>)> = (0..500)
            .map(|i| {
                let mut data = HashMap::new();
                data.insert("source".to_string(), ScriptValue::Integer(i as i64));
                data.insert("timestamp".to_string(), ScriptValue::Float(i as f64 * 0.016));

                let event_type = match i % 5 {
                    0 => "enemy_spawned",
                    1 => "player_damaged",
                    2 => "loot_dropped",
                    3 => "objective_complete",
                    _ => "timer_tick",
                };

                (event_type.to_string(), data)
            })
            .collect();

        let mut handlers: HashMap<String, Vec<fn(&HashMap<String, ScriptValue>) -> bool>> = HashMap::new();
        handlers.insert(
            "enemy_spawned".to_string(),
            vec![|_| true],
        );
        handlers.insert(
            "player_damaged".to_string(),
            vec![|_| true],
        );

        bencher.iter(|| {
            let mut handled = 0;

            for (event_type, data) in &events {
                if let Some(event_handlers) = handlers.get(event_type) {
                    for handler in event_handlers {
                        if handler(data) {
                            handled += 1;
                        }
                    }
                }
            }

            std_black_box(handled)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_script_parsing,
    bench_map_generation,
    bench_script_execution,
    bench_conversion,
    bench_sandboxing,
    bench_integration,
);

criterion_main!(benches);
