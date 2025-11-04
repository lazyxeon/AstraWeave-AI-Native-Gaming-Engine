use astraweave_input::{Action, Binding, BindingSet, GamepadButton, InputContext, InputManager};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

fn bench_binding_creation(c: &mut Criterion) {
    c.bench_function("binding_creation", |b| {
        b.iter(|| {
            black_box(Binding {
                key: Some(KeyCode::KeyW),
                mouse: Some(MouseButton::Left),
                gamepad: Some(GamepadButton::South),
            })
        })
    });
}

fn bench_binding_serialization(c: &mut Criterion) {
    let binding = Binding {
        key: Some(KeyCode::KeyW),
        mouse: Some(MouseButton::Left),
        gamepad: Some(GamepadButton::South),
    };

    c.bench_function("binding_serialization", |b| {
        b.iter(|| black_box(serde_json::to_string(&binding).unwrap()))
    });
}

fn bench_binding_deserialization(c: &mut Criterion) {
    let binding = Binding {
        key: Some(KeyCode::KeyW),
        mouse: Some(MouseButton::Left),
        gamepad: Some(GamepadButton::South),
    };
    let serialized = serde_json::to_string(&binding).unwrap();

    c.bench_function("binding_deserialization", |b| {
        b.iter(|| {
            let _: Binding = black_box(serde_json::from_str(&serialized).unwrap());
        })
    });
}

fn bench_binding_set_creation(c: &mut Criterion) {
    c.bench_function("binding_set_creation", |b| {
        b.iter(|| black_box(BindingSet::default()))
    });
}

// ========================================
// Week 5 Day 3: InputManager Benchmarks
// ========================================

fn bench_input_manager_creation(c: &mut Criterion) {
    let bindings = BindingSet::default();

    c.bench_function("input_manager_creation", |b| {
        b.iter(|| black_box(InputManager::new(InputContext::Gameplay, bindings.clone())))
    });
}

fn bench_context_switching(c: &mut Criterion) {
    let bindings = BindingSet::default();
    let mut manager = InputManager::new(InputContext::Gameplay, bindings);

    c.bench_function("context_switching", |b| {
        b.iter(|| {
            manager.set_context(black_box(InputContext::UI));
            manager.set_context(black_box(InputContext::Gameplay));
        })
    });
}

fn bench_is_down_query(c: &mut Criterion) {
    let bindings = BindingSet::default();
    let manager = InputManager::new(InputContext::Gameplay, bindings);

    c.bench_function("is_down_query", |b| {
        b.iter(|| black_box(manager.is_down(Action::MoveForward)))
    });
}

fn bench_just_pressed_query(c: &mut Criterion) {
    let bindings = BindingSet::default();
    let manager = InputManager::new(InputContext::Gameplay, bindings);

    c.bench_function("just_pressed_query", |b| {
        b.iter(|| black_box(manager.just_pressed(Action::Jump)))
    });
}

fn bench_clear_frame(c: &mut Criterion) {
    let bindings = BindingSet::default();
    let mut manager = InputManager::new(InputContext::Gameplay, bindings);

    c.bench_function("clear_frame", |b| {
        b.iter(|| {
            manager.clear_frame();
        })
    });
}

fn bench_binding_lookup(c: &mut Criterion) {
    let bindings = BindingSet::default();
    let manager = InputManager::new(InputContext::Gameplay, bindings);

    c.bench_function("binding_lookup", |b| {
        b.iter(|| black_box(manager.bindings.actions.get(&Action::MoveForward)))
    });
}

fn bench_multiple_queries(c: &mut Criterion) {
    let bindings = BindingSet::default();
    let manager = InputManager::new(InputContext::Gameplay, bindings);

    c.bench_function("multiple_queries", |b| {
        b.iter(|| {
            black_box(manager.is_down(Action::MoveForward));
            black_box(manager.is_down(Action::MoveBackward));
            black_box(manager.is_down(Action::MoveLeft));
            black_box(manager.is_down(Action::MoveRight));
            black_box(manager.just_pressed(Action::Jump));
        })
    });
}

fn bench_binding_set_clone(c: &mut Criterion) {
    let bindings = BindingSet::default();

    c.bench_function("binding_set_clone", |b| {
        b.iter(|| black_box(bindings.clone()))
    });
}

fn bench_action_insertion(c: &mut Criterion) {
    c.bench_function("action_insertion", |b| {
        b.iter(|| {
            let mut bindings = BindingSet::default();
            bindings.actions.insert(
                black_box(Action::Ability1),
                black_box(Binding {
                    key: Some(KeyCode::KeyQ),
                    mouse: None,
                    gamepad: None,
                }),
            );
        })
    });
}

fn bench_sensitivity_access(c: &mut Criterion) {
    let bindings = BindingSet::default();
    let manager = InputManager::new(InputContext::Gameplay, bindings);

    c.bench_function("sensitivity_access", |b| {
        b.iter(|| black_box(manager.look_sensitivity))
    });
}

criterion_group!(
    benches,
    bench_binding_creation,
    bench_binding_serialization,
    bench_binding_deserialization,
    bench_binding_set_creation,
    // Week 5 Day 3: InputManager benchmarks
    bench_input_manager_creation,
    bench_context_switching,
    bench_is_down_query,
    bench_just_pressed_query,
    bench_clear_frame,
    bench_binding_lookup,
    bench_multiple_queries,
    bench_binding_set_clone,
    bench_action_insertion,
    bench_sensitivity_access,
);
criterion_main!(benches);
