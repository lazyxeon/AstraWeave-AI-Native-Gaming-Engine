//! # Vehicle Physics System
//!
//! Provides realistic vehicle simulation including:
//! - Raycast suspension (industry-standard approach)
//! - Friction curves (Pacejka-inspired slip model)
//! - Engine and transmission simulation
//! - Steering mechanics
//!
//! ## Features
//!
//! - **Wheeled Vehicles**: Cars, trucks, motorcycles
//! - **Suspension**: Spring-damper raycast system
//! - **Friction**: Slip ratio and slip angle curves
//! - **Drivetrain**: Engine torque, gear ratios, differential
//!
//! ## Usage
//!
//! ```rust
//! use astraweave_physics::vehicle::{VehicleConfig, Vehicle, WheelConfig};
//! use glam::Vec3;
//!
//! let config = VehicleConfig {
//!     mass: 1500.0,
//!     wheels: vec![
//!         WheelConfig::front_left(Vec3::new(-0.8, 0.0, 1.2)),
//!         WheelConfig::front_right(Vec3::new(0.8, 0.0, 1.2)),
//!         WheelConfig::rear_left(Vec3::new(-0.8, 0.0, -1.2)),
//!         WheelConfig::rear_right(Vec3::new(0.8, 0.0, -1.2)),
//!     ],
//!     ..Default::default()
//! };
//! ```

use crate::{BodyId, PhysicsWorld};
use glam::{Quat, Vec3};

/// Unique identifier for a vehicle
pub type VehicleId = u64;

/// Wheel position preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelPosition {
    FrontLeft,
    FrontRight,
    RearLeft,
    RearRight,
    /// Custom position (e.g., for 6-wheeled trucks)
    Custom(u8),
}

/// Configuration for a single wheel
#[derive(Debug, Clone)]
pub struct WheelConfig {
    /// Position relative to vehicle center of mass
    pub position: Vec3,
    /// Wheel radius
    pub radius: f32,
    /// Wheel width (for friction calculation)
    pub width: f32,
    /// Whether this wheel can steer
    pub steerable: bool,
    /// Whether this wheel is driven (receives engine power)
    pub driven: bool,
    /// Suspension rest length
    pub suspension_rest_length: f32,
    /// Suspension spring stiffness (N/m)
    pub suspension_stiffness: f32,
    /// Suspension damping coefficient
    pub suspension_damping: f32,
    /// Maximum suspension compression
    pub suspension_max_compression: f32,
    /// Maximum suspension extension
    pub suspension_max_extension: f32,
    /// Wheel position identifier
    pub position_id: WheelPosition,
}

impl Default for WheelConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            radius: 0.35,
            width: 0.25,
            steerable: false,
            driven: false,
            suspension_rest_length: 0.3,
            suspension_stiffness: 35000.0,
            suspension_damping: 4500.0,
            suspension_max_compression: 0.1,
            suspension_max_extension: 0.2,
            position_id: WheelPosition::Custom(0),
        }
    }
}

impl WheelConfig {
    /// Create a front-left wheel configuration
    pub fn front_left(position: Vec3) -> Self {
        Self {
            position,
            steerable: true,
            driven: false, // FWD would set this true
            position_id: WheelPosition::FrontLeft,
            ..Default::default()
        }
    }

    /// Create a front-right wheel configuration
    pub fn front_right(position: Vec3) -> Self {
        Self {
            position,
            steerable: true,
            driven: false,
            position_id: WheelPosition::FrontRight,
            ..Default::default()
        }
    }

    /// Create a rear-left wheel configuration (RWD driven)
    pub fn rear_left(position: Vec3) -> Self {
        Self {
            position,
            steerable: false,
            driven: true, // RWD
            position_id: WheelPosition::RearLeft,
            ..Default::default()
        }
    }

    /// Create a rear-right wheel configuration (RWD driven)
    pub fn rear_right(position: Vec3) -> Self {
        Self {
            position,
            steerable: false,
            driven: true, // RWD
            position_id: WheelPosition::RearRight,
            ..Default::default()
        }
    }

    /// Set as AWD (all-wheel drive)
    pub fn with_drive(mut self) -> Self {
        self.driven = true;
        self
    }

    /// Set wheel radius
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set suspension parameters
    pub fn with_suspension(mut self, stiffness: f32, damping: f32, rest_length: f32) -> Self {
        self.suspension_stiffness = stiffness;
        self.suspension_damping = damping;
        self.suspension_rest_length = rest_length;
        self
    }
}

/// Drivetrain type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrivetrainType {
    /// Front-wheel drive
    FWD,
    /// Rear-wheel drive
    #[default]
    RWD,
    /// All-wheel drive
    AWD,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum engine torque (Nm)
    pub max_torque: f32,
    /// RPM at which max torque is produced
    pub max_torque_rpm: f32,
    /// Maximum engine RPM
    pub max_rpm: f32,
    /// Idle RPM
    pub idle_rpm: f32,
    /// Engine braking coefficient
    pub engine_braking: f32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_torque: 400.0,
            max_torque_rpm: 4500.0,
            max_rpm: 7000.0,
            idle_rpm: 800.0,
            engine_braking: 0.3,
        }
    }
}

impl EngineConfig {
    /// Calculate torque at given RPM using simplified torque curve
    pub fn torque_at_rpm(&self, rpm: f32) -> f32 {
        if rpm < self.idle_rpm {
            return 0.0;
        }
        if rpm > self.max_rpm {
            return 0.0;
        }

        // Simple parabolic torque curve
        let normalized = (rpm - self.idle_rpm) / (self.max_torque_rpm - self.idle_rpm);
        let falloff = (rpm - self.max_torque_rpm) / (self.max_rpm - self.max_torque_rpm);

        if rpm <= self.max_torque_rpm {
            // Rising portion
            self.max_torque * (1.0 - (1.0 - normalized).powi(2))
        } else {
            // Falling portion
            self.max_torque * (1.0 - falloff.powi(2)).max(0.0)
        }
    }
}

