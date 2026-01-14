use criterion::{criterion_group, criterion_main, Criterion};
use glam::{Mat4, Quat, Vec2, Vec3};
use std::hint::black_box;
use winit::keyboard::KeyCode;

// Import from the library
use aw_editor_lib::gizmo::{
    picking::{GizmoPicker, Ray},
    rendering::GizmoRenderer,
    rotate::RotateGizmo,
    scale::ScaleGizmo,
    scene_viewport::CameraController,
    state::{AxisConstraint, GizmoMode, GizmoState},
    translate::TranslateGizmo,
};

// ================================
// 1. STATE TRANSITIONS
// ================================

fn bench_state_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_transitions");

    group.bench_function("start_translate", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            state.start_translate();
            black_box(&state);
        });
    });

    group.bench_function("start_rotate", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            state.start_rotate();
            black_box(&state);
        });
    });

    group.bench_function("start_scale", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            state.start_scale(false);
            black_box(&state);
        });
    });

    group.bench_function("handle_key_g", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            state.handle_key(KeyCode::KeyG);
            black_box(&state);
        });
    });

    group.bench_function("handle_key_x", |b| {
        let mut state = GizmoState::new();
        state.start_translate();
        b.iter(|| {
            state.handle_key(KeyCode::KeyX);
            black_box(&state);
        });
    });

    group.bench_function("update_mouse", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            state.update_mouse(Vec2::new(100.0, 200.0));
            black_box(&state);
        });
    });

    group.finish();
}

// ================================
// 2. TRANSLATION MATH
// ================================

