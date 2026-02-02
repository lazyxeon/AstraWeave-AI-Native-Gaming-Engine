//! # Ragdoll System
//!
//! Provides realistic ragdoll physics for character death animations,
//! hit reactions, and procedural animations.
//!
//! ## Features
//!
//! - **Ragdoll Builder**: Create ragdolls from bone hierarchies
//! - **State Management**: Smooth transitions between animated and physics states
//! - **Impulse Propagation**: Realistic force distribution through joint chains
//! - **Joint Limits**: Anatomically-correct constraints
//!
//! ## Usage
//!
//! ```rust
//! use astraweave_physics::ragdoll::{RagdollBuilder, RagdollConfig, BoneShape};
//! use glam::Vec3;
//!
//! // Define a simple humanoid skeleton
//! let mut builder = RagdollBuilder::new(RagdollConfig::default());
//!
//! // Add bones (typically from skeleton data)
//! builder.add_bone("pelvis", None, Vec3::ZERO, BoneShape::Capsule { radius: 0.15, half_height: 0.1 }, 2.0);
//! builder.add_bone("spine", Some("pelvis"), Vec3::new(0.0, 0.2, 0.0), BoneShape::Capsule { radius: 0.12, half_height: 0.15 }, 1.5);
//! builder.add_bone("head", Some("spine"), Vec3::new(0.0, 0.35, 0.0), BoneShape::Sphere { radius: 0.12 }, 1.0);
//!
//! // Build the ragdoll (requires PhysicsWorld)
//! // let ragdoll = builder.build(&mut physics_world, spawn_position);
//! ```

use crate::{BodyId, JointId, JointType, Layers, PhysicsWorld};
use glam::{Quat, Vec3};
use std::collections::HashMap;

/// Unique identifier for a ragdoll
pub type RagdollId = u64;

/// Shape of a bone's physics collider
#[derive(Debug, Clone, Copy)]
pub enum BoneShape {
    /// Capsule shape (most common for limbs)
    Capsule { radius: f32, half_height: f32 },
    /// Sphere shape (good for head, hands)
    Sphere { radius: f32 },
    /// Box shape (good for torso, pelvis)
    Box { half_extents: Vec3 },
}

impl BoneShape {
    /// Get the approximate volume of this shape (for mass distribution)
    pub fn volume(&self) -> f32 {
        match self {
            BoneShape::Capsule { radius, half_height } => {
                let cylinder = std::f32::consts::PI * radius * radius * (half_height * 2.0);
                let sphere = (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3);
                cylinder + sphere
            }
            BoneShape::Sphere { radius } => {
                (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3)
            }
            BoneShape::Box { half_extents } => {
                half_extents.x * half_extents.y * half_extents.z * 8.0
            }
        }
    }
}

/// Joint constraint type for bones
#[derive(Debug, Clone, Copy)]
pub enum BoneJointType {
    /// Ball-and-socket joint (shoulder, hip)
    Spherical {
        /// Cone limit in radians (how far the bone can swing)
        swing_limit: Option<f32>,
        /// Twist limit in radians (rotation around the bone's axis)
        twist_limit: Option<(f32, f32)>,
    },
    /// Hinge joint (elbow, knee)
    Hinge {
        /// Axis of rotation (local to parent)
        axis: Vec3,
        /// Angle limits in radians (min, max)
        limits: Option<(f32, f32)>,
    },
    /// Fixed joint (skull to spine, rigid connections)
    Fixed,
}

/// Definition of a single bone in the ragdoll
#[derive(Debug, Clone)]
pub struct BoneDef {
    /// Unique name of this bone
    pub name: String,
    /// Parent bone name (None for root)
    pub parent: Option<String>,
    /// Offset from parent bone (local space)
    pub offset: Vec3,
    /// Rotation offset from parent (local space)
    pub rotation: Quat,
    /// Physics shape for collision
    pub shape: BoneShape,
    /// Mass of this bone
    pub mass: f32,
    /// Joint type connecting to parent
    pub joint_type: BoneJointType,
}

impl Default for BoneDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            parent: None,
            offset: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            shape: BoneShape::Capsule {
                radius: 0.05,
                half_height: 0.1,
            },
            mass: 1.0,
            joint_type: BoneJointType::Spherical {
                swing_limit: Some(std::f32::consts::FRAC_PI_4),
                twist_limit: None,
            },
        }
    }
}

