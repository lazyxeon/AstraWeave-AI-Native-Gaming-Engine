use criterion::{black_box, criterion_group, criterion_main, Criterion};
use astraweave_scripting::{CScript, ScriptEngineResource, script_system};
use astraweave_ecs::{App, World, SystemStage};
use rhai::{Engine, Scope, AST};
use std::sync::Arc;

fn bench_script_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("script_execution");
    
    // Setup engine once
    let engine = Engine::new();
    let ast = engine.compile("let x = 10; let y = 20; x + y").unwrap();
    let ast_arc = Arc::new(ast);
    
    group.bench_function("rhai_raw_execution", |b| {
        b.iter(|| {
            let mut scope = Scope::new();
            engine.run_ast_with_scope(&mut scope, black_box(&ast_arc)).unwrap();
        })
    });
    
    // Setup ECS world with 1000 entities
    let mut app = App::new();
    app.insert_resource(ScriptEngineResource(Arc::new(engine)));
    app.add_system(SystemStage::SIMULATION, script_system);
    
    let script_source = "let x = 10; let y = 20; x + y";
    // Pre-compile by running once manually or just setting cached_ast
    // We can't easily set cached_ast from outside because fields are public but we need the engine to compile.
    // Let's just use the system to compile on first run.
    
    for _ in 0..1000 {
        let e = app.world.spawn();
        let mut script = CScript::new("bench.rhai", script_source);
        // Pre-populate AST to benchmark execution only, not compilation
        script.cached_ast = Some(ast_arc.clone());
        app.world.insert(e, script);
    }
    
    group.bench_function("ecs_script_system_1000_entities", |b| {
        b.iter(|| {
            app.schedule.run(&mut app.world);
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_script_execution);
criterion_main!(benches);
