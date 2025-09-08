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