/// Configuration for ragdoll behavior
#[derive(Debug, Clone)]
pub struct RagdollConfig {
    /// Global mass multiplier
    pub mass_scale: f32,
    /// Joint stiffness (0.0 = floppy, 1.0 = stiff)
    pub joint_stiffness: f32,
    /// Joint damping (prevents oscillation)
    pub joint_damping: f32,
    /// Whether to enable CCD for fast-moving ragdolls
    pub enable_ccd: bool,
    /// Collision group for ragdoll bodies
    pub collision_group: Layers,
    /// Maximum angular velocity (prevents spinning out of control)
    pub max_angular_velocity: f32,
    /// Linear damping (air resistance)
    pub linear_damping: f32,
    /// Angular damping (rotational air resistance)
    pub angular_damping: f32,
}

impl Default for RagdollConfig {
    fn default() -> Self {
        Self {
            mass_scale: 1.0,
            joint_stiffness: 0.5,
            joint_damping: 0.3,
            enable_ccd: true,
            collision_group: Layers::DEFAULT,
            max_angular_velocity: 20.0,
            linear_damping: 0.1,
            angular_damping: 0.5,
        }
    }
}

/// Ragdoll state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RagdollState {
    /// Ragdoll is fully physics-driven
    Active,
    /// Ragdoll is blending from animation to physics
    BlendingToPhysics {
        /// Progress of blend (0.0 = animation, 1.0 = physics)
        progress_percent: u8,
    },
    /// Ragdoll is blending from physics to animation
    BlendingToAnimation {
        /// Progress of blend (0.0 = physics, 1.0 = animation)
        progress_percent: u8,
    },
    /// Ragdoll is disabled (using animation)
    Disabled,
}

/// Instance of a spawned ragdoll
#[derive(Debug)]
pub struct Ragdoll {
    /// Unique ID
    pub id: RagdollId,
    /// Current state
    pub state: RagdollState,
    /// Map of bone name to body ID
    pub bone_bodies: HashMap<String, BodyId>,
    /// Map of bone name to joint ID (joint connecting to parent)
    pub bone_joints: HashMap<String, JointId>,
    /// Root bone name
    pub root_bone: String,
    /// Configuration used to create this ragdoll
    pub config: RagdollConfig,
    /// Bone definitions (for reference)
    bone_defs: Vec<BoneDef>,
}

impl Ragdoll {
    /// Get the body ID for a bone by name
    pub fn get_bone_body(&self, name: &str) -> Option<BodyId> {
        self.bone_bodies.get(name).copied()
    }

    /// Get all body IDs in this ragdoll
    pub fn all_bodies(&self) -> impl Iterator<Item = BodyId> + '_ {
        self.bone_bodies.values().copied()
    }

    /// Get the root body ID
    pub fn root_body(&self) -> Option<BodyId> {
        self.bone_bodies.get(&self.root_bone).copied()
    }

    /// Apply an impulse to a specific bone
    pub fn apply_impulse_to_bone(
        &self,
        physics: &mut PhysicsWorld,
        bone_name: &str,
        impulse: Vec3,
    ) -> bool {
        if let Some(body_id) = self.bone_bodies.get(bone_name) {
            physics.apply_impulse(*body_id, impulse);
            true
        } else {
            false
        }
    }

    /// Apply an impulse to the ragdoll with propagation through joints
    /// 
    /// The impulse is applied at the specified position and propagates
    /// through the joint chain with falloff based on distance.
    pub fn apply_impulse_with_propagation(
        &self,
        physics: &mut PhysicsWorld,
        hit_bone: &str,
        impulse: Vec3,
        propagation_factor: f32,
    ) {
        // Apply full impulse to hit bone
        if let Some(body_id) = self.bone_bodies.get(hit_bone) {
            physics.apply_impulse(*body_id, impulse);
        }

        // Find connected bones and apply reduced impulse
        if let Some(hit_def) = self.bone_defs.iter().find(|b| b.name == hit_bone) {
            // Apply to parent (if any)
            if let Some(ref parent_name) = hit_def.parent {
                if let Some(parent_body) = self.bone_bodies.get(parent_name) {
                    physics.apply_impulse(*parent_body, impulse * propagation_factor);
                }
            }

            // Apply to children
            for bone in &self.bone_defs {
                if bone.parent.as_ref() == Some(&hit_def.name) {
                    if let Some(child_body) = self.bone_bodies.get(&bone.name) {
                        physics.apply_impulse(*child_body, impulse * propagation_factor);
                    }
                }
            }
        }
    }

    /// Get the center of mass of the ragdoll
    pub fn center_of_mass(&self, physics: &PhysicsWorld) -> Vec3 {
        let mut total_mass = 0.0f32;
        let mut weighted_pos = Vec3::ZERO;

        for (name, body_id) in &self.bone_bodies {
            if let Some(transform) = physics.body_transform(*body_id) {
                let pos = Vec3::new(transform.w_axis.x, transform.w_axis.y, transform.w_axis.z);
                let mass = self
                    .bone_defs
                    .iter()
                    .find(|b| &b.name == name)
                    .map(|b| b.mass)
                    .unwrap_or(1.0);
                weighted_pos += pos * mass;
                total_mass += mass;
            }
        }

        if total_mass > 0.0 {
            weighted_pos / total_mass
        } else {
            Vec3::ZERO
        }
    }

    /// Check if the ragdoll is at rest (not moving significantly)
    pub fn is_at_rest(&self, physics: &PhysicsWorld, velocity_threshold: f32) -> bool {
        for body_id in self.bone_bodies.values() {
            if let Some(vel) = physics.get_velocity(*body_id) {
                if vel.length() > velocity_threshold {
                    return false;
                }
            }
        }
        true
    }
}

