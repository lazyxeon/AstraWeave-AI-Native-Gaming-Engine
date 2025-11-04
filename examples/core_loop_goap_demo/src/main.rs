//! GOAP Craft Demo - Goal-Oriented Action Planning demonstration
//!
//! Demonstrates: Gather → Craft → Consume behavior with resource management
//!
//! Controls:
//! - Space: Play/Pause simulation
//! - [/]: Slow/Fast time scale
//! - R: Reset to initial seed
//! - G: Spawn additional resource
//! - Q: Quit
//!
//! Deterministic: Fixed seed (123) ensures reproducible behavior

use astraweave_core::{IVec2, Team, World};
use rand::{Rng, SeedableRng};

/// Resource types in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResourceType {
    Wood,
    Berries,
}

/// World resource node
#[derive(Debug, Clone)]
struct ResourceNode {
    pos: IVec2,
    resource_type: ResourceType,
    amount: i32,
}

/// Agent inventory
#[derive(Debug, Clone, Default)]
struct Inventory {
    wood: i32,
    berries: i32,
    cooked_food: i32,
}

/// GOAP action types
#[derive(Debug, Clone, PartialEq)]
enum GoapAction {
    GoToTree,
    ChopWood,
    GoToBerries,
    GatherBerries,
    GoToCampfire,
    CookFood,
    ConsumeFood,
    Idle,
}

/// GOAP goal state
#[derive(Debug, Clone, PartialEq)]
enum Goal {
    HasFood,
    Satisfied,
}

/// Demo state
struct DemoState {
    world: World,
    agent_id: u32,
    campfire_pos: IVec2,
    resources: Vec<ResourceNode>,
    inventory: Inventory,
    current_action: GoapAction,
    goal: Goal,
    plan: Vec<GoapAction>,
    tick_count: u64,
    paused: bool,
    time_scale: f32,
    #[allow(dead_code)]
    seed: u64,
    hunger: i32,
}

