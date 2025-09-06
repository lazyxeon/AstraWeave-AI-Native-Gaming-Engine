use anyhow::Result;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Merchant,
    Guard,
    Civilian,
    QuestGiver,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Persona {
    pub display_name: String,
    pub traits: Vec<String>,
    #[serde(default)]
    pub backstory: String,
    #[serde(default)]
    pub voice_speaker: Option<String>, // match your audio voice bank speaker key
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    #[serde(default)]
    pub facts: Vec<String>,
    #[serde(default)]
    pub episodes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub hour: u8,         // 0..23
    pub action: String,   // "work","patrol","rest","shop"
    pub target: [f32; 3], // position to move to
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcProfile {
    pub id: String,
    pub role: Role,
    pub persona: Persona,
    pub memory: Memory,
    #[serde(default)]
    pub home: [f32; 3],
    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,
}

impl NpcProfile {
    pub fn home_vec3(&self) -> Vec3 {
        Vec3::from(self.home)
    }
}

pub fn load_profile_from_toml_str(s: &str) -> Result<NpcProfile> {
    Ok(toml::from_str::<NpcProfile>(s)?)
}
