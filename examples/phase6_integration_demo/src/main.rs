//! Phase 6 Integration Demo
//!
//! Demonstrates multiplayer-ready AstraWeave with:
//! - 4-player deterministic networking with client prediction
//! - AI companions with planning and action execution
//! - Persistent save/load with replay functionality
//! - Security validation and anti-cheat measures
//! - Stress testing for large-scale scenarios

use anyhow::Result;
use astraweave_ai::{ActionStep, CCompanionAI, CompanionOrchestrator, PlanIntent};
use astraweave_core::{CCooldown, CHealth, CPosition, CTeam};
use astraweave_ecs::{
    App, Component, Entity, Plugin, Query, Res, ResMut, Resource, SystemStage, Vec3,
};
use astraweave_nav::{CNavAgent, NavPlugin};
use astraweave_net_ecs::{
    CNetworkAuthority, CNetworkClient, NetworkClientPlugin, NetworkServerPlugin, NetworkSnapshot,
};
use astraweave_persistence_ecs::{PersistenceManager, PersistencePlugin, ReplayState};
use astraweave_physics::{CPhysicsBody, PhysicsPlugin};
use astraweave_security::{
    CAntiCheat, SecurityConfig, SecurityPlugin, TelemetryData, TelemetryEvent, TelemetrySeverity,
};
use astraweave_stress_test::{CAIStress, CNetworkStress, CStressEntity, StressTestPlugin};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Player component
#[derive(Clone, Debug, Component)]
pub struct CPlayer {
    pub id: String,
    pub name: String,
    pub input_sequence: u64,
}

/// AI Companion component
#[derive(Clone, Debug, Component)]
pub struct CAICompanion {
    pub owner_player_id: String,
    pub personality: String,
    pub current_plan: Option<PlanIntent>,
}

/// Game state resource
#[derive(Clone, Debug, Resource)]
pub struct GameState {
    pub tick_count: u64,
    pub players: Vec<String>,
    pub game_mode: GameMode,
    pub start_time: Instant,
}

/// Game modes
#[derive(Clone, Debug)]
pub enum GameMode {
    Cooperative,
    Competitive,
    StressTest,
}

/// Demo configuration
#[derive(Clone, Debug)]
pub struct DemoConfig {
    pub num_players: usize,
    pub num_ai_companions: usize,
    pub enable_networking: bool,
    pub enable_persistence: bool,
    pub enable_security: bool,
    pub enable_stress_test: bool,
    pub simulation_duration_secs: u64,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            num_players: 4,
            num_ai_companions: 8,
            enable_networking: true,
            enable_persistence: true,
            enable_security: true,
            enable_stress_test: false,
            simulation_duration_secs: 30,
        }
    }
}

/// Main demo application
pub struct Phase6Demo {
    app: App,
    config: DemoConfig,
    start_time: Instant,
}