/// Transmission configuration
#[derive(Debug, Clone)]
pub struct TransmissionConfig {
    /// Gear ratios (index 0 = 1st gear, etc.)
    pub gear_ratios: Vec<f32>,
    /// Reverse gear ratio
    pub reverse_ratio: f32,
    /// Final drive ratio (differential)
    pub final_drive: f32,
    /// Shift time in seconds
    pub shift_time: f32,
}

impl Default for TransmissionConfig {
    fn default() -> Self {
        Self {
            gear_ratios: vec![3.5, 2.1, 1.4, 1.0, 0.8, 0.65],
            reverse_ratio: -3.2,
            final_drive: 3.7,
            shift_time: 0.2,
        }
    }
}

impl TransmissionConfig {
    /// Get effective gear ratio for current gear
    pub fn effective_ratio(&self, gear: i32) -> f32 {
        let gear_ratio = if gear == 0 {
            0.0 // Neutral
        } else if gear < 0 {
            self.reverse_ratio
        } else {
            self.gear_ratios
                .get((gear - 1) as usize)
                .copied()
                .unwrap_or(1.0)
        };

        gear_ratio * self.final_drive
    }

    /// Number of forward gears
    pub fn num_gears(&self) -> usize {
        self.gear_ratios.len()
    }
}

/// Friction curve parameters (simplified Pacejka)
#[derive(Debug, Clone, Copy)]
pub struct FrictionCurve {
    /// Optimal slip ratio for maximum friction
    pub optimal_slip: f32,
    /// Friction coefficient at optimal slip
    pub peak_friction: f32,
    /// Friction coefficient at high slip (sliding)
    pub sliding_friction: f32,
    /// Curve stiffness
    pub stiffness: f32,
}

impl Default for FrictionCurve {
    fn default() -> Self {
        Self {
            optimal_slip: 0.08,
            peak_friction: 1.2,
            sliding_friction: 0.8,
            stiffness: 10.0,
        }
    }
}

impl FrictionCurve {
    /// Calculate friction coefficient at given slip ratio
    pub fn friction_at_slip(&self, slip: f32) -> f32 {
        let abs_slip = slip.abs();

        if abs_slip < 0.001 {
            return 0.0;
        }

        // Simplified magic formula
        let x = abs_slip / self.optimal_slip;
        let peak = self.peak_friction;
        let slide = self.sliding_friction;

        if x <= 1.0 {
            // Rising portion to peak
            peak * (1.0 - (-self.stiffness * x).exp())
        } else {
            // Falling portion after peak
            let decay = ((x - 1.0) * 2.0).min(1.0);
            peak - (peak - slide) * decay
        }
    }

    /// Tarmac/asphalt friction curve
    pub fn tarmac() -> Self {
        Self {
            optimal_slip: 0.08,
            peak_friction: 1.2,
            sliding_friction: 0.9,
            stiffness: 12.0,
        }
    }

    /// Gravel friction curve
    pub fn gravel() -> Self {
        Self {
            optimal_slip: 0.15,
            peak_friction: 0.8,
            sliding_friction: 0.6,
            stiffness: 6.0,
        }
    }

    /// Ice friction curve
    pub fn ice() -> Self {
        Self {
            optimal_slip: 0.05,
            peak_friction: 0.3,
            sliding_friction: 0.15,
            stiffness: 20.0,
        }
    }

    /// Mud friction curve
    pub fn mud() -> Self {
        Self {
            optimal_slip: 0.2,
            peak_friction: 0.5,
            sliding_friction: 0.4,
            stiffness: 4.0,
        }
    }
}

/// Vehicle configuration
#[derive(Debug, Clone)]
pub struct VehicleConfig {
    /// Vehicle mass (kg)
    pub mass: f32,
    /// Wheel configurations
    pub wheels: Vec<WheelConfig>,
    /// Drivetrain type
    pub drivetrain: DrivetrainType,
    /// Engine configuration
    pub engine: EngineConfig,
    /// Transmission configuration
    pub transmission: TransmissionConfig,
    /// Longitudinal friction curve
    pub friction_forward: FrictionCurve,
    /// Lateral friction curve
    pub friction_lateral: FrictionCurve,
    /// Aerodynamic drag coefficient
    pub drag_coefficient: f32,
    /// Frontal area (m²)
    pub frontal_area: f32,
    /// Center of mass offset from geometric center
    pub center_of_mass_offset: Vec3,
    /// Maximum steering angle (radians)
    pub max_steering_angle: f32,
    /// Brake force (N)
    pub brake_force: f32,
    /// Handbrake force multiplier for rear wheels
    pub handbrake_multiplier: f32,
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            mass: 1500.0,
            wheels: vec![
                WheelConfig::front_left(Vec3::new(-0.8, 0.0, 1.2)),
                WheelConfig::front_right(Vec3::new(0.8, 0.0, 1.2)),
                WheelConfig::rear_left(Vec3::new(-0.8, 0.0, -1.2)),
                WheelConfig::rear_right(Vec3::new(0.8, 0.0, -1.2)),
            ],
            drivetrain: DrivetrainType::RWD,
            engine: EngineConfig::default(),
            transmission: TransmissionConfig::default(),
            friction_forward: FrictionCurve::tarmac(),
            friction_lateral: FrictionCurve::tarmac(),
            drag_coefficient: 0.35,
            frontal_area: 2.2,
            center_of_mass_offset: Vec3::new(0.0, -0.2, 0.0),
            max_steering_angle: 0.6, // ~35 degrees
            brake_force: 15000.0,
            handbrake_multiplier: 2.0,
        }
    }
}

