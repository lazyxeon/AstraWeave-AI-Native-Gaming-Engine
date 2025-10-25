use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use astraweave_audio::engine::{AudioEngine, ListenerPose, MusicTrack, PanMode};
use glam::{vec3, Vec3};

// ============================================================================
// Benchmark 1: Engine Initialization
// ============================================================================

fn bench_engine_creation(c: &mut Criterion) {
    c.bench_function("audio_engine_new", |b| {
        b.iter(|| {
            black_box(AudioEngine::new().expect("Failed to create engine"))
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
                    engine.play_sfx_beep(freq, 2.0, 0.8);
                }
                
                b.iter(|| {
                    engine.tick(black_box(0.016)); // 60 FPS
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
        engine.play_sfx_3d_beep(440.0, 2.0, 0, vec3(10.0, 0.0, 0.0), 0.8);
        
        let mut angle = 0.0f32;
        
        b.iter(|| {
            angle += 0.1;
            let listener = ListenerPose {
                pos: black_box(vec3(angle.cos() * 5.0, 0.0, angle.sin() * 5.0)),
                fwd: vec3(1.0, 0.0, 0.0),
                up: vec3(0.0, 1.0, 0.0),
            };
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
            engine.play_sfx_3d_beep(440.0 + (i as f32 * 50.0), 5.0, i as u64, pos, 0.8);
        }
        
        let mut angle = 0.0f32;
        
        b.iter(|| {
            angle += 0.1;
            let listener = ListenerPose {
                pos: black_box(vec3(0.0, 0.0, 0.0)), // Listener at origin
                fwd: vec3(angle.cos(), 0.0, angle.sin()), // Rotating
                up: vec3(0.0, 1.0, 0.0),
            };
            engine.update_listener(listener);
            engine.tick(0.016);
        })
    });
    
    // Benchmark: Pan mode switching
    group.bench_function("pan_mode_switching", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        engine.play_sfx_3d_beep(440.0, 2.0, 0, vec3(5.0, 0.0, 0.0), 0.8);
        
        let mut toggle = false;
        
        b.iter(|| {
            toggle = !toggle;
            let mode = if toggle { PanMode::HRTF } else { PanMode::Simple };
            engine.set_pan_mode(black_box(mode));
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
            engine.play_sfx_beep(black_box(440.0), black_box(0.5), black_box(0.8));
        })
    });
    
    group.bench_function("voice_beep", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        b.iter(|| {
            engine.play_voice_beep(black_box(50));
        })
    });
    
    group.bench_function("3d_beep", |b| {
        let mut engine = AudioEngine::new().expect("Failed to create engine");
        b.iter(|| {
            engine.play_sfx_3d_beep(
                black_box(440.0),
                black_box(0.5),
                black_box(0),
                black_box(vec3(5.0, 0.0, 0.0)),
                black_box(0.8),
            );
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

