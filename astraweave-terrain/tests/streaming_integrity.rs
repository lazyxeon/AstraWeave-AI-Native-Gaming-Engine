//! Streaming integrity soak test
//!
//! Validates terrain streaming performance over extended duration:
//! - 1,024 tick duration (17 minutes @ 60 FPS)
//! - Randomized camera movement (walk, sprint, teleport)
//! - p99 frame hitch <2ms (60 FPS sustained)
//! - Memory delta <6% from peak (no leaks)
//! - No missing chunks in view frustum

use astraweave_terrain::{
    BackgroundChunkLoader, ChunkId, LodManager, StreamingConfig, StreamingDiagnostics,
    WorldConfig, WorldGenerator,
};
use glam::Vec3;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Camera movement pattern for testing
#[derive(Debug, Clone, Copy)]
enum MovementPattern {
    Walk,      // 5 m/s
    Sprint,    // 15 m/s
    Teleport,  // Instant jump 500m
}

/// Camera path generator for soak testing
struct CameraPathGenerator {
    rng: StdRng,
    position: Vec3,
    direction: Vec3,
}

impl CameraPathGenerator {
    fn new(seed: u64, start_pos: Vec3) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            position: start_pos,
            direction: Vec3::X,
        }
    }
    
    /// Generate next camera position based on random pattern
    fn step(&mut self, dt: f32) -> (Vec3, Vec3) {
        let pattern = match self.rng.random_range(0..100) {
            0..70 => MovementPattern::Walk,
            70..95 => MovementPattern::Sprint,
            _ => MovementPattern::Teleport,
        };
        
        // Random direction change (10% chance)
        if self.rng.random_range(0..10) == 0 {
            let angle = self.rng.random_range(0.0..std::f32::consts::TAU);
            self.direction = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize();
        }
        
        // Apply movement
        match pattern {
            MovementPattern::Walk => {
                self.position += self.direction * 5.0 * dt;
            }
            MovementPattern::Sprint => {
                self.position += self.direction * 15.0 * dt;
            }
            MovementPattern::Teleport => {
                let offset = Vec3::new(
                    self.rng.random_range(-500.0..500.0),
                    0.0,
                    self.rng.random_range(-500.0..500.0),
                );
                self.position += offset;
            }
        }
        
        (self.position, self.direction)
    }
}

/// Soak test configuration
struct SoakTestConfig {
    duration_ticks: usize,
    target_fps: f32,
    hitch_threshold_ms: f32,
    max_memory_delta_percent: f32,
    chunk_size: f32,
    view_distance: u32,
}

impl Default for SoakTestConfig {
    fn default() -> Self {
        Self {
            duration_ticks: 1024,
            target_fps: 60.0,
            hitch_threshold_ms: 2.0,
            max_memory_delta_percent: 6.0,
            chunk_size: 256.0,
            view_distance: 8,
        }
    }
}

/// Soak test results
#[derive(Debug)]
#[allow(dead_code)]  // Used for display in test output
struct SoakTestResults {
    total_ticks: usize,
    average_frame_ms: f32,
    p99_frame_ms: f32,
    hitch_count: usize,
    hitch_rate: f32,
    peak_memory_mb: f32,
    final_memory_mb: f32,
    memory_delta_percent: f32,
    chunks_loaded_total: usize,
    chunks_unloaded_total: usize,
    missing_chunk_count: usize,
}

impl SoakTestResults {
    fn passed(&self, config: &SoakTestConfig) -> bool {
        self.p99_frame_ms < config.hitch_threshold_ms
            && self.memory_delta_percent.abs() < config.max_memory_delta_percent
            && self.missing_chunk_count == 0
    }
}

