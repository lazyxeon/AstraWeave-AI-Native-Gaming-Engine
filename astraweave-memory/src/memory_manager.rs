//! Memory management system for hierarchical AI memories
//!
//! This module provides the core memory management functionality,
//! including storage, retrieval, and organization of memories.

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    Memory, MemoryCluster, MemoryContent, MemoryMetadata, MemorySource, MemoryType,
    RetrievalContext,
};

/// Simple memory manager for basic memory operations
#[derive(Debug)]
pub struct MemoryManager {
    /// All memories indexed by ID
    memories: HashMap<String, Memory>,
    /// Memory clusters for organization
    clusters: HashMap<String, MemoryCluster>,
    /// Configuration
    config: MemoryManagerConfig,
    /// Memory statistics
    stats: MemoryStats,
}

/// Configuration for memory manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManagerConfig {
    /// Maximum number of memories per type
    pub max_memories_per_type: HashMap<MemoryType, usize>,
    /// Default importance threshold for memory retention
    pub importance_threshold: f32,
    /// Enable automatic memory consolidation
    pub auto_consolidation: bool,
    /// Enable forgetting mechanisms
    pub enable_forgetting: bool,
}

impl Default for MemoryManagerConfig {
    fn default() -> Self {
        let mut max_memories = HashMap::new();
        max_memories.insert(MemoryType::Sensory, 100);
        max_memories.insert(MemoryType::Working, 50);
        max_memories.insert(MemoryType::Episodic, 1000);
        max_memories.insert(MemoryType::Semantic, 5000);
        max_memories.insert(MemoryType::Procedural, 500);
        max_memories.insert(MemoryType::Emotional, 200);
        max_memories.insert(MemoryType::Social, 500);

        Self {
            max_memories_per_type: max_memories,
            importance_threshold: 0.3,
            auto_consolidation: true,
            enable_forgetting: true,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of memories
    pub total_memories: usize,
    /// Memories by type
    pub memories_by_type: HashMap<MemoryType, usize>,
    /// Average importance score
    pub average_importance: f32,
    /// Number of clusters
    pub total_clusters: usize,
    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_memories: 0,
            memories_by_type: HashMap::new(),
            average_importance: 0.0,
            total_clusters: 0,
            last_updated: Utc::now(),
        }
    }
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            clusters: HashMap::new(),
            config: MemoryManagerConfig::default(),
            stats: MemoryStats::default(),
        }
    }

    /// Create a new memory manager with custom config
    pub fn with_config(config: MemoryManagerConfig) -> Self {
        Self {
            memories: HashMap::new(),
            clusters: HashMap::new(),
            config,
            stats: MemoryStats::default(),
        }
    }

    /// Store a new memory
    pub fn store_memory(&mut self, memory: Memory) -> Result<String> {
        let memory_id = memory.id.clone();
        let memory_type = memory.memory_type.clone();

        // Check capacity limits
        if let Some(&max_count) = self.config.max_memories_per_type.get(&memory_type) {
            let current_count = self
                .memories
                .values()
                .filter(|m| m.memory_type == memory_type)
                .count();

            if current_count >= max_count {
                return Err(anyhow!(
                    "Memory type {:?} has reached maximum capacity",
                    memory_type
                ));
            }
        }

        // Store the memory
        self.memories.insert(memory_id.clone(), memory);

        // Update statistics
        self.update_stats();

        Ok(memory_id)
    }

    /// Retrieve a memory by ID
    pub fn get_memory(&mut self, memory_id: &str) -> Option<&mut Memory> {
        if let Some(memory) = self.memories.get_mut(memory_id) {
            memory.accessed();
            Some(memory)
        } else {
            None
        }
    }

    /// Retrieve multiple memories based on context
    pub fn retrieve_memories(&self, context: &RetrievalContext) -> Result<Vec<&Memory>> {
        let mut results = Vec::new();

        for memory in self.memories.values() {
            if memory.matches_context(context) {
                let relevance = memory.calculate_relevance(context);
                if relevance > 0.3 {
                    // Basic relevance threshold
                    results.push(memory);
                }
            }
        }

        // Sort by relevance (simplified)
        results.sort_by(|a, b| {
            let relevance_a = a.calculate_relevance(context);
            let relevance_b = b.calculate_relevance(context);
            relevance_b
                .partial_cmp(&relevance_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(context.limit);

        Ok(results)
    }

    /// Delete a memory
    pub fn delete_memory(&mut self, memory_id: &str) -> Result<()> {
        if self.memories.remove(memory_id).is_some() {
            self.update_stats();
            Ok(())
        } else {
            Err(anyhow!("Memory not found: {}", memory_id))
        }
    }

    /// Get all memories of a specific type
    pub fn get_memories_by_type(&self, memory_type: &MemoryType) -> Vec<&Memory> {
        self.memories
            .values()
            .filter(|memory| &memory.memory_type == memory_type)
            .collect()
    }

    /// Create a new memory cluster
    pub fn create_cluster(&mut self, name: String, memory_ids: Vec<String>) -> Result<String> {
        // Verify all memory IDs exist
        for memory_id in &memory_ids {
            if !self.memories.contains_key(memory_id) {
                return Err(anyhow!("Memory not found: {}", memory_id));
            }
        }

        let mut cluster = MemoryCluster::new(
            name,
            crate::memory_types::ClusterType::Concept,
            "Generated cluster".to_string(),
        );

        for memory_id in memory_ids {
            cluster.add_memory(memory_id);
        }

        let cluster_id = cluster.id.clone();
        self.clusters.insert(cluster_id.clone(), cluster);
        self.update_stats();

        Ok(cluster_id)
    }

    /// Get a cluster by ID
    pub fn get_cluster(&self, cluster_id: &str) -> Option<&MemoryCluster> {
        self.clusters.get(cluster_id)
    }

    /// Get all memories in a cluster
    pub fn get_cluster_memories(&self, cluster_id: &str) -> Result<Vec<&Memory>> {
        if let Some(cluster) = self.clusters.get(cluster_id) {
            let memories = cluster
                .memory_ids
                .iter()
                .filter_map(|id| self.memories.get(id))
                .collect();
            Ok(memories)
        } else {
            Err(anyhow!("Cluster not found: {}", cluster_id))
        }
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Update internal statistics
    fn update_stats(&mut self) {
        let mut memories_by_type = HashMap::new();
        let mut total_importance = 0.0;

        for memory in self.memories.values() {
            *memories_by_type
                .entry(memory.memory_type.clone())
                .or_insert(0) += 1;
            total_importance += memory.metadata.importance;
        }

        let average_importance = if !self.memories.is_empty() {
            total_importance / self.memories.len() as f32
        } else {
            0.0
        };

        self.stats = MemoryStats {
            total_memories: self.memories.len(),
            memories_by_type,
            average_importance,
            total_clusters: self.clusters.len(),
            last_updated: Utc::now(),
        };
    }

    /// Clean up weak memories based on configured thresholds
    pub fn cleanup_weak_memories(&mut self) -> Result<usize> {
        let mut to_remove = Vec::new();

        for (id, memory) in &self.memories {
            if !memory.metadata.permanent && memory.should_forget(self.config.importance_threshold)
            {
                to_remove.push(id.clone());
            }
        }

        let removed_count = to_remove.len();
        for id in to_remove {
            self.memories.remove(&id);
        }

        if removed_count > 0 {
            self.update_stats();
        }

        Ok(removed_count)
    }

    /// Get configuration
    pub fn get_config(&self) -> &MemoryManagerConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MemoryManagerConfig) {
        self.config = config;
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_types::*;

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::new();
        assert_eq!(manager.get_stats().total_memories, 0);
    }

    #[test]
    fn test_store_and_retrieve_memory() {
        let mut manager = MemoryManager::new();
        let memory = Memory::sensory("Test sensory memory".to_string(), None);
        let memory_id = memory.id.clone();

        manager.store_memory(memory).unwrap();
        assert_eq!(manager.get_stats().total_memories, 1);

        let retrieved = manager.get_memory(&memory_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content.text, "Test sensory memory");
    }

    #[test]
    fn test_memory_clustering() {
        let mut manager = MemoryManager::new();

        let memory1 = Memory::episodic("Event 1".to_string(), vec![], None);
        let memory2 = Memory::episodic("Event 2".to_string(), vec![], None);

        let id1 = memory1.id.clone();
        let id2 = memory2.id.clone();

        manager.store_memory(memory1).unwrap();
        manager.store_memory(memory2).unwrap();

        let cluster_id = manager
            .create_cluster("Test Cluster".to_string(), vec![id1, id2])
            .unwrap();

        let cluster_memories = manager.get_cluster_memories(&cluster_id).unwrap();
        assert_eq!(cluster_memories.len(), 2);
    }

    #[test]
    fn test_memory_cleanup() {
        let mut manager = MemoryManager::new();
        let mut memory = Memory::sensory("Weak memory".to_string(), None);
        memory.metadata.importance = 0.1; // Below default threshold
        memory.metadata.strength = 0.1; // Weak

        manager.store_memory(memory).unwrap();
        assert_eq!(manager.get_stats().total_memories, 1);

        let removed = manager.cleanup_weak_memories().unwrap();
        assert_eq!(removed, 1);
        assert_eq!(manager.get_stats().total_memories, 0);
    }
}
