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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile() -> NpcProfile {
        NpcProfile {
            id: "test_npc".to_string(),
            role: Role::Merchant,
            persona: Persona {
                display_name: "Test Merchant".to_string(),
                traits: vec!["friendly".to_string(), "greedy".to_string()],
                backstory: "A humble merchant".to_string(),
                voice_speaker: Some("merchant_voice".to_string()),
            },
            memory: Memory {
                facts: vec!["player bought potion".to_string()],
                episodes: vec!["met player at market".to_string()],
            },
            home: [10.0, 0.0, 20.0],
            schedule: vec![
                ScheduleEntry {
                    hour: 8,
                    action: "work".to_string(),
                    target: [15.0, 0.0, 25.0],
                },
            ],
        }
    }

    #[test]
    fn test_role_equality() {
        assert_eq!(Role::Merchant, Role::Merchant);
        assert_ne!(Role::Merchant, Role::Guard);
    }

    #[test]
    fn test_role_serialization() {
        let role = Role::Guard;
        let json = serde_json::to_string(&role).unwrap();
        let parsed: Role = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Role::Guard);
    }

    #[test]
    fn test_all_roles() {
        let roles = [Role::Merchant, Role::Guard, Role::Civilian, Role::QuestGiver];
        for role in roles {
            let json = serde_json::to_string(&role).unwrap();
            let parsed: Role = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, role);
        }
    }

    #[test]
    fn test_persona_clone() {
        let persona = Persona {
            display_name: "Test".to_string(),
            traits: vec!["trait1".to_string()],
            backstory: "backstory".to_string(),
            voice_speaker: Some("voice".to_string()),
        };
        let cloned = persona.clone();
        assert_eq!(cloned.display_name, persona.display_name);
    }

    #[test]
    fn test_persona_default_fields() {
        let json = r#"{"display_name": "Test", "traits": []}"#;
        let persona: Persona = serde_json::from_str(json).unwrap();
        assert_eq!(persona.backstory, "");
        assert!(persona.voice_speaker.is_none());
    }

    #[test]
    fn test_memory_default_fields() {
        let json = "{}";
        let memory: Memory = serde_json::from_str(json).unwrap();
        assert!(memory.facts.is_empty());
        assert!(memory.episodes.is_empty());
    }

    #[test]
    fn test_schedule_entry_serialization() {
        let entry = ScheduleEntry {
            hour: 12,
            action: "patrol".to_string(),
            target: [1.0, 2.0, 3.0],
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ScheduleEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.hour, 12);
        assert_eq!(parsed.action, "patrol");
    }

    #[test]
    fn test_npc_profile_home_vec3() {
        let profile = sample_profile();
        let home = profile.home_vec3();
        assert_eq!(home.x, 10.0);
        assert_eq!(home.y, 0.0);
        assert_eq!(home.z, 20.0);
    }

    #[test]
    fn test_npc_profile_home_vec3_zero() {
        let profile = NpcProfile {
            id: "test".to_string(),
            role: Role::Civilian,
            persona: Persona {
                display_name: "Test".to_string(),
                traits: vec![],
                backstory: String::new(),
                voice_speaker: None,
            },
            memory: Memory { facts: vec![], episodes: vec![] },
            home: [0.0, 0.0, 0.0],
            schedule: vec![],
        };
        let home = profile.home_vec3();
        assert_eq!(home, Vec3::ZERO);
    }

    #[test]
    fn test_load_profile_from_toml_str_valid() {
        let toml = r#"
            id = "merchant_1"
            role = "Merchant"
            home = [10.0, 0.0, 20.0]
            
            [persona]
            display_name = "Bob the Merchant"
            traits = ["friendly", "talkative"]
            backstory = "Born in the market district"
            
            [memory]
            facts = ["player is VIP"]
            episodes = ["sold rare item"]
            
            [[schedule]]
            hour = 8
            action = "work"
            target = [15.0, 0.0, 25.0]
        "#;
        
        let profile = load_profile_from_toml_str(toml).unwrap();
        assert_eq!(profile.id, "merchant_1");
        assert_eq!(profile.role, Role::Merchant);
        assert_eq!(profile.persona.display_name, "Bob the Merchant");
        assert_eq!(profile.schedule.len(), 1);
    }

    #[test]
    fn test_load_profile_from_toml_str_minimal() {
        let toml = r#"
            id = "npc_1"
            role = "Civilian"
            
            [persona]
            display_name = "John"
            traits = []
            
            [memory]
        "#;
        
        let profile = load_profile_from_toml_str(toml).unwrap();
        assert_eq!(profile.id, "npc_1");
        assert!(profile.schedule.is_empty());
    }

    #[test]
    fn test_load_profile_from_toml_str_invalid() {
        let toml = "not valid toml {{{{";
        let result = load_profile_from_toml_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_npc_profile_serialization_roundtrip() {
        let profile = sample_profile();
        let json = serde_json::to_string(&profile).unwrap();
        let parsed: NpcProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, profile.id);
        assert_eq!(parsed.role, profile.role);
    }
}