/// Run streaming soak test
async fn run_soak_test(config: SoakTestConfig) -> SoakTestResults {
    // Setup world generator
    let world_config = WorldConfig {
        chunk_size: config.chunk_size,
        seed: 42,
        ..Default::default()
    };
    let world_gen = Arc::new(RwLock::new(WorldGenerator::new(world_config)));
    
    // Setup streaming components
    let streaming_config = StreamingConfig {
        chunk_size: config.chunk_size,
        view_distance: config.view_distance,
        max_loaded_chunks: 256,
        prefetch_distance: 4,
        max_concurrent_loads: 8,  // Increased from 4 to 8
        adaptive_throttle_threshold_ms: 10.0,  // NEW: throttle if frame >10ms
        throttled_concurrent_loads: 2,  // NEW: reduce to 2 when throttling
    };
    
    let loader = BackgroundChunkLoader::new(streaming_config, world_gen.clone());
    let mut lod_manager = LodManager::new(Default::default(), config.chunk_size);
    let mut diagnostics = StreamingDiagnostics::new(
        config.hitch_threshold_ms,
        100, // 100-frame history
    );
    
    // Camera path generator
    let mut camera_path = CameraPathGenerator::new(12345, Vec3::new(128.0, 50.0, 128.0));
    
    let dt = 1.0 / config.target_fps;
    let mut chunks_loaded_total = 0;
    let mut chunks_unloaded_total = 0;
    let mut missing_chunk_count = 0;
    
    // Run soak test
    for tick in 0..config.duration_ticks {
        let frame_start = std::time::Instant::now();
        
        // Update camera position
        let (camera_pos, camera_dir) = camera_path.step(dt);
        loader.update_camera(camera_pos, camera_dir).await;
        diagnostics.update_camera(camera_pos);
        
        // Request chunks around camera (uses internal camera state)
        loader.request_chunks_around_camera().await;
        
        // Process background loading (now uses internal world_gen)
        loader.process_load_queue().await;
        
        // Give async tasks time to complete (simulate frame budget)
        std::thread::sleep(std::time::Duration::from_millis(5));
        
        let loaded_this_frame = loader.collect_completed_chunks().await;
        chunks_loaded_total += loaded_this_frame;
        
        // Unload distant chunks
        let unloaded = loader.unload_distant_chunks(camera_pos).await;
        chunks_unloaded_total += unloaded;
        
        // Update LOD
        let loaded_chunks: Vec<ChunkId> = loader.get_loaded_chunk_ids().await;
        lod_manager.update_all_chunks(&loaded_chunks, camera_pos);
        
        // Check for missing chunks in frustum
        let view_chunks = ChunkId::get_chunks_in_radius(
            camera_pos,
            config.view_distance,
            config.chunk_size,
        );
        for chunk_id in &view_chunks {
            if !loaded_chunks.contains(chunk_id) && !loader.is_loading(*chunk_id).await {
                missing_chunk_count += 1;
            }
        }
        
        // Update diagnostics
        let frame_time_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        diagnostics.record_frame(frame_time_ms);
        
        // Update loader with frame time for adaptive throttling (Phase 2 optimization)
        loader.set_frame_time(frame_time_ms).await;
        
        let stats = loader.get_stats().await;
        diagnostics.update_streaming_stats(stats);
        diagnostics.update_lod_stats(lod_manager.get_stats());
        diagnostics.update_memory(loaded_chunks.len(), 1024 * 1024); // Assume 1MB per chunk
        
        // Progress report every 128 ticks
        if tick % 128 == 0 && tick > 0 {
            let report = diagnostics.generate_report();
            println!(
                "[Tick {}/{}] Avg: {:.2}ms, p99: {:.2}ms, Hitches: {}, Memory: {:.1}MB, Chunks: {}",
                tick,
                config.duration_ticks,
                report.frame_stats.average_ms,
                report.frame_stats.p99_ms,
                report.frame_stats.hitch_count,
                report.memory.total_mb(),
                report.chunk_counts.loaded,
            );
        }
    }
    
    // Generate final results
    let final_report = diagnostics.generate_report();
    let memory_stats = diagnostics.memory_stats();
    
    SoakTestResults {
        total_ticks: config.duration_ticks,
        average_frame_ms: final_report.frame_stats.average_ms,
        p99_frame_ms: final_report.frame_stats.p99_ms,
        hitch_count: final_report.frame_stats.hitch_count,
        hitch_rate: final_report.frame_stats.hitch_rate,
        peak_memory_mb: memory_stats.peak_bytes as f32 / (1024.0 * 1024.0),
        final_memory_mb: memory_stats.total_mb(),
        memory_delta_percent: memory_stats.delta_from_peak_percent(),
        chunks_loaded_total,
        chunks_unloaded_total,
        missing_chunk_count,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn streaming_soak_test_1024_ticks() {
    let config = SoakTestConfig::default();
    
    println!("\n=== Streaming Integrity Soak Test ===");
    println!("Duration: {} ticks ({:.1} minutes @ 60 FPS)", 
        config.duration_ticks,
        config.duration_ticks as f32 / 60.0 / 60.0
    );
    println!("Hitch threshold: {:.2}ms", config.hitch_threshold_ms);
    println!("Max memory delta: {:.1}%\n", config.max_memory_delta_percent);
    
    let results = run_soak_test(config).await;
    
    println!("\n=== Results ===");
    println!("Average frame time: {:.2}ms ({:.1} FPS)", 
        results.average_frame_ms,
        1000.0 / results.average_frame_ms
    );
    println!("p99 frame time: {:.2}ms", results.p99_frame_ms);
    println!("Hitch count: {} ({:.2}% of frames)", 
        results.hitch_count,
        results.hitch_rate
    );
    println!("Peak memory: {:.1}MB", results.peak_memory_mb);
    println!("Final memory: {:.1}MB", results.final_memory_mb);
    println!("Memory delta: {:.2}%", results.memory_delta_percent);
    println!("Chunks loaded: {}", results.chunks_loaded_total);
    println!("Chunks unloaded: {}", results.chunks_unloaded_total);
    println!("Missing chunks: {}", results.missing_chunk_count);
    
    // Validate acceptance criteria
    let passed = results.passed(&SoakTestConfig::default());
    println!("\n=== Status: {} ===\n", if passed { "✅ PASSED" } else { "❌ FAILED" });
    
    // Assert success criteria
    assert!(
        results.p99_frame_ms < SoakTestConfig::default().hitch_threshold_ms,
        "p99 frame time {:.2}ms exceeds threshold {:.2}ms",
        results.p99_frame_ms,
        SoakTestConfig::default().hitch_threshold_ms
    );
    
    assert!(
        results.memory_delta_percent.abs() < SoakTestConfig::default().max_memory_delta_percent,
        "Memory delta {:.2}% exceeds threshold {:.1}%",
        results.memory_delta_percent,
        SoakTestConfig::default().max_memory_delta_percent
    );
    
    assert_eq!(
        results.missing_chunk_count, 0,
        "Found {} missing chunks in view frustum",
        results.missing_chunk_count
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn streaming_quick_validation() {
    // Quick 64-tick validation for CI
    let config = SoakTestConfig {
        duration_ticks: 64,
        ..Default::default()
    };
    
    let results = run_soak_test(config).await;
    
    // Just ensure no panics and basic functionality
    assert!(results.average_frame_ms > 0.0);
    assert!(results.chunks_loaded_total > 0);
}