/// Runtime state of a single wheel
#[derive(Debug, Clone)]
pub struct WheelState {
    /// Current suspension compression (0 = rest, positive = compressed)
    pub compression: f32,
    /// Previous compression (for damping)
    pub prev_compression: f32,
    /// Current wheel rotation speed (rad/s)
    pub rotation_speed: f32,
    /// Current steering angle (radians)
    pub steering_angle: f32,
    /// Is the wheel touching the ground?
    pub grounded: bool,
    /// Ground contact point (world space)
    pub contact_point: Vec3,
    /// Ground normal at contact
    pub contact_normal: Vec3,
    /// Current slip ratio (longitudinal)
    pub slip_ratio: f32,
    /// Current slip angle (lateral, radians)
    pub slip_angle: f32,
    /// Force applied by this wheel
    pub force: Vec3,
    /// Suspension force magnitude
    pub suspension_force: f32,
}

impl Default for WheelState {
    fn default() -> Self {
        Self {
            compression: 0.0,
            prev_compression: 0.0,
            rotation_speed: 0.0,
            steering_angle: 0.0,
            grounded: false,
            contact_point: Vec3::ZERO,
            contact_normal: Vec3::Y,
            slip_ratio: 0.0,
            slip_angle: 0.0,
            force: Vec3::ZERO,
            suspension_force: 0.0,
        }
    }
}

/// Vehicle input state
#[derive(Debug, Clone, Copy, Default)]
pub struct VehicleInput {
    /// Throttle (0.0 to 1.0)
    pub throttle: f32,
    /// Brake (0.0 to 1.0)
    pub brake: f32,
    /// Steering (-1.0 left to 1.0 right)
    pub steering: f32,
    /// Handbrake (0.0 to 1.0)
    pub handbrake: f32,
    /// Clutch (0.0 engaged to 1.0 disengaged)
    pub clutch: f32,
    /// Gear shift request (-1 = down, 0 = none, 1 = up)
    pub shift: i32,
}

/// Runtime state of a vehicle
#[derive(Debug)]
pub struct Vehicle {
    /// Unique ID
    pub id: VehicleId,
    /// Physics body ID
    pub body_id: BodyId,
    /// Configuration
    pub config: VehicleConfig,
    /// Wheel states
    pub wheels: Vec<WheelState>,
    /// Current gear (0 = neutral, negative = reverse)
    pub current_gear: i32,
    /// Current engine RPM
    pub engine_rpm: f32,
    /// Time remaining in gear shift
    pub shift_timer: f32,
    /// Current speed (m/s)
    pub speed: f32,
    /// Current velocity (world space)
    pub velocity: Vec3,
    /// Forward direction (world space)
    pub forward: Vec3,
    /// Right direction (world space)
    pub right: Vec3,
    /// Up direction (world space)
    pub up: Vec3,
}

impl Vehicle {
    /// Create a new vehicle
    pub fn new(id: VehicleId, body_id: BodyId, config: VehicleConfig) -> Self {
        let num_wheels = config.wheels.len();
        Self {
            id,
            body_id,
            config,
            wheels: vec![WheelState::default(); num_wheels],
            current_gear: 1, // Start in 1st
            engine_rpm: 800.0, // Idle
            shift_timer: 0.0,
            speed: 0.0,
            velocity: Vec3::ZERO,
            forward: Vec3::Z,
            right: Vec3::X,
            up: Vec3::Y,
        }
    }

    /// Update vehicle orientation from physics body
    pub fn update_orientation(&mut self, rotation: Quat) {
        self.forward = rotation * Vec3::Z;
        self.right = rotation * Vec3::X;
        self.up = rotation * Vec3::Y;
    }

    /// Get speed in km/h
    pub fn speed_kmh(&self) -> f32 {
        self.speed * 3.6
    }

    /// Get speed in mph
    pub fn speed_mph(&self) -> f32 {
        self.speed * 2.237
    }

    /// Check if currently shifting gears
    pub fn is_shifting(&self) -> bool {
        self.shift_timer > 0.0
    }

    /// Shift up a gear
    pub fn shift_up(&mut self) {
        let max_gear = self.config.transmission.num_gears() as i32;
        if self.current_gear < max_gear && !self.is_shifting() {
            self.current_gear += 1;
            self.shift_timer = self.config.transmission.shift_time;
        }
    }

    /// Shift down a gear
    pub fn shift_down(&mut self) {
        if self.current_gear > -1 && !self.is_shifting() {
            self.current_gear -= 1;
            self.shift_timer = self.config.transmission.shift_time;
        }
    }

    /// Get number of grounded wheels
    pub fn grounded_wheels(&self) -> usize {
        self.wheels.iter().filter(|w| w.grounded).count()
    }

    /// Check if vehicle is airborne
    pub fn is_airborne(&self) -> bool {
        self.grounded_wheels() == 0
    }

    /// Get total suspension force
    pub fn total_suspension_force(&self) -> f32 {
        self.wheels.iter().map(|w| w.suspension_force).sum()
    }

    /// Get average slip ratio
    pub fn average_slip_ratio(&self) -> f32 {
        let grounded: Vec<_> = self.wheels.iter().filter(|w| w.grounded).collect();
        if grounded.is_empty() {
            return 0.0;
        }
        grounded.iter().map(|w| w.slip_ratio.abs()).sum::<f32>() / grounded.len() as f32
    }

    /// Get average slip angle
    pub fn average_slip_angle(&self) -> f32 {
        let grounded: Vec<_> = self.wheels.iter().filter(|w| w.grounded).collect();
        if grounded.is_empty() {
            return 0.0;
        }
        grounded.iter().map(|w| w.slip_angle.abs()).sum::<f32>() / grounded.len() as f32
    }
}

/// Vehicle physics manager
#[derive(Debug)]
pub struct VehicleManager {
    vehicles: Vec<Vehicle>,
    next_id: VehicleId,
}

impl Default for VehicleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VehicleManager {
    /// Create a new vehicle manager
    pub fn new() -> Self {
        Self {
            vehicles: Vec::new(),
            next_id: 1,
        }
    }

