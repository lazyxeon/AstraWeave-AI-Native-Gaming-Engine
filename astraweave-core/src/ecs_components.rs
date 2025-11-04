//! ECS component types mirroring legacy World data (Phase 1 incremental migration)
use crate::IVec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CPos {
    pub pos: IVec2,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CHealth {
    pub hp: i32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CTeam {
    pub id: u8,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CAmmo {
    pub rounds: i32,
}

pub mod cooldowns {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    /// Efficient key for cooldown kinds. Known variants can be matched statically;
    /// unknown/custom keys fall back to `Custom(String)`.
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum CooldownKey {
        ThrowSmoke,
        Custom(String),
    }

    impl From<&str> for CooldownKey {
        fn from(s: &str) -> Self {
            match s {
                "throw:smoke" => CooldownKey::ThrowSmoke,
                _ => CooldownKey::Custom(s.to_string()),
            }
        }
    }

    impl From<String> for CooldownKey {
        fn from(s: String) -> Self {
            CooldownKey::from(s.as_str())
        }
    }

    impl std::fmt::Display for CooldownKey {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CooldownKey::ThrowSmoke => write!(f, "throw:smoke"),
                CooldownKey::Custom(s) => write!(f, "{}", s),
            }
        }
    }

    pub type Map = BTreeMap<CooldownKey, f32>;
}

use cooldowns::Map as CooldownMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CCooldowns {
    pub map: CooldownMap,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CDesiredPos {
    pub pos: IVec2,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CAiAgent;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// Component storing the legacy World entity id for round-trip mapping.
pub struct CLegacyId {
    pub id: crate::Entity,
}

// Temporary placeholder types to avoid circular dependency
// These will be replaced when the memory system is integrated properly

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CompanionProfile {
    pub name: String,
    pub personality_traits: Vec<String>,
    pub background: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub content: String,
    pub confidence: f32,
    pub timestamp: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub description: String,
    pub timestamp: f64,
    pub importance: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CPersona {
    pub profile: CompanionProfile,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CMemory {
    pub facts: Vec<Fact>,
    pub episodes: Vec<Episode>,
}

#[cfg(test)]
mod tests {
    use super::cooldowns::CooldownKey;
    use super::*;

    #[test]
    fn test_cpos_default() {
        let cpos = CPos::default();
        assert_eq!(cpos.pos.x, 0);
        assert_eq!(cpos.pos.y, 0);
    }

    #[test]
    fn test_chealth_default() {
        let health = CHealth::default();
        assert_eq!(health.hp, 0);
    }

    #[test]
    fn test_cteam_default() {
        let team = CTeam::default();
        assert_eq!(team.id, 0);
    }

    #[test]
    fn test_cammo_default() {
        let ammo = CAmmo::default();
        assert_eq!(ammo.rounds, 0);
    }

    #[test]
    fn test_cooldown_key_from_str_known() {
        let key = CooldownKey::from("throw:smoke");
        assert_eq!(key, CooldownKey::ThrowSmoke);
    }

    #[test]
    fn test_cooldown_key_from_str_custom() {
        let key = CooldownKey::from("custom_ability");
        assert_eq!(key, CooldownKey::Custom("custom_ability".into()));
    }

    #[test]
    fn test_cooldown_key_from_string() {
        let key = CooldownKey::from("throw:smoke".to_string());
        assert_eq!(key, CooldownKey::ThrowSmoke);

        let key2 = CooldownKey::from("other".to_string());
        assert_eq!(key2, CooldownKey::Custom("other".into()));
    }

    #[test]
    fn test_cooldown_key_display() {
        let key1 = CooldownKey::ThrowSmoke;
        assert_eq!(format!("{}", key1), "throw:smoke");

        let key2 = CooldownKey::Custom("fireball".into());
        assert_eq!(format!("{}", key2), "fireball");
    }

    #[test]
    fn test_ccooldowns_default() {
        let cds = CCooldowns::default();
        assert!(cds.map.is_empty());
    }

    #[test]
    fn test_cdesired_pos_default() {
        let pos = CDesiredPos::default();
        assert_eq!(pos.pos.x, 0);
        assert_eq!(pos.pos.y, 0);
    }
}
