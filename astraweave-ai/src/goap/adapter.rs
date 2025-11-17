// Enhanced WorldSnapshot to GOAP WorldState adapter
// Phase 2: Engine Integration

use astraweave_core::WorldSnapshot;
use super::{WorldState, StateValue, OrderedFloat};

/// Enhanced adapter with richer state extraction
pub struct SnapshotAdapter;

impl SnapshotAdapter {
    /// Convert WorldSnapshot to comprehensive GOAP WorldState
    pub fn to_world_state(snap: &WorldSnapshot) -> WorldState {
        let mut state = WorldState::new();

        // === Player State ===
        state.set("player_hp", StateValue::Int(snap.player.hp));
        state.set("player_x", StateValue::Int(snap.player.pos.x));
        state.set("player_y", StateValue::Int(snap.player.pos.y));
        state.set("player_stance", StateValue::String(snap.player.stance.clone()));

        // Player health categories
        let player_critical = snap.player.hp < 30;
        let player_wounded = snap.player.hp < 60;
        state.set("player_critical", StateValue::Bool(player_critical));
        state.set("player_wounded", StateValue::Bool(player_wounded));

        // === Companion State ===
        state.set("my_ammo", StateValue::Int(snap.me.ammo));
        state.set("my_x", StateValue::Int(snap.me.pos.x));
        state.set("my_y", StateValue::Int(snap.me.pos.y));
        state.set("my_morale", StateValue::Float(OrderedFloat(snap.me.morale)));

        // Ammo state flags
        state.set("has_ammo", StateValue::Bool(snap.me.ammo > 0));
        state.set("ammo_low", StateValue::Bool(snap.me.ammo < 10));
        state.set("ammo_critical", StateValue::Bool(snap.me.ammo < 5));

        // Morale flags
        state.set("morale_high", StateValue::Bool(snap.me.morale > 0.7));
        state.set("morale_low", StateValue::Bool(snap.me.morale < 0.4));

        // === Cooldowns ===
        for (name, value) in &snap.me.cooldowns {
            let cd_key = format!("cd_{}", name);
            state.set(&cd_key, StateValue::Float(OrderedFloat(*value)));
            
            // Add boolean flag for active cooldowns
            let active_key = format!("{}_on_cooldown", name);
            state.set(&active_key, StateValue::Bool(*value > 0.0));
        }

        // Smoke grenade availability
        let smoke_cd = snap.me.cooldowns.get("throw:smoke").copied().unwrap_or(0.0);
        state.set("smoke_cooldown", StateValue::Float(OrderedFloat(smoke_cd)));
        state.set("smoke_available", StateValue::Bool(smoke_cd <= 0.0));

        // === Enemy State (Aggregate) ===
        let enemy_count = snap.enemies.len() as i32;
        state.set("enemy_count", StateValue::Int(enemy_count));
        state.set("enemy_present", StateValue::Bool(!snap.enemies.is_empty()));

        if let Some(first_enemy) = snap.enemies.first() {
            state.set("enemy_hp", StateValue::Int(first_enemy.hp));
            state.set("enemy_x", StateValue::Int(first_enemy.pos.x));
            state.set("enemy_y", StateValue::Int(first_enemy.pos.y));
            state.set("enemy_cover", StateValue::String(first_enemy.cover.clone()));

            // Calculate distance to closest enemy
            let dist_x = (snap.me.pos.x - first_enemy.pos.x).abs();
            let dist_y = (snap.me.pos.y - first_enemy.pos.y).abs();
            let manhattan_distance = dist_x + dist_y;
            
            state.set("enemy_distance", StateValue::Int(manhattan_distance));
            
            // Range flags
            state.set("in_range", StateValue::Bool(manhattan_distance <= 8));
            state.set("in_melee_range", StateValue::Bool(manhattan_distance <= 2));
            state.set("enemy_far", StateValue::Bool(manhattan_distance > 10));
            state.set("enemy_close", StateValue::Bool(manhattan_distance < 5));

            // Threat assessment
            let enemy_dangerous = first_enemy.hp > 30 && manhattan_distance < 8;
            state.set("enemy_dangerous", StateValue::Bool(enemy_dangerous));

            // Enemy health categories
            state.set("enemy_wounded", StateValue::Bool(first_enemy.hp < 50));
            state.set("enemy_critical", StateValue::Bool(first_enemy.hp < 20));

        } else {
            state.set("enemy_distance", StateValue::Int(999));
            state.set("in_range", StateValue::Bool(false));
            state.set("enemy_far", StateValue::Bool(true));
            state.set("enemy_close", StateValue::Bool(false));
            state.set("enemy_dangerous", StateValue::Bool(false));
        }

        // === Combat State ===
        state.set("in_combat", StateValue::Bool(!snap.enemies.is_empty()));
        
        // Safe state assessment
        let low_health = snap.me.ammo < 10 || snap.player.hp < 40;
        state.set("low_health", StateValue::Bool(low_health));
        
        let safe_state = snap.enemies.is_empty() || 
                         (snap.me.ammo > 10 && snap.player.hp > 60);
        state.set("safe", StateValue::Bool(safe_state));

        // === Tactical Flags ===
        
        // Should retreat?
        let should_retreat = (snap.player.hp < 30 && !snap.enemies.is_empty()) ||
                            (snap.me.ammo < 5 && !snap.enemies.is_empty());
        state.set("should_retreat", StateValue::Bool(should_retreat));

        // Should heal?
        let should_heal = snap.player.hp < 50;
        state.set("should_heal", StateValue::Bool(should_heal));

        // Should reload?
        let should_reload = snap.me.ammo < 10 && snap.me.ammo > 0;
        state.set("should_reload", StateValue::Bool(should_reload));

        // === Cover State ===
        // Note: Real implementation would check actual cover positions
        state.set("in_cover", StateValue::Bool(false)); // Placeholder
        state.set("cover_available", StateValue::Bool(true)); // Placeholder

        // === Resource Flags ===
        state.set("has_medkit", StateValue::Bool(true)); // Placeholder - would check inventory

        // === Positional State ===
        let distance_to_player = (snap.me.pos.x - snap.player.pos.x).abs() + 
                                 (snap.me.pos.y - snap.player.pos.y).abs();
        state.set("distance_to_player", StateValue::Int(distance_to_player));
        state.set("near_player", StateValue::Bool(distance_to_player < 5));
        state.set("far_from_player", StateValue::Bool(distance_to_player > 10));

        // === Objective State ===
        if let Some(ref objective) = snap.objective {
            state.set("has_objective", StateValue::Bool(true));
            state.set("objective", StateValue::String(objective.clone()));
        } else {
            state.set("has_objective", StateValue::Bool(false));
        }

        // === POIs (Points of Interest) ===
        state.set("poi_count", StateValue::Int(snap.pois.len() as i32));
        state.set("has_pois", StateValue::Bool(!snap.pois.is_empty()));

        state
    }