    /// Spawn a vehicle
    pub fn spawn(&mut self, physics: &mut PhysicsWorld, position: Vec3, config: VehicleConfig) -> VehicleId {
        let body_id = physics.add_dynamic_box(
            position + config.center_of_mass_offset,
            Vec3::new(1.0, 0.5, 2.0), // Approximate vehicle half-extents
            config.mass,
            crate::Layers::DEFAULT,
        );

        // Enable CCD for fast-moving vehicles
        physics.enable_ccd(body_id);

        let id = self.next_id;
        self.next_id += 1;

        let vehicle = Vehicle::new(id, body_id, config);
        self.vehicles.push(vehicle);

        id
    }

    /// Get a vehicle by ID
    pub fn get(&self, id: VehicleId) -> Option<&Vehicle> {
        self.vehicles.iter().find(|v| v.id == id)
    }

    /// Get a mutable vehicle by ID
    pub fn get_mut(&mut self, id: VehicleId) -> Option<&mut Vehicle> {
        self.vehicles.iter_mut().find(|v| v.id == id)
    }

    /// Update all vehicles
    pub fn update(&mut self, physics: &mut PhysicsWorld, dt: f32) {
        for vehicle in &mut self.vehicles {
            Self::update_vehicle(vehicle, physics, dt);
        }
    }

    /// Update vehicle with input
    pub fn update_with_input(&mut self, id: VehicleId, physics: &mut PhysicsWorld, input: &VehicleInput, dt: f32) {
        if let Some(vehicle) = self.get_mut(id) {
            // Handle gear shifts
            if input.shift > 0 {
                vehicle.shift_up();
            } else if input.shift < 0 {
                vehicle.shift_down();
            }

            // Update shift timer
            if vehicle.shift_timer > 0.0 {
                vehicle.shift_timer = (vehicle.shift_timer - dt).max(0.0);
            }

            // Apply steering
            for (i, wheel) in vehicle.wheels.iter_mut().enumerate() {
                if vehicle.config.wheels[i].steerable {
                    wheel.steering_angle = input.steering * vehicle.config.max_steering_angle;
                }
            }

            Self::update_vehicle(vehicle, physics, dt);
            Self::apply_forces(vehicle, physics, input, dt);
        }
    }

    fn update_vehicle(vehicle: &mut Vehicle, physics: &PhysicsWorld, _dt: f32) {
        // Get vehicle transform
        if let Some(transform) = physics.body_transform(vehicle.body_id) {
            let position = Vec3::new(transform.w_axis.x, transform.w_axis.y, transform.w_axis.z);

            // Extract rotation (simplified - assumes orthonormal)
            let rotation = Quat::from_mat4(&transform);
            vehicle.update_orientation(rotation);

            // Get velocity
            if let Some(vel) = physics.get_velocity(vehicle.body_id) {
                vehicle.velocity = vel;
                vehicle.speed = vel.length();
            }

            // Update wheel states with raycasts
            for (i, wheel_config) in vehicle.config.wheels.iter().enumerate() {
                let wheel_state = &mut vehicle.wheels[i];

                // Calculate wheel world position
                let wheel_pos = position + rotation * wheel_config.position;

                // Raycast downward from wheel
                let ray_origin = wheel_pos + vehicle.up * wheel_config.suspension_max_extension;
                let ray_length = wheel_config.suspension_rest_length
                    + wheel_config.suspension_max_compression
                    + wheel_config.suspension_max_extension
                    + wheel_config.radius;

                // Perform raycast
                if let Some((hit_point, hit_normal, _body_id, _distance)) =
                    physics.raycast(ray_origin, -vehicle.up, ray_length)
                {
                    wheel_state.grounded = true;
                    wheel_state.contact_point = hit_point;
                    wheel_state.contact_normal = hit_normal;

                    // Calculate suspension compression
                    let suspension_length = (wheel_pos - hit_point).length() - wheel_config.radius;
                    wheel_state.prev_compression = wheel_state.compression;
                    wheel_state.compression =
                        wheel_config.suspension_rest_length - suspension_length;
                } else {
                    wheel_state.grounded = false;
                    wheel_state.prev_compression = wheel_state.compression;
                    wheel_state.compression = -wheel_config.suspension_max_extension;
                }
            }
        }
    }

