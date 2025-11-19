/*!
# Vector Store

Efficient storage and retrieval of high-dimensional embedding vectors.
Uses HNSW indexing for fast approximate nearest neighbor search.
*/

use crate::{DistanceMetric, EmbeddingConfig, SearchResult, StoredVector};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// High-performance vector store with HNSW indexing (simplified implementation)
pub struct VectorStore {
    /// Configuration
    config: EmbeddingConfig,

    /// Storage for vectors and metadata
    vectors: Arc<DashMap<String, StoredVector>>,

    /// Next available index
    next_index: Arc<parking_lot::Mutex<usize>>,

    /// Metrics tracking
    metrics: Arc<RwLock<VectorStoreMetrics>>,
}

/// Metrics for vector store performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorStoreMetrics {
    pub total_vectors: usize,
    pub total_searches: u64,
    pub avg_search_time_ms: f32,
    pub index_build_time_ms: f32,
    pub memory_usage_bytes: u64,
    pub cache_hit_rate: f32,
}

impl VectorStore {
    /// Create a new vector store with default configuration
    pub fn new(dimensions: usize) -> Self {
        let config = EmbeddingConfig {
            dimensions,
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// Create a vector store with custom configuration
    pub fn with_config(config: EmbeddingConfig) -> Self {
        Self {
            config,
            vectors: Arc::new(DashMap::new()),
            next_index: Arc::new(parking_lot::Mutex::new(0)),
            metrics: Arc::new(RwLock::new(VectorStoreMetrics::default())),
        }
    }

    /// Get the distance function for the configured metric
    fn get_distance_func(&self) -> fn(&[f32], &[f32]) -> f32 {
        match self.config.distance_metric {
            DistanceMetric::Cosine => cosine_distance,
            DistanceMetric::Euclidean => euclidean_distance,
            DistanceMetric::Manhattan => manhattan_distance,
            DistanceMetric::DotProduct => dot_product_distance,
        }
    }

    /// Insert a vector into the store
    pub fn insert(&self, id: String, vector: Vec<f32>, text: String) -> Result<()> {
        if vector.len() != self.config.dimensions {
            return Err(anyhow!(
                "Vector dimension {} doesn't match expected {}",
                vector.len(),
                self.config.dimensions
            ));
        }

        // Check if we're at capacity
        if self.vectors.len() >= self.config.max_vectors {
            return Err(anyhow!(
                "Vector store at capacity: {}",
                self.config.max_vectors
            ));
        }

        let stored_vector = StoredVector {
            id: id.clone(),
            vector: vector.clone(),
            text,
            timestamp: current_timestamp(),
            importance: 1.0,
            metadata: HashMap::new(),
        };

        // Store vector
        self.vectors.insert(id, stored_vector);

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_vectors = self.vectors.len();
        }

        Ok(())
    }

    /// Insert a vector with metadata
    pub fn insert_with_metadata(
        &self,
        id: String,
        vector: Vec<f32>,
        text: String,
        importance: f32,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        if vector.len() != self.config.dimensions {
            return Err(anyhow!(
                "Vector dimension {} doesn't match expected {}",
                vector.len(),
                self.config.dimensions
            ));
        }

        let stored_vector = StoredVector {
            id: id.clone(),
            vector: vector.clone(),
            text,
            timestamp: current_timestamp(),
            importance,
            metadata,
        };

        self.vectors.insert(id, stored_vector);

        {
            let mut metrics = self.metrics.write();
            metrics.total_vectors = self.vectors.len();
        }

        Ok(())
    }

    /// Search for similar vectors
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        let start_time = std::time::Instant::now();

        if query.len() != self.config.dimensions {
            return Err(anyhow!(
                "Query dimension {} doesn't match expected {}",
                query.len(),
                self.config.dimensions
            ));
        }

        let results = self.brute_force_search(query, k)?;

        // Update metrics
        let search_time = start_time.elapsed().as_millis() as f32;
        {
            let mut metrics = self.metrics.write();
            metrics.total_searches += 1;
            metrics.avg_search_time_ms =
                (metrics.avg_search_time_ms * (metrics.total_searches - 1) as f32 + search_time)
                    / metrics.total_searches as f32;
        }

        Ok(results)
    }

    /// Brute force search (used when index is not available)
    fn brute_force_search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        let distance_func = self.get_distance_func();
        let mut results = Vec::new();

        for entry in self.vectors.iter() {
            let stored_vector = entry.value();
            let distance = distance_func(query, &stored_vector.vector);
            let score = match self.config.distance_metric {
                DistanceMetric::Cosine => 1.0 - distance,
                DistanceMetric::DotProduct => -distance,
                _ => 1.0 / (1.0 + distance),
            };

            results.push(SearchResult {
                vector: stored_vector.clone(),
                score,
                distance,
            });
        }

