//! Memory retrieval and search functionality
//!
//! This module provides sophisticated memory retrieval using various search strategies
//! including semantic similarity, temporal proximity, and associative connections.

use crate::memory_types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for memory retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// Maximum number of memories to retrieve
    pub max_results: usize,
    /// Minimum relevance score for inclusion
    pub relevance_threshold: f32,
    /// Weight for semantic similarity
    pub semantic_weight: f32,
    /// Weight for temporal proximity
    pub temporal_weight: f32,
    /// Weight for associative strength
    pub associative_weight: f32,
    /// Whether to boost recent memories
    pub recency_boost: bool,
    /// Whether to follow associative chains
    pub follow_associations: bool,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            relevance_threshold: 0.3,
            semantic_weight: 0.6,
            temporal_weight: 0.2,
            associative_weight: 0.2,
            recency_boost: true,
            follow_associations: true,
        }
    }
}

/// Result of a memory retrieval operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// Retrieved memory
    pub memory: Memory,
    /// Overall relevance score
    pub relevance_score: f32,
    /// Breakdown of score components
    pub score_breakdown: ScoreBreakdown,
    /// Retrieval path (how this memory was found)
    pub retrieval_path: RetrievalPath,
}

/// Breakdown of relevance score components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub semantic_score: f32,
    pub temporal_score: f32,
    pub associative_score: f32,
    pub importance_score: f32,
    pub recency_score: f32,
}

/// How a memory was retrieved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalPath {
    /// Direct match to query
    Direct,
    /// Found through association with another memory
    Associative { source_memory_id: String },
    /// Found through temporal proximity
    Temporal { reference_time: chrono::DateTime<chrono::Utc> },
    /// Found through clustering
    Cluster { cluster_id: String },
}

/// Memory retrieval engine
#[derive(Debug)]
pub struct RetrievalEngine {
    config: RetrievalConfig,
}

impl RetrievalEngine {
    /// Create a new retrieval engine
    pub fn new(config: RetrievalConfig) -> Self {
        Self { config }
    }

    /// Retrieve memories based on context
    pub fn retrieve(
        &self,
        context: &RetrievalContext,
        memories: &[Memory],
    ) -> Result<Vec<RetrievalResult>> {
        let mut results = Vec::new();

        // First pass: direct matching
        for memory in memories {
            if memory.matches_context(context) {
                let relevance_score = self.calculate_relevance(context, memory)?;

                if relevance_score >= self.config.relevance_threshold {
                    let score_breakdown = self.calculate_score_breakdown(context, memory)?;

                    results.push(RetrievalResult {
                        memory: memory.clone(),
                        relevance_score,
                        score_breakdown,
                        retrieval_path: RetrievalPath::Direct,
                    });
                }
            }
        }

        // Second pass: associative retrieval
        if self.config.follow_associations {
            let associated_results = self.retrieve_associated_memories(context, memories, &results)?;
            results.extend(associated_results);
        }

        // Sort by relevance score
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        results.truncate(context.limit.min(self.config.max_results));

        Ok(results)
    }

    /// Calculate overall relevance score
    fn calculate_relevance(&self, context: &RetrievalContext, memory: &Memory) -> Result<f32> {
        let breakdown = self.calculate_score_breakdown(context, memory)?;

        let mut total_score = 0.0;
        total_score += breakdown.semantic_score * self.config.semantic_weight;
        total_score += breakdown.temporal_score * self.config.temporal_weight;
        total_score += breakdown.associative_score * self.config.associative_weight;
        total_score += breakdown.importance_score * 0.2; // Fixed weight for importance

        if self.config.recency_boost {
            total_score += breakdown.recency_score * 0.1; // Fixed weight for recency
        }

        Ok(total_score.min(1.0))
    }

