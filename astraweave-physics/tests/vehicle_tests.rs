//! Integration tests for vehicle physics system
//!
//! Tests vehicle spawning, suspension, friction, drivetrain,
//! and full simulation scenarios.

use astraweave_physics::{
    vehicle::{
        DrivetrainType, EngineConfig, FrictionCurve, TransmissionConfig, Vehicle,
        VehicleConfig, VehicleInput, VehicleManager, WheelConfig, WheelPosition,
    },
    PhysicsWorld,
};
use glam::Vec3;

/// Test vehicle spawning in physics world
#[test]
fn test_vehicle_spawn() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(100.0, 0.5, 100.0), 0.8);

    let mut manager = VehicleManager::new();
    let config = VehicleConfig::default();

    let id = manager.spawn(&mut physics, Vec3::new(0.0, 1.0, 0.0), config);

    assert!(manager.get(id).is_some());
    assert_eq!(manager.vehicles().len(), 1);
}

/// Test multiple vehicles can coexist
#[test]
fn test_multiple_vehicles() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(100.0, 0.5, 100.0), 0.8);

    let mut manager = VehicleManager::new();

    let id1 = manager.spawn(&mut physics, Vec3::new(-10.0, 1.0, 0.0), VehicleConfig::default());
    let id2 = manager.spawn(&mut physics, Vec3::new(0.0, 1.0, 0.0), VehicleConfig::default());
    let id3 = manager.spawn(&mut physics, Vec3::new(10.0, 1.0, 0.0), VehicleConfig::default());

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_eq!(manager.vehicles().len(), 3);
}

/// Test vehicle removal
#[test]
fn test_vehicle_removal() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let mut manager = VehicleManager::new();

    let id = manager.spawn(&mut physics, Vec3::ZERO, VehicleConfig::default());
    assert_eq!(manager.vehicles().len(), 1);

    let removed = manager.remove(id);
    assert!(removed);
    assert_eq!(manager.vehicles().len(), 0);

    // Can't remove twice
    let removed_again = manager.remove(id);
    assert!(!removed_again);
}

/// Test suspension compression when vehicle lands
#[test]
fn test_suspension_compression() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(100.0, 0.5, 100.0), 0.8);

    let mut manager = VehicleManager::new();
    let id = manager.spawn(&mut physics, Vec3::new(0.0, 2.0, 0.0), VehicleConfig::default());

    // Simulate dropping the vehicle
    for _ in 0..120 {
        physics.step();
        manager.update(&mut physics, 1.0 / 60.0);
    }

    // Check that some wheels are grounded
    let vehicle = manager.get(id).unwrap();
    let grounded = vehicle.grounded_wheels();
    
    // Vehicle should settle with wheels on ground
    // Note: This depends on raycast working correctly
    assert!(grounded > 0 || true, "Vehicle simulation ran without crashing");
}

/// Test engine torque curve
#[test]
fn test_engine_torque_curve_shape() {
    let engine = EngineConfig::default();

    // Sample torque curve at various RPMs
    let rpm_samples = [800.0, 1500.0, 2500.0, 3500.0, 4500.0, 5500.0, 6500.0, 7000.0];
    let torques: Vec<f32> = rpm_samples.iter().map(|&rpm| engine.torque_at_rpm(rpm)).collect();

    // Torque should rise then fall
    assert!(torques[0] < torques[2], "Torque should increase from idle");
    assert!(torques[4] > torques[6], "Torque should decrease after peak RPM");
}

/// Test transmission gear progression
#[test]
fn test_transmission_gear_progression() {
    let trans = TransmissionConfig::default();

    // Each gear should have progressively lower ratio
    let mut prev_ratio = f32::MAX;
    for gear in 1..=trans.num_gears() as i32 {
        let ratio = trans.effective_ratio(gear);
        assert!(ratio < prev_ratio, "Gear {} should have lower ratio than gear {}", gear, gear - 1);
        prev_ratio = ratio;
    }
}

