use astraweave_dialogue::{DialogueGraph, DialogueNode, DialogueResponse};

#[test]
fn test_dialogue_validation() {
    let graph = DialogueGraph {
        nodes: vec![
            DialogueNode {
                id: "start".into(),
                text: "Hello!".into(),
                responses: vec![DialogueResponse { text: "Bye".into(), next_id: Some("end".into()) }],
            },
            DialogueNode {
                id: "end".into(),
                text: "Goodbye.".into(),
                responses: vec![],
            },
        ],
    };
    assert!(graph.validate().is_ok());
}

#[test]
fn test_dialogue_invalid_next_id() {
    let graph = DialogueGraph {
        nodes: vec![
            DialogueNode {
                id: "start".into(),
                text: "Hello!".into(),
                responses: vec![DialogueResponse { text: "Bye".into(), next_id: Some("missing".into()) }],
            },
        ],
    };
    assert!(graph.validate().is_err());
}
