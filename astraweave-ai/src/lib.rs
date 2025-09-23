use astraweave_core::{ActionStep, IVec2, PlanIntent, WorldSnapshot};

pub trait Orchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent;
}

/// Minimal rule-based orchestrator:
/// If enemy in LOS-ish and "smoke" not on cooldown:
///   throw smoke midway, move up, cover fire.
/// Else: advance towards nearest enemy.
pub struct RuleOrchestrator;

impl Orchestrator for RuleOrchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent {
        let plan_id = format!("plan-{}", (snap.t * 1000.0) as i64);
        if let Some(first) = snap.enemies.first() {
            let m = &snap.me;
            let _p = &snap.player;

            // midpoint for smoke
            let mid = IVec2 {
                x: (m.pos.x + first.pos.x) / 2,
                y: (m.pos.y + first.pos.y) / 2,
            };

            let smoke_cd = snap.me.cooldowns.get("throw:smoke").copied().unwrap_or(0.0);
            if smoke_cd <= 0.0 {
                return PlanIntent {
                    plan_id,
                    steps: vec![
                        ActionStep::Throw {
                            item: "smoke".into(),
                            x: mid.x,
                            y: mid.y,
                        },
                        ActionStep::MoveTo {
                            x: m.pos.x + (first.pos.x - m.pos.x).signum() * 2,
                            y: m.pos.y + (first.pos.y - m.pos.y).signum() * 2,
                        },
                        ActionStep::CoverFire {
                            target_id: first.id,
                            duration: 2.5,
                        },
                    ],
                };
            } else {
                // advance cautiously
                return PlanIntent {
                    plan_id,
                    steps: vec![
                        ActionStep::MoveTo {
                            x: m.pos.x + (first.pos.x - m.pos.x).signum(),
                            y: m.pos.y + (first.pos.y - m.pos.y).signum(),
                        },
                        ActionStep::CoverFire {
                            target_id: first.id,
                            duration: 1.5,
                        },
                    ],
                };
            }
        }

        // fallback
        PlanIntent {
            plan_id,
            steps: vec![],
        }
    }
}

/// Utility-based orchestrator: scores a few candidate plans deterministically.
/// Heuristics:
/// - Prefer throwing smoke if an enemy exists and cooldown is ready
/// - Otherwise move closer to nearest enemy; if very close (<3), add brief cover fire
pub struct UtilityOrchestrator;

impl Orchestrator for UtilityOrchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent {
        let plan_id = format!("util-{}", (snap.t * 1000.0) as i64);
        let me = &snap.me;
        // Candidate A: Throw smoke at midpoint to first enemy
        let mut cands: Vec<(f32, Vec<ActionStep>)> = vec![];
        if let Some(enemy) = snap.enemies.first() {
            let cd = me.cooldowns.get("throw:smoke").copied().unwrap_or(0.0);
            if cd <= 0.0 {
                let mid = IVec2 { x: (me.pos.x + enemy.pos.x)/2, y: (me.pos.y + enemy.pos.y)/2 };
                let steps = vec![
                    ActionStep::Throw { item: "smoke".into(), x: mid.x, y: mid.y },
                    ActionStep::MoveTo { x: me.pos.x + (enemy.pos.x - me.pos.x).signum()*2, y: me.pos.y + (enemy.pos.y - me.pos.y).signum()*2 },
                ];
                // Score: encourage if ammo present and enemy hp > 0
                let score = 1.0 + (snap.player.hp as f32).min(100.0) * 0.0 + (enemy.hp.max(0) as f32) * 0.01;
                cands.push((score, steps));
            }
            // Candidate B: advance; if close, cover fire
            let dx = (enemy.pos.x - me.pos.x).abs();
            let dy = (enemy.pos.y - me.pos.y).abs();
            let dist = (dx + dy) as f32;
            let mut steps = vec![ActionStep::MoveTo { x: me.pos.x + (enemy.pos.x - me.pos.x).signum(), y: me.pos.y + (enemy.pos.y - me.pos.y).signum() }];
            if dist <= 3.0 {
                steps.push(ActionStep::CoverFire { target_id: enemy.id, duration: 1.0 });
            }
            let score = 0.8 + (3.0 - dist).max(0.0) * 0.05;
            cands.push((score, steps));
        }
        cands.sort_by(|a,b| b.0.partial_cmp(&a.0).unwrap());
        let steps = cands.first().map(|c| c.1.clone()).unwrap_or_default();
        PlanIntent { plan_id, steps }
    }
}

/// Minimal GOAP-style orchestrator for MoveTo -> CoverFire chain towards first enemy.
/// Preconditions: enemy exists. Goal: be within 2 cells and apply CoverFire for 1.5s.
pub struct GoapOrchestrator;

impl Orchestrator for GoapOrchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent {
        let plan_id = format!("goap-{}", (snap.t * 1000.0) as i64);
        if let Some(enemy) = snap.enemies.first() {
            let me = &snap.me;
            let dx = (enemy.pos.x - me.pos.x).abs();
            let dy = (enemy.pos.y - me.pos.y).abs();
            let dist = dx + dy;
            if dist <= 2 {
                return PlanIntent{ plan_id, steps: vec![ActionStep::CoverFire{ target_id: enemy.id, duration: 1.5 }] };
            } else {
                // Move one step closer; replan next tick
                return PlanIntent{ plan_id, steps: vec![ActionStep::MoveTo{ x: me.pos.x + (enemy.pos.x - me.pos.x).signum(), y: me.pos.y + (enemy.pos.y - me.pos.y).signum() }] };
            }
        }
        PlanIntent{ plan_id, steps: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{CompanionState, EnemyState, PlayerState, WorldSnapshot};
    use std::collections::BTreeMap;

    fn snap_basic(px: i32, py: i32, ex: i32, ey: i32, smoke_cd: f32) -> WorldSnapshot {
        WorldSnapshot{
            t: 1.234,
            player: PlayerState{ hp: 80, pos: IVec2{x:px, y:py}, stance: "stand".into(), orders: vec![] },
            me: CompanionState{ ammo: 10, cooldowns: BTreeMap::from([("throw:smoke".into(), smoke_cd)]), morale: 0.5, pos: IVec2{x:px, y:py} },
            enemies: vec![EnemyState{ id: 2, pos: IVec2{x:ex, y:ey}, hp: 50, cover: "low".into(), last_seen: 0.0 }],
            pois: vec![],
            objective: None,
        }
    }

    #[test]
    fn utility_prefers_smoke_when_ready() {
        let s = snap_basic(0,0, 4,0, 0.0);
        let o = UtilityOrchestrator;
        let plan = o.propose_plan(&s);
        match plan.steps.first() { Some(ActionStep::Throw{..}) => {}, _ => panic!("expected Throw first") }
    }

    #[test]
    fn goap_moves_then_covers() {
        let s_far = snap_basic(0,0, 5,0, 10.0);
        let goap = GoapOrchestrator;
        let plan1 = goap.propose_plan(&s_far);
        assert!(matches!(plan1.steps.first(), Some(ActionStep::MoveTo{..})));
        let s_close = snap_basic(0,0, 1,0, 10.0);
        let plan2 = goap.propose_plan(&s_close);
        assert!(matches!(plan2.steps.first(), Some(ActionStep::CoverFire{..})));
    }
}
