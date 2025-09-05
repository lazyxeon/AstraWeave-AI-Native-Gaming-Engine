//! AstraWeave Physics: 3D physics world (Rapier3D) with kinematic character controller,
//! ragdoll, buoyancy/swimming, wind, destructibles.

use glam::{vec3, Mat4, Vec3};
use rapier3d::prelude::*;
use std::collections::HashMap;

pub type BodyId = u64;

#[derive(Clone, Copy, Debug)]
pub enum ActorKind {
    Static,
    Dynamic,
    Character,
    Destructible,
    RagdollLink,
    Other,
}

bitflags::bitflags! {
    pub struct Layers: u32 {
        const DEFAULT    = 0b00000001;
        const CHARACTER  = 0b00000010;
        const CLIMBABLE  = 0b00000100;
        const WATER      = 0b00001000;
        const DESTRUCT   = 0b00010000;
    }
}

#[derive(Clone, Debug)]
pub struct Destructible {
    pub health: f32,
    pub break_impulse: f32,   // impulse threshold to auto-damage on hits
    pub fragments: u8,        // how many pieces spawn when broken
}

#[derive(Clone, Debug)]
pub struct WaterVolume {
    pub min: Vec3,
    pub max: Vec3,
    pub density: f32,         // kg/m^3
    pub linear_damp: f32,     // water drag
}

#[derive(Clone, Debug)]
pub struct Wind {
    pub dir: Vec3,            // direction (normalized or any magnitude)
    pub strength: f32,        // scalar force factor
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharState { Grounded, Climbing, Swimming }

pub struct CharacterController {
    pub handle: KinematicCharacterController,
    pub state: CharState,
    pub max_climb_angle_deg: f32,
}

pub struct Ragdoll {
    pub part_handles: Vec<RigidBodyHandle>,
    pub joints: Vec<ImpulseJointHandle>,
}

pub struct PhysicsWorld {
    // Rapier core:
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub pipeline: PhysicsPipeline,
    pub gravity: Vector<Real>,
    pub integration: IntegrationParameters,
    pub island_mgr: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub ccd: CCDSolver,
    pub hooks: (),
    pub events: ChannelEventCollector, // collects contact/impulse events

    // Book-keeping:
    body_ids: HashMap<RigidBodyHandle, BodyId>,
    body_kinds: HashMap<RigidBodyHandle, ActorKind>,
    next_body_id: BodyId,

    // Characters:
    pub char_map: HashMap<BodyId, CharacterController>,

    // Destructibles:
    pub destructibles: HashMap<BodyId, Destructible>,

    // Water + Wind:
    pub water: Vec<WaterVolume>,
    pub wind: Option<Wind>,