fn bench_translation_math(c: &mut Criterion) {
    let mut group = c.benchmark_group("translation_math");

    group.bench_function("translate_none_constraint", |b| {
        b.iter(|| {
            let translation = TranslateGizmo::calculate_translation(
                black_box(Vec2::new(100.0, 50.0)),
                black_box(AxisConstraint::None),
                black_box(10.0), // camera_distance
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(translation);
        });
    });

    group.bench_function("translate_x_constraint", |b| {
        b.iter(|| {
            let translation = TranslateGizmo::calculate_translation(
                black_box(Vec2::new(100.0, 50.0)),
                black_box(AxisConstraint::X),
                black_box(10.0), // camera_distance
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(translation);
        });
    });

    group.bench_function("translate_numeric", |b| {
        b.iter(|| {
            let translation = TranslateGizmo::calculate_translation_numeric(
                black_box(5.0),
                black_box(AxisConstraint::X),
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(translation);
        });
    });

    group.finish();
}

// ================================
// 3. ROTATION MATH
// ================================

fn bench_rotation_math(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotation_math");

    group.bench_function("rotate_x_axis", |b| {
        b.iter(|| {
            let rotation = RotateGizmo::calculate_rotation(
                black_box(Vec2::new(100.0, 50.0)),
                black_box(AxisConstraint::X),
                black_box(1.0),
                black_box(false),
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(rotation);
        });
    });

    group.bench_function("rotate_with_snap", |b| {
        b.iter(|| {
            let rotation = RotateGizmo::calculate_rotation(
                black_box(Vec2::new(100.0, 50.0)),
                black_box(AxisConstraint::X),
                black_box(1.0),
                black_box(true), // Snap enabled
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(rotation);
        });
    });

    group.bench_function("rotate_numeric", |b| {
        b.iter(|| {
            let rotation = RotateGizmo::calculate_rotation_numeric(
                black_box(90.0),
                black_box(AxisConstraint::X),
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(rotation);
        });
    });

    group.finish();
}

// ================================
// 4. SCALE MATH
// ================================

fn bench_scale_math(c: &mut Criterion) {
    let mut group = c.benchmark_group("scale_math");

    group.bench_function("scale_uniform", |b| {
        b.iter(|| {
            let scale = ScaleGizmo::calculate_scale(
                black_box(Vec2::new(100.0, 50.0)),
                black_box(AxisConstraint::None),
                black_box(true), // Uniform
                black_box(1.0),
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(scale);
        });
    });

    group.bench_function("scale_x_axis", |b| {
        b.iter(|| {
            let scale = ScaleGizmo::calculate_scale(
                black_box(Vec2::new(100.0, 50.0)),
                black_box(AxisConstraint::X),
                black_box(false), // Not uniform
                black_box(1.0),
                black_box(Quat::IDENTITY),
                black_box(false),
            );
            black_box(scale);
        });
    });

    group.bench_function("scale_numeric", |b| {
        b.iter(|| {
            let scale = ScaleGizmo::calculate_scale_numeric(
                black_box(2.0),
                black_box(AxisConstraint::X),
                black_box(false),
            );
            black_box(scale);
        });
    });

    group.finish();
}

// ================================
// 5. RENDERING
// ================================

fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    group.bench_function("generate_arrow", |b| {
        b.iter(|| {
            let vertices = GizmoRenderer::generate_arrow(black_box(Vec3::X), black_box(1.0));
            black_box(vertices);
        });
    });

    group.bench_function("generate_circle", |b| {
        b.iter(|| {
            let vertices =
                GizmoRenderer::generate_circle(black_box(Vec3::Z), black_box(1.0), black_box(32));
            black_box(vertices);
        });
    });

    group.bench_function("generate_scale_cube", |b| {
        b.iter(|| {
            let vertices = GizmoRenderer::generate_scale_cube(
                black_box(Vec3::X),
                black_box(1.0),
                black_box(0.1),
            );
            black_box(vertices);
        });
    });

    group.finish();
}

// ================================
// 6. PICKING
// ================================

fn bench_picking(c: &mut Criterion) {
    let mut group = c.benchmark_group("picking");

    group.bench_function("ray_from_screen", |b| {
        let inv_view_proj = Mat4::IDENTITY;
        b.iter(|| {
            let ray =
                Ray::from_screen(black_box(Vec2::new(400.0, 300.0)), black_box(inv_view_proj));
            black_box(ray);
        });
    });

    // Pick handle benchmark (using public API with correct GizmoMode)
    group.bench_function("pick_handle", |b| {
        let picker = GizmoPicker::default();
        let screen_pos = Vec2::new(400.0, 300.0);
        let inv_view_proj = Mat4::IDENTITY;
        b.iter(|| {
            let handle = picker.pick_handle(
                black_box(screen_pos),
                black_box(inv_view_proj),
                black_box(Vec3::ZERO),
                black_box(GizmoMode::Translate {
                    constraint: AxisConstraint::None,
                }),
            );
            black_box(handle);
        });
    });

    group.finish();
}

// ================================
// 7. CAMERA CONTROLS
// ================================

fn bench_camera(c: &mut Criterion) {
    let mut group = c.benchmark_group("camera");

    group.bench_function("orbit", |b| {
        let mut camera = CameraController::default();
        b.iter(|| {
            camera.orbit(black_box(Vec2::new(0.1, 0.1)), black_box(1.0));
            black_box(&camera);
        });
    });

    group.bench_function("pan", |b| {
        let mut camera = CameraController::default();
        b.iter(|| {
            camera.pan(black_box(Vec2::new(10.0, 10.0)), black_box(1.0));
            black_box(&camera);
        });
    });

    group.bench_function("zoom", |b| {
        let mut camera = CameraController::default();
        b.iter(|| {
            camera.zoom(black_box(-1.0), black_box(1.0));
            black_box(&camera);
        });
    });

    group.bench_function("view_matrix", |b| {
        let camera = CameraController::default();
        b.iter(|| {
            let matrix = camera.view_matrix();
            black_box(matrix);
        });
    });

    group.bench_function("projection_matrix", |b| {
        let camera = CameraController::default();
        b.iter(|| {
            let matrix = camera.projection_matrix();
            black_box(matrix);
        });
    });

    group.finish();
}

// ================================
// 8. FULL WORKFLOWS
// ================================

fn bench_full_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_workflow");

    group.bench_function("translate_workflow", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            // Start translation
            state.start_translate();

            // Apply X constraint
            state.handle_key(KeyCode::KeyX);

            // Calculate translation (5 args: mouse_delta, constraint, camera_distance, object_rotation, local_space)
            let translation = TranslateGizmo::calculate_translation(
                Vec2::new(100.0, 50.0),
                AxisConstraint::X,
                10.0, // camera_distance
                Quat::IDENTITY,
                false,
            );

            black_box(translation);
        });
    });

    group.bench_function("rotate_workflow", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            // Start rotation
            state.start_rotate();

            // Apply X constraint
            state.handle_key(KeyCode::KeyX);

            // Calculate rotation
            let rotation = RotateGizmo::calculate_rotation(
                Vec2::new(100.0, 50.0),
                AxisConstraint::X,
                1.0,
                false,
                Quat::IDENTITY,
                false,
            );

            black_box(rotation);
        });
    });

    group.bench_function("scale_workflow", |b| {
        let mut state = GizmoState::new();
        b.iter(|| {
            // Start scaling
            state.start_scale(false);

            // Apply X constraint
            state.handle_key(KeyCode::KeyX);

            // Calculate scale
            let scale = ScaleGizmo::calculate_scale(
                Vec2::new(100.0, 50.0),
                AxisConstraint::X,
                false,
                1.0,
                Quat::IDENTITY,
                false,
            );

            black_box(scale);
        });
    });

    group.finish();
}

// ================================
// CRITERION CONFIGURATION
// ================================

criterion_group!(
    benches,
    bench_state_transitions,
    bench_translation_math,
    bench_rotation_math,
    bench_scale_math,
    bench_rendering,
    bench_picking,
    bench_camera,
    bench_full_workflow,
);

criterion_main!(benches);
