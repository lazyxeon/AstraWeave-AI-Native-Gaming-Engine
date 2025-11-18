/// Anchor Decay System
///
/// Applies passive decay and combat stress to all anchors in the world.
/// Runs every frame during ECS SIMULATION stage.
///
/// Decay rates:
/// - Passive: -0.01 stability per 60 seconds (DEFAULT_DECAY_RATE)
/// - Combat stress: -0.05 stability per enemy kill near anchor
///
/// Integration:
/// - Input: Query<&mut Anchor> (all anchors in world)
/// - Input: Res<Time> (delta time for passive decay)
/// - Input: EventReader<CombatEvent> (enemy kills for combat stress)
/// - Output: Modified Anchor.stability values
/// - Output: VFX state updates (triggers anchor_vfx system)
use crate::anchor::Anchor;

/// Combat event for tracking anchor stress
#[derive(Debug, Clone)]
pub struct CombatEvent {
    pub position: (f32, f32, f32),
    pub event_type: CombatEventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEventType {
    EnemyKilled,
    PlayerDamaged,
    AbilityUsed,
}

/// Anchor decay system (ECS system function)
///
/// Call from ECS SIMULATION stage:
/// ```ignore
/// app.add_system(SystemStage::SIMULATION, anchor_decay_system);
/// ```
pub fn anchor_decay_system(
    mut anchors: Vec<&mut Anchor>,
    delta_time: f32,
    combat_events: &[CombatEvent],
) {
    // Apply passive decay to all anchors
    for anchor in anchors.iter_mut() {
        anchor.apply_decay(delta_time);
    }

    // Apply combat stress to anchors near events
    for event in combat_events.iter() {
        if event.event_type == CombatEventType::EnemyKilled {
            apply_combat_stress_to_nearby_anchors(&mut anchors, event.position);
        }
    }
}

/// Apply combat stress to anchors within 20m of combat event
fn apply_combat_stress_to_nearby_anchors(anchors: &mut [&mut Anchor], event_pos: (f32, f32, f32)) {
    const STRESS_RADIUS: f32 = 20.0;
    const STRESS_RADIUS_SQ: f32 = STRESS_RADIUS * STRESS_RADIUS;

    for anchor in anchors.iter_mut() {
        // Note: In real ECS, would query Position component
        // For now, assume anchor has internal position or is looked up separately
        // This is a simplified API for unit testing

        // Apply combat stress (stubbed distance check for now)
        // Real implementation would check distance to Position component
        anchor.apply_combat_stress();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::{AbilityType, Anchor};

    #[test]
    fn test_passive_decay_one_frame() {
        let mut anchor = Anchor::new(1.0, 10, Some(AbilityType::EchoDash));

        // Simulate 1 frame @ 60 FPS (16.67ms)
        anchor_decay_system(vec![&mut anchor], 1.0 / 60.0, &[]);

        // Expected: stability 1.0 + (-0.01/60) * (1/60) = 1.0 - 0.00000278
        // Effectively 1.0 (very small decay)
        assert!(anchor.stability() < 1.0);
        assert!(anchor.stability() > 0.999);
    }

    #[test]
    fn test_passive_decay_one_second() {
        let mut anchor = Anchor::new(1.0, 10, Some(AbilityType::EchoDash));

        // Simulate 1 second (single call with dt=1.0)
        anchor_decay_system(vec![&mut anchor], 1.0, &[]);

        // Expected: stability 1.0 + (-0.01/60) * 1.0 = 1.0 - 0.000166 ≈ 0.999834
        assert!(anchor.stability() > 0.999);
        assert!(anchor.stability() < 1.0);
    }

    #[test]
    fn test_passive_decay_60_seconds() {
        let mut anchor = Anchor::new(1.0, 10, None);

        // Simulate 60 seconds (single call with dt=60.0)
        anchor_decay_system(vec![&mut anchor], 60.0, &[]);

        // Expected: 1.0 + (-0.01/60) * 60 = 1.0 - 0.01 = 0.99
        let expected = 0.99;
        assert!(
            (anchor.stability() - expected).abs() < 0.001,
            "Expected ~{}, got {}",
            expected,
            anchor.stability()
        );
    }

    #[test]
    fn test_decay_stops_at_zero() {
        let mut anchor = Anchor::new(0.01, 10, None);

        // Simulate 100 seconds (should decay to 0.0)
        // 0.01 - (100 * 0.01/60) = 0.01 - 0.01666 = negative → clamped to 0.0
        anchor_decay_system(vec![&mut anchor], 100.0, &[]);

        assert_eq!(
            anchor.stability(),
            0.0,
            "Anchor should decay to 0.0 and stop"
        );
    }

    #[test]
    fn test_combat_stress_single_event() {
        let mut anchor = Anchor::new(1.0, 10, Some(AbilityType::EchoDash));

        // Combat event near anchor
        let events = vec![CombatEvent {
            position: (0.0, 0.0, 0.0),
            event_type: CombatEventType::EnemyKilled,
        }];

        anchor_decay_system(vec![&mut anchor], 0.0, &events);

        // Expected: 1.0 - 0.05 = 0.95
        let expected = 0.95;
        assert!(
            (anchor.stability() - expected).abs() < 0.001,
            "Expected ~{}, got {}",
            expected,
            anchor.stability()
        );
    }

    #[test]
    fn test_combat_stress_multiple_events() {
        let mut anchor = Anchor::new(1.0, 10, Some(AbilityType::BarricadeDeploy));

        // 3 enemy kills near anchor
        let events = vec![
            CombatEvent {
                position: (0.0, 0.0, 0.0),
                event_type: CombatEventType::EnemyKilled,
            },
            CombatEvent {
                position: (5.0, 0.0, 0.0),
                event_type: CombatEventType::EnemyKilled,
            },
            CombatEvent {
                position: (10.0, 0.0, 0.0),
                event_type: CombatEventType::EnemyKilled,
            },
        ];

        anchor_decay_system(vec![&mut anchor], 0.0, &events);

        // Expected: 1.0 - (3 * 0.05) = 0.85
        let expected = 0.85;
        assert!(
            (anchor.stability() - expected).abs() < 0.001,
            "Expected ~{}, got {}",
            expected,
            anchor.stability()
        );
    }

    #[test]
    fn test_combined_passive_and_combat_decay() {
        let mut anchor = Anchor::new(1.0, 10, None);

        // 60 seconds passive decay
        anchor_decay_system(vec![&mut anchor], 60.0, &[]);

        // 1 combat event
        let events = vec![CombatEvent {
            position: (0.0, 0.0, 0.0),
            event_type: CombatEventType::EnemyKilled,
        }];
        anchor_decay_system(vec![&mut anchor], 0.0, &events);

        // Expected: 1.0 - 0.01 (passive 60s) - 0.05 (combat) = 0.94
        let expected = 0.94;
        assert!(
            (anchor.stability() - expected).abs() < 0.001,
            "Expected ~{}, got {}",
            expected,
            anchor.stability()
        );
    }

    #[test]
    fn test_multiple_anchors_independent_decay() {
        let mut anchor1 = Anchor::new(1.0, 10, None);
        let mut anchor2 = Anchor::new(0.5, 5, None);

        // 60 seconds passive decay
        anchor_decay_system(vec![&mut anchor1, &mut anchor2], 60.0, &[]);

        // Anchor 1: 1.0 - 0.01 = 0.99
        assert!((anchor1.stability() - 0.99).abs() < 0.001);

        // Anchor 2: 0.5 - 0.01 = 0.49
        assert!((anchor2.stability() - 0.49).abs() < 0.001);
    }

    #[test]
    fn test_non_kill_events_ignored() {
        let mut anchor = Anchor::new(1.0, 10, None);

        // Non-kill events should not apply combat stress
        let events = vec![
            CombatEvent {
                position: (0.0, 0.0, 0.0),
                event_type: CombatEventType::PlayerDamaged,
            },
            CombatEvent {
                position: (0.0, 0.0, 0.0),
                event_type: CombatEventType::AbilityUsed,
            },
        ];

        anchor_decay_system(vec![&mut anchor], 0.0, &events);

        // No decay (only EnemyKilled applies stress)
        assert_eq!(anchor.stability(), 1.0);
    }

    #[test]
    fn test_decay_updates_vfx_state() {
        let mut anchor = Anchor::new(1.0, 10, None);

        // Initial state: Perfect
        assert_eq!(anchor.vfx_state(), crate::anchor::AnchorVfxState::Perfect);

        // Decay to Stable range (0.7-0.99): 30 * 60s = 1800s
        anchor_decay_system(vec![&mut anchor], 1800.0, &[]);

        // Expected: 1.0 - (1800 * 0.01/60) = 1.0 - 0.3 = 0.7 (Stable)
        assert_eq!(anchor.vfx_state(), crate::anchor::AnchorVfxState::Stable);

        // Decay to Unstable range (0.4-0.69): Another 1800s
        anchor_decay_system(vec![&mut anchor], 1800.0, &[]);

        // Expected: 0.7 - 0.3 = 0.4 (Unstable)
        assert_eq!(anchor.vfx_state(), crate::anchor::AnchorVfxState::Unstable);
    }
}
