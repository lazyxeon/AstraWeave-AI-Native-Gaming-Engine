//! Tool Sandbox: Validated action verbs and error taxonomy

#[cfg(feature = "profiling")]
use astraweave_profiling::span;

use anyhow::Result;
use astraweave_core::{IVec2, WorldSnapshot};
use astraweave_nav::NavMesh;
use rapier3d::prelude::*;

/// Enumeration of all validated action verbs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolVerb {
    MoveTo,
    Throw,
    CoverFire,
    Revive,
    Interact,
    UseItem,
    Stay,
    Wander,
    Hide,
    Rally,
}

/// Validation categories for each verb
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationCategory {
    Nav,
    Physics,
    Resources,
    Visibility,
    Cooldown,
}

/// Error taxonomy for tool validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    OutOfBounds,
    Cooldown,
    NoLineOfSight,
    InsufficientResource,
    InvalidTarget,
    PhysicsBlocked,
    NoPath,
    Unknown,
}

use std::fmt;
impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ToolError::OutOfBounds => "OutOfBounds",
            ToolError::Cooldown => "Cooldown",
            ToolError::NoLineOfSight => "NoLineOfSight",
            ToolError::InsufficientResource => "InsufficientResource",
            ToolError::InvalidTarget => "InvalidTarget",
            ToolError::PhysicsBlocked => "PhysicsBlocked",
            ToolError::NoPath => "NoPath",
            ToolError::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

/// Extended validation context with navmesh and physics
pub struct ValidationContext<'a> {
    pub nav_mesh: Option<&'a NavMesh>,
    pub physics_pipeline: Option<&'a PhysicsPipeline>,
    pub rigid_body_set: Option<&'a RigidBodySet>,
    pub collider_set: Option<&'a ColliderSet>,
}

impl<'a> Default for ValidationContext<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ValidationContext<'a> {
    pub fn new() -> Self {
        Self {
            nav_mesh: None,
            physics_pipeline: None,
            rigid_body_set: None,
            collider_set: None,
        }
    }

    pub fn with_nav(mut self, nav: &'a NavMesh) -> Self {
        self.nav_mesh = Some(nav);
        self
    }

    pub fn with_physics(
        mut self,
        pipeline: &'a PhysicsPipeline,
        bodies: &'a RigidBodySet,
        colliders: &'a ColliderSet,
    ) -> Self {
        self.physics_pipeline = Some(pipeline);
        self.rigid_body_set = Some(bodies);
        self.collider_set = Some(colliders);
        self
    }
}

/// Validate ammo availability
fn validate_ammo(world: &WorldSnapshot) -> Result<()> {
    if world.me.ammo == 0 {
        anyhow::bail!("action blocked: insufficient ammo");
    }
    Ok(())
}
/// Validate line of sight to target position
fn validate_line_of_sight(world: &WorldSnapshot, target_pos: Option<IVec2>) -> Result<()> {
    if let Some(pos) = target_pos {
        if !has_line_of_sight(world.me.pos, pos, world) {
            anyhow::bail!("action blocked: no line of sight");
        }
    }
    Ok(())
}

