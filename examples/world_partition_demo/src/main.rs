//! World Partition Demo
//!
//! This demo showcases the world partition system with a procedurally generated
//! 10x10 grid of cells (10km²) with random entities. It simulates a camera flythrough
//! and monitors memory usage and performance.

use astraweave_scene::world_partition::{GridConfig, GridCoord, WorldPartition, AABB};
use astraweave_scene::streaming::{StreamingConfig, WorldPartitionManager, StreamingEvent};
use astraweave_scene::partitioned_scene::PartitionedScene;
use glam::Vec3;
use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Generate a procedural demo world with entities distributed across cells
async fn generate_demo_world(partition: &Arc<RwLock<WorldPartition>>, grid_size: i32) {
    let mut rng = rand::thread_rng();
    let mut partition_write = partition.write().await;

    println!("Generating demo world with {}x{} cells...", grid_size, grid_size);

    for x in 0..grid_size {
        for z in 0..grid_size {
            let coord = GridCoord::new(x, 0, z);
            let cell = partition_write.get_or_create_cell(coord);

            // Add random number of entities per cell (simulating foliage, rocks, etc.)
            let entity_count = rng.gen_range(10..50);
            for _ in 0..entity_count {
                // In a real implementation, these would be actual entity IDs
                // For demo purposes, we just use dummy u64 values
                #[cfg(not(feature = "ecs"))]
                cell.entities.push(rng.gen());
            }

            // Add some asset references
            cell.assets.push(astraweave_scene::world_partition::AssetRef {
                path: format!("terrain/cell_{}_{}.mesh", x, z),
                asset_type: astraweave_scene::world_partition::AssetType::Mesh,
            });
            cell.assets.push(astraweave_scene::world_partition::AssetRef {
                path: format!("terrain/cell_{}_{}.tex", x, z),
                asset_type: astraweave_scene::world_partition::AssetType::Texture,
            });
        }
    }

    println!("Generated {} cells with entities and assets", grid_size * grid_size);
}

/// Simulate a camera flythrough path
fn generate_camera_path(duration_secs: f32, speed: f32) -> Vec<(f32, Vec3)> {
    let mut path = Vec::new();
    let steps = (duration_secs * 60.0) as usize; // 60 updates per second

    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let time = t * duration_secs;

        // Circular path around the world
        let radius = 2000.0; // 2km radius
        let angle = t * std::f32::consts::PI * 2.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        let y = 50.0; // 50m above ground

        path.push((time, Vec3::new(x, y, z)));
    }

    path
}

