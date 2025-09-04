use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

pub type Entity = u32;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IVec2 { pub x: i32, pub y: i32 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub t: f32,
    pub player: PlayerState,
    pub me: CompanionState,
    pub enemies: Vec<EnemyState>,
    pub pois: Vec<Poi>,
    pub objective: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub hp: i32,
    pub pos: IVec2,
    pub stance: String,
    pub orders: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanionState {
    pub ammo: i32,
    pub cooldowns: BTreeMap<String, f32>,
    pub morale: f32,
    pub pos: IVec2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyState {
    pub id: Entity,
    pub pos: IVec2,
    pub hp: i32,
    pub cover: String,
    pub last_seen: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Poi { pub k: String, pub pos: IVec2 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanIntent {
    pub plan_id: String,
    pub steps: Vec<ActionStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "act")]
pub enum ActionStep {
    MoveTo { x: i32, y: i32 },
    Throw { item: String, x: i32, y: i32 },
    CoverFire { target_id: Entity, duration: f32 },
    Revive { ally_id: Entity },
    // extend with Converse, UseAbility, etc.
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub args: BTreeMap<String, String>, // k: name, v: type ("i32","f32","enum[...]")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolRegistry {
    pub tools: Vec<ToolSpec>,
    pub constraints: Constraints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraints {
    pub enforce_cooldowns: bool,
    pub enforce_los: bool,
    pub enforce_stamina: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("invalid action: {0}")]
    InvalidAction(String),
    #[error("cooldown blocked: {0}")]
    Cooldown(String),
    #[error("line of sight blocked")]
    LosBlocked,
    #[error("path not found")]
    NoPath,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rect { pub x0:i32, pub y0:i32, pub x1:i32, pub y1:i32 }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum DirectorOp {
    Fortify { rect: Rect },                         // add obstacles
    SpawnWave { archetype: String, count: u32, origin: IVec2 },
    Collapse { a: IVec2, b: IVec2 },               // line of obstacles ("bridge down")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectorBudget {
    pub traps: i32,
    pub terrain_edits: i32,
    pub spawns: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectorPlan {
    pub ops: Vec<DirectorOp>,
}
