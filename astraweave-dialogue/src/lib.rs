//! Dialogue system: branching dialogue graphs, validation, and execution
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DialogueNode {
    pub id: String,
    pub text: String,
    pub responses: Vec<DialogueResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DialogueResponse {
    pub text: String,
    pub next_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DialogueGraph {
    pub nodes: Vec<DialogueNode>,
}

impl DialogueGraph {
    pub fn validate(&self) -> Result<(), String> {
        let ids: std::collections::HashSet<_> = self.nodes.iter().map(|n| &n.id).collect();
        for node in &self.nodes {
            for resp in &node.responses {
                if let Some(ref next) = resp.next_id {
                    if !ids.contains(next) {
                        return Err(format!("Invalid next_id: {}", next));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn get_node(&self, id: &str) -> Option<&DialogueNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}
