use crate::schema::Poi;
use crate::{CompanionState, EnemyState, Entity, IVec2, PlayerState, World, WorldSnapshot};
use std::collections::BTreeMap;

pub struct PerceptionConfig {
    pub los_max: i32,
}

pub fn build_snapshot(
    w: &World,
    t_player: Entity,
    t_companion: Entity,
    enemies: &[Entity],
    objective: Option<String>,
    cfg: &PerceptionConfig,
) -> WorldSnapshot {
    let ppos = w.pos_of(t_player)
        .expect("Player entity should have Position component");
    let cpos = w.pos_of(t_companion)
        .expect("Companion entity should have Position component");
    let player = PlayerState {
        hp: w.health(t_player)
            .expect("Player entity should have Health component").hp,
        pos: ppos,
        stance: "crouch".into(),
        orders: vec!["hold_east".into()],
    };
    let me = CompanionState {
        ammo: w.ammo(t_companion)
            .expect("Companion entity should have Ammo component").rounds,
        cooldowns: w
            .cooldowns(t_companion)
            .expect("Companion entity should have Cooldowns component")
            .map
            .clone()
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        morale: 0.8,
        pos: cpos,
    };
    let enemies = enemies
        .iter()
        .filter_map(|&e| {
            let pos = w.pos_of(e)?;
            let hp = w.health(e)?.hp;
            // LOS consider simple radius; real LOS in validator
            let cover = if (pos.x - ppos.x).abs() + (pos.y - ppos.y).abs() > cfg.los_max {
                "unknown"
            } else {
                "low"
            };
            Some(EnemyState {
                id: e,
                pos,
                hp,
                cover: cover.into(),
                last_seen: w.t,
            })
        })
        .collect::<Vec<_>>();

    WorldSnapshot {
        t: w.t,
        player,
        me,
        enemies,
        pois: vec![Poi {
            k: "breach_door".into(),
            pos: IVec2 { x: 15, y: 8 },
        }],
        obstacles: w.obstacles.iter().map(|&(x, y)| IVec2 { x, y }).collect(),
        objective,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Team, World};

    fn iv2(x: i32, y: i32) -> IVec2 {
        IVec2 { x, y }
    }

    // ===== PerceptionConfig Tests =====
    #[test]
    fn test_perception_config_creation() {
        let cfg = PerceptionConfig { los_max: 10 };
        assert_eq!(cfg.los_max, 10);
    }

    #[test]
    fn test_perception_config_large_los() {
        let cfg = PerceptionConfig { los_max: 100 };
        assert_eq!(cfg.los_max, 100);
    }

    #[test]
    fn test_perception_config_zero_los() {
        let cfg = PerceptionConfig { los_max: 0 };
        assert_eq!(cfg.los_max, 0);
    }

    // ===== build_snapshot Tests =====
    #[test]
    fn test_build_snapshot_basic() {
        let mut w = World::new();
        w.t = 5.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        let enemy = w.spawn("enemy", iv2(5, 5), Team { id: 2 }, 50, 0);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[enemy], None, &cfg);
        
        assert_eq!(snap.t, 5.0);
        assert_eq!(snap.player.hp, 100);
        assert_eq!(snap.player.pos, iv2(0, 0));
        assert_eq!(snap.me.pos, iv2(1, 1));
        assert_eq!(snap.me.ammo, 10);
        assert_eq!(snap.enemies.len(), 1);
        assert_eq!(snap.enemies[0].hp, 50);
        assert_eq!(snap.enemies[0].pos, iv2(5, 5));
    }

    #[test]
    fn test_build_snapshot_time_tracking() {
        let mut w = World::new();
        w.t = 15.5;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 80, 5);
        
        let cfg = PerceptionConfig { los_max: 10 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert_eq!(snap.t, 15.5);
    }

    #[test]
    fn test_build_snapshot_multiple_enemies() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        let enemy1 = w.spawn("enemy1", iv2(5, 5), Team { id: 2 }, 50, 0);
        let enemy2 = w.spawn("enemy2", iv2(6, 6), Team { id: 2 }, 60, 0);
        let enemy3 = w.spawn("enemy3", iv2(7, 7), Team { id: 2 }, 70, 0);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[enemy1, enemy2, enemy3], None, &cfg);
        
        assert_eq!(snap.enemies.len(), 3);
        assert_eq!(snap.enemies[0].hp, 50);
        assert_eq!(snap.enemies[1].hp, 60);
        assert_eq!(snap.enemies[2].hp, 70);
    }

    #[test]
    fn test_build_snapshot_no_enemies() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert_eq!(snap.enemies.len(), 0);
    }

    #[test]
    fn test_build_snapshot_player_state() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(10, 20), Team { id: 1 }, 75, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert_eq!(snap.player.hp, 75);
        assert_eq!(snap.player.pos, iv2(10, 20));
        assert_eq!(snap.player.stance, "crouch");
        assert_eq!(snap.player.orders.len(), 1);
        assert_eq!(snap.player.orders[0], "hold_east");
    }

    #[test]
    fn test_build_snapshot_companion_state() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(15, 25), Team { id: 1 }, 90, 7);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert_eq!(snap.me.pos, iv2(15, 25));
        assert_eq!(snap.me.ammo, 7);
        assert_eq!(snap.me.morale, 0.8);
    }

    #[test]
    fn test_build_snapshot_cooldowns() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        
        // Get cooldowns component and modify
        if let Some(cds) = w.cooldowns_mut(companion) {
            cds.map.insert("throw".to_string(), 5.0);
            cds.map.insert("heal".to_string(), 2.5);
        }
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert_eq!(snap.me.cooldowns.len(), 2);
        assert_eq!(snap.me.cooldowns.get("throw"), Some(&5.0));
        assert_eq!(snap.me.cooldowns.get("heal"), Some(&2.5));
    }

    #[test]
    fn test_build_snapshot_with_objective() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(
            &w,
            player,
            companion,
            &[],
            Some("Secure the breach point".to_string()),
            &cfg,
        );
        
        assert!(snap.objective.is_some());
        assert_eq!(snap.objective.unwrap(), "Secure the breach point");
    }

    #[test]
    fn test_build_snapshot_no_objective() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert!(snap.objective.is_none());
    }

    #[test]
    fn test_build_snapshot_pois_generated() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert_eq!(snap.pois.len(), 1);
        assert_eq!(snap.pois[0].k, "breach_door");
        assert_eq!(snap.pois[0].pos, iv2(15, 8));
    }

    #[test]
    fn test_build_snapshot_obstacles() {
        let mut w = World::new();
        w.t = 0.0;
        w.obstacles.insert((5, 5));
        w.obstacles.insert((6, 6));
        w.obstacles.insert((7, 7));
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[], None, &cfg);
        
        assert_eq!(snap.obstacles.len(), 3);
        assert!(snap.obstacles.contains(&iv2(5, 5)));
        assert!(snap.obstacles.contains(&iv2(6, 6)));
        assert!(snap.obstacles.contains(&iv2(7, 7)));
    }

    #[test]
    fn test_build_snapshot_enemy_los_close() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        let enemy = w.spawn("enemy", iv2(2, 2), Team { id: 2 }, 50, 0);
        
        let cfg = PerceptionConfig { los_max: 10 };
        let snap = build_snapshot(&w, player, companion, &[enemy], None, &cfg);
        
        assert_eq!(snap.enemies.len(), 1);
        // Enemy within los_max should have cover "low"
        assert_eq!(snap.enemies[0].cover, "low");
    }

    #[test]
    fn test_build_snapshot_enemy_los_far() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        let enemy = w.spawn("enemy", iv2(50, 50), Team { id: 2 }, 50, 0);
        
        let cfg = PerceptionConfig { los_max: 10 };
        let snap = build_snapshot(&w, player, companion, &[enemy], None, &cfg);
        
        assert_eq!(snap.enemies.len(), 1);
        // Enemy beyond los_max should have cover "unknown"
        assert_eq!(snap.enemies[0].cover, "unknown");
    }

    #[test]
    fn test_build_snapshot_enemy_last_seen() {
        let mut w = World::new();
        w.t = 12.5;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        let enemy = w.spawn("enemy", iv2(5, 5), Team { id: 2 }, 50, 0);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[enemy], None, &cfg);
        
        assert_eq!(snap.enemies.len(), 1);
        assert_eq!(snap.enemies[0].last_seen, 12.5);
    }

    #[test]
    fn test_build_snapshot_enemy_id_tracking() {
        let mut w = World::new();
        w.t = 0.0;
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 100, 0);
        let companion = w.spawn("companion", iv2(1, 1), Team { id: 1 }, 100, 10);
        let enemy1 = w.spawn("enemy1", iv2(5, 5), Team { id: 2 }, 50, 0);
        let enemy2 = w.spawn("enemy2", iv2(6, 6), Team { id: 2 }, 60, 0);
        
        let cfg = PerceptionConfig { los_max: 20 };
        let snap = build_snapshot(&w, player, companion, &[enemy1, enemy2], None, &cfg);
        
        assert_eq!(snap.enemies.len(), 2);
        assert_eq!(snap.enemies[0].id, enemy1);
        assert_eq!(snap.enemies[1].id, enemy2);
    }

    #[test]
    fn test_build_snapshot_comprehensive() {
        let mut w = World::new();
        w.t = 10.0;
        w.obstacles.insert((3, 3));
        w.obstacles.insert((4, 4));
        
        let player = w.spawn("player", iv2(0, 0), Team { id: 1 }, 85, 0);
        let companion = w.spawn("companion", iv2(2, 2), Team { id: 1 }, 95, 8);
        let enemy1 = w.spawn("enemy1", iv2(5, 5), Team { id: 2 }, 40, 0);
        let enemy2 = w.spawn("enemy2", iv2(100, 100), Team { id: 2 }, 30, 0);
        
        if let Some(cds) = w.cooldowns_mut(companion) {
            cds.map.insert("grenade".to_string(), 3.0);
        }
        
        let cfg = PerceptionConfig { los_max: 15 };
        let snap = build_snapshot(
            &w,
            player,
            companion,
            &[enemy1, enemy2],
            Some("Defend position".to_string()),
            &cfg,
        );
        
        // Verify all components
        assert_eq!(snap.t, 10.0);
        assert_eq!(snap.player.hp, 85);
        assert_eq!(snap.me.ammo, 8);
        assert_eq!(snap.me.cooldowns.len(), 1);
        assert_eq!(snap.enemies.len(), 2);
        assert_eq!(snap.enemies[0].cover, "low"); // Within LOS
        assert_eq!(snap.enemies[1].cover, "unknown"); // Beyond LOS
        assert_eq!(snap.obstacles.len(), 2);
        assert_eq!(snap.pois.len(), 1);
        assert_eq!(snap.objective, Some("Defend position".to_string()));
    }
}
