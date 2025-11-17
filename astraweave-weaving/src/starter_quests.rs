// Starter quests for Veilweaver gameplay
// Provides 3 introductory quests to teach core mechanics

use crate::{Quest, ObjectiveType, QuestReward};
use glam::Vec3;

/// Create the first starter quest: "Stabilize the Anchors"
/// Teaches players about anchor repair mechanics
pub fn quest_stabilize_anchors() -> Quest {
    Quest::new(
        "stabilize_anchors",
        "Stabilize the Anchors",
        "The reality anchors are failing. Repair 3 anchors to at least 80% stability to restore balance.",
    )
    .with_objective(ObjectiveType::Repair {
        required: 3,
        current: 0,
        min_stability: 0.8,
    })
    .with_reward(QuestReward::EchoCurrency(100))
    .with_reward(QuestReward::AbilityUnlock("Echo Dash".to_string()))
}

/// Create the second starter quest: "Clear the Corruption"
/// Teaches players about combat and enemy encounters
pub fn quest_clear_corruption() -> Quest {
    Quest::new(
        "clear_corruption",
        "Clear the Corruption",
        "Corrupted entities are threatening the anchors. Defeat 10 enemies to push back the corruption.",
    )
    .with_objective(ObjectiveType::Kill {
        target_type: "enemy".to_string(),
        required: 10,
        current: 0,
    })
    .with_reward(QuestReward::EchoCurrency(150))
    .with_reward(QuestReward::StatBoost {
        stat: "MaxHealth".to_string(),
        amount: 25.0,
    })
    .with_prerequisite("stabilize_anchors")
}

/// Create the third starter quest: "Restore the Beacon"
/// Teaches players about item collection and exploration
pub fn quest_restore_beacon() -> Quest {
    Quest::new(
        "restore_beacon",
        "Restore the Beacon",
        "Collect 5 echo shards scattered across the realm and deliver them to the central anchor to amplify its power.",
    )
    .with_objective(ObjectiveType::Fetch {
        item_name: "echo_shard".to_string(),
        required: 5,
        current: 0,
        delivery_location: Vec3::new(0.0, 0.0, 0.0), // Central anchor position
    })
    .with_objective(ObjectiveType::Explore {
        location_name: "Central Anchor".to_string(),
        target_position: Vec3::new(0.0, 0.0, 0.0),
        radius: 5.0,
        discovered: false,
    })
    .with_reward(QuestReward::EchoCurrency(200))
    .with_reward(QuestReward::AbilityUnlock("Echo Shield".to_string()))
    .with_prerequisite("clear_corruption")
}