    // collision layers
    pub default_pair_filter: QueryFilter,
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3) -> Self {
        let (tx, _rx) = crossbeam::channel::unbounded();
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            pipeline: PhysicsPipeline::new(),
            gravity: vector![gravity.x, gravity.y, gravity.z],
            integration: IntegrationParameters::default(),
            island_mgr: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd: CCDSolver::new(),
            hooks: (),
            events: ChannelEventCollector::new(tx),
            body_ids: HashMap::new(),
            body_kinds: HashMap::new(),
            next_body_id: 1,
            char_map: HashMap::new(),
            destructibles: HashMap::new(),
            water: vec![],
            wind: None,
            default_pair_filter: QueryFilter::new(),
        }
    }

    fn alloc_id(&mut self) -> BodyId { let id = self.next_body_id; self.next_body_id += 1; id }

    pub fn step(&mut self, dt: f32) {
        // Apply wind + buoyancy pre-step
        if self.wind.is_some() || !self.water.is_empty() {
            self.apply_environment_forces(dt);
        }

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
            &self.hooks,
            &self.events,
        );

        // Handle simple destructible damage from contact impulses
        self.process_destructible_hits();

        // Character controller states (swim/ climb transitions happen in control fn)
    }

    // ---------- Creation helpers ----------
    pub fn create_ground_plane(&mut self, half_extent: Vec3, friction: f32) -> BodyId {
        let rb = RigidBodyBuilder::fixed().translation(vector![0.0, 0.0, 0.0]).build();
        let h = self.bodies.insert(rb);
        let shape = ColliderBuilder::cuboid(half_extent.x, 0.1, half_extent.z)
            .friction(friction)
            .collision_groups(InteractionGroups::new(Layers::DEFAULT.bits(), u32::MAX))
            .build();
        self.colliders.insert_with_parent(shape, h, &mut self.bodies);
        self.tag_body(h, ActorKind::Static)
    }

    pub fn add_static_trimesh(&mut self, vertices: &[Vec3], indices: &[[u32;3]], groups: Layers) -> BodyId {
        let rb = RigidBodyBuilder::fixed().build();
        let h = self.bodies.insert(rb);
        let v: Vec<Point<Real>> = vertices.iter().map(|p| point![p.x, p.y, p.z]).collect();
        let i: Vec<[u32; 3]> = indices.to_vec();
        let coll = ColliderBuilder::trimesh(v, i)
            .collision_groups(InteractionGroups::new(groups.bits(), u32::MAX))
            .friction(0.9)
            .build();
        self.colliders.insert_with_parent(coll, h, &mut self.bodies);
        self.tag_body(h, ActorKind::Static)
    }

    pub fn add_dynamic_box(&mut self, pos: Vec3, half: Vec3, mass: f32, groups: Layers) -> BodyId {
        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![pos.x, pos.y, pos.z])
            .linvel(vector![0.0, 0.0, 0.0])
            .ccd_enabled(true)
            .build();
        let h = self.bodies.insert(rb);
        let coll = ColliderBuilder::cuboid(half.x, half.y, half.z)
            .mass(mass)
            .collision_groups(InteractionGroups::new(groups.bits(), u32::MAX))
            .restitution(0.1)
            .friction(0.8)
            .build();
        self.colliders.insert_with_parent(coll, h, &mut self.bodies);
        self.tag_body(h, ActorKind::Dynamic)
    }

    pub fn add_destructible_box(&mut self, pos: Vec3, half: Vec3, mass: f32, health: f32, break_impulse: f32) -> BodyId {
        let id = self.add_dynamic_box(pos, half, mass, Layers::DESTRUCT | Layers::DEFAULT);
        self.destructibles.insert(id, Destructible {
            health, break_impulse, fragments: 6
        });
        id
    }

    pub fn add_joint_fixed(&mut self, a: BodyId, b: BodyId, local_a: Vec3, local_b: Vec3) {
        let ha = self.handle_of(a).unwrap();
        let hb = self.handle_of(b).unwrap();
        let joint = FixedJointBuilder::new()
            .local_anchor1(point![local_a.x, local_a.y, local_a.z])
            .local_anchor2(point![local_b.x, local_b.y, local_b.z])
            .build();
        self.joints.insert(ha, hb, joint, true);
    }

    // Kinematic character (Rapier’s built-in KCC)
    pub fn add_character(&mut self, pos: Vec3, half: Vec3) -> BodyId {
        let rb = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![pos.x, pos.y, pos.z])
            .build();
        let h = self.bodies.insert(rb);
        let coll = ColliderBuilder::capsule_y(half.y, half.x.max(half.z))
            .collision_groups(InteractionGroups::new(Layers::CHARACTER.bits(), u32::MAX))
            .friction(0.6)
            .build();
        self.colliders.insert_with_parent(coll, h, &mut self.bodies);

        let id = self.tag_body(h, ActorKind::Character);
        self.char_map.insert(id, CharacterController {
            handle: KinematicCharacterController {
                offset: CharacterLength::Absolute(0.02),
                slide: true,
                autostep: Some(CharacterAutostep { max_height: CharacterLength::Absolute(0.4), min_width: CharacterLength::Absolute(0.2), include_dynamic_bodies: false }),
                max_slope_climb_angle: std::f32::consts::FRAC_PI_4, // 45 degrees
                min_slope_slide_angle: 50f32.to_radians(),
                up: vector![0.0, 1.0, 0.0],
                ..Default::default()
            },
            state: CharState::Grounded,
            max_climb_angle_deg: 70.0,
        });
        id
    }

    pub fn add_water_aabb(&mut self, min: Vec3, max: Vec3, density: f32, linear_damp: f32) {
        self.water.push(WaterVolume { min, max, density, linear_damp });
    }

    pub fn set_wind(&mut self, dir: Vec3, strength: f32) { self.wind = Some(Wind { dir, strength }); }

    // ---------- Queries ----------
    pub fn handle_of(&self, id: BodyId) -> Option<RigidBodyHandle> {
        self.body_ids.iter().find_map(|(h, bid)| if *bid == id { Some(*h) } else { None })
    }
    pub fn id_of(&self, handle: RigidBodyHandle) -> Option<BodyId> { self.body_ids.get(&handle).copied() }

    pub fn body_transform(&self, id: BodyId) -> Option<Mat4> {
        let h = self.handle_of(id)?;
        let rb = self.bodies.get(h)?;
        let iso = rb.position();
        let (px,py,pz) = (iso.translation.x, iso.translation.y, iso.translation.z);
        let rot = iso.rotation;
        let q = glam::Quat::from_xyzw(rot.i, rot.j, rot.key, rot.w);
        Some(Mat4::from_rotation_translation(q, vec3(px,py,pz)))
    }

    // ---------- Character update ----------
    pub fn control_character(&mut self, id: BodyId, desired_move: Vec3, dt: f32, climb_pressed: bool) {
        let Some(ctrl) = self.char_map.get_mut(&id) else { return; };
        let h = self.handle_of(id).unwrap();
        let rb = self.bodies.get(h).unwrap();
        let pos = rb.position();

        // Detect water
        let mut in_water = false;
        for w in &self.water {
            let p = vec3(pos.translation.x, pos.translation.y, pos.translation.z);
            if p.x >= w.min.x && p.x <= w.max.x &&
               p.y >= w.min.y && p.y <= w.max.y &&
               p.z >= w.min.z && p.z <= w.max.z {
                in_water = true; break;
            }
        }

        // Detect climbable surface (simple ray forward)
        let mut near_climb = false;
        if climb_pressed {
            // cast a short ray forward from chest height
            let fwd = desired_move.normalize_or_zero();
            let origin = point![pos.translation.x, pos.translation.y + 0.9, pos.translation.z];
            let dir = vector![fwd.x, fwd.y, fwd.z];
            if let Some((_, toi)) = self.narrow_phase.cast_ray(
                &self.bodies, &self.colliders,
                &Ray::new(origin, dir),
                0.7,
                true,
                QueryFilter::default(),
            ) {
                // if hit, consider climbable
                near_climb = true;
            }
        }

        // State transitions
        ctrl.state = if in_water { CharState::Swimming }
            else if near_climb && climb_pressed { CharState::Climbing }
            else { CharState::Grounded };

        let mut kcc_move = vector![desired_move.x, desired_move.y, desired_move.z];
        match ctrl.state {
            CharState::Grounded => {
                kcc_move.y -= 9.8 * dt; // gravity
            },
            CharState::Climbing => {
                // clamp to climb plane: ignore gravity, allow vertical movement
                kcc_move.y = desired_move.y.max(-1.0).min(1.0);
            },
            CharState::Swimming => {
                // reduced gravity; slight upward bias to simulate buoyancy
                kcc_move.y += 3.0 * dt;
            }
        }

        let mut collisions = vec![];
        let (tx, _rx) = crossbeam::channel::unbounded();
        let events = ChannelEventCollector::new(tx);

        let kinematics = ctrl.handle.move_shape(
            dt,
            &self.bodies,
            &self.colliders,
            &self.integration.prediction_distance,
            &ColliderSet::default(), // no custom
            &moveshape::ShapeCastOptions::default(),
            kcc_move,
            ctrl.handle.up,
            |hit| {
                collisions.push(hit);
                true
            },
        );

        // Actually update the kinematic body's position:
        if let Some(new_pos) = kinematics.effective_translation {
            let mut rb = self.bodies.get_mut(h).unwrap();
            let mut p = rb.position().clone();
            p.translation.x += new_pos.x;
            p.translation.y += new_pos.y;
            p.translation.z += new_pos.z;
            rb.set_position(p, true);
        }
    }

    // ---------- Environment forces ----------
    fn apply_environment_forces(&mut self, _dt: f32) {
        // Wind
        if let Some(w) = &self.wind {
            for (h, kind) in self.body_kinds.clone() {
                if matches!(kind, ActorKind::Dynamic | ActorKind::RagdollLink) {
                    if let Some(rb) = self.bodies.get_mut(*h) {
                        let f = vector![w.dir.x * w.strength, w.dir.y * w.strength, w.dir.z * w.strength];
                        rb.apply_force(f, true);
                    }
                }
            }
        }
        // Water buoyancy (very simplified: if center inside AABB, push up + add damping)
        for vol in &self.water {
            for (h, kind) in self.body_kinds.clone() {
                if matches!(kind, ActorKind::Dynamic | ActorKind::RagdollLink) {
                    let rb = self.bodies.get_mut(*h).unwrap();
                    let p = rb.position().translation;
                    if p.x >= vol.min.x && p.x <= vol.max.x &&
                       p.y >= vol.min.y && p.y <= vol.max.y &&
                       p.z >= vol.min.z && p.z <= vol.max.z {
                        let vol_est = 1.0; // assume 1 m^3 per body for demo
                        let buoy = vol.density * 9.81 * vol_est;
                        rb.apply_force(vector![0.0, buoy as f32, 0.0], true);
                        let lv = rb.linvel();
                        rb.apply_force(-lv * vol.linear_damp, true);
                    }
                }
            }
        }
    }

    // ---------- Destructibles ----------
    fn process_destructible_hits(&mut self) {
        // Iterate contacts; if impulses exceed threshold, damage destructibles
        let mut to_break = vec![];
        for contact_pair in self.narrow_phase.contact_pairs() {
            let Some(impulses) = contact_pair.data.total_impulse else { continue; }
            let impulse_mag = (impulses).norm();

            let a = contact_pair.collider1();
            let b = contact_pair.collider2();
            let ha = self.colliders.get(a).unwrap().parent().unwrap();
            let hb = self.colliders.get(b).unwrap().parent().unwrap();

            for h in [ha, hb] {
                if let Some(id) = self.id_of(h) {
                    if let Some(d) = self.destructibles.get_mut(&id) {
                        if impulse_mag as f32 >= d.break_impulse {
                            d.health -= impulse_mag as f32 * 0.05;
                            if d.health <= 0.0 { to_break.push(id); }
                        }
                    }
                }
            }
        }

        for id in to_break {
            self.break_destructible(id);
        }
    }

    pub fn break_destructible(&mut self, id: BodyId) {
        let Some(h) = self.handle_of(id) else { return; };
        if let Some(rb) = self.bodies.get(h) {
            let p = rb.position().translation;
            let pos = vec3(p.x, p.y, p.z);
            // remove original
            self.bodies.remove(
                h, &mut self.island_mgr, &mut self.colliders, &mut self.joints, &mut self.multibody_joints, true
            );
            self.body_ids.remove(&h);
            self.body_kinds.remove(&h);
            self.destructibles.remove(&id);

            // spawn simple fragments (cubes)
            let n = 6;
            for i in 0..n {
                let jitter = vec3(
                    (rand::random::<f32>() - 0.5) * 0.4,
                    (rand::random::<f32>() - 0.5) * 0.4 + 0.5,
                    (rand::random::<f32>() - 0.5) * 0.4
                );
                let fid = self.add_dynamic_box(pos + jitter, vec3(0.15,0.15,0.15), 1.0, Layers::DEFAULT);
                // give outward impulse
                if let Some(h) = self.handle_of(fid) {
                    if let Some(rb) = self.bodies.get_mut(h) {
                        let impulse = vector![jitter.x*2.0, jitter.y*2.0, jitter.z*2.0];
                        rb.apply_impulse(impulse, true);
                    }
                }
            }
        }
    }

    // ---------- Ragdoll ----------
    pub fn spawn_ragdoll_simple(&mut self, pos: Vec3) -> Ragdoll {
        // Very simple chain: pelvis -> spine -> head; two arms; two legs
        let mut parts = vec![];
        let mut add_capsule = |p: Vec3, half: f32| {
            let rb = RigidBodyBuilder::dynamic().translation(vector![p.x, p.y, p.z]).build();
            let h = self.bodies.insert(rb);
            let coll = ColliderBuilder::capsule_y(half, 0.12).friction(0.8).build();
            self.colliders.insert_with_parent(coll, h, &mut self.bodies);
            self.tag_body(h, ActorKind::RagdollLink);
            parts.push(h);
        };

        // pelvis/spine/head
        add_capsule(pos + vec3(0.0, 1.0, 0.0), 0.18);
        add_capsule(pos + vec3(0.0, 1.5, 0.0), 0.20);
        add_capsule(pos + vec3(0.0, 2.0, 0.0), 0.16);

        // arms
        add_capsule(pos + vec3(-0.4, 1.5, 0.0), 0.18);
        add_capsule(pos + vec3( 0.4, 1.5, 0.0), 0.18);

        // legs
        add_capsule(pos + vec3(-0.2, 0.5, 0.0), 0.24);
        add_capsule(pos + vec3( 0.2, 0.5, 0.0), 0.24);

        // simple spherical joints
        let mut joints = vec![];
        let j = |a: RigidBodyHandle, b: RigidBodyHandle, la: Vec3, lb: Vec3| {
            let j = SphericalJointBuilder::new()
                .local_anchor1(point![la.x, la.y, la.z])
                .local_anchor2(point![lb.x, lb.y, lb.z])
                .build();
            self.joints.insert(a, b, j, true)
        };

        // pelvis->spine->head
        joints.push(j(parts[0], parts[1], vec3(0.0, 0.2, 0.0), vec3(0.0,-0.2, 0.0)));
        joints.push(j(parts[1], parts[2], vec3(0.0, 0.2, 0.0), vec3(0.0,-0.2, 0.0)));
        // shoulders
        joints.push(j(parts[1], parts[3], vec3(-0.2, 0.1, 0.0), vec3(0.0,-0.2, 0.0)));
        joints.push(j(parts[1], parts[4], vec3( 0.2, 0.1, 0.0), vec3(0.0,-0.2, 0.0)));
        // hips
        joints.push(j(parts[0], parts[5], vec3(-0.1, -0.2, 0.0), vec3(0.0, 0.2, 0.0)));
        joints.push(j(parts[0], parts[6], vec3( 0.1, -0.2, 0.0), vec3(0.0, 0.2, 0.0)));

        Ragdoll { part_handles: parts, joints }
    }

    // ---------- Internals ----------
    fn tag_body(&mut self, h: RigidBodyHandle, kind: ActorKind) -> BodyId {
        let id = self.alloc_id();
        self.body_ids.insert(h, id);
        self.body_kinds.insert(h, kind);
        id
    }
}