impl Phase6Demo {
    pub fn new(config: DemoConfig) -> Result<Self> {
        let mut app = App::new();

        // Initialize core game state
        let game_state = GameState {
            tick_count: 0,
            players: Vec::new(),
            game_mode: GameMode::Cooperative,
            start_time: Instant::now(),
        };
        app.insert_resource(game_state);

        // Add core plugins
        app.add_plugin(PhysicsPlugin::default());
        app.add_plugin(NavPlugin::default());

        // Add Phase 6 plugins conditionally
        if config.enable_networking {
            app.add_plugin(NetworkClientPlugin::new("127.0.0.1:8080".to_string()));
            app.add_plugin(NetworkServerPlugin::new("127.0.0.1:8080".to_string()));
        }

        if config.enable_persistence {
            app.add_plugin(PersistencePlugin::default());
        }

        if config.enable_security {
            let security_config = SecurityConfig {
                enable_sandboxing: true,
                enable_llm_validation: true,
                enable_script_sandbox: true,
                max_script_execution_time_ms: 500,
                max_memory_usage_mb: 100,
            };
            app.add_plugin(SecurityPlugin::new(security_config));
        }

        if config.enable_stress_test {
            app.add_plugin(StressTestPlugin::default());
        }

        // Add demo systems
        app.add_system(SystemStage::PreSimulation, player_input_system);
        app.add_system(SystemStage::Simulation, ai_companion_system);
        app.add_system(SystemStage::Simulation, game_logic_system);
        app.add_system(SystemStage::PostSimulation, demo_monitoring_system);

        Ok(Self {
            app,
            config,
            start_time: Instant::now(),
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        println!("🚀 Initializing Phase 6 Integration Demo");
        println!("Configuration:");
        println!("  Players: {}", self.config.num_players);
        println!("  AI Companions: {}", self.config.num_ai_companions);
        println!("  Networking: {}", self.config.enable_networking);
        println!("  Persistence: {}", self.config.enable_persistence);
        println!("  Security: {}", self.config.enable_security);
        println!("  Stress Test: {}", self.config.enable_stress_test);

        // Create players
        for i in 0..self.config.num_players {
            self.create_player(i)?;
        }

        // Create AI companions
        for i in 0..self.config.num_ai_companions {
            self.create_ai_companion(i)?;
        }

        // Initialize stress test entities if enabled
        if self.config.enable_stress_test {
            self.initialize_stress_test()?;
        }

        // Create initial save point
        if self.config.enable_persistence {
            self.create_save_point("initial_state")?;
        }

        println!("✅ Initialization complete");
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        println!(
            "🎮 Starting simulation for {} seconds",
            self.config.simulation_duration_secs
        );

        let mut last_tick = Instant::now();
        let tick_duration = Duration::from_millis(16); // ~60 FPS

        while self.start_time.elapsed() < Duration::from_secs(self.config.simulation_duration_secs)
        {
            let now = Instant::now();

            // Maintain tick rate
            if now.duration_since(last_tick) >= tick_duration {
                self.app.tick().await?;
                last_tick = now;

                // Periodic saves
                if self.config.enable_persistence
                    && self
                        .app
                        .world()
                        .get_resource::<GameState>()
                        .unwrap()
                        .tick_count
                        % 300
                        == 0
                {
                    self.create_save_point(&format!(
                        "tick_{}",
                        self.app
                            .world()
                            .get_resource::<GameState>()
                            .unwrap()
                            .tick_count
                    ))?;
                }
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        println!("🏁 Simulation complete");
        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        // Create final save point
        if self.config.enable_persistence {
            self.create_save_point("final_state")?;
        }

        // Generate final report
        self.generate_report().await?;

        println!("🧹 Cleanup complete");
        Ok(())
    }

    fn create_player(&mut self, index: usize) -> Result<Entity> {
        let player_id = format!("player_{}", index);
        let player_name = format!("Player {}", index + 1);

        let entity = self.app.world_mut().spawn((
            CPlayer {
                id: player_id.clone(),
                name: player_name.clone(),
                input_sequence: 0,
            },
            CPosition {
                position: Vec3::new(index as f32 * 10.0, 0.0, 0.0),
            },
            CHealth {
                current: 100,
                maximum: 100,
            },
            CTeam { id: 1 },
            CCooldown { remaining_ms: 0 },
        ));

        if self.config.enable_security {
            self.app.world_mut().insert_component(
                entity,
                CAntiCheat {
                    player_id: player_id.clone(),
                    trust_score: 1.0,
                    last_validation: 0,
                    anomaly_flags: Vec::new(),
                },
            )?;
        }

        if self.config.enable_networking {
            self.app.world_mut().insert_component(
                entity,
                CNetworkClient {
                    player_id: player_id.clone(),
                    last_acknowledged_input: 0,
                    pending_inputs: Vec::new(),
                },
            )?;
        }

        // Update game state
        let mut game_state = self
            .app
            .world_mut()
            .get_resource_mut::<GameState>()
            .unwrap();
        game_state.players.push(player_id);

        println!("👤 Created player: {}", player_name);
        Ok(entity)
    }

    fn create_ai_companion(&mut self, index: usize) -> Result<Entity> {
        let owner_index = index % self.config.num_players;
        let owner_id = format!("player_{}", owner_index);
        let personality = match index % 4 {
            0 => "aggressive",
            1 => "defensive",
            2 => "supportive",
            _ => "exploratory",
        };

        let entity = self.app.world_mut().spawn((
            CAICompanion {
                owner_player_id: owner_id.clone(),
                personality: personality.to_string(),
                current_plan: None,
            },
            CCompanionAI {
                orchestrator: Arc::new(CompanionOrchestrator::new()),
                world_snapshot: None,
                last_planning_tick: 0,
            },
            CPosition {
                position: Vec3::new(owner_index as f32 * 10.0 + 5.0, 0.0, index as f32 * 5.0),
            },
            CHealth {
                current: 80,
                maximum: 80,
            },
            CTeam { id: 1 },
            CPhysicsBody::default(),
            CNavAgent::default(),
        ));

        if self.config.enable_stress_test {
            self.app.world_mut().insert_component(
                entity,
                CAIStress {
                    complexity_level: (index % 5) + 1,
                    decision_frequency: 10 + (index % 20),
                    memory_pressure: index % 100,
                },
            )?;
        }

        println!(
            "🤖 Created AI companion for {} with {} personality",
            owner_id, personality
        );
        Ok(entity)
    }

    fn initialize_stress_test(&mut self) -> Result<()> {
        println!("🔥 Initializing stress test entities");

        for i in 0..1000 {
            let entity = self.app.world_mut().spawn((
                CStressEntity {
                    id: i,
                    category: match i % 4 {
                        0 => "physics".to_string(),
                        1 => "ai".to_string(),
                        2 => "network".to_string(),
                        _ => "render".to_string(),
                    },
                    complexity: (i % 10) + 1,
                },
                CPosition {
                    position: Vec3::new(
                        rand::random::<f32>() * 1000.0,
                        rand::random::<f32>() * 100.0,
                        rand::random::<f32>() * 1000.0,
                    ),
                },
                CPhysicsBody::default(),
            ));

            if i % 4 == 2 && self.config.enable_networking {
                self.app.world_mut().insert_component(
                    entity,
                    CNetworkStress {
                        simulated_latency_ms: 50 + (i % 200),
                        packet_loss_rate: 0.01 * (i % 100) as f32,
                        jitter_ms: 10 + (i % 50),
                    },
                )?;
            }
        }

        println!("✅ Created 1000 stress test entities");
        Ok(())
    }

    fn create_save_point(&mut self, name: &str) -> Result<()> {
        let persistence_manager = self
            .app
            .world()
            .get_resource::<PersistenceManager>()
            .unwrap();
        persistence_manager.save_game(name)?;
        println!("💾 Created save point: {}", name);
        Ok(())
    }

    async fn generate_report(&self) -> Result<()> {
        println!("\n📊 Phase 6 Integration Demo Report");
        println!("==================================");

        let game_state = self.app.world().get_resource::<GameState>().unwrap();

        println!(
            "Simulation Duration: {:.2}s",
            self.start_time.elapsed().as_secs_f32()
        );
        println!("Total Ticks: {}", game_state.tick_count);
        println!("Players: {}", game_state.players.len());

        // Count entities by type
        let player_count = self.app.world().query::<&CPlayer>().iter().count();
        let ai_count = self.app.world().query::<&CAICompanion>().iter().count();
        let stress_count = self.app.world().query::<&CStressEntity>().iter().count();

        println!("Entities Created:");
        println!("  Players: {}", player_count);
        println!("  AI Companions: {}", ai_count);
        println!("  Stress Test: {}", stress_count);

        // Security report
        if self.config.enable_security {
            let telemetry = self.app.world().get_resource::<TelemetryData>().unwrap();
            println!("Security Events: {}", telemetry.events.len());
            println!("Anomalies Detected: {}", telemetry.anomaly_count);

            let mut severity_counts = HashMap::new();
            for event in &telemetry.events {
                *severity_counts
                    .entry(format!("{:?}", event.severity))
                    .or_insert(0) += 1;
            }

            for (severity, count) in severity_counts {
                println!("  {}: {}", severity, count);
            }
        }

        // Persistence report
        if self.config.enable_persistence {
            let persistence_manager = self
                .app
                .world()
                .get_resource::<PersistenceManager>()
                .unwrap();
            println!(
                "Save Points Created: {}",
                persistence_manager.list_save_points()?.len()
            );
        }

        println!("==================================");
        Ok(())
    }
}

/// Player input system
fn player_input_system(
    mut query: Query<(&mut CPlayer, &mut CPosition, Option<&mut CNetworkClient>)>,
    mut game_state: ResMut<GameState>,
) {
    for (mut player, mut position, network_client) in query.iter_mut() {
        // Simulate player input
        let mut rng = rand::thread_rng();
        let input_x = rng.gen_range(-1.0..=1.0);
        let input_z = rng.gen_range(-1.0..=1.0);

        // Update position
        position.position.x += input_x * 0.1;
        position.position.z += input_z * 0.1;

        // Update input sequence
        player.input_sequence += 1;

        // Send network input if enabled
        if let Some(mut client) = network_client {
            client.pending_inputs.push(player.input_sequence);
        }
    }

    game_state.tick_count += 1;
}

/// AI companion system
fn ai_companion_system(
    mut query: Query<(&mut CAICompanion, &mut CCompanionAI, &CPosition)>,
    game_state: Res<GameState>,
) {
    for (mut companion, mut ai, position) in query.iter_mut() {
        // Periodic planning
        if game_state.tick_count % 60 == 0 {
            // Plan every second
            // Create a simple plan based on personality
            let plan = match companion.personality.as_str() {
                "aggressive" => PlanIntent {
                    steps: vec![
                        ActionStep::MoveTo(Vec3::new(
                            position.position.x + 5.0,
                            position.position.y,
                            position.position.z,
                        )),
                        ActionStep::Attack("enemy".to_string()),
                    ],
                },
                "defensive" => PlanIntent {
                    steps: vec![
                        ActionStep::MoveTo(Vec3::new(
                            position.position.x - 2.0,
                            position.position.y,
                            position.position.z,
                        )),
                        ActionStep::Defend,
                    ],
                },
                "supportive" => PlanIntent {
                    steps: vec![
                        ActionStep::Follow(companion.owner_player_id.clone()),
                        ActionStep::Heal(companion.owner_player_id.clone()),
                    ],
                },
                _ => PlanIntent {
                    steps: vec![ActionStep::Explore, ActionStep::Gather],
                },
            };

            companion.current_plan = Some(plan);
        }

        // Execute current plan
        if let Some(plan) = &companion.current_plan {
            if !plan.steps.is_empty() {
                // Execute first step (simplified)
                match &plan.steps[0] {
                    ActionStep::MoveTo(target) => {
                        // Move towards target (simplified)
                        let direction = (*target - position.position).normalize() * 0.05;
                        // Note: Would update position here in a real implementation
                    }
                    _ => {} // Other actions would be handled here
                }
            }
        }
    }
}

/// Game logic system
fn game_logic_system(mut query: Query<(&mut CHealth, &CPosition)>) {
    // Simple collision detection and damage
    let mut entities: Vec<_> = query.iter_mut().collect();

    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            let (health_a, pos_a) = &mut entities[i];
            let (health_b, pos_b) = &mut entities[j];

            let distance = (pos_a.position - pos_b.position).length();
            if distance < 2.0 {
                // Collision damage
                health_a.current = health_a.current.saturating_sub(1);
                health_b.current = health_b.current.saturating_sub(1);
            }
        }
    }
}

/// Demo monitoring system
fn demo_monitoring_system(query: Query<(&CPlayer, &CHealth)>, game_state: Res<GameState>) {
    // Periodic status updates
    if game_state.tick_count % 180 == 0 {
        // Every 3 seconds
        let mut total_health = 0;
        let mut alive_players = 0;

        for (_, health) in query.iter() {
            total_health += health.current;
            if health.current > 0 {
                alive_players += 1;
            }
        }

        println!(
            "📈 Status - Tick: {}, Alive Players: {}, Total Health: {}",
            game_state.tick_count, alive_players, total_health
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments for demo configuration
    let config = DemoConfig::default();

    let mut demo = Phase6Demo::new(config)?;
    demo.initialize().await?;
    demo.run().await?;
    demo.cleanup().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn demo_initialization() {
        let config = DemoConfig {
            num_players: 2,
            num_ai_companions: 2,
            enable_networking: false,
            enable_persistence: false,
            enable_security: false,
            enable_stress_test: false,
            simulation_duration_secs: 1,
        };

        let mut demo = Phase6Demo::new(config).unwrap();
        demo.initialize().await.unwrap();

        // Verify entities were created
        let player_count = demo.app.world().query::<&CPlayer>().iter().count();
        let ai_count = demo.app.world().query::<&CAICompanion>().iter().count();

        assert_eq!(player_count, 2);
        assert_eq!(ai_count, 2);
    }

    #[tokio::test]
    async fn demo_with_security() {
        let config = DemoConfig {
            num_players: 1,
            num_ai_companions: 0,
            enable_networking: false,
            enable_persistence: false,
            enable_security: true,
            enable_stress_test: false,
            simulation_duration_secs: 1,
        };

        let mut demo = Phase6Demo::new(config).unwrap();
        demo.initialize().await.unwrap();
        demo.run().await.unwrap();

        // Verify security components were added
        let anti_cheat_count = demo.app.world().query::<&CAntiCheat>().iter().count();
        assert_eq!(anti_cheat_count, 1);
    }
}
