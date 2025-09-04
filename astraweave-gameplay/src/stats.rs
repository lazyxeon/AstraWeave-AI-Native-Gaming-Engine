use serde::{Serialize, Deserialize};
use crate::DamageType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub hp: i32,
    pub stamina: i32,
    pub power: i32,
    pub defense: i32,
    pub echo_amp: f32,
    pub effects: Vec<StatusEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatusEffect {
    Stagger { time: f32 },
    Bleed { dps: f32, time: f32 },
    Chill  { slow: f32, time: f32 },
}

impl Stats {
    pub fn new(hp: i32) -> Self {
        Self { hp, stamina: 100, power: 10, defense: 5, echo_amp: 1.0, effects: vec![] }
    }

    pub fn apply_damage(&mut self, amount: i32, dtype: DamageType) -> i32 {
        let mitigated = (amount as f32 - self.defense as f32 * 0.5).max(1.0) as i32;
        self.hp -= mitigated;
        mitigated
    }

    pub fn tick(&mut self, dt: f32) -> i32 {
        // returns aggregate DoT damage this tick
        let mut dot = 0.0;
        self.effects.retain_mut(|e| {
            match e {
                StatusEffect::Bleed { dps, time } => {
                    dot += *dps * dt;
                    *time -= dt; *time > 0.0
                }
                StatusEffect::Stagger { time } => { *time -= dt; *time > 0.0 }
                StatusEffect::Chill { time, .. } => { *time -= dt; *time > 0.0 }
            }
        });
        let d = dot as i32;
        self.hp -= d;
        d
    }
}
