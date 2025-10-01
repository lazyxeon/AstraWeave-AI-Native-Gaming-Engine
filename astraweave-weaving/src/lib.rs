//! Weaving System - Emergent Behavior Layer
//!
//! The weaving system detects patterns in the world state and proposes
//! emergent intents that are adjudicated by budget and cooldown constraints.
//!
//! Flow: Perception (Patterns) → Reasoning (Intents) → Planning (Adjudication) → Action (Events)

pub mod adjudicator;
pub mod intents;
pub mod patterns;

pub use adjudicator::{WeaveAdjudicator, WeaveConfig};
pub use intents::{IntentProposer, WeaveIntent};
pub use patterns::{Pattern, PatternDetector, PatternStrength};

/// Component for entities that can detect patterns and propose intents
#[derive(Debug, Clone)]
pub struct CWeaveAgent {
    /// Patterns detected in the last scan
    pub patterns_detected: std::collections::BTreeMap<String, f32>,
    /// Last time patterns were scanned
    pub last_scan: f32,
    /// Scan interval (seconds)
    pub scan_interval: f32,
}

impl CWeaveAgent {
    pub fn new(scan_interval: f32) -> Self {
        Self {
            patterns_detected: std::collections::BTreeMap::new(),
            last_scan: 0.0,
            scan_interval,
        }
    }

    pub fn should_scan(&self, current_time: f32) -> bool {
        current_time - self.last_scan >= self.scan_interval
    }
}

/// Component for weave signals/triggers in the world
#[derive(Debug, Clone)]
pub struct CWeaveSignal {
    pub kind: String,
    pub strength: f32,
    pub seed: u64,
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Event emitted when a weave intent is accepted
#[derive(Debug, Clone)]
pub struct WeaveIntentEvent {
    pub intent: WeaveIntent,
}
