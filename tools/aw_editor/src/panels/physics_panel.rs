//! Physics Debug Panel for the editor UI
//!
//! Provides comprehensive debugging and configuration for physics systems:
//! - Collider visualization settings
//! - Ragdoll configuration and testing
//! - Vehicle physics tuning
//! - Cloth simulation parameters
//! - Destruction system configuration
//! - Gravity zones and environment effects
//! - Physics statistics and profiling

use egui::{Color32, RichText, Ui, Vec2};
use std::collections::VecDeque;

use crate::panels::Panel;

/// Physics visualization modes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PhysicsVisualization {
    #[default]
    None,
    Colliders,
    ContactPoints,
    Joints,
    VelocityVectors,
    BroadPhase,
    Islands,
    All,
}

impl PhysicsVisualization {
    pub fn all() -> &'static [PhysicsVisualization] {
        &[
            PhysicsVisualization::None,
            PhysicsVisualization::Colliders,
            PhysicsVisualization::ContactPoints,
            PhysicsVisualization::Joints,
            PhysicsVisualization::VelocityVectors,
            PhysicsVisualization::BroadPhase,
            PhysicsVisualization::Islands,
            PhysicsVisualization::All,
        ]
    }

    pub fn description(&self) -> &'static str {
        match self {
            PhysicsVisualization::None => "No debug visualization",
            PhysicsVisualization::Colliders => "Show collider wireframes",
            PhysicsVisualization::ContactPoints => "Show contact points and normals",
            PhysicsVisualization::Joints => "Show joint connections and limits",
            PhysicsVisualization::VelocityVectors => "Show linear/angular velocity",
            PhysicsVisualization::BroadPhase => "Show spatial partitioning grid",
            PhysicsVisualization::Islands => "Color bodies by simulation island",
            PhysicsVisualization::All => "Show all debug visualizations",
        }
    }
}

/// Collision layer definition
#[derive(Debug, Clone)]
pub struct CollisionLayer {
    pub id: u32,
    pub name: String,
    pub color: Color32,
    pub enabled: bool,
}

/// Physics simulation mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SimulationMode {
    #[default]
    Running,
    Paused,
    StepOnce,
    SlowMotion,
}

/// Ragdoll preset configuration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RagdollPreset {
    #[default]
    Humanoid,
    Quadruped,
    Bird,
    Insect,
    Custom,
}

impl RagdollPreset {
    pub fn all() -> &'static [RagdollPreset] {
        &[
            RagdollPreset::Humanoid,
            RagdollPreset::Quadruped,
            RagdollPreset::Bird,
            RagdollPreset::Insect,
            RagdollPreset::Custom,
        ]
    }

    pub fn bone_count(&self) -> usize {
        match self {
            RagdollPreset::Humanoid => 15,
            RagdollPreset::Quadruped => 18,
            RagdollPreset::Bird => 12,
            RagdollPreset::Insect => 8,
            RagdollPreset::Custom => 0,
        }
    }
}

/// Vehicle type presets
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VehicleType {
    #[default]
    Sedan,
    SportsCar,
    Truck,
    Motorcycle,
    Tank,
    Custom,
}

impl VehicleType {
    pub fn all() -> &'static [VehicleType] {
        &[
            VehicleType::Sedan,
            VehicleType::SportsCar,
            VehicleType::Truck,
            VehicleType::Motorcycle,
            VehicleType::Tank,
            VehicleType::Custom,
        ]
    }
}

/// Cloth simulation quality
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClothQuality {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

impl ClothQuality {
    pub fn all() -> &'static [ClothQuality] {
        &[
            ClothQuality::Low,
            ClothQuality::Medium,
            ClothQuality::High,
            ClothQuality::Ultra,
        ]
    }

    pub fn iterations(&self) -> u32 {
        match self {
            ClothQuality::Low => 4,
            ClothQuality::Medium => 8,
            ClothQuality::High => 16,
            ClothQuality::Ultra => 32,
        }
    }
}

/// Destruction pattern type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DestructionPattern {
    #[default]
    Voronoi,
    Radial,
    Slice,
    Shatter,
    Crumble,
}

impl DestructionPattern {
    pub fn all() -> &'static [DestructionPattern] {
        &[
            DestructionPattern::Voronoi,
            DestructionPattern::Radial,
            DestructionPattern::Slice,
            DestructionPattern::Shatter,
            DestructionPattern::Crumble,
        ]
    }
}

/// Gravity zone shape
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GravityZoneShape {
    #[default]
    Box,
    Sphere,
    Cylinder,
    Global,
}

