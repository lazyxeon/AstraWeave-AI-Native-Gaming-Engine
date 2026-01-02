use anyhow::Result;
use astraweave_context::{
    ConversationHistory, ContextConfig, Role, OverflowStrategy
};
use astraweave_llm::LlmClient;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Mock LLM for summarization
struct MockSummarizer;

#[async_trait::async_trait]
impl LlmClient for MockSummarizer {
    async fn complete(&self, _prompt: &str) -> Result<String> {
        Ok("This is a summary of the conversation.".to_string())
    }
}

#[tokio::test]
async fn test_retrieve_message_by_id() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    let msg_id = history.add_message(Role::User, "Test message".to_string()).await.unwrap();
    
    let retrieved = history.get_message(&msg_id).unwrap();
    assert_eq!(retrieved.content, "Test message");
    assert_eq!(retrieved.role, Role::User);
}

#[tokio::test]
async fn test_prune_summarization() {
    let mut config = ContextConfig::default();
    config.max_tokens = 50; // Low limit to trigger pruning
    config.overflow_strategy = OverflowStrategy::Summarization;
    config.enable_summarization = true;
    config.summarization_threshold = 2; // Low threshold

    let summarizer = Arc::new(MockSummarizer);
    let history = ConversationHistory::with_llm_client(config, summarizer);

    // Add messages to trigger summarization
    // Use longer messages to ensure token count exceeds limit
    for i in 0..10 {
        history.add_message(Role::User, format!("This is a longer message to ensure we exceed the token limit {}", i)).await.unwrap();
    }

    // Check if summary exists
    let context = history.get_context(1000).await.unwrap();
    assert!(context.contains("SUMMARY: This is a summary of the conversation."));
    
    // Metrics should show summarization happened
    let metrics = history.get_metrics();
    assert!(metrics.summarized_messages > 0);
    assert!(metrics.prune_count > 0);
}

#[tokio::test]
async fn test_prune_hybrid() {
    let mut config = ContextConfig::default();
    config.max_tokens = 50; 
    config.overflow_strategy = OverflowStrategy::Hybrid;
    config.enable_summarization = true;
    config.summarization_threshold = 2;

    let summarizer = Arc::new(MockSummarizer);
    let history = ConversationHistory::with_llm_client(config, summarizer);

    for i in 0..20 {
        history.add_message(Role::User, format!("Message {}", i)).await.unwrap();
    }

    let context = history.get_context(1000).await.unwrap();
    assert!(context.contains("SUMMARY:"));
    
    // Should also have recent messages
    assert!(context.contains("Message 19"));
}

#[tokio::test]
async fn test_token_counting_integration() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    history.add_message(Role::User, "Hello world".to_string()).await.unwrap();
    
    let tokens = history.get_total_tokens();
    assert!(tokens > 0);
    
    // "Hello world" is typically 2-3 tokens depending on tokenizer
    assert!(tokens >= 2 && tokens <= 3); 
}

#[tokio::test]
async fn test_concurrent_access() {
    let config = ContextConfig::default();
    let history = Arc::new(ConversationHistory::new(config));
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let h = history.clone();
        handles.push(tokio::spawn(async move {
            h.add_message(Role::User, format!("Message {}", i)).await.unwrap();
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    assert_eq!(history.get_recent_messages(20).len(), 10);
}

#[tokio::test]
async fn test_metrics_tracking() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    history.add_message(Role::User, "Test".to_string()).await.unwrap();
    history.get_context(100).await.unwrap();
    
    let metrics = history.get_metrics();
    assert_eq!(metrics.total_messages, 1);
    // processing_time_ms might be 0 on fast machines
    assert!(metrics.utilization > 0.0);
}

#[tokio::test]
async fn test_message_metadata() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "test".to_string());
    metadata.insert("priority".to_string(), "high".to_string());

    let msg_id = history.add_message_with_metadata(
        Role::User, 
        "Metadata message".to_string(),
        metadata
    ).await.unwrap();

    let msg = history.get_message(&msg_id).unwrap();
    assert_eq!(msg.metadata.get("source").unwrap(), "test");
    assert_eq!(msg.metadata.get("priority").unwrap(), "high");
}

#[tokio::test]
async fn test_clear_history() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    history.add_message(Role::User, "Msg 1".to_string()).await.unwrap();
    assert_eq!(history.get_recent_messages(10).len(), 1);
    
    history.clear();
    assert_eq!(history.get_recent_messages(10).len(), 0);
    assert_eq!(history.get_total_tokens(), 0);
}

#[tokio::test]
async fn test_get_recent_n() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    for i in 0..5 {
        history.add_message(Role::User, format!("Msg {}", i)).await.unwrap();
    }

    let recent = history.get_recent_messages(3);
    assert_eq!(recent.len(), 3);
    // get_recent_messages returns chronological order of the recent subset
    // Msg 0, 1, 2, 3, 4 -> Recent 3: Msg 2, Msg 3, Msg 4
    
    assert_eq!(recent[0].content, "Msg 2");
    assert_eq!(recent[2].content, "Msg 4");
}

#[tokio::test]
async fn test_get_by_role() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    history.add_message(Role::User, "User 1".to_string()).await.unwrap();
    history.add_message(Role::Assistant, "Assist 1".to_string()).await.unwrap();
    history.add_message(Role::User, "User 2".to_string()).await.unwrap();

    let user_msgs = history.get_messages_by_role(Role::User);
    assert_eq!(user_msgs.len(), 2);
    assert_eq!(user_msgs[0].content, "User 1");
    assert_eq!(user_msgs[1].content, "User 2");
}

#[tokio::test]
async fn test_get_by_time_range() {
    let config = ContextConfig::default();
    let history = ConversationHistory::new(config);

    let start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    history.add_message(Role::User, "Msg 1".to_string()).await.unwrap();
    
    sleep(Duration::from_secs(2)).await;
    
    let mid = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
        
    history.add_message(Role::User, "Msg 2".to_string()).await.unwrap();

    let end = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Range covering only first message
    let range1 = history.get_messages_by_time_range(start, mid - 1);
    assert_eq!(range1.len(), 1);
    assert_eq!(range1[0].content, "Msg 1");

    // Range covering both
    let range2 = history.get_messages_by_time_range(start, end);
    assert_eq!(range2.len(), 2);
}