    fn apply_forces(vehicle: &mut Vehicle, physics: &mut PhysicsWorld, input: &VehicleInput, dt: f32) {
        let mut total_force = Vec3::ZERO;
        let mut total_torque = Vec3::ZERO;

        // Get vehicle transform for force application points
        let Some(transform) = physics.body_transform(vehicle.body_id) else {
            return;
        };
        let position = Vec3::new(transform.w_axis.x, transform.w_axis.y, transform.w_axis.z);
        let rotation = Quat::from_mat4(&transform);

        // Calculate engine torque
        let effective_throttle = if vehicle.is_shifting() { 0.0 } else { input.throttle };
        let gear_ratio = vehicle.config.transmission.effective_ratio(vehicle.current_gear);
        let engine_torque = vehicle.config.engine.torque_at_rpm(vehicle.engine_rpm) * effective_throttle;
        let wheel_torque = engine_torque * gear_ratio;

        // Count driven wheels
        let driven_count = vehicle.config.wheels.iter().filter(|w| w.driven).count() as f32;

        for (i, wheel_config) in vehicle.config.wheels.iter().enumerate() {
            let wheel_state = &mut vehicle.wheels[i];

            if !wheel_state.grounded {
                continue;
            }

            let wheel_world_pos = position + rotation * wheel_config.position;

            // Suspension force (spring + damper)
            let spring_force = wheel_state.compression * wheel_config.suspension_stiffness;
            let damper_velocity = (wheel_state.compression - wheel_state.prev_compression) / dt;
            let damper_force = damper_velocity * wheel_config.suspension_damping;
            let suspension_force = (spring_force + damper_force).max(0.0);
            wheel_state.suspension_force = suspension_force;

            // Normal force on this wheel
            let normal_force = suspension_force;

            // Calculate wheel direction with steering
            let steer_rotation = Quat::from_rotation_y(wheel_state.steering_angle);
            let wheel_forward = rotation * steer_rotation * Vec3::Z;
            let wheel_right = rotation * steer_rotation * Vec3::X;

            // Velocity at wheel contact point
            let contact_velocity = vehicle.velocity; // Simplified - ignores angular velocity

            // Longitudinal velocity (along wheel forward)
            let long_velocity = contact_velocity.dot(wheel_forward);

            // Lateral velocity (perpendicular to wheel forward)
            let lat_velocity = contact_velocity.dot(wheel_right);

            // Calculate slip ratio
            let wheel_speed = wheel_state.rotation_speed * wheel_config.radius;
            let slip_ratio = if long_velocity.abs() > 0.5 {
                (wheel_speed - long_velocity) / long_velocity.abs()
            } else if wheel_speed.abs() > 0.1 {
                wheel_speed.signum()
            } else {
                0.0
            };
            wheel_state.slip_ratio = slip_ratio.clamp(-1.0, 1.0);

            // Calculate slip angle
            let slip_angle = if long_velocity.abs() > 0.5 {
                (-lat_velocity / long_velocity.abs()).atan()
            } else {
                0.0
            };
            wheel_state.slip_angle = slip_angle;

            // Friction forces
            let long_friction = vehicle.config.friction_forward.friction_at_slip(slip_ratio);
            let lat_friction = vehicle.config.friction_lateral.friction_at_slip(slip_angle.abs());

            // Longitudinal force (drive/brake)
            let mut long_force = 0.0;

            // Drive force
            if wheel_config.driven && driven_count > 0.0 {
                let torque_per_wheel = wheel_torque / driven_count;
                let max_friction_force = normal_force * long_friction;
                let drive_force = (torque_per_wheel / wheel_config.radius).min(max_friction_force);
                long_force += drive_force * effective_throttle;
            }

            // Brake force
            let brake_input = if wheel_config.position_id == WheelPosition::RearLeft
                || wheel_config.position_id == WheelPosition::RearRight
            {
                input.brake + input.handbrake * vehicle.config.handbrake_multiplier
            } else {
                input.brake
            };

            if brake_input > 0.0 {
                let max_brake = vehicle.config.brake_force * brake_input;
                let brake_friction_limit = normal_force * long_friction;
                let brake_force = max_brake.min(brake_friction_limit);
                long_force -= brake_force * long_velocity.signum();
            }

            // Lateral force (cornering)
            let lat_force = -lat_velocity.signum() * normal_force * lat_friction;

            // Total wheel force
            let wheel_force = wheel_forward * long_force + wheel_right * lat_force + wheel_state.contact_normal * suspension_force;
            wheel_state.force = wheel_force;

            // Accumulate forces
            total_force += wheel_force;

            // Calculate torque from force application point
            let force_arm = wheel_world_pos - position;
            total_torque += force_arm.cross(wheel_force);

            // Update wheel rotation
            let angular_accel = if wheel_config.driven && driven_count > 0.0 {
                let torque_per_wheel = wheel_torque / driven_count;
                let wheel_inertia = 0.5 * 10.0 * wheel_config.radius.powi(2); // Simplified
                (torque_per_wheel - long_force * wheel_config.radius) / wheel_inertia
            } else {
                -long_force * wheel_config.radius / (0.5 * 10.0 * wheel_config.radius.powi(2))
            };
            wheel_state.rotation_speed += angular_accel * dt;

            // Apply friction to slow wheel when not driven
            if !wheel_config.driven || effective_throttle < 0.01 {
                wheel_state.rotation_speed *= 0.99; // Rolling resistance
            }
        }

        // Aerodynamic drag
        let speed_sq = vehicle.speed * vehicle.speed;
        let drag_force = 0.5 * 1.225 * vehicle.config.drag_coefficient * vehicle.config.frontal_area * speed_sq;
        total_force -= vehicle.forward * drag_force * vehicle.velocity.dot(vehicle.forward).signum();

        // Apply forces to physics body
        physics.apply_force(vehicle.body_id, total_force);

        // Update engine RPM based on throttle and wheel load
        // Engine revs up with throttle input
        let throttle_rpm_target = vehicle.config.engine.idle_rpm + 
            input.throttle * (vehicle.config.engine.max_rpm - vehicle.config.engine.idle_rpm) * 0.8;
        
        if gear_ratio.abs() > 0.01 && driven_count > 0.0 {
            let avg_wheel_rpm: f32 = vehicle
                .wheels
                .iter()
                .enumerate()
                .filter(|(i, _)| vehicle.config.wheels[*i].driven)
                .map(|(_, w)| w.rotation_speed.abs() * 60.0 / (2.0 * std::f32::consts::PI))
                .sum::<f32>()
                / driven_count;

            let wheel_target_rpm = avg_wheel_rpm * gear_ratio.abs();
            
            // Engine RPM is influenced by both throttle and wheel feedback
            // Throttle pulls RPM up, wheel speed provides load feedback
            let load_factor = (vehicle.speed / 20.0).clamp(0.0, 0.5); // More wheel influence at speed
            let target_rpm = throttle_rpm_target * (1.0 - load_factor) + 
                wheel_target_rpm.max(throttle_rpm_target * 0.3) * load_factor;
            
            // Smooth RPM changes
            vehicle.engine_rpm = vehicle.engine_rpm * 0.85 + target_rpm * 0.15;
        } else {
            // No gear engaged - free rev
            vehicle.engine_rpm = vehicle.engine_rpm * 0.85 + throttle_rpm_target * 0.15;
        }

        // Clamp engine RPM
        vehicle.engine_rpm = vehicle.engine_rpm.clamp(
            vehicle.config.engine.idle_rpm,
            vehicle.config.engine.max_rpm,
        );
    }