/// Validate a tool action for an agent in the world
pub fn validate_tool_action(
    _agent_id: u32,
    verb: ToolVerb,
    world: &WorldSnapshot,
    context: &ValidationContext,
    target_pos: Option<IVec2>,
) -> Result<()> {
    #[cfg(feature = "profiling")]
    span!("AI::ToolSandbox::validate");

    // Cooldown checks
    if let Some(cd) = world
        .me
        .cooldowns
        .get(&format!("{:?}", verb).to_lowercase())
    {
        if *cd > 0.0 {
            return Err(anyhow::anyhow!(
                "action blocked for verb {:?}: cooldown {:.2}",
                verb,
                cd
            ));
        }
    }
    match verb {
        ToolVerb::MoveTo => {
            if let Some(pos) = target_pos {
                // Nav check: path exists
                if let Some(nav) = context.nav_mesh {
                    let start = glam::Vec3::new(world.me.pos.x as f32, 0.0, world.me.pos.y as f32);
                    let goal = glam::Vec3::new(pos.x as f32, 0.0, pos.y as f32);
                    if nav.find_path(start, goal).is_empty() {
                        anyhow::bail!(
                            "action blocked: no path from {:?} to {:?}",
                            world.me.pos,
                            pos
                        );
                    }
                }
                // Physics check: not blocked by colliders
                if let (Some(_pipeline), Some(bodies), Some(colliders)) = (
                    context.physics_pipeline,
                    context.rigid_body_set,
                    context.collider_set,
                ) {
                    // Check if target position has overlapping colliders
                    // Create a small query AABB at the target position (assuming 2D movement on Y=0 plane)
                    let half_extents = vector![0.4, 0.1, 0.4]; // Small bounding box for agent
                    let query_pos = point![pos.x as f32, 0.0, pos.y as f32];
                    let query_aabb = Aabb::from_half_extents(query_pos, half_extents);

                    // Query for intersecting colliders, but exclude dynamic bodies (agents)
                    // Only consider static/environment colliders as obstacles
                    let mut intersecting = false;
                    for (_collider_handle, collider) in colliders.iter() {
                        // Skip colliders attached to dynamic rigid bodies (agents)
                        if let Some(parent_handle) = collider.parent() {
                            if let Some(rigid_body) = bodies.get(parent_handle) {
                                // Only consider static rigid bodies as obstacles
                                if rigid_body.is_dynamic() {
                                    continue;
                                }
                            }
                        }

                        let collider_aabb = collider.compute_aabb();
                        if query_aabb.intersects(&collider_aabb) {
                            intersecting = true;
                            break;
                        }
                    }

                    if intersecting {
                        anyhow::bail!("action blocked: physics collision at position {:?}", pos);
                    }
                }
            }
        }
        ToolVerb::Throw => {
            validate_ammo(world)?;
            validate_line_of_sight(world, target_pos)?;
        }
        ToolVerb::CoverFire => {
            validate_ammo(world)?;
            validate_line_of_sight(world, target_pos)?;
        }
        ToolVerb::Revive => {
            // Resources: stamina or something
            if world.me.morale < 0.5 {
                anyhow::bail!(
                    "action blocked: low morale for Revive (morale: {:.2}, required: 0.5)",
                    world.me.morale
                );
            }
            // Target valid: ally nearby
            if let Some(pos) = target_pos {
                let dx = pos.x - world.me.pos.x;
                let dy = pos.y - world.me.pos.y;
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > 2.0 {
                    anyhow::bail!(
                        "action blocked: target too far for Revive (distance: {:.2}, max: 2.0)",
                        dist
                    );
                }
            }
        }
        _ => {} // Other actions OK for now
    }
    Ok(())
}

