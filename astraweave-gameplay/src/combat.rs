use serde::{Serialize, Deserialize};
use glam::Vec3;
use crate::{Stats, DamageType, items::Item};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AttackKind { Light, Heavy }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComboStep {
    pub kind: AttackKind,
    pub window: (f32, f32), // input window secs after previous impact
    pub damage: i32,
    pub reach: f32, // meters
    pub stagger: f32, // seconds if hit
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComboChain {
    pub name: String,
    pub steps: Vec<ComboStep>,
}

#[derive(Clone, Debug)]
pub struct AttackState {
    pub chain: ComboChain,
    pub idx: usize,
    pub t_since_last: f32,
    pub active: bool,
}

impl AttackState {
    pub fn new(chain: ComboChain) -> Self {
        Self { chain, idx: 0, t_since_last: 0.0, active: false }
    }

    pub fn start(&mut self) { self.active = true; self.idx = 0; self.t_since_last = 0.0; }

    /// call per-frame; returns (did_hit, applied_damage)
    pub fn tick(
        &mut self,
        dt: f32,
        pressed_light: bool,
        pressed_heavy: bool,
        attacker_pos: Vec3,
        target_pos: Vec3,
        attacker_stats: &Stats,
        weapon: Option<&Item>,
        target: &mut Stats
    ) -> (bool, i32) {
        if !self.active { return (false, 0); }
        self.t_since_last += dt;
        let step = &self.chain.steps[self.idx];
        let want = match step.kind {
            AttackKind::Light => pressed_light,
            AttackKind::Heavy => pressed_heavy,
        };
        let in_win = self.t_since_last >= step.window.0 && self.t_since_last <= step.window.1;
        let mut did_hit = false;
        let mut dmg = 0;

        if want && in_win {
            // impact check: distance <= reach
            let d = attacker_pos.distance(target_pos);
            if d <= step.reach {
                let base = step.damage + attacker_stats.power;
                let modified = if let Some(w) = weapon {
                    match &w.kind {
                        crate::items::ItemKind::Weapon { base_damage, dtype } => {
                            let mult = w.echo.as_ref().map(|e| e.power_mult).unwrap_or(1.0);
                            let dtype = w.echo.as_ref().and_then(|e| e.dtype_override).unwrap_or(*dtype);
                            let out = ((base + base_damage) as f32 * mult) as i32;
                            target.apply_damage(out, dtype);
                            dmg = out;
                        }
                        _ => { let out = base; target.apply_damage(out, DamageType::Physical); dmg = out; }
                    }
                } else {
                    target.apply_damage(base, DamageType::Physical);
                    dmg = base;
                }
                // apply stagger
                target.effects.push(crate::stats::StatusEffect::Stagger { time: step.stagger });
                did_hit = true;
            }
            // next step
            self.idx += 1;
            self.t_since_last = 0.0;
            if self.idx >= self.chain.steps.len() {
                self.active = false;
            }
        }
        (did_hit, dmg)
    }
}
