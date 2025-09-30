//! Tool Sandbox: Validated action verbs and error taxonomy

use anyhow::Result;
use astraweave_core::WorldSnapshot;

/// Enumeration of all validated action verbs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolVerb {
    MoveTo,
    Throw,
    CoverFire,
    Revive,
    Interact,
    UseItem,
    Stay,
    Wander,
    Hide,
    Rally,
}

/// Validation categories for each verb
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationCategory {
    Nav,
    Physics,
    Resources,
    Visibility,
    Cooldown,
}

/// Error taxonomy for tool validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    OutOfBounds,
    Cooldown,
    NoLineOfSight,
    InsufficientResource,
    InvalidTarget,
    PhysicsBlocked,
    Unknown,
}

use std::fmt;
impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ToolError::OutOfBounds => "OutOfBounds",
            ToolError::Cooldown => "Cooldown",
            ToolError::NoLineOfSight => "NoLineOfSight",
            ToolError::InsufficientResource => "InsufficientResource",
            ToolError::InvalidTarget => "InvalidTarget",
            ToolError::PhysicsBlocked => "PhysicsBlocked",
            ToolError::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

/// Validate a tool action for an agent in the world
/// Phase 0: lightweight validation using only the WorldSnapshot (no path/LOS data yet).
/// - Throw: fails on cooldown
/// - CoverFire: fails on missing ammo
/// - MoveTo/others: always OK for now (deeper checks in core validator)
pub fn validate_tool_action(
    _agent_id: u32,
    verb: ToolVerb,
    world: &WorldSnapshot,
) -> Result<()> {
    match verb {
        ToolVerb::Throw => {
            let cd = world.me.cooldowns.get(&crate::cooldowns::CooldownKey::from("throw:smoke")).copied().unwrap_or(0.0);
            if cd > 0.0 {
                return Err(anyhow::anyhow!("action blocked: cooldown (throw:smoke)") );
            }
            Ok(())
        }
        ToolVerb::CoverFire => {
            if world.me.ammo <= 0 {
                return Err(anyhow::anyhow!("action blocked: insufficient ammo for cover fire") );
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_taxonomy_works() {
        assert_eq!(ToolError::OutOfBounds.to_string(), "OutOfBounds");
    }
}