/// Performance monitoring
struct PerformanceMonitor {
    frame_times: Vec<Duration>,
    max_frame_time: Duration,
    total_frames: usize,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            frame_times: Vec::new(),
            max_frame_time: Duration::ZERO,
            total_frames: 0,
        }
    }

    fn record_frame(&mut self, duration: Duration) {
        self.frame_times.push(duration);
        if duration > self.max_frame_time {
            self.max_frame_time = duration;
        }
        self.total_frames += 1;

        // Keep only last 60 frames for rolling average
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
    }

    fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        let sum: Duration = self.frame_times.iter().sum();
        sum / self.frame_times.len() as u32
    }

    fn report(&self) {
        let avg = self.average_frame_time();
        let avg_ms = avg.as_secs_f64() * 1000.0;
        let max_ms = self.max_frame_time.as_secs_f64() * 1000.0;
        let fps = if avg_ms > 0.0 { 1000.0 / avg_ms } else { 0.0 };

        println!("\n=== Performance Report ===");
        println!("Total frames: {}", self.total_frames);
        println!("Average frame time: {:.2}ms ({:.1} FPS)", avg_ms, fps);
        println!("Max frame time: {:.2}ms", max_ms);
        println!("Stalls >100ms: {}", self.frame_times.iter().filter(|d| d.as_millis() > 100).count());
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== AstraWeave World Partition Demo ===\n");

    // Configuration
    let grid_config = GridConfig {
        cell_size: 1000.0, // 1km cells
        world_bounds: (-5000.0, 5000.0, -5000.0, 5000.0), // 10km x 10km
    };

    let streaming_config = StreamingConfig {
        max_active_cells: 25,
        lru_cache_size: 5,
        streaming_radius: 2500.0, // 2.5km radius
        max_concurrent_loads: 4,
    };

    println!("Grid Configuration:");
    println!("  Cell size: {}m", grid_config.cell_size);
    println!("  World bounds: {:?}", grid_config.world_bounds);
    println!("\nStreaming Configuration:");
    println!("  Max active cells: {}", streaming_config.max_active_cells);
    println!("  LRU cache size: {}", streaming_config.lru_cache_size);
    println!("  Streaming radius: {}m", streaming_config.streaming_radius);
    println!("  Max concurrent loads: {}\n", streaming_config.max_concurrent_loads);

    // Create partitioned scene
    let mut scene = PartitionedScene::new(grid_config, streaming_config);

    // Add event listener for debugging
    scene.manager.add_event_listener(|event| {
        match event {
            StreamingEvent::CellLoaded(coord) => {
                println!("  [LOADED] Cell ({}, {}, {})", coord.x, coord.y, coord.z);
            }
            StreamingEvent::CellUnloaded(coord) => {
                println!("  [UNLOADED] Cell ({}, {}, {})", coord.x, coord.y, coord.z);
            }
            StreamingEvent::CellLoadFailed(coord, error) => {
                eprintln!("  [FAILED] Cell ({}, {}, {}): {}", coord.x, coord.y, coord.z, error);
            }
            _ => {}
        }
    });

    // Generate demo world
    generate_demo_world(&scene.partition, 10).await;

    // Generate camera path
    let camera_path = generate_camera_path(10.0, 100.0); // 10 seconds at 100 m/s
    println!("Generated camera path with {} waypoints\n", camera_path.len());

    // Performance monitoring
    let mut perf_monitor = PerformanceMonitor::new();

    println!("Starting camera flythrough...\n");

    // Simulate camera movement and streaming
    for (time, camera_pos) in camera_path {
        let frame_start = Instant::now();

        // Update streaming
        scene.update_streaming(camera_pos).await?;

        let frame_time = frame_start.elapsed();
        perf_monitor.record_frame(frame_time);

        // Print status every second
        if (time * 10.0) as i32 % 10 == 0 {
            let metrics = scene.metrics();
            println!("Time: {:.1}s | Camera: ({:.0}, {:.0}, {:.0})", 
                time, camera_pos.x, camera_pos.y, camera_pos.z);
            println!("  Active cells: {} | Loading: {} | Cached: {}", 
                metrics.active_cells, metrics.loading_cells, metrics.cached_cells);
            println!("  Memory: {:.2} MB | Loads: {} | Unloads: {}", 
                metrics.memory_usage_bytes as f64 / 1_048_576.0,
                metrics.total_loads, metrics.total_unloads);
        }

        // Simulate frame timing (60 FPS target)
        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    println!("\n=== Final Statistics ===");
    let final_metrics = scene.metrics();
    println!("Total loads: {}", final_metrics.total_loads);
    println!("Total unloads: {}", final_metrics.total_unloads);
    println!("Failed loads: {}", final_metrics.failed_loads);
    println!("Final memory usage: {:.2} MB", final_metrics.memory_usage_bytes as f64 / 1_048_576.0);
    println!("Active cells: {}", final_metrics.active_cells);

    perf_monitor.report();

    // Verify acceptance criteria
    println!("\n=== Acceptance Criteria Verification ===");
    
    let memory_mb = final_metrics.memory_usage_bytes as f64 / 1_048_576.0;
    let memory_ok = memory_mb < 500.0;
    println!("✓ Memory usage < 500MB: {} ({:.2} MB)", 
        if memory_ok { "PASS" } else { "FAIL" }, memory_mb);

    let max_frame_ms = perf_monitor.max_frame_time.as_secs_f64() * 1000.0;
    let no_stalls = max_frame_ms < 100.0;
    println!("✓ No stalls >100ms: {} (max: {:.2}ms)", 
        if no_stalls { "PASS" } else { "FAIL" }, max_frame_ms);

    let seamless = final_metrics.failed_loads == 0;
    println!("✓ Seamless exploration (no failed loads): {}", 
        if seamless { "PASS" } else { "FAIL" });

    if memory_ok && no_stalls && seamless {
        println!("\n🎉 All acceptance criteria met!");
    } else {
        println!("\n⚠️  Some acceptance criteria not met");
    }

    Ok(())
}