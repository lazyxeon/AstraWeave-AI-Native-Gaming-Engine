use crate::{Entity, IVec2};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub struct Health { pub hp: i32 }

#[derive(Clone, Copy, Debug)]
pub struct Team { pub id: u8 } // 0: player, 1: companion, 2: enemy

#[derive(Clone, Copy, Debug)]
pub struct Ammo { pub rounds: i32 }

#[derive(Clone, Debug)]
pub struct Cooldowns {
    pub map: HashMap<String, f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Pose { pub pos: IVec2 }

#[derive(Default)]
pub struct World {
    pub t: f32,
    pub next_id: Entity,
    pub obstacles: HashSet<(i32,i32)>,
    poses: HashMap<Entity, Pose>,
    health: HashMap<Entity, Health>,
    team: HashMap<Entity, Team>,
    ammo: HashMap<Entity, Ammo>,
    cds: HashMap<Entity, Cooldowns>,
    names: HashMap<Entity, String>,
}

impl World {
    pub fn new() -> Self { Self { t: 0.0, next_id: 1, ..Default::default() } }

    pub fn spawn(&mut self, name: &str, pos: IVec2, team: Team, hp: i32, ammo: i32) -> Entity {
        let id = self.next_id; self.next_id += 1;
        self.poses.insert(id, Pose{pos});
        self.health.insert(id, Health{hp});
        self.team.insert(id, team);
        self.ammo.insert(id, Ammo{rounds: ammo});
        self.cds.insert(id, Cooldowns{ map: HashMap::new() });
        self.names.insert(id, name.to_string());
        id
    }

    pub fn tick(&mut self, dt: f32) {
        self.t += dt;
        for cd in self.cds.values_mut() {
            for v in cd.map.values_mut() { *v = (*v - dt).max(0.0); }
        }
    }

    // getters/setters
    pub fn pose(&self, e: Entity) -> Option<Pose> { self.poses.get(&e).copied() }
    pub fn pose_mut(&mut self, e: Entity) -> Option<&mut Pose> { self.poses.get_mut(&e) }
    pub fn health(&self, e: Entity) -> Option<Health> { self.health.get(&e).copied() }
    pub fn health_mut(&mut self, e: Entity) -> Option<&mut Health> { self.health.get_mut(&e) }
    pub fn team(&self, e: Entity) -> Option<Team> { self.team.get(&e).copied() }
    pub fn ammo_mut(&mut self, e: Entity) -> Option<&mut Ammo> { self.ammo.get_mut(&e) }
    pub fn cooldowns_mut(&mut self, e: Entity) -> Option<&mut Cooldowns> { self.cds.get_mut(&e) }
    pub fn name(&self, e: Entity) -> Option<&str> { self.names.get(&e).map(|s| s.as_str()) }

    pub fn all_of_team(&self, team_id: u8) -> Vec<Entity> {
        self.team.iter().filter_map(|(e,t)| if t.id==team_id {Some(*e)} else {None}).collect()
    }
    pub fn enemies_of(&self, team_id: u8) -> Vec<Entity> {
        self.team.iter().filter_map(|(e,t)| if t.id!=team_id {Some(*e)} else {None}).collect()
    }
    pub fn pos_of(&self, e: Entity) -> Option<IVec2> { self.poses.get(&e).map(|p| p.pos) }
    pub fn obstacle(&self, p: IVec2) -> bool { self.obstacles.contains(&(p.x, p.y)) }
}
