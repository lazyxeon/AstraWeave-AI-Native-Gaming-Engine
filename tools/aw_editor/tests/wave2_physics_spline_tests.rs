//! Wave 2 Mutation Remediation — Physics + Spline Editor panels
//!
//! Targets: physics_panel.rs (2,291 lines) + spline_editor_panel.rs (2,267 lines)
//! Focus: enum Display/name/icon, Default values, numeric returns, is_* helpers

use aw_editor_lib::panels::physics_panel::{
    ClothQuality, DestructionPattern, GravityZoneShape, PhysicsAction, PhysicsStats, PhysicsTab,
    PhysicsVisualization, RagdollPreset, SimulationMode, VehicleType,
};
use aw_editor_lib::panels::spline_editor_panel::{
    CustomProfile, MeshProfile, Spline, SplinePoint, SplinePreset, SplineTab, SplineTool,
    SplineType, TangentMode, UvMode,
};

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICS VISUALIZATION — 8 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn physics_viz_default_is_none() {
    assert_eq!(PhysicsVisualization::default(), PhysicsVisualization::None);
}

#[test]
fn physics_viz_all_count() {
    assert_eq!(PhysicsVisualization::all().len(), 8);
}

#[test]
fn physics_viz_names() {
    assert_eq!(PhysicsVisualization::None.name(), "None");
    assert_eq!(PhysicsVisualization::Colliders.name(), "Colliders");
    assert_eq!(PhysicsVisualization::ContactPoints.name(), "Contact Points");
    assert_eq!(PhysicsVisualization::Joints.name(), "Joints");
    assert_eq!(
        PhysicsVisualization::VelocityVectors.name(),
        "Velocity Vectors"
    );
    assert_eq!(PhysicsVisualization::BroadPhase.name(), "Broad Phase");
    assert_eq!(PhysicsVisualization::Islands.name(), "Islands");
    assert_eq!(PhysicsVisualization::All.name(), "All");
}

