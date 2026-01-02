//! Dialogue system: branching dialogue graphs, validation, and execution
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_node_default() {
        let node = DialogueNode::default();
        assert!(node.id.is_empty());
        assert!(node.text.is_empty());
        assert!(node.responses.is_empty());
    }

    #[test]
    fn test_dialogue_response_default() {
        let resp = DialogueResponse::default();
        assert!(resp.text.is_empty());
        assert!(resp.next_id.is_none());
    }

    #[test]
    fn test_dialogue_graph_default() {
        let graph = DialogueGraph::default();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_validate_empty_graph() {
        let graph = DialogueGraph::default();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_single_node_no_responses() {
        let graph = DialogueGraph {
            nodes: vec![DialogueNode {
                id: "start".into(),
                text: "Hello!".into(),
                responses: vec![],
            }],
        };
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_valid_chain() {
        let graph = DialogueGraph {
            nodes: vec![
                DialogueNode {
                    id: "start".into(),
                    text: "Hello!".into(),
                    responses: vec![DialogueResponse {
                        text: "Continue".into(),
                        next_id: Some("middle".into()),
                    }],
                },
                DialogueNode {
                    id: "middle".into(),
                    text: "How are you?".into(),
                    responses: vec![DialogueResponse {
                        text: "End".into(),
                        next_id: Some("end".into()),
                    }],
                },
                DialogueNode {
                    id: "end".into(),
                    text: "Goodbye!".into(),
                    responses: vec![],
                },
            ],
        };
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_next_id() {
        let graph = DialogueGraph {
            nodes: vec![DialogueNode {
                id: "start".into(),
                text: "Hello!".into(),
                responses: vec![DialogueResponse {
                    text: "Bye".into(),
                    next_id: Some("nonexistent".into()),
                }],
            }],
        };
        let result = graph.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent"));
    }

    #[test]
    fn test_validate_none_next_id_is_valid() {
        let graph = DialogueGraph {
            nodes: vec![DialogueNode {
                id: "start".into(),
                text: "Hello!".into(),
                responses: vec![DialogueResponse {
                    text: "Bye".into(),
                    next_id: None,
                }],
            }],
        };
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_multiple_responses() {
        let graph = DialogueGraph {
            nodes: vec![
                DialogueNode {
                    id: "start".into(),
                    text: "Choose your path".into(),
                    responses: vec![
                        DialogueResponse {
                            text: "Option A".into(),
                            next_id: Some("path_a".into()),
                        },
                        DialogueResponse {
                            text: "Option B".into(),
                            next_id: Some("path_b".into()),
                        },
                        DialogueResponse {
                            text: "End now".into(),
                            next_id: None,
                        },
                    ],
                },
                DialogueNode {
                    id: "path_a".into(),
                    text: "You chose A".into(),
                    responses: vec![],
                },
                DialogueNode {
                    id: "path_b".into(),
                    text: "You chose B".into(),
                    responses: vec![],
                },
            ],
        };
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_get_node_found() {
        let graph = DialogueGraph {
            nodes: vec![
                DialogueNode {
                    id: "node1".into(),
                    text: "First".into(),
                    responses: vec![],
                },
                DialogueNode {
                    id: "node2".into(),
                    text: "Second".into(),
                    responses: vec![],
                },
            ],
        };
        let node = graph.get_node("node2");
        assert!(node.is_some());
        assert_eq!(node.unwrap().text, "Second");
    }

    #[test]
    fn test_get_node_not_found() {
        let graph = DialogueGraph {
            nodes: vec![DialogueNode {
                id: "node1".into(),
                text: "First".into(),
                responses: vec![],
            }],
        };
        assert!(graph.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_get_node_empty_graph() {
        let graph = DialogueGraph::default();
        assert!(graph.get_node("any").is_none());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let graph = DialogueGraph {
            nodes: vec![DialogueNode {
                id: "start".into(),
                text: "Hello!".into(),
                responses: vec![DialogueResponse {
                    text: "Reply".into(),
                    next_id: Some("end".into()),
                }],
            }],
        };
        let json = serde_json::to_string(&graph).unwrap();
        let parsed: DialogueGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.nodes[0].id, "start");
    }

    #[test]
    fn test_clone() {
        let node = DialogueNode {
            id: "test".into(),
            text: "Text".into(),
            responses: vec![],
        };
        let cloned = node.clone();
        assert_eq!(cloned.id, node.id);
    }
}
