use glam::{vec3, Mat4, Vec3};
pub use rapier3d::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "profiling")]
use astraweave_profiling::{plot, span};

// Async physics scheduler (feature-gated)
#[cfg(feature = "async-physics")]
pub mod async_scheduler;

#[cfg(feature = "async-physics")]
pub use async_scheduler::{AsyncPhysicsScheduler, PhysicsStepProfile};

// ECS integration (feature-gated)
#[cfg(feature = "ecs")]
pub mod ecs;

#[cfg(feature = "ecs")]
pub use ecs::*;

// Spatial hash grid for broad-phase collision optimization
pub mod spatial_hash;
pub use spatial_hash::{SpatialHash, SpatialHashStats, AABB};

// Projectile system for shooters and combat games
pub mod projectile;
pub use projectile::{
    ExplosionConfig, ExplosionResult, FalloffCurve, ProjectileConfig, ProjectileHit,
    ProjectileId, ProjectileKind, ProjectileManager,
};

// Gravity system for variable gravity, zones, and attractors
pub mod gravity;
pub use gravity::{
    BodyGravityId, BodyGravitySettings, GravityManager, GravityZone, GravityZoneId,
    GravityZoneShape,
};

// Ragdoll system for physics-based character animations
pub mod ragdoll;
pub use ragdoll::{
    BoneDef, BoneJointType, BoneShape, Ragdoll, RagdollBuilder, RagdollConfig, RagdollId,
    RagdollPresets, RagdollState,
};

// Vehicle physics for cars, trucks, motorcycles
pub mod vehicle;
pub use vehicle::{
    DrivetrainType, EngineConfig, FrictionCurve, TransmissionConfig, Vehicle, VehicleConfig,
    VehicleId, VehicleInput, VehicleManager, WheelConfig, WheelPosition, WheelState,
};

// Environmental physics (wind, water)
pub mod environment;
pub use environment::{
    EnvironmentManager, GustEvent, WaterVolume, WaterVolumeId, WindType, WindZone,
    WindZoneConfig, WindZoneId, WindZoneShape,
};

// Destruction system
pub mod destruction;
pub use destruction::{
    Debris, DebrisConfig, DebrisId, DebrisShape, Destructible, DestructibleConfig,
    DestructibleId, DestructibleState, DestructionEvent, DestructionManager,
    DestructionTrigger, FracturePattern,
};

// Cloth simulation
pub mod cloth;
pub use cloth::{
    Cloth, ClothCollider, ClothConfig, ClothId, ClothManager, ClothParticle,
    DistanceConstraint,
};

pub type BodyId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActorKind {
    Static,
    Dynamic,
    Character,
    Other,
}

impl ActorKind {
    /// Returns the name of the actor kind.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Dynamic => "Dynamic",
            Self::Character => "Character",
            Self::Other => "Other",
        }
    }

    /// Returns true if this is a static actor.
    #[inline]
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static)
    }

    /// Returns true if this is a dynamic actor.
    #[inline]
    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic)
    }

    /// Returns true if this is a character actor.
    #[inline]
    pub fn is_character(&self) -> bool {
        matches!(self, Self::Character)
    }

    /// Returns true if this is an other actor.
    #[inline]
    pub fn is_other(&self) -> bool {
        matches!(self, Self::Other)
    }

    /// Returns true if the actor can move (dynamic or character).
    #[inline]
    pub fn is_movable(&self) -> bool {
        matches!(self, Self::Dynamic | Self::Character)
    }

    /// Returns all actor kinds.
    pub fn all() -> [ActorKind; 4] {
        [Self::Static, Self::Dynamic, Self::Character, Self::Other]
    }
}

impl std::fmt::Display for ActorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 3],
}

impl DebugLine {
    /// Creates a new debug line.
    #[inline]
    pub fn new(start: [f32; 3], end: [f32; 3], color: [f32; 3]) -> Self {
        Self { start, end, color }
    }

    /// Creates a debug line from Vec3 points.
    #[inline]
    pub fn from_vec3(start: Vec3, end: Vec3, color: [f32; 3]) -> Self {
        Self {
            start: [start.x, start.y, start.z],
            end: [end.x, end.y, end.z],
            color,
        }
    }

    /// Returns the length of the line.
    #[inline]
    pub fn length(&self) -> f32 {
        let dx = self.end[0] - self.start[0];
        let dy = self.end[1] - self.start[1];
        let dz = self.end[2] - self.start[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Returns the squared length of the line.
    #[inline]
    pub fn length_squared(&self) -> f32 {
        let dx = self.end[0] - self.start[0];
        let dy = self.end[1] - self.start[1];
        let dz = self.end[2] - self.start[2];
        dx * dx + dy * dy + dz * dz
    }

    /// Returns the midpoint of the line.
    #[inline]
    pub fn midpoint(&self) -> [f32; 3] {
        [
            (self.start[0] + self.end[0]) * 0.5,
            (self.start[1] + self.end[1]) * 0.5,
            (self.start[2] + self.end[2]) * 0.5,
        ]
    }

    /// Returns the direction vector of the line (not normalized).
    #[inline]
    pub fn direction(&self) -> [f32; 3] {
        [
            self.end[0] - self.start[0],
            self.end[1] - self.start[1],
            self.end[2] - self.start[2],
        ]
    }

    /// Returns true if the line has zero length.
    #[inline]
    pub fn is_degenerate(&self) -> bool {
        self.length_squared() < 1e-10
    }

    /// Creates a red debug line.
    #[inline]
    pub fn red(start: [f32; 3], end: [f32; 3]) -> Self {
        Self::new(start, end, [1.0, 0.0, 0.0])
    }

    /// Creates a green debug line.
    #[inline]
    pub fn green(start: [f32; 3], end: [f32; 3]) -> Self {
        Self::new(start, end, [0.0, 1.0, 0.0])
    }

    /// Creates a blue debug line.
    #[inline]
    pub fn blue(start: [f32; 3], end: [f32; 3]) -> Self {
        Self::new(start, end, [0.0, 0.0, 1.0])
    }

    /// Creates a white debug line.
    #[inline]
    pub fn white(start: [f32; 3], end: [f32; 3]) -> Self {
        Self::new(start, end, [1.0, 1.0, 1.0])
    }
}

impl std::fmt::Display for DebugLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Line([{:.2},{:.2},{:.2}] -> [{:.2},{:.2},{:.2}], rgb=[{:.2},{:.2},{:.2}])",
            self.start[0], self.start[1], self.start[2],
            self.end[0], self.end[1], self.end[2],
            self.color[0], self.color[1], self.color[2]
        )
    }
}

struct LineCollector {
    lines: Vec<DebugLine>,
}

impl LineCollector {
    fn new() -> Self {
        Self { lines: Vec::new() }
    }
}

impl DebugRenderBackend for LineCollector {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject,
        a: rapier3d::prelude::Point<Real>,
        b: rapier3d::prelude::Point<Real>,
        color: [f32; 4],
    ) {
        self.lines.push(DebugLine {
            start: [a.x, a.y, a.z],
            end: [b.x, b.y, b.z],
            color: [color[0], color[1], color[2]],
        });
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Layers: u32 {
        const DEFAULT   = 0b00000001;
        const CHARACTER = 0b00000010;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharState {
    Grounded,
}

impl CharState {
    /// Returns the name of the state.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Grounded => "Grounded",
        }
    }

    /// Returns true if grounded.
    #[inline]
    pub fn is_grounded(&self) -> bool {
        matches!(self, Self::Grounded)
    }

    /// Returns all character states.
    pub fn all() -> [CharState; 1] {
        [Self::Grounded]
    }
}

impl std::fmt::Display for CharState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CharacterController {
    pub state: CharState,
    pub max_climb_angle_deg: f32,
    pub radius: f32,
    pub height: f32,
    pub max_step: f32,
    
    // Jump / Gravity
    pub vertical_velocity: f32,
    pub gravity_scale: f32,
    
    // Timers (State)
    pub time_since_grounded: f32,
    pub jump_buffer_timer: f32,
    
    // Config
    pub coyote_time_limit: f32,
    pub jump_buffer_limit: f32,
    
    pub pending_jump_velocity: f32,
}

impl CharacterController {
    /// Creates a new character controller with given dimensions.
    pub fn new(radius: f32, height: f32) -> Self {
        Self {
            state: CharState::Grounded,
            max_climb_angle_deg: 45.0,
            radius,
            height,
            max_step: 0.3,
            vertical_velocity: 0.0,
            gravity_scale: 1.0,
            time_since_grounded: 0.0,
            jump_buffer_timer: 0.0,
            coyote_time_limit: 0.15,
            jump_buffer_limit: 0.15,
            pending_jump_velocity: 0.0,
        }
    }

    /// Returns true if the character is grounded.
    #[inline]
    pub fn is_grounded(&self) -> bool {
        self.state.is_grounded()
    }

    /// Returns true if coyote time is still active.
    #[inline]
    pub fn has_coyote_time(&self) -> bool {
        self.time_since_grounded < self.coyote_time_limit
    }

    /// Returns true if jump is buffered.
    #[inline]
    pub fn has_buffered_jump(&self) -> bool {
        self.jump_buffer_timer > 0.0
    }