    /// Remove a vehicle
    pub fn remove(&mut self, id: VehicleId) -> bool {
        if let Some(pos) = self.vehicles.iter().position(|v| v.id == id) {
            self.vehicles.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all vehicles
    pub fn vehicles(&self) -> &[Vehicle] {
        &self.vehicles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wheel_config_defaults() {
        let wheel = WheelConfig::default();
        assert!((wheel.radius - 0.35).abs() < 0.01);
        assert!(wheel.suspension_stiffness > 0.0);
    }

    #[test]
    fn test_wheel_config_presets() {
        let fl = WheelConfig::front_left(Vec3::new(-1.0, 0.0, 1.5));
        assert!(fl.steerable);
        assert!(!fl.driven);
        assert_eq!(fl.position_id, WheelPosition::FrontLeft);

        let rr = WheelConfig::rear_right(Vec3::new(1.0, 0.0, -1.5));
        assert!(!rr.steerable);
        assert!(rr.driven);
        assert_eq!(rr.position_id, WheelPosition::RearRight);
    }

    #[test]
    fn test_wheel_config_awd() {
        let wheel = WheelConfig::front_left(Vec3::ZERO).with_drive();
        assert!(wheel.driven);
    }

    #[test]
    fn test_engine_torque_curve() {
        let engine = EngineConfig::default();

        // Below idle: no torque
        assert!((engine.torque_at_rpm(500.0)).abs() < 0.01);

        // At max torque RPM: should be near max
        let torque_at_peak = engine.torque_at_rpm(engine.max_torque_rpm);
        assert!((torque_at_peak - engine.max_torque).abs() < 50.0);

        // Above max RPM: no torque
        assert!((engine.torque_at_rpm(8000.0)).abs() < 0.01);
    }

    #[test]
    fn test_transmission_gear_ratios() {
        let trans = TransmissionConfig::default();

        // Neutral
        assert!((trans.effective_ratio(0)).abs() < 0.01);

        // First gear should have highest ratio
        let first = trans.effective_ratio(1);
        let second = trans.effective_ratio(2);
        assert!(first > second);

        // Reverse should be negative
        assert!(trans.effective_ratio(-1) < 0.0);
    }

    #[test]
    fn test_friction_curve_tarmac() {
        let curve = FrictionCurve::tarmac();

        // Zero slip: zero friction
        assert!(curve.friction_at_slip(0.0).abs() < 0.01);

        // Optimal slip: peak friction
        let peak = curve.friction_at_slip(curve.optimal_slip);
        assert!(peak > 1.0, "Peak friction should exceed 1.0 on tarmac");

        // High slip: reduced friction
        let slide = curve.friction_at_slip(0.5);
        assert!(slide < peak, "Sliding friction should be less than peak");
    }

    #[test]
    fn test_friction_curve_ice() {
        let ice = FrictionCurve::ice();
        let tarmac = FrictionCurve::tarmac();

        let ice_friction = ice.friction_at_slip(ice.optimal_slip);
        let tarmac_friction = tarmac.friction_at_slip(tarmac.optimal_slip);

        assert!(ice_friction < tarmac_friction, "Ice should have less grip than tarmac");
    }

    #[test]
    fn test_vehicle_config_default() {
        let config = VehicleConfig::default();
        assert_eq!(config.wheels.len(), 4);
        assert!((config.mass - 1500.0).abs() < 0.1);
    }

    #[test]
    fn test_vehicle_creation() {
        let config = VehicleConfig::default();
        let vehicle = Vehicle::new(1, 42, config);

        assert_eq!(vehicle.id, 1);
        assert_eq!(vehicle.body_id, 42);
        assert_eq!(vehicle.wheels.len(), 4);
        assert_eq!(vehicle.current_gear, 1);
    }

    #[test]
    fn test_vehicle_speed_conversion() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);
        vehicle.speed = 27.78; // ~100 km/h

        let kmh = vehicle.speed_kmh();
        assert!((kmh - 100.0).abs() < 1.0);

        let mph = vehicle.speed_mph();
        assert!((mph - 62.1).abs() < 1.0);
    }

    #[test]
    fn test_vehicle_shifting() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        assert_eq!(vehicle.current_gear, 1);

        vehicle.shift_up();
        assert_eq!(vehicle.current_gear, 2);
        assert!(vehicle.is_shifting());

        // Can't shift while already shifting
        vehicle.shift_up();
        assert_eq!(vehicle.current_gear, 2);

        // Clear shift timer
        vehicle.shift_timer = 0.0;

        vehicle.shift_down();
        assert_eq!(vehicle.current_gear, 1);
    }

    #[test]
    fn test_vehicle_grounded_wheels() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Initially no wheels grounded
        assert_eq!(vehicle.grounded_wheels(), 0);
        assert!(vehicle.is_airborne());

