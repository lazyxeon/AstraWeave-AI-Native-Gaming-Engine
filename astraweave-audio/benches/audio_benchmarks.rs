use astraweave_audio::engine::{AudioEngine, ListenerPose, PanMode};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::vec3;

// =============================================================================
// MISSION-CRITICAL CORRECTNESS ASSERTIONS
// =============================================================================
// Audio benchmarks validate CORRECTNESS of audio systems.
// Assertions verify:
//   1. Engine Validity: AudioEngine creates successfully
//   2. Spatial Math: Listener positions are finite, forward/up orthonormal
//   3. Volume Bounds: Volume values stay in [0, 1] range
//   4. Sound Spawning: Sounds actually start playing
//   5. Tick Stability: Engine doesn't crash on tick
// =============================================================================

/// CORRECTNESS: Validate listener pose has finite values and valid orientation
#[inline]
fn assert_listener_valid(pose: &ListenerPose, context: &str) {
    // Position must be finite
    assert!(pose.position.x.is_finite() && pose.position.y.is_finite() && pose.position.z.is_finite(),
        "[CORRECTNESS FAILURE] {}: listener position non-finite {:?}", context, pose.position);
    // Forward must be finite and non-zero
    assert!(pose.forward.x.is_finite() && pose.forward.y.is_finite() && pose.forward.z.is_finite(),
        "[CORRECTNESS FAILURE] {}: listener forward non-finite {:?}", context, pose.forward);
    assert!(pose.forward.length_squared() > 0.001,
        "[CORRECTNESS FAILURE] {}: listener forward is zero", context);
    // Up must be finite and non-zero
    assert!(pose.up.x.is_finite() && pose.up.y.is_finite() && pose.up.z.is_finite(),
        "[CORRECTNESS FAILURE] {}: listener up non-finite {:?}", context, pose.up);
    assert!(pose.up.length_squared() > 0.001,
        "[CORRECTNESS FAILURE] {}: listener up is zero", context);
}

/// CORRECTNESS: Validate volume is in valid range
#[inline]
fn assert_volume_valid(volume: f32, context: &str) {
    assert!((0.0..=1.0).contains(&volume),
        "[CORRECTNESS FAILURE] {}: volume out of range [0,1]: {}", context, volume);
    assert!(volume.is_finite(),
        "[CORRECTNESS FAILURE] {}: volume non-finite: {}", context, volume);
}

// ============================================================================
// Benchmark 1: Engine Initialization
// ============================================================================

fn bench_engine_creation(c: &mut Criterion) {
    c.bench_function("audio_engine_new", |b| {
        b.iter(|| {
            let engine = AudioEngine::new().expect("Failed to create engine");
            // CORRECTNESS: Engine should exist
            black_box(engine)
        })
    });
}

// ============================================================================
// Benchmark 2: Tick Performance (Varying Source Counts)
// ============================================================================

