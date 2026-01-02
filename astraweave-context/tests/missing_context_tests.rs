use astraweave_context::{
    ContextWindow, ContextWindowConfig, WindowType, Role, Message, SerializableContextWindow
};
use serde_json;

#[test]
fn test_context_window_serialization() {
    let mut config = ContextWindowConfig::default();
    config.max_messages = 5;
    config.window_type = WindowType::Sliding;
    
    let mut window = ContextWindow::new(config);
    
    window.add_message(Message::new(Role::User, "Message 1".to_string())).unwrap();
    window.add_message(Message::new(Role::Assistant, "Response 1".to_string())).unwrap();
    
    // Export
    let exported = window.export();
    
    // Serialize exported data
    let serialized = serde_json::to_string(&exported).unwrap();
    
    // Deserialize
    let deserialized_data: SerializableContextWindow = serde_json::from_str(&serialized).unwrap();
    
    // Import back
    let imported_window = ContextWindow::import(deserialized_data);
    
    assert_eq!(imported_window.message_count(), 2);
    assert_eq!(imported_window.get_messages()[0].content, "Message 1");
}

#[test]
fn test_context_compression_via_pruning() {
    let mut config = ContextWindowConfig::default();
    config.max_tokens = 100; // Small limit
    config.window_type = WindowType::Attention;
    
    let mut window = ContextWindow::new(config);
    
    // Add messages to fill window
    // "Hello" is ~1 token. We need more.
    let long_msg = "This is a somewhat long message that should take up some tokens.".to_string();
    
    // Add until full
    for i in 0..10 {
        window.add_message(Message::new(Role::User, format!("{} {}", i, long_msg))).unwrap();
    }
    
    let util_before = window.utilization();
    // It should be full or close to full, so utilization ~ 1.0 (or pruned to < 1.0)
    
    // If pruning works, utilization should be <= 1.0
    assert!(util_before <= 1.0);
    
    // Add a very important message
    let mut important = Message::new(Role::System, "CRITICAL MESSAGE".to_string());
    important.preserve = true;
    window.add_message(important).unwrap();
    
    // Verify important message is kept
    let messages = window.get_messages();
    assert!(messages.iter().any(|m| m.content == "CRITICAL MESSAGE"));
    
    // Verify utilization is still managed
    assert!(window.utilization() <= 1.0);
}
