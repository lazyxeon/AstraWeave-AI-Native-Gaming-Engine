use astraweave_embeddings::{Memory, MemoryCategory, MockEmbeddingClient, SearchResult, StoredVector};
use astraweave_llm::MockLlm;
use astraweave_rag::{
    RagConfig, RagPipeline, MemoryQuery, InjectionStrategy,
    OrderingStrategy, VectorStoreInterface,
};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use anyhow::Result;

// Mock VectorStore for testing pipeline logic independently of actual VectorStore implementation
struct MockVectorStore {
    memories: Arc<RwLock<HashMap<String, (Memory, Vec<f32>)>>>,
}

impl MockVectorStore {
    fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl VectorStoreInterface for MockVectorStore {
    async fn search(&self, _query_vector: &[f32], _k: usize) -> Result<Vec<SearchResult>> {
        let memories = self.memories.read();
        let mut results = Vec::new();
        
        for (_, (memory, _)) in memories.iter() {
            // Convert Memory back to SearchResult format (simulating retrieval)
            let mut metadata = HashMap::new();
            metadata.insert("entities".to_string(), serde_json::to_string(&memory.entities)?);
            metadata.insert("category".to_string(), serde_json::to_string(&memory.category)?);
            metadata.insert("valence".to_string(), memory.valence.to_string());
            
            let vector = StoredVector {
                id: memory.id.clone(),
                vector: vec![0.0; 384], // Dummy vector
                text: memory.text.clone(),
                timestamp: memory.timestamp,
                importance: memory.importance,
                metadata,
            };
            
            results.push(SearchResult {
                vector,
                score: 0.9, // Dummy score
                distance: 0.1, // Dummy distance
            });
        }
        
        Ok(results)
    }

    async fn insert(&self, _id: String, _vector: Vec<f32>, _text: String) -> Result<()> {
        Ok(())
    }

    async fn insert_memory(&self, memory: Memory, vector: Vec<f32>) -> Result<()> {
        self.memories.write().insert(memory.id.clone(), (memory, vector));
        Ok(())
    }

    async fn get(&self, id: &str) -> Option<Memory> {
        self.memories.read().get(id).map(|(m, _)| m.clone())
    }

    async fn remove(&self, id: &str) -> Option<Memory> {
        self.memories.write().remove(id).map(|(m, _)| m)
    }

    fn len(&self) -> usize {
        self.memories.read().len()
    }

    async fn get_all_memories(&self) -> Vec<Memory> {
        self.memories.read().values().map(|(m, _)| m.clone()).collect()
    }
}

fn create_test_pipeline() -> RagPipeline {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let llm_client = Arc::new(MockLlm);
    let config = RagConfig::default();

    RagPipeline::new(embedding_client, vector_store, Some(llm_client), config)
}

#[tokio::test]
async fn test_pipeline_initialization() {
    let pipeline = create_test_pipeline();
    let metrics = pipeline.get_metrics();
    
    assert_eq!(metrics.total_queries, 0);
    assert_eq!(metrics.total_memories_stored, 0);
    assert!(pipeline.has_llm_client());
}

#[tokio::test]
async fn test_add_memory() {
    let mut pipeline = create_test_pipeline();
    
    let id = pipeline.add_memory("Test memory".to_string()).await.unwrap();
    assert!(!id.is_empty());
    
    let metrics = pipeline.get_metrics();
    assert_eq!(metrics.total_memories_stored, 1);
}

#[tokio::test]
async fn test_add_memory_async() {
    let pipeline = Arc::new(create_test_pipeline());
    
    let id = pipeline.add_memory_async("Async memory".to_string()).await.unwrap();
    assert!(!id.is_empty());
    
    let metrics = pipeline.get_metrics();
    assert_eq!(metrics.total_memories_stored, 1);
}

#[tokio::test]
async fn test_retrieve_basic() {
    let mut pipeline = create_test_pipeline();
    
    pipeline.add_memory("The dragon breathes fire".to_string()).await.unwrap();
    pipeline.add_memory("The knight has a sword".to_string()).await.unwrap();
    
    let results = pipeline.retrieve("dragon", 1).await.unwrap();
    
    assert_eq!(results.len(), 1);
    // Note: MockEmbeddingClient might return random or constant embeddings, 
    // so we can't strictly assert semantic relevance without a real model,
    // but we can check structure.
    assert!(!results[0].memory.text.is_empty());
}

#[tokio::test]
async fn test_retrieve_with_filters() {
    let mut pipeline = create_test_pipeline();
    
    // Add memories with specific categories manually to control metadata
    let memory_combat = Memory {
        id: "1".to_string(),
        text: "Combat memory".to_string(),
        timestamp: 1000,
        importance: 0.8,
        valence: 0.0,
        category: MemoryCategory::Combat,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let memory_social = Memory {
        id: "2".to_string(),
        text: "Social memory".to_string(),
        timestamp: 1000,
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Social,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(memory_combat).await.unwrap();
    pipeline.add_memory_obj(memory_social).await.unwrap();
    
    let query = MemoryQuery::text("memory")
        .with_category(MemoryCategory::Combat);
        
    let results = pipeline.retrieve_with_query(&query, 10).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].memory.category, MemoryCategory::Combat);
}

#[tokio::test]
async fn test_retrieve_texts() {
    let mut pipeline = create_test_pipeline();
    pipeline.add_memory("Memory 1".to_string()).await.unwrap();
    
    let texts = pipeline.retrieve_texts("Memory", 1).await.unwrap();
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0], "Memory 1");
}