impl GravityZoneShape {
    pub fn all() -> &'static [GravityZoneShape] {
        &[
            GravityZoneShape::Box,
            GravityZoneShape::Sphere,
            GravityZoneShape::Cylinder,
            GravityZoneShape::Global,
        ]
    }
}

/// Physics statistics
#[derive(Debug, Clone, Default)]
pub struct PhysicsStats {
    pub rigid_body_count: usize,
    pub collider_count: usize,
    pub joint_count: usize,
    pub contact_pair_count: usize,
    pub active_body_count: usize,
    pub island_count: usize,
    pub step_time_ms: f32,
    pub broad_phase_time_ms: f32,
    pub narrow_phase_time_ms: f32,
    pub solver_time_ms: f32,
    pub ccd_time_ms: f32,
    pub ragdoll_count: usize,
    pub vehicle_count: usize,
    pub cloth_count: usize,
    pub destructible_count: usize,
}

/// Panel tabs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PhysicsTab {
    #[default]
    Overview,
    Visualization,
    Ragdoll,
    Vehicle,
    Cloth,
    Destruction,
    Environment,
}

/// Main Physics Debug Panel
pub struct PhysicsPanel {
    // Tab state
    active_tab: PhysicsTab,

    // Simulation settings
    simulation_mode: SimulationMode,
    time_scale: f32,
    gravity: [f32; 3],
    substeps: u32,
    solver_iterations: u32,
    ccd_enabled: bool,

    // Visualization
    visualization_mode: PhysicsVisualization,
    collider_alpha: f32,
    show_sleeping_bodies: bool,
    show_aabbs: bool,
    show_mass_centers: bool,
    collision_layers: Vec<CollisionLayer>,

    // Ragdoll settings
    ragdoll_preset: RagdollPreset,
    ragdoll_joint_stiffness: f32,
    ragdoll_joint_damping: f32,
    ragdoll_impulse_multiplier: f32,
    ragdoll_blend_time: f32,

    // Vehicle settings
    vehicle_type: VehicleType,
    engine_torque: f32,
    suspension_stiffness: f32,
    suspension_damping: f32,
    wheel_friction: f32,
    downforce_coefficient: f32,

    // Cloth settings
    cloth_quality: ClothQuality,
    cloth_gravity_scale: f32,
    cloth_damping: f32,
    cloth_stiffness: f32,
    cloth_wind_influence: f32,
    cloth_self_collision: bool,

    // Destruction settings
    destruction_pattern: DestructionPattern,
    debris_lifetime: f32,
    debris_count_max: u32,
    destruction_impulse_threshold: f32,
    destruction_chain_reaction: bool,

    // Environment settings
    wind_enabled: bool,
    wind_direction: [f32; 3],
    wind_strength: f32,
    wind_turbulence: f32,
    gravity_zones: Vec<GravityZoneConfig>,
    selected_gravity_zone: Option<usize>,

    // Statistics
    stats: PhysicsStats,
    step_time_history: VecDeque<f32>,

    // Test tools
    spawn_test_objects: bool,
    test_impulse_strength: f32,
}

/// Gravity zone configuration
#[derive(Debug, Clone)]
pub struct GravityZoneConfig {
    pub name: String,
    pub shape: GravityZoneShape,
    pub strength: f32,
    pub direction: [f32; 3],
    pub falloff: bool,
    pub enabled: bool,
}

impl Default for GravityZoneConfig {
    fn default() -> Self {
        Self {
            name: "New Zone".to_string(),
            shape: GravityZoneShape::Sphere,
            strength: 9.81,
            direction: [0.0, -1.0, 0.0],
            falloff: true,
            enabled: true,
        }
    }
}

