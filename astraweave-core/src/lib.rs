pub mod capture_replay;
pub use capture_replay::*;
pub mod ecs_adapter;
pub mod ecs_bridge;
pub mod ecs_components;
pub mod ecs_events;
pub mod metrics;
pub mod perception;
pub mod schema;
pub mod sim;
pub mod tool_sandbox;
pub mod tool_vocabulary;
pub mod tools;
pub mod util;
pub mod validation;
pub mod world;

pub use perception::*;
pub use schema::*;
pub use sim::*;
// Note: tools::Poi and schema::Poi are different types - using qualified imports where needed
pub use ecs_components::*;
pub use tool_sandbox::*;
pub use tool_vocabulary::*;
pub use tools::{
    astar_path, find_cover_positions, glam_to_schema, los_clear, path_exists, schema_to_glam,
};
pub use validation::*;
pub use world::*;

/// Construct a default ToolRegistry matching MVP verbs.
pub fn default_tool_registry() -> ToolRegistry {
    use std::collections::BTreeMap;
    ToolRegistry {
        tools: vec![
            ToolSpec {
                name: "move_to".into(),
                args: BTreeMap::from([("x".into(), "i32".into()), ("y".into(), "i32".into())]),
            },
            ToolSpec {
                name: "throw".into(),
                args: BTreeMap::from([
                    ("item".into(), "enum[smoke,grenade]".into()),
                    ("x".into(), "i32".into()),
                    ("y".into(), "i32".into()),
                ]),
            },
            ToolSpec {
                name: "cover_fire".into(),
                args: BTreeMap::from([
                    ("target_id".into(), "u32".into()),
                    ("duration".into(), "f32".into()),
                ]),
            },
            ToolSpec {
                name: "revive".into(),
                args: BTreeMap::from([("ally_id".into(), "u32".into())]),
            },
        ],
        constraints: Constraints {
            enforce_cooldowns: true,
            enforce_los: true,
            enforce_stamina: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tool_registry_has_four_tools() {
        let registry = default_tool_registry();
        assert_eq!(registry.tools.len(), 4);
    }

    #[test]
    fn test_default_tool_registry_move_to() {
        let registry = default_tool_registry();
        let move_to = registry.tools.iter().find(|t| t.name == "move_to");
        assert!(move_to.is_some());
        let tool = move_to.unwrap();
        assert_eq!(tool.args.get("x"), Some(&"i32".to_string()));
        assert_eq!(tool.args.get("y"), Some(&"i32".to_string()));
    }

    #[test]
    fn test_default_tool_registry_throw() {
        let registry = default_tool_registry();
        let throw = registry.tools.iter().find(|t| t.name == "throw");
        assert!(throw.is_some());
        let tool = throw.unwrap();
        assert_eq!(
            tool.args.get("item"),
            Some(&"enum[smoke,grenade]".to_string())
        );
    }

    #[test]
    fn test_default_tool_registry_constraints() {
        let registry = default_tool_registry();
        assert!(registry.constraints.enforce_cooldowns);
        assert!(registry.constraints.enforce_los);
        assert!(registry.constraints.enforce_stamina);
    }

    #[test]
    fn test_default_tool_registry_cover_fire() {
        let registry = default_tool_registry();
        let cover_fire = registry.tools.iter().find(|t| t.name == "cover_fire");
        assert!(cover_fire.is_some());
    }

    #[test]
    fn test_default_tool_registry_revive() {
        let registry = default_tool_registry();
        let revive = registry.tools.iter().find(|t| t.name == "revive");
        assert!(revive.is_some());
    }
}