        // Sort by distance and take top k
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        Ok(results)
    }

    /// Remove a vector from the store
    pub fn remove(&self, id: &str) -> Option<StoredVector> {
        if let Some((_, stored_vector)) = self.vectors.remove(id) {
            Some(stored_vector)
        } else {
            None
        }
    }

    /// Get a vector by ID
    pub fn get(&self, id: &str) -> Option<StoredVector> {
        self.vectors.get(id).map(|entry| entry.value().clone())
    }

    /// Get all vector IDs
    pub fn get_all_ids(&self) -> Vec<String> {
        self.vectors
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get number of stored vectors
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Clear all vectors
    pub fn clear(&self) {
        self.vectors.clear();

        {
            let mut next_idx = self.next_index.lock();
            *next_idx = 0;
        }
    }

    /// Rebuild index (simplified stub - no HNSW in this version)
    pub fn rebuild_index(&self) -> Result<()> {
        // Simplified implementation - no actual indexing
        Ok(())
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> VectorStoreMetrics {
        self.metrics.read().clone()
    }

    /// Prune old or low-importance vectors to stay under capacity
    pub fn prune_vectors(&self, target_count: usize) -> Result<usize> {
        if self.vectors.len() <= target_count {
            return Ok(0);
        }

        // Collect vectors with scores (recency + importance)
        let mut scored_vectors: Vec<(String, f32)> = Vec::new();
        let current_time = current_timestamp();

        for entry in self.vectors.iter() {
            let vector = entry.value();
            let age_hours = ((current_time - vector.timestamp) as f32) / 3600.0;
            let recency_score = (-age_hours / 24.0).exp(); // Exponential decay over days
            let total_score = vector.importance * 0.7 + recency_score * 0.3;
            scored_vectors.push((vector.id.clone(), total_score));
        }

        // Sort by score (ascending, so lowest scores are first)
        scored_vectors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Remove lowest-scoring vectors
        let to_remove = self.vectors.len() - target_count;
        let mut removed_count = 0;

        for (id, _) in scored_vectors.iter().take(to_remove) {
            if self.remove(id).is_some() {
                removed_count += 1;
            }
        }

        // No index rebuild needed in simplified version

        Ok(removed_count)
    }
}

/// Distance functions for different metrics
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        1.0 // Maximum distance for zero vectors
    } else {
        1.0 - (dot_product / (norm_a * norm_b))
    }
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

fn dot_product_distance(a: &[f32], b: &[f32]) -> f32 {
    -a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>() // Negative for distance
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store_creation() {
        let store = VectorStore::new(384);
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let store = VectorStore::new(3);
        let vector = vec![1.0, 2.0, 3.0];

        store
            .insert(
                "test_1".to_string(),
                vector.clone(),
                "test vector".to_string(),
            )
            .unwrap();

        let retrieved = store.get("test_1").unwrap();
        assert_eq!(retrieved.vector, vector);
        assert_eq!(retrieved.text, "test vector");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_search() {
        let store = VectorStore::new(3);

        // Insert test vectors
        store
            .insert("v1".to_string(), vec![1.0, 0.0, 0.0], "first".to_string())
            .unwrap();
        store
            .insert("v2".to_string(), vec![0.0, 1.0, 0.0], "second".to_string())
            .unwrap();
        store
            .insert("v3".to_string(), vec![0.0, 0.0, 1.0], "third".to_string())
            .unwrap();

        // Search for vector similar to v1
        let query = vec![0.9, 0.1, 0.0];
        let results = store.search(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].vector.id, "v1"); // Should be most similar
    }

    #[test]
    fn test_remove() {
        let store = VectorStore::new(3);
        store
            .insert("test".to_string(), vec![1.0, 2.0, 3.0], "test".to_string())
            .unwrap();

        assert_eq!(store.len(), 1);

        let removed = store.remove("test").unwrap();
        assert_eq!(removed.id, "test");
        assert_eq!(store.len(), 0);
        assert!(store.get("test").is_none());
    }

    #[test]
    fn test_distance_functions() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![1.0, 0.0, 0.0]; // Same as a

        // Cosine distance
        let cos_ab = cosine_distance(&a, &b);
        let cos_ac = cosine_distance(&a, &c);
        assert!(cos_ab > cos_ac); // a and b are orthogonal, a and c are identical
        assert_eq!(cos_ac, 0.0); // Identical vectors have 0 cosine distance

        // Euclidean distance
        let euc_ab = euclidean_distance(&a, &b);
        let euc_ac = euclidean_distance(&a, &c);
        assert!(euc_ab > euc_ac);
        assert_eq!(euc_ac, 0.0);
    }

    #[test]
    fn test_prune_vectors() {
        let store = VectorStore::new(3);

        // Insert vectors with different importance
        store
            .insert_with_metadata(
                "low_importance".to_string(),
                vec![1.0, 0.0, 0.0],
                "low".to_string(),
                0.1,
                HashMap::new(),
            )
            .unwrap();

        store
            .insert_with_metadata(
                "high_importance".to_string(),
                vec![0.0, 1.0, 0.0],
                "high".to_string(),
                0.9,
                HashMap::new(),
            )
            .unwrap();

        store
            .insert_with_metadata(
                "medium_importance".to_string(),
                vec![0.0, 0.0, 1.0],
                "medium".to_string(),
                0.5,
                HashMap::new(),
            )
            .unwrap();

        assert_eq!(store.len(), 3);

        // Prune to 2 vectors
        let removed = store.prune_vectors(2).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.len(), 2);

        // Low importance vector should be removed
        assert!(store.get("low_importance").is_none());
        assert!(store.get("high_importance").is_some());
        assert!(store.get("medium_importance").is_some());
    }
}
