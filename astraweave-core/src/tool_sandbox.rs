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
        // Movement
        ActionStep::MoveTo { .. } => "MoveTo",
        ActionStep::Approach { .. } => "Approach",
        ActionStep::Retreat { .. } => "Retreat",
        ActionStep::TakeCover { .. } => "TakeCover",
        ActionStep::Strafe { .. } => "Strafe",
        ActionStep::Patrol { .. } => "Patrol",
        
        // Offensive
        ActionStep::Attack { .. } => "Attack",
        ActionStep::AimedShot { .. } => "AimedShot",
        ActionStep::QuickAttack { .. } => "QuickAttack",
        ActionStep::HeavyAttack { .. } => "HeavyAttack",
        ActionStep::AoEAttack { .. } => "AoEAttack",
        ActionStep::ThrowExplosive { .. } => "ThrowExplosive",
        ActionStep::CoverFire { .. } => "CoverFire",
        ActionStep::Charge { .. } => "Charge",
        
        // Defensive
        ActionStep::Block => "Block",
        ActionStep::Dodge { .. } => "Dodge",
        ActionStep::Parry => "Parry",
        ActionStep::ThrowSmoke { .. } => "ThrowSmoke",
        ActionStep::Heal { .. } => "Heal",
        ActionStep::UseDefensiveAbility { .. } => "UseDefensiveAbility",
        
        // Equipment
        ActionStep::EquipWeapon { .. } => "EquipWeapon",
        ActionStep::SwitchWeapon { .. } => "SwitchWeapon",
        ActionStep::Reload => "Reload",
        ActionStep::UseItem { .. } => "UseItem",
        ActionStep::DropItem { .. } => "DropItem",
        
        // Tactical
        ActionStep::CallReinforcements { .. } => "CallReinforcements",
        ActionStep::MarkTarget { .. } => "MarkTarget",
        ActionStep::RequestCover { .. } => "RequestCover",
        ActionStep::CoordinateAttack { .. } => "CoordinateAttack",
        ActionStep::SetAmbush { .. } => "SetAmbush",
        ActionStep::Distract { .. } => "Distract",
        ActionStep::Regroup { .. } => "Regroup",
        
        // Utility
        ActionStep::Scan { .. } => "Scan",
        ActionStep::Wait { .. } => "Wait",
        ActionStep::Interact { .. } => "Interact",
        ActionStep::UseAbility { .. } => "UseAbility",
        ActionStep::Taunt { .. } => "Taunt",
        
        // Legacy
        ActionStep::Throw { .. } => "Throw",
        ActionStep::Revive { .. } => "Revive",
    };
    
    match err {
        EngineError::Cooldown(s) => ToolBlock {
            reason: ToolBlockReason::Cooldown,
            tool,
            msg: format!("cooldown: {}", s),
        },
        EngineError::LosBlocked => ToolBlock {
            reason: ToolBlockReason::LineOfSight,
            tool,
            msg: "line of sight".into(),
        },
        EngineError::NoPath => ToolBlock {
            reason: ToolBlockReason::PathBlocked,
            tool,
            msg: "no path".into(),
        },
        EngineError::InvalidAction(m) => ToolBlock {
            reason: ToolBlockReason::Invalid,
            tool,
            msg: m.clone(),
        },
        EngineError::Resource(k) => ToolBlock {
            reason: ToolBlockReason::Other,
            tool,
            msg: format!("resource:{}", k),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        validation::{validate_and_execute, ValidateCfg},
        world::World,
        IVec2, PlanIntent, Team,
    };
    #[test]
    fn taxonomy_maps_correctly() {
        let step = ActionStep::MoveTo { x: 0, y: 0, speed: None };
        let b = map_engine_error(&step, &EngineError::NoPath);
        assert_eq!(b.reason, ToolBlockReason::PathBlocked);
        assert_eq!(b.tool, "MoveTo");
        let b2 = map_engine_error(
            &ActionStep::Throw {
                item: "smoke".into(),
                x: 0,
                y: 0,
            },
            &EngineError::Cooldown("throw:smoke".into()),
        );
        assert_eq!(b2.reason, ToolBlockReason::Cooldown);
        assert_eq!(b2.tool, "Throw");
    }

    #[test]
    fn resource_maps_to_other() {
        let step = ActionStep::CoverFire {
            target_id: 1,
            duration: 1.0,
        };
        let b = map_engine_error(&step, &EngineError::Resource("ammo".into()));
        assert_eq!(b.reason, ToolBlockReason::Other);
        assert!(b.msg.contains("ammo"));
    }

    #[test]
    fn los_and_cooldown_map_to_taxonomy() {
        // World: two entities with an obstacle between
        let mut w = World::new();
        let a = w.spawn("ally", IVec2 { x: 0, y: 0 }, Team { id: 1 }, 100, 10);
        let e = w.spawn("enemy", IVec2 { x: 2, y: 0 }, Team { id: 2 }, 50, 0);
        w.obstacles.insert((1, 0)); // blocks LOS
        let cfg = ValidateCfg {
            world_bounds: (-10, -10, 10, 10),
        };
        let mut log = |_s: String| {};
        // Try cover fire -> expect LOS blocked
        let intent = PlanIntent {
            plan_id: "p".into(),
            steps: vec![ActionStep::CoverFire {
                target_id: e,
                duration: 1.0,
            }],
        };
        let err = validate_and_execute(&mut w, a, &intent, &cfg, &mut log).unwrap_err();
        let tb = map_engine_error(intent.steps.first().unwrap(), &err);
        assert_eq!(tb.reason, ToolBlockReason::LineOfSight);
        // Cooldown: prime a cooldown and verify mapping
        let intent = PlanIntent {
            plan_id: "p2".into(),
            steps: vec![ActionStep::Throw {
                item: "smoke".into(),
                x: 0,
                y: 0,
            }],
        };
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
        let a = w.spawn("ally", IVec2 { x: 0, y: 0 }, Team { id: 1 }, 100, 0);
        let e = w.spawn("enemy", IVec2 { x: 3, y: 0 }, Team { id: 2 }, 50, 0);
        // First step succeeds (move), second fails (cover_fire no ammo)
        let intent = PlanIntent {
            plan_id: "p3".into(),
            steps: vec![
                ActionStep::MoveTo { x: 1, y: 0, speed: None },
                ActionStep::CoverFire {
                    target_id: e,
                    duration: 1.0,
                },
                ActionStep::MoveTo { x: 2, y: 0, speed: None }, // should not execute
            ],
        };
        let cfg = ValidateCfg {
            world_bounds: (-10, -10, 10, 10),
        };
        let mut log = |_s: String| {};
        let res = validate_and_execute(&mut w, a, &intent, &cfg, &mut log);
        assert!(matches!(res, Err(EngineError::Resource(_))));
        // Ensure the first move happened, but not the last
        assert_eq!(w.pos_of(a).unwrap(), IVec2 { x: 1, y: 0 });
    }

    // ===== Comprehensive ActionStep Mapping Tests =====
    
    #[test]
    fn test_map_movement_actions() {
        let err = EngineError::NoPath;
        
        let steps = vec![
            (ActionStep::MoveTo { x: 0, y: 0, speed: None }, "MoveTo"),
            (ActionStep::Approach { target_id: 1, distance: 5.0 }, "Approach"),
            (ActionStep::Retreat { target_id: 1, distance: 10.0 }, "Retreat"),
            (ActionStep::TakeCover { position: None }, "TakeCover"),
            (ActionStep::Strafe { target_id: 1, direction: crate::schema::StrafeDirection::Left }, "Strafe"),
            (ActionStep::Patrol { waypoints: vec![] }, "Patrol"),
        ];
        
        for (step, expected_name) in steps {
            let block = map_engine_error(&step, &err);
            assert_eq!(block.tool, expected_name);
            assert_eq!(block.reason, ToolBlockReason::PathBlocked);
        }
    }

    #[test]
    fn test_map_offensive_actions() {
        let err = EngineError::LosBlocked;
        
        let steps = vec![
            (ActionStep::Attack { target_id: 1 }, "Attack"),
            (ActionStep::AimedShot { target_id: 1 }, "AimedShot"),
            (ActionStep::QuickAttack { target_id: 1 }, "QuickAttack"),
            (ActionStep::HeavyAttack { target_id: 1 }, "HeavyAttack"),
            (ActionStep::AoEAttack { x: 0, y: 0, radius: 3.0 }, "AoEAttack"),
            (ActionStep::ThrowExplosive { x: 5, y: 5 }, "ThrowExplosive"),
            (ActionStep::CoverFire { target_id: 1, duration: 2.0 }, "CoverFire"),
            (ActionStep::Charge { target_id: 1 }, "Charge"),
        ];
        
        for (step, expected_name) in steps {
            let block = map_engine_error(&step, &err);
            assert_eq!(block.tool, expected_name);
            assert_eq!(block.reason, ToolBlockReason::LineOfSight);
            assert_eq!(block.msg, "line of sight");
        }
    }

    #[test]
    fn test_map_defensive_actions() {
        let err = EngineError::Cooldown("defensive".into());
        
        let steps = vec![
            (ActionStep::Block, "Block"),
            (ActionStep::Dodge { direction: None }, "Dodge"),
            (ActionStep::Parry, "Parry"),
            (ActionStep::ThrowSmoke { x: 0, y: 0 }, "ThrowSmoke"),
            (ActionStep::Heal { target_id: None }, "Heal"),
            (ActionStep::UseDefensiveAbility { ability_name: "shield".into() }, "UseDefensiveAbility"),
        ];
        
        for (step, expected_name) in steps {
            let block = map_engine_error(&step, &err);
            assert_eq!(block.tool, expected_name);
            assert_eq!(block.reason, ToolBlockReason::Cooldown);
            assert!(block.msg.contains("defensive"));
        }
    }

    #[test]
    fn test_map_equipment_actions() {
        let err = EngineError::InvalidAction("invalid".into());
        
        let steps = vec![
            (ActionStep::EquipWeapon { weapon_name: "rifle".into() }, "EquipWeapon"),
            (ActionStep::SwitchWeapon { slot: 2 }, "SwitchWeapon"),
            (ActionStep::Reload, "Reload"),
            (ActionStep::UseItem { item_name: "medkit".into() }, "UseItem"),
            (ActionStep::DropItem { item_name: "ammo".into() }, "DropItem"),
        ];
        
        for (step, expected_name) in steps {
            let block = map_engine_error(&step, &err);
            assert_eq!(block.tool, expected_name);
            assert_eq!(block.reason, ToolBlockReason::Invalid);
            assert_eq!(block.msg, "invalid");
        }
    }

    #[test]
    fn test_map_tactical_actions() {
        let err = EngineError::Resource("tactical".into());
        
        let steps = vec![
            (ActionStep::CallReinforcements { count: 3 }, "CallReinforcements"),
            (ActionStep::MarkTarget { target_id: 1 }, "MarkTarget"),
            (ActionStep::RequestCover { duration: 5.0 }, "RequestCover"),
            (ActionStep::CoordinateAttack { target_id: 1 }, "CoordinateAttack"),
            (ActionStep::SetAmbush { position: IVec2 { x: 5, y: 5 } }, "SetAmbush"),
            (ActionStep::Distract { target_id: 1 }, "Distract"),
            (ActionStep::Regroup { rally_point: IVec2 { x: 0, y: 0 } }, "Regroup"),
        ];
        
        for (step, expected_name) in steps {
            let block = map_engine_error(&step, &err);
            assert_eq!(block.tool, expected_name);
            assert_eq!(block.reason, ToolBlockReason::Other);
            assert!(block.msg.contains("tactical"));
        }
    }

    #[test]
    fn test_map_utility_actions() {
        let err = EngineError::NoPath;
        
        let steps = vec![
            (ActionStep::Scan { radius: 10.0 }, "Scan"),
            (ActionStep::Wait { duration: 2.0 }, "Wait"),
            (ActionStep::Interact { target_id: 1 }, "Interact"),
            (ActionStep::UseAbility { ability_name: "stealth".into() }, "UseAbility"),
            (ActionStep::Taunt { target_id: 1 }, "Taunt"),
        ];
        
        for (step, expected_name) in steps {
            let block = map_engine_error(&step, &err);
            assert_eq!(block.tool, expected_name);
            assert_eq!(block.reason, ToolBlockReason::PathBlocked);
        }
    }

    #[test]
    fn test_map_legacy_actions() {
        let err = EngineError::Cooldown("legacy".into());
        
        let steps = vec![
            (ActionStep::Throw { item: "grenade".into(), x: 5, y: 5 }, "Throw"),
            (ActionStep::Revive { ally_id: 1 }, "Revive"),
        ];
        
        for (step, expected_name) in steps {
            let block = map_engine_error(&step, &err);
            assert_eq!(block.tool, expected_name);
            assert_eq!(block.reason, ToolBlockReason::Cooldown);
        }
    }

    #[test]
    fn test_all_error_types() {
        let step = ActionStep::Attack { target_id: 1 };
        
        // Test Cooldown
        let block = map_engine_error(&step, &EngineError::Cooldown("attack".into()));
        assert_eq!(block.reason, ToolBlockReason::Cooldown);
        assert!(block.msg.contains("attack"));
        
        // Test LosBlocked
        let block = map_engine_error(&step, &EngineError::LosBlocked);
        assert_eq!(block.reason, ToolBlockReason::LineOfSight);
        assert_eq!(block.msg, "line of sight");
        
        // Test NoPath
        let block = map_engine_error(&step, &EngineError::NoPath);
        assert_eq!(block.reason, ToolBlockReason::PathBlocked);
        assert_eq!(block.msg, "no path");
        
        // Test InvalidAction
        let block = map_engine_error(&step, &EngineError::InvalidAction("test message".into()));
        assert_eq!(block.reason, ToolBlockReason::Invalid);
        assert_eq!(block.msg, "test message");
        
        // Test Resource
        let block = map_engine_error(&step, &EngineError::Resource("energy".into()));
        assert_eq!(block.reason, ToolBlockReason::Other);
        assert!(block.msg.contains("energy"));
    }

    #[test]
    fn test_tool_block_equality() {
        let block1 = ToolBlock {
            reason: ToolBlockReason::Cooldown,
            tool: "Attack",
            msg: "test".into(),
        };
        
        let block2 = ToolBlock {
            reason: ToolBlockReason::Cooldown,
            tool: "Attack",
            msg: "test".into(),
        };
        
        assert_eq!(block1, block2);
    }

    #[test]
    fn test_tool_block_debug() {
        let block = ToolBlock {
            reason: ToolBlockReason::LineOfSight,
            tool: "CoverFire",
            msg: "blocked".into(),
        };
        
        let debug_str = format!("{:?}", block);
        assert!(debug_str.contains("LineOfSight"));
        assert!(debug_str.contains("CoverFire"));
        assert!(debug_str.contains("blocked"));
    }

    #[test]
    fn test_tool_block_clone() {
        let block = ToolBlock {
            reason: ToolBlockReason::Invalid,
            tool: "MoveTo",
            msg: "out of bounds".into(),
        };
        
        let cloned = block.clone();
        assert_eq!(block, cloned);
    }
}

