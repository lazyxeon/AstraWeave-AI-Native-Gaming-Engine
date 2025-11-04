//! BT Patrol Demo - Behavior Tree AI demonstration
//!
//! Demonstrates: Patrol → Detect (LOS) → Chase → Attack behavior pattern
//!
//! Controls:
//! - Space: Play/Pause simulation
//! - [/]: Slow/Fast time scale
//! - R: Reset to initial seed
//! - Q: Quit
//!
//! Deterministic: Fixed seed (42) ensures reproducible behavior

use astraweave_core::{IVec2, Team, World};
use rand::{Rng, SeedableRng};
use std::io::{self, Read};

/// Simple Behavior Tree state for patrol demo
#[derive(Debug, Clone, PartialEq)]
enum BtState {
    Patrol,
    Detect,
    Chase,
    Attack,
}

/// Demo state
struct DemoState {
    world: World,
    agent_id: u32,
    target_id: u32,
    bt_state: BtState,
    patrol_waypoints: Vec<IVec2>,
    current_waypoint: usize,
    tick_count: u64,
    paused: bool,
    time_scale: f32,
    #[allow(dead_code)]
    seed: u64,
}

impl DemoState {
    fn new(seed: u64) -> Self {
        let mut world = World::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        // Create patrol waypoints (deterministic)
        let patrol_waypoints = vec![
            IVec2 { x: 5, y: 5 },
            IVec2 { x: 15, y: 5 },
            IVec2 { x: 15, y: 15 },
            IVec2 { x: 5, y: 15 },
        ];

        // Spawn target (player dummy) at random position
        let target_x = rng.random_range(8..12);
        let target_y = rng.random_range(8..12);
        let target_id = world.spawn(
            "Target",
            IVec2 {
                x: target_x,
                y: target_y,
            },
            Team { id: 0 },
            100,
            0,
        );

        // Spawn BT agent at first waypoint
        let agent_id = world.spawn("BTAgent", patrol_waypoints[0], Team { id: 1 }, 80, 10);

        // Add some obstacles (deterministic)
        for _i in 0..5 {
            let x = rng.random_range(2..18);
            let y = rng.random_range(2..18);
            world.obstacles.insert((x, y));
        }

        Self {
            world,
            agent_id,
            target_id,
            bt_state: BtState::Patrol,
            patrol_waypoints,
            current_waypoint: 0,
            tick_count: 0,
            paused: false,
            time_scale: 1.0,
            seed,
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        *self = Self::new(self.seed);
    }

    fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        self.world.tick(dt * self.time_scale);
        self.tick_count += 1;

        // Get positions
        let agent_pos = self
            .world
            .pos_of(self.agent_id)
            .expect("Agent entity should have Position component");
        let target_pos = self
            .world
            .pos_of(self.target_id)
            .expect("Target entity should have Position component");

        // Simple LOS check (Manhattan distance for simplicity)
        let distance = (agent_pos.x - target_pos.x).abs() + (agent_pos.y - target_pos.y).abs();
        let has_los = distance <= 6;

        // Update BT state
        match self.bt_state {
            BtState::Patrol => {
                // Check if reached waypoint
                let waypoint = self.patrol_waypoints[self.current_waypoint];
                if agent_pos == waypoint {
                    self.current_waypoint =
                        (self.current_waypoint + 1) % self.patrol_waypoints.len();
                }

                // Check for detection
                if has_los {
                    self.bt_state = BtState::Detect;
                } else {
                    // Move toward waypoint
                    self.move_toward(waypoint);
                }
            }
            BtState::Detect => {
                if !has_los {
                    self.bt_state = BtState::Patrol;
                } else if distance <= 2 {
                    self.bt_state = BtState::Attack;
                } else {
                    self.bt_state = BtState::Chase;
                }
            }
            BtState::Chase => {
                if !has_los {
                    self.bt_state = BtState::Patrol;
                } else if distance <= 2 {
                    self.bt_state = BtState::Attack;
                } else {
                    self.move_toward(target_pos);
                }
            }
            BtState::Attack => {
                if distance > 2 {
                    self.bt_state = BtState::Chase;
                } else {
                    // Deal damage
                    if let Some(hp) = self.world.health_mut(self.target_id) {
                        hp.hp = (hp.hp - 5).max(0);
                    }
                }
            }
        }
    }

