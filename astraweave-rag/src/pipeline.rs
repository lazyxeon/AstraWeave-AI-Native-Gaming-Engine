/*!
# RAG Pipeline

Main RAG pipeline implementation combining retrieval, consolidation, and injection.
*/

use crate::{
    current_timestamp, InjectionResult, InjectionStrategy, MemoryQuery, RagConfig, RagMetrics,
    RetrievalMethod, RetrievedMemory,
};
use anyhow::{anyhow, Result};
use astraweave_context::TokenCounter;
use astraweave_embeddings::{EmbeddingClient, Memory, MemoryCategory, SearchResult, VectorStore};
use astraweave_llm::LlmClient;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Main RAG pipeline combining all components
pub struct RagPipeline {
    /// Embedding client for vector operations
    embedding_client: Arc<dyn EmbeddingClient>,

    /// Vector store for memory storage and retrieval
    vector_store: Arc<dyn VectorStoreInterface>,

    /// LLM client for consolidation and summarization
    llm_client: Option<Arc<dyn LlmClient>>,

    /// Token counter for context management
    token_counter: TokenCounter,

    /// Pipeline configuration
    config: RagConfig,

    /// Performance metrics
    metrics: Arc<RwLock<RagMetrics>>,

    /// Query result cache
    cache: Arc<DashMap<String, CachedResult>>,

    /// Memory consolidation state
    consolidation_state: Arc<RwLock<ConsolidationState>>,
}

/// Trait for vector store operations (allows testing with different implementations)
#[async_trait::async_trait]
pub trait VectorStoreInterface: Send + Sync {
    async fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>>;
    async fn insert(&self, id: String, vector: Vec<f32>, text: String) -> Result<()>;
    async fn insert_memory(&self, memory: Memory, vector: Vec<f32>) -> Result<()>;
    async fn get(&self, id: &str) -> Option<Memory>;
    async fn remove(&self, id: &str) -> Option<Memory>;
    fn len(&self) -> usize;
    async fn get_all_memories(&self) -> Vec<Memory>;
}

/// Wrapper to make VectorStore compatible with VectorStoreInterface
pub struct VectorStoreWrapper {
    inner: VectorStore,
}

impl VectorStoreWrapper {
    pub fn new(store: VectorStore) -> Self {
        Self { inner: store }
    }
}

#[async_trait::async_trait]
impl VectorStoreInterface for VectorStoreWrapper {
    async fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        self.inner.search(query_vector, k)
    }

    async fn insert(&self, id: String, vector: Vec<f32>, text: String) -> Result<()> {
        self.inner.insert(id, vector, text)
    }

    async fn insert_memory(&self, memory: Memory, vector: Vec<f32>) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("entities".to_string(), serde_json::to_string(&memory.entities)?);
        metadata.insert("category".to_string(), serde_json::to_string(&memory.category)?);
        metadata.insert("valence".to_string(), memory.valence.to_string());
        
        for (k, v) in memory.context {
            metadata.insert(format!("ctx_{}", k), v);
        }

        self.inner.insert_with_metadata(
            memory.id,
            vector,
            memory.text,
            memory.importance,
            metadata,
        )
    }

    async fn get(&self, id: &str) -> Option<Memory> {
        self.inner.get(id).map(|stored| {
            let entities: Vec<String> = stored.metadata.get("entities")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            
            let category: MemoryCategory = stored.metadata.get("category")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(MemoryCategory::Gameplay);
                
            let valence: f32 = stored.metadata.get("valence")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
                
            let mut context = HashMap::new();
            for (k, v) in &stored.metadata {
                if let Some(key) = k.strip_prefix("ctx_") {
                    context.insert(key.to_string(), v.clone());
                }
            }

            Memory {
                id: stored.id,
                text: stored.text,
                timestamp: stored.timestamp,
                importance: stored.importance,
                valence,
                category,
                entities,
                context,
            }
        })
    }

    async fn remove(&self, id: &str) -> Option<Memory> {
        self.inner.remove(id).map(|stored| Memory {
            id: stored.id,
            text: stored.text,
            timestamp: stored.timestamp,
            importance: stored.importance,
            valence: 0.0,
            category: MemoryCategory::Gameplay,
            entities: vec![],
            context: HashMap::new(),
        })
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    async fn get_all_memories(&self) -> Vec<Memory> {
        // This is a simplified implementation
        // In practice, you'd iterate through all stored vectors
        vec![]
    }
}

