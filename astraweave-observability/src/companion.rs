use serde::{Deserialize, Serialize};

/// Telemetry event emitted whenever the companion executes a GOAP action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanionActionEvent {
    pub action_id: String,
    pub success: bool,
    pub latency_ms: f32,
}

/// Telemetry event emitted when the companion unlocks an adaptive ability during the slice.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanionAdaptiveUnlock {
    pub unlock_id: String,
}

pub fn log_companion_action(event: &CompanionActionEvent) {
    tracing::info!(
        target = "veilweaver.companion.telemetry",
        event = "CompanionAction",
        action_id = %event.action_id,
        success = event.success,
        latency_ms = event.latency_ms
    );
}

pub fn log_companion_unlock(event: &CompanionAdaptiveUnlock) {
    tracing::info!(
        target = "veilweaver.companion.telemetry",
        event = "CompanionAdaptiveUnlock",
        unlock_id = %event.unlock_id
    );
}
