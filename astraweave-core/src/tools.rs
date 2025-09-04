use crate::{IVec2, Entity};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
