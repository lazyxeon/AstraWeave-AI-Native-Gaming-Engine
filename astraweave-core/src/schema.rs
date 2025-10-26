// Default implementations for test scaffolding
impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            hp: 100,
            pos: IVec2 { x: 0, y: 0 },
            stance: "stand".to_string(),
            orders: vec![],
        }
    }
}

impl Default for CompanionState {
    fn default() -> Self {
        CompanionState {
            ammo: 10,
            cooldowns: BTreeMap::new(),
            morale: 1.0,
            pos: IVec2 { x: 0, y: 0 },
        }
    }
}

impl Default for EnemyState {
    fn default() -> Self {
        EnemyState {
            id: 0,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            cover: "none".to_string(),
            last_seen: 0.0,
        }
    }
}

impl Default for Poi {
    fn default() -> Self {
        Poi {
            k: "poi".to_string(),
            pos: IVec2 { x: 0, y: 0 },
        }
    }
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        WorldSnapshot {
            t: 0.0,
            player: PlayerState::default(),
            me: CompanionState::default(),
            enemies: vec![],
            pois: vec![],
            obstacles: vec![],
            objective: None,
        }
    }
}
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Entity = u32;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub t: f32,
    pub player: PlayerState,
    pub me: CompanionState,
    pub enemies: Vec<EnemyState>,
    pub pois: Vec<Poi>,
    pub obstacles: Vec<IVec2>,
    pub objective: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub hp: i32,
    pub pos: IVec2,
    pub stance: String,
    pub orders: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanionState {
    pub ammo: i32,
    pub cooldowns: BTreeMap<String, f32>,
    pub morale: f32,
    pub pos: IVec2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyState {
    pub id: Entity,
    pub pos: IVec2,
    pub hp: i32,
    pub cover: String,
    pub last_seen: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Poi {
    pub k: String,
    pub pos: IVec2,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlanIntent {
    pub plan_id: String,
    pub steps: Vec<ActionStep>,
}

// ============================================================================
// MOVEMENT SPEED & DIRECTION ENUMS
// ============================================================================

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MovementSpeed {
    Walk,
    Run,
    Sprint,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StrafeDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AttackType {
    Light,
    Heavy,
}

// ============================================================================
// ACTION STEP ENUM - 37 Tools Across 6 Categories
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "act")]
pub enum ActionStep {
    // ═══════════════════════════════════════
    // MOVEMENT (6 tools)
    // ═══════════════════════════════════════
    
    /// Move to a specific position
    MoveTo { 
        x: i32, 
        y: i32,
        #[serde(default)]
        speed: Option<MovementSpeed>,
    },
    
    /// Move toward target entity while maintaining distance
    Approach { 
        target_id: Entity,
        /// Desired distance (e.g., melee=2, ranged=15)
        distance: f32,
    },
    
    /// Move away from target entity
    Retreat { 
        target_id: Entity,
        /// Safe distance to reach
        distance: f32,
    },
    
    /// Take cover behind nearest obstacle
    TakeCover {
        /// Optional: specific cover position
        position: Option<IVec2>,
    },
    
    /// Strafe around target (circle)
    Strafe {
        target_id: Entity,
        direction: StrafeDirection,
    },
    
    /// Patrol between waypoints
    Patrol {
        waypoints: Vec<IVec2>,
    },
    
    // ═══════════════════════════════════════
    // OFFENSIVE (8 tools)
    // ═══════════════════════════════════════
    
    /// Basic attack targeting entity
    Attack { 
        target_id: Entity,
    },
    
    /// Aimed shot with higher accuracy
    AimedShot {
        target_id: Entity,
    },
    
    /// Quick attack with lower damage
    QuickAttack {
        target_id: Entity,
    },
    
    /// Heavy attack with higher damage
    HeavyAttack {
        target_id: Entity,
    },
    
    /// Area-of-effect attack
    AoEAttack {
        x: i32,
        y: i32,
        radius: f32,
    },
    
    /// Throw explosive (grenade, etc.)
    ThrowExplosive {
        x: i32,
        y: i32,
    },
    
    /// Suppressive covering fire
    CoverFire { 
        target_id: Entity, 
        duration: f32,
    },
    
    /// Charge at target
    Charge {
        target_id: Entity,
    },
    
    // ═══════════════════════════════════════
    // DEFENSIVE (6 tools)
    // ═══════════════════════════════════════
    
    /// Block incoming attack
    Block,
    
    /// Dodge attack
    Dodge {
        direction: Option<StrafeDirection>,
    },
    
    /// Parry incoming attack
    Parry,
    
    /// Throw smoke grenade
    ThrowSmoke {
        x: i32,
        y: i32,
    },
    
    /// Heal self or ally
    Heal {
        target_id: Option<Entity>,
    },
    
    /// Use defensive ability
    UseDefensiveAbility {
        ability_name: String,
    },
    
    // ═══════════════════════════════════════
    // EQUIPMENT (5 tools)
    // ═══════════════════════════════════════
    
    /// Equip weapon
    EquipWeapon {
        weapon_name: String,
    },
    
    /// Switch to different weapon
    SwitchWeapon {
        slot: u32,
    },
    
    /// Reload current weapon
    Reload,
    
    /// Use item from inventory
    UseItem {
        item_name: String,
    },
    
    /// Drop item
    DropItem {
        item_name: String,
    },
    
    // ═══════════════════════════════════════
    // TACTICAL (7 tools)
    // ═══════════════════════════════════════
    
    /// Call for reinforcements
    CallReinforcements {
        count: u32,
    },
    
    /// Mark target for allies
    MarkTarget {
        target_id: Entity,
    },
    
    /// Request covering fire
    RequestCover {
        duration: f32,
    },
    
    /// Coordinate attack with allies
    CoordinateAttack {
        target_id: Entity,
    },
    
    /// Set up ambush
    SetAmbush {
        position: IVec2,
    },
    
    /// Distract enemy
    Distract {
        target_id: Entity,
    },
    
    /// Regroup with allies
    Regroup {
        rally_point: IVec2,
    },
    
    // ═══════════════════════════════════════
    // UTILITY (5 tools)
    // ═══════════════════════════════════════
    
    /// Scan area for threats
    Scan {
        radius: f32,
    },
    
    /// Wait for duration
    Wait {
        duration: f32,
    },
    
    /// Interact with object
    Interact {
        target_id: Entity,
    },
    
    /// Use special ability
    UseAbility {
        ability_name: String,
    },
    
    /// Taunt enemy
    Taunt {
        target_id: Entity,
    },
    
    // ═══════════════════════════════════════
    // LEGACY (kept for backward compatibility)
    // ═══════════════════════════════════════
    
    /// Generic throw (now use ThrowSmoke or ThrowExplosive)
    Throw { 
        item: String, 
        x: i32, 
        y: i32,
    },
    
    /// Revive ally
    Revive { 
        ally_id: Entity,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub args: BTreeMap<String, String>, // k: name, v: type ("i32","f32","enum[...]")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolRegistry {
    pub tools: Vec<ToolSpec>,
    pub constraints: Constraints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraints {
    pub enforce_cooldowns: bool,
    pub enforce_los: bool,
    pub enforce_stamina: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("invalid action: {0}")]
    InvalidAction(String),
    #[error("cooldown blocked: {0}")]
    Cooldown(String),
    #[error("line of sight blocked")]
    LosBlocked,
    #[error("path not found")]
    NoPath,
    #[error("resource missing: {0}")]
    Resource(String),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum DirectorOp {
    Fortify {
        rect: Rect,
    }, // add obstacles
    SpawnWave {
        archetype: String,
        count: u32,
        origin: IVec2,
    },
    Collapse {
        a: IVec2,
        b: IVec2,
    }, // line of obstacles ("bridge down")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectorBudget {
    pub traps: i32,
    pub terrain_edits: i32,
    pub spawns: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectorPlan {
    pub ops: Vec<DirectorOp>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivec2_default() {
        let v = IVec2::default();
        assert_eq!(v.x, 0);
        assert_eq!(v.y, 0);
    }

    #[test]
    fn test_ivec2_equality() {
        let v1 = IVec2 { x: 5, y: 10 };
        let v2 = IVec2 { x: 5, y: 10 };
        let v3 = IVec2 { x: 3, y: 10 };
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_player_state_default() {
        let player = PlayerState::default();
        assert_eq!(player.hp, 100);
        assert_eq!(player.pos, IVec2 { x: 0, y: 0 });
        assert_eq!(player.stance, "stand");
        assert!(player.orders.is_empty());
    }

    #[test]
    fn test_companion_state_default() {
        let companion = CompanionState::default();
        assert_eq!(companion.ammo, 10);
        assert_eq!(companion.morale, 1.0);
        assert_eq!(companion.pos, IVec2 { x: 0, y: 0 });
        assert!(companion.cooldowns.is_empty());
    }

    #[test]
    fn test_enemy_state_default() {
        let enemy = EnemyState::default();
        assert_eq!(enemy.id, 0);
        assert_eq!(enemy.hp, 100);
        assert_eq!(enemy.cover, "none");
        assert_eq!(enemy.last_seen, 0.0);
    }

    #[test]
    fn test_poi_default() {
        let poi = Poi::default();
        assert_eq!(poi.k, "poi");
        assert_eq!(poi.pos, IVec2 { x: 0, y: 0 });
    }

    #[test]
    fn test_world_snapshot_default() {
        let snapshot = WorldSnapshot::default();
        assert_eq!(snapshot.t, 0.0);
        assert!(snapshot.enemies.is_empty());
        assert!(snapshot.pois.is_empty());
        assert!(snapshot.obstacles.is_empty());
        assert!(snapshot.objective.is_none());
    }

    #[test]
    fn test_plan_intent_default() {
        let intent = PlanIntent::default();
        assert!(intent.plan_id.is_empty());
        assert!(intent.steps.is_empty());
    }

    #[test]
    fn test_movement_speed_serde() {
        let walk = MovementSpeed::Walk;
        let json = serde_json::to_string(&walk).unwrap();
        assert_eq!(json, "\"walk\"");
        
        let deserialized: MovementSpeed = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MovementSpeed::Walk);
    }

    #[test]
    fn test_strafe_direction_serde() {
        let left = StrafeDirection::Left;
        let json = serde_json::to_string(&left).unwrap();
        assert_eq!(json, "\"left\"");
        
        let deserialized: StrafeDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, StrafeDirection::Left);
    }

    #[test]
    fn test_attack_type_serde() {
        let heavy = AttackType::Heavy;
        let json = serde_json::to_string(&heavy).unwrap();
        assert_eq!(json, "\"heavy\"");
    }

    #[test]
    fn test_action_step_move_to() {
        let action = ActionStep::MoveTo { 
            x: 10, 
            y: 20, 
            speed: Some(MovementSpeed::Run) 
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"act\":\"MoveTo\""));
        assert!(json.contains("\"x\":10"));
        assert!(json.contains("\"y\":20"));
    }

    #[test]
    fn test_action_step_attack() {
        let action = ActionStep::Attack { target_id: 42 };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"act\":\"Attack\""));
        assert!(json.contains("\"target_id\":42"));
    }

    #[test]
    fn test_action_step_take_cover() {
        let action = ActionStep::TakeCover { position: None };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"act\":\"TakeCover\""));
    }

    #[test]
    fn test_action_step_reload() {
        let action = ActionStep::Reload;
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"act\":\"Reload\""));
    }

    #[test]
    fn test_rect_structure() {
        let rect = Rect { x0: 0, y0: 0, x1: 10, y1: 10 };
        assert_eq!(rect.x0, 0);
        assert_eq!(rect.x1, 10);
    }

    #[test]
    fn test_director_op_spawn_wave() {
        let op = DirectorOp::SpawnWave {
            archetype: "zombie".to_string(),
            count: 5,
            origin: IVec2 { x: 100, y: 200 },
        };
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("\"op\":\"SpawnWave\""));
        assert!(json.contains("\"count\":5"));
    }

    #[test]
    fn test_director_budget() {
        let budget = DirectorBudget {
            traps: 3,
            terrain_edits: 5,
            spawns: 10,
        };
        assert_eq!(budget.traps, 3);
        assert_eq!(budget.spawns, 10);
    }

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry {
            tools: vec![],
            constraints: Constraints {
                enforce_cooldowns: true,
                enforce_los: false,
                enforce_stamina: true,
            },
        };
        assert!(registry.constraints.enforce_cooldowns);
        assert!(!registry.constraints.enforce_los);
    }

    #[test]
    fn test_engine_error_display() {
        let err = EngineError::InvalidAction("test".to_string());
        assert_eq!(err.to_string(), "invalid action: test");
        
        let err2 = EngineError::LosBlocked;
        assert_eq!(err2.to_string(), "line of sight blocked");
    }

    #[test]
    fn test_world_snapshot_with_data() {
        let snapshot = WorldSnapshot {
            t: 10.5,
            player: PlayerState::default(),
            me: CompanionState::default(),
            enemies: vec![EnemyState::default()],
            pois: vec![Poi::default()],
            obstacles: vec![IVec2 { x: 5, y: 5 }],
            objective: Some("Survive".to_string()),
        };
        assert_eq!(snapshot.t, 10.5);
        assert_eq!(snapshot.enemies.len(), 1);
        assert_eq!(snapshot.pois.len(), 1);
        assert_eq!(snapshot.obstacles.len(), 1);
        assert_eq!(snapshot.objective, Some("Survive".to_string()));
    }

    #[test]
    fn test_action_step_deserialization() {
        let json = r#"{"act":"MoveTo","x":5,"y":10,"speed":"run"}"#;
        let action: ActionStep = serde_json::from_str(json).unwrap();
        
        match action {
            ActionStep::MoveTo { x, y, speed } => {
                assert_eq!(x, 5);
                assert_eq!(y, 10);
                assert_eq!(speed, Some(MovementSpeed::Run));
            }
            _ => panic!("Expected MoveTo action"),
        }
    }

    #[test]
    fn test_companion_state_cooldowns() {
        let mut companion = CompanionState::default();
        companion.cooldowns.insert("attack".to_string(), 2.5);
        companion.cooldowns.insert("reload".to_string(), 1.0);
        
        assert_eq!(companion.cooldowns.get("attack"), Some(&2.5));
        assert_eq!(companion.cooldowns.len(), 2);
    }

    #[test]
    fn test_plan_intent_with_steps() {
        let intent = PlanIntent {
            plan_id: "plan_123".to_string(),
            steps: vec![
                ActionStep::MoveTo { x: 10, y: 20, speed: None },
                ActionStep::Attack { target_id: 5 },
                ActionStep::Reload,
            ],
        };
        assert_eq!(intent.plan_id, "plan_123");
        assert_eq!(intent.steps.len(), 3);
    }

    #[test]
    fn test_all_action_steps_compile() {
        // Ensure all ActionStep variants compile
        let _ = ActionStep::MoveTo { x: 0, y: 0, speed: None };
        let _ = ActionStep::Approach { target_id: 1, distance: 5.0 };
        let _ = ActionStep::Retreat { target_id: 1, distance: 10.0 };
        let _ = ActionStep::TakeCover { position: None };
        let _ = ActionStep::Strafe { target_id: 1, direction: StrafeDirection::Left };
        let _ = ActionStep::Patrol { waypoints: vec![] };
        let _ = ActionStep::Attack { target_id: 1 };
        let _ = ActionStep::AimedShot { target_id: 1 };
        let _ = ActionStep::QuickAttack { target_id: 1 };
        let _ = ActionStep::HeavyAttack { target_id: 1 };
        let _ = ActionStep::AoEAttack { x: 0, y: 0, radius: 5.0 };
        let _ = ActionStep::ThrowExplosive { x: 0, y: 0 };
        let _ = ActionStep::CoverFire { target_id: 1, duration: 3.0 };
        let _ = ActionStep::Charge { target_id: 1 };
        let _ = ActionStep::Block;
        let _ = ActionStep::Dodge { direction: None };
        let _ = ActionStep::Parry;
        let _ = ActionStep::ThrowSmoke { x: 0, y: 0 };
        let _ = ActionStep::Heal { target_id: None };
        let _ = ActionStep::UseDefensiveAbility { ability_name: "shield".to_string() };
        let _ = ActionStep::EquipWeapon { weapon_name: "sword".to_string() };
        let _ = ActionStep::SwitchWeapon { slot: 1 };
        let _ = ActionStep::Reload;
        let _ = ActionStep::UseItem { item_name: "potion".to_string() };
        let _ = ActionStep::DropItem { item_name: "trash".to_string() };
        let _ = ActionStep::CallReinforcements { count: 3 };
        let _ = ActionStep::MarkTarget { target_id: 1 };
        let _ = ActionStep::RequestCover { duration: 5.0 };
        let _ = ActionStep::CoordinateAttack { target_id: 1 };
        let _ = ActionStep::SetAmbush { position: IVec2 { x: 0, y: 0 } };
        let _ = ActionStep::Distract { target_id: 1 };
        let _ = ActionStep::Regroup { rally_point: IVec2 { x: 0, y: 0 } };
        let _ = ActionStep::Scan { radius: 10.0 };
        let _ = ActionStep::Wait { duration: 2.0 };
        let _ = ActionStep::Interact { target_id: 1 };
        let _ = ActionStep::UseAbility { ability_name: "fireball".to_string() };
        let _ = ActionStep::Taunt { target_id: 1 };
        let _ = ActionStep::Throw { item: "grenade".to_string(), x: 0, y: 0 };
        let _ = ActionStep::Revive { ally_id: 1 };
    }
}