    /// Returns true if the character can jump.
    #[inline]
    pub fn can_jump(&self) -> bool {
        self.is_grounded() || self.has_coyote_time()
    }

    /// Returns the capsule volume approximation.
    #[inline]
    pub fn volume(&self) -> f32 {
        let cylinder_height = self.height - 2.0 * self.radius;
        let cylinder_volume = std::f32::consts::PI * self.radius * self.radius * cylinder_height;
        let sphere_volume = (4.0 / 3.0) * std::f32::consts::PI * self.radius.powi(3);
        cylinder_volume + sphere_volume
    }

    /// Returns the max climb angle in radians.
    #[inline]
    pub fn max_climb_angle_rad(&self) -> f32 {
        self.max_climb_angle_deg.to_radians()
    }

    /// Returns true if character is falling.
    #[inline]
    pub fn is_falling(&self) -> bool {
        self.vertical_velocity < -0.01
    }

    /// Returns true if character is rising.
    #[inline]
    pub fn is_rising(&self) -> bool {
        self.vertical_velocity > 0.01
    }

    /// Resets the controller to default state.
    pub fn reset(&mut self) {
        self.state = CharState::Grounded;
        self.vertical_velocity = 0.0;
        self.time_since_grounded = 0.0;
        self.jump_buffer_timer = 0.0;
        self.pending_jump_velocity = 0.0;
    }
}

impl std::fmt::Display for CharacterController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CharController(r={:.2}, h={:.2}, state={}, vel_y={:.2})",
            self.radius, self.height, self.state, self.vertical_velocity
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub ccd_enabled: bool,
    pub max_ccd_substeps: usize,
    pub time_step: f32,
    pub water_level: f32,
    pub fluid_density: f32,
}

impl PhysicsConfig {
    /// Creates a new physics config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a physics config with custom gravity.
    pub fn with_gravity(mut self, gravity: Vec3) -> Self {
        self.gravity = gravity;
        self
    }

    /// Enables CCD with the given substeps.
    pub fn with_ccd(mut self, substeps: usize) -> Self {
        self.ccd_enabled = true;
        self.max_ccd_substeps = substeps;
        self
    }

    /// Sets the time step (delta time).
    pub fn with_time_step(mut self, dt: f32) -> Self {
        self.time_step = dt;
        self
    }

    /// Sets the water level for buoyancy.
    pub fn with_water(mut self, level: f32, density: f32) -> Self {
        self.water_level = level;
        self.fluid_density = density;
        self
    }

    /// Returns the target FPS based on time step.
    #[inline]
    pub fn target_fps(&self) -> f32 {
        1.0 / self.time_step
    }

    /// Returns true if water simulation is enabled.
    #[inline]
    pub fn has_water(&self) -> bool {
        self.water_level.is_finite()
    }

    /// Returns the gravity magnitude.
    #[inline]
    pub fn gravity_magnitude(&self) -> f32 {
        self.gravity.length()
    }

    /// Returns true if this is Earth-like gravity.
    #[inline]
    pub fn is_earth_gravity(&self) -> bool {
        (self.gravity.y + 9.81).abs() < 0.1 && self.gravity.x.abs() < 0.01 && self.gravity.z.abs() < 0.01
    }

    /// Returns true if gravity is zero.
    #[inline]
    pub fn is_zero_gravity(&self) -> bool {
        self.gravity.length_squared() < 1e-6
    }
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ccd_enabled: false,
            max_ccd_substeps: 1,
            time_step: 1.0 / 60.0,
            water_level: f32::NEG_INFINITY,
            fluid_density: 1000.0,
        }
    }
}

impl std::fmt::Display for PhysicsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PhysicsConfig(g=[{:.2},{:.2},{:.2}], dt={:.4}, ccd={})",
            self.gravity.x, self.gravity.y, self.gravity.z,
            self.time_step, self.ccd_enabled
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JointType {
    Fixed,
    Revolute {
        axis: Vec3,
        limits: Option<(f32, f32)>,
    },
    Prismatic {
        axis: Vec3,
        limits: Option<(f32, f32)>,
    },
    Spherical,
}

impl JointType {
    /// Returns the name of the joint type.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Fixed => "Fixed",
            Self::Revolute { .. } => "Revolute",
            Self::Prismatic { .. } => "Prismatic",
            Self::Spherical => "Spherical",
        }
    }

    /// Returns true if this is a fixed joint.
    #[inline]
    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed)
    }

    /// Returns true if this is a revolute joint.
    #[inline]
    pub fn is_revolute(&self) -> bool {
        matches!(self, Self::Revolute { .. })
    }

    /// Returns true if this is a prismatic joint.
    #[inline]
    pub fn is_prismatic(&self) -> bool {
        matches!(self, Self::Prismatic { .. })
    }

    /// Returns true if this is a spherical joint.
    #[inline]
    pub fn is_spherical(&self) -> bool {
        matches!(self, Self::Spherical)
    }

    /// Returns true if the joint has limits.
    #[inline]
    pub fn has_limits(&self) -> bool {
        match self {
            Self::Revolute { limits, .. } | Self::Prismatic { limits, .. } => limits.is_some(),
            _ => false,
        }
    }

    /// Returns the axis if applicable.
    #[inline]
    pub fn axis(&self) -> Option<Vec3> {
        match self {
            Self::Revolute { axis, .. } | Self::Prismatic { axis, .. } => Some(*axis),
            _ => None,
        }
    }

    /// Returns the limits if applicable.
    #[inline]
    pub fn limits(&self) -> Option<(f32, f32)> {
        match self {
            Self::Revolute { limits, .. } | Self::Prismatic { limits, .. } => *limits,
            _ => None,
        }
    }

    /// Returns true if this is a rotational joint.
    #[inline]
    pub fn is_rotational(&self) -> bool {
        matches!(self, Self::Revolute { .. } | Self::Spherical)
    }

    /// Returns true if this is a linear joint.
    #[inline]
    pub fn is_linear(&self) -> bool {
        matches!(self, Self::Prismatic { .. })
    }

    /// Returns the degrees of freedom for this joint.
    #[inline]
    pub fn degrees_of_freedom(&self) -> u8 {
        match self {
            Self::Fixed => 0,
            Self::Revolute { .. } | Self::Prismatic { .. } => 1,
            Self::Spherical => 3,
        }
    }

    /// Creates a revolute joint around the Y axis.
    pub fn revolute_y() -> Self {
        Self::Revolute { axis: Vec3::Y, limits: None }
    }

    /// Creates a revolute joint around the X axis.
    pub fn revolute_x() -> Self {
        Self::Revolute { axis: Vec3::X, limits: None }
    }

    /// Creates a revolute joint around the Z axis.
    pub fn revolute_z() -> Self {
        Self::Revolute { axis: Vec3::Z, limits: None }
    }

    /// Creates a prismatic joint along the Y axis.
    pub fn prismatic_y() -> Self {
        Self::Prismatic { axis: Vec3::Y, limits: None }
    }
}

impl std::fmt::Display for JointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed => write!(f, "Fixed"),
            Self::Revolute { axis, limits } => {
                if let Some((min, max)) = limits {
                    write!(f, "Revolute(axis=[{:.2},{:.2},{:.2}], limits=[{:.2},{:.2}])", 
                        axis.x, axis.y, axis.z, min, max)
                } else {
                    write!(f, "Revolute(axis=[{:.2},{:.2},{:.2}])", axis.x, axis.y, axis.z)
                }
            }
            Self::Prismatic { axis, limits } => {
                if let Some((min, max)) = limits {
                    write!(f, "Prismatic(axis=[{:.2},{:.2},{:.2}], limits=[{:.2},{:.2}])", 
                        axis.x, axis.y, axis.z, min, max)
                } else {
                    write!(f, "Prismatic(axis=[{:.2},{:.2},{:.2}])", axis.x, axis.y, axis.z)
                }
            }
            Self::Spherical => write!(f, "Spherical"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct JointId(pub u64);

impl JointId {
    /// Creates a new joint ID.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw ID value.
    #[inline]
    pub const fn raw(&self) -> u64 {
        self.0
    }

    /// Returns true if this is a valid ID (non-zero).
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }

    /// Returns the invalid/null joint ID.
    #[inline]
    pub const fn invalid() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for JointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JointId({})", self.0)
    }
}

impl From<u64> for JointId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<JointId> for u64 {
    fn from(id: JointId) -> Self {
        id.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct BuoyancyData {
    pub volume: f32,
    pub drag: f32,
}

impl BuoyancyData {
    /// Creates new buoyancy data.
    #[inline]
    pub const fn new(volume: f32, drag: f32) -> Self {
        Self { volume, drag }
    }

    /// Creates buoyancy data with zero drag.
    #[inline]
    pub const fn with_volume(volume: f32) -> Self {
        Self { volume, drag: 0.0 }
    }

    /// Returns true if volume is positive.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.volume > 0.0
    }

    /// Calculates buoyancy force for the given fluid density.
    #[inline]
    pub fn buoyancy_force(&self, fluid_density: f32) -> f32 {
        self.volume * fluid_density * 9.81
    }

    /// Calculates drag force for the given velocity.
    #[inline]
    pub fn drag_force(&self, velocity: f32) -> f32 {
        0.5 * self.drag * velocity * velocity
    }
}

impl std::fmt::Display for BuoyancyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Buoyancy(vol={:.3}, drag={:.3})", self.volume, self.drag)
    }
}

