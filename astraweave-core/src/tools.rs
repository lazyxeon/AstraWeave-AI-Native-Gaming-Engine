use crate::Entity;
use serde::{Serialize, Deserialize};
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
    Throw,      // item: smoke|grenade
    CoverFire,  // duration
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
        argspecs.insert(MoveTo, vec![("x","i32"),("y","i32")]);
        argspecs.insert(Throw,  vec![("item","enum[smoke,grenade]"),("x","i32"),("y","i32")]);
        argspecs.insert(CoverFire, vec![("target_id","u32"),("duration","f32")]);
        argspecs.insert(Revive, vec![("ally_id","u32")]);
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
pub fn los_clear(obstacles: &std::collections::HashSet<(i32,i32)>, a: crate::IVec2, b: crate::IVec2) -> bool {
    // Convert schema::IVec2 to glam::IVec2
    let a_glam = schema_to_glam(a);
    let b_glam = schema_to_glam(b);
    
    let mut x = a_glam.x; let mut y = a_glam.y;
    let dx = (b_glam.x - a_glam.x).signum();
    let dy = (b_glam.y - a_glam.y).signum();
    while x != b_glam.x || y != b_glam.y {
        if obstacles.contains(&(x,y)) { return false; }
        if x != b_glam.x { x += dx; }
        if y != b_glam.y { y += dy; }
    }
    true
}

// 4-neighbor BFS for path existence
pub fn path_exists(obstacles: &HashSet<(i32,i32)>, start: crate::IVec2, goal: crate::IVec2, bounds: (i32,i32,i32,i32)) -> bool {
    // Convert schema::IVec2 to glam::IVec2
    let start_glam = schema_to_glam(start);
    let goal_glam = schema_to_glam(goal);
    
    use std::collections::VecDeque;
    let (minx, miny, maxx, maxy) = bounds;
    let mut q = VecDeque::new();
    let mut seen = HashSet::new();
    q.push_back(start_glam); seen.insert((start_glam.x,start_glam.y));
    while let Some(p) = q.pop_front() {
        if p.x == goal_glam.x && p.y == goal_glam.y { return true; }
        for (dx,dy) in [(1,0),(-1,0),(0,1),(0,-1)] {
            let nx = p.x + dx; let ny = p.y + dy;
            if nx<minx||ny<miny||nx>maxx||ny>maxy { continue; }
            if obstacles.contains(&(nx,ny)) { continue; }
            if seen.insert((nx,ny)) { q.push_back(GlamIVec2::new(nx,ny)); }
        }
    }
    false
}

use std::cmp::Ordering;

// A* grid path (4-neighbor) returning a path (including start & goal) or empty if none.
pub fn astar_path(
    obstacles: &std::collections::HashSet<(i32,i32)>,
    start: crate::IVec2,
    goal: crate::IVec2,
    bounds: (i32,i32,i32,i32),
) -> Vec<crate::IVec2> {
    // Convert schema::IVec2 to glam::IVec2
    let start_glam = schema_to_glam(start);
    let goal_glam = schema_to_glam(goal);
    
    use std::collections::{BinaryHeap, HashMap, HashSet};
    #[derive(Copy, Clone, Eq, PartialEq)]
    struct Node { f: i32, x: i32, y: i32 }
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering { other.f.cmp(&self.f) }
    }
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
    }

    let (minx,miny,maxx,maxy) = bounds;
    let h = |a: GlamIVec2, b: GlamIVec2| (a.x - b.x).abs() + (a.y - b.y).abs();
    let mut open = BinaryHeap::new();
    let mut came: HashMap<(i32,i32),(i32,i32)> = HashMap::new();
    let mut g: HashMap<(i32,i32), i32> = HashMap::new();
    let mut seen: HashSet<(i32,i32)> = HashSet::new();

    let s = (start_glam.x,start_glam.y);
    let t = (goal_glam.x,goal_glam.y);
    g.insert(s, 0);
    open.push(Node{ f: h(start_glam, goal_glam), x:start_glam.x, y:start_glam.y });
    seen.insert(s);

    while let Some(Node{f:_, x, y}) = open.pop() {
        if (x,y) == t {
            // reconstruct
            let mut path = vec![glam_to_schema(GlamIVec2::new(x,y))];
            let mut cur = (x,y);
            while let Some(&prev) = came.get(&cur) {
                if prev == cur { break; }
                cur = prev;
                path.push(glam_to_schema(GlamIVec2::new(cur.0, cur.1)));
            }
            path.reverse();
            return path;
        }
        let cur_g = *g.get(&(x,y)).unwrap_or(&i32::MAX);
        for (dx,dy) in [(1,0),(-1,0),(0,1),(0,-1)] {
            let nx = x + dx; let ny = y + dy;
            if nx<minx||ny<miny||nx>maxx||ny>maxy { continue; }
            if obstacles.contains(&(nx,ny)) { continue; }
            let ng = cur_g + 1;
            let pos = (nx,ny);
            if ng < *g.get(&pos).unwrap_or(&i32::MAX) {
                came.insert(pos, (x,y));
                g.insert(pos, ng);
                let prio = ng + h(GlamIVec2::new(nx,ny), goal_glam);
                open.push(Node{ f:prio, x:nx, y:ny });
            }
        }
    }
    vec![]
}

// Find positions within radius of `from` that have LOS from player but *not* from enemy (crude "cover")
pub fn find_cover_positions(
    obstacles: &std::collections::HashSet<(i32,i32)>,
    bounds: (i32,i32,i32,i32),
    from: crate::IVec2,
    player: crate::IVec2,
    enemy: crate::IVec2,
    radius: i32
) -> Vec<crate::IVec2> {
    // Convert schema::IVec2 to glam::IVec2
    let from_glam = schema_to_glam(from);
    let player_glam = schema_to_glam(player);
    let enemy_glam = schema_to_glam(enemy);
    
    let (minx,miny,maxx,maxy) = bounds;
    let mut out = vec![];
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let nx = from_glam.x + dx; let ny = from_glam.y + dy;
            if nx<minx||ny<miny||nx>maxx||ny>maxy { continue; }
            if obstacles.contains(&(nx,ny)) { continue; }
            let p = GlamIVec2::new(nx,ny);
            let los_player = los_clear(obstacles, player, glam_to_schema(p));
            let los_enemy  = los_clear(obstacles, enemy, glam_to_schema(p));
            if los_player && !los_enemy {
                out.push(glam_to_schema(p));
            }
        }
    }
    out
}