    /// Calculate detailed score breakdown
    fn calculate_score_breakdown(
        &self,
        context: &RetrievalContext,
        memory: &Memory,
    ) -> Result<ScoreBreakdown> {
        let semantic_score = self.calculate_semantic_similarity(&context.query, &memory.content.text);
        let temporal_score = self.calculate_temporal_score(context, memory);
        let associative_score = self.calculate_associative_score(context, memory);
        let importance_score = memory.metadata.importance;
        let recency_score = self.calculate_recency_score(memory);

        Ok(ScoreBreakdown {
            semantic_score,
            temporal_score,
            associative_score,
            importance_score,
            recency_score,
        })
    }

    /// Calculate semantic similarity between query and memory content
    fn calculate_semantic_similarity(&self, query: &str, content: &str) -> f32 {
        // Simple word-based similarity (in practice, use embeddings)
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        if query_words.is_empty() || content_words.is_empty() {
            return 0.0;
        }

        let common_words = query_words
            .iter()
            .filter(|word| content_words.contains(word))
            .count();

        common_words as f32 / query_words.len() as f32
    }

    /// Calculate temporal proximity score
    fn calculate_temporal_score(&self, context: &RetrievalContext, memory: &Memory) -> f32 {
        if let Some(time_window) = &context.time_window {
            if memory.metadata.created_at >= time_window.start
                && memory.metadata.created_at <= time_window.end {
                return 1.0;
            }

            // Calculate distance from time window
            let distance_start = (memory.metadata.created_at - time_window.start).num_days().abs();
            let distance_end = (memory.metadata.created_at - time_window.end).num_days().abs();
            let min_distance = distance_start.min(distance_end) as f32;

            // Exponential decay based on distance
            (-min_distance / 7.0).exp()
        } else {
            0.5 // Neutral score if no temporal context
        }
    }

    /// Calculate associative strength score
    fn calculate_associative_score(&self, context: &RetrievalContext, memory: &Memory) -> f32 {
        let mut max_association_strength: f32 = 0.0;

        for recent_memory_id in &context.recent_memory_ids {
            if let Some(association) = memory
                .associations
                .iter()
                .find(|assoc| &assoc.memory_id == recent_memory_id)
            {
                max_association_strength = max_association_strength.max(association.strength);
            }
        }

        max_association_strength
    }

    /// Calculate recency score
    fn calculate_recency_score(&self, memory: &Memory) -> f32 {
        let now = chrono::Utc::now();
        let age_days = (now - memory.metadata.created_at).num_days() as f32;
        let last_access_days = (now - memory.metadata.last_accessed).num_days() as f32;

        // Combine creation recency and access recency
        let creation_recency = (-age_days / 30.0).exp(); // Decay over 30 days
        let access_recency = (-last_access_days / 7.0).exp(); // Decay over 7 days

        (creation_recency + access_recency) / 2.0
    }

    /// Retrieve memories through associative connections
    fn retrieve_associated_memories(
        &self,
        context: &RetrievalContext,
        all_memories: &[Memory],
        direct_results: &[RetrievalResult],
    ) -> Result<Vec<RetrievalResult>> {
        let mut associated_results = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        // Collect IDs of direct results to avoid duplicates
        for result in direct_results {
            processed_ids.insert(result.memory.id.clone());
        }

        // Follow associations from direct results
        for result in direct_results {
            for association in &result.memory.associations {
                if processed_ids.contains(&association.memory_id) {
                    continue;
                }

                // Find the associated memory
                if let Some(associated_memory) = all_memories
                    .iter()
                    .find(|m| m.id == association.memory_id)
                {
                    // Calculate relevance for associated memory
                    let base_relevance = self.calculate_relevance(context, associated_memory)?;
                    let association_boost = association.strength * 0.3; // Boost from association
                    let final_relevance = (base_relevance + association_boost).min(1.0);

                    if final_relevance >= self.config.relevance_threshold {
                        let score_breakdown = self.calculate_score_breakdown(context, associated_memory)?;

                        associated_results.push(RetrievalResult {
                            memory: associated_memory.clone(),
                            relevance_score: final_relevance,
                            score_breakdown,
                            retrieval_path: RetrievalPath::Associative {
                                source_memory_id: result.memory.id.clone(),
                            },
                        });

                        processed_ids.insert(association.memory_id.clone());
                    }
                }
            }
        }

        Ok(associated_results)
    }

