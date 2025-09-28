pub mod capture_replay;
pub use capture_replay::*;
pub mod perception;
pub mod schema;
pub mod sim;
pub mod tool_sandbox;
pub mod tools;
pub mod util;
pub mod validation;
pub mod world;

pub use perception::*;
pub use schema::*;
pub use sim::*;
// Note: tools::Poi and schema::Poi are different types - using qualified imports where needed
pub use tool_sandbox::*;
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
