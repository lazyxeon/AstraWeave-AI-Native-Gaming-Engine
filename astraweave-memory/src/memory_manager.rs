use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use lru::LruCache;
use dashmap::DashMap;
use chrono::Utc;

use astraweave_embeddings::EmbeddingClient;
use astraweave_rag::RagPipeline;

use crate::{
    Memory, MemoryType, MemoryCluster, RetrievalContext, ConsolidationState,
    ForgettingCurve, SharingMetadata, MemoryContent, MemoryMetadata, MemorySource
};

/// Advanced memory manager with hierarchical memory types and intelligent management
pub struct AdvancedMemoryManager {
    /// All memories indexed by ID
    memories: Arc<DashMap<String, Memory>>,
    /// Memory clusters for organization
    clusters: Arc<RwLock<HashMap<String, MemoryCluster>>>,
    /// LRU cache for frequently accessed memories
    memory_cache: Arc<RwLock<LruCache<String, Memory>>>,
    /// Embedding client for vector operations
    embedding_client: Arc<dyn EmbeddingClient>,
    /// RAG pipeline for semantic retrieval
    rag_pipeline: Arc<RagPipeline>,
    /// Configuration
    config: MemoryManagerConfig,
    /// Consolidation states for memories
    consolidation_states: Arc<RwLock<HashMap<String, ConsolidationState>>>,
    /// Forgetting curves for different memory types
    forgetting_curves: Arc<RwLock<HashMap<MemoryType, ForgettingCurve>>>,
    /// Memory sharing metadata
    sharing_metadata: Arc<RwLock<HashMap<String, SharingMetadata>>>,
    /// Memory statistics
    stats: Arc<RwLock<MemoryStats>>,
}

/// Configuration for memory manager
#[derive(Debug, Clone)]
pub struct MemoryManagerConfig {
    /// Maximum number of memories per type
    pub max_memories_per_type: HashMap<MemoryType, usize>,
    /// Cache size for frequently accessed memories
    pub cache_size: usize,
    /// Consolidation interval in hours
    pub consolidation_interval: u64,
    /// Forgetting threshold (memories below this strength are forgotten)
    pub forgetting_threshold: f32,
    /// Enable automatic memory consolidation
    pub enable_consolidation: bool,
    /// Enable intelligent forgetting
    pub enable_forgetting: bool,
    /// Enable memory compression
    pub enable_compression: bool,
    /// Minimum importance for permanent storage
    pub permanent_threshold: f32,
}

impl Default for MemoryManagerConfig {
    fn default() -> Self {
        let mut max_memories = HashMap::new();
        max_memories.insert(MemoryType::Sensory, 1000);
        max_memories.insert(MemoryType::Working, 50);
        max_memories.insert(MemoryType::Episodic, 5000);
        max_memories.insert(MemoryType::Semantic, 10000);
        max_memories.insert(MemoryType::Procedural, 1000);
        max_memories.insert(MemoryType::Emotional, 2000);
        max_memories.insert(MemoryType::Social, 3000);

        Self {
            max_memories_per_type: max_memories,
            cache_size: 100,
            consolidation_interval: 24, // 24 hours
            forgetting_threshold: 0.1,
            enable_consolidation: true,
            enable_forgetting: true,
            enable_compression: true,
            permanent_threshold: 0.9,
        }
    }
}

/// Memory statistics for monitoring and optimization
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub memories_by_type: HashMap<MemoryType, usize>,
    pub cache_hit_rate: f32,
    pub consolidation_runs: u64,
    pub memories_forgotten: u64,
    pub memories_compressed: u64,
    pub average_retrieval_time_ms: f32,
    pub memory_usage_mb: f32,
}

/// Memory retrieval result with metadata
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub memories: Vec<Memory>,
    pub relevance_scores: Vec<f32>,
    pub retrieval_time_ms: u64,
    pub cache_hits: usize,
    pub total_searched: usize,
}

/// Memory consolidation result
#[derive(Debug, Clone)]
pub struct ConsolidationResult {
    pub memories_consolidated: usize,
    pub new_associations_formed: usize,
    pub clusters_created: usize,
    pub clusters_merged: usize,
    pub processing_time_ms: u64,
}

