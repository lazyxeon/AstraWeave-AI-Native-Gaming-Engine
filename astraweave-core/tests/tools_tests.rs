//! Comprehensive tests for astraweave-core/src/tools.rs
//! Tests cover: ToolCtx, coordinate conversion, LOS, pathfinding (BFS/A*), cover positions

use astraweave_core::tools::{
    astar_path, find_cover_positions, glam_to_schema, los_clear, path_exists, schema_to_glam,
    Poi, Tool, ToolCtx,
};
use astraweave_core::IVec2;
use std::collections::HashSet;

// ============================================================================
// ToolCtx Tests
// ============================================================================

#[test]
fn test_tool_ctx_basic_combat() {
    let ctx = ToolCtx::basic_combat();

    // Verify all 4 tools are allowed
    assert!(ctx.allowed.contains(&Tool::MoveTo));
    assert!(ctx.allowed.contains(&Tool::Throw));
    assert!(ctx.allowed.contains(&Tool::CoverFire));
    assert!(ctx.allowed.contains(&Tool::Revive));
    assert_eq!(ctx.allowed.len(), 4);

    // Verify argspecs exist for each tool
    assert!(ctx.argspecs.contains_key(&Tool::MoveTo));
    assert!(ctx.argspecs.contains_key(&Tool::Throw));
    assert!(ctx.argspecs.contains_key(&Tool::CoverFire));
    assert!(ctx.argspecs.contains_key(&Tool::Revive));

    // Verify MoveTo argspecs
    let moveto_args = ctx.argspecs.get(&Tool::MoveTo).unwrap();
    assert_eq!(moveto_args.len(), 2);
    assert_eq!(moveto_args[0], ("x", "i32"));
    assert_eq!(moveto_args[1], ("y", "i32"));

    // Verify Throw argspecs
    let throw_args = ctx.argspecs.get(&Tool::Throw).unwrap();
    assert_eq!(throw_args.len(), 3);
    assert_eq!(throw_args[0], ("item", "enum[smoke,grenade]"));
    assert_eq!(throw_args[1], ("x", "i32"));
    assert_eq!(throw_args[2], ("y", "i32"));

    // Verify CoverFire argspecs
    let coverfire_args = ctx.argspecs.get(&Tool::CoverFire).unwrap();
    assert_eq!(coverfire_args.len(), 2);
    assert_eq!(coverfire_args[0], ("target_id", "u32"));
    assert_eq!(coverfire_args[1], ("duration", "f32"));

    // Verify Revive argspecs
    let revive_args = ctx.argspecs.get(&Tool::Revive).unwrap();
    assert_eq!(revive_args.len(), 1);
    assert_eq!(revive_args[0], ("ally_id", "u32"));
}

// ============================================================================
// Coordinate Conversion Tests
// ============================================================================

#[test]
fn test_schema_to_glam_conversion() {
    let schema_pos = IVec2 { x: 10, y: 20 };
    let glam_pos = schema_to_glam(schema_pos);

    assert_eq!(glam_pos.x, 10);
    assert_eq!(glam_pos.y, 20);
}

#[test]
fn test_glam_to_schema_conversion() {
    let glam_pos = glam::IVec2::new(15, 25);
    let schema_pos = glam_to_schema(glam_pos);

    assert_eq!(schema_pos.x, 15);
    assert_eq!(schema_pos.y, 25);
}

#[test]
fn test_coordinate_roundtrip() {
    let original = IVec2 { x: 42, y: -17 };
    let glam = schema_to_glam(original);
    let back = glam_to_schema(glam);

    assert_eq!(original.x, back.x);
    assert_eq!(original.y, back.y);
}

// ============================================================================
// Line-of-Sight (LOS) Tests
// ============================================================================

#[test]
fn test_los_clear_no_obstacles() {
    let obstacles = HashSet::new();
    let a = IVec2 { x: 0, y: 0 };
    let b = IVec2 { x: 5, y: 5 };

    assert!(los_clear(&obstacles, a, b));
}

