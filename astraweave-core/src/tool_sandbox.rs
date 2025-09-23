use crate::{ActionStep, EngineError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolBlockReason {
    Cooldown,
    LineOfSight,
    PathBlocked,
    Invalid,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolBlock {
    pub reason: ToolBlockReason,
    pub tool: &'static str,
    pub msg: String,
}

/// Map engine validation errors to a stable tool taxonomy with light metadata.
pub fn map_engine_error(step: &ActionStep, err: &EngineError) -> ToolBlock {
    let tool = match step {
        ActionStep::MoveTo { .. } => "MoveTo",
        ActionStep::Throw { .. } => "Throw",
        ActionStep::CoverFire { .. } => "CoverFire",
        ActionStep::Revive { .. } => "Revive",
    };
    match err {
        EngineError::Cooldown(s) => ToolBlock { reason: ToolBlockReason::Cooldown, tool, msg: format!("cooldown: {}", s) },
        EngineError::LosBlocked => ToolBlock { reason: ToolBlockReason::LineOfSight, tool, msg: "line of sight".into() },
        EngineError::NoPath => ToolBlock { reason: ToolBlockReason::PathBlocked, tool, msg: "no path".into() },
        EngineError::InvalidAction(m) => ToolBlock { reason: ToolBlockReason::Invalid, tool, msg: m.clone() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn taxonomy_maps_correctly() {
        let step = ActionStep::MoveTo{ x: 0, y: 0 };
        let b = map_engine_error(&step, &EngineError::NoPath);
        assert_eq!(b.reason, ToolBlockReason::PathBlocked);
        assert_eq!(b.tool, "MoveTo");
        let b2 = map_engine_error(&ActionStep::Throw{ item: "smoke".into(), x:0, y:0 }, &EngineError::Cooldown("throw:smoke".into()));
        assert_eq!(b2.reason, ToolBlockReason::Cooldown);
        assert_eq!(b2.tool, "Throw");
    }
}
