use astraweave_core::{ActionStep, IVec2, ToolRegistry, WorldSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicConfig {
    pub rules: Vec<HeuristicRule>,
}

impl Default for HeuristicConfig {
    fn default() -> Self {
        Self {
            rules: vec![
                // Default rules matching previous hardcoded logic
                HeuristicRule {
                    condition: HeuristicCondition::LowMorale { threshold: 30.0 },
                    action: HeuristicAction::HealSelf,
                },
                HeuristicRule {
                    condition: HeuristicCondition::LowAmmo { threshold: 0 },
                    action: HeuristicAction::Reload,
                },
                HeuristicRule {
                    condition: HeuristicCondition::EnemyNearby { max_distance: 3.0 },
                    action: HeuristicAction::AttackNearestEnemy,
                },
                HeuristicRule {
                    condition: HeuristicCondition::EnemyVisible, // Fallback if not close enough to attack
                    action: HeuristicAction::TakeCover { distance: 2.0 },
                },
                HeuristicRule {
                    condition: HeuristicCondition::ObjectiveContains {
                        keyword: "extract".to_string(),
                    },
                    action: HeuristicAction::MoveToObjective,
                },
                HeuristicRule {
                    condition: HeuristicCondition::ObjectiveContains {
                        keyword: "reach".to_string(),
                    },
                    action: HeuristicAction::MoveToObjective,
                },
                HeuristicRule {
                    condition: HeuristicCondition::Always, // "Nothing urgent" fallback
                    action: HeuristicAction::Scan { radius: 10.0 },
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicRule {
    pub condition: HeuristicCondition,
    pub action: HeuristicAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HeuristicCondition {
    LowMorale { threshold: f32 },
    LowAmmo { threshold: u32 },
    EnemyNearby { max_distance: f32 },
    EnemyVisible,
    ObjectiveContains { keyword: String },
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HeuristicAction {
    HealSelf,
    Reload,
    AttackNearestEnemy,
    TakeCover { distance: f32 },
    MoveToObjective,
    Scan { radius: f32 },
}

impl HeuristicRule {
    pub fn evaluate(&self, snap: &WorldSnapshot, reg: &ToolRegistry) -> Option<ActionStep> {
        if self.check_condition(snap) {
            self.create_action(snap, reg)
        } else {
            None
        }
    }

    fn check_condition(&self, snap: &WorldSnapshot) -> bool {
        match &self.condition {
            HeuristicCondition::LowMorale { threshold } => snap.me.morale < *threshold,
            HeuristicCondition::LowAmmo { threshold } => snap.me.ammo <= *threshold as i32,
            HeuristicCondition::EnemyNearby { max_distance } => {
                if let Some(enemy) = snap.enemies.first() {
                    let dist = ((snap.me.pos.x - enemy.pos.x).abs()
                        + (snap.me.pos.y - enemy.pos.y).abs())
                        as f32;
                    dist <= *max_distance
                } else {
                    false
                }
            }
            HeuristicCondition::EnemyVisible => !snap.enemies.is_empty(),
            HeuristicCondition::ObjectiveContains { keyword } => snap
                .objective
                .as_ref()
                .map(|o| o.contains(keyword))
                .unwrap_or(false),
            HeuristicCondition::Always => true,
        }
    }

    fn create_action(&self, snap: &WorldSnapshot, reg: &ToolRegistry) -> Option<ActionStep> {
        match &self.action {
            HeuristicAction::HealSelf => {
                if reg.tools.iter().any(|t| t.name == "heal") {
                    Some(ActionStep::Heal { target_id: Some(0) })
                } else {
                    None
                }
            }
            HeuristicAction::Reload => {
                if reg.tools.iter().any(|t| t.name == "reload") {
                    Some(ActionStep::Reload)
                } else {
                    None
                }
            }
            HeuristicAction::AttackNearestEnemy => {
                if let Some(enemy) = snap.enemies.first() {
                    if reg.tools.iter().any(|t| t.name == "attack") {
                        Some(ActionStep::Attack {
                            target_id: enemy.id,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            HeuristicAction::TakeCover { distance } => {
                if let Some(enemy) = snap.enemies.first() {
                    if reg.tools.iter().any(|t| t.name == "take_cover") {
                        let dist_int = *distance as i32;
                        let cover_x = if snap.me.pos.x > enemy.pos.x {
                            snap.me.pos.x + dist_int
                        } else {
                            snap.me.pos.x - dist_int
                        };
                        Some(ActionStep::TakeCover {
                            position: Some(IVec2 {
                                x: cover_x,
                                y: snap.me.pos.y,
                            }),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            HeuristicAction::MoveToObjective => {
                if let Some(poi) = snap.pois.first() {
                    if reg.tools.iter().any(|t| t.name == "move_to") {
                        Some(ActionStep::MoveTo {
                            x: poi.pos.x,
                            y: poi.pos.y,
                            speed: None,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            HeuristicAction::Scan { radius } => {
                if reg.tools.iter().any(|t| t.name == "scan") {
                    Some(ActionStep::Scan { radius: *radius })
                } else {
                    None
                }
            }
        }
    }
}