#[test]
fn test_los_blocked_by_obstacle() {
    let mut obstacles = HashSet::new();
    obstacles.insert((2, 2));

    let a = IVec2 { x: 0, y: 0 };
    let b = IVec2 { x: 5, y: 5 };

    // LOS from (0,0) to (5,5) passes through (2,2)
    assert!(!los_clear(&obstacles, a, b));
}

#[test]
fn test_los_horizontal() {
    let obstacles = HashSet::new();
    let a = IVec2 { x: 0, y: 5 };
    let b = IVec2 { x: 10, y: 5 };

    assert!(los_clear(&obstacles, a, b));
}

#[test]
fn test_los_vertical() {
    let obstacles = HashSet::new();
    let a = IVec2 { x: 5, y: 0 };
    let b = IVec2 { x: 5, y: 10 };

    assert!(los_clear(&obstacles, a, b));
}

// ============================================================================
// Pathfinding (BFS) Tests
// ============================================================================

#[test]
fn test_path_exists_no_obstacles() {
    let obstacles = HashSet::new();
    let start = IVec2 { x: 0, y: 0 };
    let goal = IVec2 { x: 5, y: 5 };
    let bounds = (0, 0, 10, 10);

    assert!(path_exists(&obstacles, start, goal, bounds));
}

#[test]
fn test_path_exists_with_obstacles() {
    let mut obstacles = HashSet::new();
    // Create L-shaped path
    obstacles.insert((3, 3));

    let start = IVec2 { x: 0, y: 0 };
    let goal = IVec2 { x: 5, y: 5 };
    let bounds = (0, 0, 10, 10);

    assert!(path_exists(&obstacles, start, goal, bounds));
}

#[test]
fn test_path_blocked_fully() {
    let mut obstacles = HashSet::new();
    // Create wall blocking path
    for x in 0..=10 {
        obstacles.insert((x, 5));
    }

    let start = IVec2 { x: 5, y: 0 };
    let goal = IVec2 { x: 5, y: 10 };
    let bounds = (0, 0, 10, 10);

    assert!(!path_exists(&obstacles, start, goal, bounds));
}

#[test]
fn test_path_exists_same_position() {
    let obstacles = HashSet::new();
    let pos = IVec2 { x: 5, y: 5 };
    let bounds = (0, 0, 10, 10);

    assert!(path_exists(&obstacles, pos, pos, bounds));
}

// ============================================================================
// A* Pathfinding Tests
// ============================================================================

#[test]
fn test_astar_straight_path() {
    let obstacles = HashSet::new();
    let start = IVec2 { x: 0, y: 0 };
    let goal = IVec2 { x: 3, y: 0 };
    let bounds = (0, 0, 10, 10);

    let path = astar_path(&obstacles, start, goal, bounds);

    assert!(!path.is_empty());
    assert_eq!(path[0], start);
    assert_eq!(path[path.len() - 1], goal);
    // Path should be 4 steps: (0,0) -> (1,0) -> (2,0) -> (3,0)
    assert_eq!(path.len(), 4);
}

#[test]
fn test_astar_with_obstacle() {
    let mut obstacles = HashSet::new();
    obstacles.insert((1, 0)); // Block direct path

    let start = IVec2 { x: 0, y: 0 };
    let goal = IVec2 { x: 2, y: 0 };
    let bounds = (0, 0, 10, 10);

    let path = astar_path(&obstacles, start, goal, bounds);

    assert!(!path.is_empty());
    assert_eq!(path[0], start);
    assert_eq!(path[path.len() - 1], goal);
    // Path must go around obstacle (e.g., (0,0) -> (0,1) -> (1,1) -> (2,1) -> (2,0))
    assert!(path.len() > 3); // Longer than direct path
}

