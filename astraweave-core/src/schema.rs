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

impl IVec2 {
    /// Convenience constructor matching glam-style APIs.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsContext {
    pub blocking_objects: Vec<BlockingObject>,
    pub interactable_objects: Vec<InteractableObject>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockingObject {
    pub id: Entity,
    pub pos: IVec2,
    pub object_type: String,
    pub is_locked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractableObject {
    pub id: Entity,
    pub pos: IVec2,
    pub object_type: String,
    pub requires_item: Option<String>,
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
// TERRAIN GENERATION DSL - AI-Orchestrated Dynamic Terrain (Phase 10)
// ============================================================================

/// Terrain feature types for LLM-driven generation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum TerrainFeatureType {
    /// Underground cave system with specified depth
    Cave { depth: u32 },
    /// Impact crater with specified radius
    Crater { radius: u32 },
    /// Vertical cliff face with specified height
    Cliff { height: u32 },
    /// Body of water with specified depth
    Lake { depth: u32 },
    /// Dense vegetation area with specified density (0.0-1.0)
    Forest { density: f32 },
    /// Ancient ruins with specified complexity level (1-5)
    Ruins { complexity: u32 },
    /// Custom feature type with arbitrary parameters
    Custom {
        feature_id: String,
        params: std::collections::HashMap<String, f32>,
    },
}

impl Default for TerrainFeatureType {
    fn default() -> Self {
        TerrainFeatureType::Crater { radius: 10 }
    }
}

/// Cardinal direction for spatial references
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CardinalDirection {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl CardinalDirection {
    /// Convert to a unit vector (x, z) on the horizontal plane
    pub fn to_unit_vector(self) -> (f32, f32) {
        match self {
            CardinalDirection::North => (0.0, -1.0),
            CardinalDirection::South => (0.0, 1.0),
            CardinalDirection::East => (1.0, 0.0),
            CardinalDirection::West => (-1.0, 0.0),
            CardinalDirection::NorthEast => (0.707, -0.707),
            CardinalDirection::NorthWest => (-0.707, -0.707),
            CardinalDirection::SouthEast => (0.707, 0.707),
            CardinalDirection::SouthWest => (-0.707, 0.707),
        }
    }
}

/// Distance categories to prevent LLM spatial hallucination
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DistanceCategory {
    /// 10-50 units from reference point
    Near,
    /// 50-150 units from reference point
    Medium,
    /// 150-500 units from reference point
    Far,
}

impl DistanceCategory {
    /// Get the actual distance range (min, max) in world units
    pub fn to_range(self) -> (f32, f32) {
        match self {
            DistanceCategory::Near => (10.0, 50.0),
            DistanceCategory::Medium => (50.0, 150.0),
            DistanceCategory::Far => (150.0, 500.0),
        }
    }

    /// Get the midpoint distance for this category
    pub fn midpoint(self) -> f32 {
        let (min, max) = self.to_range();
        (min + max) / 2.0
    }
}

/// Relative location for LLM-friendly spatial references
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "method")]
pub enum RelativeLocation {
    /// Place at the point the camera/player is looking at
    LineOfSight {
        /// Maximum raycast distance
        look_distance: f32,
    },
    /// Place in a direction from the current position
    DirectionFrom {
        /// Cardinal direction
        cardinal: CardinalDirection,
        /// Distance category
        distance: DistanceCategory,
    },
    /// Explicit world coordinates (fallback)
    Coordinates { x: f32, y: f32, z: f32 },
}

impl Default for RelativeLocation {
    fn default() -> Self {
        RelativeLocation::LineOfSight {
            look_distance: 50.0,
        }
    }
}

/// Persistence mode for terrain modifications
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PersistenceMode {
    /// Terrain changes are lost when the session ends
    #[default]
    SessionOnly,
    /// Terrain changes are saved to disk and persist across sessions
    Persistent,
}

/// LLM terrain generation request (DSL for Hermes 2 Pro)
///
/// This struct represents a complete terrain modification request that can be
/// generated by an LLM and validated by the TerrainSolver before execution.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TerrainGenerationRequest {
    /// Unique identifier for this request (UUID v4)
    pub request_id: String,
    /// Type of terrain feature to generate
    pub feature_type: TerrainFeatureType,
    /// Where to place the feature relative to the player/camera
    pub relative_location: RelativeLocation,
    /// Intensity of the modification (0.0 = subtle, 1.0 = dramatic)
    pub intensity: f32,
    /// Narrative justification for the terrain change (max 100 chars)
    pub narrative_reason: String,
    /// Whether this change should persist across sessions
    pub persistence_mode: PersistenceMode,
    /// Biome constraints - if non-empty, feature can only spawn in these biomes
    pub biome_constraints: Vec<String>,
    /// Optional seed for deterministic generation and replay validation
    pub seed: Option<u64>,
}

