use anyhow::Result;
use astraweave_embeddings::{
    Memory, MemoryCategory, MockEmbeddingClient, VectorStore,
};
use astraweave_llm::MockLlm;
use astraweave_rag::{
    pipeline::{RagPipeline, VectorStoreWrapper},
    RagConfig,
};
use std::collections::HashMap;
use std::sync::Arc;

// Helper to create a standard test pipeline
async fn create_test_pipeline() -> RagPipeline {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    // Use a small dimension for testing
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    let llm_client = Arc::new(MockLlm);
    let config = RagConfig::default();

    RagPipeline::new(
        embedding_client,
        vector_store,
        Some(llm_client),
        config,
    )
}

#[tokio::test]
async fn test_pipeline_initialization() {
    let pipeline = create_test_pipeline().await;
    let metrics = pipeline.get_metrics();
    assert_eq!(metrics.total_queries, 0);
    assert_eq!(metrics.total_memories_stored, 0);
}

#[tokio::test]
async fn test_add_memory() -> Result<()> {
    let mut pipeline = create_test_pipeline().await;
    
    let id = pipeline.add_memory("Test memory".to_string()).await?;
    assert!(!id.is_empty());
    
    let metrics = pipeline.get_metrics();
    assert_eq!(metrics.total_memories_stored, 1);
    
    Ok(())
}

#[tokio::test]
async fn test_retrieve_memory() -> Result<()> {
    let mut pipeline = create_test_pipeline().await;
    
    // Add a memory
    pipeline.add_memory("The dragon breathes fire".to_string()).await?;
    
    // Retrieve it
    let results = pipeline.retrieve("dragon", 1).await?;
    assert!(!results.is_empty());
    assert!(results[0].memory.text.contains("dragon"));
    
    Ok(())
}

