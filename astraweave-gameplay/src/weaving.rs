use crate::{WeaveBudget, WeaveConsequence, WeaveOp, WeaveOpKind};
use anyhow::Result;
use astraweave_core::{
    apply_director_plan, DirectorBudget, DirectorOp, DirectorPlan, IVec2, World,
};
use astraweave_nav::{NavMesh, Triangle};
use astraweave_physics::PhysicsWorld;
use glam::vec3;

/// Apply a single weave op to the world + physics, consume budgets, and return consequences.
pub fn apply_weave_op(
    w: &mut World,
    phys: &mut PhysicsWorld,
    nav_src: &[Triangle],
    budget: &mut WeaveBudget,
    op: &WeaveOp,
    log: &mut impl FnMut(String),
) -> Result<WeaveConsequence> {
    let mut core_budget = DirectorBudget {
        traps: 0,
        terrain_edits: 3,
        spawns: 0,
    };
    let mut plan = DirectorPlan { ops: vec![] };

    match op.kind {
        WeaveOpKind::ReinforcePath => {
            if budget.terrain_edits <= 0 {
                anyhow::bail!("No terrain budget");
            }
            // fortify a small rect around A
            let a = op.a;
            plan.ops.push(DirectorOp::Fortify {
                rect: astraweave_core::Rect {
                    x0: a.x as i32 - 1,
                    y0: a.z as i32 - 1,
                    x1: a.x as i32 + 1,
                    y1: a.z as i32 + 1,
                },
            });
            budget.terrain_edits -= 1;
        }
        WeaveOpKind::CollapseBridge => {
            if budget.terrain_edits <= 0 {
                anyhow::bail!("No terrain budget");
            }
            let a = op.a;
            let b = op.b.ok_or_else(|| anyhow::anyhow!("Collapse needs A->B"))?;
            plan.ops.push(DirectorOp::Collapse {
                a: IVec2 {
                    x: a.x as i32,
                    y: a.z as i32,
                },
                b: IVec2 {
                    x: b.x as i32,
                    y: b.z as i32,
                },
            });
            budget.terrain_edits -= 1;
        }
        WeaveOpKind::RedirectWind => {
            if budget.weather_ops <= 0 {
                anyhow::bail!("No weather budget");
            }
            let dir = (op.b.unwrap_or(op.a + vec3(1.0, 0.0, 0.0)) - op.a).normalize_or_zero();
            phys.set_wind(dir, 10.0);
            budget.weather_ops -= 1;
            log("Weave: Wind redirected".into());
        }
        WeaveOpKind::LowerWater => {
            if budget.weather_ops <= 0 {
                anyhow::bail!("No weather budget");
            }
            // crude: clear water volumes entirely (demo)
            phys.clear_water();
            budget.weather_ops -= 1;
            log("Weave: Waters receded".into());
        }
        WeaveOpKind::RaisePlatform => {
            if budget.terrain_edits <= 0 {
                anyhow::bail!("No terrain budget");
            }
            let a = op.a;
            plan.ops.push(DirectorOp::Fortify {
                rect: astraweave_core::Rect {
                    x0: a.x as i32,
                    y0: a.z as i32,
                    x1: a.x as i32,
                    y1: a.z as i32,
                },
            });
            budget.terrain_edits -= 1;
        }
    }

    if !plan.ops.is_empty() {
        apply_director_plan(w, &mut core_budget, &plan, log);
    }

    // Re-bake a small navmesh from provided triangles (demo pathing update)
    let _nav = NavMesh::bake(nav_src, 0.5, 55.0);

    // Return a rough “world consequence”
    let consequence = match op.kind {
        WeaveOpKind::ReinforcePath => WeaveConsequence {
            drop_multiplier: 1.1,
            faction_disposition: 5,
            weather_shift: None,
        },
        WeaveOpKind::CollapseBridge => WeaveConsequence {
            drop_multiplier: 0.9,
            faction_disposition: -10,
            weather_shift: None,
        },
        WeaveOpKind::RedirectWind => WeaveConsequence {
            drop_multiplier: 1.0,
            faction_disposition: 0,
            weather_shift: Some("windy".into()),
        },
        WeaveOpKind::LowerWater => WeaveConsequence {
            drop_multiplier: 1.0,
            faction_disposition: 0,
            weather_shift: Some("dry".into()),
        },
        WeaveOpKind::RaisePlatform => WeaveConsequence {
            drop_multiplier: 1.05,
            faction_disposition: 0,
            weather_shift: None,
        },
    };

    Ok(consequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;

    fn create_test_world() -> World {
        World::new()
    }

    fn create_test_physics() -> PhysicsWorld {
        PhysicsWorld::new(vec3(0.0, -9.81, 0.0))
    }

    fn create_test_budget() -> WeaveBudget {
        WeaveBudget {
            terrain_edits: 5,
            weather_ops: 3,
        }
    }

    #[test]
    fn test_apply_weave_op_reinforce_path_success() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = create_test_budget();
        let op = WeaveOp {
            kind: WeaveOpKind::ReinforcePath,
            a: vec3(5.0, 0.0, 5.0),
            b: None,
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_ok(), "ReinforcePath should succeed");
        assert_eq!(budget.terrain_edits, 4, "Should consume 1 terrain edit");
        
        let consequence = result.unwrap();
        assert_eq!(consequence.drop_multiplier, 1.1);
        assert_eq!(consequence.faction_disposition, 5);
        assert_eq!(consequence.weather_shift, None);
    }

    #[test]
    fn test_apply_weave_op_reinforce_path_no_budget() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = WeaveBudget {
            terrain_edits: 0, // No budget!
            weather_ops: 3,
        };
        let op = WeaveOp {
            kind: WeaveOpKind::ReinforcePath,
            a: vec3(5.0, 0.0, 5.0),
            b: None,
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_err(), "Should fail with no budget");
        assert!(result.unwrap_err().to_string().contains("No terrain budget"));
    }

    #[test]
    fn test_apply_weave_op_collapse_bridge_success() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = create_test_budget();
        let op = WeaveOp {
            kind: WeaveOpKind::CollapseBridge,
            a: vec3(0.0, 0.0, 0.0),
            b: Some(vec3(10.0, 0.0, 10.0)),
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_ok(), "CollapseBridge should succeed");
        assert_eq!(budget.terrain_edits, 4, "Should consume 1 terrain edit");
        
        let consequence = result.unwrap();
        assert_eq!(consequence.drop_multiplier, 0.9);
        assert_eq!(consequence.faction_disposition, -10);
    }

    #[test]
    fn test_apply_weave_op_collapse_bridge_no_point_b() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = create_test_budget();
        let op = WeaveOp {
            kind: WeaveOpKind::CollapseBridge,
            a: vec3(0.0, 0.0, 0.0),
            b: None, // Missing required point B!
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_err(), "Should fail without point B");
        assert!(result.unwrap_err().to_string().contains("Collapse needs A->B"));
    }

    #[test]
    fn test_apply_weave_op_redirect_wind_success() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = create_test_budget();
        let op = WeaveOp {
            kind: WeaveOpKind::RedirectWind,
            a: vec3(0.0, 0.0, 0.0),
            b: Some(vec3(1.0, 0.0, 0.0)),
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_ok(), "RedirectWind should succeed");
        assert_eq!(budget.weather_ops, 2, "Should consume 1 weather op");
        assert!(log_output.iter().any(|msg| msg.contains("Wind redirected")), "Should log wind redirect");
        
        let consequence = result.unwrap();
        assert_eq!(consequence.weather_shift, Some("windy".to_string()));
    }

    #[test]
    fn test_apply_weave_op_redirect_wind_no_budget() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = WeaveBudget {
            terrain_edits: 5,
            weather_ops: 0, // No weather budget!
        };
        let op = WeaveOp {
            kind: WeaveOpKind::RedirectWind,
            a: vec3(0.0, 0.0, 0.0),
            b: Some(vec3(1.0, 0.0, 0.0)),
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_err(), "Should fail with no weather budget");
        assert!(result.unwrap_err().to_string().contains("No weather budget"));
    }

    #[test]
    fn test_apply_weave_op_lower_water_success() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = create_test_budget();
        let op = WeaveOp {
            kind: WeaveOpKind::LowerWater,
            a: vec3(0.0, 0.0, 0.0),
            b: None,
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_ok(), "LowerWater should succeed");
        assert_eq!(budget.weather_ops, 2, "Should consume 1 weather op");
        assert!(log_output.iter().any(|msg| msg.contains("Waters receded")), "Should log water lowering");
        
        let consequence = result.unwrap();
        assert_eq!(consequence.weather_shift, Some("dry".to_string()));
    }

    #[test]
    fn test_apply_weave_op_raise_platform_success() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = create_test_budget();
        let op = WeaveOp {
            kind: WeaveOpKind::RaisePlatform,
            a: vec3(5.0, 0.0, 5.0),
            b: None,
            budget_cost: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        let result = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op, &mut logger);

        assert!(result.is_ok(), "RaisePlatform should succeed");
        assert_eq!(budget.terrain_edits, 4, "Should consume 1 terrain edit");
        
        let consequence = result.unwrap();
        assert_eq!(consequence.drop_multiplier, 1.05);
        assert_eq!(consequence.faction_disposition, 0);
    }

    #[test]
    fn test_apply_weave_op_multiple_operations() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = create_test_budget();
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        // Apply ReinforcePath
        let op1 = WeaveOp {
            kind: WeaveOpKind::ReinforcePath,
            a: vec3(0.0, 0.0, 0.0),
            b: None,
            budget_cost: 1,
        };
        apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op1, &mut logger).unwrap();

        // Apply RedirectWind
        let op2 = WeaveOp {
            kind: WeaveOpKind::RedirectWind,
            a: vec3(0.0, 0.0, 0.0),
            b: Some(vec3(1.0, 0.0, 0.0)),
            budget_cost: 1,
        };
        apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op2, &mut logger).unwrap();

        assert_eq!(budget.terrain_edits, 4, "Should have used 1 terrain edit");
        assert_eq!(budget.weather_ops, 2, "Should have used 1 weather op");
    }

    #[test]
    fn test_apply_weave_op_budget_depletion() {
        let mut world = create_test_world();
        let mut physics = create_test_physics();
        let nav_src = vec![];
        let mut budget = WeaveBudget {
            terrain_edits: 1,
            weather_ops: 1,
        };
        let mut log_output = Vec::new();
        let mut logger = |msg: String| log_output.push(msg);

        // First terrain op succeeds
        let op1 = WeaveOp {
            kind: WeaveOpKind::ReinforcePath,
            a: vec3(0.0, 0.0, 0.0),
            b: None,
            budget_cost: 1,
        };
        let result1 = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op1, &mut logger);
        assert!(result1.is_ok(), "First terrain op should succeed");

        // Second terrain op fails (budget depleted)
        let op2 = WeaveOp {
            kind: WeaveOpKind::CollapseBridge,
            a: vec3(0.0, 0.0, 0.0),
            b: Some(vec3(1.0, 0.0, 1.0)),
            budget_cost: 1,
        };
        let result2 = apply_weave_op(&mut world, &mut physics, &nav_src, &mut budget, &op2, &mut logger);
        assert!(result2.is_err(), "Second terrain op should fail");
    }
}