impl Default for PhysicsPanel {
    fn default() -> Self {
        let default_layers = vec![
            CollisionLayer {
                id: 1,
                name: "Default".to_string(),
                color: Color32::WHITE,
                enabled: true,
            },
            CollisionLayer {
                id: 2,
                name: "Characters".to_string(),
                color: Color32::GREEN,
                enabled: true,
            },
            CollisionLayer {
                id: 4,
                name: "Projectiles".to_string(),
                color: Color32::RED,
                enabled: true,
            },
            CollisionLayer {
                id: 8,
                name: "Triggers".to_string(),
                color: Color32::YELLOW,
                enabled: true,
            },
            CollisionLayer {
                id: 16,
                name: "Debris".to_string(),
                color: Color32::GRAY,
                enabled: true,
            },
        ];

        Self {
            active_tab: PhysicsTab::Overview,

            simulation_mode: SimulationMode::Running,
            time_scale: 1.0,
            gravity: [0.0, -9.81, 0.0],
            substeps: 1,
            solver_iterations: 4,
            ccd_enabled: true,

            visualization_mode: PhysicsVisualization::None,
            collider_alpha: 0.5,
            show_sleeping_bodies: false,
            show_aabbs: false,
            show_mass_centers: false,
            collision_layers: default_layers,

            ragdoll_preset: RagdollPreset::Humanoid,
            ragdoll_joint_stiffness: 100.0,
            ragdoll_joint_damping: 10.0,
            ragdoll_impulse_multiplier: 1.0,
            ragdoll_blend_time: 0.2,

            vehicle_type: VehicleType::Sedan,
            engine_torque: 500.0,
            suspension_stiffness: 50000.0,
            suspension_damping: 4500.0,
            wheel_friction: 1.0,
            downforce_coefficient: 0.3,

            cloth_quality: ClothQuality::Medium,
            cloth_gravity_scale: 1.0,
            cloth_damping: 0.01,
            cloth_stiffness: 1.0,
            cloth_wind_influence: 1.0,
            cloth_self_collision: false,

            destruction_pattern: DestructionPattern::Voronoi,
            debris_lifetime: 5.0,
            debris_count_max: 50,
            destruction_impulse_threshold: 100.0,
            destruction_chain_reaction: true,

            wind_enabled: false,
            wind_direction: [1.0, 0.0, 0.0],
            wind_strength: 10.0,
            wind_turbulence: 0.3,
            gravity_zones: Vec::new(),
            selected_gravity_zone: None,

            stats: PhysicsStats::default(),
            step_time_history: VecDeque::with_capacity(120),

            spawn_test_objects: false,
            test_impulse_strength: 500.0,
        }
    }
}