impl DemoState {
    fn new(seed: u64) -> Self {
        let mut world = World::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        // Spawn agent at center
        let agent_id = world.spawn("CraftAgent", IVec2 { x: 10, y: 10 }, Team { id: 0 }, 100, 0);

        // Create campfire
        let campfire_pos = IVec2 { x: 10, y: 10 };

        // Spawn resource nodes (deterministic)
        let resources = vec![
            ResourceNode {
                pos: IVec2 { x: 5, y: 5 },
                resource_type: ResourceType::Wood,
                amount: 10,
            },
            ResourceNode {
                pos: IVec2 { x: 15, y: 5 },
                resource_type: ResourceType::Berries,
                amount: 10,
            },
            ResourceNode {
                pos: IVec2 { x: 5, y: 15 },
                resource_type: ResourceType::Wood,
                amount: 10,
            },
            ResourceNode {
                pos: IVec2 { x: 15, y: 15 },
                resource_type: ResourceType::Berries,
                amount: 10,
            },
        ];

        // Add some obstacles
        for _ in 0..5 {
            let x = rng.random_range(2..18);
            let y = rng.random_range(2..18);
            if (x, y) != (campfire_pos.x, campfire_pos.y) {
                world.obstacles.insert((x, y));
            }
        }

        Self {
            world,
            agent_id,
            campfire_pos,
            resources,
            inventory: Inventory::default(),
            current_action: GoapAction::Idle,
            goal: Goal::HasFood,
            plan: Vec::new(),
            tick_count: 0,
            paused: false,
            time_scale: 1.0,
            seed,
            hunger: 100,
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
        self.hunger = (self.hunger - 1).max(0);

        // Re-plan if needed
        if self.plan.is_empty() && self.goal != Goal::Satisfied {
            self.plan = self.generate_plan();
        }

        // Execute current action
        if !self.plan.is_empty() {
            let action = self.plan[0].clone();
            if self.execute_action(&action) {
                self.plan.remove(0);
                self.current_action = if self.plan.is_empty() {
                    GoapAction::Idle
                } else {
                    self.plan[0].clone()
                };
            }
        }

        // Check goal satisfaction
        if self.inventory.cooked_food > 0 {
            self.goal = Goal::Satisfied;
        }
    }

    fn generate_plan(&self) -> Vec<GoapAction> {
        // Simple GOAP planner: gather resources → craft → consume
        match self.goal {
            Goal::HasFood => {
                if self.inventory.cooked_food > 0 {
                    vec![GoapAction::ConsumeFood]
                } else if self.inventory.wood >= 2 && self.inventory.berries >= 2 {
                    vec![GoapAction::GoToCampfire, GoapAction::CookFood]
                } else if self.inventory.wood < 2 {
                    vec![GoapAction::GoToTree, GoapAction::ChopWood]
                } else {
                    vec![GoapAction::GoToBerries, GoapAction::GatherBerries]
                }
            }
            Goal::Satisfied => Vec::new(),
        }
    }

    fn execute_action(&mut self, action: &GoapAction) -> bool {
        let agent_pos = self
            .world
            .pos_of(self.agent_id)
            .expect("Agent entity should have Position component");

        match action {
            GoapAction::GoToTree => {
                if let Some(node) = self
                    .resources
                    .iter()
                    .find(|n| n.resource_type == ResourceType::Wood && n.amount > 0)
                {
                    self.move_toward(node.pos)
                } else {
                    true // No wood available, skip
                }
            }
            GoapAction::ChopWood => {
                if let Some(node) = self
                    .resources
                    .iter_mut()
                    .find(|n| n.resource_type == ResourceType::Wood && n.pos == agent_pos)
                {
                    if node.amount > 0 {
                        node.amount -= 1;
                        self.inventory.wood += 1;
                    }
                    true
                } else {
                    true
                }
            }
            GoapAction::GoToBerries => {
                if let Some(node) = self
                    .resources
                    .iter()
                    .find(|n| n.resource_type == ResourceType::Berries && n.amount > 0)
                {
                    self.move_toward(node.pos)
                } else {
                    true
                }
            }
            GoapAction::GatherBerries => {
                if let Some(node) = self
                    .resources
                    .iter_mut()
                    .find(|n| n.resource_type == ResourceType::Berries && n.pos == agent_pos)
                {
                    if node.amount > 0 {
                        node.amount -= 1;
                        self.inventory.berries += 1;
                    }
                    true
                } else {
                    true
                }
            }
            GoapAction::GoToCampfire => self.move_toward(self.campfire_pos),
            GoapAction::CookFood => {
                if agent_pos == self.campfire_pos
                    && self.inventory.wood >= 2
                    && self.inventory.berries >= 2
                {
                    self.inventory.wood -= 2;
                    self.inventory.berries -= 2;
                    self.inventory.cooked_food += 1;
                }
                true
            }
            GoapAction::ConsumeFood => {
                if self.inventory.cooked_food > 0 {
                    self.inventory.cooked_food -= 1;
                    self.hunger = 100;
                }
                true
            }
            GoapAction::Idle => true,
        }
    }

    fn move_toward(&mut self, target: IVec2) -> bool {
        let agent_pos = self
            .world
            .pos_of(self.agent_id)
            .expect("Agent entity should have Position component");

        if agent_pos == target {
            return true;
        }

        let dx = (target.x - agent_pos.x).signum();
        let dy = (target.y - agent_pos.y).signum();

        let new_pos = IVec2 {
            x: agent_pos.x + dx,
            y: agent_pos.y + dy,
        };

        if !self.world.obstacle(new_pos) {
            if let Some(pose) = self.world.pose_mut(self.agent_id) {
                pose.pos = new_pos;
            }
        }

        false
    }

    fn render_hud(&self) {
        println!("\n=== GOAP CRAFT DEMO ===");
        println!("Mode: GOAP (Goal-Oriented Action Planning)");
        println!("Tick: {}", self.tick_count);
        println!(
            "Time: {:.2}s (scale: {:.1}x)",
            self.world.t, self.time_scale
        );
        println!("Status: {}", if self.paused { "PAUSED" } else { "RUNNING" });

        // Goal state
        println!("\nGoal: {:?}", self.goal);
        println!("Current Action: {:?}", self.current_action);
        println!("Plan Length: {} steps", self.plan.len());
        if !self.plan.is_empty() {
            println!("Next Actions: {:?}", &self.plan[..self.plan.len().min(3)]);
        }

        // Agent state
        let agent_pos = self
            .world
            .pos_of(self.agent_id)
            .expect("Agent entity should have Position component");
        println!("\nAgent: pos=({}, {})", agent_pos.x, agent_pos.y);
        println!("Hunger: {}/100", self.hunger);

        // Inventory
        println!("\nInventory:");
        println!("  Wood: {}", self.inventory.wood);
        println!("  Berries: {}", self.inventory.berries);
        println!("  Cooked Food: {}", self.inventory.cooked_food);

        // Resources
        println!("\nWorld Resources:");
        for (_i, node) in self.resources.iter().enumerate() {
            println!(
                "  {:?} at ({}, {}): {} remaining",
                node.resource_type, node.pos.x, node.pos.y, node.amount
            );
        }

        println!("\nControls: [Space] Pause | [/] Speed | [R] Reset | [G] Add Resource | [Q] Quit");
        println!("==========================");
    }

    #[allow(dead_code)]
    fn spawn_resource(&mut self) {
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed + self.tick_count);
        let x = rng.random_range(2..18);
        let y = rng.random_range(2..18);
        let resource_type = if rng.random_bool(0.5) {
            ResourceType::Wood
        } else {
            ResourceType::Berries
        };

        self.resources.push(ResourceNode {
            pos: IVec2 { x, y },
            resource_type,
            amount: 5,
        });

        println!("✨ Spawned {:?} at ({}, {})", resource_type, x, y);
    }
}

fn main() -> anyhow::Result<()> {
    println!("GOAP Craft Demo - Goal-Oriented Action Planning");
    println!("Seed: 123 (deterministic)");
    println!("\nGoal: Gather resources → Craft food → Consume");
    println!("\nInitializing...\n");

    let mut demo = DemoState::new(123);

    // Main loop (100 ticks or until goal satisfied)
    for _ in 0..100 {
        demo.render_hud();
        demo.update(0.1);

        // Check for goal satisfaction
        if demo.goal == Goal::Satisfied && demo.inventory.cooked_food == 0 {
            println!("\n🎯 GOAL SATISFIED! Food consumed successfully.");
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n✅ Demo finished. Final tick: {}", demo.tick_count);
    Ok(())
}
