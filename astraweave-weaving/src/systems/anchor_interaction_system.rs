/// Anchor Interaction System
///
/// Handles player E key press when in proximity to anchors.
/// Opens inspection modal UI showing anchor info.
///
/// Integration:
/// - Input: ProximityEvents (from anchor_proximity_system)
/// - Input: Res<Input> (E key state)
/// - Output: InteractionEvents (for UI system)
use crate::anchor::Anchor;

/// Input state for interaction (simplified for testing)
#[derive(Debug, Clone)]
pub struct InputState {
    pub e_pressed: bool,
    pub e_just_pressed: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            e_pressed: false,
            e_just_pressed: false,
        }
    }

    pub fn press_e(&mut self) {
        self.e_pressed = true;
        self.e_just_pressed = true;
    }

    pub fn release_e(&mut self) {
        self.e_pressed = false;
        self.e_just_pressed = false;
    }
}

/// Interaction event for UI system
#[derive(Debug, Clone)]
pub struct InteractionEvent {
    pub anchor_id: usize,
    pub event_type: InteractionEventType,
    pub anchor_data: AnchorInspectionData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionEventType {
    Inspect, // Open inspection modal
    Close,   // Close inspection modal
}

/// Data for inspection modal UI
#[derive(Debug, Clone)]
pub struct AnchorInspectionData {
    pub stability: f32,
    pub repair_cost: u32,
    pub unlocks_ability: Option<String>,
    pub is_repaired: bool,
}

impl AnchorInspectionData {
    pub fn from_anchor(anchor: &Anchor) -> Self {
        Self {
            stability: anchor.stability(),
            repair_cost: anchor.repair_cost(),
            unlocks_ability: anchor.unlocks_ability().map(|a| format!("{:?}", a)),
            is_repaired: anchor.is_repaired(),
        }
    }
}

/// Anchor interaction system (ECS system function)
pub fn anchor_interaction_system(
    in_proximity_anchor: Option<usize>,
    anchors: &[(usize, &Anchor)],
    input: &InputState,
) -> Option<InteractionEvent> {
    // Only handle interaction if player is in proximity and pressed E
    if !input.e_just_pressed {
        return None;
    }

    let anchor_id = in_proximity_anchor?;

    // Find anchor data
    let (_, anchor) = anchors.iter().find(|(id, _)| *id == anchor_id)?;

    Some(InteractionEvent {
        anchor_id,
        event_type: InteractionEventType::Inspect,
        anchor_data: AnchorInspectionData::from_anchor(anchor),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::{AbilityType, Anchor};

    #[test]
    fn test_inspect_anchor_in_proximity() {
        let anchor = Anchor::new(0.7, 5, Some(AbilityType::EchoDash));
        let anchors = vec![(1, &anchor)];

        let mut input = InputState::new();
        input.press_e();

        let event = anchor_interaction_system(Some(1), &anchors, &input);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.anchor_id, 1);
        assert_eq!(event.event_type, InteractionEventType::Inspect);
        assert!((event.anchor_data.stability - 0.7).abs() < 0.001);
        assert_eq!(event.anchor_data.repair_cost, 5);
        assert_eq!(
            event.anchor_data.unlocks_ability,
            Some("EchoDash".to_string())
        );
    }

    #[test]
    fn test_no_interaction_when_not_in_proximity() {
        let anchor = Anchor::new(1.0, 10, None);
        let anchors = vec![(1, &anchor)];

        let mut input = InputState::new();
        input.press_e();

        let event = anchor_interaction_system(None, &anchors, &input);

        assert!(event.is_none());
    }

    #[test]
    fn test_no_interaction_without_e_press() {
        let anchor = Anchor::new(0.5, 3, Some(AbilityType::BarricadeDeploy));
        let anchors = vec![(1, &anchor)];

        let input = InputState::new(); // E not pressed

        let event = anchor_interaction_system(Some(1), &anchors, &input);

        assert!(event.is_none());
    }

    #[test]
    fn test_inspection_data_includes_repair_status() {
        let mut anchor = Anchor::new(0.5, 2, None);
        anchor.repair(); // Mark as repaired

        let anchors = vec![(1, &anchor)];

        let mut input = InputState::new();
        input.press_e();

        let event = anchor_interaction_system(Some(1), &anchors, &input);

        assert!(event.is_some());
        let event = event.unwrap();
        assert!(event.anchor_data.is_repaired);
    }

    #[test]
    fn test_multiple_anchors_only_closest_inspected() {
        let anchor1 = Anchor::new(1.0, 10, Some(AbilityType::EchoDash));
        let anchor2 = Anchor::new(0.5, 5, Some(AbilityType::BarricadeDeploy));
        let anchors = vec![(1, &anchor1), (2, &anchor2)];

        let mut input = InputState::new();
        input.press_e();

        // Proximity system would return closest (2 in this case)
        let event = anchor_interaction_system(Some(2), &anchors, &input);

        assert!(event.is_some());
        assert_eq!(event.unwrap().anchor_id, 2);
    }

    #[test]
    fn test_broken_anchor_can_be_inspected() {
        let anchor = Anchor::new(0.0, 1, None); // Broken anchor
        let anchors = vec![(1, &anchor)];

        let mut input = InputState::new();
        input.press_e();

        let event = anchor_interaction_system(Some(1), &anchors, &input);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.anchor_data.stability, 0.0);
        assert_eq!(event.anchor_data.repair_cost, 1);
    }

    #[test]
    fn test_perfect_anchor_inspection() {
        let anchor = Anchor::new(1.0, 0, None); // Perfect, no repair cost
        let anchors = vec![(1, &anchor)];

        let mut input = InputState::new();
        input.press_e();

        let event = anchor_interaction_system(Some(1), &anchors, &input);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.anchor_data.stability, 1.0);
        assert_eq!(event.anchor_data.repair_cost, 0);
        assert_eq!(event.anchor_data.unlocks_ability, None);
    }

    #[test]
    fn test_e_key_state_transitions() {
        let anchor = Anchor::new(0.7, 5, None);
        let anchors = vec![(1, &anchor)];

        let mut input = InputState::new();

        // Frame 1: Press E
        input.press_e();
        let event1 = anchor_interaction_system(Some(1), &anchors, &input);
        assert!(event1.is_some());

        // Frame 2: E still held (not just_pressed)
        input.e_just_pressed = false;
        let event2 = anchor_interaction_system(Some(1), &anchors, &input);
        assert!(event2.is_none());

        // Frame 3: Release E
        input.release_e();
        let event3 = anchor_interaction_system(Some(1), &anchors, &input);
        assert!(event3.is_none());
    }
}
