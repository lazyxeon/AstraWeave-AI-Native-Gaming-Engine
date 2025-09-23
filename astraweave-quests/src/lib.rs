//! Quest system: authorable quest steps, validation, and execution
use serde::{Serialize, Deserialize};

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
