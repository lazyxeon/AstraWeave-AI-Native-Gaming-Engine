//! Tool Sandbox: Validated action verbs and error taxonomy

use anyhow::Result;

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
pub fn validate_tool_action(
    agent_id: u32,
    verb: ToolVerb,
    world: &crate::WorldSnapshot,
) -> Result<(), ToolError> {
    // TODO: Implement validation logic per verb/category
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_taxonomy_works() {
        assert_eq!(ToolError::OutOfBounds.to_string(), "OutOfBounds");
    }
}