#[tokio::test]
async fn test_inject_context() {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let llm_client = Arc::new(MockLlm);
    let mut config = RagConfig::default();
    config.injection.injection_template = "{memories}\nPrompt: {prompt}\nQuery: {query}".to_string();

    let mut pipeline = RagPipeline::new(embedding_client, vector_store, Some(llm_client), config);
    pipeline.add_memory("Secret code is 1234".to_string()).await.unwrap();
    
    let enhanced = pipeline.inject_context("What is the code?", "code").await.unwrap();
    
    assert!(enhanced.contains("Secret code is 1234"));
    assert!(enhanced.contains("What is the code?"));
}

#[tokio::test]
async fn test_inject_context_detailed() {
    let mut pipeline = create_test_pipeline();
    pipeline.add_memory("Detailed memory".to_string()).await.unwrap();
    
    let result = pipeline.inject_context_detailed("Prompt", "query").await.unwrap();
    
    assert!(!result.injected_memories.is_empty());
    assert!(result.context_tokens > 0);
    assert_eq!(result.metadata.strategy, InjectionStrategy::Insert);
}

#[tokio::test]
async fn test_caching() {
    let mut pipeline = create_test_pipeline();
    pipeline.add_memory("Cache test".to_string()).await.unwrap();
    
    // First retrieval - miss
    let _ = pipeline.retrieve("test", 1).await.unwrap();
    let _metrics_1 = pipeline.get_metrics();
    
    // Second retrieval - hit
    let _ = pipeline.retrieve("test", 1).await.unwrap();
    let metrics_2 = pipeline.get_metrics();
    
    assert!(metrics_2.cache_hit_rate > 0.0);
}

#[tokio::test]
async fn test_clear_cache() {
    let mut pipeline = create_test_pipeline();
    pipeline.add_memory("Cache clear test".to_string()).await.unwrap();
    
    let _ = pipeline.retrieve("test", 1).await.unwrap();
    pipeline.clear_cache();
    
    // Should be a miss again (conceptually, though metrics might accumulate)
    // We can't easily check internal cache state without exposing it, 
    // but we can check that it doesn't crash.
}

#[tokio::test]
async fn test_consolidation_trigger() {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let llm_client = Arc::new(MockLlm);
    
    let mut config = RagConfig::default();
    config.consolidation.trigger_threshold = 2; // Low threshold for testing
    
    let mut pipeline = RagPipeline::new(embedding_client, vector_store, Some(llm_client), config);
    
    pipeline.add_memory("Mem 1".to_string()).await.unwrap();
    pipeline.add_memory("Mem 2".to_string()).await.unwrap();
    
    // Third memory should trigger consolidation check
    pipeline.add_memory("Mem 3".to_string()).await.unwrap();
    
    let metrics = pipeline.get_metrics();
    assert!(metrics.consolidations_performed > 0);
}

