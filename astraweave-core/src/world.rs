use crate::{Entity, IVec2};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub struct Health {
    pub hp: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Team {
    pub id: u8,
} // 0: player, 1: companion, 2: enemy

#[derive(Clone, Copy, Debug)]
pub struct Ammo {
    pub rounds: i32,
}

#[derive(Clone, Debug)]
pub struct Cooldowns {
    pub map: HashMap<String, f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub pos: IVec2,
}

#[derive(Default)]
pub struct World {
    pub t: f32,
    pub next_id: Entity,
    pub obstacles: HashSet<(i32, i32)>,
    poses: HashMap<Entity, Pose>,
    health: HashMap<Entity, Health>,
    team: HashMap<Entity, Team>,
    ammo: HashMap<Entity, Ammo>,
    cds: HashMap<Entity, Cooldowns>,
    names: HashMap<Entity, String>,
}

impl World {
    pub fn new() -> Self {
        Self {
            t: 0.0,
            next_id: 1,
            ..Default::default()
        }
    }

    pub fn spawn(&mut self, name: &str, pos: IVec2, team: Team, hp: i32, ammo: i32) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        self.poses.insert(id, Pose { pos });
        self.health.insert(id, Health { hp });
        self.team.insert(id, team);
        self.ammo.insert(id, Ammo { rounds: ammo });
        self.cds.insert(
            id,
            Cooldowns {
                map: HashMap::new(),
            },
        );
        self.names.insert(id, name.to_string());
        id
    }

    pub fn tick(&mut self, dt: f32) {
        self.t += dt;
        for cd in self.cds.values_mut() {
            for v in cd.map.values_mut() {
                *v = (*v - dt).max(0.0);
            }
        }
    }

    // getters/setters
    pub fn pose(&self, e: Entity) -> Option<Pose> {
        self.poses.get(&e).copied()
    }
    pub fn pose_mut(&mut self, e: Entity) -> Option<&mut Pose> {
        self.poses.get_mut(&e)
    }
    pub fn health(&self, e: Entity) -> Option<Health> {
        self.health.get(&e).copied()
    }
    pub fn health_mut(&mut self, e: Entity) -> Option<&mut Health> {
        self.health.get_mut(&e)
    }
    pub fn team(&self, e: Entity) -> Option<Team> {
        self.team.get(&e).copied()
    }
    pub fn ammo(&self, e: Entity) -> Option<Ammo> {
        self.ammo.get(&e).copied()
    }
    pub fn ammo_mut(&mut self, e: Entity) -> Option<&mut Ammo> {
        self.ammo.get_mut(&e)
    }
    pub fn cooldowns(&self, e: Entity) -> Option<&Cooldowns> {
        self.cds.get(&e)
    }
    pub fn cooldowns_mut(&mut self, e: Entity) -> Option<&mut Cooldowns> {
        self.cds.get_mut(&e)
    }
    pub fn name(&self, e: Entity) -> Option<&str> {
        self.names.get(&e).map(|s| s.as_str())
    }

    pub fn all_of_team(&self, team_id: u8) -> Vec<Entity> {
        self.team
            .iter()
            .filter_map(|(e, t)| if t.id == team_id { Some(*e) } else { None })
            .collect()
    }
    pub fn enemies_of(&self, team_id: u8) -> Vec<Entity> {
        self.team
            .iter()
            .filter_map(|(e, t)| if t.id != team_id { Some(*e) } else { None })
            .collect()
    }
    pub fn pos_of(&self, e: Entity) -> Option<IVec2> {
        self.poses.get(&e).map(|p| p.pos)
    }
    /// Return a list of all entity ids currently present in the world.
    pub fn entities(&self) -> Vec<Entity> {
        self.poses.keys().copied().collect()
    }
    pub fn obstacle(&self, p: IVec2) -> bool {
        self.obstacles.contains(&(p.x, p.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_new() {
        let w = World::new();
        assert_eq!(w.t, 0.0);
        assert_eq!(w.next_id, 1);
        assert!(w.obstacles.is_empty());
        assert!(w.entities().is_empty());
    }

    #[test]
    fn test_world_default() {
        let w = World::default();
        assert_eq!(w.t, 0.0);
        assert_eq!(w.next_id, 0);
        assert!(w.obstacles.is_empty());
    }

    #[test]
    fn test_spawn_entity() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 5, y: 10 }, Team { id: 0 }, 100, 30);
        
        assert_eq!(e, 1);
        assert_eq!(w.next_id, 2);
        assert_eq!(w.name(e), Some("player"));
        assert_eq!(w.pose(e).unwrap().pos, IVec2 { x: 5, y: 10 });
        assert_eq!(w.health(e).unwrap().hp, 100);
        assert_eq!(w.team(e).unwrap().id, 0);
        assert_eq!(w.ammo(e).unwrap().rounds, 30);
        assert!(w.cooldowns(e).unwrap().map.is_empty());
    }

    #[test]
    fn test_spawn_multiple_entities() {
        let mut w = World::new();
        let e1 = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        let e2 = w.spawn("enemy", IVec2 { x: 10, y: 10 }, Team { id: 2 }, 50, 15);
        let e3 = w.spawn("companion", IVec2 { x: 5, y: 5 }, Team { id: 1 }, 80, 20);
        
        assert_eq!(e1, 1);
        assert_eq!(e2, 2);
        assert_eq!(e3, 3);
        assert_eq!(w.next_id, 4);
        assert_eq!(w.entities().len(), 3);
    }

    #[test]
    fn test_tick_updates_time() {
        let mut w = World::new();
        w.tick(0.1);
        assert!((w.t - 0.1).abs() < 1e-6);
        w.tick(0.2);
        assert!((w.t - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_tick_decrements_cooldowns() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        w.cooldowns_mut(e).unwrap().map.insert("attack".into(), 5.0);
        w.cooldowns_mut(e).unwrap().map.insert("heal".into(), 10.0);
        
        w.tick(2.0);
        
        let cds = w.cooldowns(e).unwrap();
        assert!((cds.map.get("attack").unwrap() - 3.0).abs() < 1e-6);
        assert!((cds.map.get("heal").unwrap() - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_tick_cooldowns_bottom_at_zero() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        w.cooldowns_mut(e).unwrap().map.insert("attack".into(), 1.0);
        w.tick(2.0);
        
        let cds = w.cooldowns(e).unwrap();
        assert_eq!(*cds.map.get("attack").unwrap(), 0.0);
    }

    #[test]
    fn test_pose_getter() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 7, y: 13 }, Team { id: 0 }, 100, 30);
        
        let pose = w.pose(e).unwrap();
        assert_eq!(pose.pos.x, 7);
        assert_eq!(pose.pos.y, 13);
    }

    #[test]
    fn test_pose_mut() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        w.pose_mut(e).unwrap().pos = IVec2 { x: 20, y: 30 };
        
        assert_eq!(w.pose(e).unwrap().pos, IVec2 { x: 20, y: 30 });
    }

    #[test]
    fn test_pose_nonexistent_entity() {
        let w = World::new();
        assert!(w.pose(999).is_none());
        assert_eq!(w.pos_of(999), None);
    }

    #[test]
    fn test_health_getter() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 75, 30);
        
        assert_eq!(w.health(e).unwrap().hp, 75);
    }

    #[test]
    fn test_health_mut() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        w.health_mut(e).unwrap().hp = 50;
        
        assert_eq!(w.health(e).unwrap().hp, 50);
    }

    #[test]
    fn test_health_nonexistent_entity() {
        let w = World::new();
        assert!(w.health(999).is_none());
    }

    #[test]
    fn test_team_getter() {
        let mut w = World::new();
        let e = w.spawn("enemy", IVec2 { x: 0, y: 0 }, Team { id: 2 }, 50, 15);
        
        assert_eq!(w.team(e).unwrap().id, 2);
    }

    #[test]
    fn test_team_nonexistent_entity() {
        let w = World::new();
        assert!(w.team(999).is_none());
    }

    #[test]
    fn test_ammo_getter() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 42);
        
        assert_eq!(w.ammo(e).unwrap().rounds, 42);
    }

    #[test]
    fn test_ammo_mut() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        w.ammo_mut(e).unwrap().rounds = 10;
        
        assert_eq!(w.ammo(e).unwrap().rounds, 10);
    }

    #[test]
    fn test_ammo_nonexistent_entity() {
        let w = World::new();
        assert!(w.ammo(999).is_none());
    }

    #[test]
    fn test_cooldowns_getter() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        let cds = w.cooldowns(e).unwrap();
        assert!(cds.map.is_empty());
    }

    #[test]
    fn test_cooldowns_mut() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        w.cooldowns_mut(e).unwrap().map.insert("attack".into(), 5.0);
        
        let cds = w.cooldowns(e).unwrap();
        assert_eq!(*cds.map.get("attack").unwrap(), 5.0);
    }

    #[test]
    fn test_cooldowns_nonexistent_entity() {
        let w = World::new();
        assert!(w.cooldowns(999).is_none());
    }

    #[test]
    fn test_name_getter() {
        let mut w = World::new();
        let e = w.spawn("hero", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        
        assert_eq!(w.name(e), Some("hero"));
    }

    #[test]
    fn test_name_nonexistent_entity() {
        let w = World::new();
        assert!(w.name(999).is_none());
    }

    #[test]
    fn test_all_of_team() {
        let mut w = World::new();
        let p1 = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        let e1 = w.spawn("enemy1", IVec2 { x: 10, y: 10 }, Team { id: 2 }, 50, 15);
        let e2 = w.spawn("enemy2", IVec2 { x: 15, y: 15 }, Team { id: 2 }, 50, 15);
        let c1 = w.spawn("companion", IVec2 { x: 5, y: 5 }, Team { id: 1 }, 80, 20);
        
        let team_0 = w.all_of_team(0);
        assert_eq!(team_0.len(), 1);
        assert!(team_0.contains(&p1));
        
        let team_1 = w.all_of_team(1);
        assert_eq!(team_1.len(), 1);
        assert!(team_1.contains(&c1));
        
        let team_2 = w.all_of_team(2);
        assert_eq!(team_2.len(), 2);
        assert!(team_2.contains(&e1));
        assert!(team_2.contains(&e2));
    }

    #[test]
    fn test_all_of_team_empty() {
        let w = World::new();
        let team_0 = w.all_of_team(0);
        assert!(team_0.is_empty());
    }

    #[test]
    fn test_enemies_of() {
        let mut w = World::new();
        let p1 = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        let e1 = w.spawn("enemy1", IVec2 { x: 10, y: 10 }, Team { id: 2 }, 50, 15);
        let e2 = w.spawn("enemy2", IVec2 { x: 15, y: 15 }, Team { id: 2 }, 50, 15);
        let c1 = w.spawn("companion", IVec2 { x: 5, y: 5 }, Team { id: 1 }, 80, 20);
        
        let enemies_of_player = w.enemies_of(0);
        assert_eq!(enemies_of_player.len(), 3);
        assert!(enemies_of_player.contains(&e1));
        assert!(enemies_of_player.contains(&e2));
        assert!(enemies_of_player.contains(&c1));
        assert!(!enemies_of_player.contains(&p1));
    }

    #[test]
    fn test_enemies_of_empty() {
        let w = World::new();
        let enemies = w.enemies_of(0);
        assert!(enemies.is_empty());
    }

    #[test]
    fn test_pos_of() {
        let mut w = World::new();
        let e = w.spawn("player", IVec2 { x: 12, y: 34 }, Team { id: 0 }, 100, 30);
        
        let pos = w.pos_of(e).unwrap();
        assert_eq!(pos.x, 12);
        assert_eq!(pos.y, 34);
    }

    #[test]
    fn test_pos_of_nonexistent() {
        let w = World::new();
        assert!(w.pos_of(999).is_none());
    }

    #[test]
    fn test_entities() {
        let mut w = World::new();
        assert!(w.entities().is_empty());
        
        let e1 = w.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        let e2 = w.spawn("enemy", IVec2 { x: 10, y: 10 }, Team { id: 2 }, 50, 15);
        
        let entities = w.entities();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
    }

    #[test]
    fn test_obstacle_present() {
        let mut w = World::new();
        w.obstacles.insert((5, 10));
        
        assert!(w.obstacle(IVec2 { x: 5, y: 10 }));
    }

    #[test]
    fn test_obstacle_absent() {
        let w = World::new();
        assert!(!w.obstacle(IVec2 { x: 5, y: 10 }));
    }

    #[test]
    fn test_obstacle_multiple() {
        let mut w = World::new();
        w.obstacles.insert((0, 0));
        w.obstacles.insert((5, 5));
        w.obstacles.insert((10, 10));
        
        assert!(w.obstacle(IVec2 { x: 0, y: 0 }));
        assert!(w.obstacle(IVec2 { x: 5, y: 5 }));
        assert!(w.obstacle(IVec2 { x: 10, y: 10 }));
        assert!(!w.obstacle(IVec2 { x: 7, y: 7 }));
    }
}