    /// Extract key tactical situation for logging
    pub fn tactical_summary(snap: &WorldSnapshot) -> String {
        let enemy_count = snap.enemies.len();
        let ammo = snap.me.ammo;
        let hp = snap.player.hp;
        
        let enemy_dist = snap.enemies.first()
            .map(|e| (snap.me.pos.x - e.pos.x).abs() + (snap.me.pos.y - e.pos.y).abs())
            .unwrap_or(999);

        format!(
            "HP:{} Ammo:{} Enemies:{} Dist:{} Morale:{:.1}",
            hp, ammo, enemy_count, enemy_dist, snap.me.morale
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use astraweave_core::{PlayerState, CompanionState, EnemyState, IVec2};

    fn make_test_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            t: 1.0,
            player: PlayerState {
                hp: 100,
                pos: IVec2 { x: 0, y: 0 },
                stance: "stand".to_string(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 20,
                cooldowns: BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 5, y: 5 },
            },
            enemies: vec![EnemyState {
                id: 1,
                pos: IVec2 { x: 10, y: 10 },
                hp: 50,
                cover: "none".to_string(),
                last_seen: 1.0,
            }],
            pois: vec![],
            obstacles: vec![],
            objective: Some("Defeat enemies".to_string()),
        }
    }

    #[test]
    fn test_enhanced_adapter_basic_state() {
        let snap = make_test_snapshot();
        let state = SnapshotAdapter::to_world_state(&snap);

        assert_eq!(state.get("player_hp"), Some(&StateValue::Int(100)));
        assert_eq!(state.get("my_ammo"), Some(&StateValue::Int(20)));
        assert_eq!(state.get("enemy_count"), Some(&StateValue::Int(1)));
    }

    #[test]
    fn test_enhanced_adapter_tactical_flags() {
        let snap = make_test_snapshot();
        let state = SnapshotAdapter::to_world_state(&snap);

        assert_eq!(state.get("enemy_present"), Some(&StateValue::Bool(true)));
        assert_eq!(state.get("has_ammo"), Some(&StateValue::Bool(true)));
        assert_eq!(state.get("in_combat"), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_enhanced_adapter_distance_calculation() {
        let snap = make_test_snapshot();
        let state = SnapshotAdapter::to_world_state(&snap);

        // Enemy at (10,10), companion at (5,5) -> distance = 10
        assert_eq!(state.get("enemy_distance"), Some(&StateValue::Int(10)));
        assert_eq!(state.get("in_range"), Some(&StateValue::Bool(false))); // >8
    }

    #[test]
    fn test_enhanced_adapter_health_categories() {
        let mut snap = make_test_snapshot();
        snap.player.hp = 25;
        
        let state = SnapshotAdapter::to_world_state(&snap);

        assert_eq!(state.get("player_critical"), Some(&StateValue::Bool(true)));
        assert_eq!(state.get("should_retreat"), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_enhanced_adapter_ammo_flags() {
        let mut snap = make_test_snapshot();
        snap.me.ammo = 3;
        
        let state = SnapshotAdapter::to_world_state(&snap);

        assert_eq!(state.get("ammo_critical"), Some(&StateValue::Bool(true)));
        assert_eq!(state.get("has_ammo"), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_tactical_summary() {
        let snap = make_test_snapshot();
        let summary = SnapshotAdapter::tactical_summary(&snap);

        assert!(summary.contains("HP:100"));
        assert!(summary.contains("Ammo:20"));
        assert!(summary.contains("Enemies:1"));
    }
}