#[tokio::test]
async fn test_diversity_application() {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let llm_client = Arc::new(MockLlm);
    
    let mut config = RagConfig::default();
    config.diversity.enabled = true;
    config.diversity.diversity_factor = 0.5;
    
    let mut pipeline = RagPipeline::new(embedding_client, vector_store, Some(llm_client), config);
    
    pipeline.add_memory("Apple".to_string()).await.unwrap();
    pipeline.add_memory("Banana".to_string()).await.unwrap();
    pipeline.add_memory("Car".to_string()).await.unwrap(); // Semantically different
    
    let results = pipeline.retrieve("fruit", 3).await.unwrap();
    
    // With diversity, we expect reordering or specific selection, 
    // but with MockEmbeddingClient it's hard to guarantee semantic distance.
    // We mainly check that it runs without error and returns results.
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_ordering_strategy() {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let llm_client = Arc::new(MockLlm);
    
    let mut config = RagConfig::default();
    config.injection.ordering_strategy = OrderingStrategy::ImportanceDesc;
    
    let mut pipeline = RagPipeline::new(embedding_client, vector_store, Some(llm_client), config);
    
    let mem1 = Memory {
        id: "1".to_string(),
        text: "Important".to_string(),
        timestamp: 1000,
        importance: 0.9,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let mem2 = Memory {
        id: "2".to_string(),
        text: "Unimportant".to_string(),
        timestamp: 1000,
        importance: 0.1,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(mem1).await.unwrap();
    pipeline.add_memory_obj(mem2).await.unwrap();
    
    let results = pipeline.retrieve("test", 2).await.unwrap();
    
    assert_eq!(results.len(), 2);
    assert!(results[0].memory.importance > results[1].memory.importance);
}

#[tokio::test]
async fn test_memory_filtering_time_range() {
    let mut pipeline = create_test_pipeline();
    
    let old_mem = Memory {
        id: "1".to_string(),
        text: "Old".to_string(),
        timestamp: 100,
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    let new_mem = Memory {
        id: "2".to_string(),
        text: "New".to_string(),
        timestamp: 1000,
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(old_mem).await.unwrap();
    pipeline.add_memory_obj(new_mem).await.unwrap();
    
    let query = MemoryQuery::text("test").with_time_range(500, 1500);
    let results = pipeline.retrieve_with_query(&query, 10).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].memory.text, "New");
}

#[tokio::test]
async fn test_memory_filtering_entities() {
    let mut pipeline = create_test_pipeline();
    
    let mem_with_entity = Memory {
        id: "1".to_string(),
        text: "Meeting with King".to_string(),
        timestamp: 1000,
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Social,
        entities: vec!["King".to_string()],
        context: HashMap::new(),
    };
    
    let mem_without = Memory {
        id: "2".to_string(),
        text: "Strolling alone".to_string(),
        timestamp: 1000,
        importance: 0.5,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(mem_with_entity).await.unwrap();
    pipeline.add_memory_obj(mem_without).await.unwrap();
    
    let query = MemoryQuery::text("test").with_entity("King");
    let results = pipeline.retrieve_with_query(&query, 10).await.unwrap();
    
    for r in &results {
        println!("Retrieved: ID={}, Text='{}', Entities={:?}", r.memory.id, r.memory.text, r.memory.entities);
    }

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].memory.entities[0], "King");
}

#[tokio::test]
async fn test_summarization_trigger() {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let llm_client = Arc::new(MockLlm);
    
    let mut config = RagConfig::default();
    config.injection.max_context_tokens = 10; // Very low to trigger summarization
    config.injection.enable_summarization = true;
    
    let mut pipeline = RagPipeline::new(embedding_client, vector_store, Some(llm_client), config);
    
    pipeline.add_memory("A very long memory that will definitely exceed the token limit of ten tokens".to_string()).await.unwrap();
    
    let result = pipeline.inject_context_detailed("Prompt", "query").await.unwrap();
    
    assert!(result.metadata.summarized);
}

#[tokio::test]
async fn test_no_llm_client_behavior() {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let config = RagConfig::default();
    
    let mut pipeline = RagPipeline::new(embedding_client, vector_store, None, config);
    
    assert!(!pipeline.has_llm_client());
    
    // Should still work for basic retrieval
    pipeline.add_memory("Test".to_string()).await.unwrap();
    let results = pipeline.retrieve("Test", 1).await.unwrap();
    assert!(!results.is_empty());
    
    // Summarization should fallback to truncation
    // (We can't easily test internal fallback path without mocking TokenCounter or checking logs, 
    // but we can ensure it doesn't panic)
    let _ = pipeline.inject_context("Prompt", "Test").await;
}

#[tokio::test]
async fn test_forgetting_trigger() {
    let embedding_client = Arc::new(MockEmbeddingClient::new());
    let vector_store = Arc::new(MockVectorStore::new());
    let llm_client = Arc::new(MockLlm);
    
    let mut config = RagConfig::default();
    config.forgetting.enabled = true;
    config.forgetting.min_importance_threshold = 0.5;
    
    let mut pipeline = RagPipeline::new(embedding_client, vector_store, Some(llm_client), config);
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
        
    let old_timestamp = now - 360000; // 100 hours ago

    // Add important memory (recent)
    let important_mem = Memory {
        id: "1".to_string(),
        text: "Important".to_string(),
        timestamp: now,
        importance: 0.9,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    // Add unimportant memory (old)
    let unimportant_mem = Memory {
        id: "2".to_string(),
        text: "Unimportant".to_string(),
        timestamp: old_timestamp,
        importance: 0.1,
        valence: 0.0,
        category: MemoryCategory::Gameplay,
        entities: vec![],
        context: HashMap::new(),
    };
    
    pipeline.add_memory_obj(important_mem).await.unwrap();
    pipeline.add_memory_obj(unimportant_mem).await.unwrap();
    
    // Trigger forgetting
    pipeline.trigger_forgetting().await.unwrap();
    
    let metrics = pipeline.get_metrics();
    assert!(metrics.memories_forgotten > 0);
    
    // Verify unimportant memory is gone
    let memories = pipeline.retrieve("Unimportant", 10).await.unwrap();
    let found_unimportant = memories.iter().any(|m| m.memory.id == "2");
    assert!(!found_unimportant, "Unimportant memory should have been forgotten");
    
    // Verify important memory is still there
    let found_important = pipeline.retrieve("Important", 10).await.unwrap().iter().any(|m| m.memory.id == "1");
    assert!(found_important, "Important memory should be retained");
}
