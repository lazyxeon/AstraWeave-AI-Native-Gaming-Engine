//! ECS component types mirroring legacy World data (Phase 1 incremental migration)
use crate::IVec2;

#[derive(Clone, Copy, Debug, Default)]
pub struct CPos { pub pos: IVec2 }

#[derive(Clone, Copy, Debug, Default)]
pub struct CHealth { pub hp: i32 }

#[derive(Clone, Copy, Debug, Default)]
pub struct CTeam { pub id: u8 }

#[derive(Clone, Copy, Debug, Default)]
pub struct CAmmo { pub rounds: i32 }

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

	pub type Map = BTreeMap<CooldownKey, f32>;
}

use cooldowns::Map as CooldownMap;

#[derive(Clone, Debug, Default)]
pub struct CCooldowns { pub map: CooldownMap }

#[derive(Clone, Copy, Debug, Default)]
pub struct CDesiredPos { pub pos: IVec2 }

#[derive(Clone, Copy, Debug, Default)]
pub struct CAiAgent;

#[derive(Clone, Copy, Debug, Default)]
/// Component storing the legacy World entity id for round-trip mapping.
pub struct CLegacyId { pub id: crate::Entity }