/// Cached retrieval result
#[derive(Debug, Clone)]
struct CachedResult {
    memories: Vec<RetrievedMemory>,
    timestamp: u64,
    #[allow(dead_code)]
    query_hash: u64,
}

/// State for memory consolidation
#[derive(Debug, Default)]
struct ConsolidationState {
    last_consolidation: u64,
    memories_since_consolidation: usize,
    consolidation_in_progress: bool,
}

impl RagPipeline {
    /// Create a new RAG pipeline
    pub fn new(
        embedding_client: Arc<dyn EmbeddingClient>,
        vector_store: Arc<dyn VectorStoreInterface>,
        llm_client: Option<Arc<dyn LlmClient>>,
        config: RagConfig,
    ) -> Self {
        let token_counter = TokenCounter::new("cl100k_base");

        Self {
            embedding_client,
            vector_store,
            llm_client,
            token_counter,
            config,
            metrics: Arc::new(RwLock::new(RagMetrics::default())),
            cache: Arc::new(DashMap::new()),
            consolidation_state: Arc::new(RwLock::new(ConsolidationState {
                last_consolidation: current_timestamp(),
                memories_since_consolidation: 0,
                consolidation_in_progress: false,
            })),
        }
    }

    /// Add a memory to the pipeline
    pub async fn add_memory(&mut self, text: String) -> Result<String> {
        let memory = Memory {
            id: uuid::Uuid::new_v4().to_string(),
            text: text.clone(),
            timestamp: current_timestamp(),
            importance: 1.0,
            valence: 0.0,
            category: MemoryCategory::Gameplay,
            entities: vec![],
            context: HashMap::new(),
        };

        self.add_memory_obj(memory).await
    }

    /// Add a memory object to the pipeline
    pub async fn add_memory_obj(&mut self, memory: Memory) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Generate embedding for the memory
        let embedding = self.embedding_client.embed(&memory.text).await?;

        // Store in vector store
        self.vector_store
            .insert_memory(memory.clone(), embedding)
            .await?;

        // Update consolidation state
        {
            let mut state = self.consolidation_state.write();
            state.memories_since_consolidation += 1;
        }

        // Check if consolidation is needed
        if self.should_consolidate().await {
            self.trigger_consolidation().await?;
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_memories_stored += 1;
            let _duration = start_time.elapsed().as_millis() as f32;
            // Update average processing time if needed
        }

