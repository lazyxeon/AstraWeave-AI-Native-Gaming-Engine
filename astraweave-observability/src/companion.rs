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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_companion_action_event_creation() {
        let event = CompanionActionEvent {
            action_id: "goap_heal".to_string(),
            success: true,
            latency_ms: 15.5,
        };
        assert_eq!(event.action_id, "goap_heal");
        assert!(event.success);
        assert!((event.latency_ms - 15.5).abs() < 0.01);
    }

    #[test]
    fn test_companion_action_event_failure() {
        let event = CompanionActionEvent {
            action_id: "goap_attack".to_string(),
            success: false,
            latency_ms: 25.0,
        };
        assert!(!event.success);
        assert_eq!(event.action_id, "goap_attack");
    }

    #[test]
    fn test_companion_adaptive_unlock_creation() {
        let event = CompanionAdaptiveUnlock {
            unlock_id: "ability_fireball".to_string(),
        };
        assert_eq!(event.unlock_id, "ability_fireball");
    }

    #[test]
    fn test_companion_action_event_serialization() {
        let event = CompanionActionEvent {
            action_id: "collect_loot".to_string(),
            success: true,
            latency_ms: 5.25,
        };
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("collect_loot"));
        assert!(serialized.contains("true"));
    }

    #[test]
    fn test_companion_unlock_event_serialization() {
        let event = CompanionAdaptiveUnlock {
            unlock_id: "ultimate_strike".to_string(),
        };
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("ultimate_strike"));
    }

    #[test]
    fn test_log_companion_action() {
        // Simply verify that the function runs without panic
        let event = CompanionActionEvent {
            action_id: "test_action".to_string(),
            success: true,
            latency_ms: 10.0,
        };
        log_companion_action(&event);
    }

    #[test]
    fn test_log_companion_unlock() {
        // Simply verify that the function runs without panic
        let event = CompanionAdaptiveUnlock {
            unlock_id: "test_unlock".to_string(),
        };
        log_companion_unlock(&event);
    }

    #[test]
    fn test_companion_action_clone() {
        let event = CompanionActionEvent {
            action_id: "original".to_string(),
            success: true,
            latency_ms: 20.0,
        };
        let cloned = event.clone();
        assert_eq!(event.action_id, cloned.action_id);
        assert_eq!(event.success, cloned.success);
    }

    #[test]
    fn test_companion_unlock_clone() {
        let event = CompanionAdaptiveUnlock {
            unlock_id: "original_unlock".to_string(),
        };
        let cloned = event.clone();
        assert_eq!(event.unlock_id, cloned.unlock_id);
    }

    #[test]
    fn test_companion_action_debug_format() {
        let event = CompanionActionEvent {
            action_id: "debug_test".to_string(),
            success: false,
            latency_ms: 100.5,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("100.5"));
    }
}