impl PhysicsPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (PhysicsTab::Overview, "📊 Overview"),
                (PhysicsTab::Visualization, "👁 Viz"),
                (PhysicsTab::Ragdoll, "🦴 Ragdoll"),
                (PhysicsTab::Vehicle, "🚗 Vehicle"),
                (PhysicsTab::Cloth, "🧵 Cloth"),
                (PhysicsTab::Destruction, "💥 Destruction"),
                (PhysicsTab::Environment, "🌍 Environment"),
            ];

            for (tab, label) in tabs {
                let is_selected = self.active_tab == tab;
                let button = egui::Button::new(label).fill(if is_selected {
                    Color32::from_rgb(60, 100, 160)
                } else {
                    Color32::from_rgb(50, 50, 55)
                });

                if ui.add(button).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        // Simulation controls
        ui.horizontal(|ui| {
            let mode_icon = match self.simulation_mode {
                SimulationMode::Running => "▶",
                SimulationMode::Paused => "⏸",
                SimulationMode::StepOnce => "⏭",
                SimulationMode::SlowMotion => "🐢",
            };
            ui.label(format!("{} {:?}", mode_icon, self.simulation_mode));
            ui.label(format!("| {} bodies", self.stats.rigid_body_count));
            ui.label(format!("| {:.2}ms", self.stats.step_time_ms));
        });

        ui.separator();
    }

    fn show_overview_tab(&mut self, ui: &mut Ui) {
        ui.heading("📊 Physics Overview");
        ui.add_space(10.0);

        // Simulation controls
        ui.group(|ui| {
            ui.label(RichText::new("Simulation Controls").strong());

            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.simulation_mode == SimulationMode::Running, "▶ Run")
                    .clicked()
                {
                    self.simulation_mode = SimulationMode::Running;
                }
                if ui
                    .selectable_label(self.simulation_mode == SimulationMode::Paused, "⏸ Pause")
                    .clicked()
                {
                    self.simulation_mode = SimulationMode::Paused;
                }
                if ui.button("⏭ Step").clicked() {
                    self.simulation_mode = SimulationMode::StepOnce;
                }
                if ui
                    .selectable_label(self.simulation_mode == SimulationMode::SlowMotion, "🐢 Slow")
                    .clicked()
                {
                    self.simulation_mode = SimulationMode::SlowMotion;
                }
            });

            ui.add(
                egui::Slider::new(&mut self.time_scale, 0.1..=2.0)
                    .text("Time Scale")
                    .logarithmic(true),
            );
        });

        ui.add_space(10.0);

        // World settings
        ui.group(|ui| {
            ui.label(RichText::new("World Settings").strong());

            egui::Grid::new("world_settings_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Gravity X:");
                    ui.add(egui::DragValue::new(&mut self.gravity[0]).speed(0.1));
                    ui.end_row();

                    ui.label("Gravity Y:");
                    ui.add(egui::DragValue::new(&mut self.gravity[1]).speed(0.1));
                    ui.end_row();

                    ui.label("Gravity Z:");
                    ui.add(egui::DragValue::new(&mut self.gravity[2]).speed(0.1));
                    ui.end_row();

                    ui.label("Substeps:");
                    ui.add(egui::DragValue::new(&mut self.substeps).range(1..=8));
                    ui.end_row();

                    ui.label("Solver Iterations:");
                    ui.add(egui::DragValue::new(&mut self.solver_iterations).range(1..=16));
                    ui.end_row();

                    ui.label("CCD Enabled:");
                    ui.checkbox(&mut self.ccd_enabled, "Continuous collision");
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // Statistics
        ui.group(|ui| {
            ui.label(RichText::new("Statistics").strong());

            egui::Grid::new("physics_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Rigid Bodies:");
                    ui.label(format!(
                        "{} ({} active)",
                        self.stats.rigid_body_count, self.stats.active_body_count
                    ));
                    ui.end_row();

                    ui.label("Colliders:");
                    ui.label(format!("{}", self.stats.collider_count));
                    ui.end_row();

                    ui.label("Joints:");
                    ui.label(format!("{}", self.stats.joint_count));
                    ui.end_row();

                    ui.label("Contact Pairs:");
                    ui.label(format!("{}", self.stats.contact_pair_count));
                    ui.end_row();

                    ui.label("Islands:");
                    ui.label(format!("{}", self.stats.island_count));
                    ui.end_row();
                });

            ui.add_space(5.0);

            // Timing breakdown
            ui.label("Step Time Breakdown:");
            self.draw_timing_bar(ui);
        });

        ui.add_space(10.0);

        // Subsystem counts
        ui.group(|ui| {
            ui.label(RichText::new("Active Subsystems").strong());

            ui.horizontal(|ui| {
                ui.label(format!("🦴 {} Ragdolls", self.stats.ragdoll_count));
                ui.separator();
                ui.label(format!("🚗 {} Vehicles", self.stats.vehicle_count));
                ui.separator();
                ui.label(format!("🧵 {} Cloth", self.stats.cloth_count));
                ui.separator();
                ui.label(format!("💥 {} Destructibles", self.stats.destructible_count));
            });
        });
    }

    fn show_visualization_tab(&mut self, ui: &mut Ui) {
        ui.heading("👁 Debug Visualization");
        ui.add_space(10.0);

        // Visualization mode
        ui.group(|ui| {
            ui.label(RichText::new("Visualization Mode").strong());

            for mode in PhysicsVisualization::all() {
                if ui
                    .selectable_label(self.visualization_mode == *mode, format!("{:?}", mode))
                    .clicked()
                {
                    self.visualization_mode = *mode;
                }
            }

            ui.label(self.visualization_mode.description());
        });

        ui.add_space(10.0);

        // Visualization options
        ui.group(|ui| {
            ui.label(RichText::new("Options").strong());

            ui.add(
                egui::Slider::new(&mut self.collider_alpha, 0.1..=1.0)
                    .text("Collider Alpha"),
            );
            ui.checkbox(&mut self.show_sleeping_bodies, "Show sleeping bodies (dimmed)");
            ui.checkbox(&mut self.show_aabbs, "Show AABBs");
            ui.checkbox(&mut self.show_mass_centers, "Show center of mass");
        });

        ui.add_space(10.0);

        // Collision layers
        ui.group(|ui| {
            ui.label(RichText::new("Collision Layers").strong());

            for layer in &mut self.collision_layers {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut layer.enabled, "");
                    ui.colored_label(layer.color, &layer.name);
                    ui.label(format!("({})", layer.id));
                });
            }
        });
    }

    fn show_ragdoll_tab(&mut self, ui: &mut Ui) {
        ui.heading("🦴 Ragdoll Physics");
        ui.add_space(10.0);

        // Preset selection
        ui.group(|ui| {
            ui.label(RichText::new("Ragdoll Preset").strong());

            ui.horizontal_wrapped(|ui| {
                for preset in RagdollPreset::all() {
                    if ui
                        .selectable_label(self.ragdoll_preset == *preset, format!("{:?}", preset))
                        .clicked()
                    {
                        self.ragdoll_preset = *preset;
                    }
                }
            });

            ui.label(format!("Bone count: {}", self.ragdoll_preset.bone_count()));
        });

        ui.add_space(10.0);

        // Joint settings
        ui.group(|ui| {
            ui.label(RichText::new("Joint Settings").strong());

            ui.add(
                egui::Slider::new(&mut self.ragdoll_joint_stiffness, 0.0..=500.0)
                    .text("Stiffness"),
            );
            ui.add(
                egui::Slider::new(&mut self.ragdoll_joint_damping, 0.0..=50.0)
                    .text("Damping"),
            );
            ui.add(
                egui::Slider::new(&mut self.ragdoll_impulse_multiplier, 0.1..=5.0)
                    .text("Impulse Multiplier"),
            );
            ui.add(
                egui::Slider::new(&mut self.ragdoll_blend_time, 0.0..=1.0)
                    .text("Blend Time")
                    .suffix(" s"),
            );
        });

        ui.add_space(10.0);

        // Test tools
        ui.group(|ui| {
            ui.label(RichText::new("Test Tools").strong());

            ui.horizontal(|ui| {
                if ui.button("🦴 Spawn Ragdoll").clicked() {
                    // Spawn test ragdoll
                }
                if ui.button("💨 Apply Impulse").clicked() {
                    // Apply test impulse
                }
                if ui.button("🔄 Reset All").clicked() {
                    // Reset all ragdolls
                }
            });

            ui.add(
                egui::Slider::new(&mut self.test_impulse_strength, 100.0..=2000.0)
                    .text("Impulse Strength"),
            );
        });
    }

    fn show_vehicle_tab(&mut self, ui: &mut Ui) {
        ui.heading("🚗 Vehicle Physics");
        ui.add_space(10.0);

        // Vehicle type
        ui.group(|ui| {
            ui.label(RichText::new("Vehicle Type").strong());

            ui.horizontal_wrapped(|ui| {
                for vtype in VehicleType::all() {
                    if ui
                        .selectable_label(self.vehicle_type == *vtype, format!("{:?}", vtype))
                        .clicked()
                    {
                        self.vehicle_type = *vtype;
                        self.apply_vehicle_preset(*vtype);
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Engine settings
        ui.group(|ui| {
            ui.label(RichText::new("Engine").strong());

            ui.add(
                egui::Slider::new(&mut self.engine_torque, 100.0..=2000.0)
                    .text("Max Torque")
                    .suffix(" Nm"),
            );
        });

        ui.add_space(10.0);

        // Suspension settings
        ui.group(|ui| {
            ui.label(RichText::new("Suspension").strong());

            ui.add(
                egui::Slider::new(&mut self.suspension_stiffness, 10000.0..=100000.0)
                    .text("Stiffness")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.suspension_damping, 1000.0..=10000.0)
                    .text("Damping"),
            );
        });

        ui.add_space(10.0);

        // Wheels and aerodynamics
        ui.group(|ui| {
            ui.label(RichText::new("Wheels & Aero").strong());

            ui.add(
                egui::Slider::new(&mut self.wheel_friction, 0.5..=2.0)
                    .text("Wheel Friction"),
            );
            ui.add(
                egui::Slider::new(&mut self.downforce_coefficient, 0.0..=1.0)
                    .text("Downforce"),
            );
        });

        ui.add_space(10.0);

        // Test tools
        ui.group(|ui| {
            ui.label(RichText::new("Test Tools").strong());

            ui.horizontal(|ui| {
                if ui.button("🚗 Spawn Vehicle").clicked() {
                    // Spawn test vehicle
                }
                if ui.button("🔄 Reset Position").clicked() {
                    // Reset vehicle position
                }
            });
        });
    }

    fn show_cloth_tab(&mut self, ui: &mut Ui) {
        ui.heading("🧵 Cloth Simulation");
        ui.add_space(10.0);

        // Quality settings
        ui.group(|ui| {
            ui.label(RichText::new("Quality").strong());

            ui.horizontal(|ui| {
                for quality in ClothQuality::all() {
                    if ui
                        .selectable_label(self.cloth_quality == *quality, format!("{:?}", quality))
                        .clicked()
                    {
                        self.cloth_quality = *quality;
                    }
                }
            });

            ui.label(format!(
                "Solver iterations: {}",
                self.cloth_quality.iterations()
            ));
        });

        ui.add_space(10.0);

        // Cloth physics
        ui.group(|ui| {
            ui.label(RichText::new("Physics").strong());

            ui.add(
                egui::Slider::new(&mut self.cloth_gravity_scale, 0.0..=2.0)
                    .text("Gravity Scale"),
            );
            ui.add(
                egui::Slider::new(&mut self.cloth_damping, 0.0..=0.1)
                    .text("Damping"),
            );
            ui.add(
                egui::Slider::new(&mut self.cloth_stiffness, 0.1..=2.0)
                    .text("Stiffness"),
            );
            ui.add(
                egui::Slider::new(&mut self.cloth_wind_influence, 0.0..=2.0)
                    .text("Wind Influence"),
            );
            ui.checkbox(&mut self.cloth_self_collision, "Self Collision");
        });

        ui.add_space(10.0);

        // Test tools
        ui.group(|ui| {
            ui.label(RichText::new("Test Tools").strong());

            ui.horizontal(|ui| {
                if ui.button("🧵 Spawn Cloth").clicked() {
                    // Spawn test cloth
                }
                if ui.button("💨 Apply Wind Gust").clicked() {
                    // Apply wind gust
                }
                if ui.button("📌 Pin Corners").clicked() {
                    // Pin cloth corners
                }
            });
        });
    }

    fn show_destruction_tab(&mut self, ui: &mut Ui) {
        ui.heading("💥 Destruction System");
        ui.add_space(10.0);

        // Fracture pattern
        ui.group(|ui| {
            ui.label(RichText::new("Fracture Pattern").strong());

            ui.horizontal_wrapped(|ui| {
                for pattern in DestructionPattern::all() {
                    if ui
                        .selectable_label(
                            self.destruction_pattern == *pattern,
                            format!("{:?}", pattern),
                        )
                        .clicked()
                    {
                        self.destruction_pattern = *pattern;
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Debris settings
        ui.group(|ui| {
            ui.label(RichText::new("Debris").strong());

            ui.add(
                egui::Slider::new(&mut self.debris_lifetime, 1.0..=30.0)
                    .text("Lifetime")
                    .suffix(" s"),
            );
            ui.add(
                egui::DragValue::new(&mut self.debris_count_max)
                    .range(10..=200)
                    .prefix("Max Count: "),
            );
        });

        ui.add_space(10.0);

        // Trigger settings
        ui.group(|ui| {
            ui.label(RichText::new("Triggers").strong());

            ui.add(
                egui::Slider::new(&mut self.destruction_impulse_threshold, 10.0..=500.0)
                    .text("Impulse Threshold"),
            );
            ui.checkbox(&mut self.destruction_chain_reaction, "Chain Reaction");
        });

        ui.add_space(10.0);

        // Test tools
        ui.group(|ui| {
            ui.label(RichText::new("Test Tools").strong());

            ui.horizontal(|ui| {
                if ui.button("🧱 Spawn Destructible").clicked() {
                    // Spawn destructible
                }
                if ui.button("💥 Trigger Explosion").clicked() {
                    // Trigger explosion
                }
                if ui.button("🧹 Clear Debris").clicked() {
                    // Clear all debris
                }
            });
        });
    }

    fn show_environment_tab(&mut self, ui: &mut Ui) {
        ui.heading("🌍 Environment Physics");
        ui.add_space(10.0);

        // Wind settings
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("💨 Wind").strong());
                ui.checkbox(&mut self.wind_enabled, "Enabled");
            });

            if self.wind_enabled {
                ui.add_space(5.0);

                egui::Grid::new("wind_grid")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Direction X:");
                        ui.add(egui::DragValue::new(&mut self.wind_direction[0]).speed(0.1));
                        ui.end_row();

                        ui.label("Direction Y:");
                        ui.add(egui::DragValue::new(&mut self.wind_direction[1]).speed(0.1));
                        ui.end_row();

                        ui.label("Direction Z:");
                        ui.add(egui::DragValue::new(&mut self.wind_direction[2]).speed(0.1));
                        ui.end_row();
                    });

                ui.add(
                    egui::Slider::new(&mut self.wind_strength, 0.0..=50.0)
                        .text("Strength"),
                );
                ui.add(
                    egui::Slider::new(&mut self.wind_turbulence, 0.0..=1.0)
                        .text("Turbulence"),
                );
            }
        });

        ui.add_space(10.0);

        // Gravity zones
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🌐 Gravity Zones").strong());
                if ui.button("➕ Add").clicked() {
                    self.gravity_zones.push(GravityZoneConfig::default());
                }
            });

            ui.separator();

            if self.gravity_zones.is_empty() {
                ui.label("No gravity zones defined.");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        let mut to_remove = None;

                        for (i, zone) in self.gravity_zones.iter_mut().enumerate() {
                            let is_selected = self.selected_gravity_zone == Some(i);

                            ui.horizontal(|ui| {
                                ui.checkbox(&mut zone.enabled, "");
                                if ui.selectable_label(is_selected, &zone.name).clicked() {
                                    self.selected_gravity_zone = Some(i);
                                }
                                ui.label(format!("{:?}", zone.shape));
                                ui.label(format!("{:.1} m/s²", zone.strength));

                                if ui.small_button("🗑").clicked() {
                                    to_remove = Some(i);
                                }
                            });
                        }

                        if let Some(i) = to_remove {
                            self.gravity_zones.remove(i);
                            self.selected_gravity_zone = None;
                        }
                    });
            }
        });

        // Selected zone editor
        if let Some(idx) = self.selected_gravity_zone {
            if let Some(zone) = self.gravity_zones.get_mut(idx) {
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.label(RichText::new("Zone Editor").strong());

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut zone.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Shape:");
                        for shape in GravityZoneShape::all() {
                            if ui
                                .selectable_label(zone.shape == *shape, format!("{:?}", shape))
                                .clicked()
                            {
                                zone.shape = *shape;
                            }
                        }
                    });

                    ui.add(
                        egui::Slider::new(&mut zone.strength, 0.0..=30.0)
                            .text("Strength")
                            .suffix(" m/s²"),
                    );

                    ui.checkbox(&mut zone.falloff, "Distance Falloff");
                });
            }
        }
    }

    fn draw_timing_bar(&self, ui: &mut Ui) {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), egui::Sense::hover());

        let total = self.stats.step_time_ms.max(0.001);
        let bar_width = rect.width();

        // Calculate widths
        let broad_width = (self.stats.broad_phase_time_ms / total) * bar_width;
        let narrow_width = (self.stats.narrow_phase_time_ms / total) * bar_width;
        let solver_width = (self.stats.solver_time_ms / total) * bar_width;
        let ccd_width = (self.stats.ccd_time_ms / total) * bar_width;

        let mut x = rect.left();

        // Broad phase (blue)
        let broad_rect = egui::Rect::from_min_size(
            egui::Pos2::new(x, rect.top()),
            Vec2::new(broad_width, rect.height()),
        );
        ui.painter()
            .rect_filled(broad_rect, 0.0, Color32::from_rgb(100, 150, 255));
        x += broad_width;

        // Narrow phase (green)
        let narrow_rect = egui::Rect::from_min_size(
            egui::Pos2::new(x, rect.top()),
            Vec2::new(narrow_width, rect.height()),
        );
        ui.painter()
            .rect_filled(narrow_rect, 0.0, Color32::from_rgb(100, 200, 100));
        x += narrow_width;

        // Solver (orange)
        let solver_rect = egui::Rect::from_min_size(
            egui::Pos2::new(x, rect.top()),
            Vec2::new(solver_width, rect.height()),
        );
        ui.painter()
            .rect_filled(solver_rect, 0.0, Color32::from_rgb(255, 165, 0));
        x += solver_width;

        // CCD (red)
        let ccd_rect = egui::Rect::from_min_size(
            egui::Pos2::new(x, rect.top()),
            Vec2::new(ccd_width, rect.height()),
        );
        ui.painter()
            .rect_filled(ccd_rect, 0.0, Color32::from_rgb(255, 100, 100));

        // Legend
        ui.horizontal(|ui| {
            ui.colored_label(Color32::from_rgb(100, 150, 255), "Broad");
            ui.colored_label(Color32::from_rgb(100, 200, 100), "Narrow");
            ui.colored_label(Color32::from_rgb(255, 165, 0), "Solver");
            ui.colored_label(Color32::from_rgb(255, 100, 100), "CCD");
        });
    }

    fn apply_vehicle_preset(&mut self, vtype: VehicleType) {
        match vtype {
            VehicleType::Sedan => {
                self.engine_torque = 300.0;
                self.suspension_stiffness = 35000.0;
                self.suspension_damping = 4000.0;
                self.wheel_friction = 1.0;
                self.downforce_coefficient = 0.1;
            }
            VehicleType::SportsCar => {
                self.engine_torque = 600.0;
                self.suspension_stiffness = 60000.0;
                self.suspension_damping = 5000.0;
                self.wheel_friction = 1.2;
                self.downforce_coefficient = 0.5;
            }
            VehicleType::Truck => {
                self.engine_torque = 800.0;
                self.suspension_stiffness = 80000.0;
                self.suspension_damping = 8000.0;
                self.wheel_friction = 0.9;
                self.downforce_coefficient = 0.05;
            }
            VehicleType::Motorcycle => {
                self.engine_torque = 200.0;
                self.suspension_stiffness = 25000.0;
                self.suspension_damping = 3000.0;
                self.wheel_friction = 1.1;
                self.downforce_coefficient = 0.0;
            }
            VehicleType::Tank => {
                self.engine_torque = 1500.0;
                self.suspension_stiffness = 100000.0;
                self.suspension_damping = 12000.0;
                self.wheel_friction = 1.5;
                self.downforce_coefficient = 0.0;
            }
            VehicleType::Custom => {}
        }
    }

    // Getters for testing
    pub fn visualization_mode(&self) -> PhysicsVisualization {
        self.visualization_mode
    }

    pub fn simulation_mode(&self) -> SimulationMode {
        self.simulation_mode
    }

    pub fn ragdoll_preset(&self) -> RagdollPreset {
        self.ragdoll_preset
    }

    pub fn vehicle_type(&self) -> VehicleType {
        self.vehicle_type
    }

    pub fn cloth_quality(&self) -> ClothQuality {
        self.cloth_quality
    }

    pub fn destruction_pattern(&self) -> DestructionPattern {
        self.destruction_pattern
    }

    pub fn gravity_zone_count(&self) -> usize {
        self.gravity_zones.len()
    }

    pub fn is_wind_enabled(&self) -> bool {
        self.wind_enabled
    }

    pub fn set_visualization_mode(&mut self, mode: PhysicsVisualization) {
        self.visualization_mode = mode;
    }

    pub fn set_simulation_mode(&mut self, mode: SimulationMode) {
        self.simulation_mode = mode;
    }

    pub fn add_gravity_zone(&mut self, zone: GravityZoneConfig) {
        self.gravity_zones.push(zone);
    }

    pub fn set_wind(&mut self, enabled: bool, strength: f32) {
        self.wind_enabled = enabled;
        self.wind_strength = strength;
    }

    pub fn collision_layer_count(&self) -> usize {
        self.collision_layers.len()
    }
}

