use serde::{Serialize, Deserialize};
use glam::Vec3;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EmoteKind { Wave, Nod, Shrug, Point }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NpcAction {
    Say { text: String },
    MoveTo { pos: Vec3, speed: f32 },
    Emote { kind: EmoteKind },
    OpenShop,
    GiveQuest { id: String },
    CallGuards { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcPlan {
    pub actions: Vec<NpcAction>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcMode { Idle, Patrolling, Working, Conversing, Flee, Combat }
