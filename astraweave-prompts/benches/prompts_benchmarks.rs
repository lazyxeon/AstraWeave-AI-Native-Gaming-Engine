use astraweave_prompts::{PromptContext, PromptEngine, PromptTemplate, TemplateEngine};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::hint::black_box;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a simple template
fn create_simple_template() -> PromptTemplate {
    PromptTemplate::new("simple", "Hello {{name}}, you are {{role}}!")
}

/// Create a complex template with multiple variables
fn create_complex_template() -> PromptTemplate {
    PromptTemplate::new(
        "complex",
        "Character: {{character.name}} ({{character.class}})\n\
         Location: {{location.name}} - {{location.description}}\n\
         Objective: {{objective}}\n\
         Inventory: {{inventory.count}} items\n\
         Status: Health {{status.health}}/{{status.max_health}}, \
         Mana {{status.mana}}/{{status.max_mana}}",
    )
}

/// Create a dialogue template
fn create_dialogue_template() -> PromptTemplate {
    PromptTemplate::new(
        "dialogue",
        "You are {{npc.name}}, a {{npc.role}}. \
         Your personality is {{npc.personality}}. \
         The player says: {{player.message}}. \
         Respond in character, considering: {{context}}",
    )
}

/// Create a simple context
fn create_simple_context() -> PromptContext {
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "Player".into());
    ctx.set("role".to_string(), "Warrior".into());
    ctx
}

/// Create a complex context
fn create_complex_context() -> PromptContext {
    let mut ctx = PromptContext::new();
    ctx.set("character.name".to_string(), "Aria".into());
    ctx.set("character.class".to_string(), "Mage".into());
    ctx.set("location.name".to_string(), "Ancient Library".into());
    ctx.set(
        "location.description".to_string(),
        "A vast repository of magical knowledge".into(),
    );
    ctx.set("objective".to_string(), "Find the Crystal of Wisdom".into());
    ctx.set("inventory.count".to_string(), "12".into());
    ctx.set("status.health".to_string(), "85".into());
    ctx.set("status.max_health".to_string(), "100".into());
    ctx.set("status.mana".to_string(), "45".into());
    ctx.set("status.max_mana".to_string(), "80".into());
    ctx
}

/// Create a dialogue context
fn create_dialogue_context() -> PromptContext {
    let mut ctx = PromptContext::new();
    ctx.set("npc.name".to_string(), "Elena".into());
    ctx.set("npc.role".to_string(), "Wise Mage".into());
    ctx.set(
        "npc.personality".to_string(),
        "mysterious and helpful".into(),
    );
    ctx.set(
        "player.message".to_string(),
        "What magic can you teach me?".into(),
    );
    ctx.set(
        "context".to_string(),
        "The player has proven their worth".into(),
    );
    ctx
}

// ============================================================================
// Benchmark 1: Template Creation
// ============================================================================

fn bench_template_creation_simple(c: &mut Criterion) {
    c.bench_function("template_creation_simple", |b| {
        b.iter(|| {
            let template = create_simple_template();
            black_box(template)
        })
    });
}

fn bench_template_creation_complex(c: &mut Criterion) {
    c.bench_function("template_creation_complex", |b| {
        b.iter(|| {
            let template = create_complex_template();
            black_box(template)
        })
    });
}

fn bench_template_creation_dialogue(c: &mut Criterion) {
    c.bench_function("template_creation_dialogue", |b| {
        b.iter(|| {
            let template = create_dialogue_template();
            black_box(template)
        })
    });
}

// ============================================================================
// Benchmark 2: Context Creation
// ============================================================================

fn bench_context_creation_simple(c: &mut Criterion) {
    c.bench_function("context_creation_simple", |b| {
        b.iter(|| {
            let ctx = create_simple_context();
            black_box(ctx)
        })
    });
}

fn bench_context_creation_complex(c: &mut Criterion) {
    c.bench_function("context_creation_complex", |b| {
        b.iter(|| {
            let ctx = create_complex_context();
            black_box(ctx)
        })
    });
}

// ============================================================================
// Benchmark 3: Template Rendering
// ============================================================================

fn bench_template_render_simple(c: &mut Criterion) {
    let template = create_simple_template();
    let context = create_simple_context();

    c.bench_function("template_render_simple", |b| {
        b.iter(|| {
            let rendered = template.render(&context).unwrap();
            black_box(rendered)
        })
    });
}

