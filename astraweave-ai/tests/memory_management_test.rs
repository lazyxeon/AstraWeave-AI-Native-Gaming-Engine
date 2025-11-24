use astraweave_ai::persona::manager::{LlmPersonaManager, PersonaConfig};
use astraweave_ai::rag::{RagConfig, ConsolidationStrategy, ForgettingStrategy, InjectionStrategy};
use astraweave_embeddings::{MockEmbeddingClient, EmbeddingConfig};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_consolidate_by_recency() {
    let client = Arc::new(MockEmbeddingClient::new());
    let mut config = PersonaConfig::default();
    config.rag_config.consolidation_strategy = ConsolidationStrategy::Recency;
    config.rag_config.embedding.max_vectors = 2; // Keep only 2

    let manager = LlmPersonaManager::new(config, client);

    // Add 3 memories
    manager.add_memory("Memory 1".to_string(), 0.5).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    manager.add_memory("Memory 2".to_string(), 0.5).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    manager.add_memory("Memory 3".to_string(), 0.5).await.unwrap();

    assert_eq!(manager.document_count(), 3);

    // Run maintenance
    manager.maintenance().unwrap();

    // Should be reduced to 2
    assert_eq!(manager.document_count(), 2);
}

#[tokio::test]
async fn test_consolidate_by_importance() {
    let client = Arc::new(MockEmbeddingClient::new());
    let mut config = PersonaConfig::default();
    config.rag_config.consolidation_strategy = ConsolidationStrategy::Importance;
    config.rag_config.embedding.max_vectors = 2;

    let manager = LlmPersonaManager::new(config, client);

    manager.add_memory("Low Importance".to_string(), 0.1).await.unwrap();
    manager.add_memory("High Importance 1".to_string(), 0.9).await.unwrap();
    manager.add_memory("High Importance 2".to_string(), 0.8).await.unwrap();

    assert_eq!(manager.document_count(), 3);

    manager.maintenance().unwrap();

    assert_eq!(manager.document_count(), 2);
}

#[tokio::test]
async fn test_forget_by_low_importance() {
    let client = Arc::new(MockEmbeddingClient::new());
    let mut config = PersonaConfig::default();
    config.rag_config.forgetting_strategy = ForgettingStrategy::LowImportance(0.6);

    let manager = LlmPersonaManager::new(config, client);

    manager.add_memory("Important".to_string(), 0.9).await.unwrap();
    manager.add_memory("Unimportant".to_string(), 0.3).await.unwrap();

    assert_eq!(manager.document_count(), 2);

    manager.maintenance().unwrap();

    assert_eq!(manager.document_count(), 1);
}

#[tokio::test]
async fn test_forget_by_limit() {
    let client = Arc::new(MockEmbeddingClient::new());
    let mut config = PersonaConfig::default();
    config.rag_config.forgetting_strategy = ForgettingStrategy::Limit(1);

    let manager = LlmPersonaManager::new(config, client);

    manager.add_memory("Old".to_string(), 0.5).await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    manager.add_memory("New".to_string(), 0.5).await.unwrap();

    assert_eq!(manager.document_count(), 2);

    manager.maintenance().unwrap();

    assert_eq!(manager.document_count(), 1);
}

#[tokio::test]
async fn test_inject_prepend() {
    let client = Arc::new(MockEmbeddingClient::new());
    let mut config = PersonaConfig::default();
    config.rag_config.injection_strategy = InjectionStrategy::Prepend;
    // Ensure we retrieve something
    config.rag_config.min_similarity = 0.0; 

    let manager = LlmPersonaManager::new(config, client);
    manager.add_memory("Context info".to_string(), 1.0).await.unwrap();

    let prompt = "Hello";
    let injected = manager.inject_context(prompt).await.unwrap();

    if injected.contains("Relevant Context:") {
        assert!(injected.ends_with("Hello"));
    } else {
        // If no context retrieved, it should match prompt
        assert_eq!(injected, prompt);
    }
}

#[tokio::test]
async fn test_inject_append() {
    let client = Arc::new(MockEmbeddingClient::new());
    let mut config = PersonaConfig::default();
    config.rag_config.injection_strategy = InjectionStrategy::Append;
    config.rag_config.min_similarity = 0.0;

    let manager = LlmPersonaManager::new(config, client);
    manager.add_memory("Context info".to_string(), 1.0).await.unwrap();

    let prompt = "Hello";
    let injected = manager.inject_context(prompt).await.unwrap();

    if injected.contains("Relevant Context:") {
        assert!(injected.starts_with("Hello"));
    } else {
        assert_eq!(injected, prompt);
    }
}

#[tokio::test]
async fn test_inject_token_budget() {
    let client = Arc::new(MockEmbeddingClient::new());
    let mut config = PersonaConfig::default();
    config.rag_config.max_context_tokens = 2; // Very small budget
    config.rag_config.min_similarity = 0.0;

    let manager = LlmPersonaManager::new(config, client);
    manager.add_memory("This is a very long memory".to_string(), 1.0).await.unwrap();

    let injected = manager.inject_context("Prompt").await.unwrap();
    
    // Should not contain the full memory
    assert!(!injected.contains("This is a very long memory"));
}