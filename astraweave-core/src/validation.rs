use crate::{
    tools::{los_clear, path_exists},
    ActionStep, EngineError, Entity, IVec2, PlanIntent, World,
};

pub struct ValidateCfg {
    pub world_bounds: (i32, i32, i32, i32),
}

pub fn validate_and_execute(
    w: &mut World,
    actor: Entity,
    intent: &PlanIntent,
    cfg: &ValidateCfg,
    log: &mut impl FnMut(String),
) -> Result<(), EngineError> {
    log(format!(
        "Plan {} with {} steps",
        intent.plan_id,
        intent.steps.len()
    ));
    for (i, step) in intent.steps.iter().enumerate() {
        match step {
            ActionStep::MoveTo { x, y } => {
                let from = w.pos_of(actor).unwrap();
                let to = IVec2 { x: *x, y: *y };
                if !path_exists(&w.obstacles, from, to, cfg.world_bounds) {
                    return Err(EngineError::NoPath);
                }
                w.pose_mut(actor).unwrap().pos = to;
                log(format!("  [{}] MOVE_TO -> ({},{})", i, x, y));
            }
            ActionStep::Throw { item, x, y } => {
                let from = w.pos_of(actor).unwrap();
                let target = IVec2 { x: *x, y: *y };
                if !los_clear(&w.obstacles, from, target) {
                    return Err(EngineError::LosBlocked);
                }
                let cds = w.cooldowns_mut(actor).unwrap();
                let cd_key = format!("throw:{}", item);
                if cds.map.get(&cd_key).copied().unwrap_or(0.0) > 0.0 {
                    return Err(EngineError::Cooldown(cd_key));
                }
                cds.map.insert(cd_key.clone(), 8.0);
                log(format!("  [{}] THROW {} -> ({},{})", i, item, x, y));
            }
            ActionStep::CoverFire {
                target_id,
                duration,
            } => {
                let my = w.pos_of(actor).unwrap();
                let tgt = w
                    .pos_of(*target_id)
                    .ok_or_else(|| EngineError::InvalidAction("target gone".into()))?;
                if !los_clear(&w.obstacles, my, tgt) {
                    return Err(EngineError::LosBlocked);
                }
                // Ensure ammo present
                if let Some(am) = w.ammo(actor) {
                    if am.rounds <= 0 { return Err(EngineError::Resource("ammo".into())); }
                }
                // simulate: reduce target hp a bit depending on duration
                if let Some(h) = w.health_mut(*target_id) {
                    let dmg = ((*duration) * 5.0) as i32;
                    h.hp -= dmg.max(1);
                }
                let ammo = w.ammo_mut(actor).unwrap();
                ammo.rounds = (ammo.rounds - 3).max(0);
                log(format!(
                    "  [{}] COVER_FIRE on #{} for {:.1}s",
                    i, target_id, duration
                ));
            }
            ActionStep::Revive { ally_id } => {
                if let Some(h) = w.health_mut(*ally_id) {
                    if h.hp <= 0 {
                        h.hp = 20;
                    }
                }
                log(format!("  [{}] REVIVE #{}", i, ally_id));
            }
        }
    }
    Ok(())
}

use crate::{DirectorOp, DirectorPlan, Rect};

