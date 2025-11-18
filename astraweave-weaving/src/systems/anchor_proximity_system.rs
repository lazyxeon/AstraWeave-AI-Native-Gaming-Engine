/// Anchor Proximity System
///
/// Detects when player is within interaction range of anchors (default 3m).
/// Shows UI prompt "Press E to Inspect Anchor" when in range.
///
/// Integration:
/// - Input: Query<&Anchor, &Position> (all anchors with positions)
/// - Input: Res<PlayerPosition> (player's current position)
/// - Output: ProximityEvents (for UI system)
/// - Runs: ECS SIMULATION stage, every frame
use crate::anchor::Anchor;

/// Player position resource (ECS resource)
#[derive(Debug, Clone, Copy)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl PlayerPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn distance_to(&self, other: (f32, f32, f32)) -> f32 {
        let dx = self.x - other.0;
        let dy = self.y - other.1;
        let dz = self.z - other.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Proximity event for UI system
#[derive(Debug, Clone)]
pub struct ProximityEvent {
    pub anchor_id: usize,
    pub anchor_position: (f32, f32, f32),
    pub distance: f32,
    pub event_type: ProximityEventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProximityEventType {
    Entered, // Player entered interaction range
    Exited,  // Player exited interaction range
    InRange, // Player remains in range (no state change)
}

/// Entity with anchor and position (simplified ECS query)
pub struct AnchorEntity {
    pub id: usize,
    pub anchor: Anchor,
    pub position: (f32, f32, f32),
}

/// Anchor proximity system (ECS system function)
///
/// Call from ECS SIMULATION stage:
/// ```ignore
/// app.add_system(SystemStage::SIMULATION, anchor_proximity_system);
/// ```
pub fn anchor_proximity_system(
    anchors: &[AnchorEntity],
    player_pos: PlayerPosition,
    previous_in_range: &mut Option<usize>, // Track state changes
) -> Vec<ProximityEvent> {
    let mut events = Vec::new();
    let mut current_in_range: Option<usize> = None;

    // Find closest anchor in range
    let mut closest_distance = f32::MAX;
    let mut closest_id: Option<usize> = None;

    for entity in anchors.iter() {
        let distance = player_pos.distance_to(entity.position);

        if entity
            .anchor
            .is_in_proximity(entity.position, (player_pos.x, player_pos.y, player_pos.z))
        {
            if distance < closest_distance {
                closest_distance = distance;
                closest_id = Some(entity.id);
            }
        }
    }

    // Generate events based on state changes
    match (*previous_in_range, closest_id) {
        (Some(prev_id), Some(curr_id)) if prev_id == curr_id => {
            // Same anchor, still in range
            events.push(ProximityEvent {
                anchor_id: curr_id,
                anchor_position: anchors
                    .iter()
                    .find(|e| e.id == curr_id)
                    .map(|e| e.position)
                    .unwrap_or((0.0, 0.0, 0.0)),
                distance: closest_distance,
                event_type: ProximityEventType::InRange,
            });
            current_in_range = Some(curr_id);
        }
        (Some(prev_id), Some(curr_id)) if prev_id != curr_id => {
            // Switched anchors
            events.push(ProximityEvent {
                anchor_id: prev_id,
                anchor_position: (0.0, 0.0, 0.0), // Would look up from ECS
                distance: 0.0,
                event_type: ProximityEventType::Exited,
            });
            events.push(ProximityEvent {
                anchor_id: curr_id,
                anchor_position: anchors
                    .iter()
                    .find(|e| e.id == curr_id)
                    .map(|e| e.position)
                    .unwrap_or((0.0, 0.0, 0.0)),
                distance: closest_distance,
                event_type: ProximityEventType::Entered,
            });
            current_in_range = Some(curr_id);
        }
        (Some(prev_id), None) => {
            // Exited range
            events.push(ProximityEvent {
                anchor_id: prev_id,
                anchor_position: (0.0, 0.0, 0.0),
                distance: 0.0,
                event_type: ProximityEventType::Exited,
            });
            current_in_range = None;
        }
        (None, Some(curr_id)) => {
            // Entered range
            events.push(ProximityEvent {
                anchor_id: curr_id,
                anchor_position: anchors
                    .iter()
                    .find(|e| e.id == curr_id)
                    .map(|e| e.position)
                    .unwrap_or((0.0, 0.0, 0.0)),
                distance: closest_distance,
                event_type: ProximityEventType::Entered,
            });
            current_in_range = Some(curr_id);
        }
        (None, None) => {
            // No anchors in range
        }
        _ => {}
    }

    *previous_in_range = current_in_range;
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::{AbilityType, Anchor};

    #[test]
    fn test_enter_proximity() {
        let anchors = vec![AnchorEntity {
            id: 1,
            anchor: Anchor::new(1.0, 10, Some(AbilityType::EchoDash)),
            position: (0.0, 0.0, 0.0),
        }];

        let player_pos = PlayerPosition::new(2.0, 0.0, 0.0); // 2m away
        let mut previous = None;

        let events = anchor_proximity_system(&anchors, player_pos, &mut previous);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].anchor_id, 1);
        assert_eq!(events[0].event_type, ProximityEventType::Entered);
        assert_eq!(previous, Some(1));
    }

    #[test]
    fn test_exit_proximity() {
        let anchors = vec![AnchorEntity {
            id: 1,
            anchor: Anchor::new(1.0, 10, None),
            position: (0.0, 0.0, 0.0),
        }];

        let player_pos = PlayerPosition::new(10.0, 0.0, 0.0); // 10m away (outside range)
        let mut previous = Some(1); // Was in range

        let events = anchor_proximity_system(&anchors, player_pos, &mut previous);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].anchor_id, 1);
        assert_eq!(events[0].event_type, ProximityEventType::Exited);
        assert_eq!(previous, None);
    }

    #[test]
    fn test_stay_in_proximity() {
        let anchors = vec![AnchorEntity {
            id: 1,
            anchor: Anchor::new(1.0, 10, Some(AbilityType::BarricadeDeploy)),
            position: (0.0, 0.0, 0.0),
        }];

        let player_pos = PlayerPosition::new(2.5, 0.0, 0.0); // 2.5m away
        let mut previous = Some(1); // Already in range

        let events = anchor_proximity_system(&anchors, player_pos, &mut previous);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].anchor_id, 1);
        assert_eq!(events[0].event_type, ProximityEventType::InRange);
        assert_eq!(previous, Some(1));
    }

    #[test]
    fn test_no_anchors_in_range() {
        let anchors = vec![AnchorEntity {
            id: 1,
            anchor: Anchor::new(1.0, 10, None),
            position: (0.0, 0.0, 0.0),
        }];

        let player_pos = PlayerPosition::new(100.0, 0.0, 0.0); // 100m away
        let mut previous = None;

        let events = anchor_proximity_system(&anchors, player_pos, &mut previous);

        assert_eq!(events.len(), 0);
        assert_eq!(previous, None);
    }

    #[test]
    fn test_multiple_anchors_closest_selected() {
        let anchors = vec![
            AnchorEntity {
                id: 1,
                anchor: Anchor::new(1.0, 10, Some(AbilityType::EchoDash)),
                position: (0.0, 0.0, 0.0),
            },
            AnchorEntity {
                id: 2,
                anchor: Anchor::new(0.5, 5, Some(AbilityType::BarricadeDeploy)),
                position: (1.0, 0.0, 0.0),
            },
        ];

        let player_pos = PlayerPosition::new(1.5, 0.0, 0.0); // Closer to anchor 2
        let mut previous = None;

        let events = anchor_proximity_system(&anchors, player_pos, &mut previous);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].anchor_id, 2); // Closest anchor selected
        assert_eq!(events[0].event_type, ProximityEventType::Entered);
    }

    #[test]
    fn test_switch_between_anchors() {
        let anchors = vec![
            AnchorEntity {
                id: 1,
                anchor: Anchor::new(1.0, 10, None),
                position: (0.0, 0.0, 0.0),
            },
            AnchorEntity {
                id: 2,
                anchor: Anchor::new(0.8, 8, None),
                position: (10.0, 0.0, 0.0),
            },
        ];

        let player_pos = PlayerPosition::new(9.0, 0.0, 0.0); // Near anchor 2
        let mut previous = Some(1); // Was near anchor 1

        let events = anchor_proximity_system(&anchors, player_pos, &mut previous);

        // Should generate Exited for anchor 1, Entered for anchor 2
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, ProximityEventType::Exited);
        assert_eq!(events[0].anchor_id, 1);
        assert_eq!(events[1].event_type, ProximityEventType::Entered);
        assert_eq!(events[1].anchor_id, 2);
        assert_eq!(previous, Some(2));
    }

    #[test]
    fn test_distance_calculation() {
        let player_pos = PlayerPosition::new(3.0, 4.0, 0.0);
        let anchor_pos = (0.0, 0.0, 0.0);

        let distance = player_pos.distance_to(anchor_pos);

        // 3-4-5 triangle: distance = 5.0
        assert!((distance - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_proximity_radius_exact_boundary() {
        let anchors = vec![AnchorEntity {
            id: 1,
            anchor: Anchor::new(1.0, 10, Some(AbilityType::EchoDash)),
            position: (0.0, 0.0, 0.0),
        }];

        // Exactly 3.0m away (default proximity radius)
        let player_pos = PlayerPosition::new(3.0, 0.0, 0.0);
        let mut previous = None;

        let events = anchor_proximity_system(&anchors, player_pos, &mut previous);

        // Should be in range (inclusive)
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, ProximityEventType::Entered);
    }

    #[test]
    fn test_3d_distance() {
        let player_pos = PlayerPosition::new(1.0, 1.0, 1.0);
        let anchor_pos = (0.0, 0.0, 0.0);

        let distance = player_pos.distance_to(anchor_pos);

        // sqrt(1^2 + 1^2 + 1^2) = sqrt(3) ≈ 1.732
        assert!((distance - 1.732).abs() < 0.01);
    }
}