    /// Find memories similar to a given memory
    pub fn find_similar(&self, target_memory: &Memory, all_memories: &[Memory]) -> Result<Vec<RetrievalResult>> {
        let mut results = Vec::new();

        for memory in all_memories {
            if memory.id == target_memory.id {
                continue; // Skip the target memory itself
            }

            let similarity = self.calculate_memory_similarity(target_memory, memory)?;

            if similarity >= self.config.relevance_threshold {
                let score_breakdown = ScoreBreakdown {
                    semantic_score: similarity,
                    temporal_score: 0.0,
                    associative_score: 0.0,
                    importance_score: memory.metadata.importance,
                    recency_score: self.calculate_recency_score(memory),
                };

                results.push(RetrievalResult {
                    memory: memory.clone(),
                    relevance_score: similarity,
                    score_breakdown,
                    retrieval_path: RetrievalPath::Direct,
                });
            }
        }

        // Sort by similarity
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results.truncate(self.config.max_results);
        Ok(results)
    }

    /// Calculate similarity between two memories
    fn calculate_memory_similarity(&self, memory1: &Memory, memory2: &Memory) -> Result<f32> {
        let mut similarity = 0.0;

        // Text similarity
        let text_sim = self.calculate_semantic_similarity(&memory1.content.text, &memory2.content.text);
        similarity += text_sim * 0.5;

        // Type similarity
        if memory1.memory_type == memory2.memory_type {
            similarity += 0.2;
        }

        // Location similarity
        if memory1.content.context.location == memory2.content.context.location
            && memory1.content.context.location.is_some()
        {
            similarity += 0.1;
        }

        // Participant overlap
        let common_participants = memory1
            .content
            .context
            .participants
            .iter()
            .filter(|p| memory2.content.context.participants.contains(p))
            .count();

        if !memory1.content.context.participants.is_empty() {
            let participant_sim = common_participants as f32 / memory1.content.context.participants.len() as f32;
            similarity += participant_sim * 0.2;
        }

        Ok(similarity.min(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_retrieval_engine_creation() {
        let config = RetrievalConfig::default();
        let engine = RetrievalEngine::new(config);
        assert_eq!(engine.config.max_results, 10);
    }

    #[test]
    fn test_semantic_similarity() {
        let engine = RetrievalEngine::new(RetrievalConfig::default());
        let similarity = engine.calculate_semantic_similarity("hello world", "world hello");
        assert!(similarity > 0.0);
    }

    #[test]
    fn test_memory_retrieval() {
        let engine = RetrievalEngine::new(RetrievalConfig::default());
        let memories = vec![
            Memory::episodic(
                "Went to the park with Alice".to_string(),
                vec!["Alice".to_string()],
                Some("park".to_string()),
            ),
            Memory::semantic("The sky is blue".to_string(), "color".to_string()),
        ];

        let context = RetrievalContext {
            query: "park".to_string(),
            emotional_state: None,
            location: Some("park".to_string()),
            recent_memory_ids: Vec::new(),
            preferred_types: vec![MemoryType::Episodic],
            time_window: None,
            limit: 5,
        };

        let results = engine.retrieve(&context, &memories).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_find_similar_memories() {
        let engine = RetrievalEngine::new(RetrievalConfig::default());
        let target_memory = Memory::episodic(
            "Had lunch with Bob".to_string(),
            vec!["Bob".to_string()],
            Some("restaurant".to_string()),
        );

        let all_memories = vec![
            Memory::episodic(
                "Had dinner with Bob".to_string(),
                vec!["Bob".to_string()],
                Some("restaurant".to_string()),
            ),
            Memory::semantic("Cats are independent".to_string(), "animals".to_string()),
        ];

        let results = engine.find_similar(&target_memory, &all_memories).unwrap();
        assert!(!results.is_empty());
    }
}