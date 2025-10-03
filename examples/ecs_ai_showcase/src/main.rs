//! ECS AI Showcase - Demonstrates the AI-native ECS capabilities
//!
//! This example shows:
//! - AI Perception → Reasoning → Planning → Action loop
//! - Event-driven AI behaviors
//! - System stage ordering
//! - Query ergonomics with QueryMut
//! - Resource management

use anyhow::Result;
use astraweave_ecs::*;
use glam::Vec3;
use std::collections::HashMap;

// ============================================================================
// Components
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct Position {
    pos: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct Velocity {
    vel: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Team {
    Player,
    Enemy,
    Neutral,
}

#[derive(Clone, Debug)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Clone, Debug)]
struct AIAgent {
    perception_radius: f32,
    target: Option<Entity>,
    state: AIState,
}

#[derive(Clone, Debug, PartialEq)]
enum AIState {
    Idle,
    Patrolling,
    Chasing,
    Attacking,
    Fleeing,
}

#[derive(Clone, Debug)]
struct Player {
    name: String,
}

// ============================================================================
// Resources
// ============================================================================

#[derive(Clone, Debug)]
struct GameTime {
    tick: u64,
    delta_time: f32,
}

#[derive(Clone, Debug, Default)]
struct GameStats {
    enemies_defeated: u32,
    player_deaths: u32,
    total_damage_dealt: i32,
}

// ============================================================================
// Events
// ============================================================================

#[derive(Clone, Debug)]
struct DamageEvent {
    attacker: Entity,
    target: Entity,
    damage: i32,
}
impl Event for DamageEvent {}

#[derive(Clone, Debug)]
struct AIStateChangedEvent {
    entity: Entity,
    old_state: AIState,
    new_state: AIState,
}
impl Event for AIStateChangedEvent {}

// ============================================================================
// Systems - Perception Stage
// ============================================================================

/// AI Perception: Update AI agents' perception of the world
fn ai_perception_system(world: &mut World) {
    // Get all AI agents
    let ai_entities: Vec<Entity> = world.entities_with::<AIAgent>();
    
    // For each AI agent, find nearby enemies
    for agent_entity in ai_entities {
        let agent_pos = if let Some(pos) = world.get::<Position>(agent_entity) {
            pos.pos
        } else {
            continue;
        };

        let perception_radius = if let Some(ai) = world.get::<AIAgent>(agent_entity) {
            ai.perception_radius
        } else {
            continue;
        };

        // Find closest enemy
        let mut closest_enemy: Option<(Entity, f32)> = None;
        
        let all_entities: Vec<Entity> = world.entities_with::<Position>();
        for other_entity in all_entities {
            if other_entity == agent_entity {
                continue;
            }

            // Check if enemy team
            let is_enemy = match (world.get::<Team>(agent_entity), world.get::<Team>(other_entity)) {
                (Some(Team::Enemy), Some(Team::Player)) => true,
                (Some(Team::Player), Some(Team::Enemy)) => true,
                _ => false,
            };

            if !is_enemy {
                continue;
            }

            if let Some(other_pos) = world.get::<Position>(other_entity) {
                let distance = (other_pos.pos - agent_pos).length();
                
                if distance <= perception_radius {
                    if let Some((_, closest_dist)) = closest_enemy {
                        if distance < closest_dist {
                            closest_enemy = Some((other_entity, distance));
                        }
                    } else {
                        closest_enemy = Some((other_entity, distance));
                    }
                }
            }
        }

        // Update AI target
        if let Some(ai) = world.get_mut::<AIAgent>(agent_entity) {
            ai.target = closest_enemy.map(|(e, _)| e);
        }
    }
}

// ============================================================================
// Systems - AI Planning Stage
// ============================================================================

/// AI Planning: Decide actions based on perception
fn ai_planning_system(world: &mut World) {
    let ai_entities: Vec<Entity> = world.entities_with::<AIAgent>();

    // Get events resource
    let events_ptr = world.get_resource_mut::<Events>() as *mut Events;
    
    for entity in ai_entities {
        let (current_state, target, health) = {
            let ai = if let Some(ai) = world.get::<AIAgent>(entity) {
                ai
            } else {
                continue;
            };
            
            let health = world.get::<Health>(entity).map(|h| h.current).unwrap_or(100);
            (ai.state.clone(), ai.target, health)
        };

        // State machine for AI decision making
        let new_state = match current_state {
            AIState::Idle => {
                if target.is_some() {
                    AIState::Chasing
                } else {
                    AIState::Patrolling
                }
            }
            AIState::Patrolling => {
                if target.is_some() {
                    AIState::Chasing
                } else {
                    AIState::Patrolling
                }
            }
            AIState::Chasing => {
                if target.is_none() {
                    AIState::Idle
                } else if health < 30 {
                    AIState::Fleeing
                } else {
                    // Check distance to target
                    if let Some(target_entity) = target {
                        if let (Some(my_pos), Some(target_pos)) = (
                            world.get::<Position>(entity),
                            world.get::<Position>(target_entity),
                        ) {
                            let distance = (target_pos.pos - my_pos.pos).length();
                            if distance < 2.0 {
                                AIState::Attacking
                            } else {
                                AIState::Chasing
                            }
                        } else {
                            AIState::Chasing
                        }
                    } else {
                        AIState::Idle
                    }
                }
            }
            AIState::Attacking => {
                if target.is_none() || health < 30 {
                    AIState::Fleeing
                } else {
                    AIState::Attacking
                }
            }
            AIState::Fleeing => {
                if health > 50 {
                    AIState::Idle
                } else {
                    AIState::Fleeing
                }
            }
        };

        // Update state and emit event if changed
        if new_state != current_state {
            if let Some(ai) = world.get_mut::<AIAgent>(entity) {
                ai.state = new_state.clone();
            }

            // Emit state changed event
            if let Some(events) = unsafe { events_ptr.as_mut() } {
                events.send(AIStateChangedEvent {
                    entity,
                    old_state: current_state,
                    new_state,
                });
            }
        }
    }
}

// ============================================================================
// Systems - Simulation Stage
// ============================================================================

/// Movement system: Apply velocity to position
fn movement_system(world: &mut World) {
    let delta_time = world
        .get_resource::<GameTime>()
        .map(|t| t.delta_time)
        .unwrap_or(1.0 / 60.0);

    let entities: Vec<Entity> = world.entities_with::<Position>();
    
    for entity in entities {
        if let (Some(pos), Some(vel)) = (
            world.get_mut::<Position>(entity),
            world.get::<Velocity>(entity),
        ) {
            pos.pos += vel.vel * delta_time;
        }
    }
}

/// AI behavior execution based on state
fn ai_behavior_system(world: &mut World) {
    let ai_entities: Vec<Entity> = world.entities_with::<AIAgent>();

    for entity in ai_entities {
        let state = if let Some(ai) = world.get::<AIAgent>(entity) {
            ai.state.clone()
        } else {
            continue;
        };

        match state {
            AIState::Chasing => {
                // Move towards target
                if let Some(ai) = world.get::<AIAgent>(entity) {
                    if let Some(target) = ai.target {
                        if let (Some(my_pos), Some(target_pos)) = (
                            world.get::<Position>(entity),
                            world.get::<Position>(target),
                        ) {
                            let direction = (target_pos.pos - my_pos.pos).normalize_or_zero();
                            let speed = 5.0;
                            
                            if let Some(vel) = world.get_mut::<Velocity>(entity) {
                                vel.vel = direction * speed;
                            }
                        }
                    }
                }
            }
            AIState::Fleeing => {
                // Move away from target
                if let Some(ai) = world.get::<AIAgent>(entity) {
                    if let Some(target) = ai.target {
                        if let (Some(my_pos), Some(target_pos)) = (
                            world.get::<Position>(entity),
                            world.get::<Position>(target),
                        ) {
                            let direction = (my_pos.pos - target_pos.pos).normalize_or_zero();
                            let speed = 7.0; // Flee faster
                            
                            if let Some(vel) = world.get_mut::<Velocity>(entity) {
                                vel.vel = direction * speed;
                            }
                        }
                    }
                }
            }
            AIState::Attacking => {
                // Stop moving and attack
                if let Some(vel) = world.get_mut::<Velocity>(entity) {
                    vel.vel = Vec3::ZERO;
                }
                
                // Emit damage event
                if let Some(ai) = world.get::<AIAgent>(entity) {
                    if let Some(target) = ai.target {
                        if let Some(events) = world.get_resource_mut::<Events>() {
                            events.send(DamageEvent {
                                attacker: entity,
                                target,
                                damage: 10,
                            });
                        }
                    }
                }
            }
            AIState::Patrolling => {
                // Simple patrol: random movement
                if let Some(vel) = world.get_mut::<Velocity>(entity) {
                    vel.vel = Vec3::new(1.0, 0.0, 1.0).normalize() * 2.0;
                }
            }
            AIState::Idle => {
                if let Some(vel) = world.get_mut::<Velocity>(entity) {
                    vel.vel = Vec3::ZERO;
                }
            }
        }
    }
}

/// Combat system: Process damage events
fn combat_system(world: &mut World) {
    // Read damage events
    let damage_events: Vec<DamageEvent> = {
        if let Some(events) = world.get_resource::<Events>() {
            events.read::<DamageEvent>().cloned().collect()
        } else {
            Vec::new()
        }
    };

    let mut stats_update = (0, 0); // (damage_dealt, enemies_defeated)

    for event in damage_events {
        if let Some(health) = world.get_mut::<Health>(event.target) {
            health.current -= event.damage;
            stats_update.0 += event.damage;

            if health.current <= 0 {
                // Entity defeated
                stats_update.1 += 1;
                
                // Emit health changed event
                if let Some(events) = world.get_resource_mut::<Events>() {
                    events.send(HealthChangedEvent {
                        entity: event.target,
                        old_health: health.current + event.damage,
                        new_health: health.current,
                        source: Some(event.attacker),
                    });
                }
            }
        }
    }

    // Update game stats
    if let Some(stats) = world.get_resource_mut::<GameStats>() {
        stats.total_damage_dealt += stats_update.0;
        stats.enemies_defeated += stats_update.1;
    }
}

// ============================================================================
// Systems - Post-Simulation Stage
// ============================================================================

/// Stats display system
fn stats_display_system(world: &mut World) {
    let tick = world
        .get_resource::<GameTime>()
        .map(|t| t.tick)
        .unwrap_or(0);

    // Print stats every 60 ticks (1 second at 60 FPS)
    if tick % 60 == 0 {
        if let Some(stats) = world.get_resource::<GameStats>() {
            println!("\n=== Game Stats (Tick {}) ===", tick);
            println!("Enemies Defeated: {}", stats.enemies_defeated);
            println!("Total Damage: {}", stats.total_damage_dealt);
            println!("Player Deaths: {}", stats.player_deaths);
        }

        // Count AI states
        let ai_entities: Vec<Entity> = world.entities_with::<AIAgent>();
        let mut state_counts: HashMap<String, u32> = HashMap::new();
        
        for entity in ai_entities {
            if let Some(ai) = world.get::<AIAgent>(entity) {
                let state_name = format!("{:?}", ai.state);
                *state_counts.entry(state_name).or_insert(0) += 1;
            }
        }

        println!("\n=== AI States ===");
        for (state, count) in state_counts {
            println!("{}: {}", state, count);
        }
    }
}

// ============================================================================
// Setup and Main
// ============================================================================

fn setup_world(app: &mut App) {
    // Insert resources
    app.world.insert_resource(GameTime {
        tick: 0,
        delta_time: 1.0 / 60.0,
    });
    app.world.insert_resource(GameStats::default());
    app.world.insert_resource(Events::new());

    // Spawn player
    let player = app.world.spawn();
    app.world.insert(player, Player {
        name: "Hero".to_string(),
    });
    app.world.insert(player, Position {
        pos: Vec3::new(0.0, 0.0, 0.0),
    });
    app.world.insert(player, Velocity {
        vel: Vec3::ZERO,
    });
    app.world.insert(player, Health {
        current: 100,
        max: 100,
    });
    app.world.insert(player, Team::Player);

    // Spawn enemies
    for i in 0..5 {
        let enemy = app.world.spawn();
        app.world.insert(enemy, Position {
            pos: Vec3::new(10.0 + i as f32 * 3.0, 0.0, 10.0),
        });
        app.world.insert(enemy, Velocity {
            vel: Vec3::ZERO,
        });
        app.world.insert(enemy, Health {
            current: 50,
            max: 50,
        });
        app.world.insert(enemy, Team::Enemy);
        app.world.insert(enemy, AIAgent {
            perception_radius: 15.0,
            target: None,
            state: AIState::Idle,
        });
    }

    println!("🎮 ECS AI Showcase initialized!");
    println!("   Player: 1");
    println!("   Enemies: 5");
    println!("   Running AI-native game loop: Perception → Planning → Simulation\n");
}

fn main() -> Result<()> {
    let mut app = App::new();

    // Register systems in AI-native order:
    // Perception → AI Planning → Simulation → Post-Simulation
    
    app.add_system(SystemStage::PERCEPTION, ai_perception_system);
    app.add_system(SystemStage::AI_PLANNING, ai_planning_system);
    app.add_system(SystemStage::SIMULATION, ai_behavior_system);
    app.add_system(SystemStage::SIMULATION, movement_system);
    app.add_system(SystemStage::SIMULATION, combat_system);
    app.add_system(SystemStage::POST_SIMULATION, stats_display_system);

    setup_world(&mut app);

    // Run simulation
    println!("🚀 Starting simulation...\n");
    
    for _ in 0..300 {
        // Update game time
        if let Some(time) = app.world.get_resource_mut::<GameTime>() {
            time.tick += 1;
        }

        // Run all systems
        app.schedule.run(&mut app.world);

        // Update events
        if let Some(events) = app.world.get_resource_mut::<Events>() {
            events.update();
        }
    }

    println!("\n✅ Simulation complete!");

    Ok(())
}
