/// ECS Systems for Anchor and Echo Currency
/// 
/// This module contains all gameplay systems for the loom anchor mechanics:
/// 
/// - `anchor_decay_system`: Passive decay + combat stress
/// - `anchor_proximity_system`: Player proximity detection + UI prompts
/// - `anchor_interaction_system`: E key handling + inspection modal
/// - `anchor_repair_system`: Echo spending + 5s animation + stability boost
/// - `echo_pickup_system`: Grant Echoes on kill/shard pickup
/// - `echo_transaction_system`: Log all gains/spends
/// - `hud_echo_system`: Display count + transaction feedback floats

pub mod anchor_decay_system;
pub mod anchor_proximity_system;
pub mod anchor_interaction_system;
pub mod anchor_repair_system;
pub mod echo_pickup_system;
pub mod echo_transaction_system;
pub mod hud_echo_system;

pub use anchor_decay_system::{anchor_decay_system, CombatEvent, CombatEventType};
pub use anchor_proximity_system::{
    anchor_proximity_system, PlayerPosition, ProximityEvent, ProximityEventType, AnchorEntity,
};
pub use anchor_interaction_system::{
    anchor_interaction_system, InputState, InteractionEvent, InteractionEventType,
    AnchorInspectionData,
};
pub use anchor_repair_system::{
    anchor_repair_system, RepairRequest, RepairEvent, RepairResult,
};
pub use echo_pickup_system::{
    echo_pickup_system, CombatRewardEvent, PickupEvent, EnemyType, PickupType,
};
pub use echo_transaction_system::{
    echo_transaction_system, TransactionStats, TransactionFeedbackEvent,
};
pub use hud_echo_system::{
    hud_echo_system, EchoHudState, FeedbackFloat,
};