impl Default for TerrainGenerationRequest {
    fn default() -> Self {
        TerrainGenerationRequest {
            request_id: String::new(),
            feature_type: TerrainFeatureType::default(),
            relative_location: RelativeLocation::default(),
            intensity: 0.5,
            narrative_reason: String::new(),
            persistence_mode: PersistenceMode::default(),
            biome_constraints: Vec::new(),
            seed: None,
        }
    }
}

impl TerrainGenerationRequest {
    /// Validate the request before processing
    pub fn validate(&self) -> Result<(), String> {
        // Validate intensity bounds
        if self.intensity < 0.0 || self.intensity > 1.0 {
            return Err("Intensity must be between 0.0 and 1.0".to_string());
        }

        // Validate narrative reason length
        if self.narrative_reason.len() > 100 {
            return Err("Narrative reason exceeds 100 characters".to_string());
        }

        // Validate request_id is not empty
        if self.request_id.is_empty() {
            return Err("Request ID cannot be empty".to_string());
        }

        Ok(())
    }
}

// ============================================================================
// ACTION STEP ENUM - 38 Tools Across 7 Categories (includes ModifyTerrain)
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
    Patrol { waypoints: Vec<IVec2> },

    // ═══════════════════════════════════════
    // OFFENSIVE (8 tools)
    // ═══════════════════════════════════════
    /// Basic attack targeting entity
    Attack { target_id: Entity },

    /// Aimed shot with higher accuracy
    AimedShot { target_id: Entity },

    /// Quick attack with lower damage
    QuickAttack { target_id: Entity },

    /// Heavy attack with higher damage
    HeavyAttack { target_id: Entity },

    /// Area-of-effect attack
    AoEAttack { x: i32, y: i32, radius: f32 },

    /// Throw explosive (grenade, etc.)
    ThrowExplosive { x: i32, y: i32 },

    /// Suppressive covering fire
    CoverFire { target_id: Entity, duration: f32 },

    /// Charge at target
    Charge { target_id: Entity },

    // ═══════════════════════════════════════
    // DEFENSIVE (6 tools)
    // ═══════════════════════════════════════
    /// Block incoming attack
    Block,

    /// Dodge attack
    Dodge { direction: Option<StrafeDirection> },

    /// Parry incoming attack
    Parry,

    /// Throw smoke grenade
    ThrowSmoke { x: i32, y: i32 },

    /// Heal self or ally
    Heal { target_id: Option<Entity> },

    /// Use defensive ability
    UseDefensiveAbility { ability_name: String },

    // ═══════════════════════════════════════
    // EQUIPMENT (5 tools)
    // ═══════════════════════════════════════
    /// Equip weapon
    EquipWeapon { weapon_name: String },

    /// Switch to different weapon
    SwitchWeapon { slot: u32 },

    /// Reload current weapon
    Reload,

    /// Use item from inventory
    UseItem { item_name: String },

    /// Drop item
    DropItem { item_name: String },

    // ═══════════════════════════════════════
    // TACTICAL (7 tools)
    // ═══════════════════════════════════════
    /// Call for reinforcements
    CallReinforcements { count: u32 },

    /// Mark target for allies
    MarkTarget { target_id: Entity },

    /// Request covering fire
    RequestCover { duration: f32 },

    /// Coordinate attack with allies
    CoordinateAttack { target_id: Entity },

    /// Set up ambush
    SetAmbush { position: IVec2 },

    /// Distract enemy
    Distract { target_id: Entity },

    /// Regroup with allies
    Regroup { rally_point: IVec2 },

    // ═══════════════════════════════════════
    // UTILITY (5 tools)
    // ═══════════════════════════════════════
    /// Scan area for threats
    Scan { radius: f32 },

    /// Wait for duration
    Wait { duration: f32 },

    /// Interact with object
    Interact { target_id: Entity },

    /// Use special ability
    UseAbility { ability_name: String },

    /// Taunt enemy
    Taunt { target_id: Entity },

    // ═══════════════════════════════════════
    // LEGACY (kept for backward compatibility)
    // ═══════════════════════════════════════
    /// Generic throw (now use ThrowSmoke or ThrowExplosive)
    Throw { item: String, x: i32, y: i32 },

    /// Revive ally
    Revive { ally_id: Entity },

    // ═══════════════════════════════════════
    // TERRAIN (1 tool) - AI-Orchestrated Dynamic Terrain
    // ═══════════════════════════════════════
    /// Modify terrain at a location (LLM-driven, validated by TerrainSolver)
    ModifyTerrain {
        /// Unique request identifier
        request_id: String,
        /// Full terrain generation request payload
        payload: TerrainGenerationRequest,
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
            speed: Some(MovementSpeed::Run),
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
        let rect = Rect {
            x0: 0,
            y0: 0,
            x1: 10,
            y1: 10,
        };
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
                ActionStep::MoveTo {
                    x: 10,
                    y: 20,
                    speed: None,
                },
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
        let _ = ActionStep::MoveTo {
            x: 0,
            y: 0,
            speed: None,
        };
        let _ = ActionStep::Approach {
            target_id: 1,
            distance: 5.0,
        };
        let _ = ActionStep::Retreat {
            target_id: 1,
            distance: 10.0,
        };
        let _ = ActionStep::TakeCover { position: None };
        let _ = ActionStep::Strafe {
            target_id: 1,
            direction: StrafeDirection::Left,
        };
        let _ = ActionStep::Patrol { waypoints: vec![] };
        let _ = ActionStep::Attack { target_id: 1 };
        let _ = ActionStep::AimedShot { target_id: 1 };
        let _ = ActionStep::QuickAttack { target_id: 1 };
        let _ = ActionStep::HeavyAttack { target_id: 1 };
        let _ = ActionStep::AoEAttack {
            x: 0,
            y: 0,
            radius: 5.0,
        };
        let _ = ActionStep::ThrowExplosive { x: 0, y: 0 };
        let _ = ActionStep::CoverFire {
            target_id: 1,
            duration: 3.0,
        };
        let _ = ActionStep::Charge { target_id: 1 };
        let _ = ActionStep::Block;
        let _ = ActionStep::Dodge { direction: None };
        let _ = ActionStep::Parry;
        let _ = ActionStep::ThrowSmoke { x: 0, y: 0 };
        let _ = ActionStep::Heal { target_id: None };
        let _ = ActionStep::UseDefensiveAbility {
            ability_name: "shield".to_string(),
        };
        let _ = ActionStep::EquipWeapon {
            weapon_name: "sword".to_string(),
        };
        let _ = ActionStep::SwitchWeapon { slot: 1 };
        let _ = ActionStep::Reload;
        let _ = ActionStep::UseItem {
            item_name: "potion".to_string(),
        };
        let _ = ActionStep::DropItem {
            item_name: "trash".to_string(),
        };
        let _ = ActionStep::CallReinforcements { count: 3 };
        let _ = ActionStep::MarkTarget { target_id: 1 };
        let _ = ActionStep::RequestCover { duration: 5.0 };
        let _ = ActionStep::CoordinateAttack { target_id: 1 };
        let _ = ActionStep::SetAmbush {
            position: IVec2 { x: 0, y: 0 },
        };
        let _ = ActionStep::Distract { target_id: 1 };
        let _ = ActionStep::Regroup {
            rally_point: IVec2 { x: 0, y: 0 },
        };
        let _ = ActionStep::Scan { radius: 10.0 };
        let _ = ActionStep::Wait { duration: 2.0 };
        let _ = ActionStep::Interact { target_id: 1 };
        let _ = ActionStep::UseAbility {
            ability_name: "fireball".to_string(),
        };
        let _ = ActionStep::Taunt { target_id: 1 };
        let _ = ActionStep::Throw {
            item: "grenade".to_string(),
            x: 0,
            y: 0,
        };
        let _ = ActionStep::Revive { ally_id: 1 };
        // New terrain action
        let _ = ActionStep::ModifyTerrain {
            request_id: "test-req-1".to_string(),
            payload: TerrainGenerationRequest::default(),
        };
    }

    // =========================================================================
    // TERRAIN DSL TESTS
    // =========================================================================

    #[test]
    fn test_terrain_feature_type_default() {
        let feature = TerrainFeatureType::default();
        if let TerrainFeatureType::Crater { radius } = feature {
            assert_eq!(radius, 10);
        } else {
            panic!("Default should be Crater");
        }
    }

    #[test]
    fn test_terrain_feature_type_serde() {
        let cave = TerrainFeatureType::Cave { depth: 50 };
        let json = serde_json::to_string(&cave).unwrap();
        assert!(json.contains("\"type\":\"Cave\""));
        assert!(json.contains("\"depth\":50"));

        let forest = TerrainFeatureType::Forest { density: 0.8 };
        let json = serde_json::to_string(&forest).unwrap();
        assert!(json.contains("\"type\":\"Forest\""));
    }

    #[test]
    fn test_cardinal_direction_to_unit_vector() {
        let (x, z) = CardinalDirection::North.to_unit_vector();
        assert!((x - 0.0).abs() < 0.001);
        assert!((z - (-1.0)).abs() < 0.001);

        let (x, z) = CardinalDirection::East.to_unit_vector();
        assert!((x - 1.0).abs() < 0.001);
        assert!((z - 0.0).abs() < 0.001);

        let (x, z) = CardinalDirection::NorthEast.to_unit_vector();
        assert!((x - 0.707).abs() < 0.001);
        assert!((z - (-0.707)).abs() < 0.001);
    }

    #[test]
    fn test_distance_category_to_range() {
        let (min, max) = DistanceCategory::Near.to_range();
        assert_eq!(min, 10.0);
        assert_eq!(max, 50.0);

        let (min, max) = DistanceCategory::Medium.to_range();
        assert_eq!(min, 50.0);
        assert_eq!(max, 150.0);

        let (min, max) = DistanceCategory::Far.to_range();
        assert_eq!(min, 150.0);
        assert_eq!(max, 500.0);
    }

    #[test]
    fn test_distance_category_midpoint() {
        assert_eq!(DistanceCategory::Near.midpoint(), 30.0);
        assert_eq!(DistanceCategory::Medium.midpoint(), 100.0);
        assert_eq!(DistanceCategory::Far.midpoint(), 325.0);
    }

    #[test]
    fn test_relative_location_serde() {
        let los = RelativeLocation::LineOfSight {
            look_distance: 100.0,
        };
        let json = serde_json::to_string(&los).unwrap();
        assert!(json.contains("\"method\":\"LineOfSight\""));
        assert!(json.contains("\"look_distance\":100"));

        let dir = RelativeLocation::DirectionFrom {
            cardinal: CardinalDirection::North,
            distance: DistanceCategory::Medium,
        };
        let json = serde_json::to_string(&dir).unwrap();
        assert!(json.contains("\"method\":\"DirectionFrom\""));
        assert!(json.contains("\"cardinal\":\"north\""));
        assert!(json.contains("\"distance\":\"medium\""));
    }

    #[test]
    fn test_persistence_mode_default() {
        let mode = PersistenceMode::default();
        assert_eq!(mode, PersistenceMode::SessionOnly);
    }

    #[test]
    fn test_terrain_generation_request_default() {
        let req = TerrainGenerationRequest::default();
        assert!(req.request_id.is_empty());
        assert_eq!(req.intensity, 0.5);
        assert!(req.biome_constraints.is_empty());
        assert_eq!(req.persistence_mode, PersistenceMode::SessionOnly);
    }

    #[test]
    fn test_terrain_generation_request_validate_success() {
        let req = TerrainGenerationRequest {
            request_id: "test-123".to_string(),
            feature_type: TerrainFeatureType::Crater { radius: 20 },
            relative_location: RelativeLocation::LineOfSight {
                look_distance: 50.0,
            },
            intensity: 0.7,
            narrative_reason: "Creating a crater for quest objective".to_string(),
            persistence_mode: PersistenceMode::Persistent,
            biome_constraints: vec!["grassland".to_string()],
            seed: Some(12345),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_terrain_generation_request_validate_intensity_too_high() {
        let req = TerrainGenerationRequest {
            request_id: "test-123".to_string(),
            intensity: 1.5, // Invalid
            ..TerrainGenerationRequest::default()
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Intensity"));
    }

    #[test]
    fn test_terrain_generation_request_validate_intensity_negative() {
        let req = TerrainGenerationRequest {
            request_id: "test-123".to_string(),
            intensity: -0.1, // Invalid
            ..TerrainGenerationRequest::default()
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_terrain_generation_request_validate_narrative_too_long() {
        let req = TerrainGenerationRequest {
            request_id: "test-123".to_string(),
            narrative_reason: "x".repeat(101), // Too long
            ..TerrainGenerationRequest::default()
        };
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("100 characters"));
    }

    #[test]
    fn test_terrain_generation_request_validate_empty_request_id() {
        let req = TerrainGenerationRequest::default(); // Empty request_id
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Request ID"));
    }

    #[test]
    fn test_action_step_modify_terrain_serde() {
        let action = ActionStep::ModifyTerrain {
            request_id: "terrain-001".to_string(),
            payload: TerrainGenerationRequest {
                request_id: "terrain-001".to_string(),
                feature_type: TerrainFeatureType::Cave { depth: 30 },
                relative_location: RelativeLocation::DirectionFrom {
                    cardinal: CardinalDirection::North,
                    distance: DistanceCategory::Near,
                },
                intensity: 0.6,
                narrative_reason: "Secret cave for quest".to_string(),
                persistence_mode: PersistenceMode::Persistent,
                biome_constraints: vec!["mountain".to_string(), "forest".to_string()],
                seed: Some(42),
            },
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"act\":\"ModifyTerrain\""));
        assert!(json.contains("\"request_id\":\"terrain-001\""));
        assert!(json.contains("\"type\":\"Cave\""));
        assert!(json.contains("\"depth\":30"));
    }
}