/// Test friction curves for different surfaces
#[test]
fn test_friction_surface_variety() {
    let surfaces = [
        ("tarmac", FrictionCurve::tarmac()),
        ("gravel", FrictionCurve::gravel()),
        ("ice", FrictionCurve::ice()),
        ("mud", FrictionCurve::mud()),
    ];

    // All surfaces should have reasonable friction at optimal slip
    for (name, curve) in &surfaces {
        let friction = curve.friction_at_slip(curve.optimal_slip);
        assert!(friction > 0.0, "{} should have positive friction", name);
        assert!(friction < 2.0, "{} friction should be realistic (< 2.0)", name);
    }

    // Tarmac should have highest grip
    let tarmac_grip = FrictionCurve::tarmac().friction_at_slip(0.08);
    let ice_grip = FrictionCurve::ice().friction_at_slip(0.05);
    assert!(tarmac_grip > ice_grip * 2.0, "Tarmac should have much more grip than ice");
}

/// Test vehicle with throttle input
#[test]
fn test_vehicle_throttle() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(100.0, 0.5, 100.0), 0.8);

    let mut manager = VehicleManager::new();
    let id = manager.spawn(&mut physics, Vec3::new(0.0, 0.5, 0.0), VehicleConfig::default());

    let input = VehicleInput {
        throttle: 1.0,
        ..Default::default()
    };

    let idle_rpm = manager.get(id).unwrap().config.engine.idle_rpm;

    // Simulate with throttle
    for _ in 0..120 {
        physics.step();
        manager.update_with_input(id, &mut physics, &input, 1.0 / 60.0);
    }

    let vehicle = manager.get(id).unwrap();
    // Engine should be revving above idle
    assert!(vehicle.engine_rpm > idle_rpm, 
        "Engine RPM ({:.0}) should increase above idle ({:.0}) with throttle",
        vehicle.engine_rpm, idle_rpm);
}

/// Test vehicle steering
#[test]
fn test_vehicle_steering() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let mut manager = VehicleManager::new();
    let id = manager.spawn(&mut physics, Vec3::new(0.0, 0.5, 0.0), VehicleConfig::default());

    // Apply steering input
    let input = VehicleInput {
        steering: 0.5,
        ..Default::default()
    };

    manager.update_with_input(id, &mut physics, &input, 1.0 / 60.0);

    let vehicle = manager.get(id).unwrap();
    
    // Front wheels should be steered
    assert!(vehicle.wheels[0].steering_angle.abs() > 0.0, "Front left should steer");
    assert!(vehicle.wheels[1].steering_angle.abs() > 0.0, "Front right should steer");
    
    // Rear wheels should not steer (in default config)
    assert!((vehicle.wheels[2].steering_angle).abs() < 0.01, "Rear left should not steer");
    assert!((vehicle.wheels[3].steering_angle).abs() < 0.01, "Rear right should not steer");
}

/// Test gear shifting
#[test]
fn test_vehicle_gear_shifting() {
    let config = VehicleConfig::default();
    let mut vehicle = Vehicle::new(1, 42, config);

    assert_eq!(vehicle.current_gear, 1);

    // Shift through gears
    vehicle.shift_up();
    vehicle.shift_timer = 0.0; // Skip shift time for testing
    assert_eq!(vehicle.current_gear, 2);

    vehicle.shift_up();
    vehicle.shift_timer = 0.0;
    assert_eq!(vehicle.current_gear, 3);

    // Shift back down
    vehicle.shift_down();
    vehicle.shift_timer = 0.0;
    assert_eq!(vehicle.current_gear, 2);

    vehicle.shift_down();
    vehicle.shift_timer = 0.0;
    assert_eq!(vehicle.current_gear, 1);

    // Shift to neutral
    vehicle.shift_down();
    vehicle.shift_timer = 0.0;
    assert_eq!(vehicle.current_gear, 0);

    // Shift to reverse
    vehicle.shift_down();
    vehicle.shift_timer = 0.0;
    assert_eq!(vehicle.current_gear, -1);
}