pub struct PhysicsWorld {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub pipeline: PhysicsPipeline,
    pub gravity: Vector<Real>,
    pub integration: IntegrationParameters,
    pub island_mgr: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub query_pipeline: QueryPipeline,
    pub ccd: CCDSolver,
    pub event_handler: ChannelEventCollector,
    pub collision_recv: rapier3d::crossbeam::channel::Receiver<CollisionEvent>,
    pub contact_force_recv: rapier3d::crossbeam::channel::Receiver<ContactForceEvent>,
    body_ids: HashMap<RigidBodyHandle, BodyId>,
    body_kinds: HashMap<RigidBodyHandle, ActorKind>,
    next_body_id: BodyId,
    pub char_map: HashMap<BodyId, CharacterController>,
    next_joint_id: u64,
    debug_render_pipeline: DebugRenderPipeline,
    pub buoyancy_bodies: HashMap<BodyId, BuoyancyData>,
    pub water_level: f32,
    pub fluid_density: f32,
    pub wind: Vec3,

    /// Async physics scheduler (feature-gated)
    #[cfg(feature = "async-physics")]
    pub async_scheduler: Option<AsyncPhysicsScheduler>,
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3) -> Self {
        let (collision_send, collision_recv) = rapier3d::crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = rapier3d::crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            pipeline: PhysicsPipeline::new(),
            gravity: vector![gravity.x, gravity.y, gravity.z],
            integration: IntegrationParameters::default(),
            island_mgr: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            query_pipeline: QueryPipeline::new(),
            ccd: CCDSolver::new(),
            event_handler,
            collision_recv,
            contact_force_recv,
            body_ids: HashMap::new(),
            body_kinds: HashMap::new(),
            next_body_id: 1,
            char_map: HashMap::new(),
            next_joint_id: 1,
            debug_render_pipeline: DebugRenderPipeline::default(),
            buoyancy_bodies: HashMap::new(),
            water_level: f32::NEG_INFINITY,
            fluid_density: 1000.0,
            wind: Vec3::ZERO,
            #[cfg(feature = "async-physics")]
            async_scheduler: None,
        }
    }

    pub fn from_config(config: PhysicsConfig) -> Self {
        let (collision_send, collision_recv) = rapier3d::crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = rapier3d::crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        let integration = IntegrationParameters {
            dt: config.time_step,
            ..Default::default()
        };

        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            pipeline: PhysicsPipeline::new(),
            gravity: vector![config.gravity.x, config.gravity.y, config.gravity.z],
            integration,
            island_mgr: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            query_pipeline: QueryPipeline::new(),
            ccd: CCDSolver::new(),
            event_handler,
            collision_recv,
            contact_force_recv,
            body_ids: HashMap::new(),
            body_kinds: HashMap::new(),
            next_body_id: 1,
            char_map: HashMap::new(),
            next_joint_id: 1,
            debug_render_pipeline: DebugRenderPipeline::default(),
            buoyancy_bodies: HashMap::new(),
            water_level: f32::NEG_INFINITY,
            fluid_density: 1000.0,
            wind: Vec3::ZERO,
            #[cfg(feature = "async-physics")]
            async_scheduler: None,
        }
    }

    /// Enable async physics with optional thread count (0 = auto-detect)
    /// This configures Rayon's global thread pool, which Rapier3D uses for parallel solving
    #[cfg(feature = "async-physics")]
    pub fn enable_async_physics(&mut self, thread_count: usize) {
        // Configure Rayon thread pool if not already initialized
        if thread_count > 0 {
            // Try to build thread pool (may fail if already initialized, which is fine)
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build_global();
        }

        self.async_scheduler = Some(if thread_count > 0 {
            AsyncPhysicsScheduler::with_threads(thread_count)
        } else {
            AsyncPhysicsScheduler::new()
        });
    }

    /// Get last physics step profile (for telemetry)
    #[cfg(feature = "async-physics")]
    pub fn get_last_profile(&self) -> Option<PhysicsStepProfile> {
        self.async_scheduler.as_ref().map(|s| s.get_last_profile())
    }

    fn alloc_id(&mut self) -> BodyId {
        let id = self.next_body_id;
        self.next_body_id += 1;
        id
    }

    pub fn step(&mut self) {
        #[cfg(feature = "profiling")]
        span!("Physics::World::step");

        #[cfg(feature = "async-physics")]
        {
            // When async scheduler is enabled, Rapier3D automatically uses
            // Rayon's global thread pool for parallel island solving.
            // The thread count was configured when enable_async_physics was called.
            if self.async_scheduler.is_some() {
                use std::time::Instant;
                let start = Instant::now();

                self.step_internal();

                let duration = start.elapsed();

                // Update telemetry
                if let Some(scheduler) = &mut self.async_scheduler {
                    scheduler.record_step_telemetry(duration);
                }
                return;
            }
        }

        // Fallback to regular step (single-threaded)
        self.step_internal();
    }

    /// Internal physics step (shared by sync and async paths)
    /// When called with async scheduler enabled, Rapier3D uses Rayon for parallel solving
    fn step_internal(&mut self) {
        #[cfg(feature = "profiling")]
        {
            span!("Physics::Rapier::pipeline");
            plot!("Physics::collider_count", self.colliders.len() as u64);
        }

        // Apply buoyancy forces before physics step
        self.apply_buoyancy_forces();

        self.pipeline.step(
            &self.gravity,
            &self.integration,
            &mut self.island_mgr,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.multibody_joints,
            &mut self.ccd,
            Some(&mut self.query_pipeline),
            &(),
            &self.event_handler,
        );

        // CRITICAL FIX (Week 2 Day 3): Update query pipeline after physics step
        // Without this, raycasts in control_character() use stale geometry,
        // causing character controller to fail ground detection
        self.query_pipeline.update(&self.colliders);
    }

    pub fn apply_force(&mut self, id: BodyId, force: Vec3) {
        if let Some(h) = self.handle_of(id) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.add_force(vector![force.x, force.y, force.z], true);
            }
        }
    }

    pub fn apply_impulse(&mut self, id: BodyId, impulse: Vec3) {
        if let Some(h) = self.handle_of(id) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);
            }
        }
    }

    pub fn get_velocity(&self, id: BodyId) -> Option<Vec3> {
        let h = self.handle_of(id)?;
        let rb = self.bodies.get(h)?;
        let v = rb.linvel();
        Some(Vec3::new(v.x, v.y, v.z))
    }

    pub fn set_velocity(&mut self, id: BodyId, vel: Vec3) {
        if let Some(h) = self.handle_of(id) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.set_linvel(vector![vel.x, vel.y, vel.z], true);
            }
        }
    }

    pub fn create_ground_plane(&mut self, half: Vec3, friction: f32) -> BodyId {
        let rb = RigidBodyBuilder::fixed().build();
        let h = self.bodies.insert(rb);
        let shape = ColliderBuilder::cuboid(half.x, 0.1, half.z)
            .friction(friction)
            .collision_groups(InteractionGroups::new(
                Group::from_bits_truncate(Layers::DEFAULT.bits()),
                Group::ALL,
            ))
            .build();
        self.colliders
            .insert_with_parent(shape, h, &mut self.bodies);
        self.tag_body(h, ActorKind::Static)
    }

    pub fn add_static_trimesh(
        &mut self,
        vertices: &[Vec3],
        indices: &[[u32; 3]],
        groups: Layers,
    ) -> BodyId {
        let rb = RigidBodyBuilder::fixed().build();
        let h = self.bodies.insert(rb);
        let v: Vec<Point<Real>> = vertices.iter().map(|p| point![p.x, p.y, p.z]).collect();
        let i: Vec<[u32; 3]> = indices.to_vec();
        let coll = ColliderBuilder::trimesh(v, i)
            .collision_groups(InteractionGroups::new(
                Group::from_bits_truncate(groups.bits()),
                Group::ALL,
            ))
            .friction(0.9)
            .build();
        self.colliders.insert_with_parent(coll, h, &mut self.bodies);
        self.tag_body(h, ActorKind::Static)
    }

    pub fn add_dynamic_box(&mut self, pos: Vec3, half: Vec3, mass: f32, groups: Layers) -> BodyId {
        #[cfg(feature = "profiling")]
        {
            span!("Physics::RigidBody::create");
            plot!("Physics::rigid_body_count", self.bodies.len() as u64);
        }

        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![pos.x, pos.y, pos.z])
            .build();
        let h = self.bodies.insert(rb);
        let coll = ColliderBuilder::cuboid(half.x, half.y, half.z)
            .mass(mass)
            .collision_groups(InteractionGroups::new(
                Group::from_bits_truncate(groups.bits()),
                Group::ALL,
            ))
            .friction(0.8)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();
        self.colliders.insert_with_parent(coll, h, &mut self.bodies);
        self.tag_body(h, ActorKind::Dynamic)
    }

    pub fn add_character(&mut self, pos: Vec3, half: Vec3) -> BodyId {
        #[cfg(feature = "profiling")]
        {
            span!("Physics::Character::create");
            plot!("Physics::character_count", self.char_map.len() as u64);
        }

        let rb = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![pos.x, pos.y, pos.z])
            .build();
        let h = self.bodies.insert(rb);
        let coll = ColliderBuilder::capsule_y(half.y, half.x.max(half.z))
            .collision_groups(InteractionGroups::new(
                Group::from_bits_truncate(Layers::CHARACTER.bits()),
                Group::ALL,
            ))
            .friction(0.6)
            .build();
        self.colliders.insert_with_parent(coll, h, &mut self.bodies);
        let id = self.tag_body(h, ActorKind::Character);
        self.char_map.insert(
            id,
            CharacterController {
                state: CharState::Grounded,
                max_climb_angle_deg: 70.0,
                radius: half.x.max(half.z),
                height: half.y * 2.0,
                max_step: 0.4,
                vertical_velocity: 0.0,
                gravity_scale: 1.0,
                time_since_grounded: 0.0,
                jump_buffer_timer: 0.0,
                coyote_time_limit: 0.1, // 100ms
                jump_buffer_limit: 0.1, // 100ms
                pending_jump_velocity: 0.0,
            },
        );
        id
    }

    pub fn jump(&mut self, id: BodyId, height: f32) {
        if let Some(ctrl) = self.char_map.get_mut(&id) {
            ctrl.jump_buffer_timer = ctrl.jump_buffer_limit;
            let g = 9.81 * ctrl.gravity_scale;
            ctrl.pending_jump_velocity = (2.0 * g * height).sqrt();
        }
    }

    pub fn control_character(&mut self, id: BodyId, desired_move: Vec3, dt: f32, _climb: bool) {
        #[cfg(feature = "profiling")]
        span!("Physics::CharacterController::move");

        let Some(mut ctrl) = self.char_map.get(&id).copied() else {
            return;
        };
        let Some(h) = self.handle_of(id) else {
            return;
        };
        let Some(rb) = self.bodies.get(h) else {
            return;
        };
        let pos = *rb.position();
        let start = glam::Vec3::new(pos.translation.x, pos.translation.y, pos.translation.z);
        // Update timers
        ctrl.jump_buffer_timer -= dt;
        
        // Apply gravity (if not climbing)
        if !_climb {
            ctrl.vertical_velocity -= 9.81 * ctrl.gravity_scale * dt;
        } else {
            ctrl.vertical_velocity = 0.0;
        }

        // Check Jump
        let can_jump = ctrl.time_since_grounded < ctrl.coyote_time_limit;
        let wants_jump = ctrl.jump_buffer_timer > 0.0;
        
        if can_jump && wants_jump && !_climb {
            ctrl.vertical_velocity = ctrl.pending_jump_velocity;
            ctrl.time_since_grounded = ctrl.coyote_time_limit + 1.0; // Invalidate coyote
            ctrl.jump_buffer_timer = 0.0; // Consume buffer
        }

        let mut d = desired_move * dt;
        let has_horizontal_move = d.length_squared() >= 1e-6;
        let has_vertical_move = ctrl.vertical_velocity.abs() > 1e-4 || _climb;

        if !has_horizontal_move && !has_vertical_move {
            self.char_map.insert(id, ctrl);
            return;
        }

        if has_horizontal_move {
            // Basic obstacle avoidance: raycast forward; slide along hit normal
            let dir = d.normalize();
            let ray_origin = start + glam::Vec3::Y * (ctrl.height * 0.5);
            let ray = rapier3d::prelude::Ray::new(
                point![ray_origin.x, ray_origin.y, ray_origin.z],
                vector![dir.x, dir.y, dir.z],
            );
            // BUG FIX (Week 2 Day 3): Exclude character's own colliders from raycasts
            // Without this, the character detects its own capsule as an obstacle
            let filter = QueryFilter::default().exclude_rigid_body(h);
            if let Some((_, hit)) = self.query_pipeline.cast_ray_and_get_normal(
                &self.bodies,
                &self.colliders,
                &ray,
                d.length() + ctrl.radius + 0.05,
                true,
                filter,
            ) {
                // Deflect movement along tangent plane
                let n = glam::Vec3::new(hit.normal.x, hit.normal.y, hit.normal.z).normalize();
                d = d - n * d.dot(n);
            }
        }

        // Tentative horizontal move
        let mut new_pos = start + glam::Vec3::new(d.x, 0.0, d.z);

        if _climb {
            // Simple vertical climb
            new_pos.y = start.y + 2.0 * dt;
            ctrl.time_since_grounded = 0.0;
        } else {
            // Apply vertical velocity
            new_pos.y = start.y + ctrl.vertical_velocity * dt;

            // Ground check / Step / Slope
            if ctrl.vertical_velocity <= 0.0 {
                let cast_origin = new_pos + glam::Vec3::Y * (ctrl.height);
                let ray_down = rapier3d::prelude::Ray::new(
                    point![cast_origin.x, cast_origin.y, cast_origin.z],
                    vector![0.0, -1.0, 0.0],
                );
                if let Some((_, hit)) = self.query_pipeline.cast_ray_and_get_normal(
                    &self.bodies,
                    &self.colliders,
                    &ray_down,
                    ctrl.height + ctrl.max_step + 1.0,
                    true,
                    QueryFilter::default().exclude_rigid_body(h),
                ) {
                    let ground_normal =
                        glam::Vec3::new(hit.normal.x, hit.normal.y, hit.normal.z).normalize();
                    let slope = ground_normal.dot(glam::Vec3::Y).acos().to_degrees();
                    let ground_y = cast_origin.y - hit.time_of_impact;
                    
                    if slope <= ctrl.max_climb_angle_deg + 1e-2 {
                        // Snap to ground if close enough
                        if new_pos.y <= ground_y + 0.05 {
                             new_pos.y = ground_y;
                             ctrl.vertical_velocity = 0.0;
                             ctrl.time_since_grounded = 0.0;
                        }
                    }
                } else {
                    ctrl.time_since_grounded += dt;
                }
            } else {
                ctrl.time_since_grounded += dt;
            }
        }

        // Commit move
        let mut p = pos;
        p.translation.x = new_pos.x;
        p.translation.y = new_pos.y;
        p.translation.z = new_pos.z;
        if let Some(rbmut) = self.bodies.get_mut(h) {
            // BUG FIX (Week 2 Day 3): Use set_next_kinematic_position for kinematic bodies
            // set_position() with wake=true doesn't properly update kinematic bodies
            // across multiple frames - position gets reset by physics step
            rbmut.set_next_kinematic_position(p);
        }
    }

    pub fn handle_of(&self, id: BodyId) -> Option<RigidBodyHandle> {
        self.body_ids
            .iter()
            .find_map(|(h, bid)| if *bid == id { Some(*h) } else { None })
    }

    pub fn id_of(&self, handle: RigidBodyHandle) -> Option<BodyId> {
        self.body_ids.get(&handle).copied()
    }

    pub fn body_transform(&self, id: BodyId) -> Option<Mat4> {
        let h = self.handle_of(id)?;
        let rb = self.bodies.get(h)?;
        let iso = rb.position();
        let rot = glam::Quat::from_xyzw(
            iso.rotation.i,
            iso.rotation.j,
            iso.rotation.k,
            iso.rotation.w,
        );
        Some(Mat4::from_rotation_translation(
            rot,
            vec3(iso.translation.x, iso.translation.y, iso.translation.z),
        ))
    }

    fn tag_body(&mut self, h: RigidBodyHandle, kind: ActorKind) -> BodyId {
        let id = self.alloc_id();
        self.body_ids.insert(h, id);
        self.body_kinds.insert(h, kind);
        id
    }

    pub fn add_buoyancy(&mut self, body: BodyId, volume: f32, drag: f32) {
        self.buoyancy_bodies
            .insert(body, BuoyancyData { volume, drag });
    }

    fn apply_buoyancy_forces(&mut self) {
        for (body_id, buoyancy_data) in &self.buoyancy_bodies {
            if let Some(handle) = self.handle_of(*body_id) {
                if let Some(rb) = self.bodies.get_mut(handle) {
                    let pos = rb.position();
                    let body_y = pos.translation.y;

                    // Only apply buoyancy if body is below water level
                    if body_y < self.water_level {
                        // Buoyancy force = volume * fluid_density * gravity (upward)
                        let buoyancy_force = buoyancy_data.volume * self.fluid_density * 9.81;

                        // Drag force = -velocity * drag coefficient
                        let velocity = rb.linvel();
                        let drag_force = vector![
                            -velocity.x * buoyancy_data.drag,
                            -velocity.y * buoyancy_data.drag,
                            -velocity.z * buoyancy_data.drag
                        ];

                        // Total force (buoyancy up + drag)
                        let total_force =
                            vector![drag_force.x, buoyancy_force + drag_force.y, drag_force.z];

                        rb.add_force(total_force, true);
                    }
                }
            }
        }
    }

    pub fn add_water_aabb(&mut self, _min: Vec3, _max: Vec3, _density: f32, _linear_damp: f32) {}
    
    pub fn set_wind(&mut self, dir: Vec3, strength: f32) {
        self.wind = dir.normalize_or_zero() * strength;
    }

    /// Apply radial explosion impulse to all dynamic bodies in range
    ///
    /// # Arguments
    /// * `center` - Explosion center
    /// * `radius` - Maximum effect radius
    /// * `force` - Force at center (falls off with distance)
    /// * `falloff` - Falloff curve type
    /// * `upward_bias` - Upward impulse bias (0.0 = radial, 1.0 = fully upward)
    ///
    /// # Returns
    /// Number of bodies affected
    pub fn apply_radial_impulse(
        &mut self,
        center: Vec3,
        radius: f32,
        force: f32,
        falloff: projectile::FalloffCurve,
        upward_bias: f32,
    ) -> usize {
        // Collect body positions and IDs
        let mut affected = Vec::new();

        for (&handle, &_body_id) in &self.body_ids {
            if let Some(rb) = self.bodies.get(handle) {
                if !rb.is_dynamic() {
                    continue;
                }
                let pos = rb.position().translation;
                let body_pos = Vec3::new(pos.x, pos.y, pos.z);
                let to_body = body_pos - center;
                let distance = to_body.length();

                if distance < radius {
                    affected.push((handle, body_pos, distance, to_body));
                }
            }
        }

        // Apply impulses
        let count = affected.len();
        for (handle, _body_pos, distance, to_body) in affected {
            let falloff_mult = falloff.calculate(distance, radius);
            let force_mag = force * falloff_mult;

            let radial_dir = if distance > 0.001 {
                to_body.normalize()
            } else {
                Vec3::Y
            };

            let biased_dir =
                (radial_dir * (1.0 - upward_bias) + Vec3::Y * upward_bias).normalize();
            let impulse = biased_dir * force_mag;

            if let Some(rb) = self.bodies.get_mut(handle) {
                rb.apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);
            }
        }

        count
    }

    /// Perform a raycast and return hit info
    ///
    /// # Returns
    /// (hit_position, hit_normal, body_id, distance) if hit
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(Vec3, Vec3, Option<BodyId>, f32)> {
        let ray = rapier3d::prelude::Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        self.query_pipeline
            .cast_ray_and_get_normal(
                &self.bodies,
                &self.colliders,
                &ray,
                max_distance,
                true,
                QueryFilter::default(),
            )
            .map(|(collider_handle, hit)| {
                let hit_pos = origin + direction * hit.time_of_impact;
                let normal = Vec3::new(hit.normal.x, hit.normal.y, hit.normal.z);
                
                // Get body ID from collider
                let body_id = self.colliders
                    .get(collider_handle)
                    .and_then(|c| c.parent())
                    .and_then(|rb_handle| self.body_ids.get(&rb_handle).copied());

                (hit_pos, normal, body_id, hit.time_of_impact)
            })
    }

    pub fn clear_water(&mut self) {}
    pub fn add_destructible_box(
        &mut self,
        pos: Vec3,
        half: Vec3,
        mass: f32,
        _health: f32,
        _break_impulse: f32,
    ) -> BodyId {
        self.add_dynamic_box(pos, half, mass, Layers::DEFAULT)
    }
    pub fn break_destructible(&mut self, id: BodyId) {
        if let Some(h) = self.handle_of(id) {
            // Remove from Rapier sets
            self.bodies.remove(
                h,
                &mut self.island_mgr,
                &mut self.colliders,
                &mut self.joints,
                &mut self.multibody_joints,
                true,
            );

            // Remove from our mappings
            self.body_ids.remove(&h);
            self.body_kinds.remove(&h);
            self.char_map.remove(&id);
            self.buoyancy_bodies.remove(&id);
        }
    }

    #[allow(dead_code)]
    fn process_destructible_hits(&mut self) {}

    pub fn set_body_position(&mut self, id: BodyId, pos: Vec3) {
        if let Some(h) = self.handle_of(id) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.set_translation(vector![pos.x, pos.y, pos.z], true);
            }
        }
    }

    pub fn enable_ccd(&mut self, id: BodyId) {
        if let Some(h) = self.handle_of(id) {
            if let Some(rb) = self.bodies.get_mut(h) {
                rb.enable_ccd(true);
            }
        }
    }

    pub fn add_joint(&mut self, body1: BodyId, body2: BodyId, joint_type: JointType) -> JointId {
        let Some(handle1) = self.handle_of(body1) else {
            return JointId(0);
        };
        let Some(handle2) = self.handle_of(body2) else {
            return JointId(0);
        };

        let joint = match joint_type {
            JointType::Fixed => GenericJointBuilder::new(JointAxesMask::LOCKED_FIXED_AXES).build(),
            JointType::Revolute { axis, limits } => {
                let local_axis = UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]);
                let mut builder = RevoluteJointBuilder::new(local_axis);
                if let Some((min, max)) = limits {
                    builder = builder.limits([min, max]);
                }
                builder.build().into()
            }
            JointType::Prismatic { axis, limits } => {
                let local_axis = UnitVector::new_normalize(vector![axis.x, axis.y, axis.z]);
                let mut builder = PrismaticJointBuilder::new(local_axis);
                if let Some((min, max)) = limits {
                    builder = builder.limits([min, max]);
                }
                builder.build().into()
            }
            JointType::Spherical => SphericalJointBuilder::new().build().into(),
        };

        self.joints.insert(handle1, handle2, joint, true);

        let joint_id = self.next_joint_id;
        self.next_joint_id += 1;
        JointId(joint_id)
    }

    pub fn get_debug_lines(&mut self) -> Vec<DebugLine> {
        let mut collector = LineCollector::new();
        self.debug_render_pipeline.render(
            &mut collector,
            &self.bodies,
            &self.colliders,
            &self.joints,
            &self.multibody_joints,
            &self.narrow_phase,
        );
        collector.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_position_updates() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let _ground = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
        let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));

        // Check initial position
        let pos0 = pw.body_transform(char_id).unwrap().w_axis;
        assert!(
            (pos0.x - 0.0).abs() < 0.01,
            "initial x should be ~0, got {}",
            pos0.x
        );

        // Move once
        pw.control_character(char_id, Vec3::new(1.0, 0.0, 0.0), 1.0 / 60.0, false);
        pw.step();

        let pos1 = pw.body_transform(char_id).unwrap().w_axis;

        // Move again
        pw.control_character(char_id, Vec3::new(1.0, 0.0, 0.0), 1.0 / 60.0, false);
        pw.step();

        let pos2 = pw.body_transform(char_id).unwrap().w_axis;

        // Position should accumulate
        assert!(
            pos2.x > pos1.x,
            "x should increase: frame1={}, frame2={}",
            pos1.x,
            pos2.x
        );
    }

    #[test]
    fn character_moves_forward() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let _ground = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
        let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));
        for _ in 0..60 {
            pw.control_character(char_id, Vec3::new(1.0, 0.0, 0.0), 1.0 / 60.0, false);
            pw.step();
        }
        let x = pw.body_transform(char_id).unwrap().w_axis.x;
        assert!(x > 0.5, "character should have moved forward, x={}", x);
    }

    // ===== PhysicsWorld Basic Tests =====

    #[test]
    fn test_physics_world_new() {
        let pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        assert_eq!(pw.gravity.x, 0.0);
        assert_eq!(pw.gravity.y, -9.8);
        assert_eq!(pw.gravity.z, 0.0);
        assert!(pw.bodies.is_empty());
        assert!(pw.colliders.is_empty());
    }

    #[test]
    fn test_physics_world_from_config() {
        let config = PhysicsConfig {
            gravity: Vec3::new(0.0, -10.0, 0.0),
            time_step: 1.0 / 120.0,
            ccd_enabled: false,
            max_ccd_substeps: 1,
            water_level: f32::NEG_INFINITY,
            fluid_density: 1000.0,
        };
        let pw = PhysicsWorld::from_config(config);
        assert_eq!(pw.gravity.y, -10.0);
        assert_eq!(pw.integration.dt, 1.0 / 120.0);
    }

    #[test]
    fn test_create_ground_plane() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let ground = pw.create_ground_plane(Vec3::new(5.0, 0.5, 5.0), 0.8);
        
        assert!(pw.body_transform(ground).is_some());
        assert!(!pw.bodies.is_empty());
        assert!(!pw.colliders.is_empty());
    }

    #[test]
    fn test_add_dynamic_box() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        assert!(pw.body_transform(box_id).is_some());
        
        // Step physics - box should fall
        for _ in 0..60 {
            pw.step();
        }
        
        let y = pw.body_transform(box_id).unwrap().w_axis.y;
        assert!(y < 5.0, "Box should have fallen, y={}", y);
    }

    #[test]
    fn test_apply_force() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let _ground = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        pw.apply_force(box_id, Vec3::new(100.0, 0.0, 0.0));
        pw.step();
        
        let vel = pw.get_velocity(box_id).unwrap();
        assert!(vel.x > 0.0, "Force should have applied positive x velocity");
    }

    #[test]
    fn test_apply_impulse() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        pw.apply_impulse(box_id, Vec3::new(0.0, 50.0, 0.0));
        
        let vel = pw.get_velocity(box_id).unwrap();
        assert!(vel.y > 0.0, "Impulse should have given upward velocity");
    }

    #[test]
    fn test_get_set_velocity() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        // Get initial velocity
        let initial_vel = pw.get_velocity(box_id).unwrap();
        assert_eq!(initial_vel.x, 0.0);
        
        // Set velocity
        pw.set_velocity(box_id, Vec3::new(10.0, 0.0, 5.0));
        
        let new_vel = pw.get_velocity(box_id).unwrap();
        assert_eq!(new_vel.x, 10.0);
        assert_eq!(new_vel.z, 5.0);
    }

    #[test]
    fn test_get_velocity_invalid_id() {
        let pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let vel = pw.get_velocity(9999);
        assert!(vel.is_none());
    }

    #[test]
    fn test_body_transform_invalid_id() {
        let pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let transform = pw.body_transform(9999);
        assert!(transform.is_none());
    }

    // ===== Static Trimesh Tests =====

    #[test]
    fn test_add_static_trimesh() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        
        // Create simple triangle
        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let indices = vec![[0, 1, 2]];
        
        let mesh_id = pw.add_static_trimesh(&vertices, &indices, Layers::DEFAULT);
        assert!(pw.body_transform(mesh_id).is_some());
    }

    // ===== Character Controller Tests =====

    #[test]
    fn test_character_jump() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let _ground = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
        let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));
        
        // Initial position
        let y0 = pw.body_transform(char_id).unwrap().w_axis.y;
        
        // Jump
        pw.jump(char_id, 2.0);
        
        // Run physics
        for _ in 0..30 {
            pw.control_character(char_id, Vec3::ZERO, 1.0 / 60.0, false);
            pw.step();
        }
        
        let y1 = pw.body_transform(char_id).unwrap().w_axis.y;
        // After 30 frames of jump, character should have moved
        assert!(y1 != y0, "Jump should change position");
    }

    #[test]
    fn test_control_character_invalid_id() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        // Should not panic on invalid ID
        pw.control_character(9999, Vec3::new(1.0, 0.0, 0.0), 1.0 / 60.0, false);
    }

    #[test]
    fn test_jump_invalid_id() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        // Should not panic on invalid ID
        pw.jump(9999, 2.0);
    }

    // ===== Buoyancy & Environment Tests =====

    #[test]
    fn test_add_buoyancy() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        pw.add_buoyancy(box_id, 1.0, 0.5);
        assert!(pw.buoyancy_bodies.contains_key(&box_id));
    }

    #[test]
    fn test_set_wind() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        pw.set_wind(Vec3::new(1.0, 0.0, 0.0), 5.0);
        
        // set_wind stores dir * strength
        assert_eq!(pw.wind.x, 5.0); // 1.0 * 5.0
        assert_eq!(pw.wind.y, 0.0);
        assert_eq!(pw.wind.z, 0.0);
    }

    // ===== Handle/ID Mapping Tests =====

    #[test]
    fn test_handle_of_valid() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        assert!(pw.handle_of(box_id).is_some());
    }

    #[test]
    fn test_handle_of_invalid() {
        let pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        assert!(pw.handle_of(9999).is_none());
    }

    // ===== ActorKind Tests =====

    #[test]
    fn test_actor_kind_variants() {
        let _ = ActorKind::Static;
        let _ = ActorKind::Dynamic;
        let _ = ActorKind::Character;
        let _ = ActorKind::Other;
    }

    // ===== Layers Tests =====

    #[test]
    fn test_layers_bits() {
        assert_eq!(Layers::DEFAULT.bits(), 0b0001);
        assert_eq!(Layers::CHARACTER.bits(), 0b0010);
    }

    #[test]
    fn test_layers_all() {
        let all = Layers::all();
        assert!(all.contains(Layers::DEFAULT));
        assert!(all.contains(Layers::CHARACTER));
    }

    // ===== PhysicsConfig Tests =====

    #[test]
    fn test_physics_config_default() {
        let config = PhysicsConfig::default();
        assert_eq!(config.time_step, 1.0 / 60.0);
        assert_eq!(config.gravity.y, -9.81);
    }

    // ===== Debug Line Tests =====

    #[test]
    fn test_debug_line_creation() {
        let line = DebugLine {
            start: [0.0, 0.0, 0.0],
            end: [1.0, 1.0, 1.0],
            color: [1.0, 0.0, 0.0],
        };
        
        assert_eq!(line.start, [0.0, 0.0, 0.0]);
        assert_eq!(line.color, [1.0, 0.0, 0.0]);
    }

    // ===== Multiple Bodies Tests =====

    #[test]
    fn test_multiple_bodies() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let _ground = pw.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
        
        let box1 = pw.add_dynamic_box(Vec3::new(-2.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        let box2 = pw.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        let char1 = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));
        
        assert!(pw.body_transform(box1).is_some());
        assert!(pw.body_transform(box2).is_some());
        assert!(pw.body_transform(char1).is_some());
        
        // All should have different IDs
        assert_ne!(box1, box2);
        assert_ne!(box2, char1);
    }

    // ===== Step Integration Test =====

    #[test]
    fn test_physics_step_integration() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let _ground = pw.create_ground_plane(Vec3::new(20.0, 0.5, 20.0), 0.9);
        
        // Add falling box
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        let y_start = pw.body_transform(box_id).unwrap().w_axis.y;
        
        // Step 120 frames (2 seconds)
        for _ in 0..120 {
            pw.step();
        }
        
        let y_end = pw.body_transform(box_id).unwrap().w_axis.y;
        
        // Box should have fallen significantly
        assert!(y_end < y_start - 1.0, "Box should fall, start={}, end={}", y_start, y_end);
    }

    #[test]
    fn test_apply_radial_impulse() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        let count = pw.apply_radial_impulse(
            Vec3::ZERO,
            5.0,
            100.0,
            FalloffCurve::Linear,
            0.0,
        );
        
        assert_eq!(count, 1);
        let vel = pw.get_velocity(box_id).unwrap();
        assert!(vel.length() > 0.0);
    }

    #[test]
    fn test_raycast() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        pw.step(); // Update query pipeline
        
        let hit = pw.raycast(Vec3::ZERO, Vec3::X, 10.0);
        assert!(hit.is_some());
        let (_, _, hit_id, _) = hit.unwrap();
        assert_eq!(hit_id, Some(box_id));
    }

    #[test]
    fn test_break_destructible() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        
        assert!(pw.handle_of(box_id).is_some());
        pw.break_destructible(box_id);
        assert!(pw.handle_of(box_id).is_none());
    }

    #[test]
    fn test_add_joint() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let b1 = pw.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        let b2 = pw.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        
        let j1 = pw.add_joint(b1, b2, JointType::Fixed);
        assert_ne!(j1.0, 0);
        
        let j2 = pw.add_joint(b1, b2, JointType::Revolute { axis: Vec3::Y, limits: None });
        assert_ne!(j2.0, 0);

        let j3 = pw.add_joint(b1, b2, JointType::Prismatic { axis: Vec3::X, limits: Some((-1.0, 1.0)) });
        assert_ne!(j3.0, 0);

        let j4 = pw.add_joint(b1, b2, JointType::Spherical);
        assert_ne!(j4.0, 0);
    }

    #[test]
    fn test_set_body_position() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        
        pw.set_body_position(box_id, Vec3::new(10.0, 10.0, 10.0));
        let transform = pw.body_transform(box_id).unwrap();
        assert!((transform.w_axis.x - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_enable_ccd() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let box_id = pw.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        pw.enable_ccd(box_id);
    }

    #[test]
    fn test_get_debug_lines() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        pw.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        let lines = pw.get_debug_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_character_climb() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        let char_id = pw.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.4, 0.9, 0.4));
        
        let y0 = pw.body_transform(char_id).unwrap().w_axis.y;
        pw.control_character(char_id, Vec3::ZERO, 0.1, true);
        pw.step();
        
        let y1 = pw.body_transform(char_id).unwrap().w_axis.y;
        assert!(y1 > y0);
    }

    #[test]
    fn test_buoyancy_application() {
        let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
        pw.water_level = 10.0;
        let box_id = pw.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.5, 0.5, 0.5), 1.0, Layers::DEFAULT);
        pw.add_buoyancy(box_id, 1.0, 0.5);
        
        // Step should apply buoyancy force
        pw.step();
        let vel = pw.get_velocity(box_id).unwrap();
        assert!(vel.y > 0.0);
    }

    // ===== ActorKind Helper Tests =====

    #[test]
    fn test_actor_kind_name() {
        assert_eq!(ActorKind::Static.name(), "Static");
        assert_eq!(ActorKind::Dynamic.name(), "Dynamic");
        assert_eq!(ActorKind::Character.name(), "Character");
        assert_eq!(ActorKind::Other.name(), "Other");
    }

    #[test]
    fn test_actor_kind_is_static() {
        assert!(ActorKind::Static.is_static());
        assert!(!ActorKind::Dynamic.is_static());
        assert!(!ActorKind::Character.is_static());
        assert!(!ActorKind::Other.is_static());
    }

    #[test]
    fn test_actor_kind_is_dynamic() {
        assert!(!ActorKind::Static.is_dynamic());
        assert!(ActorKind::Dynamic.is_dynamic());
        assert!(!ActorKind::Character.is_dynamic());
        assert!(!ActorKind::Other.is_dynamic());
    }

    #[test]
    fn test_actor_kind_is_character() {
        assert!(!ActorKind::Static.is_character());
        assert!(!ActorKind::Dynamic.is_character());
        assert!(ActorKind::Character.is_character());
        assert!(!ActorKind::Other.is_character());
    }

    #[test]
    fn test_actor_kind_is_movable() {
        assert!(!ActorKind::Static.is_movable());
        assert!(ActorKind::Dynamic.is_movable());
        assert!(ActorKind::Character.is_movable());
        assert!(!ActorKind::Other.is_movable());
    }

    #[test]
    fn test_actor_kind_all() {
        let all = ActorKind::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&ActorKind::Static));
        assert!(all.contains(&ActorKind::Dynamic));
        assert!(all.contains(&ActorKind::Character));
        assert!(all.contains(&ActorKind::Other));
    }

    #[test]
    fn test_actor_kind_display() {
        assert_eq!(format!("{}", ActorKind::Static), "Static");
        assert_eq!(format!("{}", ActorKind::Dynamic), "Dynamic");
        assert_eq!(format!("{}", ActorKind::Character), "Character");
        assert_eq!(format!("{}", ActorKind::Other), "Other");
    }

    // ===== DebugLine Helper Tests =====

    #[test]
    fn test_debug_line_new() {
        let line = DebugLine::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(line.start, [0.0, 0.0, 0.0]);
        assert_eq!(line.end, [1.0, 0.0, 0.0]);
        assert_eq!(line.color, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_debug_line_from_vec3() {
        let line = DebugLine::from_vec3(Vec3::ZERO, Vec3::X, [0.0, 1.0, 0.0]);
        assert_eq!(line.start, [0.0, 0.0, 0.0]);
        assert_eq!(line.end, [1.0, 0.0, 0.0]);
        assert_eq!(line.color, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_debug_line_length() {
        let line = DebugLine::new([0.0, 0.0, 0.0], [3.0, 4.0, 0.0], [1.0, 1.0, 1.0]);
        assert!((line.length() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_debug_line_length_squared() {
        let line = DebugLine::new([0.0, 0.0, 0.0], [3.0, 4.0, 0.0], [1.0, 1.0, 1.0]);
        assert!((line.length_squared() - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_debug_line_midpoint() {
        let line = DebugLine::new([0.0, 0.0, 0.0], [2.0, 4.0, 6.0], [1.0, 1.0, 1.0]);
        let mid = line.midpoint();
        assert!((mid[0] - 1.0).abs() < 0.001);
        assert!((mid[1] - 2.0).abs() < 0.001);
        assert!((mid[2] - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_debug_line_direction() {
        let line = DebugLine::new([1.0, 2.0, 3.0], [4.0, 6.0, 9.0], [1.0, 1.0, 1.0]);
        let dir = line.direction();
        assert!((dir[0] - 3.0).abs() < 0.001);
        assert!((dir[1] - 4.0).abs() < 0.001);
        assert!((dir[2] - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_debug_line_is_degenerate() {
        let degen = DebugLine::new([1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [1.0, 1.0, 1.0]);
        assert!(degen.is_degenerate());
        
        let valid = DebugLine::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(!valid.is_degenerate());
    }

    #[test]
    fn test_debug_line_color_constructors() {
        let red = DebugLine::red([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(red.color, [1.0, 0.0, 0.0]);
        
        let green = DebugLine::green([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(green.color, [0.0, 1.0, 0.0]);
        
        let blue = DebugLine::blue([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(blue.color, [0.0, 0.0, 1.0]);
        
        let white = DebugLine::white([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(white.color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_debug_line_display() {
        let line = DebugLine::new([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [0.5, 0.5, 0.5]);
        let display = format!("{}", line);
        assert!(display.contains("Line"));
        assert!(display.contains("0.00"));
    }

    // ===== CharState Helper Tests =====

    #[test]
    fn test_char_state_name() {
        assert_eq!(CharState::Grounded.name(), "Grounded");
    }

    #[test]
    fn test_char_state_is_grounded() {
        assert!(CharState::Grounded.is_grounded());
    }

    #[test]
    fn test_char_state_all() {
        let all = CharState::all();
        assert_eq!(all.len(), 1);
        assert!(all.contains(&CharState::Grounded));
    }

    #[test]
    fn test_char_state_display() {
        assert_eq!(format!("{}", CharState::Grounded), "Grounded");
    }

    // ===== CharacterController Helper Tests =====

    #[test]
    fn test_character_controller_new() {
        let cc = CharacterController::new(0.5, 2.0);
        assert_eq!(cc.radius, 0.5);
        assert_eq!(cc.height, 2.0);
        assert!(cc.is_grounded());
    }

    #[test]
    fn test_character_controller_is_grounded() {
        let cc = CharacterController::new(0.5, 2.0);
        assert!(cc.is_grounded());
    }

    #[test]
    fn test_character_controller_has_coyote_time() {
        let mut cc = CharacterController::new(0.5, 2.0);
        assert!(cc.has_coyote_time()); // time_since_grounded = 0.0
        
        cc.time_since_grounded = 0.2;
        assert!(!cc.has_coyote_time());
    }

    #[test]
    fn test_character_controller_has_buffered_jump() {
        let mut cc = CharacterController::new(0.5, 2.0);
        assert!(!cc.has_buffered_jump());
        
        cc.jump_buffer_timer = 0.1;
        assert!(cc.has_buffered_jump());
    }

    #[test]
    fn test_character_controller_can_jump() {
        let cc = CharacterController::new(0.5, 2.0);
        // When grounded, can_jump is always true regardless of time_since_grounded
        assert!(cc.can_jump());
        
        // Test coyote time - still returns true when time_since_grounded < coyote_time_limit
        let mut cc2 = CharacterController::new(0.5, 2.0);
        cc2.time_since_grounded = 0.1; // within coyote time
        assert!(cc2.can_jump()); // should be true (grounded OR coyote time)
    }

    #[test]
    fn test_character_controller_volume() {
        let cc = CharacterController::new(1.0, 4.0);
        let vol = cc.volume();
        // Volume should be positive and reasonable
        assert!(vol > 0.0);
        assert!(vol < 100.0);
    }

    #[test]
    fn test_character_controller_max_climb_angle_rad() {
        let cc = CharacterController::new(0.5, 2.0);
        let rad = cc.max_climb_angle_rad();
        assert!((rad - 45.0_f32.to_radians()).abs() < 0.001);
    }

    #[test]
    fn test_character_controller_is_falling_rising() {
        let mut cc = CharacterController::new(0.5, 2.0);
        
        cc.vertical_velocity = -5.0;
        assert!(cc.is_falling());
        assert!(!cc.is_rising());
        
        cc.vertical_velocity = 5.0;
        assert!(!cc.is_falling());
        assert!(cc.is_rising());
        
        cc.vertical_velocity = 0.0;
        assert!(!cc.is_falling());
        assert!(!cc.is_rising());
    }

    #[test]
    fn test_character_controller_reset() {
        let mut cc = CharacterController::new(0.5, 2.0);
        cc.vertical_velocity = 10.0;
        cc.time_since_grounded = 1.0;
        cc.jump_buffer_timer = 0.5;
        cc.pending_jump_velocity = 5.0;
        
        cc.reset();
        
        assert!(cc.is_grounded());
        assert_eq!(cc.vertical_velocity, 0.0);
        assert_eq!(cc.time_since_grounded, 0.0);
        assert_eq!(cc.jump_buffer_timer, 0.0);
        assert_eq!(cc.pending_jump_velocity, 0.0);
    }

    #[test]
    fn test_character_controller_display() {
        let cc = CharacterController::new(0.5, 2.0);
        let display = format!("{}", cc);
        assert!(display.contains("CharController"));
        assert!(display.contains("0.50"));
        assert!(display.contains("2.00"));
    }

    // ===== PhysicsConfig Helper Tests =====

    #[test]
    fn test_physics_config_new() {
        let config = PhysicsConfig::new();
        assert!((config.gravity.y + 9.81).abs() < 0.01);
        assert!(!config.ccd_enabled);
    }

    #[test]
    fn test_physics_config_with_gravity() {
        let config = PhysicsConfig::new().with_gravity(Vec3::new(0.0, -20.0, 0.0));
        assert_eq!(config.gravity.y, -20.0);
    }

    #[test]
    fn test_physics_config_with_ccd() {
        let config = PhysicsConfig::new().with_ccd(4);
        assert!(config.ccd_enabled);
        assert_eq!(config.max_ccd_substeps, 4);
    }

    #[test]
    fn test_physics_config_with_time_step() {
        let config = PhysicsConfig::new().with_time_step(1.0 / 120.0);
        assert!((config.time_step - 1.0 / 120.0).abs() < 0.0001);
    }

    #[test]
    fn test_physics_config_with_water() {
        let config = PhysicsConfig::new().with_water(5.0, 1025.0);
        assert_eq!(config.water_level, 5.0);
        assert_eq!(config.fluid_density, 1025.0);
    }

    #[test]
    fn test_physics_config_target_fps() {
        let config = PhysicsConfig::new();
        assert!((config.target_fps() - 60.0).abs() < 0.1);
        
        let config120 = PhysicsConfig::new().with_time_step(1.0 / 120.0);
        assert!((config120.target_fps() - 120.0).abs() < 0.1);
    }

    #[test]
    fn test_physics_config_has_water() {
        let config = PhysicsConfig::new();
        assert!(!config.has_water()); // default is NEG_INFINITY
        
        let with_water = PhysicsConfig::new().with_water(0.0, 1000.0);
        assert!(with_water.has_water());
    }

    #[test]
    fn test_physics_config_gravity_magnitude() {
        let config = PhysicsConfig::new();
        assert!((config.gravity_magnitude() - 9.81).abs() < 0.01);
    }

    #[test]
    fn test_physics_config_is_earth_gravity() {
        let config = PhysicsConfig::new();
        assert!(config.is_earth_gravity());
        
        let moon = PhysicsConfig::new().with_gravity(Vec3::new(0.0, -1.62, 0.0));
        assert!(!moon.is_earth_gravity());
    }

    #[test]
    fn test_physics_config_is_zero_gravity() {
        let zero = PhysicsConfig::new().with_gravity(Vec3::ZERO);
        assert!(zero.is_zero_gravity());
        
        let earth = PhysicsConfig::new();
        assert!(!earth.is_zero_gravity());
    }

    #[test]
    fn test_physics_config_display() {
        let config = PhysicsConfig::new();
        let display = format!("{}", config);
        assert!(display.contains("PhysicsConfig"));
        assert!(display.contains("-9.81"));
    }

    // ===== JointType Helper Tests =====

    #[test]
    fn test_joint_type_name() {
        assert_eq!(JointType::Fixed.name(), "Fixed");
        assert_eq!(JointType::Revolute { axis: Vec3::Y, limits: None }.name(), "Revolute");
        assert_eq!(JointType::Prismatic { axis: Vec3::X, limits: None }.name(), "Prismatic");
        assert_eq!(JointType::Spherical.name(), "Spherical");
    }

    #[test]
    fn test_joint_type_is_fixed() {
        assert!(JointType::Fixed.is_fixed());
        assert!(!JointType::Spherical.is_fixed());
    }

    #[test]
    fn test_joint_type_is_revolute() {
        assert!(JointType::Revolute { axis: Vec3::Y, limits: None }.is_revolute());
        assert!(!JointType::Fixed.is_revolute());
    }

    #[test]
    fn test_joint_type_is_prismatic() {
        assert!(JointType::Prismatic { axis: Vec3::X, limits: None }.is_prismatic());
        assert!(!JointType::Fixed.is_prismatic());
    }

    #[test]
    fn test_joint_type_is_spherical() {
        assert!(JointType::Spherical.is_spherical());
        assert!(!JointType::Fixed.is_spherical());
    }

    #[test]
    fn test_joint_type_has_limits() {
        let no_limits = JointType::Revolute { axis: Vec3::Y, limits: None };
        assert!(!no_limits.has_limits());
        
        let with_limits = JointType::Revolute { axis: Vec3::Y, limits: Some((-1.0, 1.0)) };
        assert!(with_limits.has_limits());
        
        assert!(!JointType::Fixed.has_limits());
        assert!(!JointType::Spherical.has_limits());
    }

    #[test]
    fn test_joint_type_axis() {
        let revolute = JointType::Revolute { axis: Vec3::Y, limits: None };
        assert_eq!(revolute.axis(), Some(Vec3::Y));
        
        let prismatic = JointType::Prismatic { axis: Vec3::X, limits: None };
        assert_eq!(prismatic.axis(), Some(Vec3::X));
        
        assert_eq!(JointType::Fixed.axis(), None);
        assert_eq!(JointType::Spherical.axis(), None);
    }

    #[test]
    fn test_joint_type_limits() {
        let with_limits = JointType::Revolute { axis: Vec3::Y, limits: Some((-1.5, 1.5)) };
        assert_eq!(with_limits.limits(), Some((-1.5, 1.5)));
        
        let no_limits = JointType::Revolute { axis: Vec3::Y, limits: None };
        assert_eq!(no_limits.limits(), None);
    }

    #[test]
    fn test_joint_type_is_rotational() {
        assert!(JointType::Revolute { axis: Vec3::Y, limits: None }.is_rotational());
        assert!(JointType::Spherical.is_rotational());
        assert!(!JointType::Fixed.is_rotational());
        assert!(!JointType::Prismatic { axis: Vec3::X, limits: None }.is_rotational());
    }

    #[test]
    fn test_joint_type_is_linear() {
        assert!(JointType::Prismatic { axis: Vec3::X, limits: None }.is_linear());
        assert!(!JointType::Revolute { axis: Vec3::Y, limits: None }.is_linear());
        assert!(!JointType::Fixed.is_linear());
    }

    #[test]
    fn test_joint_type_degrees_of_freedom() {
        assert_eq!(JointType::Fixed.degrees_of_freedom(), 0);
        assert_eq!(JointType::Revolute { axis: Vec3::Y, limits: None }.degrees_of_freedom(), 1);
        assert_eq!(JointType::Prismatic { axis: Vec3::X, limits: None }.degrees_of_freedom(), 1);
        assert_eq!(JointType::Spherical.degrees_of_freedom(), 3);
    }

    #[test]
    fn test_joint_type_factory_methods() {
        let ry = JointType::revolute_y();
        assert!(ry.is_revolute());
        assert_eq!(ry.axis(), Some(Vec3::Y));
        
        let rx = JointType::revolute_x();
        assert_eq!(rx.axis(), Some(Vec3::X));
        
        let rz = JointType::revolute_z();
        assert_eq!(rz.axis(), Some(Vec3::Z));
        
        let py = JointType::prismatic_y();
        assert!(py.is_prismatic());
        assert_eq!(py.axis(), Some(Vec3::Y));
    }

    #[test]
    fn test_joint_type_display() {
        assert_eq!(format!("{}", JointType::Fixed), "Fixed");
        assert_eq!(format!("{}", JointType::Spherical), "Spherical");
        
        let revolute = JointType::Revolute { axis: Vec3::Y, limits: None };
        let display = format!("{}", revolute);
        assert!(display.contains("Revolute"));
        
        let with_limits = JointType::Revolute { axis: Vec3::Y, limits: Some((-1.0, 1.0)) };
        let display2 = format!("{}", with_limits);
        assert!(display2.contains("limits"));
    }

    // ===== JointId Helper Tests =====

    #[test]
    fn test_joint_id_new() {
        let id = JointId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_joint_id_raw() {
        let id = JointId::new(123);
        assert_eq!(id.raw(), 123);
    }

    #[test]
    fn test_joint_id_is_valid() {
        let valid = JointId::new(1);
        assert!(valid.is_valid());
        
        let invalid = JointId::new(0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_joint_id_invalid() {
        let invalid = JointId::invalid();
        assert_eq!(invalid.0, 0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_joint_id_display() {
        let id = JointId::new(42);
        assert_eq!(format!("{}", id), "JointId(42)");
    }

    #[test]
    fn test_joint_id_from_u64() {
        let id: JointId = 100u64.into();
        assert_eq!(id.raw(), 100);
    }

    #[test]
    fn test_joint_id_into_u64() {
        let id = JointId::new(200);
        let raw: u64 = id.into();
        assert_eq!(raw, 200);
    }

    #[test]
    fn test_joint_id_default() {
        let id = JointId::default();
        assert_eq!(id.0, 0);
        assert!(!id.is_valid());
    }

    // ===== BuoyancyData Helper Tests =====

    #[test]
    fn test_buoyancy_data_new() {
        let bd = BuoyancyData::new(1.5, 0.5);
        assert_eq!(bd.volume, 1.5);
        assert_eq!(bd.drag, 0.5);
    }

    #[test]
    fn test_buoyancy_data_with_volume() {
        let bd = BuoyancyData::with_volume(2.0);
        assert_eq!(bd.volume, 2.0);
        assert_eq!(bd.drag, 0.0);
    }

    #[test]
    fn test_buoyancy_data_is_valid() {
        let valid = BuoyancyData::new(1.0, 0.5);
        assert!(valid.is_valid());
        
        let invalid = BuoyancyData::new(0.0, 0.5);
        assert!(!invalid.is_valid());
        
        let negative = BuoyancyData::new(-1.0, 0.5);
        assert!(!negative.is_valid());
    }

    #[test]
    fn test_buoyancy_data_buoyancy_force() {
        let bd = BuoyancyData::new(1.0, 0.0);
        let force = bd.buoyancy_force(1000.0);
        // F = V * rho * g = 1.0 * 1000.0 * 9.81 = 9810
        assert!((force - 9810.0).abs() < 1.0);
    }

    #[test]
    fn test_buoyancy_data_drag_force() {
        let bd = BuoyancyData::new(1.0, 1.0);
        let force = bd.drag_force(10.0);
        // F = 0.5 * drag * v^2 = 0.5 * 1.0 * 100 = 50
        assert!((force - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_buoyancy_data_display() {
        let bd = BuoyancyData::new(1.5, 0.25);
        let display = format!("{}", bd);
        assert!(display.contains("Buoyancy"));
        assert!(display.contains("1.500"));
        assert!(display.contains("0.250"));
    }

    #[test]
    fn test_buoyancy_data_default() {
        let bd = BuoyancyData::default();
        assert_eq!(bd.volume, 0.0);
        assert_eq!(bd.drag, 0.0);
    }
}
