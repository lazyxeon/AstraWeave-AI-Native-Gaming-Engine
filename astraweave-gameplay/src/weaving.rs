use anyhow::Result;
use glam::{vec3, Vec3};
use astraweave_core::{
    DirectorOp, DirectorBudget, DirectorPlan,
    apply_director_plan, World, IVec2
};
use astraweave_physics::PhysicsWorld;
use astraweave_nav::{NavMesh, Triangle};
use crate::{WeaveOp, WeaveOpKind, WeaveBudget, WeaveConsequence};

/// Apply a single weave op to the world + physics, consume budgets, and return consequences.
pub fn apply_weave_op(
    w: &mut World,
    phys: &mut PhysicsWorld,
    nav_src: &[Triangle],
    budget: &mut WeaveBudget,
    op: &WeaveOp,
    log: &mut impl FnMut(String),
) -> Result<WeaveConsequence> {
    let mut core_budget = DirectorBudget { traps: 0, terrain_edits: 3, spawns: 0 };
    let mut plan = DirectorPlan { ops: vec![] };

    match op.kind {
        WeaveOpKind::ReinforcePath => {
            if budget.terrain_edits <= 0 { anyhow::bail!("No terrain budget"); }
            // fortify a small rect around A
            let a = op.a;
            plan.ops.push(DirectorOp::Fortify { rect: astraweave_core::Rect {
                x0: a.x as i32 - 1, y0: a.z as i32 - 1, x1: a.x as i32 + 1, y1: a.z as i32 + 1
            }});
            budget.terrain_edits -= 1;
        }
        WeaveOpKind::CollapseBridge => {
            if budget.terrain_edits <= 0 { anyhow::bail!("No terrain budget"); }
            let a = op.a; let b = op.b.ok_or_else(|| anyhow::anyhow!("Collapse needs A->B"))?;
            plan.ops.push(DirectorOp::Collapse { a: IVec2{x:a.x as i32, y:a.z as i32}, b: IVec2{x:b.x as i32, y:b.z as i32} });
            budget.terrain_edits -= 1;
        }
        WeaveOpKind::RedirectWind => {
            if budget.weather_ops <= 0 { anyhow::bail!("No weather budget"); }
            let dir = (op.b.unwrap_or(op.a + vec3(1.0,0.0,0.0)) - op.a).normalize_or_zero();
            phys.set_wind(dir, 10.0);
            budget.weather_ops -= 1;
            log("Weave: Wind redirected".into());
        }
        WeaveOpKind::LowerWater => {
            if budget.weather_ops <= 0 { anyhow::bail!("No weather budget"); }
            // crude: clear water volumes entirely (demo)
            phys.water.clear();
            budget.weather_ops -= 1;
            log("Weave: Waters receded".into());
        }
        WeaveOpKind::RaisePlatform => {
            if budget.terrain_edits <= 0 { anyhow::bail!("No terrain budget"); }
            let a = op.a;
            plan.ops.push(DirectorOp::Fortify { rect: astraweave_core::Rect {
                x0: a.x as i32, y0: a.z as i32, x1: a.x as i32, y1: a.z as i32
            }});
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
        WeaveOpKind::ReinforcePath => WeaveConsequence { drop_multiplier: 1.1, faction_disposition: 5, weather_shift: None },
        WeaveOpKind::CollapseBridge => WeaveConsequence { drop_multiplier: 0.9, faction_disposition: -10, weather_shift: None },
        WeaveOpKind::RedirectWind => WeaveConsequence { drop_multiplier: 1.0, faction_disposition: 0, weather_shift: Some("windy".into()) },
        WeaveOpKind::LowerWater => WeaveConsequence { drop_multiplier: 1.0, faction_disposition: 0, weather_shift: Some("dry".into()) },
        WeaveOpKind::RaisePlatform => WeaveConsequence { drop_multiplier: 1.05, faction_disposition: 0, weather_shift: None },
    };

    Ok(consequence)
}
