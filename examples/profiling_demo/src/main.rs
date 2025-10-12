/*!
# AstraWeave Profiling Demo

Demonstrates Tracy profiling integration with AstraWeave engine.

This demo creates a realistic game scenario with 1,000 entities and instruments
all major systems (ECS, AI, Physics, Rendering) to capture performance data.

## Usage

1. **Download Tracy**: https://github.com/wolfpld/tracy/releases
2. **Start Tracy server**: Run `Tracy.exe` (Windows) or `tracy-profiler` (Linux/Mac)
3. **Build with profiling**:
   ```
   cargo build -p profiling_demo --release --features profiling
   ```
4. **Run demo**:
   ```
   cargo run -p profiling_demo --release --features profiling
   ```
5. **Capture data**: Tracy will automatically connect and show real-time profiling data

## What to Look For

- **Frame Time**: Should be < 16.67ms for 60 FPS
- **System Breakdown**:
  - ECS tick: Archetype iteration, component access
  - AI planning: GOAP/BT execution
  - Physics: Collision detection, rigid body simulation
  - Rendering: Mesh submission, draw calls
- **Hot Spots**: Functions consuming >5% frame time
- **Memory**: Allocation patterns, heap churn

## Key Metrics

The demo tracks:
- **Entity Count**: 1,000 entities (500 AI agents, 500 physics objects)
- **Component Churn**: Add/remove operations
- **AI Planning**: GOAP cache hit rate
- **Physics**: Collision pairs, contact generation
- **Render**: Draw calls, vertex count

Press ESC to exit.
*/

use astraweave_ecs::{App, Entity, Query2, SystemStage, World};
use astraweave_physics::{SpatialHash, AABB};
use astraweave_profiling::{frame_mark, message, plot, span};
use anyhow::Result;
use glam::Vec3;
use std::time::Instant;

// Command-line argument parsing
fn parse_args() -> (usize, usize) {
    let args: Vec<String> = std::env::args().collect();
    let mut entity_count = 1000; // Default: stress test
    let mut max_frames = 1000;   // Default: 1000 frames (~16s @ 60 FPS)
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--entities" | "-e" => {
                if i + 1 < args.len() {
                    entity_count = args[i + 1].parse().unwrap_or(1000);
                    i += 1;
                }
            }
            "--frames" | "-f" => {
                if i + 1 < args.len() {
                    max_frames = args[i + 1].parse().unwrap_or(1000);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("AstraWeave Profiling Demo - Tracy Performance Analysis");
                println!("\nUsage:");
                println!("  profiling_demo [OPTIONS]");
                println!("\nOptions:");
                println!("  --entities, -e <N>   Number of entities to spawn (default: 1000)");
                println!("  --frames, -f <N>     Number of frames to capture (default: 1000)");
                println!("  --help, -h           Show this help message");
                println!("\nExamples:");
                println!("  profiling_demo --entities 200   # Low load baseline");
                println!("  profiling_demo --entities 500   # Medium load (target capacity)");
                println!("  profiling_demo --entities 1000  # High load (stress test)");
                println!("  profiling_demo -e 500 -f 2000   # 500 entities, 2000 frames");
                println!("\nTracy Setup:");
                println!("  1. Download Tracy from: https://github.com/wolfpld/tracy/releases");
                println!("  2. Start Tracy server (Tracy.exe) BEFORE running this demo");
                println!("  3. Tracy will auto-connect on localhost:8086");
                println!("  4. Capture profiling data in real-time");
                println!("  5. Save trace: File > Save Trace > baseline_<entities>.tracy");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}. Use --help for usage.", args[i]);
            }
        }
        i += 1;
    }
    
    (entity_count, max_frames)
}

/// Simple position component
#[derive(Debug, Clone, Copy)]
struct Position(Vec3);

/// Simple velocity component
#[derive(Debug, Clone, Copy)]
struct Velocity(Vec3);

/// AI agent marker
#[derive(Debug, Clone, Copy)]
struct AIAgent {
    state: AgentState,
}

#[derive(Debug, Clone, Copy)]
enum AgentState {
    Idle,
    Moving,
    Attacking,
}

/// Physics body marker
#[derive(Debug, Clone, Copy)]
struct RigidBody {
    mass: f32,
}

/// Renderable marker
#[derive(Debug, Clone, Copy)]
struct Renderable {
    mesh_id: u32,
}

/// Game state
struct GameState {
    app: App,
    entity_count: usize,
    frame_count: u64,
    start_time: Instant,
}