#[test]
fn test_astar_no_path_available() {
    let mut obstacles = HashSet::new();
    // Create complete wall
    for y in 0..=10 {
        obstacles.insert((5, y));
    }

    let start = IVec2 { x: 0, y: 5 };
    let goal = IVec2 { x: 10, y: 5 };
    let bounds = (0, 0, 10, 10);

    let path = astar_path(&obstacles, start, goal, bounds);

    assert!(path.is_empty());
}

#[test]
fn test_astar_same_position() {
    let obstacles = HashSet::new();
    let pos = IVec2 { x: 5, y: 5 };
    let bounds = (0, 0, 10, 10);

    let path = astar_path(&obstacles, pos, pos, bounds);

    assert_eq!(path.len(), 1);
    assert_eq!(path[0], pos);
}

#[test]
fn test_astar_l_shaped_path() {
    let obstacles = HashSet::new();
    let start = IVec2 { x: 0, y: 0 };
    let goal = IVec2 { x: 3, y: 3 };
    let bounds = (0, 0, 10, 10);

    let path = astar_path(&obstacles, start, goal, bounds);

    assert!(!path.is_empty());
    assert_eq!(path[0], start);
    assert_eq!(path[path.len() - 1], goal);
    // Manhattan distance is 6, so path should be 7 steps (including start)
    assert_eq!(path.len(), 7);
}

// ============================================================================
// Cover Position Tests
// ============================================================================

#[test]
fn test_find_cover_positions_simple() {
    let mut obstacles = HashSet::new();
    obstacles.insert((5, 5)); // Single wall

    let from = IVec2 { x: 4, y: 5 };
    let player = IVec2 { x: 0, y: 5 };
    let enemy = IVec2 { x: 10, y: 5 };
    let bounds = (0, 0, 10, 10);
    let radius = 2;

    let cover_positions = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);

    // Position (4, 5) has wall at (5, 5) blocking enemy LOS
    // Should find positions with player LOS but not enemy LOS
    assert!(!cover_positions.is_empty());
}

#[test]
fn test_find_cover_no_cover_available() {
    let obstacles = HashSet::new(); // No walls

    let from = IVec2 { x: 5, y: 5 };
    let player = IVec2 { x: 0, y: 5 };
    let enemy = IVec2 { x: 10, y: 5 };
    let bounds = (0, 0, 10, 10);
    let radius = 2;

    let cover_positions = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);

    // With no obstacles, no position can block enemy LOS while maintaining player LOS
    assert!(cover_positions.is_empty());
}

#[test]
fn test_find_cover_respects_radius() {
    let mut obstacles = HashSet::new();
    obstacles.insert((5, 5));

    let from = IVec2 { x: 5, y: 5 };
    let player = IVec2 { x: 0, y: 5 };
    let enemy = IVec2 { x: 10, y: 5 };
    let bounds = (0, 0, 20, 20);
    let radius = 1;

    let cover_positions = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);

    // All positions should be within radius=1 of (5, 5)
    for pos in cover_positions {
        let dx = (pos.x - from.x).abs();
        let dy = (pos.y - from.y).abs();
        assert!(dx <= radius && dy <= radius);
    }
}

// ============================================================================
// Poi Tests
// ============================================================================

#[test]
fn test_poi_creation() {
    let poi = Poi {
        key: "test_poi".to_string(),
        position: glam::IVec2::new(10, 20),
        description: Some("Test point of interest".to_string()),
        active: true,
    };

    assert_eq!(poi.key, "test_poi");
    assert_eq!(poi.position.x, 10);
    assert_eq!(poi.position.y, 20);
    assert!(poi.description.is_some());
    assert!(poi.active);
}

#[test]
fn test_poi_inactive() {
    let poi = Poi {
        key: "inactive_poi".to_string(),
        position: glam::IVec2::new(5, 5),
        description: None,
        active: false,
    };

    assert!(!poi.active);
    assert!(poi.description.is_none());
}
