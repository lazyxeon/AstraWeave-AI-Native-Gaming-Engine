use crate::{IVec2, Entity};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use glam::IVec2;

#[derive(Debug, Clone)]
pub struct Poi {
    /// Unique key or name for the POI
    pub key: String,
    /// Grid or world position of the POI
    pub position: IVec2,
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

// simple grid LOS
pub fn los_clear(obstacles: &std::collections::HashSet<(i32,i32)>, a: IVec2, b: IVec2) -> bool {
    let mut x = a.x; let mut y = a.y;
    let dx = (b.x - a.x).signum();
    let dy = (b.y - a.y).signum();
    while x != b.x || y != b.y {
        if obstacles.contains(&(x,y)) { return false; }
        if x != b.x { x += dx; }
        if y != b.y { y += dy; }
    }
    true
}

// 4-neighbor BFS for path existence
pub fn path_exists(obstacles: &HashSet<(i32,i32)>, start: IVec2, goal: IVec2, bounds: (i32,i32,i32,i32)) -> bool {
    use std::collections::VecDeque;
    let (minx, miny, maxx, maxy) = bounds;
    let mut q = VecDeque::new();
    let mut seen = HashSet::new();
    q.push_back(start); seen.insert((start.x,start.y));
    while let Some(p) = q.pop_front() {
        if p.x == goal.x && p.y == goal.y { return true; }
        for (dx,dy) in [(1,0),(-1,0),(0,1),(0,-1)] {
            let nx = p.x + dx; let ny = p.y + dy;
            if nx<minx||ny<miny||nx>maxx||ny>maxy { continue; }
            if obstacles.contains(&(nx,ny)) { continue; }
            if seen.insert((nx,ny)) { q.push_back(IVec2{x:nx,y:ny}); }
        }
    }
    false
}

use std::cmp::Ordering;

// A* grid path (4-neighbor) returning a path (including start & goal) or empty if none.
pub fn astar_path(
    obstacles: &std::collections::HashSet<(i32,i32)>,
    start: IVec2,
    goal: IVec2,
    bounds: (i32,i32,i32,i32),
) -> Vec<IVec2> {
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
    let h = |a: IVec2, b: IVec2| (a.x - b.x).abs() + (a.y - b.y).abs();
    let mut open = BinaryHeap::new();
    let mut came: HashMap<(i32,i32),(i32,i32)> = HashMap::new();
    let mut g: HashMap<(i32,i32), i32> = HashMap::new();
    let mut seen: HashSet<(i32,i32)> = HashSet::new();

    let s = (start.x,start.y);
    let t = (goal.x,goal.y);
    g.insert(s, 0);
    open.push(Node{ f: h(start, goal), x:start.x, y:start.y });
    seen.insert(s);

    while let Some(Node{f:_, x, y}) = open.pop() {
        if (x,y) == t {
            // reconstruct
            let mut path = vec![IVec2{x,y}];
            let mut cur = (x,y);
            while let Some(&prev) = came.get(&cur) {
                if prev == cur { break; }
                cur = prev;
                path.push(IVec2{ x:cur.0, y:cur.1 });
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
                let prio = ng + h(IVec2{x:nx,y:ny}, goal);
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
    from: IVec2,
    player: IVec2,
    enemy: IVec2,
    radius: i32
) -> Vec<IVec2> {
    let (minx,miny,maxx,maxy) = bounds;
    let mut out = vec![];
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let nx = from.x + dx; let ny = from.y + dy;
            if nx<minx||ny<miny||nx>maxx||ny>maxy { continue; }
            if obstacles.contains(&(nx,ny)) { continue; }
            let p = IVec2{x:nx,y:ny};
            let los_player = los_clear(obstacles, player, p);
            let los_enemy  = los_clear(obstacles, enemy,  p);
            if los_player && !los_enemy {
                out.push(p);
            }
        }
    }
    out
}