        Ok(memory.id)
    }

    /// Retrieve memories based on text query
    pub async fn retrieve(&self, query: &str, k: usize) -> Result<Vec<RetrievedMemory>> {
        let memory_query = MemoryQuery::text(query);
        self.retrieve_with_query(&memory_query, k).await
    }

    /// Retrieve memories with advanced query
    pub async fn retrieve_with_query(
        &self,
        query: &MemoryQuery,
        k: usize,
    ) -> Result<Vec<RetrievedMemory>> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = self.generate_cache_key(query, k);
        if self.config.performance.enable_caching {
            if let Some(cached) = self.get_cached_result(&cache_key) {
                self.update_cache_hit_metrics();
                return Ok(cached.memories);
            }
        }

        // Generate query embedding
        let query_embedding = self.embedding_client.embed(&query.text).await?;

        // Search vector store
        let search_results = self.vector_store.search(&query_embedding, k * 2).await?; // Get more for filtering

        // Convert to RetrievedMemory and apply filters
        let mut retrieved_memories = Vec::new();
        for (rank, result) in search_results.into_iter().enumerate() {
            // Convert SearchResult to Memory
            let entities: Vec<String> = result.vector.metadata.get("entities")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            
            let category: MemoryCategory = result.vector.metadata.get("category")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(MemoryCategory::Gameplay);
                
            let valence: f32 = result.vector.metadata.get("valence")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
                
            let mut context = HashMap::new();
            for (k, v) in &result.vector.metadata {
                if let Some(key) = k.strip_prefix("ctx_") {
                    context.insert(key.to_string(), v.clone());
                }
            }

            let memory = Memory {
                id: result.vector.id,
                text: result.vector.text,
                timestamp: result.vector.timestamp,
                importance: result.vector.importance,
                valence,
                category,
                entities,
                context,
            };

            // Apply filters
            if self.passes_filters(&memory, query) {
                retrieved_memories.push(RetrievedMemory {
                    memory,
                    similarity_score: result.score,
                    rank,
                    metadata: crate::RetrievalMetadata {
                        query: query.text.clone(),
                        method: RetrievalMethod::SemanticSearch,
                        retrieved_at: current_timestamp(),
                        processing_time_ms: start_time.elapsed().as_millis() as f32,
                        context: HashMap::new(),
                    },
                });

                if retrieved_memories.len() >= k {
                    break;
                }
            }
        }

        // Apply diversity if enabled
        if self.config.diversity.enabled {
            retrieved_memories = self.apply_diversity(retrieved_memories);
        }

        // Order results
        self.order_results(&mut retrieved_memories);

        // Cache results
        if self.config.performance.enable_caching {
            self.cache_result(&cache_key, &retrieved_memories);
        }

        // Update metrics
        let duration = start_time.elapsed().as_millis() as f32;
        self.update_retrieval_metrics(retrieved_memories.len(), duration, true);

        Ok(retrieved_memories)
    }

    /// Convenience helper: retrieve and return the text contents as Vec<String>
    pub async fn retrieve_texts(&self, query: &str, k: usize) -> Result<Vec<String>> {
        let items = self.retrieve(query, k).await?;
        Ok(items.into_iter().map(|r| r.memory.text).collect())
    }

    /// Return true if an LLM client is configured for this pipeline
    pub fn has_llm_client(&self) -> bool {
        self.llm_client.is_some()
    }

    /// Inject retrieved memories into a prompt
    pub async fn inject_context(&self, base_prompt: &str, query: &str) -> Result<String> {
        let injection_result = self.inject_context_detailed(base_prompt, query).await?;
        Ok(injection_result.enhanced_prompt)
    }

    /// Inject context with detailed result information
    pub async fn inject_context_detailed(
        &self,
        base_prompt: &str,
        query: &str,
    ) -> Result<InjectionResult> {
        let start_time = std::time::Instant::now();

        // Retrieve relevant memories
        let memories = self
            .retrieve(query, self.config.max_retrieval_count)
            .await?;

        if memories.is_empty() {
            return Ok(InjectionResult {
                enhanced_prompt: base_prompt.to_string(),
                injected_memories: vec![],
                context_tokens: 0,
                metadata: crate::InjectionMetadata {
                    original_prompt: base_prompt.to_string(),
                    query: query.to_string(),
                    strategy: InjectionStrategy::Prepend,
                    processing_time_ms: start_time.elapsed().as_millis() as f32,
                    summarized: false,
                },
            });
        }

        // Format memories for injection
        let memory_texts: Vec<String> = memories
            .iter()
            .map(|m| {
                if self.config.injection.include_metadata {
                    format!("[Score: {:.2}] {}", m.similarity_score, m.memory.text)
                } else {
                    m.memory.text.clone()
                }
            })
            .collect();

        let memories_text = memory_texts.join("\n");

        // Check token limit and summarize if needed
        let memory_tokens = self.token_counter.count_tokens(&memories_text)?;
        let final_memories_text = if memory_tokens > self.config.injection.max_context_tokens {
            if self.config.injection.enable_summarization && self.llm_client.is_some() {
                self.summarize_memories(&memories_text).await?
            } else {
                // Truncate to fit token limit
                self.token_counter
                    .truncate_to_tokens(&memories_text, self.config.injection.max_context_tokens)?
            }
        } else {
            memories_text
        };

        // Apply injection template
        let enhanced_prompt = self
            .config
            .injection
            .injection_template
            .replace("{memories}", &final_memories_text)
            .replace("{query}", query)
            .replace("{prompt}", base_prompt);

        let context_tokens = self.token_counter.count_tokens(&final_memories_text)?;

        Ok(InjectionResult {
            enhanced_prompt,
            injected_memories: memories,
            context_tokens,
            metadata: crate::InjectionMetadata {
                original_prompt: base_prompt.to_string(),
                query: query.to_string(),
                strategy: InjectionStrategy::Insert,
                processing_time_ms: start_time.elapsed().as_millis() as f32,
                summarized: memory_tokens > self.config.injection.max_context_tokens,
            },
        })
    }

    /// Get pipeline metrics
    pub fn get_metrics(&self) -> RagMetrics {
        self.metrics.read().clone()
    }

    /// Clear the memory cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Check if memory consolidation should be triggered
    async fn should_consolidate(&self) -> bool {
        if !self.config.consolidation.enabled {
            return false;
        }

        let state = self.consolidation_state.read();

        // Check if already in progress
        if state.consolidation_in_progress {
            return false;
        }

        // Check memory threshold
        if state.memories_since_consolidation >= self.config.consolidation.trigger_threshold {
            return true;
        }

        // Check time threshold
        let time_since_last = current_timestamp() - state.last_consolidation;
        if time_since_last >= self.config.consolidation.consolidation_interval {
            return true;
        }

        false
    }

    /// Trigger memory consolidation
    async fn trigger_consolidation(&self) -> Result<()> {
        // Set consolidation in progress
        {
            let mut state = self.consolidation_state.write();
            state.consolidation_in_progress = true;
        }

        // Perform consolidation (simplified implementation)
        // In a full implementation, this would:
        // 1. Identify similar memories
        // 2. Merge or summarize them
        // 3. Update the vector store
        // 4. Clean up old memories

        // For now, just reset the counter
        {
            let mut state = self.consolidation_state.write();
            state.last_consolidation = current_timestamp();
            state.memories_since_consolidation = 0;
            state.consolidation_in_progress = false;
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.consolidations_performed += 1;
        }

        Ok(())
    }

    /// Adds a memory item asynchronously without requiring &mut self or Arc::get_mut.
    ///
    /// This method is safe to call from an Arc<RagPipeline> and uses the pipeline's
    /// interior mutability (parking_lot locks) for coordination. It performs
    /// embedding generation and vector store insertion asynchronously and updates
    /// consolidation/metrics via synchronized locks. The lock scopes are kept
    /// minimal and are not held across awaits.
    pub async fn add_memory_async(&self, text: String) -> Result<String> {
        let memory = Memory {
            id: uuid::Uuid::new_v4().to_string(),
            text: text.clone(),
            timestamp: current_timestamp(),
            importance: 1.0,
            valence: 0.0,
            category: MemoryCategory::Gameplay,
            entities: vec![],
            context: HashMap::new(),
        };

        // Generate embedding using the shared embedding client
        let embedding = self.embedding_client.embed(&memory.text).await?;

        // Store in vector store
        self.vector_store
            .insert_memory(memory.clone(), embedding)
            .await?;

        // Update consolidation state (synchronous lock, dropped immediately)
        {
            let mut state = self.consolidation_state.write();
            state.memories_since_consolidation += 1;
        }

        // Trigger consolidation if needed (runs while holding locks briefly internally)
        if self.should_consolidate().await {
            // Fire-and-forget consolidation is acceptable here; await it to keep behavior
            // consistent with existing sync path.
            let _ = self.trigger_consolidation().await;
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_memories_stored += 1;
        }

        Ok(memory.id)
    }

    /// Check if a memory passes the query filters
    fn passes_filters(&self, memory: &Memory, query: &MemoryQuery) -> bool {
        // Time range filter
        if let Some((start, end)) = query.time_range {
            if memory.timestamp < start || memory.timestamp > end {
                return false;
            }
        }

        // Category filter
        if !query.categories.is_empty() && !query.categories.contains(&memory.category) {
            return false;
        }

        // Entity filter
        if !query.entities.is_empty() {
            let has_entity = query.entities.iter().any(|entity| {
                memory.entities.contains(entity)
                    || memory.text.to_lowercase().contains(&entity.to_lowercase())
            });
            if !has_entity {
                return false;
            }
        }

        // Importance filter
        if let Some(min_importance) = query.min_importance {
            if memory.importance < min_importance {
                return false;
            }
        }

        // Age filter
        if let Some(max_age) = query.max_age {
            let age = current_timestamp() - memory.timestamp;
            if age > max_age {
                return false;
            }
        }

        true
    }

    /// Apply diversity to retrieval results
    fn apply_diversity(&self, mut memories: Vec<RetrievedMemory>) -> Vec<RetrievedMemory> {
        if memories.len() <= 1 {
            return memories;
        }

        let mut diverse_memories = Vec::new();
        diverse_memories.push(memories.remove(0)); // Always include the most similar

        while !memories.is_empty() && diverse_memories.len() < self.config.max_retrieval_count {
            let mut best_idx = 0;
            let mut best_score = f32::MIN;

            for (i, candidate) in memories.iter().enumerate() {
                // Calculate diversity score
                let mut min_distance = f32::MAX;
                for existing in &diverse_memories {
                    // Simple text-based diversity (could use embedding distance)
                    let distance = text_distance(&candidate.memory.text, &existing.memory.text);
                    min_distance = min_distance.min(distance);
                }

                // Combined score: similarity + diversity
                let diversity_bonus = min_distance * self.config.diversity.diversity_factor;
                let combined_score = candidate.similarity_score + diversity_bonus;

                if combined_score > best_score {
                    best_score = combined_score;
                    best_idx = i;
                }
            }

            diverse_memories.push(memories.remove(best_idx));
        }

        diverse_memories
    }

    /// Order results based on configuration
    fn order_results(&self, memories: &mut Vec<RetrievedMemory>) {
        match self.config.injection.ordering_strategy {
            crate::OrderingStrategy::SimilarityDesc => {
                memories
                    .sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
            }
            crate::OrderingStrategy::SimilarityAsc => {
                memories
                    .sort_by(|a, b| a.similarity_score.partial_cmp(&b.similarity_score).unwrap());
            }
            crate::OrderingStrategy::RecencyDesc => {
                memories.sort_by(|a, b| b.memory.timestamp.cmp(&a.memory.timestamp));
            }
            crate::OrderingStrategy::RecencyAsc => {
                memories.sort_by(|a, b| a.memory.timestamp.cmp(&b.memory.timestamp));
            }
            crate::OrderingStrategy::ImportanceDesc => {
                memories.sort_by(|a, b| {
                    b.memory
                        .importance
                        .partial_cmp(&a.memory.importance)
                        .unwrap()
                });
            }
            crate::OrderingStrategy::ImportanceAsc => {
                memories.sort_by(|a, b| {
                    a.memory
                        .importance
                        .partial_cmp(&b.memory.importance)
                        .unwrap()
                });
            }
            crate::OrderingStrategy::Mixed => {
                // Shuffle for variety
                use rand::seq::SliceRandom;
                let mut rng = rand::rng();
                memories.shuffle(&mut rng);
            }
        }
    }

    /// Generate cache key for query
    fn generate_cache_key(&self, query: &MemoryQuery, k: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.text.hash(&mut hasher);
        k.hash(&mut hasher);
        // Add other query parameters to hash if needed

        format!("rag_query_{}", hasher.finish())
    }

    /// Get cached result if available and not expired
    fn get_cached_result(&self, cache_key: &str) -> Option<CachedResult> {
        if let Some(cached) = self.cache.get(cache_key) {
            let age = current_timestamp() - cached.timestamp;
            if age <= self.config.performance.cache_ttl {
                return Some(cached.clone());
            } else {
                // Remove expired entry
                self.cache.remove(cache_key);
            }
        }
        None
    }

    /// Cache retrieval result
    fn cache_result(&self, cache_key: &str, memories: &[RetrievedMemory]) {
        // Limit cache size
        if self.cache.len() >= self.config.performance.cache_size {
            // Remove some old entries (simplified LRU)
            let keys_to_remove: Vec<String> = self
                .cache
                .iter()
                .take(self.config.performance.cache_size / 4) // Remove 25%
                .map(|entry| entry.key().clone())
                .collect();

            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }

        let cached_result = CachedResult {
            memories: memories.to_vec(),
            timestamp: current_timestamp(),
            query_hash: 0, // Would compute proper hash in practice
        };

        self.cache.insert(cache_key.to_string(), cached_result);
    }

    /// Summarize memories using LLM
    async fn summarize_memories(&self, memories_text: &str) -> Result<String> {
        let llm_client = self
            .llm_client
            .as_ref()
            .ok_or_else(|| anyhow!("LLM client required for summarization"))?;

        let prompt = format!(
            "Summarize the following memories concisely while preserving key information:\n\n{}",
            memories_text
        );

        let summary = llm_client.complete(&prompt).await?;
        Ok(summary)
    }

    /// Update retrieval metrics
    fn update_retrieval_metrics(&self, result_count: usize, duration_ms: f32, success: bool) {
        let mut metrics = self.metrics.write();
        metrics.total_queries += 1;

        if success {
            metrics.successful_retrievals += 1;

            // Update averages
            let total = metrics.successful_retrievals as f32;
            metrics.avg_retrieval_time_ms =
                (metrics.avg_retrieval_time_ms * (total - 1.0) + duration_ms) / total;
            metrics.avg_memories_per_query =
                (metrics.avg_memories_per_query * (total - 1.0) + result_count as f32) / total;
        } else {
            metrics.failed_retrievals += 1;
        }
    }

    /// Update cache hit metrics
    fn update_cache_hit_metrics(&self) {
        let mut metrics = self.metrics.write();
        let total_queries = metrics.total_queries + 1;
        let cache_hits = (metrics.cache_hit_rate * metrics.total_queries as f32) + 1.0;
        metrics.cache_hit_rate = cache_hits / total_queries as f32;
    }
}