impl AdvancedMemoryManager {
    pub fn new(
        embedding_client: Arc<dyn EmbeddingClient>,
        rag_pipeline: Arc<RagPipeline>,
        config: MemoryManagerConfig,
    ) -> Self {
        let memory_cache = LruCache::new(
            std::num::NonZeroUsize::new(config.cache_size).unwrap()
        );

        // Initialize default forgetting curves
        let mut forgetting_curves = HashMap::new();
        forgetting_curves.insert(MemoryType::Sensory, ForgettingCurve {
            initial_strength: 1.0,
            decay_rate: 0.5,
            half_life: 0.1, // 2.4 hours
            retention_threshold: 0.1,
            immune: false,
        });
        forgetting_curves.insert(MemoryType::Working, ForgettingCurve {
            initial_strength: 1.0,
            decay_rate: 0.3,
            half_life: 1.0, // 1 day
            retention_threshold: 0.2,
            immune: false,
        });
        forgetting_curves.insert(MemoryType::Episodic, ForgettingCurve {
            initial_strength: 1.0,
            decay_rate: 0.1,
            half_life: 30.0, // 30 days
            retention_threshold: 0.1,
            immune: false,
        });
        forgetting_curves.insert(MemoryType::Semantic, ForgettingCurve {
            initial_strength: 1.0,
            decay_rate: 0.05,
            half_life: 365.0, // 1 year
            retention_threshold: 0.05,
            immune: true, // Semantic memories are often permanent
        });

        Self {
            memories: Arc::new(DashMap::new()),
            clusters: Arc::new(RwLock::new(HashMap::new())),
            memory_cache: Arc::new(RwLock::new(memory_cache)),
            embedding_client,
            rag_pipeline,
            config,
            consolidation_states: Arc::new(RwLock::new(HashMap::new())),
            forgetting_curves: Arc::new(RwLock::new(forgetting_curves)),
            sharing_metadata: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(MemoryStats::default())),
        }
    }

    /// Store a new memory
    pub async fn store_memory(&self, mut memory: Memory) -> Result<String> {
        debug!("Storing memory: {}", memory.content.text);

        let memory_id = memory.id.clone();
        let memory_type = memory.memory_type.clone();

        // Generate embedding if not present
        if memory.embedding.is_none() {
            match self.embedding_client.embed(&memory.content.text).await {
                Ok(embedding) => memory.embedding = Some(embedding),
                Err(e) => warn!("Failed to generate embedding for memory {}: {}", memory_id, e),
            }
        }

        // Check memory limits
        self.enforce_memory_limits(&memory_type).await?;

        // Store in RAG pipeline for semantic search
        self.rag_pipeline.store_memory(&memory.content.text, memory.metadata.importance).await
            .map_err(|e| anyhow!("Failed to store memory in RAG pipeline: {}", e))?;

        // Initialize consolidation state
        let consolidation_state = ConsolidationState {
            consolidation_level: 0.0,
            passes: 0,
            last_consolidation: Utc::now(),
            needs_consolidation: false,
            priority: memory.metadata.importance,
        };

        {
            let mut states = self.consolidation_states.write().await;
            states.insert(memory_id.clone(), consolidation_state);
        }

        // Store memory
        self.memories.insert(memory_id.clone(), memory);

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_memories += 1;
            *stats.memories_by_type.entry(memory_type).or_insert(0) += 1;
        }

        info!("Stored memory: {}", memory_id);
        Ok(memory_id)
    }

    /// Retrieve memories based on context
    pub async fn retrieve_memories(&self, context: RetrievalContext) -> Result<RetrievalResult> {
        let start_time = std::time::Instant::now();
        debug!("Retrieving memories for query: {}", context.query);

        let mut retrieved_memories = Vec::new();
        let mut relevance_scores = Vec::new();
        let mut cache_hits = 0;

        // First, check cache for recent memories
        {
            let cache = self.memory_cache.read().await;
            for memory_id in &context.recent_memory_ids {
                if let Some(memory) = cache.peek(memory_id) {
                    if memory.matches_context(&context) {
                        let relevance = memory.calculate_relevance(&context);
                        retrieved_memories.push(memory.clone());
                        relevance_scores.push(relevance);
                        cache_hits += 1;
                    }
                }
            }
        }

        // Semantic search through RAG pipeline
        let semantic_results = self.rag_pipeline.retrieve(&context.query, context.limit).await
            .unwrap_or_else(|e| {
                warn!("RAG retrieval failed: {}", e);
                Vec::new()
            });

        // Find corresponding memories and calculate relevance
        let mut memory_candidates = Vec::new();
        for semantic_text in semantic_results {
            // Find memory by content (simplified - would use better indexing)
            for memory_entry in self.memories.iter() {
                let memory = memory_entry.value();
                if memory.content.text.contains(&semantic_text) && memory.matches_context(&context) {
                    let relevance = memory.calculate_relevance(&context);
                    memory_candidates.push((memory.clone(), relevance));
                }
            }
        }

        // Sort by relevance and take top results
        memory_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let remaining_slots = context.limit.saturating_sub(retrieved_memories.len());

        for (memory, relevance) in memory_candidates.into_iter().take(remaining_slots) {
            retrieved_memories.push(memory);
            relevance_scores.push(relevance);
        }

        // Update access counts and cache frequently accessed memories
        {
            let mut cache = self.memory_cache.write().await;
            for memory in &mut retrieved_memories {
                // Update memory access in main storage
                if let Some(mut stored_memory) = self.memories.get_mut(&memory.id) {
                    stored_memory.accessed();
                }
                
                // Add to cache
                cache.put(memory.id.clone(), memory.clone());
            }
        }

        let retrieval_time = start_time.elapsed().as_millis() as u64;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            if cache_hits + retrieved_memories.len() > 0 {
                let hit_rate = cache_hits as f32 / (cache_hits + retrieved_memories.len()) as f32;
                stats.cache_hit_rate = (stats.cache_hit_rate * 0.9) + (hit_rate * 0.1); // Moving average
            }
            stats.average_retrieval_time_ms = (stats.average_retrieval_time_ms * 0.9) + (retrieval_time as f32 * 0.1);
        }

        let result = RetrievalResult {
            memories: retrieved_memories,
            relevance_scores,
            retrieval_time_ms: retrieval_time,
            cache_hits,
            total_searched: self.memories.len(),
        };

        debug!("Retrieved {} memories in {}ms", result.memories.len(), retrieval_time);
        Ok(result)
    }

    /// Consolidate memories to form associations and clusters
    pub async fn consolidate_memories(&self) -> Result<ConsolidationResult> {
        if !self.config.enable_consolidation {
            return Ok(ConsolidationResult {
                memories_consolidated: 0,
                new_associations_formed: 0,
                clusters_created: 0,
                clusters_merged: 0,
                processing_time_ms: 0,
            });
        }

        let start_time = std::time::Instant::now();
        info!("Starting memory consolidation");

        let mut memories_consolidated = 0;
        let mut new_associations_formed = 0;
        let mut clusters_created = 0;

        // Get memories that need consolidation
        let memories_to_consolidate = self.get_memories_needing_consolidation().await;

        for memory_id in memories_to_consolidate {
            if let Some(mut memory) = self.memories.get_mut(&memory_id) {
                // Find similar memories for association
                let similar_memories = self.find_similar_memories(&memory).await?;

                for (similar_id, similarity_score) in similar_memories {
                    if similarity_score > 0.7 && !memory.associations.iter().any(|a| a.memory_id == similar_id) {
                        memory.add_association(similar_id, crate::AssociationType::Conceptual, similarity_score);
                        new_associations_formed += 1;
                    }
                }

                // Update consolidation state
                {
                    let mut states = self.consolidation_states.write().await;
                    if let Some(state) = states.get_mut(&memory_id) {
                        state.consolidation_level = (state.consolidation_level + 0.2).min(1.0);
                        state.passes += 1;
                        state.last_consolidation = Utc::now();
                        state.needs_consolidation = state.consolidation_level < 0.8;
                    }
                }

                memories_consolidated += 1;
            }
        }

        // Create new clusters for highly associated memories
        clusters_created += self.create_memory_clusters().await?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.consolidation_runs += 1;
        }

        let result = ConsolidationResult {
            memories_consolidated,
            new_associations_formed,
            clusters_created,
            clusters_merged: 0, // Placeholder for cluster merging
            processing_time_ms: processing_time,
        };

        info!("Consolidation complete: {} memories processed, {} associations formed, {} clusters created",
              result.memories_consolidated, result.new_associations_formed, result.clusters_created);

        Ok(result)
    }

    /// Apply intelligent forgetting to remove obsolete memories
    pub async fn apply_forgetting(&self) -> Result<usize> {
        if !self.config.enable_forgetting {
            return Ok(0);
        }

        debug!("Applying intelligent forgetting");

        let mut memories_forgotten = 0;
        let forgetting_curves = self.forgetting_curves.read().await;

        // Collect memories to potentially forget
        let mut candidates_to_forget = Vec::new();

        for memory_entry in self.memories.iter() {
            let memory = memory_entry.value();
            
            if memory.metadata.permanent {
                continue;
            }

            // Check against forgetting curve
            if let Some(curve) = forgetting_curves.get(&memory.memory_type) {
                if !curve.immune {
                    let current_strength = memory.calculate_current_strength();
                    if current_strength < curve.retention_threshold {
                        candidates_to_forget.push(memory.id.clone());
                    }
                }
            } else if memory.should_forget(self.config.forgetting_threshold) {
                candidates_to_forget.push(memory.id.clone());
            }
        }

        // Remove forgotten memories
        for memory_id in candidates_to_forget {
            self.forget_memory(&memory_id).await?;
            memories_forgotten += 1;
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.memories_forgotten += memories_forgotten as u64;
        }

        if memories_forgotten > 0 {
            info!("Forgot {} memories", memories_forgotten);
        }

        Ok(memories_forgotten)
    }

    /// Get a specific memory by ID
    pub async fn get_memory(&self, memory_id: &str) -> Option<Memory> {
        // Check cache first
        {
            let cache = self.memory_cache.read().await;
            if let Some(memory) = cache.peek(memory_id) {
                return Some(memory.clone());
            }
        }

        // Get from main storage
        if let Some(mut memory) = self.memories.get_mut(memory_id) {
            memory.accessed();
            
            // Add to cache
            {
                let mut cache = self.memory_cache.write().await;
                cache.put(memory_id.to_string(), memory.clone());
            }

            Some(memory.clone())
        } else {
            None
        }
    }

    /// Update an existing memory
    pub async fn update_memory(&self, memory_id: &str, updated_memory: Memory) -> Result<()> {
        if let Some(mut memory) = self.memories.get_mut(memory_id) {
            *memory = updated_memory;

            // Update cache if present
            {
                let mut cache = self.memory_cache.write().await;
                cache.put(memory_id.to_string(), memory.clone());
            }

            debug!("Updated memory: {}", memory_id);
            Ok(())
        } else {
            Err(anyhow!("Memory {} not found", memory_id))
        }
    }

    /// Delete a memory
    pub async fn delete_memory(&self, memory_id: &str) -> Result<()> {
        self.forget_memory(memory_id).await
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let mut stats = self.stats.write().await;
        
        // Update current statistics
        stats.total_memories = self.memories.len();
        stats.memories_by_type.clear();
        
        for memory_entry in self.memories.iter() {
            let memory_type = &memory_entry.value().memory_type;
            *stats.memories_by_type.entry(memory_type.clone()).or_insert(0) += 1;
        }

        // Estimate memory usage (simplified)
        stats.memory_usage_mb = (stats.total_memories * 1024) as f32 / (1024.0 * 1024.0); // Rough estimate

        stats.clone()
    }

    /// Get memories that need consolidation
    async fn get_memories_needing_consolidation(&self) -> Vec<String> {
        let states = self.consolidation_states.read().await;
        let mut candidates = Vec::new();

        for (memory_id, state) in states.iter() {
            if state.needs_consolidation {
                candidates.push(memory_id.clone());
            }
        }

        // Sort by priority
        candidates.sort_by(|a, b| {
            let priority_a = states.get(a).map(|s| s.priority).unwrap_or(0.0);
            let priority_b = states.get(b).map(|s| s.priority).unwrap_or(0.0);
            priority_b.partial_cmp(&priority_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top candidates
        candidates.into_iter().take(100).collect()
    }

    /// Find similar memories for association formation
    async fn find_similar_memories(&self, target_memory: &Memory) -> Result<Vec<(String, f32)>> {
        let mut similar_memories = Vec::new();

        // Use RAG pipeline for semantic similarity
        let similar_texts = self.rag_pipeline.retrieve(&target_memory.content.text, 10).await
            .unwrap_or_else(|e| {
                warn!("Failed to find similar memories: {}", e);
                Vec::new()
            });

        // Find corresponding memory IDs and calculate similarity scores
        for similar_text in similar_texts {
            for memory_entry in self.memories.iter() {
                let memory = memory_entry.value();
                if memory.id != target_memory.id && memory.content.text.contains(&similar_text) {
                    // Calculate similarity score (simplified)
                    let similarity = self.calculate_memory_similarity(target_memory, memory);
                    similar_memories.push((memory.id.clone(), similarity));
                }
            }
        }

        // Sort by similarity
        similar_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similar_memories.truncate(5); // Top 5 similar memories

        Ok(similar_memories)
    }

    /// Calculate similarity between two memories
    fn calculate_memory_similarity(&self, memory1: &Memory, memory2: &Memory) -> f32 {
        let mut similarity = 0.0;

        // Type similarity
        if memory1.memory_type == memory2.memory_type {
            similarity += 0.3;
        }

        // Content similarity (simplified word overlap)
        let words1: Vec<&str> = memory1.content.text.split_whitespace().collect();
        let words2: Vec<&str> = memory2.content.text.split_whitespace().collect();

        let common_words = words1.iter().filter(|word| words2.contains(word)).count();
        let total_words = (words1.len() + words2.len()) as f32;
        
        if total_words > 0.0 {
            similarity += (common_words as f32 / total_words) * 0.5;
        }

        // Temporal similarity
        let time_diff = (memory1.metadata.created_at - memory2.metadata.created_at)
            .num_hours().abs() as f32;
        
        if time_diff < 24.0 {
            similarity += 0.2 * (1.0 - time_diff / 24.0);
        }

        similarity.min(1.0)
    }

    /// Create memory clusters from highly associated memories
    async fn create_memory_clusters(&self) -> Result<usize> {
        let mut clusters_created = 0;
        let mut potential_clusters = HashMap::new();

        // Find groups of highly associated memories
        for memory_entry in self.memories.iter() {
            let memory = memory_entry.value();
            let strong_associations = memory.get_strong_associations(0.8);

            if strong_associations.len() >= 2 {
                // Create a cluster key from sorted memory IDs
                let mut cluster_members = vec![memory.id.clone()];
                cluster_members.extend(strong_associations.iter().map(|a| a.memory_id.clone()));
                cluster_members.sort();
                
                let cluster_key = cluster_members.join(",");
                potential_clusters.entry(cluster_key).or_insert_with(Vec::new).push(memory.id.clone());
            }
        }

        // Create clusters for groups with multiple members
        let mut clusters = self.clusters.write().await;
        for (_, cluster_members) in potential_clusters {
            if cluster_members.len() >= 3 {
                let cluster_name = format!("Auto Cluster {}", clusters.len() + 1);
                let mut cluster = MemoryCluster::new(
                    cluster_name,
                    crate::ClusterType::Concept,
                    "Auto-generated".to_string(),
                );

                for member_id in cluster_members {
                    cluster.add_memory(member_id);
                }

                clusters.insert(cluster.id.clone(), cluster);
                clusters_created += 1;
            }
        }

        Ok(clusters_created)
    }

    /// Enforce memory limits by type
    async fn enforce_memory_limits(&self, memory_type: &MemoryType) -> Result<()> {
        if let Some(&max_memories) = self.config.max_memories_per_type.get(memory_type) {
            let current_count = self.memories.iter()
                .filter(|entry| &entry.value().memory_type == memory_type)
                .count();

            if current_count >= max_memories {
                // Remove oldest, least important memories of this type
                let mut memories_to_remove = Vec::new();

                for memory_entry in self.memories.iter() {
                    let memory = memory_entry.value();
                    if &memory.memory_type == memory_type && !memory.metadata.permanent {
                        memories_to_remove.push((
                            memory.id.clone(),
                            memory.metadata.importance * memory.calculate_current_strength(),
                        ));
                    }
                }

                // Sort by importance * strength (ascending) and remove lowest
                memories_to_remove.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                let to_remove_count = current_count - max_memories + 1;

                for (memory_id, _) in memories_to_remove.into_iter().take(to_remove_count) {
                    self.forget_memory(&memory_id).await?;
                }
            }
        }

        Ok(())
    }

    /// Forget (remove) a specific memory
    async fn forget_memory(&self, memory_id: &str) -> Result<()> {
        // Remove from main storage
        if let Some((_, memory)) = self.memories.remove(memory_id) {
            debug!("Forgetting memory: {}", memory.content.text);

            // Remove from cache
            {
                let mut cache = self.memory_cache.write().await;
                cache.pop(memory_id);
            }

            // Remove consolidation state
            {
                let mut states = self.consolidation_states.write().await;
                states.remove(memory_id);
            }

            // Remove sharing metadata
            {
                let mut sharing = self.sharing_metadata.write().await;
                sharing.remove(memory_id);
            }

            // Remove from clusters
            {
                let mut clusters = self.clusters.write().await;
                for cluster in clusters.values_mut() {
                    cluster.remove_memory(memory_id);
                }
                // Clean up empty clusters
                clusters.retain(|_, cluster| !cluster.memory_ids.is_empty());
            }

            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.total_memories = stats.total_memories.saturating_sub(1);
                if let Some(count) = stats.memories_by_type.get_mut(&memory.memory_type) {
                    *count = count.saturating_sub(1);
                }
            }

            Ok(())
        } else {
            Err(anyhow!("Memory {} not found for forgetting", memory_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_embeddings::MockEmbeddingClient;
    use astraweave_rag::MockRagPipeline;
    use crate::{Memory, MemoryType, MemoryContent, SpatialTemporalContext};

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let rag_pipeline = Arc::new(MockRagPipeline::new());
        let config = MemoryManagerConfig::default();

        let manager = AdvancedMemoryManager::new(embedding_client, rag_pipeline, config);
        assert_eq!(manager.memories.len(), 0);
    }

    #[tokio::test]
    async fn test_memory_storage_and_retrieval() {
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let rag_pipeline = Arc::new(MockRagPipeline::new());
        let manager = AdvancedMemoryManager::new(embedding_client, rag_pipeline, MemoryManagerConfig::default());

        let memory = Memory::sensory("I see a red apple".to_string(), None);
        let memory_id = manager.store_memory(memory).await.unwrap();

        let retrieved = manager.get_memory(&memory_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content.text, "I see a red apple");
    }

    #[tokio::test]
    async fn test_memory_limits_enforcement() {
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let rag_pipeline = Arc::new(MockRagPipeline::new());
        
        let mut config = MemoryManagerConfig::default();
        config.max_memories_per_type.insert(MemoryType::Sensory, 2);
        
        let manager = AdvancedMemoryManager::new(embedding_client, rag_pipeline, config);

        // Store memories up to limit
        for i in 0..3 {
            let memory = Memory::sensory(format!("Memory {}", i), None);
            manager.store_memory(memory).await.unwrap();
        }

        let stats = manager.get_stats().await;
        assert_eq!(stats.memories_by_type.get(&MemoryType::Sensory).unwrap_or(&0), &2);
    }
}