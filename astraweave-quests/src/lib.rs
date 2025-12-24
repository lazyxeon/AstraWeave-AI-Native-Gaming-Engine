//! Quest system: authorable quest steps, validation, and execution
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct QuestStep {
    pub description: String,
    pub completed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Quest {
    pub title: String,
    pub steps: Vec<QuestStep>,
}

impl Quest {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.is_empty() {
            return Err("Quest title cannot be empty".into());
        }
        if self.steps.is_empty() {
            return Err("Quest must have at least one step".into());
        }
        Ok(())
    }
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| s.completed)
    }
}

// LLM-powered quest system modules
pub mod llm_quests;
pub use llm_quests::*;

pub mod components;
pub use components::*;

pub mod systems;
pub use systems::*;

// Phase 10: Terrain-driven quest generation
pub mod terrain_quests;
pub use terrain_quests::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_step_default() {
        let step = QuestStep::default();
        assert!(step.description.is_empty());
        assert!(!step.completed);
    }

    #[test]
    fn test_quest_step_with_description() {
        let step = QuestStep {
            description: "Find the sword".to_string(),
            completed: false,
        };
        assert_eq!(step.description, "Find the sword");
        assert!(!step.completed);
    }

    #[test]
    fn test_quest_step_completed() {
        let step = QuestStep {
            description: "Collect herbs".to_string(),
            completed: true,
        };
        assert!(step.completed);
    }

    #[test]
    fn test_quest_default() {
        let quest = Quest::default();
        assert!(quest.title.is_empty());
        assert!(quest.steps.is_empty());
    }

    #[test]
    fn test_quest_validate_empty_title() {
        let quest = Quest {
            title: String::new(),
            steps: vec![QuestStep::default()],
        };
        let result = quest.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Quest title cannot be empty");
    }

    #[test]
    fn test_quest_validate_empty_steps() {
        let quest = Quest {
            title: "Epic Quest".to_string(),
            steps: vec![],
        };
        let result = quest.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Quest must have at least one step");
    }

    #[test]
    fn test_quest_validate_success() {
        let quest = Quest {
            title: "Dragon Slayer".to_string(),
            steps: vec![QuestStep {
                description: "Find the dragon".to_string(),
                completed: false,
            }],
        };
        assert!(quest.validate().is_ok());
    }

    #[test]
    fn test_quest_is_complete_empty_steps() {
        let quest = Quest {
            title: "Empty Quest".to_string(),
            steps: vec![],
        };
        // Empty iterator returns true for all()
        assert!(quest.is_complete());
    }

    #[test]
    fn test_quest_is_complete_all_done() {
        let quest = Quest {
            title: "Finished Quest".to_string(),
            steps: vec![
                QuestStep {
                    description: "Step 1".to_string(),
                    completed: true,
                },
                QuestStep {
                    description: "Step 2".to_string(),
                    completed: true,
                },
            ],
        };
        assert!(quest.is_complete());
    }

    #[test]
    fn test_quest_is_complete_partial() {
        let quest = Quest {
            title: "In Progress".to_string(),
            steps: vec![
                QuestStep {
                    description: "Step 1".to_string(),
                    completed: true,
                },
                QuestStep {
                    description: "Step 2".to_string(),
                    completed: false,
                },
            ],
        };
        assert!(!quest.is_complete());
    }

    #[test]
    fn test_quest_is_complete_none_done() {
        let quest = Quest {
            title: "Fresh Quest".to_string(),
            steps: vec![
                QuestStep {
                    description: "Step 1".to_string(),
                    completed: false,
                },
                QuestStep {
                    description: "Step 2".to_string(),
                    completed: false,
                },
            ],
        };
        assert!(!quest.is_complete());
    }

    #[test]
    fn test_quest_serialization() {
        let quest = Quest {
            title: "Test Quest".to_string(),
            steps: vec![QuestStep {
                description: "Do something".to_string(),
                completed: true,
            }],
        };
        let json = serde_json::to_string(&quest).unwrap();
        let deserialized: Quest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, quest.title);
        assert_eq!(deserialized.steps.len(), 1);
        assert!(deserialized.steps[0].completed);
    }

    #[test]
    fn test_quest_step_serialization() {
        let step = QuestStep {
            description: "Collect 10 apples".to_string(),
            completed: false,
        };
        let json = serde_json::to_string(&step).unwrap();
        let deserialized: QuestStep = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.description, step.description);
        assert_eq!(deserialized.completed, step.completed);
    }
}