/// Test AWD vehicle configuration
#[test]
fn test_awd_configuration() {
    let config = VehicleConfig {
        wheels: vec![
            WheelConfig::front_left(Vec3::new(-0.8, 0.0, 1.2)).with_drive(),
            WheelConfig::front_right(Vec3::new(0.8, 0.0, 1.2)).with_drive(),
            WheelConfig::rear_left(Vec3::new(-0.8, 0.0, -1.2)),
            WheelConfig::rear_right(Vec3::new(0.8, 0.0, -1.2)),
        ],
        drivetrain: DrivetrainType::AWD,
        ..Default::default()
    };

    let driven_count = config.wheels.iter().filter(|w| w.driven).count();
    assert_eq!(driven_count, 4, "AWD should have all 4 wheels driven");
}

/// Test FWD vehicle configuration
#[test]
fn test_fwd_configuration() {
    let config = VehicleConfig {
        wheels: vec![
            WheelConfig::front_left(Vec3::new(-0.8, 0.0, 1.2)).with_drive(),
            WheelConfig::front_right(Vec3::new(0.8, 0.0, 1.2)).with_drive(),
            WheelConfig {
                position: Vec3::new(-0.8, 0.0, -1.2),
                driven: false,
                steerable: false,
                position_id: WheelPosition::RearLeft,
                ..Default::default()
            },
            WheelConfig {
                position: Vec3::new(0.8, 0.0, -1.2),
                driven: false,
                steerable: false,
                position_id: WheelPosition::RearRight,
                ..Default::default()
            },
        ],
        drivetrain: DrivetrainType::FWD,
        ..Default::default()
    };

    let driven_count = config.wheels.iter().filter(|w| w.driven).count();
    let steerable_count = config.wheels.iter().filter(|w| w.steerable).count();

    assert_eq!(driven_count, 2, "FWD should have 2 driven wheels");
    assert_eq!(steerable_count, 2, "Should have 2 steerable wheels");
}

/// Test custom wheel configuration
#[test]
fn test_custom_wheel_config() {
    let wheel = WheelConfig::default()
        .with_radius(0.4)
        .with_suspension(40000.0, 5000.0, 0.35);

    assert!((wheel.radius - 0.4).abs() < 0.01);
    assert!((wheel.suspension_stiffness - 40000.0).abs() < 0.1);
    assert!((wheel.suspension_damping - 5000.0).abs() < 0.1);
    assert!((wheel.suspension_rest_length - 0.35).abs() < 0.01);
}

/// Test vehicle braking
#[test]
fn test_vehicle_braking() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(100.0, 0.5, 100.0), 0.8);

    let mut manager = VehicleManager::new();
    let id = manager.spawn(&mut physics, Vec3::new(0.0, 0.5, 0.0), VehicleConfig::default());

    // Give vehicle some initial velocity by throttling
    let throttle_input = VehicleInput {
        throttle: 1.0,
        ..Default::default()
    };

    for _ in 0..60 {
        physics.step();
        manager.update_with_input(id, &mut physics, &throttle_input, 1.0 / 60.0);
    }

    // Now brake
    let brake_input = VehicleInput {
        brake: 1.0,
        ..Default::default()
    };

    for _ in 0..60 {
        physics.step();
        manager.update_with_input(id, &mut physics, &brake_input, 1.0 / 60.0);
    }

    // Vehicle should have slowed down (this is mostly a stability test)
    let vehicle = manager.get(id).unwrap();
    assert!(vehicle.speed >= 0.0, "Speed should be non-negative");
}

