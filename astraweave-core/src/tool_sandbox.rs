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
        EngineError::Resource(k) => ToolBlock { reason: ToolBlockReason::Other, tool, msg: format!("resource:{}", k) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{validation::{validate_and_execute, ValidateCfg}, world::World, IVec2, Team, PlanIntent};
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

    #[test]
    fn resource_maps_to_other() {
        let step = ActionStep::CoverFire{ target_id: 1, duration: 1.0 };
        let b = map_engine_error(&step, &EngineError::Resource("ammo".into()));
        assert_eq!(b.reason, ToolBlockReason::Other);
        assert!(b.msg.contains("ammo"));
    }

    #[test]
    fn los_and_cooldown_map_to_taxonomy() {
        // World: two entities with an obstacle between
        let mut w = World::new();
        let a = w.spawn("ally", IVec2{ x:0, y:0 }, Team{ id:1 }, 100, 10);
        let e = w.spawn("enemy", IVec2{ x:2, y:0 }, Team{ id:2 }, 50, 0);
        w.obstacles.insert((1,0)); // blocks LOS
        let cfg = ValidateCfg{ world_bounds: (-10,-10,10,10) };
        let mut log = |_s: String|{};
        // Try cover fire -> expect LOS blocked
        let intent = PlanIntent{ plan_id: "p".into(), steps: vec![ActionStep::CoverFire{ target_id: e, duration: 1.0 }] };
        let err = validate_and_execute(&mut w, a, &intent, &cfg, &mut log).unwrap_err();
        let tb = map_engine_error(intent.steps.first().unwrap(), &err);
        assert_eq!(tb.reason, ToolBlockReason::LineOfSight);
        // Cooldown: prime a cooldown and verify mapping
        let intent = PlanIntent{ plan_id: "p2".into(), steps: vec![ActionStep::Throw{ item: "smoke".into(), x:0, y:0 }] };
        // First use sets cooldown
        w.obstacles.clear();
        let _ = validate_and_execute(&mut w, a, &intent, &cfg, &mut log);
        // Second use should hit cooldown
        let err2 = validate_and_execute(&mut w, a, &intent, &cfg, &mut log).unwrap_err();
        let tb2 = map_engine_error(intent.steps.first().unwrap(), &err2);
        assert_eq!(tb2.reason, ToolBlockReason::Cooldown);
        assert!(tb2.msg.contains("throw:smoke"));
    }

    #[test]
    fn partial_failure_multi_step_stops_on_error() {
        let mut w = World::new();
        let a = w.spawn("ally", IVec2{ x:0, y:0 }, Team{ id:1 }, 100, 0);
        let e = w.spawn("enemy", IVec2{ x:3, y:0 }, Team{ id:2 }, 50, 0);
        // First step succeeds (move), second fails (cover_fire no ammo)
        let intent = PlanIntent{ plan_id: "p3".into(), steps: vec![
            ActionStep::MoveTo{ x:1, y:0 },
            ActionStep::CoverFire{ target_id: e, duration: 1.0 },
            ActionStep::MoveTo{ x:2, y:0 }, // should not execute
        ]};
        let cfg = ValidateCfg{ world_bounds: (-10,-10,10,10) };
        let mut log = |_s: String|{};
        let res = validate_and_execute(&mut w, a, &intent, &cfg, &mut log);
        assert!(matches!(res, Err(EngineError::Resource(_))));
        // Ensure the first move happened, but not the last
        assert_eq!(w.pos_of(a).unwrap(), IVec2{ x:1, y:0 });
    }
}
