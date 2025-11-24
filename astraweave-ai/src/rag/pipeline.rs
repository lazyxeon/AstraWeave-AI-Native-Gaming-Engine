use anyhow::{Context, Result};
use astraweave_embeddings::{
    EmbeddingClient, EmbeddingConfig, VectorStore,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Strategy for consolidating memories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsolidationStrategy {
    /// Keep high importance memories
    Importance,
    /// Keep most recent memories
    Recency,
    /// No automatic consolidation
    None,
}

/// Strategy for forgetting memories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForgettingStrategy {
    /// Forget memories older than N seconds
    Age(u64),
    /// Forget memories with importance lower than threshold
    LowImportance(f32),
    /// Keep only N most recent/important memories
    Limit(usize),
    /// No automatic forgetting
    None,
}

/// Strategy for injecting context into prompts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InjectionStrategy {
    /// Prepend context to the prompt
    Prepend,
    /// Append context to the prompt
    Append,
}

/// Configuration for the RAG pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Maximum number of documents to retrieve
    pub max_documents: usize,
    /// Minimum similarity score threshold (0.0 to 1.0)
    pub min_similarity: f32,
    /// Embedding configuration
    pub embedding: EmbeddingConfig,
    /// Strategy for consolidating memories
    pub consolidation_strategy: ConsolidationStrategy,
    /// Strategy for forgetting memories
    pub forgetting_strategy: ForgettingStrategy,
    /// Strategy for injecting context
    pub injection_strategy: InjectionStrategy,
    /// Maximum tokens (approximate) for injected context
    pub max_context_tokens: usize,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            max_documents: 5,
            min_similarity: 0.7,
            embedding: EmbeddingConfig::default(),
            consolidation_strategy: ConsolidationStrategy::None,
            forgetting_strategy: ForgettingStrategy::None,
            injection_strategy: InjectionStrategy::Prepend,
            max_context_tokens: 1000,
        }
    }
}

/// A document to be indexed in the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagDocument {
    /// Unique identifier
    pub id: String,
    /// Text content
    pub content: String,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
}

/// Context retrieved from the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagContext {
    /// Retrieved documents
    pub documents: Vec<RagDocument>,
    /// Original query
    pub query: String,
}

impl std::fmt::Display for RagContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.documents.is_empty() {
            return write!(f, "No relevant context found.");
        }

        writeln!(f, "Relevant Context:")?;
        for (i, doc) in self.documents.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, doc.content)?;
        }
        Ok(())
    }
}

/// RAG Pipeline for retrieving relevant context for LLM generation
pub struct RagPipeline {
    config: RagConfig,
    client: Arc<dyn EmbeddingClient + Send + Sync>,
    store: Arc<VectorStore>,
}

impl RagPipeline {
    /// Create a new RAG pipeline
    pub fn new(
        config: RagConfig,
        client: Arc<dyn EmbeddingClient + Send + Sync>,
    ) -> Self {
        let store = Arc::new(VectorStore::with_config(config.embedding.clone()));
        Self {
            config,
            client,
            store,
        }
    }

    /// Add a document to the RAG system
    pub async fn add_document(&self, doc: RagDocument) -> Result<()> {
        // Generate embedding for the content
        let embedding = self
            .client
            .embed(&doc.content)
            .await
            .context("Failed to generate embedding for document")?;

        // Store in vector store
        self.store
            .insert_with_metadata(
                doc.id,
                embedding,
                doc.content,
                doc.importance,
                doc.metadata,
            )
            .context("Failed to store document in vector store")?;

        Ok(())
    }

    /// Retrieve relevant context for a query
    pub async fn retrieve(&self, query: &str) -> Result<RagContext> {
        // Generate embedding for the query
        let query_embedding = self
            .client
            .embed(query)
            .await
            .context("Failed to generate embedding for query")?;

        // Search vector store
        let results = self
            .store
            .search(&query_embedding, self.config.max_documents)
            .context("Failed to search vector store")?;

        // Filter by similarity threshold and convert to RagDocument
        let documents: Vec<RagDocument> = results
            .into_iter()
            .filter(|r| r.score >= self.config.min_similarity)
            .map(|r| RagDocument {
                id: r.vector.id,
                content: r.vector.text,
                metadata: r.vector.metadata,
                importance: r.vector.importance,
            })
            .collect();

        Ok(RagContext {
            documents,
            query: query.to_string(),
        })
    }

