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

#[derive(Clone, Copy, Debug)]
pub enum ActorKind {
    Static,
    Dynamic,
    Character,
    Other,
}

#[derive(Clone, Debug)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 3],
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharState {
    Grounded,
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

#[derive(Clone, Debug)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub ccd_enabled: bool,
    pub max_ccd_substeps: usize,
    pub time_step: f32,
    pub water_level: f32,
    pub fluid_density: f32,
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JointId(pub u64);

#[derive(Clone, Copy, Debug)]
pub struct BuoyancyData {
    pub volume: f32,
    pub drag: f32,
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
}
