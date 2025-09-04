use crate::{World, PlanIntent, ActionStep, EngineError, tools::{los_clear, path_exists}, IVec2, Entity};

pub struct ValidateCfg {
    pub world_bounds: (i32,i32,i32,i32),
}

pub fn validate_and_execute(
    w: &mut World,
    actor: Entity,
    intent: &PlanIntent,
    cfg: &ValidateCfg,
    log: &mut impl FnMut(String),
) -> Result<(), EngineError> {
    log(format!("Plan {} with {} steps", intent.plan_id, intent.steps.len()));
    for (i, step) in intent.steps.iter().enumerate() {
        match step {
            ActionStep::MoveTo { x, y } => {
                let from = w.pos_of(actor).unwrap();
                let to = IVec2{x:*x, y:*y};
                if !path_exists(&w.obstacles, from, to, cfg.world_bounds) {
                    return Err(EngineError::NoPath);
                }
                w.pose_mut(actor).unwrap().pos = to;
                log(format!("  [{}] MOVE_TO -> ({},{})", i, x,y));
            }
            ActionStep::Throw { item, x, y } => {
                let from = w.pos_of(actor).unwrap();
                let target = IVec2{x:*x, y:*y};
                if !los_clear(&w.obstacles, from, target) {
                    return Err(EngineError::LosBlocked);
                }
                let cds = w.cooldowns_mut(actor).unwrap();
                let cd_key = format!("throw:{}", item);
                if cds.map.get(&cd_key).copied().unwrap_or(0.0) > 0.0 {
                    return Err(EngineError::Cooldown(cd_key));
                }
                cds.map.insert(cd_key.clone(), 8.0);
                log(format!("  [{}] THROW {} -> ({},{})", i, item, x,y));
            }
            ActionStep::CoverFire { target_id, duration } => {
                let my = w.pos_of(actor).unwrap();
                let tgt = w.pos_of(*target_id).ok_or_else(|| EngineError::InvalidAction("target gone".into()))?;
                if !los_clear(&w.obstacles, my, tgt) {
                    return Err(EngineError::LosBlocked);
                }
                // simulate: reduce target hp a bit depending on duration
                if let Some(h) = w.health_mut(*target_id) {
                    let dmg = ((*duration)*5.0) as i32;
                    h.hp -= dmg.max(1);
                }
                let ammo = w.ammo_mut(actor).unwrap();
                ammo.rounds = (ammo.rounds - 3).max(0);
                log(format!("  [{}] COVER_FIRE on #{} for {:.1}s", i, target_id, duration));
            }
            ActionStep::Revive { ally_id } => {
                if let Some(h) = w.health_mut(*ally_id) {
                    if h.hp <= 0 { h.hp = 20; }
                }
                log(format!("  [{}] REVIVE #{}", i, ally_id));
            }
        }
    }
    Ok(())
}