impl GameState {
    fn new(entity_count: usize) -> Result<Self> {
        span!("GameState::new");

        let mut app = App::new();

        // Register systems
        app.add_system(SystemStage::PRE_SIMULATION, ai_perception_system);
        app.add_system(SystemStage::AI_PLANNING, ai_planning_system);
        app.add_system(SystemStage::SIMULATION, movement_system);
        app.add_system(SystemStage::PHYSICS, physics_system);
        app.add_system(SystemStage::POST_SIMULATION, cleanup_system);
        app.add_system(SystemStage::PRESENTATION, rendering_system);

        // Spawn entities
        message!("Spawning {} entities", entity_count);
        {
            span!("entity_spawn");

            for i in 0..entity_count {
                let pos = Position(Vec3::new(
                    (i % 32) as f32 * 2.0,
                    ((i / 32) % 32) as f32 * 2.0,
                    (i / 1024) as f32 * 2.0,
                ));
                let vel = Velocity(Vec3::new(
                    (i as f32 * 0.1).sin() * 0.1,
                    (i as f32 * 0.1).cos() * 0.1,
                    0.0,
                ));

                let entity = app.world.spawn();
                app.world.insert(entity, pos);
                app.world.insert(entity, vel);

                if i % 2 == 0 {
                    // AI agent
                    app.world.insert(entity, AIAgent {
                        state: AgentState::Idle,
                    });
                    app.world.insert(entity, Renderable { mesh_id: 1 });
                } else {
                    // Physics object
                    app.world.insert(entity, RigidBody { mass: 1.0 });
                    app.world.insert(entity, Renderable { mesh_id: 2 });
                }
            }
        }

        message!("Entities spawned: {}", entity_count);

        Ok(Self {
            app,
            entity_count,
            frame_count: 0,
            start_time: Instant::now(),
        })
    }

    fn tick(&mut self) -> Result<()> {
        span!("GameState::tick");

        // Update frame metrics
        self.frame_count += 1;
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let fps = self.frame_count as f64 / elapsed;

        plot!("FPS", fps);
        plot!("EntityCount", self.entity_count as f64);
        plot!("FrameNumber", self.frame_count as f64);

        // Run ECS systems
        {
            span!("schedule_run");
            self.app.schedule.run(&mut self.app.world);
        }

        frame_mark!();

        Ok(())
    }
}

// System implementations (instrumented with profiling)

fn ai_perception_system(world: &mut World) {
    span!("ai_perception");

    let mut count = 0;
    // Simulate AI perception queries
    let query = Query2::<Position, AIAgent>::new(world);
    for (_entity, _pos, _agent) in query {
        count += 1;
        // Simulate perception work
        let _ = (count as f32).sin();
    }

    plot!("AI.Agents", count as f64);
}

fn ai_planning_system(world: &mut World) {
    span!("ai_planning");

    let mut planning_count = 0;
    let mut cache_hits = 0;

    // Simulate AI planning with cache - need to collect entities first to avoid borrow conflicts
    let entities: Vec<Entity> = {
        let query = Query2::<Position, AIAgent>::new(world);
        query.map(|(entity, _, _)| entity).collect()
    };

    for entity in entities {
        if let Some(_agent) = world.get_mut::<AIAgent>(entity) {
            planning_count += 1;

            // Simulate GOAP planning with cache hits
            if planning_count % 10 == 0 {
                // Cache miss - expensive planning
                span!("goap_planning");
                for _ in 0..100 {
                    let _ = (planning_count as f32).sqrt();
                }
            } else {
                // Cache hit - cheap lookup
                cache_hits += 1;
            }

            // State transitions
            let agent = world.get_mut::<AIAgent>(entity).unwrap();
            agent.state = match agent.state {
                AgentState::Idle => AgentState::Moving,
                AgentState::Moving => AgentState::Attacking,
                AgentState::Attacking => AgentState::Idle,
            };
        }
    }

    let cache_hit_rate = if planning_count > 0 {
        (cache_hits as f64 / planning_count as f64) * 100.0
    } else {
        0.0
    };

    plot!("AI.PlanningOperations", planning_count as f64);
    plot!("AI.CacheHitRate", cache_hit_rate);
}

fn movement_system(world: &mut World) {
    span!("movement");

    let mut moved_count = 0;

    // Collect entities into contiguous arrays for SIMD processing
    let (entities, mut positions, velocities) = {
        let query = Query2::<Position, Velocity>::new(world);
        let data: Vec<(Entity, Vec3, Vec3)> = query
            .map(|(entity, pos, vel)| (entity, pos.0, vel.0))
            .collect();
        
        let count = data.len();
        let mut ents = Vec::with_capacity(count);
        let mut pos_vec = Vec::with_capacity(count);
        let mut vel_vec = Vec::with_capacity(count);
        
        for (e, p, v) in data {
            ents.push(e);
            pos_vec.push(p);
            vel_vec.push(v);
        }
        
        (ents, pos_vec, vel_vec)
    };

    // SIMD-optimized position update (Day 3 baseline - proven fastest)
    astraweave_math::simd_movement::update_positions_simd(&mut positions[..], &velocities[..], 1.0);
    
    // Write back to ECS and apply bounds wrapping
    for (entity, new_pos) in entities.iter().zip(positions.iter_mut()) {
        if let Some(pos) = world.get_mut::<Position>(*entity) {
            pos.0 = *new_pos;
            moved_count += 1;

            // Wrap positions
            if pos.0.x.abs() > 64.0 {
                pos.0.x = -pos.0.x.signum() * 64.0;
                new_pos.x = pos.0.x; // Update local copy for consistency
            }
            if pos.0.y.abs() > 64.0 {
                pos.0.y = -pos.0.y.signum() * 64.0;
                new_pos.y = pos.0.y;
            }
        }
    }

    plot!("Movement.Updates", moved_count as f64);
}

