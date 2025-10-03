// Test for editor serialization and hot-reload signaling
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Default)]
struct DialogueDoc {
    title: String,
    nodes: Vec<DialogueNode>,
}
#[derive(Clone, Serialize, Deserialize, Default)]
struct DialogueNode {
    id: String,
    text: String,
    responses: Vec<DialogueResponse>,
}
#[derive(Clone, Serialize, Deserialize, Default)]
struct DialogueResponse {
    text: String,
    next_id: Option<String>,
}

#[test]
fn test_dialogue_serialization_and_reload_signal() {
    let doc = DialogueDoc {
        title: "Test Dialogue".into(),
        nodes: vec![DialogueNode {
            id: "start".into(),
            text: "Hello!".into(),
            responses: vec![DialogueResponse {
                text: "Bye".into(),
                next_id: None,
            }],
        }],
    };
    let dir = PathBuf::from("test_content/dialogue");
    let _ = fs::create_dir_all(&dir);
    let p = dir.join("test.dialogue.toml");
    let txt = toml::to_string_pretty(&doc).unwrap();
    fs::write(&p, txt).unwrap();
    // Simulate hot-reload signal
    let reload_path = PathBuf::from("test_content/reload.signal");
    fs::write(&reload_path, Uuid::new_v4().to_string()).unwrap();
    assert!(p.exists());
    assert!(reload_path.exists());
    // Clean up
    let _ = fs::remove_file(&p);
    let _ = fs::remove_file(&reload_path);
}