        // Ground some wheels
        vehicle.wheels[0].grounded = true;
        vehicle.wheels[1].grounded = true;
        assert_eq!(vehicle.grounded_wheels(), 2);
        assert!(!vehicle.is_airborne());
    }

    #[test]
    fn test_vehicle_manager_creation() {
        let manager = VehicleManager::new();
        assert_eq!(manager.vehicles().len(), 0);
    }

    #[test]
    fn test_vehicle_input_default() {
        let input = VehicleInput::default();
        assert!((input.throttle).abs() < 0.01);
        assert!((input.brake).abs() < 0.01);
        assert!((input.steering).abs() < 0.01);
    }

    #[test]
    fn test_wheel_state_default() {
        let state = WheelState::default();
        assert!(!state.grounded);
        assert!((state.compression).abs() < 0.01);
        assert!((state.rotation_speed).abs() < 0.01);
    }

    #[test]
    fn test_drivetrain_types() {
        assert_eq!(DrivetrainType::default(), DrivetrainType::RWD);
    }

    #[test]
    fn test_suspension_force_calculation() {
        let wheel = WheelConfig::default();
        let compression = 0.05; // 5cm compressed

        let spring_force = compression * wheel.suspension_stiffness;
        assert!(spring_force > 0.0);
        assert!(spring_force < wheel.suspension_stiffness); // Sanity check
    }

    #[test]
    fn test_vehicle_orientation() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Default orientation
        vehicle.update_orientation(Quat::IDENTITY);
        assert!((vehicle.forward - Vec3::Z).length() < 0.01);
        assert!((vehicle.right - Vec3::X).length() < 0.01);
        assert!((vehicle.up - Vec3::Y).length() < 0.01);

        // Rotated 90 degrees around Y
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        vehicle.update_orientation(rotation);
        assert!((vehicle.forward - Vec3::X).length() < 0.1);
    }

    #[test]
    fn test_total_suspension_force() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Set suspension forces on wheels
        vehicle.wheels[0].suspension_force = 1000.0;
        vehicle.wheels[1].suspension_force = 1000.0;
        vehicle.wheels[2].suspension_force = 800.0;
        vehicle.wheels[3].suspension_force = 800.0;

        let total = vehicle.total_suspension_force();
        assert!((total - 3600.0).abs() < 0.01);
    }

    #[test]
    fn test_average_slip_ratio_all_grounded() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Ground all wheels and set slip ratios
        for (i, wheel) in vehicle.wheels.iter_mut().enumerate() {
            wheel.grounded = true;
            wheel.slip_ratio = (i as f32 + 1.0) * 0.1; // 0.1, 0.2, 0.3, 0.4
        }

        let avg = vehicle.average_slip_ratio();
        // Average of 0.1 + 0.2 + 0.3 + 0.4 = 1.0 / 4 = 0.25
        assert!((avg - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_average_slip_ratio_some_airborne() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Only ground front wheels
        vehicle.wheels[0].grounded = true;
        vehicle.wheels[0].slip_ratio = 0.2;
        vehicle.wheels[1].grounded = true;
        vehicle.wheels[1].slip_ratio = 0.4;
        vehicle.wheels[2].grounded = false;
        vehicle.wheels[3].grounded = false;

        let avg = vehicle.average_slip_ratio();
        // Average of 0.2 + 0.4 = 0.6 / 2 = 0.3
        assert!((avg - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_average_slip_ratio_airborne() {
        let config = VehicleConfig::default();
        let vehicle = Vehicle::new(1, 42, config);

        // All wheels airborne by default
        let avg = vehicle.average_slip_ratio();
        assert!((avg).abs() < 0.01);
    }

    #[test]
    fn test_average_slip_angle() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Ground all wheels and set slip angles
        for (i, wheel) in vehicle.wheels.iter_mut().enumerate() {
            wheel.grounded = true;
            wheel.slip_angle = (i as f32 + 1.0) * 0.05; // 0.05, 0.10, 0.15, 0.20
        }

        let avg = vehicle.average_slip_angle();
        // Average of 0.05 + 0.10 + 0.15 + 0.20 = 0.50 / 4 = 0.125
        assert!((avg - 0.125).abs() < 0.01);
    }

    #[test]
    fn test_friction_curve_gravel() {
        let curve = FrictionCurve::gravel();
        
        assert!((curve.optimal_slip - 0.15).abs() < 0.01);
        assert!(curve.peak_friction < FrictionCurve::tarmac().peak_friction);
    }

    #[test]
    fn test_friction_curve_mud() {
        let curve = FrictionCurve::mud();
        
        assert!((curve.optimal_slip - 0.2).abs() < 0.01);
        assert!(curve.peak_friction < FrictionCurve::gravel().peak_friction);
    }

    #[test]
    fn test_friction_curve_defaults() {
        let curve = FrictionCurve::default();
        
        assert!(curve.optimal_slip > 0.0);
        assert!(curve.peak_friction > 0.0);
        assert!(curve.sliding_friction > 0.0);
        assert!(curve.stiffness > 0.0);
    }

    #[test]
    fn test_friction_rising_portion() {
        let curve = FrictionCurve::tarmac();
        
        // Below optimal slip, friction should be increasing
        let f1 = curve.friction_at_slip(0.02);
        let f2 = curve.friction_at_slip(0.05);
        assert!(f2 > f1);
    }

    #[test]
    fn test_friction_falling_portion() {
        let curve = FrictionCurve::tarmac();
        
        // Well above optimal slip, friction should be lower than peak
        let peak = curve.friction_at_slip(curve.optimal_slip);
        let high_slip = curve.friction_at_slip(0.5);
        assert!(high_slip < peak);
    }

    #[test]
    fn test_wheel_config_with_radius() {
        let wheel = WheelConfig::default().with_radius(0.5);
        assert!((wheel.radius - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_wheel_config_with_suspension() {
        let wheel = WheelConfig::default().with_suspension(40000.0, 5000.0, 0.35);
        assert!((wheel.suspension_stiffness - 40000.0).abs() < 0.01);
        assert!((wheel.suspension_damping - 5000.0).abs() < 0.01);
        assert!((wheel.suspension_rest_length - 0.35).abs() < 0.01);
    }

    #[test]
    fn test_wheel_position_custom() {
        let wheel = WheelConfig {
            position_id: WheelPosition::Custom(5),
            ..Default::default()
        };
        assert_eq!(wheel.position_id, WheelPosition::Custom(5));
    }

    #[test]
    fn test_vehicle_config_mass() {
        let config = VehicleConfig::default();
        assert!(config.mass > 0.0);
        assert!((config.mass - 1500.0).abs() < 0.1);
    }

    #[test]
    fn test_vehicle_config_drag() {
        let config = VehicleConfig::default();
        assert!(config.drag_coefficient > 0.0);
        assert!(config.frontal_area > 0.0);
    }

    #[test]
    fn test_vehicle_config_brake_force() {
        let config = VehicleConfig::default();
        assert!(config.brake_force > 0.0);
        assert!(config.handbrake_multiplier > 1.0);
    }

    #[test]
    fn test_transmission_num_gears() {
        let trans = TransmissionConfig::default();
        assert_eq!(trans.num_gears(), 6);
    }

    #[test]
    fn test_engine_idle_torque() {
        let engine = EngineConfig::default();
        let torque = engine.torque_at_rpm(engine.idle_rpm);
        assert!(torque >= 0.0);
    }

    #[test]
    fn test_vehicle_manager_get_nonexistent() {
        let manager = VehicleManager::new();
        assert!(manager.get(999).is_none());
    }

    #[test]
    fn test_vehicle_shift_to_neutral() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Start in 1st, shift to neutral (0)
        assert_eq!(vehicle.current_gear, 1);
        vehicle.shift_down();
        vehicle.shift_timer = 0.0; // Clear shift timer
        assert_eq!(vehicle.current_gear, 0);
    }

    #[test]
    fn test_vehicle_shift_to_reverse() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Shift from 1st to neutral to reverse
        vehicle.shift_down();
        vehicle.shift_timer = 0.0;
        vehicle.shift_down();
        vehicle.shift_timer = 0.0;
        
        assert_eq!(vehicle.current_gear, -1);
    }

    #[test]
    fn test_vehicle_shift_down_limit() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Shift all the way down
        for _ in 0..5 {
            vehicle.shift_down();
            vehicle.shift_timer = 0.0;
        }

        // Should not go below -1
        assert!(vehicle.current_gear >= -1);
    }

    #[test]
    fn test_vehicle_shift_up_limit() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);

        // Shift all the way up
        for _ in 0..10 {
            vehicle.shift_up();
            vehicle.shift_timer = 0.0;
        }

        // Should not exceed max gears
        let max_gear = vehicle.config.transmission.num_gears() as i32;
        assert!(vehicle.current_gear <= max_gear);
    }

    #[test]
    fn test_vehicle_input_creation() {
        let input = VehicleInput {
            throttle: 0.8,
            brake: 0.0,
            steering: -0.5,
            handbrake: 0.0,
            clutch: 0.0,
            shift: 1,
        };

        assert!((input.throttle - 0.8).abs() < 0.01);
        assert!((input.steering - -0.5).abs() < 0.01);
        assert_eq!(input.shift, 1);
    }

    #[test]
    fn test_wheel_state_contact_normal() {
        let mut state = WheelState::default();
        state.contact_normal = Vec3::new(0.0, 1.0, 0.0);
        
        assert!((state.contact_normal.y - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_vehicle_manager_default() {
        let manager = VehicleManager::default();
        assert!(manager.vehicles().is_empty());
    }

    #[test]
    fn test_vehicle_config_center_of_mass() {
        let config = VehicleConfig::default();
        // Center of mass should be slightly below geometric center
        assert!(config.center_of_mass_offset.y < 0.0);
    }

    #[test]
    fn test_vehicle_config_steering_angle() {
        let config = VehicleConfig::default();
        // Max steering angle should be reasonable (e.g., 30-45 degrees)
        assert!(config.max_steering_angle > 0.4);
        assert!(config.max_steering_angle < 1.0);
    }

    #[test]
    fn test_vehicle_manager_spawn_and_get() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut manager = VehicleManager::new();
        let config = VehicleConfig::default();
        
        let id = manager.spawn(&mut physics, Vec3::ZERO, config);
        assert_eq!(manager.vehicles().len(), 1);
        assert!(manager.get(id).is_some());
        assert!(manager.get_mut(id).is_some());
    }

    #[test]
    fn test_vehicle_orientation_update() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);
        
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        vehicle.update_orientation(rotation);
        
        // Forward should now be along X axis
        assert!(vehicle.forward.x > 0.9);
    }

    #[test]
    fn test_vehicle_slip_averages() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);
        
        // Airborne case
        assert_eq!(vehicle.average_slip_ratio(), 0.0);
        assert_eq!(vehicle.average_slip_angle(), 0.0);
        
        // Grounded case
        vehicle.wheels[0].grounded = true;
        vehicle.wheels[0].slip_ratio = 0.5;
        vehicle.wheels[0].slip_angle = 0.1;
        
        assert!((vehicle.average_slip_ratio() - 0.5).abs() < 0.01);
        assert!((vehicle.average_slip_angle() - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_engine_torque_falling_curve() {
        let engine = EngineConfig {
            max_torque: 400.0,
            max_torque_rpm: 4000.0,
            max_rpm: 6000.0,
            idle_rpm: 1000.0,
            ..Default::default()
        };
        
        // In falling portion (between 4000 and 6000)
        let torque = engine.torque_at_rpm(5000.0);
        assert!(torque > 0.0 && torque < 400.0);
    }

    #[test]
    fn test_vehicle_manager_update_logic() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut manager = VehicleManager::new();
        let id = manager.spawn(&mut physics, Vec3::ZERO, VehicleConfig::default());
        
        // Update without input
        manager.update(&mut physics, 0.016);
        
        // Update with input
        let input = VehicleInput {
            throttle: 1.0,
            steering: 0.5,
            shift: 1,
            ..Default::default()
        };
        manager.update_with_input(id, &mut physics, &input, 0.016);
        
        let vehicle = manager.get(id).unwrap();
        assert_eq!(vehicle.current_gear, 2);
    }

    #[test]
    fn test_vehicle_suspension_force() {
        let config = VehicleConfig::default();
        let mut vehicle = Vehicle::new(1, 42, config);
        vehicle.wheels[0].suspension_force = 1000.0;
        assert_eq!(vehicle.total_suspension_force(), 1000.0);
    }

    #[test]
    fn test_friction_curves_all() {
        let _ = FrictionCurve::gravel();
        let _ = FrictionCurve::mud();
        let _ = FrictionCurve::default();
    }

    #[test]
    fn test_wheel_config_setters() {
        let wheel = WheelConfig::default()
            .with_radius(0.5)
            .with_suspension(40000.0, 5000.0, 0.4);
        assert_eq!(wheel.radius, 0.5);
        assert_eq!(wheel.suspension_stiffness, 40000.0);
    }
}