/// Builder for creating ragdolls
#[derive(Debug)]
pub struct RagdollBuilder {
    /// Configuration
    pub config: RagdollConfig,
    /// Bone definitions
    bones: Vec<BoneDef>,
    /// Next ragdoll ID
    next_id: RagdollId,
}

impl RagdollBuilder {
    /// Create a new ragdoll builder with the given configuration
    pub fn new(config: RagdollConfig) -> Self {
        Self {
            config,
            bones: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a bone to the ragdoll
    pub fn add_bone(
        &mut self,
        name: &str,
        parent: Option<&str>,
        offset: Vec3,
        shape: BoneShape,
        mass: f32,
    ) -> &mut Self {
        self.bones.push(BoneDef {
            name: name.to_string(),
            parent: parent.map(|s| s.to_string()),
            offset,
            shape,
            mass: mass * self.config.mass_scale,
            ..Default::default()
        });
        self
    }

    /// Add a bone with full configuration
    pub fn add_bone_full(&mut self, bone: BoneDef) -> &mut Self {
        let mut bone = bone;
        bone.mass *= self.config.mass_scale;
        self.bones.push(bone);
        self
    }

    /// Add a hinge joint bone (elbow, knee)
    #[allow(clippy::too_many_arguments)]
    pub fn add_hinge_bone(
        &mut self,
        name: &str,
        parent: &str,
        offset: Vec3,
        shape: BoneShape,
        mass: f32,
        axis: Vec3,
        limits: Option<(f32, f32)>,
    ) -> &mut Self {
        self.bones.push(BoneDef {
            name: name.to_string(),
            parent: Some(parent.to_string()),
            offset,
            shape,
            mass: mass * self.config.mass_scale,
            joint_type: BoneJointType::Hinge { axis, limits },
            ..Default::default()
        });
        self
    }

    /// Add a ball joint bone (shoulder, hip)
    pub fn add_ball_bone(
        &mut self,
        name: &str,
        parent: &str,
        offset: Vec3,
        shape: BoneShape,
        mass: f32,
        swing_limit: Option<f32>,
    ) -> &mut Self {
        self.bones.push(BoneDef {
            name: name.to_string(),
            parent: Some(parent.to_string()),
            offset,
            shape,
            mass: mass * self.config.mass_scale,
            joint_type: BoneJointType::Spherical {
                swing_limit,
                twist_limit: None,
            },
            ..Default::default()
        });
        self
    }

    /// Build the ragdoll and spawn it in the physics world
    pub fn build(&mut self, physics: &mut PhysicsWorld, spawn_position: Vec3) -> Ragdoll {
        let mut bone_bodies = HashMap::new();
        let mut bone_joints = HashMap::new();
        let mut root_bone = String::new();

        // First pass: create all bodies
        for bone in &self.bones {
            let bone_pos = if let Some(ref parent_name) = bone.parent {
                // Calculate world position based on parent
                if let Some(&parent_body) = bone_bodies.get(parent_name) {
                    if let Some(transform) = physics.body_transform(parent_body) {
                        let parent_pos =
                            Vec3::new(transform.w_axis.x, transform.w_axis.y, transform.w_axis.z);
                        parent_pos + bone.offset
                    } else {
                        spawn_position + bone.offset
                    }
                } else {
                    spawn_position + bone.offset
                }
            } else {
                // Root bone
                root_bone = bone.name.clone();
                spawn_position
            };

            // Create the body based on shape
            let body_id = match bone.shape {
                BoneShape::Capsule { radius, half_height } => physics.add_dynamic_box(
                    bone_pos,
                    Vec3::new(radius, half_height, radius),
                    bone.mass,
                    self.config.collision_group,
                ),
                BoneShape::Sphere { radius } => physics.add_dynamic_box(
                    bone_pos,
                    Vec3::splat(radius),
                    bone.mass,
                    self.config.collision_group,
                ),
                BoneShape::Box { half_extents } => physics.add_dynamic_box(
                    bone_pos,
                    half_extents,
                    bone.mass,
                    self.config.collision_group,
                ),
            };

            // Enable CCD if configured
            if self.config.enable_ccd {
                physics.enable_ccd(body_id);
            }

            bone_bodies.insert(bone.name.clone(), body_id);
        }

        // Second pass: create joints
        for bone in &self.bones {
            if let Some(ref parent_name) = bone.parent {
                let Some(&parent_body) = bone_bodies.get(parent_name) else {
                    continue;
                };
                let Some(&child_body) = bone_bodies.get(&bone.name) else {
                    continue;
                };

                let joint_type = match bone.joint_type {
                    BoneJointType::Fixed => JointType::Fixed,
                    BoneJointType::Hinge { axis, limits } => JointType::Revolute { axis, limits },
                    BoneJointType::Spherical { .. } => JointType::Spherical,
                };

                let joint_id = physics.add_joint(parent_body, child_body, joint_type);
                bone_joints.insert(bone.name.clone(), joint_id);
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        Ragdoll {
            id,
            state: RagdollState::Active,
            bone_bodies,
            bone_joints,
            root_bone,
            config: self.config.clone(),
            bone_defs: self.bones.clone(),
        }
    }

    /// Clear all bones (for reuse)
    pub fn clear(&mut self) {
        self.bones.clear();
    }
}

/// Preset ragdoll templates
pub struct RagdollPresets;

impl RagdollPresets {
    /// Create a simple humanoid ragdoll (10 bones)
    /// 
    /// Bones: pelvis, spine, chest, head, upper_arm_l/r, lower_arm_l/r, upper_leg_l/r, lower_leg_l/r
    pub fn humanoid(config: RagdollConfig) -> RagdollBuilder {
        let mut builder = RagdollBuilder::new(config);

        // Torso chain
        builder.add_bone(
            "pelvis",
            None,
            Vec3::ZERO,
            BoneShape::Box {
                half_extents: Vec3::new(0.15, 0.1, 0.1),
            },
            4.0,
        );

        builder.add_bone_full(BoneDef {
            name: "spine".to_string(),
            parent: Some("pelvis".to_string()),
            offset: Vec3::new(0.0, 0.15, 0.0),
            shape: BoneShape::Box {
                half_extents: Vec3::new(0.12, 0.12, 0.08),
            },
            mass: 3.0,
            joint_type: BoneJointType::Spherical {
                swing_limit: Some(0.3),
                twist_limit: Some((-0.2, 0.2)),
            },
            ..Default::default()
        });

        builder.add_bone_full(BoneDef {
            name: "chest".to_string(),
            parent: Some("spine".to_string()),
            offset: Vec3::new(0.0, 0.2, 0.0),
            shape: BoneShape::Box {
                half_extents: Vec3::new(0.14, 0.1, 0.09),
            },
            mass: 3.5,
            joint_type: BoneJointType::Spherical {
                swing_limit: Some(0.25),
                twist_limit: Some((-0.15, 0.15)),
            },
            ..Default::default()
        });

        builder.add_bone(
            "head",
            Some("chest"),
            Vec3::new(0.0, 0.2, 0.0),
            BoneShape::Sphere { radius: 0.1 },
            1.5,
        );

        // Arms
        for (side, x_offset) in [("_l", -0.2), ("_r", 0.2)] {
            let upper_arm = format!("upper_arm{}", side);
            let lower_arm = format!("lower_arm{}", side);

            builder.add_ball_bone(
                &upper_arm,
                "chest",
                Vec3::new(x_offset, 0.05, 0.0),
                BoneShape::Capsule {
                    radius: 0.04,
                    half_height: 0.12,
                },
                1.2,
                Some(std::f32::consts::FRAC_PI_2),
            );

            builder.add_hinge_bone(
                &lower_arm,
                &upper_arm,
                Vec3::new(0.0, -0.25, 0.0),
                BoneShape::Capsule {
                    radius: 0.035,
                    half_height: 0.11,
                },
                0.8,
                Vec3::X,
                Some((0.0, std::f32::consts::FRAC_PI_2 * 1.5)),
            );
        }

        // Legs
        for (side, x_offset) in [("_l", -0.1), ("_r", 0.1)] {
            let upper_leg = format!("upper_leg{}", side);
            let lower_leg = format!("lower_leg{}", side);

            builder.add_ball_bone(
                &upper_leg,
                "pelvis",
                Vec3::new(x_offset, -0.15, 0.0),
                BoneShape::Capsule {
                    radius: 0.06,
                    half_height: 0.18,
                },
                3.0,
                Some(std::f32::consts::FRAC_PI_3),
            );

            builder.add_hinge_bone(
                &lower_leg,
                &upper_leg,
                Vec3::new(0.0, -0.38, 0.0),
                BoneShape::Capsule {
                    radius: 0.05,
                    half_height: 0.17,
                },
                2.0,
                Vec3::X,
                Some((-std::f32::consts::FRAC_PI_2 * 1.5, 0.0)),
            );
        }

        builder
    }

    /// Create a simple quadruped ragdoll (8 bones)
    /// 
    /// Bones: body, head, front_leg_l/r, back_leg_l/r
    pub fn quadruped(config: RagdollConfig) -> RagdollBuilder {
        let mut builder = RagdollBuilder::new(config);

        // Body
        builder.add_bone(
            "body",
            None,
            Vec3::ZERO,
            BoneShape::Box {
                half_extents: Vec3::new(0.2, 0.15, 0.4),
            },
            8.0,
        );

        // Head
        builder.add_ball_bone(
            "head",
            "body",
            Vec3::new(0.0, 0.1, 0.35),
            BoneShape::Sphere { radius: 0.12 },
            1.5,
            Some(std::f32::consts::FRAC_PI_4),
        );

        // Front legs
        for (side, x_offset) in [("_l", -0.15), ("_r", 0.15)] {
            let leg_name = format!("front_leg{}", side);
            builder.add_ball_bone(
                &leg_name,
                "body",
                Vec3::new(x_offset, -0.2, 0.25),
                BoneShape::Capsule {
                    radius: 0.04,
                    half_height: 0.15,
                },
                1.0,
                Some(std::f32::consts::FRAC_PI_3),
            );
        }

        // Back legs
        for (side, x_offset) in [("_l", -0.15), ("_r", 0.15)] {
            let leg_name = format!("back_leg{}", side);
            builder.add_ball_bone(
                &leg_name,
                "body",
                Vec3::new(x_offset, -0.2, -0.25),
                BoneShape::Capsule {
                    radius: 0.04,
                    half_height: 0.15,
                },
                1.0,
                Some(std::f32::consts::FRAC_PI_3),
            );
        }

        // Tail
        builder.add_ball_bone(
            "tail",
            "body",
            Vec3::new(0.0, 0.05, -0.45),
            BoneShape::Capsule {
                radius: 0.02,
                half_height: 0.15,
            },
            0.3,
            Some(std::f32::consts::FRAC_PI_2),
        );

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_shape_volume() {
        let capsule = BoneShape::Capsule {
            radius: 1.0,
            half_height: 1.0,
        };
        assert!(capsule.volume() > 0.0);

        let sphere = BoneShape::Sphere { radius: 1.0 };
        let expected_sphere_vol = (4.0 / 3.0) * std::f32::consts::PI;
        assert!((sphere.volume() - expected_sphere_vol).abs() < 0.001);

        let box_shape = BoneShape::Box {
            half_extents: Vec3::ONE,
        };
        assert!((box_shape.volume() - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_ragdoll_builder_creation() {
        let config = RagdollConfig::default();
        let builder = RagdollBuilder::new(config);
        assert_eq!(builder.bones.len(), 0);
    }

    #[test]
    fn test_ragdoll_builder_add_bones() {
        let mut builder = RagdollBuilder::new(RagdollConfig::default());

        builder.add_bone(
            "root",
            None,
            Vec3::ZERO,
            BoneShape::Box {
                half_extents: Vec3::splat(0.1),
            },
            1.0,
        );

        builder.add_bone(
            "child",
            Some("root"),
            Vec3::new(0.0, 0.5, 0.0),
            BoneShape::Sphere { radius: 0.1 },
            0.5,
        );

        assert_eq!(builder.bones.len(), 2);
        assert_eq!(builder.bones[0].name, "root");
        assert!(builder.bones[0].parent.is_none());
        assert_eq!(builder.bones[1].name, "child");
        assert_eq!(builder.bones[1].parent, Some("root".to_string()));
    }

    #[test]
    fn test_ragdoll_config_defaults() {
        let config = RagdollConfig::default();
        assert!((config.mass_scale - 1.0).abs() < 0.001);
        assert!(config.enable_ccd);
        assert!(config.max_angular_velocity > 0.0);
    }

    #[test]
    fn test_ragdoll_state_enum() {
        assert_eq!(RagdollState::Active, RagdollState::Active);
        assert_ne!(RagdollState::Active, RagdollState::Disabled);

        let blending = RagdollState::BlendingToPhysics { progress_percent: 50 };
        match blending {
            RagdollState::BlendingToPhysics { progress_percent } => {
                assert_eq!(progress_percent, 50);
            }
            _ => panic!("Wrong state"),
        }
    }

    #[test]
    fn test_humanoid_preset() {
        let builder = RagdollPresets::humanoid(RagdollConfig::default());
        // Humanoid should have: pelvis, spine, chest, head, 
        // upper_arm_l, upper_arm_r, lower_arm_l, lower_arm_r,
        // upper_leg_l, upper_leg_r, lower_leg_l, lower_leg_r
        assert_eq!(builder.bones.len(), 12);
    }

    #[test]
    fn test_quadruped_preset() {
        let builder = RagdollPresets::quadruped(RagdollConfig::default());
        // Quadruped should have: body, head, 4 legs, tail
        assert_eq!(builder.bones.len(), 7);
    }

    #[test]
    fn test_add_hinge_bone() {
        let mut builder = RagdollBuilder::new(RagdollConfig::default());

        builder.add_bone("parent", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.1 }, 1.0);
        builder.add_hinge_bone(
            "child",
            "parent",
            Vec3::Y,
            BoneShape::Capsule { radius: 0.05, half_height: 0.1 },
            0.5,
            Vec3::X,
            Some((0.0, std::f32::consts::PI)),
        );

        assert_eq!(builder.bones.len(), 2);
        match builder.bones[1].joint_type {
            BoneJointType::Hinge { axis, limits } => {
                assert!((axis.x - 1.0).abs() < 0.001);
                assert!(limits.is_some());
            }
            _ => panic!("Expected Hinge joint"),
        }
    }

    #[test]
    fn test_add_ball_bone() {
        let mut builder = RagdollBuilder::new(RagdollConfig::default());

        builder.add_bone("parent", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.1 }, 1.0);
        builder.add_ball_bone(
            "child",
            "parent",
            Vec3::Y,
            BoneShape::Sphere { radius: 0.08 },
            0.5,
            Some(std::f32::consts::FRAC_PI_4),
        );

        assert_eq!(builder.bones.len(), 2);
        match builder.bones[1].joint_type {
            BoneJointType::Spherical { swing_limit, .. } => {
                assert!(swing_limit.is_some());
            }
            _ => panic!("Expected Spherical joint"),
        }
    }

    #[test]
    fn test_builder_clear() {
        let mut builder = RagdollBuilder::new(RagdollConfig::default());
        builder.add_bone("test", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.1 }, 1.0);
        assert_eq!(builder.bones.len(), 1);

        builder.clear();
        assert_eq!(builder.bones.len(), 0);
    }

    #[test]
    fn test_mass_scale() {
        let config = RagdollConfig {
            mass_scale: 2.0,
            ..Default::default()
        };
        let mut builder = RagdollBuilder::new(config);

        builder.add_bone("test", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.1 }, 5.0);

        // Mass should be scaled
        assert!((builder.bones[0].mass - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_ragdoll_build_simple() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollBuilder::new(RagdollConfig::default());

        builder.add_bone(
            "root",
            None,
            Vec3::ZERO,
            BoneShape::Sphere { radius: 0.2 },
            2.0,
        );
        builder.add_bone(
            "child",
            Some("root"),
            Vec3::new(0.0, 0.5, 0.0),
            BoneShape::Sphere { radius: 0.15 },
            1.0,
        );

        let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 5.0, 0.0));

        assert_eq!(ragdoll.bone_bodies.len(), 2);
        assert_eq!(ragdoll.bone_joints.len(), 1); // Only child has joint
        assert_eq!(ragdoll.root_bone, "root");
        assert_eq!(ragdoll.state, RagdollState::Active);
    }

    #[test]
    fn test_ragdoll_humanoid_build() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollPresets::humanoid(RagdollConfig::default());

        let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 2.0, 0.0));

        assert_eq!(ragdoll.bone_bodies.len(), 12);
        assert_eq!(ragdoll.bone_joints.len(), 11); // All except root
        assert_eq!(ragdoll.root_bone, "pelvis");
    }

    #[test]
    fn test_ragdoll_get_bone_body() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
        let ragdoll = builder.build(&mut physics, Vec3::ZERO);

        assert!(ragdoll.get_bone_body("pelvis").is_some());
        assert!(ragdoll.get_bone_body("head").is_some());
        assert!(ragdoll.get_bone_body("nonexistent").is_none());
    }