fn physics_system(world: &mut World) {
    span!("physics");

    let mut collision_checks = 0;
    let mut collisions = 0;

    // Spatial hash collision detection (optimized)
    {
        span!("collision_detection");

        // Collect entities with positions
        let entities_data: Vec<(Entity, Vec3)> = {
            let query = Query2::<Position, RigidBody>::new(world);
            query.map(|(entity, pos, _)| (entity, pos.0)).collect()
        };

        if !entities_data.is_empty() {
            // Build entity index map for O(1) lookups (fix for O(n²) regression)
            let entity_map: std::collections::HashMap<u64, (usize, Vec3)> = entities_data.iter()
                .enumerate()
                .map(|(i, (e, pos))| (e.id(), (i, *pos)))
                .collect();

            // Build spatial hash grid (use entity ID as u32)
            let mut grid = SpatialHash::new(2.0); // Cell size = 2× collision radius
            
            for (entity, pos) in &entities_data {
                let aabb = AABB::from_sphere(*pos, 0.5); // Collision radius = 0.5
                grid.insert(entity.id(), aabb);
            }

            // Query for collisions using spatial hash
            for (i, (_entity, pos)) in entities_data.iter().enumerate() {
                // Query radius must match collision distance (1.0), not object radius (0.5)!
                let query_aabb = AABB::from_sphere(*pos, 1.0);  // collision_distance = 1.0
                let candidates = grid.query(query_aabb);

                for &candidate_id in &candidates {
                    // O(1) lookup instead of O(n) find!
                    if let Some(&(j, candidate_pos)) = entity_map.get(&candidate_id) {
                        // Only check each pair once (i < j)
                        if i < j {
                            collision_checks += 1;

                            let dist = pos.distance(candidate_pos);
                            if dist < 1.0 {
                                collisions += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    plot!("Physics.CollisionChecks", collision_checks as f64);
    plot!("Physics.Collisions", collisions as f64);
}

fn cleanup_system(_world: &mut World) {
    span!("cleanup");
    // Simulate cleanup work
}

fn rendering_system(world: &mut World) {
    span!("rendering");

    let mut draw_calls = 0;
    let mut vertex_count = 0;

    // Simulate rendering
    {
        span!("render_submit");

        let query = Query2::<Renderable, Position>::new(world);
        for (_, renderable, _pos) in query {
            draw_calls += 1;
            // Simulate vertex count per mesh
            vertex_count += if renderable.mesh_id == 1 { 36 } else { 64 };
        }
    }

    plot!("Render.DrawCalls", draw_calls as f64);
    plot!("Render.VertexCount", vertex_count as f64);
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let (entity_count, max_frames) = parse_args();
    
    println!("=== AstraWeave Profiling Demo ===");
    println!("Tracy profiling enabled: {}", astraweave_profiling::Profiler::is_enabled());
    println!("Configuration:");
    println!("  Entities: {}", entity_count);
    println!("  Frames: {} (~{:.1}s @ 60 FPS)", max_frames, max_frames as f64 / 60.0);
    println!("\nControls:");
    println!("  Run for {} frames then exit", max_frames);
    println!("\nStart Tracy server before running for best results.");
    println!("Tracy will auto-connect and capture profiling data.\n");

    message!("=== Profiling Demo Start === Entities: {}, Frames: {}", entity_count, max_frames);

    // Create game state
    let mut game = GameState::new(entity_count)?;

    message!("Game initialized with {} entities", entity_count);

    // Run profiling loop
    let start = Instant::now();

    for frame in 0..max_frames {
        if frame % 100 == 0 {
            println!("Frame {}/{}", frame, max_frames);
            message!("Milestone: Frame {}", frame);
        }

        game.tick()?;
    }

    let elapsed = start.elapsed();
    let avg_fps = max_frames as f64 / elapsed.as_secs_f64();
    let avg_frame_ms = elapsed.as_millis() as f64 / max_frames as f64;

    println!("\n=== Profiling Complete ===");
    println!("Configuration: {} entities, {} frames", entity_count, max_frames);
    println!("Total time: {:.2}s", elapsed.as_secs_f64());
    println!("Average FPS: {:.2}", avg_fps);
    println!("Average frame time: {:.3}ms", avg_frame_ms);
    println!("\nCheck Tracy for detailed profiling data!");
    println!("Save trace: File > Save Trace > baseline_{}.tracy", entity_count);

    message!("=== Profiling Demo Complete === Entities: {}, FPS: {:.2}, Frame: {:.3}ms", entity_count, avg_fps, avg_frame_ms);

    Ok(())
}
