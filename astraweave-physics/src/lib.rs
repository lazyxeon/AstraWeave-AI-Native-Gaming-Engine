use glam::{vec3, Mat4, Vec3};
use rapier3d::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "profiling")]
use astraweave_profiling::{plot, span};

// Async physics scheduler (feature-gated)
#[cfg(feature = "async-physics")]
pub mod async_scheduler;

#[cfg(feature = "async-physics")]
pub use async_scheduler::{AsyncPhysicsScheduler, PhysicsStepProfile};

// Spatial hash grid for broad-phase collision optimization
pub mod spatial_hash;
pub use spatial_hash::{SpatialHash, SpatialHashStats, AABB};

pub type BodyId = u64;

#[derive(Clone, Copy, Debug)]
pub enum ActorKind {
    Static,
    Dynamic,
    Character,
    Other,
}

bitflags::bitflags! {
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
    body_ids: HashMap<RigidBodyHandle, BodyId>,
    body_kinds: HashMap<RigidBodyHandle, ActorKind>,
    next_body_id: BodyId,
    pub char_map: HashMap<BodyId, CharacterController>,

    /// Async physics scheduler (feature-gated)
    #[cfg(feature = "async-physics")]
    pub async_scheduler: Option<AsyncPhysicsScheduler>,
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3) -> Self {
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
            body_ids: HashMap::new(),
            body_kinds: HashMap::new(),
            next_body_id: 1,
            char_map: HashMap::new(),
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

        let events = ();
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
            &events,
            &(),
        );

        // CRITICAL FIX (Week 2 Day 3): Update query pipeline after physics step
        // Without this, raycasts in control_character() use stale geometry,
        // causing character controller to fail ground detection
        self.query_pipeline.update(&self.colliders);
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
            },
        );
        id
    }

    pub fn control_character(&mut self, id: BodyId, desired_move: Vec3, dt: f32, _climb: bool) {
        #[cfg(feature = "profiling")]
        span!("Physics::CharacterController::move");

        let Some(ctrl) = self.char_map.get(&id).copied() else {
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
        let mut d = desired_move * dt;
        if d.length_squared() < 1e-6 {
            return;
        }

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

        // Tentative horizontal move
        let mut new_pos = start + glam::Vec3::new(d.x, 0.0, d.z);

        // Step/slope correction: raycast down from above feet
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
            QueryFilter::default(),
        ) {
            let ground_normal =
                glam::Vec3::new(hit.normal.x, hit.normal.y, hit.normal.z).normalize();
            let slope = ground_normal.dot(glam::Vec3::Y).acos().to_degrees();
            // Compute ground height at hit
            let ground_y = cast_origin.y - hit.time_of_impact;
            // Allow stepping up small ledges and restrict slope
            if slope <= ctrl.max_climb_angle_deg + 1e-2 {
                // If we need to step up more than max_step, clamp to max_step
                let desired_y = (ground_y + ctrl.radius).max(start.y - 0.1);
                let climb = desired_y - start.y;
                let clamp_climb = climb.clamp(-ctrl.max_step, ctrl.max_step);
                new_pos.y = start.y + clamp_climb;
            } else {
                // Too steep; cancel vertical change
                new_pos.y = start.y;
            }
        } else {
            // No ground: keep current height
            new_pos.y = start.y;
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

    pub fn add_water_aabb(&mut self, _min: Vec3, _max: Vec3, _density: f32, _linear_damp: f32) {}
    pub fn set_wind(&mut self, _dir: Vec3, _strength: f32) {}
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
    pub fn break_destructible(&mut self, _id: BodyId) {}

    #[allow(dead_code)]
    fn process_destructible_hits(&mut self) {}
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