/// Simple LOS check (placeholder: check obstacles)
fn has_line_of_sight(from: IVec2, to: IVec2, world: &WorldSnapshot) -> bool {
    // Bresenham line, check if any obstacle intersects
    let dx = (to.x - from.x).abs();
    let dy = (to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = from.x;
    let mut y = from.y;

    while x != to.x || y != to.y {
        if world.obstacles.iter().any(|obs| obs.x == x && obs.y == y) {
            return false;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{CompanionState, PlayerState, WorldSnapshot};

    #[test]
    fn error_taxonomy_works() {
        assert_eq!(ToolError::OutOfBounds.to_string(), "OutOfBounds");
        assert_eq!(ToolError::NoPath.to_string(), "NoPath");
        assert_eq!(ToolError::NoLineOfSight.to_string(), "NoLineOfSight");
        assert_eq!(
            ToolError::InsufficientResource.to_string(),
            "InsufficientResource"
        );
        assert_eq!(ToolError::Cooldown.to_string(), "Cooldown");
        assert_eq!(ToolError::InvalidTarget.to_string(), "InvalidTarget");
        assert_eq!(ToolError::PhysicsBlocked.to_string(), "PhysicsBlocked");
        assert_eq!(ToolError::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn validate_move_to_no_path() {
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![IVec2 { x: 1, y: 0 }], // obstacle at (1,0)
        };
        let nav = NavMesh::bake(&[], 0.4, 60.0); // empty nav, no path
        let context = ValidationContext::new().with_nav(&nav);
        let result = validate_tool_action(
            0,
            ToolVerb::MoveTo,
            &world,
            &context,
            Some(IVec2 { x: 2, y: 0 }),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no path"));
    }

    #[test]
    fn validate_throw_insufficient_ammo() {
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 0,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };
        let context = ValidationContext::new();
        let result = validate_tool_action(
            0,
            ToolVerb::Throw,
            &world,
            &context,
            Some(IVec2 { x: 1, y: 0 }),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient ammo"));
    }

    #[test]
    fn validate_move_to_physics_blocked() {
        // Create a minimal physics world with a collider at the target position
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        // Add a static rigid body with a collider at position (2, 0, 2)
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(vector![2.0, 0.0, 2.0])
            .build();
        let body_handle = rigid_body_set.insert(rigid_body);

        let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5).build();
        collider_set.insert_with_parent(collider, body_handle, &mut rigid_body_set);

        let physics_pipeline = PhysicsPipeline::new();

        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new().with_physics(
            &physics_pipeline,
            &rigid_body_set,
            &collider_set,
        );
        let result = validate_tool_action(
            0,
            ToolVerb::MoveTo,
            &world,
            &context,
            Some(IVec2 { x: 2, y: 2 }),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("physics collision"));
    }

    // ===== NEW TESTS (8 tests added) =====

    #[test]
    fn test_cover_fire_insufficient_ammo() {
        // CoverFire requires ammo, should fail when ammo = 0
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 0, // No ammo
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(
            0,
            ToolVerb::CoverFire,
            &world,
            &context,
            Some(IVec2 { x: 3, y: 3 }),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient ammo"));
    }

    #[test]
    fn test_cover_fire_no_line_of_sight() {
        // CoverFire requires LoS, should fail when obstacles block
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![IVec2 { x: 1, y: 0 }, IVec2 { x: 2, y: 0 }], // Block path
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(
            0,
            ToolVerb::CoverFire,
            &world,
            &context,
            Some(IVec2 { x: 3, y: 0 }),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no line of sight"));
    }

    #[test]
    fn test_cover_fire_success_with_ammo_and_los() {
        // CoverFire should succeed when ammo > 0 and LoS is clear
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![], // No obstacles
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(
            0,
            ToolVerb::CoverFire,
            &world,
            &context,
            Some(IVec2 { x: 3, y: 3 }),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_revive_low_morale() {
        // Revive requires morale >= 0.5, should fail when low
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 0.3, // Too low
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(
            0,
            ToolVerb::Revive,
            &world,
            &context,
            Some(IVec2 { x: 1, y: 1 }),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("low morale"));
    }

    #[test]
    fn test_revive_target_too_far() {
        // Revive requires target within 2.0 distance
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(
            0,
            ToolVerb::Revive,
            &world,
            &context,
            Some(IVec2 { x: 5, y: 5 }), // Distance = ~7.07, exceeds 2.0
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("target too far"));
    }

    #[test]
    fn test_validation_context_builders() {
        // Test ValidationContext builder pattern
        let nav = NavMesh::bake(&[], 0.4, 60.0);

        let rigid_body_set = RigidBodySet::new();
        let collider_set = ColliderSet::new();
        let physics_pipeline = PhysicsPipeline::new();

        // Test with_nav
        let context1 = ValidationContext::new().with_nav(&nav);
        assert!(context1.nav_mesh.is_some());
        assert!(context1.physics_pipeline.is_none());

        // Test with_physics
        let context2 = ValidationContext::new().with_physics(
            &physics_pipeline,
            &rigid_body_set,
            &collider_set,
        );
        assert!(context2.nav_mesh.is_none());
        assert!(context2.physics_pipeline.is_some());
        assert!(context2.rigid_body_set.is_some());
        assert!(context2.collider_set.is_some());

        // Test chaining both
        let context3 = ValidationContext::new().with_nav(&nav).with_physics(
            &physics_pipeline,
            &rigid_body_set,
            &collider_set,
        );
        assert!(context3.nav_mesh.is_some());
        assert!(context3.physics_pipeline.is_some());
    }

    #[test]
    fn test_cooldown_blocking() {
        // Test that cooldowns block actions
        let mut cooldowns = std::collections::BTreeMap::new();
        cooldowns.insert("coverfire".to_string(), 2.5); // CoverFire on cooldown

        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns,
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(
            0,
            ToolVerb::CoverFire,
            &world,
            &context,
            Some(IVec2 { x: 3, y: 3 }),
        );

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("cooldown"));
        assert!(err_msg.contains("2.5")); // Cooldown value should be in error message
    }

    #[test]
    fn test_stay_and_wander_always_valid() {
        // Stay and Wander actions should always succeed (no validation checks)
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 0, // Even with no ammo
                cooldowns: std::collections::BTreeMap::new(),
                morale: 0.0, // Even with zero morale
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();

        // Stay should always succeed
        let result_stay = validate_tool_action(0, ToolVerb::Stay, &world, &context, None);
        assert!(result_stay.is_ok());

        // Wander should always succeed
        let result_wander = validate_tool_action(0, ToolVerb::Wander, &world, &context, None);
        assert!(result_wander.is_ok());
    }

    // ========================================
    // Additional ToolVerb Variant Tests
    // ========================================

    #[test]
    fn test_interact_always_valid() {
        // Interact should succeed with minimal world state
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 0,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 0.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(0, ToolVerb::Interact, &world, &context, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_item_always_valid() {
        // UseItem should succeed with minimal world state
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 0,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 0.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(0, ToolVerb::UseItem, &world, &context, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hide_always_valid() {
        // Hide should succeed with minimal world state
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 0,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 0.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(0, ToolVerb::Hide, &world, &context, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rally_always_valid() {
        // Rally should succeed with minimal world state
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 0,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 0.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(0, ToolVerb::Rally, &world, &context, None);
        assert!(result.is_ok());
    }

    // ========================================
    // ToolVerb Enum Tests
    // ========================================

    #[test]
    fn test_tool_verb_debug() {
        // Test Debug derive for all variants
        let verbs = vec![
            ToolVerb::MoveTo,
            ToolVerb::Throw,
            ToolVerb::CoverFire,
            ToolVerb::Revive,
            ToolVerb::Interact,
            ToolVerb::UseItem,
            ToolVerb::Stay,
            ToolVerb::Wander,
            ToolVerb::Hide,
            ToolVerb::Rally,
        ];

        for verb in verbs {
            let debug_str = format!("{:?}", verb);
            assert!(!debug_str.is_empty(), "Debug output should not be empty");
        }
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn test_tool_verb_clone_and_copy() {
        // Test Clone and Copy derives
        let verb1 = ToolVerb::MoveTo;
        let verb2 = verb1; // Copy
        let verb3 = verb1.clone(); // Clone

        assert_eq!(verb1, verb2);
        assert_eq!(verb1, verb3);
    }

    #[test]
    fn test_tool_verb_partial_eq() {
        // Test PartialEq
        assert_eq!(ToolVerb::MoveTo, ToolVerb::MoveTo);
        assert_ne!(ToolVerb::MoveTo, ToolVerb::Throw);
        assert_eq!(ToolVerb::CoverFire, ToolVerb::CoverFire);
    }

    #[test]
    fn test_tool_verb_hash() {
        // Test Hash derive (via HashMap insertion)
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ToolVerb::MoveTo, 1);
        map.insert(ToolVerb::Throw, 2);
        map.insert(ToolVerb::CoverFire, 3);

        assert_eq!(map.get(&ToolVerb::MoveTo), Some(&1));
        assert_eq!(map.get(&ToolVerb::Throw), Some(&2));
        assert_eq!(map.get(&ToolVerb::CoverFire), Some(&3));
    }

    // ========================================
    // ValidationCategory Enum Tests
    // ========================================

    #[test]
    fn test_validation_category_all_variants() {
        // Test all 5 ValidationCategory variants
        let categories = vec![
            ValidationCategory::Nav,
            ValidationCategory::Physics,
            ValidationCategory::Resources,
            ValidationCategory::Visibility,
            ValidationCategory::Cooldown,
        ];

        for category in categories {
            let debug_str = format!("{:?}", category);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_validation_category_partial_eq() {
        assert_eq!(ValidationCategory::Nav, ValidationCategory::Nav);
        assert_ne!(ValidationCategory::Nav, ValidationCategory::Physics);
        assert_eq!(ValidationCategory::Cooldown, ValidationCategory::Cooldown);
    }

    #[test]
    fn test_validation_category_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ValidationCategory::Nav, "navigation");
        map.insert(ValidationCategory::Physics, "physics");
        map.insert(ValidationCategory::Resources, "resources");

        assert_eq!(map.get(&ValidationCategory::Nav), Some(&"navigation"));
        assert_eq!(map.get(&ValidationCategory::Physics), Some(&"physics"));
    }

    // ========================================
    // ToolError Enum Tests
    // ========================================

    #[test]
    fn test_tool_error_clone() {
        let err1 = ToolError::OutOfBounds;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_tool_error_partial_eq() {
        assert_eq!(ToolError::NoPath, ToolError::NoPath);
        assert_ne!(ToolError::NoPath, ToolError::Cooldown);
    }

    #[test]
    fn test_tool_error_debug() {
        let err = ToolError::InvalidTarget;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidTarget"));
    }

    // ========================================
    // has_line_of_sight Edge Cases
    // ========================================

    #[test]
    fn test_has_line_of_sight_same_position() {
        // LoS from position to itself should succeed
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 5, y: 5 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let has_los = has_line_of_sight(IVec2 { x: 5, y: 5 }, IVec2 { x: 5, y: 5 }, &world);
        assert!(has_los, "LoS to same position should succeed");
    }

    #[test]
    fn test_has_line_of_sight_horizontal_line() {
        // Horizontal line (y constant)
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![], // No obstacles
        };

        let has_los = has_line_of_sight(IVec2 { x: 0, y: 0 }, IVec2 { x: 5, y: 0 }, &world);
        assert!(has_los, "Clear horizontal LoS should succeed");
    }

    #[test]
    fn test_has_line_of_sight_vertical_line() {
        // Vertical line (x constant)
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let has_los = has_line_of_sight(IVec2 { x: 0, y: 0 }, IVec2 { x: 0, y: 5 }, &world);
        assert!(has_los, "Clear vertical LoS should succeed");
    }

    #[test]
    fn test_has_line_of_sight_diagonal_line() {
        // Diagonal line (both x and y change)
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let has_los = has_line_of_sight(IVec2 { x: 0, y: 0 }, IVec2 { x: 5, y: 5 }, &world);
        assert!(has_los, "Clear diagonal LoS should succeed");
    }

    #[test]
    fn test_has_line_of_sight_blocked_midpoint() {
        // Obstacle at midpoint of line
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![IVec2 { x: 2, y: 0 }], // Midpoint obstacle
        };

        let has_los = has_line_of_sight(IVec2 { x: 0, y: 0 }, IVec2 { x: 4, y: 0 }, &world);
        assert!(!has_los, "Blocked LoS should fail");
    }

    // ========================================
    // validate_line_of_sight with None
    // ========================================

    #[test]
    fn test_validate_line_of_sight_none_target() {
        // validate_line_of_sight with None should always pass
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![IVec2 { x: 1, y: 0 }], // Obstacles present
        };

        let result = validate_line_of_sight(&world, None);
        assert!(result.is_ok(), "None target should always pass LoS check");
    }

    // ========================================
    // MoveTo with None target
    // ========================================

    #[test]
    fn test_move_to_none_target() {
        // MoveTo with None target should succeed (no validation)
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(0, ToolVerb::MoveTo, &world, &context, None);
        assert!(result.is_ok(), "MoveTo with None target should succeed");
    }

    // ========================================
    // Revive with None target
    // ========================================

    #[test]
    fn test_revive_none_target() {
        // Revive with None target should skip distance check
        let world = WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                hp: 100,

                pos: IVec2 { x: 0, y: 0 },
                stance: "standing".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0, // High morale
                pos: IVec2 { x: 0, y: 0 },
            },
            enemies: vec![],
            pois: vec![],
            objective: None,
            obstacles: vec![],
        };

        let context = ValidationContext::new();
        let result = validate_tool_action(0, ToolVerb::Revive, &world, &context, None);
        assert!(
            result.is_ok(),
            "Revive with None target should succeed if morale OK"
        );
    }

    // ========================================
    // ValidationContext Default
    // ========================================

    #[test]
    fn test_validation_context_default() {
        let context = ValidationContext::default();
        assert!(context.nav_mesh.is_none());
        assert!(context.physics_pipeline.is_none());
        assert!(context.rigid_body_set.is_none());
        assert!(context.collider_set.is_none());
    }
}