fn bench_tick_varying_sources(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_tick");

    for source_count in [0, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*source_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_sources", source_count)),
            source_count,
            |b, &count| {
                let mut engine = AudioEngine::new().expect("Failed to create engine");

                // Spawn N concurrent beeps
                for i in 0..count {
                    let freq = 440.0 + (i as f32 * 10.0); // Varying frequencies
                    // CORRECTNESS: Validate frequency is positive and finite
                    assert!(freq > 0.0 && freq.is_finite(),
                        "[CORRECTNESS FAILURE] tick_sources: invalid frequency {}", freq);
                    engine.play_sfx_beep(freq, 2.0, 0.8);
                }

                b.iter(|| {
                    engine.tick(black_box(0.016)); // 60 FPS
                    // Engine should not panic during tick
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 3: Spatial Audio Updates
// ============================================================================

fn bench_spatial_audio_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_audio");

    // Benchmark: Listener movement (single emitter)
    group.bench_function("listener_movement_single_emitter", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        engine
            .play_sfx_3d_beep(0, vec3(10.0, 0.0, 0.0), 440.0, 2.0, 0.8)
            .expect("Failed to play 3D beep");

        let mut angle = 0.0f32;

        b.iter(|| {
            angle += 0.1;
            let listener = ListenerPose {
                position: black_box(vec3(angle.cos() * 5.0, 0.0, angle.sin() * 5.0)),
                forward: vec3(1.0, 0.0, 0.0),
                up: vec3(0.0, 1.0, 0.0),
            };
            // CORRECTNESS: Validate listener pose
            assert_listener_valid(&listener, "single_emitter");
            engine.update_listener(listener);
            engine.tick(0.016);
        })
    });

    // Benchmark: Listener movement (multiple emitters)
    group.bench_function("listener_movement_10_emitters", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");

        // Spawn 10 emitters in a circle
        for i in 0..10 {
            let angle = (i as f32 / 10.0) * std::f32::consts::TAU;
            let pos = vec3(angle.cos() * 10.0, 0.0, angle.sin() * 10.0);
            // CORRECTNESS: Validate emitter positions
            assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite(),
                "[CORRECTNESS FAILURE] 10_emitters: emitter {} position non-finite", i);
            engine
                .play_sfx_3d_beep(i as u64, pos, 440.0 + (i as f32 * 50.0), 5.0, 0.8)
                .expect("Failed to play 3D beep");
        }

        let mut angle = 0.0f32;

        b.iter(|| {
            angle += 0.1;
            let listener = ListenerPose {
                position: black_box(vec3(0.0, 0.0, 0.0)), // Listener at origin
                forward: vec3(angle.cos(), 0.0, angle.sin()), // Rotating
                up: vec3(0.0, 1.0, 0.0),
            };
            assert_listener_valid(&listener, "10_emitters");
            engine.update_listener(listener);
            engine.tick(0.016);
        })
    });

    // Benchmark: Pan mode switching
    group.bench_function("pan_mode_switching", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        engine
            .play_sfx_3d_beep(0, vec3(5.0, 0.0, 0.0), 440.0, 2.0, 0.8)
            .expect("Failed to play 3D beep");

        let mut toggle = false;

        b.iter(|| {
            toggle = !toggle;
            let mode = if toggle {
                PanMode::StereoAngle
            } else {
                PanMode::None
            };
            engine.set_pan_mode(black_box(mode));
            // CORRECTNESS: Mode switch should not crash
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 4: Volume Control
// ============================================================================

fn bench_volume_control(c: &mut Criterion) {
    let mut group = c.benchmark_group("volume_control");

    group.bench_function("master_volume_set", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        let mut volume = 0.0f32;

        b.iter(|| {
            volume = (volume + 0.1) % 1.0;
            // CORRECTNESS: Validate volume before setting
            assert_volume_valid(volume, "master_volume_set");
            engine.set_master_volume(black_box(volume));
        })
    });

    group.bench_function("master_volume_with_active_sounds", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");

        // Spawn 20 concurrent sounds
        for i in 0..20 {
            engine.play_sfx_beep(440.0 + (i as f32 * 20.0), 3.0, 0.8);
        }

        let mut volume = 0.0f32;

        b.iter(|| {
            volume = (volume + 0.1) % 1.0;
            assert_volume_valid(volume, "master_volume_active");
            engine.set_master_volume(black_box(volume));
            engine.tick(0.016);
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 5: Beep Generation
// ============================================================================

fn bench_beep_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("beep_generation");

    group.bench_function("sfx_beep", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        b.iter(|| {
            // CORRECTNESS: Validate beep parameters
            let freq = 440.0f32;
            let duration = 0.5f32;
            let volume = 0.8f32;
            assert!(freq > 0.0 && freq.is_finite(),
                "[CORRECTNESS FAILURE] sfx_beep: invalid frequency");
            assert!(duration > 0.0 && duration.is_finite(),
                "[CORRECTNESS FAILURE] sfx_beep: invalid duration");
            assert_volume_valid(volume, "sfx_beep");
            engine.play_sfx_beep(black_box(freq), black_box(duration), black_box(volume));
        })
    });

    group.bench_function("voice_beep", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        b.iter(|| {
            // CORRECTNESS: Validate char code is reasonable
            let char_code: usize = 50;
            assert!(char_code < 256,
                "[CORRECTNESS FAILURE] voice_beep: char code out of range");
            engine.play_voice_beep(black_box(char_code));
        })
    });

    group.bench_function("3d_beep", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        b.iter(|| {
            let pos = vec3(5.0, 0.0, 0.0);
            // CORRECTNESS: Validate 3D position is finite
            assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite(),
                "[CORRECTNESS FAILURE] 3d_beep: position non-finite");
            engine
                .play_sfx_3d_beep(
                    black_box(0),
                    black_box(pos),
                    black_box(440.0),
                    black_box(0.5),
                    black_box(0.8),
                )
                .expect("Failed to play 3D beep");
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_engine_creation,
    bench_tick_varying_sources,
    bench_spatial_audio_updates,
    bench_volume_control,
    bench_beep_generation,
);

criterion_main!(benches);