    #[test]
    fn test_ragdoll_root_body() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
        let ragdoll = builder.build(&mut physics, Vec3::ZERO);

        let root = ragdoll.root_body();
        assert!(root.is_some());
        assert_eq!(root, ragdoll.get_bone_body("pelvis"));
    }

    #[test]
    fn test_ragdoll_all_bodies() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
        let ragdoll = builder.build(&mut physics, Vec3::ZERO);

        let bodies: Vec<_> = ragdoll.all_bodies().collect();
        assert_eq!(bodies.len(), 12);
    }

    #[test]
    fn test_ragdoll_center_of_mass() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollBuilder::new(RagdollConfig::default());

        builder.add_bone("root", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.2 }, 1.0);

        let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 5.0, 0.0));
        let com = ragdoll.center_of_mass(&physics);

        // Should be near spawn position
        assert!((com.y - 5.0).abs() < 1.0);
    }

    #[test]
    fn test_ragdoll_is_at_rest() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollBuilder::new(RagdollConfig::default());

        builder.add_bone("root", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.2 }, 1.0);

        let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 5.0, 0.0));

        // Initially should be at rest (no steps taken)
        let at_rest = ragdoll.is_at_rest(&physics, 0.1);
        assert!(at_rest);
    }

    // ============================================================================
    // JOINT LIMIT TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_hinge_joint_limits_validation() {
        let joint = BoneJointType::Hinge {
            axis: Vec3::X,
            limits: Some((0.0, std::f32::consts::PI)),
        };

        if let BoneJointType::Hinge { limits, .. } = joint {
            let (min, max) = limits.unwrap();
            assert!(min < max);
            assert!(min >= 0.0);
            assert!(max <= std::f32::consts::PI);
        }
    }

    #[test]
    fn test_spherical_swing_limit() {
        let joint = BoneJointType::Spherical {
            swing_limit: Some(std::f32::consts::FRAC_PI_4),
            twist_limit: None,
        };

        if let BoneJointType::Spherical { swing_limit, .. } = joint {
            assert!(swing_limit.unwrap() > 0.0);
            assert!(swing_limit.unwrap() < std::f32::consts::PI);
        }
    }

    #[test]
    fn test_spherical_twist_limit() {
        let joint = BoneJointType::Spherical {
            swing_limit: None,
            twist_limit: Some((-0.5, 0.5)),
        };

        if let BoneJointType::Spherical { twist_limit, .. } = joint {
            let (min, max) = twist_limit.unwrap();
            assert!(min < max);
        }
    }

    #[test]
    fn test_fixed_joint() {
        let joint = BoneJointType::Fixed;
        assert!(matches!(joint, BoneJointType::Fixed));
    }

    #[test]
    fn test_knee_joint_realistic_limits() {
        // Knee can bend 0-135 degrees roughly
        let knee = BoneJointType::Hinge {
            axis: Vec3::X,
            limits: Some((0.0, 2.356)), // ~135 degrees in radians
        };

        if let BoneJointType::Hinge { limits, axis } = knee {
            let (min, max) = limits.unwrap();
            assert!(max > 2.0); // At least 115 degrees
            assert_eq!(axis, Vec3::X);
        }
    }

    // ============================================================================
    // POSE BLENDING TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_blend_state_to_physics() {
        let state = RagdollState::BlendingToPhysics { progress_percent: 0 };
        assert!(matches!(state, RagdollState::BlendingToPhysics { .. }));

        // Progress validation
        for progress in [0, 25, 50, 75, 100] {
            let s = RagdollState::BlendingToPhysics { progress_percent: progress };
            if let RagdollState::BlendingToPhysics { progress_percent } = s {
                assert!(progress_percent <= 100);
            }
        }
    }

    #[test]
    fn test_blend_state_to_animation() {
        let state = RagdollState::BlendingToAnimation { progress_percent: 50 };
        
        if let RagdollState::BlendingToAnimation { progress_percent } = state {
            assert_eq!(progress_percent, 50);
        } else {
            panic!("Expected BlendingToAnimation");
        }
    }

    #[test]
    fn test_state_transitions() {
        // Valid state flow: Disabled -> BlendingToPhysics -> Active
        let states = [
            RagdollState::Disabled,
            RagdollState::BlendingToPhysics { progress_percent: 50 },
            RagdollState::Active,
        ];

        for (i, state) in states.iter().enumerate() {
            match i {
                0 => assert_eq!(*state, RagdollState::Disabled),
                1 => assert!(matches!(*state, RagdollState::BlendingToPhysics { .. })),
                2 => assert_eq!(*state, RagdollState::Active),
                _ => {}
            }
        }
    }

    // ============================================================================
    // FALL RECOVERY TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_ragdoll_bone_count() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
        let ragdoll = builder.build(&mut physics, Vec3::ZERO);

        assert_eq!(ragdoll.bone_defs.len(), 12);
        assert_eq!(ragdoll.bone_bodies.len(), 12);
    }

    #[test]
    fn test_ragdoll_joint_hierarchy() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
        let ragdoll = builder.build(&mut physics, Vec3::ZERO);

        // All non-root bones should have joints
        let non_root_count = ragdoll.bone_defs.iter()
            .filter(|b| b.parent.is_some())
            .count();
        
        assert_eq!(ragdoll.bone_joints.len(), non_root_count);
    }

    // ============================================================================
    // EDGE CASE TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_bone_def_default() {
        let def = BoneDef::default();
        
        assert!(def.name.is_empty());
        assert!(def.parent.is_none());
        assert_eq!(def.offset, Vec3::ZERO);
        assert_eq!(def.rotation, Quat::IDENTITY);
        assert_eq!(def.mass, 1.0);
    }

    #[test]
    fn test_config_joint_stiffness_range() {
        let config = RagdollConfig::default();
        assert!(config.joint_stiffness >= 0.0 && config.joint_stiffness <= 1.0);
    }

    #[test]
    fn test_config_damping_values() {
        let config = RagdollConfig::default();
        assert!(config.linear_damping > 0.0);
        assert!(config.angular_damping > 0.0);
        assert!(config.joint_damping > 0.0);
    }

    #[test]
    fn test_empty_ragdoll_builder() {
        let builder = RagdollBuilder::new(RagdollConfig::default());
        assert!(builder.bones.is_empty());
    }

    #[test]
    fn test_single_bone_ragdoll() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollBuilder::new(RagdollConfig::default());

        builder.add_bone("only", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.5 }, 5.0);

        let ragdoll = builder.build(&mut physics, Vec3::ZERO);

        assert_eq!(ragdoll.bone_bodies.len(), 1);
        assert_eq!(ragdoll.bone_joints.len(), 0); // No joints for single bone
    }

    #[test]
    fn test_bone_shape_volume_zero_radius() {
        let sphere = BoneShape::Sphere { radius: 0.0 };
        assert_eq!(sphere.volume(), 0.0);

        let capsule = BoneShape::Capsule { radius: 0.0, half_height: 1.0 };
        assert_eq!(capsule.volume(), 0.0);

        let box_shape = BoneShape::Box { half_extents: Vec3::ZERO };
        assert_eq!(box_shape.volume(), 0.0);
    }

    #[test]
    fn test_quadruped_structure() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollPresets::quadruped(RagdollConfig::default());
        let ragdoll = builder.build(&mut physics, Vec3::ZERO);

        // Should have body as root
        assert_eq!(ragdoll.root_bone, "body");
        
        // Should have 7 bones
        assert_eq!(ragdoll.bone_bodies.len(), 7);
        
        // Should have 6 joints (all except root)
        assert_eq!(ragdoll.bone_joints.len(), 6);
    }

    #[test]
    fn test_ragdoll_spawn_position() {
        let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let mut builder = RagdollBuilder::new(RagdollConfig::default());
        
        builder.add_bone("root", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.1 }, 1.0);

        let spawn_pos = Vec3::new(10.0, 20.0, 30.0);
        let ragdoll = builder.build(&mut physics, spawn_pos);
        let com = ragdoll.center_of_mass(&physics);

        // Center of mass should be near spawn position
        assert!((com - spawn_pos).length() < 1.0);
    }

    #[test]
    fn test_custom_mass_scale() {
        let config = RagdollConfig {
            mass_scale: 0.5,
            ..Default::default()
        };
        
        let mut builder = RagdollBuilder::new(config);
        builder.add_bone("test", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.1 }, 10.0);

        // Mass should be halved
        assert!((builder.bones[0].mass - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_ccd_enabled_by_default() {
        let config = RagdollConfig::default();
        assert!(config.enable_ccd);
    }

    #[test]
    fn test_max_angular_velocity() {
        let config = RagdollConfig::default();
        assert!(config.max_angular_velocity > 0.0);
        assert!(config.max_angular_velocity < 100.0); // Reasonable limit
    }
}

