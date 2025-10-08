use astraweave_core::{DirectorBudget, DirectorOp, DirectorPlan, IVec2, Rect, WorldSnapshot};

/// Minimal heuristic boss director:
/// - If player trends ranged (distance > 8), fortify a choke around midpoint.
/// - Else: spawn a small wave behind player and collapse a nearby bridge line.
pub struct BossDirector;

mod phase;
pub use phase::*;

mod llm_director;
pub use llm_director::*;

mod components;
pub use components::*;

mod systems;
pub use systems::*;

impl BossDirector {
    pub fn plan(&self, snap: &WorldSnapshot, budget: &DirectorBudget) -> DirectorPlan {
        let mut ops = vec![];
        let ppos = snap.player.pos;
        let _mpos = snap.me.pos;
        // choose a target enemy if any
        let tgt = snap.enemies.first().map(|e| e.pos).unwrap_or(IVec2 {
            x: ppos.x + 6,
            y: ppos.y,
        });

        let dist = (ppos.x - tgt.x).abs() + (ppos.y - tgt.y).abs();
        if dist > 8 && budget.terrain_edits > 0 {
            // Fortify: draw a small rectangle near target as a makeshift choke
            let xm = (ppos.x + tgt.x) / 2;
            let ym = (ppos.y + tgt.y) / 2;
            ops.push(DirectorOp::Fortify {
                rect: Rect {
                    x0: xm - 1,
                    y0: ym - 1,
                    x1: xm + 1,
                    y1: ym + 1,
                },
            });
        } else {
            // Spawn wave behind player, collapse a line between player and target
            if budget.spawns > 0 {
                let origin = IVec2 {
                    x: ppos.x - 2,
                    y: ppos.y + 1,
                };
                ops.push(DirectorOp::SpawnWave {
                    archetype: "minion".into(),
                    count: 3,
                    origin,
                });
            }
            if budget.terrain_edits > 0 {
                let line_b = IVec2 {
                    x: (ppos.x + tgt.x) / 2,
                    y: (ppos.y + tgt.y) / 2,
                };
                ops.push(DirectorOp::Collapse { a: ppos, b: line_b });
            }
        }
        DirectorPlan { ops }
    }
}