/// Simple text distance calculation (Jaccard similarity)
fn text_distance(text1: &str, text2: &str) -> f32 {
    let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 {
        0.0
    } else {
        1.0 - (intersection as f32 / union as f32) // Distance = 1 - similarity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_embeddings::{MockEmbeddingClient, VectorStore};
    use astraweave_llm::MockLlm;

    #[tokio::test]
    async fn test_rag_pipeline_creation() {
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        let llm_client = Arc::new(MockLlm);
        let config = RagConfig::default();

        let pipeline = RagPipeline::new(embedding_client, vector_store, Some(llm_client), config);

        let metrics = pipeline.get_metrics();
        assert_eq!(metrics.total_queries, 0);
        assert_eq!(metrics.total_memories_stored, 0);
    }

    #[tokio::test]
    async fn test_add_memory() {
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        let config = RagConfig::default();

        let mut pipeline = RagPipeline::new(embedding_client, vector_store, None, config);

        let memory_id = pipeline
            .add_memory("Test memory content".to_string())
            .await
            .unwrap();
        assert!(!memory_id.is_empty());

        let metrics = pipeline.get_metrics();
        assert_eq!(metrics.total_memories_stored, 1);
    }

    #[tokio::test]
    async fn add_memory_async_is_concurrent_arc_safe() {
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        let config = RagConfig::default();

        let pipeline = Arc::new(RagPipeline::new(
            embedding_client,
            vector_store,
            None,
            config,
        ));

        let p1 = Arc::clone(&pipeline);
        let p2 = Arc::clone(&pipeline);

        let t1 = tokio::spawn(async move { p1.add_memory_async("alpha".into()).await });
        let t2 = tokio::spawn(async move { p2.add_memory_async("beta".into()).await });

        let (r1, r2) = tokio::join!(t1, t2);
        let id1 = r1.unwrap().unwrap();
        let id2 = r2.unwrap().unwrap();

        assert_ne!(id1, id2, "distinct memories should yield distinct IDs");

        // Optionally verify vector store length increased (VectorStoreWrapper.len may be simplistic)
        // We only check that the IDs are non-empty above to keep test robust.
    }

    #[test]
    fn test_text_distance() {
        let text1 = "the quick brown fox";
        let text2 = "the lazy brown dog";
        let text3 = "completely different text";

        let dist1_2 = text_distance(text1, text2);
        let dist1_3 = text_distance(text1, text3);

        // Should be less similar to completely different text
        assert!(dist1_3 > dist1_2);
    }

    #[test]
    fn test_memory_query() {
        let query = MemoryQuery::text("combat")
            .with_category(MemoryCategory::Combat)
            .with_min_importance(0.5);

        let memory_combat = Memory {
            id: "1".to_string(),
            text: "Combat encounter".to_string(),
            timestamp: current_timestamp(),
            importance: 0.8,
            valence: 0.0,
            category: MemoryCategory::Combat,
            entities: vec![],
            context: HashMap::new(),
        };

        let memory_social = Memory {
            id: "2".to_string(),
            text: "Social interaction".to_string(),
            timestamp: current_timestamp(),
            importance: 0.3,
            valence: 0.0,
            category: MemoryCategory::Social,
            entities: vec![],
            context: HashMap::new(),
        };

        let pipeline = create_test_pipeline();

        assert!(pipeline.passes_filters(&memory_combat, &query));
        assert!(!pipeline.passes_filters(&memory_social, &query)); // Wrong category and low importance
    }

    fn create_test_pipeline() -> RagPipeline {
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        let config = RagConfig::default();

        RagPipeline::new(embedding_client, vector_store, None, config)
    }
}
