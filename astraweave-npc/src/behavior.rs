use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmoteKind {
    Wave,
    Nod,
    Shrug,
    Point,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NpcAction {
    Say { text: String },
    MoveTo { pos: Vec3, speed: f32 },
    Emote { kind: EmoteKind },
    OpenShop,
    GiveQuest { id: String },
    CallGuards { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NpcPlan {
    pub actions: Vec<NpcAction>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcMode {
    Idle,
    Patrolling,
    Working,
    Conversing,
    Flee,
    Combat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emote_kind_equality() {
        assert_eq!(EmoteKind::Wave, EmoteKind::Wave);
        assert_ne!(EmoteKind::Wave, EmoteKind::Nod);
    }

    #[test]
    fn test_emote_kind_serialization() {
        for kind in [EmoteKind::Wave, EmoteKind::Nod, EmoteKind::Shrug, EmoteKind::Point] {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: EmoteKind = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn test_npc_action_say() {
        let action = NpcAction::Say { text: "Hello!".to_string() };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: NpcAction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, action);
    }

    #[test]
    fn test_npc_action_move_to() {
        let action = NpcAction::MoveTo { pos: Vec3::new(1.0, 2.0, 3.0), speed: 5.0 };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: NpcAction = serde_json::from_str(&json).unwrap();
        if let NpcAction::MoveTo { pos, speed } = parsed {
            assert_eq!(pos, Vec3::new(1.0, 2.0, 3.0));
            assert_eq!(speed, 5.0);
        } else {
            panic!("Expected MoveTo");
        }
    }

    #[test]
    fn test_npc_action_emote() {
        let action = NpcAction::Emote { kind: EmoteKind::Nod };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: NpcAction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, action);
    }

    #[test]
    fn test_npc_action_open_shop() {
        let action = NpcAction::OpenShop;
        let json = serde_json::to_string(&action).unwrap();
        let parsed: NpcAction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, action);
    }

    #[test]
    fn test_npc_action_give_quest() {
        let action = NpcAction::GiveQuest { id: "quest_001".to_string() };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: NpcAction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, action);
    }

    #[test]
    fn test_npc_action_call_guards() {
        let action = NpcAction::CallGuards { reason: "Thief spotted!".to_string() };
        let json = serde_json::to_string(&action).unwrap();
        let parsed: NpcAction = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, action);
    }

    #[test]
    fn test_npc_plan_default() {
        let plan = NpcPlan::default();
        assert!(plan.actions.is_empty());
    }

    #[test]
    fn test_npc_plan_with_actions() {
        let plan = NpcPlan {
            actions: vec![
                NpcAction::Say { text: "Hello".to_string() },
                NpcAction::Emote { kind: EmoteKind::Wave },
                NpcAction::OpenShop,
            ],
        };
        assert_eq!(plan.actions.len(), 3);
    }

    #[test]
    fn test_npc_plan_serialization() {
        let plan = NpcPlan {
            actions: vec![
                NpcAction::Say { text: "Hi".to_string() },
                NpcAction::GiveQuest { id: "q1".to_string() },
            ],
        };
        let json = serde_json::to_string(&plan).unwrap();
        let parsed: NpcPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.actions.len(), 2);
    }

    #[test]
    fn test_npc_mode_equality() {
        assert_eq!(NpcMode::Idle, NpcMode::Idle);
        assert_ne!(NpcMode::Idle, NpcMode::Combat);
    }

    #[test]
    fn test_npc_mode_all_variants() {
        let modes = [
            NpcMode::Idle,
            NpcMode::Patrolling,
            NpcMode::Working,
            NpcMode::Conversing,
            NpcMode::Flee,
            NpcMode::Combat,
        ];
        for mode in modes {
            let cloned = mode;
            assert_eq!(mode, cloned);
        }
    }

    #[test]
    fn test_npc_plan_clone() {
        let plan = NpcPlan {
            actions: vec![NpcAction::OpenShop],
        };
        let cloned = plan.clone();
        assert_eq!(cloned.actions.len(), 1);
    }
}