#[test]
fn physics_viz_icons_nonempty() {
    for v in PhysicsVisualization::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn physics_viz_descriptions_nonempty() {
    for v in PhysicsVisualization::all() {
        assert!(!v.description().is_empty());
    }
}

#[test]
fn physics_viz_is_visible() {
    assert!(!PhysicsVisualization::None.is_visible());
    assert!(PhysicsVisualization::Colliders.is_visible());
    assert!(PhysicsVisualization::ContactPoints.is_visible());
    assert!(PhysicsVisualization::Joints.is_visible());
    assert!(PhysicsVisualization::VelocityVectors.is_visible());
    assert!(PhysicsVisualization::BroadPhase.is_visible());
    assert!(PhysicsVisualization::Islands.is_visible());
    assert!(PhysicsVisualization::All.is_visible());
}

#[test]
fn physics_viz_display_contains_name() {
    for v in PhysicsVisualization::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIMULATION MODE — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn simulation_mode_default_is_running() {
    assert_eq!(SimulationMode::default(), SimulationMode::Running);
}

#[test]
fn simulation_mode_all_count() {
    assert_eq!(SimulationMode::all().len(), 4);
}

#[test]
fn simulation_mode_names() {
    assert_eq!(SimulationMode::Running.name(), "Running");
    assert_eq!(SimulationMode::Paused.name(), "Paused");
    assert_eq!(SimulationMode::StepOnce.name(), "Step Once");
    assert_eq!(SimulationMode::SlowMotion.name(), "Slow Motion");
}

#[test]
fn simulation_mode_icons_nonempty() {
    for v in SimulationMode::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn simulation_mode_is_active() {
    assert!(SimulationMode::Running.is_active());
    assert!(!SimulationMode::Paused.is_active());
    assert!(!SimulationMode::StepOnce.is_active());
    assert!(SimulationMode::SlowMotion.is_active());
}

#[test]
fn simulation_mode_time_scale() {
    assert!((SimulationMode::Running.time_scale() - 1.0).abs() < f32::EPSILON);
    assert!((SimulationMode::Paused.time_scale() - 0.0).abs() < f32::EPSILON);
    assert!((SimulationMode::StepOnce.time_scale() - 0.0).abs() < f32::EPSILON);
    assert!((SimulationMode::SlowMotion.time_scale() - 0.2).abs() < f32::EPSILON);
}

#[test]
fn simulation_mode_display_contains_name() {
    for v in SimulationMode::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RAGDOLL PRESET — 5 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ragdoll_preset_default_is_humanoid() {
    assert_eq!(RagdollPreset::default(), RagdollPreset::Humanoid);
}

#[test]
fn ragdoll_preset_all_count() {
    assert_eq!(RagdollPreset::all().len(), 5);
}

#[test]
fn ragdoll_preset_names() {
    assert_eq!(RagdollPreset::Humanoid.name(), "Humanoid");
    assert_eq!(RagdollPreset::Quadruped.name(), "Quadruped");
    assert_eq!(RagdollPreset::Bird.name(), "Bird");
    assert_eq!(RagdollPreset::Insect.name(), "Insect");
    assert_eq!(RagdollPreset::Custom.name(), "Custom");
}

#[test]
fn ragdoll_preset_bone_count() {
    assert_eq!(RagdollPreset::Humanoid.bone_count(), 15);
    assert_eq!(RagdollPreset::Quadruped.bone_count(), 18);
    assert_eq!(RagdollPreset::Bird.bone_count(), 12);
    assert_eq!(RagdollPreset::Insect.bone_count(), 8);
    assert_eq!(RagdollPreset::Custom.bone_count(), 0);
}

#[test]
fn ragdoll_preset_display_contains_name() {
    for v in RagdollPreset::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// VEHICLE TYPE — 6 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn vehicle_type_default_is_sedan() {
    assert_eq!(VehicleType::default(), VehicleType::Sedan);
}

#[test]
fn vehicle_type_all_count() {
    assert_eq!(VehicleType::all().len(), 6);
}

#[test]
fn vehicle_type_names() {
    assert_eq!(VehicleType::Sedan.name(), "Sedan");
    assert_eq!(VehicleType::SportsCar.name(), "Sports Car");
    assert_eq!(VehicleType::Truck.name(), "Truck");
    assert_eq!(VehicleType::Motorcycle.name(), "Motorcycle");
    assert_eq!(VehicleType::Tank.name(), "Tank");
    assert_eq!(VehicleType::Custom.name(), "Custom");
}

#[test]
fn vehicle_type_wheel_count() {
    assert_eq!(VehicleType::Sedan.wheel_count(), 4);
    assert_eq!(VehicleType::SportsCar.wheel_count(), 4);
    assert_eq!(VehicleType::Truck.wheel_count(), 6);
    assert_eq!(VehicleType::Motorcycle.wheel_count(), 2);
    assert_eq!(VehicleType::Tank.wheel_count(), 0);
    assert_eq!(VehicleType::Custom.wheel_count(), 4);
}

#[test]
fn vehicle_type_display_contains_name() {
    for v in VehicleType::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CLOTH QUALITY — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cloth_quality_default_is_medium() {
    assert_eq!(ClothQuality::default(), ClothQuality::Medium);
}

#[test]
fn cloth_quality_all_count() {
    assert_eq!(ClothQuality::all().len(), 4);
}

#[test]
fn cloth_quality_names() {
    assert_eq!(ClothQuality::Low.name(), "Low");
    assert_eq!(ClothQuality::Medium.name(), "Medium");
    assert_eq!(ClothQuality::High.name(), "High");
    assert_eq!(ClothQuality::Ultra.name(), "Ultra");
}

#[test]
fn cloth_quality_iterations() {
    assert_eq!(ClothQuality::Low.iterations(), 4);
    assert_eq!(ClothQuality::Medium.iterations(), 8);
    assert_eq!(ClothQuality::High.iterations(), 16);
    assert_eq!(ClothQuality::Ultra.iterations(), 32);
}

#[test]
fn cloth_quality_cpu_cost() {
    assert_eq!(ClothQuality::Low.cpu_cost(), 1);
    assert_eq!(ClothQuality::Medium.cpu_cost(), 2);
    assert_eq!(ClothQuality::High.cpu_cost(), 4);
    assert_eq!(ClothQuality::Ultra.cpu_cost(), 8);
}

#[test]
fn cloth_quality_iterations_monotonically_increase() {
    let vals: Vec<u32> = ClothQuality::all().iter().map(|q| q.iterations()).collect();
    for w in vals.windows(2) {
        assert!(w[0] < w[1]);
    }
}

#[test]
fn cloth_quality_display_matches_name() {
    for v in ClothQuality::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DESTRUCTION PATTERN — 5 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn destruction_pattern_default_is_voronoi() {
    assert_eq!(DestructionPattern::default(), DestructionPattern::Voronoi);
}

#[test]
fn destruction_pattern_all_count() {
    assert_eq!(DestructionPattern::all().len(), 5);
}

#[test]
fn destruction_pattern_names() {
    assert_eq!(DestructionPattern::Voronoi.name(), "Voronoi");
    assert_eq!(DestructionPattern::Radial.name(), "Radial");
    assert_eq!(DestructionPattern::Slice.name(), "Slice");
    assert_eq!(DestructionPattern::Shatter.name(), "Shatter");
    assert_eq!(DestructionPattern::Crumble.name(), "Crumble");
}

#[test]
fn destruction_pattern_fragment_count_range() {
    let (lo, hi) = DestructionPattern::Voronoi.fragment_count_range();
    assert_eq!(lo, 10);
    assert_eq!(hi, 30);

    let (lo, hi) = DestructionPattern::Radial.fragment_count_range();
    assert_eq!(lo, 8);
    assert_eq!(hi, 16);

    let (lo, hi) = DestructionPattern::Slice.fragment_count_range();
    assert_eq!(lo, 2);
    assert_eq!(hi, 4);

    let (lo, hi) = DestructionPattern::Shatter.fragment_count_range();
    assert_eq!(lo, 20);
    assert_eq!(hi, 50);

    let (lo, hi) = DestructionPattern::Crumble.fragment_count_range();
    assert_eq!(lo, 30);
    assert_eq!(hi, 100);
}

#[test]
fn destruction_pattern_range_lo_less_than_hi() {
    for v in DestructionPattern::all() {
        let (lo, hi) = v.fragment_count_range();
        assert!(lo < hi, "lo >= hi for {:?}: ({}, {})", v, lo, hi);
    }
}

#[test]
fn destruction_pattern_display_contains_name() {
    for v in DestructionPattern::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GRAVITY ZONE SHAPE — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn gravity_zone_shape_default_is_box() {
    assert_eq!(GravityZoneShape::default(), GravityZoneShape::Box);
}

#[test]
fn gravity_zone_shape_all_count() {
    assert_eq!(GravityZoneShape::all().len(), 4);
}

#[test]
fn gravity_zone_shape_names() {
    assert_eq!(GravityZoneShape::Box.name(), "Box");
    assert_eq!(GravityZoneShape::Sphere.name(), "Sphere");
    assert_eq!(GravityZoneShape::Cylinder.name(), "Cylinder");
    assert_eq!(GravityZoneShape::Global.name(), "Global");
}

#[test]
fn gravity_zone_shape_is_volumetric() {
    assert!(GravityZoneShape::Box.is_volumetric());
    assert!(GravityZoneShape::Sphere.is_volumetric());
    assert!(GravityZoneShape::Cylinder.is_volumetric());
    assert!(!GravityZoneShape::Global.is_volumetric());
}

#[test]
fn gravity_zone_shape_display_contains_name() {
    for v in GravityZoneShape::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICS STATS — Default
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn physics_stats_default_all_zero() {
    let s = PhysicsStats::default();
    assert_eq!(s.rigid_body_count, 0);
    assert_eq!(s.collider_count, 0);
    assert_eq!(s.joint_count, 0);
    assert_eq!(s.contact_pair_count, 0);
    assert_eq!(s.active_body_count, 0);
    assert_eq!(s.island_count, 0);
    assert!((s.step_time_ms - 0.0).abs() < f32::EPSILON);
    assert!((s.broad_phase_time_ms - 0.0).abs() < f32::EPSILON);
    assert!((s.narrow_phase_time_ms - 0.0).abs() < f32::EPSILON);
    assert!((s.solver_time_ms - 0.0).abs() < f32::EPSILON);
    assert!((s.ccd_time_ms - 0.0).abs() < f32::EPSILON);
    assert_eq!(s.ragdoll_count, 0);
    assert_eq!(s.vehicle_count, 0);
    assert_eq!(s.cloth_count, 0);
    assert_eq!(s.destructible_count, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICS TAB — 7 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn physics_tab_default_is_overview() {
    assert_eq!(PhysicsTab::default(), PhysicsTab::Overview);
}

#[test]
fn physics_tab_all_count() {
    assert_eq!(PhysicsTab::all().len(), 7);
}

#[test]
fn physics_tab_names() {
    assert_eq!(PhysicsTab::Overview.name(), "Overview");
    assert_eq!(PhysicsTab::Visualization.name(), "Visualization");
    assert_eq!(PhysicsTab::Ragdoll.name(), "Ragdoll");
    assert_eq!(PhysicsTab::Vehicle.name(), "Vehicle");
    assert_eq!(PhysicsTab::Cloth.name(), "Cloth");
    assert_eq!(PhysicsTab::Destruction.name(), "Destruction");
    assert_eq!(PhysicsTab::Environment.name(), "Environment");
}

#[test]
fn physics_tab_display_contains_name() {
    for v in PhysicsTab::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PHYSICS ACTION — Display, name, is_* classification
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn physics_action_play_name() {
    assert_eq!(PhysicsAction::Play.name(), "Play");
}

#[test]
fn physics_action_pause_name() {
    assert_eq!(PhysicsAction::Pause.name(), "Pause");
}

#[test]
fn physics_action_step_name() {
    assert_eq!(PhysicsAction::Step.name(), "Step");
}

#[test]
fn physics_action_reset_name() {
    assert_eq!(PhysicsAction::Reset.name(), "Reset");
}

#[test]
fn physics_action_is_simulation_control() {
    assert!(PhysicsAction::Play.is_simulation_control());
    assert!(PhysicsAction::Pause.is_simulation_control());
    assert!(PhysicsAction::Step.is_simulation_control());
    assert!(PhysicsAction::Reset.is_simulation_control());
    assert!(PhysicsAction::SetTimeScale(1.0).is_simulation_control());
    assert!(PhysicsAction::SetSimulationMode(SimulationMode::Running).is_simulation_control());
    assert!(!PhysicsAction::SpawnTestRagdoll.is_simulation_control());
    assert!(!PhysicsAction::ToggleWind(true).is_simulation_control());
}

#[test]
fn physics_action_is_visualization() {
    assert!(PhysicsAction::SetVisualization(PhysicsVisualization::Colliders).is_visualization());
    assert!(!PhysicsAction::Play.is_visualization());
    assert!(!PhysicsAction::SetVehicleType(VehicleType::Sedan).is_visualization());
}

#[test]
fn physics_action_is_system_config() {
    assert!(PhysicsAction::ApplyRagdollPreset(RagdollPreset::Humanoid).is_system_config());
    assert!(PhysicsAction::SetVehicleType(VehicleType::Truck).is_system_config());
    assert!(PhysicsAction::SetClothQuality(ClothQuality::High).is_system_config());
    assert!(PhysicsAction::SetDestructionPattern(DestructionPattern::Shatter).is_system_config());
    assert!(!PhysicsAction::Play.is_system_config());
    assert!(!PhysicsAction::ToggleWind(false).is_system_config());
}

#[test]
fn physics_action_is_environment() {
    assert!(PhysicsAction::AddGravityZone {
        name: "test".to_string()
    }
    .is_environment());
    assert!(PhysicsAction::RemoveGravityZone { index: 0 }.is_environment());
    assert!(PhysicsAction::ToggleWind(true).is_environment());
    assert!(!PhysicsAction::Play.is_environment());
    assert!(!PhysicsAction::SetClothQuality(ClothQuality::Low).is_environment());
}

#[test]
fn physics_action_display_nonempty() {
    let actions: Vec<PhysicsAction> = vec![
        PhysicsAction::Play,
        PhysicsAction::Pause,
        PhysicsAction::Step,
        PhysicsAction::Reset,
        PhysicsAction::SetTimeScale(2.0),
        PhysicsAction::SpawnTestRagdoll,
        PhysicsAction::ApplyTestImpulse { strength: 10.0 },
        PhysicsAction::ToggleWind(true),
    ];
    for a in &actions {
        assert!(!format!("{a}").is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SPLINE TYPE — 5 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn spline_type_default_is_catmull_rom() {
    assert_eq!(SplineType::default(), SplineType::CatmullRom);
}

#[test]
fn spline_type_all_count() {
    assert_eq!(SplineType::all().len(), 5);
}

#[test]
fn spline_type_names() {
    assert_eq!(SplineType::CatmullRom.name(), "Catmull-Rom");
    assert_eq!(SplineType::Bezier.name(), "Bezier");
    assert_eq!(SplineType::Linear.name(), "Linear");
    assert_eq!(SplineType::Hermite.name(), "Hermite");
    assert_eq!(SplineType::BSpline.name(), "B-Spline");
}

#[test]
fn spline_type_is_smooth() {
    assert!(SplineType::CatmullRom.is_smooth());
    assert!(SplineType::Bezier.is_smooth());
    assert!(!SplineType::Linear.is_smooth());
    assert!(SplineType::Hermite.is_smooth());
    assert!(SplineType::BSpline.is_smooth());
}

#[test]
fn spline_type_display_matches_name() {
    for v in SplineType::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SPLINE PRESET — 8 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn spline_preset_default_is_custom() {
    assert_eq!(SplinePreset::default(), SplinePreset::Custom);
}

#[test]
fn spline_preset_all_count() {
    assert_eq!(SplinePreset::all().len(), 8);
}

#[test]
fn spline_preset_names() {
    assert_eq!(SplinePreset::Custom.name(), "Custom");
    assert_eq!(SplinePreset::Road.name(), "Road");
    assert_eq!(SplinePreset::Rail.name(), "Rail");
    assert_eq!(SplinePreset::River.name(), "River");
    assert_eq!(SplinePreset::Fence.name(), "Fence");
    assert_eq!(SplinePreset::Cable.name(), "Cable");
    assert_eq!(SplinePreset::AnimationPath.name(), "Animation Path");
    assert_eq!(SplinePreset::CameraRail.name(), "Camera Rail");
}

#[test]
fn spline_preset_is_infrastructure() {
    assert!(!SplinePreset::Custom.is_infrastructure());
    assert!(SplinePreset::Road.is_infrastructure());
    assert!(SplinePreset::Rail.is_infrastructure());
    assert!(!SplinePreset::River.is_infrastructure());
    assert!(SplinePreset::Fence.is_infrastructure());
    assert!(SplinePreset::Cable.is_infrastructure());
    assert!(!SplinePreset::AnimationPath.is_infrastructure());
    assert!(!SplinePreset::CameraRail.is_infrastructure());
}

#[test]
fn spline_preset_display_contains_name() {
    for v in SplinePreset::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TANGENT MODE — 5 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tangent_mode_default_is_auto() {
    assert_eq!(TangentMode::default(), TangentMode::Auto);
}

#[test]
fn tangent_mode_all_count() {
    assert_eq!(TangentMode::all().len(), 5);
}

#[test]
fn tangent_mode_names() {
    assert_eq!(TangentMode::Auto.name(), "Auto");
    assert_eq!(TangentMode::Smooth.name(), "Smooth");
    assert_eq!(TangentMode::Linear.name(), "Linear");
    assert_eq!(TangentMode::Free.name(), "Free");
    assert_eq!(TangentMode::Aligned.name(), "Aligned");
}

#[test]
fn tangent_mode_display_matches_name() {
    for v in TangentMode::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MESH PROFILE — 6 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mesh_profile_default_is_flat() {
    assert_eq!(MeshProfile::default(), MeshProfile::Flat);
}

#[test]
fn mesh_profile_all_count() {
    assert_eq!(MeshProfile::all().len(), 6);
}

#[test]
fn mesh_profile_names() {
    assert_eq!(MeshProfile::Flat.name(), "Flat");
    assert_eq!(MeshProfile::Round.name(), "Round");
    assert_eq!(MeshProfile::Square.name(), "Square");
    assert_eq!(MeshProfile::RoadWithSidewalk.name(), "Road with Sidewalk");
    assert_eq!(MeshProfile::RiverBed.name(), "River Bed");
    assert_eq!(MeshProfile::Custom.name(), "Custom");
}

#[test]
fn mesh_profile_display_matches_name() {
    for v in MeshProfile::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UV MODE — 3 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn uv_mode_default_is_stretch() {
    assert_eq!(UvMode::default(), UvMode::Stretch);
}

#[test]
fn uv_mode_all_count() {
    assert_eq!(UvMode::all().len(), 3);
}

#[test]
fn uv_mode_names() {
    assert_eq!(UvMode::Stretch.name(), "Stretch");
    assert_eq!(UvMode::Tile.name(), "Tile");
    assert_eq!(UvMode::TileWorld.name(), "Tile World");
}

#[test]
fn uv_mode_display_matches_name() {
    for v in UvMode::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SPLINE TOOL — 6 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn spline_tool_default_is_select() {
    assert_eq!(SplineTool::default(), SplineTool::Select);
}

#[test]
fn spline_tool_all_count() {
    assert_eq!(SplineTool::all().len(), 6);
}

#[test]
fn spline_tool_names() {
    assert_eq!(SplineTool::Select.name(), "Select");
    assert_eq!(SplineTool::AddPoint.name(), "Add Point");
    assert_eq!(SplineTool::InsertPoint.name(), "Insert Point");
    assert_eq!(SplineTool::DeletePoint.name(), "Delete Point");
    assert_eq!(SplineTool::MoveTangent.name(), "Move Tangent");
    assert_eq!(SplineTool::Draw.name(), "Draw");
}

#[test]
fn spline_tool_is_destructive() {
    assert!(!SplineTool::Select.is_destructive());
    assert!(!SplineTool::AddPoint.is_destructive());
    assert!(!SplineTool::InsertPoint.is_destructive());
    assert!(SplineTool::DeletePoint.is_destructive());
    assert!(!SplineTool::MoveTangent.is_destructive());
    assert!(!SplineTool::Draw.is_destructive());
}

#[test]
fn spline_tool_display_contains_name() {
    for v in SplineTool::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SPLINE TAB — 6 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn spline_tab_default_is_splines() {
    assert_eq!(SplineTab::default(), SplineTab::Splines);
}

#[test]
fn spline_tab_all_count() {
    assert_eq!(SplineTab::all().len(), 6);
}

#[test]
fn spline_tab_names() {
    assert_eq!(SplineTab::Splines.name(), "Splines");
    assert_eq!(SplineTab::Points.name(), "Points");
    assert_eq!(SplineTab::Mesh.name(), "Mesh");
    assert_eq!(SplineTab::Terrain.name(), "Terrain");
    assert_eq!(SplineTab::Animation.name(), "Animation");
    assert_eq!(SplineTab::Profiles.name(), "Profiles");
}

#[test]
fn spline_tab_display_contains_name() {
    for v in SplineTab::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SPLINE STRUCT DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn spline_point_default_values() {
    let p = SplinePoint::default();
    assert_eq!(p.id, 0);
    assert_eq!(p.position, [0.0, 0.0, 0.0]);
    assert_eq!(p.rotation, [0.0, 0.0, 0.0]);
    assert_eq!(p.scale, [1.0, 1.0, 1.0]);
    assert_eq!(p.in_tangent, [-1.0, 0.0, 0.0]);
    assert_eq!(p.out_tangent, [1.0, 0.0, 0.0]);
    assert_eq!(p.tangent_mode, TangentMode::Auto);
    assert!((p.roll - 0.0).abs() < f32::EPSILON);
    assert!((p.width - 1.0).abs() < f32::EPSILON);
    assert!((p.custom_data - 0.0).abs() < f32::EPSILON);
}

#[test]
fn spline_default_values() {
    let s = Spline::default();
    assert_eq!(s.id, 0);
    assert_eq!(s.name, "New Spline");
    assert_eq!(s.spline_type, SplineType::CatmullRom);
    assert_eq!(s.preset, SplinePreset::Custom);
    assert!(!s.closed);
    assert!(s.points.is_empty());
    assert_eq!(s.color, [0.2, 0.6, 1.0]);
    assert!((s.thickness - 2.0).abs() < f32::EPSILON);
    assert!(s.show_points);
    assert!(s.show_tangents);
    assert!(!s.generate_mesh);
    assert_eq!(s.mesh_profile, MeshProfile::Flat);
    assert_eq!(s.uv_mode, UvMode::Tile);
    assert!((s.segment_length - 1.0).abs() < f32::EPSILON);
    assert!(!s.deform_terrain);
    assert!((s.terrain_width - 5.0).abs() < f32::EPSILON);
    assert!((s.terrain_falloff - 2.0).abs() < f32::EPSILON);
    assert!((s.animation_duration - 10.0).abs() < f32::EPSILON);
    assert!(s.loop_animation);
    assert!((s.total_length - 0.0).abs() < f32::EPSILON);
}

#[test]
fn custom_profile_default_values() {
    let p = CustomProfile::default();
    assert_eq!(p.name, "Custom");
    assert_eq!(p.points.len(), 2);
    assert!((p.points[0].x - (-1.0)).abs() < f32::EPSILON);
    assert!((p.points[0].y - 0.0).abs() < f32::EPSILON);
    assert!((p.points[1].x - 1.0).abs() < f32::EPSILON);
    assert!((p.points[1].y - 0.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════
// CROSS-CUTTING: unique names within enums
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn physics_viz_names_unique() {
    let names: Vec<&str> = PhysicsVisualization::all()
        .iter()
        .map(|v| v.name())
        .collect();
    let set: std::collections::HashSet<&&str> = names.iter().collect();
    assert_eq!(names.len(), set.len());
}

#[test]
fn vehicle_type_names_unique() {
    let names: Vec<&str> = VehicleType::all().iter().map(|v| v.name()).collect();
    let set: std::collections::HashSet<&&str> = names.iter().collect();
    assert_eq!(names.len(), set.len());
}

#[test]
fn spline_preset_names_unique() {
    let names: Vec<&str> = SplinePreset::all().iter().map(|v| v.name()).collect();
    let set: std::collections::HashSet<&&str> = names.iter().collect();
    assert_eq!(names.len(), set.len());
}

#[test]
fn spline_tool_names_unique() {
    let names: Vec<&str> = SplineTool::all().iter().map(|v| v.name()).collect();
    let set: std::collections::HashSet<&&str> = names.iter().collect();
    assert_eq!(names.len(), set.len());
}
