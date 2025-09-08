use astraweave_input::{Binding, BindingSet, GamepadButton};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

criterion_group!(
    benches,
    bench_binding_creation,
    bench_binding_serialization,
    bench_binding_deserialization,
    bench_binding_set_creation
);
criterion_main!(benches);
