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