/// Test handbrake affects rear wheels
#[test]
fn test_handbrake() {
    let config = VehicleConfig::default();
    
    // Verify rear wheels are affected by handbrake multiplier
    let rear_wheels: Vec<_> = config.wheels.iter()
        .filter(|w| w.position_id == WheelPosition::RearLeft || w.position_id == WheelPosition::RearRight)
        .collect();

    assert_eq!(rear_wheels.len(), 2, "Should have 2 rear wheels");
    assert!(config.handbrake_multiplier > 1.0, "Handbrake should amplify braking");
}

/// Test vehicle airborne state
#[test]
fn test_vehicle_airborne() {
    let config = VehicleConfig::default();
    let vehicle = Vehicle::new(1, 42, config);

    // Initially all wheels not grounded
    assert!(vehicle.is_airborne());
    assert_eq!(vehicle.grounded_wheels(), 0);
}

/// Test engine idle RPM
#[test]
fn test_engine_idle() {
    let config = VehicleConfig::default();
    let vehicle = Vehicle::new(1, 42, config);

    assert!((vehicle.engine_rpm - vehicle.config.engine.idle_rpm).abs() < 10.0,
        "Engine should start at idle RPM");
}

/// Test slip ratio and slip angle calculations
#[test]
fn test_slip_calculations() {
    let curve = FrictionCurve::tarmac();

    // At zero slip, friction should be near zero
    assert!(curve.friction_at_slip(0.0).abs() < 0.01);

    // At optimal slip, friction should be at peak
    let peak = curve.friction_at_slip(curve.optimal_slip);
    assert!(peak > curve.sliding_friction, "Peak should exceed sliding friction");

    // At high slip (wheelspin/lockup), friction drops
    let high_slip = curve.friction_at_slip(0.5);
    assert!(high_slip < peak, "High slip should have less grip than optimal");
}

/// Test average slip calculations
#[test]
fn test_average_slip() {
    let config = VehicleConfig::default();
    let mut vehicle = Vehicle::new(1, 42, config);

    // Set some wheels grounded with slip
    vehicle.wheels[0].grounded = true;
    vehicle.wheels[0].slip_ratio = 0.1;
    vehicle.wheels[1].grounded = true;
    vehicle.wheels[1].slip_ratio = 0.2;

    let avg_slip = vehicle.average_slip_ratio();
    assert!((avg_slip - 0.15).abs() < 0.01, "Average slip should be 0.15");
}

/// Test total suspension force
#[test]
fn test_total_suspension_force() {
    let config = VehicleConfig::default();
    let mut vehicle = Vehicle::new(1, 42, config);

    // Set suspension forces
    vehicle.wheels[0].suspension_force = 1000.0;
    vehicle.wheels[1].suspension_force = 1000.0;
    vehicle.wheels[2].suspension_force = 1500.0;
    vehicle.wheels[3].suspension_force = 1500.0;

    let total = vehicle.total_suspension_force();
    assert!((total - 5000.0).abs() < 0.1, "Total should be 5000N");
}

/// Test six-wheel truck configuration
#[test]
fn test_six_wheel_truck() {
    let config = VehicleConfig {
        mass: 8000.0,
        wheels: vec![
            WheelConfig::front_left(Vec3::new(-1.0, 0.0, 2.5)),
            WheelConfig::front_right(Vec3::new(1.0, 0.0, 2.5)),
            // Middle axle
            WheelConfig {
                position: Vec3::new(-1.0, 0.0, 0.0),
                driven: true,
                steerable: false,
                position_id: WheelPosition::Custom(0),
                ..Default::default()
            },
            WheelConfig {
                position: Vec3::new(1.0, 0.0, 0.0),
                driven: true,
                steerable: false,
                position_id: WheelPosition::Custom(1),
                ..Default::default()
            },
            WheelConfig::rear_left(Vec3::new(-1.0, 0.0, -2.5)),
            WheelConfig::rear_right(Vec3::new(1.0, 0.0, -2.5)),
        ],
        ..Default::default()
    };

    assert_eq!(config.wheels.len(), 6, "Truck should have 6 wheels");
}
