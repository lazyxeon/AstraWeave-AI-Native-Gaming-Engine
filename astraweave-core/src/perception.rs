use crate::{World, WorldSnapshot, PlayerState, CompanionState, EnemyState, IVec2, Entity};
use crate::schema::Poi;
use std::collections::BTreeMap;

pub struct PerceptionConfig {
    pub los_max: i32,
}

pub fn build_snapshot(
    w: &World,
    t_player: Entity,
    t_companion: Entity,
    enemies: &[Entity],
    objective: Option<String>,
    cfg: &PerceptionConfig
) -> WorldSnapshot {
    let ppos = w.pos_of(t_player).unwrap();
    let cpos = w.pos_of(t_companion).unwrap();
    let player = PlayerState {
        hp: w.health(t_player).unwrap().hp,
        pos: ppos,
        stance: "crouch".into(),
        orders: vec!["hold_east".into()],
    };
    let me = CompanionState {
        ammo: w.ammo(t_companion).unwrap().rounds,
        cooldowns: w
            .cooldowns(t_companion)
            .unwrap()
            .map
            .clone()
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        morale: 0.8,
        pos: cpos,
    };
    let enemies = enemies.iter().filter_map(|&e| {
        let pos = w.pos_of(e)?;
        let hp  = w.health(e)?.hp;
        // LOS consider simple radius; real LOS in validator
        let cover = if (pos.x - ppos.x).abs() + (pos.y - ppos.y).abs() > cfg.los_max { "unknown" } else { "low" };
        Some(EnemyState { id: e, pos, hp, cover: cover.into(), last_seen: w.t })
    }).collect::<Vec<_>>();

    WorldSnapshot {
        t: w.t,
        player,
        me,
        enemies,
        pois: vec![Poi {
            k: "breach_door".into(),
            pos: IVec2 { x: 15, y: 8 },
        }],
        objective,
    }
}