#[tokio::test]
async fn test_retrieve_with_category_filter() -> Result<()> {
    let mut pipeline = create_test_pipeline().await;
    
    // Add memories with different categories
    // We need to use add_memory_obj to set categories manually since add_memory defaults to Gameplay
    
    let combat_memory = Memory {
        id: "combat1".to_string(),
        text: "Fought a goblin".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.8,
        valence: -0.5,
        category: MemoryCategory::Combat,
        entities: vec!["goblin".to_string()],
        context: HashMap::new(),
    };
    
    let social_memory = Memory {
        id: "social1".to_string(),
        text: "Talked to the merchant".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.5,
        valence: 0.5,
        category: MemoryCategory::Social,
        entities: vec!["merchant".to_string()],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(combat_memory).await?;
    pipeline.add_memory_obj(social_memory).await?;
    
    // Query with Combat category
    let query = astraweave_rag::MemoryQuery::text("interaction")
        .with_category(MemoryCategory::Combat);
        
    let results = pipeline.retrieve_with_query(&query, 5).await?;
    
    // Should only find the combat memory (even if text match is poor, filter is hard)
    // Note: MockEmbeddingClient might return random embeddings, so similarity is unreliable,
    // but the filter logic in RagPipeline::passes_filters is deterministic.
    // However, retrieve_with_query gets results from vector_store.search first.
    // If vector_store returns both, passes_filters will filter out Social.
    
    for result in &results {
        assert_eq!(result.memory.category, MemoryCategory::Combat);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_context_injection() -> Result<()> {
    let mut config = RagConfig::default();
    // Ensure prompt is included in template
    config.injection.injection_template = "Memories: {memories}\nPrompt: {prompt}\nQuery: {query}".to_string();
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    let llm_client = Arc::new(MockLlm);
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        Some(llm_client),
        config,
    );
    
    pipeline.add_memory("The secret code is 1234".to_string()).await?;
    
    let base_prompt = "What is the code?";
    let enhanced = pipeline.inject_context(base_prompt, "secret code").await?;
    
    assert!(enhanced.contains("1234"));
    assert!(enhanced.contains(base_prompt));
    
    Ok(())
}

#[tokio::test]
async fn test_consolidation_trigger() -> Result<()> {
    let mut config = RagConfig::default();
    config.consolidation.enabled = true;
    config.consolidation.trigger_threshold = 3; // Trigger after 3 memories
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    let llm_client = Arc::new(MockLlm);
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        Some(llm_client),
        config,
    );
    
    pipeline.add_memory("Mem 1".to_string()).await?;
    let metrics_1 = pipeline.get_metrics();
    assert_eq!(metrics_1.consolidations_performed, 0, "Should not consolidate after 1");
    
    pipeline.add_memory("Mem 2".to_string()).await?;
    let metrics_2 = pipeline.get_metrics();
    assert_eq!(metrics_2.consolidations_performed, 0, "Should not consolidate after 2");
    
    pipeline.add_memory("Mem 3".to_string()).await?;
    // Should trigger consolidation now
    
    let metrics_3 = pipeline.get_metrics();
    assert_eq!(metrics_3.consolidations_performed, 1, "Should consolidate after 3");
    
    Ok(())
}

#[tokio::test]
async fn test_retrieve_with_entity_filter() -> Result<()> {
    let mut pipeline = create_test_pipeline().await;
    
    let mem1 = Memory {
        id: "1".to_string(),
        text: "Found a sword".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec!["sword".to_string()],
        context: HashMap::new(),
    };
    
    let mem2 = Memory {
        id: "2".to_string(),
        text: "Found a shield".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec!["shield".to_string()],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(mem1).await?;
    pipeline.add_memory_obj(mem2).await?;
    
    let query = astraweave_rag::MemoryQuery::text("found")
        .with_entity("sword");
        
    let results = pipeline.retrieve_with_query(&query, 5).await?;
    
    // Debug print
    for r in &results {
        println!("Retrieved: {} Entities: {:?}", r.memory.text, r.memory.entities);
    }

    assert!(!results.is_empty(), "Should find at least one memory");
    for result in &results {
        assert!(result.memory.entities.contains(&"sword".to_string()), "Memory {} should contain sword", result.memory.id);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_async_add_memory() -> Result<()> {
    let pipeline = Arc::new(create_test_pipeline().await);
    
    let p1 = pipeline.clone();
    let h1 = tokio::spawn(async move {
        p1.add_memory_async("Async memory 1".to_string()).await
    });
    
    let p2 = pipeline.clone();
    let h2 = tokio::spawn(async move {
        p2.add_memory_async("Async memory 2".to_string()).await
    });
    
    let (r1, r2) = tokio::join!(h1, h2);
    r1??;
    r2??;
    
    let metrics = pipeline.get_metrics();
    assert_eq!(metrics.total_memories_stored, 2);
    
    Ok(())
}

#[tokio::test]
async fn test_retrieve_texts_helper() -> Result<()> {
    let mut pipeline = create_test_pipeline().await;
    pipeline.add_memory("Text content".to_string()).await?;
    
    let texts = pipeline.retrieve_texts("content", 1).await?;
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0], "Text content");
    
    Ok(())
}

#[tokio::test]
async fn test_clear_cache() -> Result<()> {
    let mut pipeline = create_test_pipeline().await;
    pipeline.add_memory("Memory".to_string()).await?;
    
    // Populate cache
    let _ = pipeline.retrieve("Memory", 1).await?;
    
    // Clear cache
    pipeline.clear_cache();
    
    // Retrieve again - should be a miss (but we can't easily check internal cache state directly without exposing it)
    // However, we can check that it doesn't panic
    let _ = pipeline.retrieve("Memory", 1).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_ordering_strategy() -> Result<()> {
    let mut config = RagConfig::default();
    config.injection.ordering_strategy = astraweave_rag::OrderingStrategy::ImportanceDesc;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    let llm_client = Arc::new(MockLlm);
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        Some(llm_client),
        config,
    );
    
    let low_imp = Memory {
        id: "low".to_string(),
        text: "Low importance".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.1,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let high_imp = Memory {
        id: "high".to_string(),
        text: "High importance".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.9,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(low_imp).await?;
    pipeline.add_memory_obj(high_imp).await?;
    
    let results = pipeline.retrieve("importance", 2).await?;
    
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].memory.id, "high"); // Should be first due to ImportanceDesc
    assert_eq!(results[1].memory.id, "low");
    
    Ok(())
}

#[tokio::test]
async fn test_summarization_trigger() -> Result<()> {
    let mut config = RagConfig::default();
    config.injection.max_context_tokens = 10; // Very low limit to force summarization
    config.injection.enable_summarization = true;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    let llm_client = Arc::new(MockLlm);
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        Some(llm_client),
        config,
    );
    
    // Add a long memory
    pipeline.add_memory("This is a very long memory that should definitely exceed the ten token limit set in the configuration.".to_string()).await?;
    
    let result = pipeline.inject_context_detailed("Prompt", "memory").await?;
    
    assert!(result.metadata.summarized);
    
    Ok(())
}

#[tokio::test]
async fn test_no_llm_client() -> Result<()> {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    let config = RagConfig::default();
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None, // No LLM
        config,
    );
    
    assert!(!pipeline.has_llm_client());
    
    // Should still work for basic retrieval
    let _ = pipeline.add_memory("Test".to_string()).await?;
    let results = pipeline.retrieve("Test", 1).await?;
    assert!(!results.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_empty_vector_store_retrieval() -> Result<()> {
    let pipeline = create_test_pipeline().await;
    
    // Retrieve from empty store
    let results = pipeline.retrieve("anything", 5).await?;
    assert!(results.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_ordering_strategy_similarity_asc() -> Result<()> {
    let mut config = RagConfig::default();
    config.injection.ordering_strategy = astraweave_rag::OrderingStrategy::SimilarityAsc;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    let llm_client = Arc::new(MockLlm);
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        Some(llm_client),
        config,
    );
    
    pipeline.add_memory("High match".to_string()).await?;
    pipeline.add_memory("Low match".to_string()).await?;
    
    let results = pipeline.retrieve("High", 2).await?;
    
    // Should be sorted ascending by similarity (lowest first)
    if results.len() >= 2 {
        assert!(results[0].similarity_score <= results[1].similarity_score);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_ordering_strategy_recency_desc() -> Result<()> {
    let mut config = RagConfig::default();
    config.injection.ordering_strategy = astraweave_rag::OrderingStrategy::RecencyDesc;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    let old_memory = Memory {
        id: "old".to_string(),
        text: "Old memory".to_string(),
        timestamp: astraweave_rag::current_timestamp() - 1000,
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let new_memory = Memory {
        id: "new".to_string(),
        text: "New memory".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(old_memory).await?;
    pipeline.add_memory_obj(new_memory).await?;
    
    let results = pipeline.retrieve("memory", 2).await?;
    
    if results.len() >= 2 {
        assert!(results[0].memory.timestamp >= results[1].memory.timestamp);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_ordering_strategy_recency_asc() -> Result<()> {
    let mut config = RagConfig::default();
    config.injection.ordering_strategy = astraweave_rag::OrderingStrategy::RecencyAsc;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    let old_memory = Memory {
        id: "old".to_string(),
        text: "Old memory".to_string(),
        timestamp: astraweave_rag::current_timestamp() - 1000,
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let new_memory = Memory {
        id: "new".to_string(),
        text: "New memory".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(old_memory).await?;
    pipeline.add_memory_obj(new_memory).await?;
    
    let results = pipeline.retrieve("memory", 2).await?;
    
    if results.len() >= 2 {
        assert!(results[0].memory.timestamp <= results[1].memory.timestamp);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_ordering_strategy_importance_asc() -> Result<()> {
    let mut config = RagConfig::default();
    config.injection.ordering_strategy = astraweave_rag::OrderingStrategy::ImportanceAsc;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    let low_imp = Memory {
        id: "low".to_string(),
        text: "Low importance".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.1,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let high_imp = Memory {
        id: "high".to_string(),
        text: "High importance".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.9,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(low_imp).await?;
    pipeline.add_memory_obj(high_imp).await?;
    
    let results = pipeline.retrieve("importance", 2).await?;
    
    if results.len() >= 2 {
        assert!(results[0].memory.importance <= results[1].memory.importance);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_ordering_strategy_mixed() -> Result<()> {
    let mut config = RagConfig::default();
    config.injection.ordering_strategy = astraweave_rag::OrderingStrategy::Mixed;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    for i in 0..5 {
        pipeline.add_memory(format!("Memory {}", i)).await?;
    }
    
    let results = pipeline.retrieve("Memory", 5).await?;
    
    // With Mixed strategy, order is shuffled (non-deterministic)
    // Just verify we got results
    assert_eq!(results.len(), 5);
    
    Ok(())
}

#[tokio::test]
async fn test_diversity_strategy_semantic() -> Result<()> {
    let mut config = RagConfig::default();
    config.diversity.enabled = true;
    config.diversity.strategy = astraweave_rag::DiversityStrategy::Semantic;
    config.diversity.diversity_factor = 0.5;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    pipeline.add_memory("The cat is sleeping peacefully".to_string()).await?;
    pipeline.add_memory("The cat is resting quietly".to_string()).await?; // Very similar
    pipeline.add_memory("The dragon breathes fire".to_string()).await?; // Different
    
    let results = pipeline.retrieve("cat", 3).await?;
    
    // With diversity enabled, should get diverse results
    assert!(!results.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_diversity_strategy_temporal() -> Result<()> {
    let mut config = RagConfig::default();
    config.diversity.enabled = true;
    config.diversity.strategy = astraweave_rag::DiversityStrategy::Temporal;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    pipeline.add_memory("Event 1".to_string()).await?;
    pipeline.add_memory("Event 2".to_string()).await?;
    pipeline.add_memory("Event 3".to_string()).await?;
    
    let results = pipeline.retrieve("Event", 3).await?;
    
    assert_eq!(results.len(), 3);
    
    Ok(())
}

#[tokio::test]
async fn test_diversity_strategy_category() -> Result<()> {
    let mut config = RagConfig::default();
    config.diversity.enabled = true;
    config.diversity.strategy = astraweave_rag::DiversityStrategy::Category;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    let mem1 = Memory {
        id: "1".to_string(),
        text: "Combat event".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Combat,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let mem2 = Memory {
        id: "2".to_string(),
        text: "Social event".to_string(),
        timestamp: astraweave_rag::current_timestamp(),
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Social,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(mem1).await?;
    pipeline.add_memory_obj(mem2).await?;
    
    let results = pipeline.retrieve("event", 2).await?;
    
    assert_eq!(results.len(), 2);
    
    Ok(())
}

#[tokio::test]
async fn test_diversity_strategy_combined() -> Result<()> {
    let mut config = RagConfig::default();
    config.diversity.enabled = true;
    config.diversity.strategy = astraweave_rag::DiversityStrategy::Combined;
    
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
    
    let mut pipeline = RagPipeline::new(
        embedding_client,
        vector_store,
        None,
        config,
    );
    
    for i in 0..5 {
        pipeline.add_memory(format!("Memory {}", i)).await?;
    }
    
    let results = pipeline.retrieve("Memory", 5).await?;
    
    assert_eq!(results.len(), 5);
    
    Ok(())
}