    fn move_toward(&mut self, target: IVec2) {
        let agent_pos = self
            .world
            .pos_of(self.agent_id)
            .expect("Agent entity should have Position component");

        // Simple movement (one step toward target)
        let dx = (target.x - agent_pos.x).signum();
        let dy = (target.y - agent_pos.y).signum();

        let new_pos = IVec2 {
            x: agent_pos.x + dx,
            y: agent_pos.y + dy,
        };

        // Check if not blocked
        if !self.world.obstacle(new_pos) {
            if let Some(pose) = self.world.pose_mut(self.agent_id) {
                pose.pos = new_pos;
            }
        }
    }

    fn render_hud(&self) {
        println!("\n=== BT PATROL DEMO ===");
        println!("Mode: BehaviorTree");
        println!("Tick: {}", self.tick_count);
        println!(
            "Time: {:.2}s (scale: {:.1}x)",
            self.world.t, self.time_scale
        );
        println!("Current Node: {:?}", self.bt_state);
        println!("Status: {}", if self.paused { "PAUSED" } else { "RUNNING" });

        // Agent state
        let agent_pos = self
            .world
            .pos_of(self.agent_id)
            .expect("Agent entity should have Position component");
        let agent_hp = self
            .world
            .health(self.agent_id)
            .expect("Agent entity should have Health component")
            .hp;
        println!(
            "\nAgent: pos=({}, {}), hp={}",
            agent_pos.x, agent_pos.y, agent_hp
        );

        // Target state
        let target_pos = self
            .world
            .pos_of(self.target_id)
            .expect("Target entity should have Position component");
        let target_hp = self
            .world
            .health(self.target_id)
            .expect("Target entity should have Health component")
            .hp;
        println!(
            "Target: pos=({}, {}), hp={}",
            target_pos.x, target_pos.y, target_hp
        );

        // Distance/LOS
        let distance = (agent_pos.x - target_pos.x).abs() + (agent_pos.y - target_pos.y).abs();
        let has_los = distance <= 6;
        println!("Distance: {}, LOS: {}", distance, has_los);

        // Waypoint info
        if self.bt_state == BtState::Patrol {
            let waypoint = self.patrol_waypoints[self.current_waypoint];
            println!("Next Waypoint: ({}, {})", waypoint.x, waypoint.y);
        }

        println!("\nControls: [Space] Play/Pause | [/] Speed | [R] Reset | [Q] Quit");
        println!("===============================");
    }

    #[allow(dead_code)]
    fn handle_input(&mut self) -> bool {
        // Non-blocking input check
        let mut buffer = [0u8; 1];
        if let Ok(n) = io::stdin().read(&mut buffer) {
            if n > 0 {
                match buffer[0] {
                    b' ' => self.paused = !self.paused,
                    b'[' => self.time_scale = (self.time_scale * 0.5).max(0.1),
                    b']' => self.time_scale = (self.time_scale * 2.0).min(10.0),
                    b'r' | b'R' => self.reset(),
                    b'q' | b'Q' => return false,
                    _ => {}
                }
            }
        }
        true
    }
}

fn main() -> anyhow::Result<()> {
    println!("BT Patrol Demo - Deterministic Behavior Tree AI");
    println!("Seed: 42 (deterministic)");
    println!("\nInitializing...\n");

    let mut demo = DemoState::new(42);

    // Simple main loop (tick every 0.1s for 100 ticks or until quit)
    for _ in 0..100 {
        demo.render_hud();
        demo.update(0.1);

        // Check for target defeated
        let target_hp = demo
            .world
            .health(demo.target_id)
            .expect("Target entity should have Health component")
            .hp;
        if target_hp <= 0 {
            println!("\n🎯 TARGET DEFEATED! Demo complete.");
            break;
        }

        // Simple delay (not production-ready, but good for demo)
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n✅ Demo finished. Final tick: {}", demo.tick_count);
    Ok(())
}
