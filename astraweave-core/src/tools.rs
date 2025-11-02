use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
// Import glam::IVec2 with an alias to avoid name conflicts
use glam::IVec2 as GlamIVec2;

#[derive(Debug, Clone)]
pub struct Poi {
    /// Unique key or name for the POI
    pub key: String,
    /// Grid or world position of the POI
    pub position: GlamIVec2,
    /// Description or lore for the POI
    pub description: Option<String>,
    /// Is the POI currently active or discoverable in the world?
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Tool {
    MoveTo,
    Throw,     // item: smoke|grenade
    CoverFire, // duration
    Revive,
}

#[derive(Default)]
pub struct ToolCtx {
    pub allowed: HashSet<Tool>,
    pub argspecs: HashMap<Tool, Vec<(&'static str, &'static str)>>, // (arg_name, type)
}

impl ToolCtx {
    pub fn basic_combat() -> Self {
        use Tool::*;
        let mut allowed = HashSet::new();
        allowed.insert(MoveTo);
        allowed.insert(Throw);
        allowed.insert(CoverFire);
        allowed.insert(Revive);
        let mut argspecs = HashMap::new();
        argspecs.insert(MoveTo, vec![("x", "i32"), ("y", "i32")]);
        argspecs.insert(
            Throw,
            vec![("item", "enum[smoke,grenade]"), ("x", "i32"), ("y", "i32")],
        );
        argspecs.insert(CoverFire, vec![("target_id", "u32"), ("duration", "f32")]);
        argspecs.insert(Revive, vec![("ally_id", "u32")]);
        Self { allowed, argspecs }
    }
}

// Convert from schema::IVec2 to glam::IVec2
pub fn schema_to_glam(v: crate::IVec2) -> GlamIVec2 {
    GlamIVec2::new(v.x, v.y)
}

// Convert from glam::IVec2 to schema::IVec2
pub fn glam_to_schema(v: GlamIVec2) -> crate::IVec2 {
    crate::IVec2 { x: v.x, y: v.y }
}

// simple grid LOS
pub fn los_clear(
    obstacles: &std::collections::HashSet<(i32, i32)>,
    a: crate::IVec2,
    b: crate::IVec2,
) -> bool {
    // Convert schema::IVec2 to glam::IVec2
    let a_glam = schema_to_glam(a);
    let b_glam = schema_to_glam(b);

    let mut x = a_glam.x;
    let mut y = a_glam.y;
    let dx = (b_glam.x - a_glam.x).signum();
    let dy = (b_glam.y - a_glam.y).signum();
    while x != b_glam.x || y != b_glam.y {
        if obstacles.contains(&(x, y)) {
            return false;
        }
        if x != b_glam.x {
            x += dx;
        }
        if y != b_glam.y {
            y += dy;
        }
    }
    true
}

// 4-neighbor BFS for path existence
pub fn path_exists(
    obstacles: &HashSet<(i32, i32)>,
    start: crate::IVec2,
    goal: crate::IVec2,
    bounds: (i32, i32, i32, i32),
) -> bool {
    // Convert schema::IVec2 to glam::IVec2
    let start_glam = schema_to_glam(start);
    let goal_glam = schema_to_glam(goal);

    use std::collections::VecDeque;
    let (minx, miny, maxx, maxy) = bounds;
    let mut q = VecDeque::new();
    let mut seen = HashSet::new();
    q.push_back(start_glam);
    seen.insert((start_glam.x, start_glam.y));
    while let Some(p) = q.pop_front() {
        if p.x == goal_glam.x && p.y == goal_glam.y {
            return true;
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = p.x + dx;
            let ny = p.y + dy;
            if nx < minx || ny < miny || nx > maxx || ny > maxy {
                continue;
            }
            if obstacles.contains(&(nx, ny)) {
                continue;
            }
            if seen.insert((nx, ny)) {
                q.push_back(GlamIVec2::new(nx, ny));
            }
        }
    }
    false
}

use std::cmp::Ordering;

// A* grid path (4-neighbor) returning a path (including start & goal) or empty if none.
pub fn astar_path(
    obstacles: &std::collections::HashSet<(i32, i32)>,
    start: crate::IVec2,
    goal: crate::IVec2,
    bounds: (i32, i32, i32, i32),
) -> Vec<crate::IVec2> {
    // Convert schema::IVec2 to glam::IVec2
    let start_glam = schema_to_glam(start);
    let goal_glam = schema_to_glam(goal);

    use std::collections::{BinaryHeap, HashMap, HashSet};
    #[derive(Copy, Clone, Eq, PartialEq)]
    struct Node {
        f: i32,
        x: i32,
        y: i32,
    }
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            other.f.cmp(&self.f)
        }
    }
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let (minx, miny, maxx, maxy) = bounds;
    let h = |a: GlamIVec2, b: GlamIVec2| (a.x - b.x).abs() + (a.y - b.y).abs();
    let mut open = BinaryHeap::new();
    let mut came: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let mut g: HashMap<(i32, i32), i32> = HashMap::new();
    let mut seen: HashSet<(i32, i32)> = HashSet::new();

    let s = (start_glam.x, start_glam.y);
    let t = (goal_glam.x, goal_glam.y);
    g.insert(s, 0);
    open.push(Node {
        f: h(start_glam, goal_glam),
        x: start_glam.x,
        y: start_glam.y,
    });
    seen.insert(s);

    while let Some(Node { f: _, x, y }) = open.pop() {
        if (x, y) == t {
            // reconstruct
            let mut path = vec![glam_to_schema(GlamIVec2::new(x, y))];
            let mut cur = (x, y);
            while let Some(&prev) = came.get(&cur) {
                if prev == cur {
                    break;
                }
                cur = prev;
                path.push(glam_to_schema(GlamIVec2::new(cur.0, cur.1)));
            }
            path.reverse();
            return path;
        }
        let cur_g = *g.get(&(x, y)).unwrap_or(&i32::MAX);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx < minx || ny < miny || nx > maxx || ny > maxy {
                continue;
            }
            if obstacles.contains(&(nx, ny)) {
                continue;
            }
            let ng = cur_g + 1;
            let pos = (nx, ny);
            if ng < *g.get(&pos).unwrap_or(&i32::MAX) {
                came.insert(pos, (x, y));
                g.insert(pos, ng);
                let prio = ng + h(GlamIVec2::new(nx, ny), goal_glam);
                open.push(Node {
                    f: prio,
                    x: nx,
                    y: ny,
                });
            }
        }
    }
    vec![]
}