fn bench_template_render_complex(c: &mut Criterion) {
    let template = create_complex_template();
    let context = create_complex_context();

    c.bench_function("template_render_complex", |b| {
        b.iter(|| {
            let rendered = template.render(&context).unwrap();
            black_box(rendered)
        })
    });
}

fn bench_template_render_dialogue(c: &mut Criterion) {
    let template = create_dialogue_template();
    let context = create_dialogue_context();

    c.bench_function("template_render_dialogue", |b| {
        b.iter(|| {
            let rendered = template.render(&context).unwrap();
            black_box(rendered)
        })
    });
}

// ============================================================================
// Benchmark 4: Template Engine Operations
// ============================================================================

fn bench_engine_creation(c: &mut Criterion) {
    c.bench_function("engine_creation", |b| {
        b.iter(|| {
            let engine = TemplateEngine::new();
            black_box(engine)
        })
    });
}

fn bench_engine_register_template(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_register_template");

    for count in [1, 10, 50] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_with_setup(
                || TemplateEngine::new(),
                |mut engine| {
                    for i in 0..count {
                        let template = PromptTemplate::new(
                            format!("template_{}", i),
                            format!("Test template {}: {{{{var{}}}}}", i, i),
                        );
                        let _ = engine.register_template(&format!("template_{}", i), template);
                    }
                    black_box(engine)
                },
            )
        });
    }

    group.finish();
}

fn bench_engine_render(c: &mut Criterion) {
    let mut engine = TemplateEngine::new();
    let template = create_simple_template();
    let _ = engine.register_template("test", template);
    let context = create_simple_context();

    c.bench_function("engine_render", |b| {
        b.iter(|| {
            let rendered = engine.render("test", &context).unwrap();
            black_box(rendered)
        })
    });
}

// ============================================================================
// Benchmark 5: Batch Rendering
// ============================================================================

fn bench_batch_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_render");

    for count in [10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_with_setup(
                || {
                    let template = create_simple_template();
                    let context = create_simple_context();
                    (template, context)
                },
                |(template, context)| {
                    let rendered: Vec<_> = (0..count)
                        .map(|_| template.render(&context).unwrap())
                        .collect();
                    black_box(rendered)
                },
            )
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 6: Context Modifications
// ============================================================================

fn bench_context_add_variables(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_add_variables");

    for count in [5, 10, 20] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_with_setup(
                || PromptContext::new(),
                |mut context| {
                    for i in 0..count {
                        context.set(format!("var_{}", i), format!("value_{}", i).into());
                    }
                    black_box(context)
                },
            )
        });
    }

    group.finish();
}

fn bench_context_to_string_map(c: &mut Criterion) {
    let context = create_complex_context();

    c.bench_function("context_to_string_map", |b| {
        b.iter(|| {
            let map = context.to_string_map();
            black_box(map)
        })
    });
}

// ============================================================================
// Benchmark 7: Template Cloning & Comparison
// ============================================================================

fn bench_template_clone(c: &mut Criterion) {
    let template = create_complex_template();

    c.bench_function("template_clone", |b| {
        b.iter(|| {
            let cloned = template.clone();
            black_box(cloned)
        })
    });
}

fn bench_context_clone(c: &mut Criterion) {
    let context = create_complex_context();

    c.bench_function("context_clone", |b| {
        b.iter(|| {
            let cloned = context.clone();
            black_box(cloned)
        })
    });
}

// ============================================================================
// Benchmark 8: Template HashMap Rendering (Backward Compat)
// ============================================================================

fn bench_template_render_map(c: &mut Criterion) {
    let template = create_simple_template();
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Player".to_string());
    map.insert("role".to_string(), "Warrior".to_string());

    c.bench_function("template_render_map", |b| {
        b.iter(|| {
            let rendered = template.render_map(&map).unwrap();
            black_box(rendered)
        })
    });
}

// ============================================================================
// Benchmark Registration
// ============================================================================

criterion_group!(
    benches,
    bench_template_creation_simple,
    bench_template_creation_complex,
    bench_template_creation_dialogue,
    bench_context_creation_simple,
    bench_context_creation_complex,
    bench_template_render_simple,
    bench_template_render_complex,
    bench_template_render_dialogue,
    bench_engine_creation,
    bench_engine_register_template,
    bench_engine_render,
    bench_batch_render,
    bench_context_add_variables,
    bench_context_to_string_map,
    bench_template_clone,
    bench_context_clone,
    bench_template_render_map,
);

criterion_main!(benches);