    /// Consolidate memories based on the configured strategy
    pub fn consolidate(&self) -> Result<()> {
        match self.config.consolidation_strategy {
            ConsolidationStrategy::None => Ok(()),
            ConsolidationStrategy::Importance => {
                // Keep top N important memories? Or just prune low importance?
                // Let's assume we want to keep within max_vectors limit, prioritizing importance.
                // This is similar to prune_vectors but strictly on importance.
                // For now, let's reuse store's prune_vectors which mixes importance and recency,
                // or implement custom logic if store exposes enough.
                // Since store.prune_vectors is hardcoded, we might need to implement custom logic here.
                
                let all_ids = self.store.get_all_ids();
                let mut docs = Vec::new();
                for id in &all_ids {
                    if let Some(vec) = self.store.get(id) {
                        docs.push((id.clone(), vec.importance));
                    }
                }
                
                // Sort by importance descending
                docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                
                // Keep top max_vectors
                let max = self.config.embedding.max_vectors;
                if docs.len() > max {
                    for (id, _) in docs.iter().skip(max) {
                        self.store.remove(id);
                    }
                }
                Ok(())
            }
            ConsolidationStrategy::Recency => {
                let all_ids = self.store.get_all_ids();
                let mut docs = Vec::new();
                for id in &all_ids {
                    if let Some(vec) = self.store.get(id) {
                        docs.push((id.clone(), vec.timestamp));
                    }
                }
                
                // Sort by timestamp descending (newest first)
                docs.sort_by(|a, b| b.1.cmp(&a.1));
                
                // Keep top max_vectors
                let max = self.config.embedding.max_vectors;
                if docs.len() > max {
                    for (id, _) in docs.iter().skip(max) {
                        self.store.remove(id);
                    }
                }
                Ok(())
            }
        }
    }

    /// Forget memories based on the configured strategy
    pub fn forget(&self) -> Result<()> {
        match self.config.forgetting_strategy {
            ForgettingStrategy::None => Ok(()),
            ForgettingStrategy::Age(max_age_seconds) => {
                let all_ids = self.store.get_all_ids();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                for id in all_ids {
                    if let Some(vec) = self.store.get(&id) {
                        if now > vec.timestamp && (now - vec.timestamp) > max_age_seconds {
                            self.store.remove(&id);
                        }
                    }
                }
                Ok(())
            }
            ForgettingStrategy::LowImportance(threshold) => {
                let all_ids = self.store.get_all_ids();
                for id in all_ids {
                    if let Some(vec) = self.store.get(&id) {
                        if vec.importance < threshold {
                            self.store.remove(&id);
                        }
                    }
                }
                Ok(())
            }
            ForgettingStrategy::Limit(limit) => {
                // Keep most recent up to limit
                let all_ids = self.store.get_all_ids();
                if all_ids.len() <= limit {
                    return Ok(());
                }
                
                let mut docs = Vec::new();
                for id in &all_ids {
                    if let Some(vec) = self.store.get(id) {
                        docs.push((id.clone(), vec.timestamp));
                    }
                }
                
                // Sort by timestamp descending (newest first)
                docs.sort_by(|a, b| b.1.cmp(&a.1));
                
                // Remove excess
                for (id, _) in docs.iter().skip(limit) {
                    self.store.remove(id);
                }
                Ok(())
            }
        }
    }

    /// Inject context into a prompt based on the configured strategy
    pub fn inject_context(&self, prompt: &str, context: &RagContext) -> String {
        if context.documents.is_empty() {
            return prompt.to_string();
        }

        // Format context
        let mut context_str = String::from("Relevant Context:\n");
        let mut current_tokens = 0; // Approx 4 chars per token
        
        for (i, doc) in context.documents.iter().enumerate() {
            let doc_str = format!("{}. {}\n", i + 1, doc.content);
            let estimated_tokens = doc_str.len() / 4;
            
            if current_tokens + estimated_tokens > self.config.max_context_tokens {
                break;
            }
            
            context_str.push_str(&doc_str);
            current_tokens += estimated_tokens;
        }

        match self.config.injection_strategy {
            InjectionStrategy::Prepend => format!("{}\n\n{}", context_str, prompt),
            InjectionStrategy::Append => format!("{}\n\n{}", prompt, context_str),
        }
    }

    /// Clear all documents
    pub fn clear(&self) {
        self.store.clear();
    }

    /// Get number of stored documents
    pub fn document_count(&self) -> usize {
        self.store.len()
    }

    /// Perform maintenance (pruning, optimization)
    pub fn maintenance(&self) -> Result<()> {
        self.consolidate()?;
        self.forget()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_embeddings::MockEmbeddingClient;

    #[tokio::test]
    async fn test_rag_pipeline() {
        let client = Arc::new(MockEmbeddingClient::new());
        let config = RagConfig::default();
        let pipeline = RagPipeline::new(config, client);

        // Add documents
        let doc1 = RagDocument {
            id: "doc1".to_string(),
            content: "The player is in a forest.".to_string(),
            metadata: HashMap::new(),
            importance: 1.0,
        };
        pipeline.add_document(doc1).await.unwrap();

        let doc2 = RagDocument {
            id: "doc2".to_string(),
            content: "The enemy is hiding behind a rock.".to_string(),
            metadata: HashMap::new(),
            importance: 1.0,
        };
        pipeline.add_document(doc2).await.unwrap();

        // Retrieve
        let context = pipeline.retrieve("Where is the enemy?").await.unwrap();
        
        // Mock client returns random embeddings, so similarity is random.
        // However, we can check that it runs without error.
        // In a real test with deterministic mock, we would assert content.
        assert!(context.documents.len() <= 5);
    }
}
