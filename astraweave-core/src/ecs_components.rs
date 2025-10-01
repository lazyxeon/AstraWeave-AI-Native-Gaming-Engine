//! ECS component types mirroring legacy World data (Phase 1 incremental migration)
use crate::IVec2;

#[derive(Clone, Copy, Debug, Default)]
pub struct CPos {
    pub pos: IVec2,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CHealth {
    pub hp: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CTeam {
    pub id: u8,
}

#[derive(Clone, Copy, Debug, Default)]
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

#[derive(Clone, Debug, Default)]
pub struct CCooldowns {
    pub map: CooldownMap,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CDesiredPos {
    pub pos: IVec2,
}

#[derive(Clone, Debug, Default)]
pub struct CAiAgent;

#[derive(Clone, Debug, Default)]
/// Component storing the legacy World entity id for round-trip mapping.
pub struct CLegacyId {
    pub id: crate::Entity,
}

#[derive(Clone, Debug)]
pub struct CPersona {
    pub profile: astraweave_memory::CompanionProfile,
}

impl Default for CPersona {
    fn default() -> Self {
        Self {
            profile: astraweave_memory::CompanionProfile::new_default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CMemory {
    pub facts: Vec<astraweave_memory::Fact>,
    pub episodes: Vec<astraweave_memory::Episode>,
}