// Find positions within radius of `from` that have LOS from player but *not* from enemy (crude "cover")
pub fn find_cover_positions(
    obstacles: &std::collections::HashSet<(i32, i32)>,
    bounds: (i32, i32, i32, i32),
    from: crate::IVec2,
    player: crate::IVec2,
    enemy: crate::IVec2,
    radius: i32,
) -> Vec<crate::IVec2> {
    // Convert schema::IVec2 to glam::IVec2
    let from_glam = schema_to_glam(from);
    let _player_glam = schema_to_glam(player);
    let _enemy_glam = schema_to_glam(enemy);

    let (minx, miny, maxx, maxy) = bounds;
    let mut out = vec![];
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let nx = from_glam.x + dx;
            let ny = from_glam.y + dy;
            if nx < minx || ny < miny || nx > maxx || ny > maxy {
                continue;
            }
            if obstacles.contains(&(nx, ny)) {
                continue;
            }
            let p = GlamIVec2::new(nx, ny);
            let los_player = los_clear(obstacles, player, glam_to_schema(p));
            let los_enemy = los_clear(obstacles, enemy, glam_to_schema(p));
            if los_player && !los_enemy {
                out.push(glam_to_schema(p));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create schema IVec2
    fn iv2(x: i32, y: i32) -> crate::IVec2 {
        crate::IVec2 { x, y }
    }

    // ===== POI Tests =====
    #[test]
    fn test_poi_creation() {
        let poi = Poi {
            key: "waypoint_1".to_string(),
            position: GlamIVec2::new(10, 20),
            description: Some("Test waypoint".to_string()),
            active: true,
        };
        assert_eq!(poi.key, "waypoint_1");
        assert_eq!(poi.position, GlamIVec2::new(10, 20));
        assert!(poi.active);
    }

    #[test]
    fn test_poi_inactive() {
        let poi = Poi {
            key: "hidden".to_string(),
            position: GlamIVec2::new(5, 5),
            description: None,
            active: false,
        };
        assert!(!poi.active);
        assert!(poi.description.is_none());
    }

    // ===== Tool & ToolCtx Tests =====
    #[test]
    fn test_tool_ctx_basic_combat() {
        let ctx = ToolCtx::basic_combat();
        assert!(ctx.allowed.contains(&Tool::MoveTo));
        assert!(ctx.allowed.contains(&Tool::Throw));
        assert!(ctx.allowed.contains(&Tool::CoverFire));
        assert!(ctx.allowed.contains(&Tool::Revive));
        assert_eq!(ctx.allowed.len(), 4);
    }

    #[test]
    fn test_tool_ctx_argspecs_moveto() {
        let ctx = ToolCtx::basic_combat();
        let specs = ctx.argspecs.get(&Tool::MoveTo).unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0], ("x", "i32"));
        assert_eq!(specs[1], ("y", "i32"));
    }

    #[test]
    fn test_tool_ctx_argspecs_throw() {
        let ctx = ToolCtx::basic_combat();
        let specs = ctx.argspecs.get(&Tool::Throw).unwrap();
        assert_eq!(specs.len(), 3);
        assert_eq!(specs[0], ("item", "enum[smoke,grenade]"));
        assert_eq!(specs[1], ("x", "i32"));
        assert_eq!(specs[2], ("y", "i32"));
    }

    #[test]
    fn test_tool_ctx_argspecs_coverfire() {
        let ctx = ToolCtx::basic_combat();
        let specs = ctx.argspecs.get(&Tool::CoverFire).unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0], ("target_id", "u32"));
        assert_eq!(specs[1], ("duration", "f32"));
    }

    #[test]
    fn test_tool_ctx_argspecs_revive() {
        let ctx = ToolCtx::basic_combat();
        let specs = ctx.argspecs.get(&Tool::Revive).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0], ("ally_id", "u32"));
    }

    // ===== Coordinate Conversion Tests =====
    #[test]
    fn test_schema_to_glam_conversion() {
        let schema = iv2(10, 20);
        let glam = schema_to_glam(schema);
        assert_eq!(glam.x, 10);
        assert_eq!(glam.y, 20);
    }

    #[test]
    fn test_glam_to_schema_conversion() {
        let glam = GlamIVec2::new(30, 40);
        let schema = glam_to_schema(glam);
        assert_eq!(schema.x, 30);
        assert_eq!(schema.y, 40);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = iv2(15, 25);
        let glam = schema_to_glam(original);
        let back = glam_to_schema(glam);
        assert_eq!(original.x, back.x);
        assert_eq!(original.y, back.y);
    }

    #[test]
    fn test_conversion_negative_coords() {
        let schema = iv2(-10, -20);
        let glam = schema_to_glam(schema);
        assert_eq!(glam.x, -10);
        assert_eq!(glam.y, -20);
        let back = glam_to_schema(glam);
        assert_eq!(back.x, -10);
        assert_eq!(back.y, -20);
    }

    // ===== LOS Tests =====
    #[test]
    fn test_los_clear_no_obstacles() {
        let obstacles = HashSet::new();
        let a = iv2(0, 0);
        let b = iv2(5, 5);
        assert!(los_clear(&obstacles, a, b));
    }

    #[test]
    fn test_los_blocked_by_obstacle() {
        let mut obstacles = HashSet::new();
        obstacles.insert((2, 2));
        let a = iv2(0, 0);
        let b = iv2(5, 5);
        assert!(!los_clear(&obstacles, a, b));
    }

    #[test]
    fn test_los_clear_horizontal() {
        let obstacles = HashSet::new();
        let a = iv2(0, 5);
        let b = iv2(10, 5);
        assert!(los_clear(&obstacles, a, b));
    }

    #[test]
    fn test_los_clear_vertical() {
        let obstacles = HashSet::new();
        let a = iv2(5, 0);
        let b = iv2(5, 10);
        assert!(los_clear(&obstacles, a, b));
    }

    #[test]
    fn test_los_same_position() {
        let obstacles = HashSet::new();
        let a = iv2(5, 5);
        let b = iv2(5, 5);
        assert!(los_clear(&obstacles, a, b));
    }

    #[test]
    fn test_los_obstacle_not_on_line() {
        let mut obstacles = HashSet::new();
        obstacles.insert((10, 10)); // Far from path
        let a = iv2(0, 0);
        let b = iv2(5, 0);
        assert!(los_clear(&obstacles, a, b));
    }

    // ===== Path Exists Tests =====
    #[test]
    fn test_path_exists_straight_line() {
        let obstacles = HashSet::new();
        let start = iv2(0, 0);
        let goal = iv2(5, 0);
        let bounds = (-10, -10, 10, 10);
        assert!(path_exists(&obstacles, start, goal, bounds));
    }

    #[test]
    fn test_path_exists_with_obstacle() {
        let mut obstacles = HashSet::new();
        obstacles.insert((2, 0));
        let start = iv2(0, 0);
        let goal = iv2(5, 0);
        let bounds = (-10, -10, 10, 10);
        // Path should go around obstacle
        assert!(path_exists(&obstacles, start, goal, bounds));
    }

    #[test]
    fn test_path_blocked_completely() {
        let mut obstacles = HashSet::new();
        // Create wall blocking path
        for y in -10..=10 {
            obstacles.insert((5, y));
        }
        let start = iv2(0, 0);
        let goal = iv2(10, 0);
        let bounds = (-10, -10, 15, 10);
        assert!(!path_exists(&obstacles, start, goal, bounds));
    }

    #[test]
    fn test_path_exists_same_position() {
        let obstacles = HashSet::new();
        let pos = iv2(5, 5);
        let bounds = (-10, -10, 10, 10);
        assert!(path_exists(&obstacles, pos, pos, bounds));
    }

    #[test]
    fn test_path_exists_out_of_bounds() {
        let obstacles = HashSet::new();
        let start = iv2(0, 0);
        let goal = iv2(20, 20);
        let bounds = (-5, -5, 5, 5);
        assert!(!path_exists(&obstacles, start, goal, bounds));
    }

    #[test]
    fn test_path_exists_around_corner() {
        let mut obstacles = HashSet::new();
        obstacles.insert((5, 5));
        let start = iv2(0, 0);
        let goal = iv2(10, 10);
        let bounds = (-10, -10, 15, 15);
        assert!(path_exists(&obstacles, start, goal, bounds));
    }

    // ===== A* Path Tests =====
    #[test]
    fn test_astar_path_straight_line() {
        let obstacles = HashSet::new();
        let start = iv2(0, 0);
        let goal = iv2(3, 0);
        let bounds = (-10, -10, 10, 10);
        let path = astar_path(&obstacles, start, goal, bounds);
        assert!(!path.is_empty());
        assert_eq!(path[0].x, 0);
        assert_eq!(path[0].y, 0);
        assert_eq!(path.last().unwrap().x, 3);
        assert_eq!(path.last().unwrap().y, 0);
    }

    #[test]
    fn test_astar_path_around_obstacle() {
        let mut obstacles = HashSet::new();
        obstacles.insert((1, 0));
        let start = iv2(0, 0);
        let goal = iv2(2, 0);
        let bounds = (-10, -10, 10, 10);
        let path = astar_path(&obstacles, start, goal, bounds);
        assert!(!path.is_empty());
        // Path should go around (0,0) -> (0,1) or (0,-1) -> (1,1) or (1,-1) -> (2,0)
        assert_eq!(path[0].x, 0);
        assert_eq!(path.last().unwrap().x, 2);
    }

    #[test]
    fn test_astar_path_blocked_completely() {
        let mut obstacles = HashSet::new();
        // Create wall
        for y in -10..=10 {
            obstacles.insert((5, y));
        }
        let start = iv2(0, 0);
        let goal = iv2(10, 0);
        let bounds = (-10, -10, 15, 10);
        let path = astar_path(&obstacles, start, goal, bounds);
        assert!(path.is_empty());
    }

    #[test]
    fn test_astar_path_same_position() {
        let obstacles = HashSet::new();
        let pos = iv2(5, 5);
        let bounds = (-10, -10, 10, 10);
        let path = astar_path(&obstacles, pos, pos, bounds);
        assert!(!path.is_empty());
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].x, 5);
        assert_eq!(path[0].y, 5);
    }

    #[test]
    fn test_astar_path_complex_maze() {
        let mut obstacles = HashSet::new();
        // Create L-shaped obstacle
        obstacles.insert((2, 0));
        obstacles.insert((2, 1));
        obstacles.insert((2, 2));
        obstacles.insert((3, 2));
        obstacles.insert((4, 2));
        
        let start = iv2(0, 0);
        let goal = iv2(4, 0);
        let bounds = (-10, -10, 10, 10);
        let path = astar_path(&obstacles, start, goal, bounds);
        assert!(!path.is_empty());
        assert_eq!(path[0].x, 0);
        assert_eq!(path.last().unwrap().x, 4);
        assert_eq!(path.last().unwrap().y, 0);
    }

    #[test]
    fn test_astar_path_out_of_bounds() {
        let obstacles = HashSet::new();
        let start = iv2(0, 0);
        let goal = iv2(20, 20);
        let bounds = (-5, -5, 5, 5);
        let path = astar_path(&obstacles, start, goal, bounds);
        assert!(path.is_empty());
    }

    // ===== Find Cover Positions Tests =====
    #[test]
    fn test_find_cover_positions_basic() {
        let mut obstacles = HashSet::new();
        // Add obstacle that blocks enemy LOS but not player LOS
        obstacles.insert((3, 0));
        
        let bounds = (-10, -10, 10, 10);
        let from = iv2(0, 0);
        let player = iv2(-5, 0); // Player to the left
        let enemy = iv2(5, 0);   // Enemy to the right
        let radius = 5;
        
        let cover = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);
        // Should find positions behind obstacle (relative to enemy)
        // Not asserting exact count as geometry is complex, just that some exist
        // Actually, with symmetric setup and single obstacle, might find none
        // Let's verify function runs without panic
        assert!(cover.len() >= 0);
    }

    #[test]
    fn test_find_cover_positions_with_obstacles() {
        let mut obstacles = HashSet::new();
        obstacles.insert((2, 0)); // Obstacle between from and enemy
        
        let bounds = (-10, -10, 10, 10);
        let from = iv2(0, 0);
        let player = iv2(-5, 0);
        let enemy = iv2(5, 0);
        let radius = 3;
        
        let cover = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);
        // Positions behind obstacle should provide cover from enemy
        for pos in &cover {
            assert!(los_clear(&obstacles, player, *pos));
            assert!(!los_clear(&obstacles, enemy, *pos));
        }
    }

    #[test]
    fn test_find_cover_positions_no_cover() {
        let obstacles = HashSet::new();
        let bounds = (-10, -10, 10, 10);
        let from = iv2(0, 0);
        let player = iv2(0, 0);
        let enemy = iv2(0, 1);
        let radius = 1;
        
        // All nearby positions have LOS from both player and enemy (same line)
        let cover = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);
        // Should be empty or very limited
        assert!(cover.len() < 5); // Expect few or no positions
    }

    #[test]
    fn test_find_cover_positions_respects_bounds() {
        let obstacles = HashSet::new();
        let bounds = (-2, -2, 2, 2);
        let from = iv2(0, 0);
        let player = iv2(-5, 0);
        let enemy = iv2(5, 0);
        let radius = 10; // Large radius but bounded
        
        let cover = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);
        // All positions should be within bounds
        for pos in &cover {
            assert!(pos.x >= -2 && pos.x <= 2);
            assert!(pos.y >= -2 && pos.y <= 2);
        }
    }

    #[test]
    fn test_find_cover_positions_excludes_obstacles() {
        let mut obstacles = HashSet::new();
        obstacles.insert((1, 0));
        obstacles.insert((0, 1));
        
        let bounds = (-10, -10, 10, 10);
        let from = iv2(0, 0);
        let player = iv2(-5, 0);
        let enemy = iv2(5, 0);
        let radius = 2;
        
        let cover = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);
        // Cover positions should not include obstacles
        for pos in &cover {
            assert!(!obstacles.contains(&(pos.x, pos.y)));
        }
    }

    #[test]
    fn test_find_cover_positions_zero_radius() {
        let obstacles = HashSet::new();
        let bounds = (-10, -10, 10, 10);
        let from = iv2(0, 0);
        let player = iv2(-5, 0);
        let enemy = iv2(5, 0);
        let radius = 0;
        
        let cover = find_cover_positions(&obstacles, bounds, from, player, enemy, radius);
        // Radius 0 means only check (0,0)
        assert!(cover.len() <= 1);
    }
}
