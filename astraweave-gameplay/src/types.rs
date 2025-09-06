use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Echo,
    Fire,
    Frost,
    Shock,
    Poison,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum ResourceKind {
    Wood,
    Crystal,
    Ore,
    Fiber,
    Essence,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DropTableEntry {
    pub kind: ResourceKind,
    pub min: u32,
    pub max: u32,
    pub weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeaveConsequence {
    pub drop_multiplier: f32,          // affects harvesting in region
    pub faction_disposition: i32,      // -100..100
    pub weather_shift: Option<String>, // e.g., "windy", "calm", "storm"
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WeaveOpKind {
    ReinforcePath,
    CollapseBridge,
    RedirectWind,
    LowerWater,
    RaisePlatform,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeaveOp {
    pub kind: WeaveOpKind,
    pub a: Vec3,
    pub b: Option<Vec3>,
    pub budget_cost: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeaveBudget {
    pub terrain_edits: i32,
    pub weather_ops: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryEntry {
    pub kind: ResourceKind,
    pub count: u32,
}
