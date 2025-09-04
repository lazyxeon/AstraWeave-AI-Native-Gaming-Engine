use astraweave_core::*;
#[derive(Clone, Debug)]
pub struct PhaseSpec {
    pub name: String,
    pub hp_threshold: i32,      // when boss HP <= threshold, switch to next phase
    pub terrain_bias: f32,      // 0..1: how much to prefer terrain edits vs spawns
    pub aggression: f32,        // 0..1: how aggressive the plan should be
}
#[derive(Clone, Debug)]
pub struct PhaseState {
    pub idx: usize,
    pub last_switch_t: f32,
    pub telegraph: Option<String>,
}
pub struct PhaseDirector {
    pub phases: Vec<PhaseSpec>,
    pub state: PhaseState,
}
pub struct PhasePlan {
    pub phase_name: String,
    pub telegraphs: Vec<String>,
    pub director: DirectorPlan,
}
impl PhaseDirector {
    pub fn new(phases: Vec<PhaseSpec>) -> Self {
        Self { phases, state: PhaseState { idx:0, last_switch_t: 0.0, telegraph: None } }
    }
    /// Given a snapshot (boss = enemies[0]) and budget, devise a plan and maybe switch phase.
    pub fn step(&mut self, snap: &WorldSnapshot, budget: &DirectorBudget) -> PhasePlan {
        let mut tele = vec![];
        if let Some(boss) = snap.enemies.first() {
            // phase switch by hp
            while self.state.idx + 1 < self.phases.len() &&
                  boss.hp <= self.phases[self.state.idx+1].hp_threshold {
                self.state.idx += 1;
                self.state.telegraph = Some(format!("Boss shifts into phase: {}", self.phases[self.state.idx].name));
                tele.push(self.state.telegraph.clone().unwrap());
            }
        }
        // craft a plan using simple bias rules
        let phase = &self.phases[self.state.idx];
        let ppos = snap.player.pos;
        let tgt  = snap.enemies.first().map(|e| e.pos)
                    .unwrap_or(IVec2{x:ppos.x+6, y:ppos.y});
        let mut ops = vec![];
        if phase.terrain_bias > 0.5 && budget.terrain_edits > 0 {
            // prefer fortify/choke
            let xm = (ppos.x + tgt.x)/2;
            let ym = (ppos.y + tgt.y)/2;
            ops.push(DirectorOp::Fortify { rect: Rect { x0:xm-1, y0:ym-1, x1:xm+1, y1:ym+1 }});
            tele.push("The ground trembles—ramparts rise!".into());
        } else {
            if budget.spawns > 0 {
                ops.push(DirectorOp::SpawnWave { archetype:"phase_add".into(), count: 4, origin: IVec2{x: ppos.x-2, y: ppos.y+1} });
                tele.push("A spectral cohort joins the fray!".into());
            }
            if budget.terrain_edits > 0 {
                ops.push(DirectorOp::Collapse { a: ppos, b: IVec2{x:(ppos.x+tgt.x)/2, y:(ppos.y+tgt.y)/2} });
                tele.push("Bridges shatter—paths rerouted!".into());
            }
        }
        PhasePlan {
            phase_name: phase.name.clone(),
            telegraphs: tele,
            director: DirectorPlan { ops }
        }
    }
}