impl Panel for PhysicsPanel {
    fn name(&self) -> &'static str {
        "Physics"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            PhysicsTab::Overview => self.show_overview_tab(ui),
            PhysicsTab::Visualization => self.show_visualization_tab(ui),
            PhysicsTab::Ragdoll => self.show_ragdoll_tab(ui),
            PhysicsTab::Vehicle => self.show_vehicle_tab(ui),
            PhysicsTab::Cloth => self.show_cloth_tab(ui),
            PhysicsTab::Destruction => self.show_destruction_tab(ui),
            PhysicsTab::Environment => self.show_environment_tab(ui),
        }
    }

    fn update(&mut self) {
        // Sample step time history
        if self.step_time_history.len() >= 120 {
            self.step_time_history.pop_front();
        }
        self.step_time_history.push_back(self.stats.step_time_ms);

        // Handle step-once mode
        if self.simulation_mode == SimulationMode::StepOnce {
            self.simulation_mode = SimulationMode::Paused;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_panel_creation() {
        let panel = PhysicsPanel::new();
        assert_eq!(panel.visualization_mode(), PhysicsVisualization::None);
        assert_eq!(panel.simulation_mode(), SimulationMode::Running);
    }

    #[test]
    fn test_visualization_modes() {
        let mut panel = PhysicsPanel::new();
        panel.set_visualization_mode(PhysicsVisualization::Colliders);
        assert_eq!(panel.visualization_mode(), PhysicsVisualization::Colliders);
    }

    #[test]
    fn test_simulation_modes() {
        let mut panel = PhysicsPanel::new();
        panel.set_simulation_mode(SimulationMode::Paused);
        assert_eq!(panel.simulation_mode(), SimulationMode::Paused);
    }

    #[test]
    fn test_ragdoll_presets() {
        let panel = PhysicsPanel::new();
        assert_eq!(panel.ragdoll_preset(), RagdollPreset::Humanoid);
        assert_eq!(RagdollPreset::Humanoid.bone_count(), 15);
        assert_eq!(RagdollPreset::Quadruped.bone_count(), 18);
    }

    #[test]
    fn test_vehicle_types() {
        let panel = PhysicsPanel::new();
        assert_eq!(panel.vehicle_type(), VehicleType::Sedan);
    }

    #[test]
    fn test_cloth_quality() {
        let panel = PhysicsPanel::new();
        assert_eq!(panel.cloth_quality(), ClothQuality::Medium);
        assert_eq!(ClothQuality::Medium.iterations(), 8);
        assert_eq!(ClothQuality::Ultra.iterations(), 32);
    }

    #[test]
    fn test_destruction_patterns() {
        let panel = PhysicsPanel::new();
        assert_eq!(panel.destruction_pattern(), DestructionPattern::Voronoi);
    }

    #[test]
    fn test_gravity_zones() {
        let mut panel = PhysicsPanel::new();
        assert_eq!(panel.gravity_zone_count(), 0);

        panel.add_gravity_zone(GravityZoneConfig::default());
        assert_eq!(panel.gravity_zone_count(), 1);
    }

    #[test]
    fn test_wind_settings() {
        let mut panel = PhysicsPanel::new();
        assert!(!panel.is_wind_enabled());

        panel.set_wind(true, 25.0);
        assert!(panel.is_wind_enabled());
        assert_eq!(panel.wind_strength, 25.0);
    }

    #[test]
    fn test_collision_layers() {
        let panel = PhysicsPanel::new();
        assert_eq!(panel.collision_layer_count(), 5); // Default, Characters, Projectiles, Triggers, Debris
    }

    #[test]
    fn test_panel_trait_implementation() {
        let panel = PhysicsPanel::new();
        assert_eq!(panel.name(), "Physics");
    }
}