/// Create all starter quests and return them as a Vec
pub fn all_starter_quests() -> Vec<Quest> {
    vec![
        quest_stabilize_anchors(),
        quest_clear_corruption(),
        quest_restore_beacon(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stabilize_anchors_quest() {
        let quest = quest_stabilize_anchors();
        
        assert_eq!(quest.id, "stabilize_anchors");
        assert_eq!(quest.title, "Stabilize the Anchors");
        assert_eq!(quest.objectives.len(), 1);
        assert_eq!(quest.rewards.len(), 2);
        assert!(quest.prerequisites.is_empty());
        
        // Check objective
        if let ObjectiveType::Repair { required, current, min_stability } = &quest.objectives[0] {
            assert_eq!(*required, 3);
            assert_eq!(*current, 0);
            assert_eq!(*min_stability, 0.8);
        } else {
            panic!("Wrong objective type");
        }
        
        // Check rewards
        assert!(matches!(quest.rewards[0], QuestReward::EchoCurrency(100)));
        if let QuestReward::AbilityUnlock(ability) = &quest.rewards[1] {
            assert_eq!(ability, "Echo Dash");
        } else {
            panic!("Wrong reward type");
        }
    }

    #[test]
    fn test_clear_corruption_quest() {
        let quest = quest_clear_corruption();
        
        assert_eq!(quest.id, "clear_corruption");
        assert_eq!(quest.title, "Clear the Corruption");
        assert_eq!(quest.objectives.len(), 1);
        assert_eq!(quest.rewards.len(), 2);
        assert_eq!(quest.prerequisites.len(), 1);
        assert_eq!(quest.prerequisites[0], "stabilize_anchors");
        
        // Check objective
        if let ObjectiveType::Kill { target_type, required, current } = &quest.objectives[0] {
            assert_eq!(target_type, "enemy");
            assert_eq!(*required, 10);
            assert_eq!(*current, 0);
        } else {
            panic!("Wrong objective type");
        }
        
        // Check rewards
        assert!(matches!(quest.rewards[0], QuestReward::EchoCurrency(150)));
        if let QuestReward::StatBoost { stat, amount } = &quest.rewards[1] {
            assert_eq!(stat, "MaxHealth");
            assert_eq!(*amount, 25.0);
        } else {
            panic!("Wrong reward type");
        }
    }

    #[test]
    fn test_restore_beacon_quest() {
        let quest = quest_restore_beacon();
        
        assert_eq!(quest.id, "restore_beacon");
        assert_eq!(quest.title, "Restore the Beacon");
        assert_eq!(quest.objectives.len(), 2);
        assert_eq!(quest.rewards.len(), 2);
        assert_eq!(quest.prerequisites.len(), 1);
        assert_eq!(quest.prerequisites[0], "clear_corruption");
        
        // Check objectives
        if let ObjectiveType::Fetch { item_name, required, current, delivery_location } = &quest.objectives[0] {
            assert_eq!(item_name, "echo_shard");
            assert_eq!(*required, 5);
            assert_eq!(*current, 0);
            assert_eq!(*delivery_location, Vec3::ZERO);
        } else {
            panic!("Wrong first objective type");
        }
        
        if let ObjectiveType::Explore { location_name, target_position, radius, discovered } = &quest.objectives[1] {
            assert_eq!(location_name, "Central Anchor");
            assert_eq!(*target_position, Vec3::ZERO);
            assert_eq!(*radius, 5.0);
            assert!(!discovered);
        } else {
            panic!("Wrong second objective type");
        }
        
        // Check rewards
        assert!(matches!(quest.rewards[0], QuestReward::EchoCurrency(200)));
        if let QuestReward::AbilityUnlock(ability) = &quest.rewards[1] {
            assert_eq!(ability, "Echo Shield");
        } else {
            panic!("Wrong reward type");
        }
    }

    #[test]
    fn test_all_starter_quests() {
        let quests = all_starter_quests();
        
        assert_eq!(quests.len(), 3);
        assert_eq!(quests[0].id, "stabilize_anchors");
        assert_eq!(quests[1].id, "clear_corruption");
        assert_eq!(quests[2].id, "restore_beacon");
    }

    #[test]
    fn test_quest_progression_chain() {
        let quests = all_starter_quests();
        
        // Quest 1 has no prerequisites
        assert!(quests[0].prerequisites.is_empty());
        
        // Quest 2 requires Quest 1
        assert_eq!(quests[1].prerequisites.len(), 1);
        assert_eq!(quests[1].prerequisites[0], "stabilize_anchors");
        
        // Quest 3 requires Quest 2
        assert_eq!(quests[2].prerequisites.len(), 1);
        assert_eq!(quests[2].prerequisites[0], "clear_corruption");
    }

    #[test]
    fn test_escalating_rewards() {
        let quests = all_starter_quests();
        
        // Quest 1: 100 Echo + ability
        if let QuestReward::EchoCurrency(amt) = quests[0].rewards[0] {
            assert_eq!(amt, 100);
        }
        
        // Quest 2: 150 Echo + stat boost (more valuable)
        if let QuestReward::EchoCurrency(amt) = quests[1].rewards[0] {
            assert_eq!(amt, 150);
        }
        
        // Quest 3: 200 Echo + ability (most valuable)
        if let QuestReward::EchoCurrency(amt) = quests[2].rewards[0] {
            assert_eq!(amt, 200);
        }
    }
}
