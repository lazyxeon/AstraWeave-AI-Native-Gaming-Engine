//! Memory consolidation and compression functionality
//!
//! This module handles consolidating and compressing memories to optimize storage and retrieval.

use anyhow::Result;
use astraweave_embeddings::Memory;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use astraweave_embeddings::MemoryCategory;

/// Consolidation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Minimum similarity threshold for merging memories
    pub merge_threshold: f32,
    /// Maximum age in seconds for memories to be considered for consolidation
    pub max_age_seconds: u64,
    /// Whether to enable automatic consolidation
    pub auto_consolidate: bool,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            merge_threshold: 0.85,
            max_age_seconds: 86400, // 24 hours
            auto_consolidate: true,
        }
    }
}

/// Result of a consolidation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    /// Number of memories processed
    pub processed_count: usize,
    /// Number of memories merged
    pub merged_count: usize,
    /// Number of memories removed
    pub removed_count: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Memory consolidation engine
#[derive(Debug)]
pub struct ConsolidationEngine {
    /// Configuration
    config: ConsolidationConfig,
}

impl ConsolidationEngine {
    /// Create a new consolidation engine
    pub fn new(config: ConsolidationConfig) -> Self {
        Self { config }
    }

    /// Consolidate a collection of memories
    pub fn consolidate(&self, memories: Vec<Memory>) -> Result<(Vec<Memory>, ConsolidationResult)> {
        let start_time = std::time::Instant::now();
        let processed_count = memories.len();
        let mut consolidated_memories = Vec::new();
        let mut merged_count = 0;
        let mut removed_count = 0;

        // Simple consolidation logic - merge similar memories
        let mut processed_indices = std::collections::HashSet::new();

        for (i, memory) in memories.iter().enumerate() {
            if processed_indices.contains(&i) {
                continue;
            }

            let mut merged_memory = memory.clone();
            let mut merge_occurred = false;

            // Look for similar memories to merge
            for (j, other_memory) in memories.iter().enumerate().skip(i + 1) {
                if processed_indices.contains(&j) {
                    continue;
                }

                if self.should_merge(memory, other_memory) {
                    merged_memory = self.merge_memories(&merged_memory, other_memory)?;
                    processed_indices.insert(j);
                    merge_occurred = true;
                    removed_count += 1;
                }
            }

            consolidated_memories.push(merged_memory);
            processed_indices.insert(i);

            if merge_occurred {
                merged_count += 1;
            }
        }

        let result = ConsolidationResult {
            processed_count,
            merged_count,
            removed_count,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        };

        Ok((consolidated_memories, result))
    }

    /// Check if two memories should be merged
    fn should_merge(&self, memory1: &Memory, memory2: &Memory) -> bool {
        // Same category
        if memory1.category != memory2.category {
            return false;
        }

        // Similar content (simplified similarity check)
        let similarity = self.calculate_similarity(&memory1.text, &memory2.text);
        similarity >= self.config.merge_threshold
    }

    /// Merge two memories into one
    fn merge_memories(&self, memory1: &Memory, memory2: &Memory) -> Result<Memory> {
        let mut merged = memory1.clone();

        // Combine content
        merged.text = format!("{} {}", memory1.text, memory2.text);

        // Take the most recent timestamp
        merged.timestamp = memory1.timestamp.max(memory2.timestamp);

        // Merge context
        for (key, value) in &memory2.context {
            merged.context.insert(key.clone(), value.clone());
        }

        // Generate new ID
        merged.id = format!("merged_{}", uuid::Uuid::new_v4());

        Ok(merged)
    }

    /// Calculate content similarity (simplified)
    fn calculate_similarity(&self, content1: &str, content2: &str) -> f32 {
        let words1: Vec<&str> = content1.split_whitespace().collect();
        let words2: Vec<&str> = content2.split_whitespace().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let common_words = words1.iter().filter(|word| words2.contains(word)).count();

        let total_unique_words = words1.len() + words2.len() - common_words;
        common_words as f32 / total_unique_words as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consolidation_engine() {
        let config = ConsolidationConfig::default();
        let engine = ConsolidationEngine::new(config);

        let memories = vec![
            Memory {
                id: "1".to_string(),
                text: "The cat is sleeping".to_string(),
                category: MemoryCategory::Social,
                timestamp: chrono::Utc::now().timestamp() as u64,
                importance: 0.5,
                valence: 0.0,
                entities: vec![],
                context: HashMap::new(),
            },
            Memory {
                id: "2".to_string(),
                text: "The cat is resting".to_string(),
                category: MemoryCategory::Social,
                timestamp: chrono::Utc::now().timestamp() as u64,
                importance: 0.5,
                valence: 0.0,
                entities: vec![],
                context: HashMap::new(),
            },
        ];

        let (consolidated, result) = engine.consolidate(memories).unwrap();
        assert!(!consolidated.is_empty());
        assert_eq!(result.processed_count, 2);
    }
}
