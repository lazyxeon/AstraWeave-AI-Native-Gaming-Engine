/*!
# Vector Store

Efficient storage and retrieval of high-dimensional embedding vectors.
Uses HNSW indexing for fast approximate nearest neighbor search.
*/

use crate::{DistanceMetric, EmbeddingConfig, SearchResult, StoredVector};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use hnsw_rs::prelude::*;
use nalgebra as na;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// High-performance vector store with HNSW indexing
pub struct VectorStore {
    /// Configuration
    config: EmbeddingConfig,
    
    /// HNSW index for fast similarity search
    index: Arc<RwLock<Option<Hnsw<f32, DistanceFunc>>>>,
    
    /// Storage for vectors and metadata
    vectors: Arc<DashMap<String, StoredVector>>,
    
    /// ID to HNSW index mapping
    id_to_index: Arc<DashMap<String, usize>>,
    
    /// Index to ID mapping
    index_to_id: Arc<DashMap<usize, String>>,
    
    /// Next available index
    next_index: Arc<parking_lot::Mutex<usize>>,
    
    /// Metrics tracking
    metrics: Arc<RwLock<VectorStoreMetrics>>,
}

/// Distance function type for HNSW
type DistanceFunc = fn(&[f32], &[f32]) -> f32;

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
        let mut config = EmbeddingConfig::default();
        config.dimensions = dimensions;
        Self::with_config(config)
    }
    
    /// Create a vector store with custom configuration
    pub fn with_config(config: EmbeddingConfig) -> Self {
        Self {
            config,
            index: Arc::new(RwLock::new(None)),
            vectors: Arc::new(DashMap::new()),
            id_to_index: Arc::new(DashMap::new()),
            index_to_id: Arc::new(DashMap::new()),
            next_index: Arc::new(parking_lot::Mutex::new(0)),
            metrics: Arc::new(RwLock::new(VectorStoreMetrics::default())),
        }
    }
    
    /// Get the distance function for the configured metric
    fn get_distance_func(&self) -> DistanceFunc {
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
            return Err(anyhow!("Vector store at capacity: {}", self.config.max_vectors));
        }
        
        let stored_vector = StoredVector {
            id: id.clone(),
            vector: vector.clone(),
            text,
            timestamp: current_timestamp(),
            importance: 1.0,
            metadata: HashMap::new(),
        };
        
        // Get next index
        let index = {
            let mut next_idx = self.next_index.lock();
            let idx = *next_idx;
            *next_idx += 1;
            idx
        };
        
        // Store vector and mappings
        self.vectors.insert(id.clone(), stored_vector);
        self.id_to_index.insert(id.clone(), index);
        self.index_to_id.insert(index, id);
        
        // Rebuild index if needed
        self.rebuild_index_if_needed()?;
        
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
        
        let index = {
            let mut next_idx = self.next_index.lock();
            let idx = *next_idx;
            *next_idx += 1;
            idx
        };
        
        self.vectors.insert(id.clone(), stored_vector);
        self.id_to_index.insert(id.clone(), index);
        self.index_to_id.insert(index, id);
        
        self.rebuild_index_if_needed()?;
        
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
        
        let results = {
            let index_guard = self.index.read();
            if let Some(index) = index_guard.as_ref() {
                // Use HNSW index for fast search
                let neighbors = index.search(query, k, 200); // ef = 200
                
                let mut results = Vec::with_capacity(neighbors.len());
                for neighbor in neighbors {
                    if let Some(id) = self.index_to_id.get(&neighbor.d_id) {
                        if let Some(stored_vector) = self.vectors.get(&*id) {
                            let distance = neighbor.distance;
                            let score = match self.config.distance_metric {
                                DistanceMetric::Cosine => 1.0 - distance,
                                DistanceMetric::DotProduct => -distance, // Flip sign for dot product
                                _ => 1.0 / (1.0 + distance), // Convert distance to similarity
                            };
                            
                            results.push(SearchResult {
                                vector: stored_vector.clone(),
                                score,
                                distance,
                            });
                        }
                    }
                }
                results
            } else {
                // Fallback to brute force search
                self.brute_force_search(query, k)?
            }
        };
        
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
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        results.truncate(k);
        
        Ok(results)
    }
    
    /// Remove a vector from the store
    pub fn remove(&self, id: &str) -> Option<StoredVector> {
        if let Some((_, stored_vector)) = self.vectors.remove(id) {
            // Remove from mappings
            if let Some((_, index)) = self.id_to_index.remove(id) {
                self.index_to_id.remove(&index);
            }
            
            // Mark index as needing rebuild
            {
                let mut index_guard = self.index.write();
                *index_guard = None;
            }
            
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
        self.vectors.iter().map(|entry| entry.key().clone()).collect()
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
        self.id_to_index.clear();
        self.index_to_id.clear();
        
        {
            let mut next_idx = self.next_index.lock();
            *next_idx = 0;
        }
        
        {
            let mut index_guard = self.index.write();
            *index_guard = None;
        }
    }
    
    /// Rebuild the HNSW index
    pub fn rebuild_index(&self) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        if self.vectors.is_empty() {
            return Ok(());
        }
        
        let distance_func = self.get_distance_func();
        
        // Create new HNSW index
        let mut hnsw = Hnsw::new(
            16, // max connections per layer
            self.config.dimensions,
            16, // max layers
            200, // ef construction
            distance_func,
        );
        
        // Insert all vectors
        for entry in self.vectors.iter() {
            let stored_vector = entry.value();
            if let Some(index) = self.id_to_index.get(&stored_vector.id) {
                hnsw.insert((&stored_vector.vector, *index));
            }
        }
        
        // Set ef for search
        hnsw.set_ef(100);
        
        // Replace index
        {
            let mut index_guard = self.index.write();
            *index_guard = Some(hnsw);
        }
        
        // Update metrics
        let build_time = start_time.elapsed().as_millis() as f32;
        {
            let mut metrics = self.metrics.write();
            metrics.index_build_time_ms = build_time;
        }
        
        Ok(())
    }
    
    /// Rebuild index if it doesn't exist or is stale
    fn rebuild_index_if_needed(&self) -> Result<()> {
        let needs_rebuild = {
            let index_guard = self.index.read();
            index_guard.is_none()
        };
        
        if needs_rebuild && !self.vectors.is_empty() {
            self.rebuild_index()?;
        }
        
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
        scored_vectors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Remove lowest-scoring vectors
        let to_remove = self.vectors.len() - target_count;
        let mut removed_count = 0;
        
        for (id, _) in scored_vectors.iter().take(to_remove) {
            if self.remove(id).is_some() {
                removed_count += 1;
            }
        }
        
        // Rebuild index after pruning
        if removed_count > 0 {
            self.rebuild_index()?;
        }
        
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
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum()
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
        
        store.insert("test_1".to_string(), vector.clone(), "test vector".to_string()).unwrap();
        
        let retrieved = store.get("test_1").unwrap();
        assert_eq!(retrieved.vector, vector);
        assert_eq!(retrieved.text, "test vector");
        assert_eq!(store.len(), 1);
    }
    
    #[test]
    fn test_search() {
        let store = VectorStore::new(3);
        
        // Insert test vectors
        store.insert("v1".to_string(), vec![1.0, 0.0, 0.0], "first".to_string()).unwrap();
        store.insert("v2".to_string(), vec![0.0, 1.0, 0.0], "second".to_string()).unwrap();
        store.insert("v3".to_string(), vec![0.0, 0.0, 1.0], "third".to_string()).unwrap();
        
        // Search for vector similar to v1
        let query = vec![0.9, 0.1, 0.0];
        let results = store.search(&query, 2).unwrap();
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].vector.id, "v1"); // Should be most similar
    }
    
    #[test]
    fn test_remove() {
        let store = VectorStore::new(3);
        store.insert("test".to_string(), vec![1.0, 2.0, 3.0], "test".to_string()).unwrap();
        
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
        store.insert_with_metadata(
            "low_importance".to_string(),
            vec![1.0, 0.0, 0.0],
            "low".to_string(),
            0.1,
            HashMap::new(),
        ).unwrap();
        
        store.insert_with_metadata(
            "high_importance".to_string(),
            vec![0.0, 1.0, 0.0],
            "high".to_string(),
            0.9,
            HashMap::new(),
        ).unwrap();
        
        store.insert_with_metadata(
            "medium_importance".to_string(),
            vec![0.0, 0.0, 1.0],
            "medium".to_string(),
            0.5,
            HashMap::new(),
        ).unwrap();
        
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