fn fill_rect_obs(obs: &mut std::collections::HashSet<(i32, i32)>, r: Rect) {
    for x in r.x0.min(r.x1)..=r.x0.max(r.x1) {
        for y in r.y0.min(r.y1)..=r.y0.max(r.y1) {
            obs.insert((x, y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Team, World};

    fn mk_world_clear() -> World { World::new() }

    #[test]
    fn cover_fire_requires_ammo() {
        let mut w = mk_world_clear();
        let actor = w.spawn("ally", IVec2 { x: 0, y: 0 }, Team { id: 1 }, 100, 0);
        let enemy = w.spawn("enemy", IVec2 { x: 3, y: 0 }, Team { id: 2 }, 50, 0);
        let intent = PlanIntent {
            plan_id: "t".into(),
            steps: vec![ActionStep::CoverFire { target_id: enemy, duration: 1.0 }],
        };
        let cfg = ValidateCfg { world_bounds: (-10, -10, 10, 10) };
        let mut log = |_s: String| {};
        let res = validate_and_execute(&mut w, actor, &intent, &cfg, &mut log);
        match res { Err(EngineError::Resource(k)) => assert_eq!(k, "ammo"), _ => panic!("expected Resource(ammo)") }
    }

    #[test]
    fn cover_fire_consumes_ammo_and_damages() {
        let mut w = mk_world_clear();
        let actor = w.spawn("ally", IVec2 { x: 0, y: 0 }, Team { id: 1 }, 100, 10);
        let enemy = w.spawn("enemy", IVec2 { x: 2, y: 0 }, Team { id: 2 }, 50, 0);
        let intent = PlanIntent {
            plan_id: "t".into(),
            steps: vec![ActionStep::CoverFire { target_id: enemy, duration: 1.0 }],
        };
        let cfg = ValidateCfg { world_bounds: (-10, -10, 10, 10) };
        let mut log = |_s: String| {};
        let hp_before = w.health(enemy).unwrap().hp;
        let ammo_before = w.ammo(actor).unwrap().rounds;
        let res = validate_and_execute(&mut w, actor, &intent, &cfg, &mut log);
        assert!(res.is_ok());
        let hp_after = w.health(enemy).unwrap().hp;
        let ammo_after = w.ammo(actor).unwrap().rounds;
        assert!(hp_after < hp_before, "enemy should take damage");
        assert_eq!(ammo_after, (ammo_before - 3).max(0));
    }
}
fn draw_line_obs(obs: &mut std::collections::HashSet<(i32, i32)>, a: IVec2, b: IVec2) {
    let mut x = a.x;
    let mut y = a.y;
    let dx = (b.x - a.x).signum();
    let dy = (b.y - a.y).signum();
    while x != b.x || y != b.y {
        obs.insert((x, y));
        if x != b.x {
            x += dx;
        }
        if y != b.y {
            y += dy;
        }
    }
    obs.insert((b.x, b.y));
}

// Execute a DirectorPlan with crude budgets (you can move this into a Director crate too)
pub fn apply_director_plan(
    w: &mut World,
    budget: &mut crate::DirectorBudget,
    plan: &DirectorPlan,
    log: &mut impl FnMut(String),
) {
    for (i, op) in plan.ops.iter().enumerate() {
        match op {
            DirectorOp::Fortify { rect } => {
                if budget.terrain_edits <= 0 {
                    log(format!("  [op{}] Fortify SKIPPED (budget)", i));
                    continue;
                }
                fill_rect_obs(&mut w.obstacles, *rect);
                budget.terrain_edits -= 1;
                log(format!(
                    "  [op{}] Fortify rect=({},{}..{},{}))",
                    i, rect.x0, rect.y0, rect.x1, rect.y1
                ));
            }
            DirectorOp::Collapse { a, b } => {
                if budget.terrain_edits <= 0 {
                    log(format!("  [op{}] Collapse SKIPPED (budget)", i));
                    continue;
                }
                draw_line_obs(&mut w.obstacles, *a, *b);
                budget.terrain_edits -= 1;
                log(format!(
                    "  [op{}] Collapse line=({},{})→({},{})",
                    i, a.x, a.y, b.x, b.y
                ));
            }
            DirectorOp::SpawnWave {
                archetype,
                count,
                origin,
            } => {
                if budget.spawns <= 0 {
                    log(format!("  [op{}] SpawnWave SKIPPED (budget)", i));
                    continue;
                }
                for k in 0..*count {
                    let off = IVec2 {
                        x: origin.x + (k as i32 % 3) - 1,
                        y: origin.y + (k as i32 / 3),
                    };
                    let id = w.spawn(
                        &format!("{}{}", archetype, k),
                        off,
                        crate::Team { id: 2 },
                        40,
                        0,
                    );
                    log(format!("  [op{}] Spawned {} at {:?}", i, id, off));
                }
                budget.spawns -= 1;
            }
        }
    }
}
