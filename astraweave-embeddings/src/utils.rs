/*!
# Utilities

Helper functions and utilities for embedding operations.
*/

use crate::{Memory, MemoryCategory, StoredVector};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Text preprocessing utilities
pub struct TextPreprocessor;

impl TextPreprocessor {
    /// Clean and normalize text for embedding
    pub fn preprocess(text: &str) -> String {
        // Remove extra whitespace
        let text = text.trim();
        
        // Normalize Unicode
        let text = text.chars().collect::<String>();
        
        // Limit length (most embedding models have token limits)
        if text.len() > 8192 {
            text[..8192].to_string()
        } else {
            text
        }
    }
    
    /// Split long text into chunks
    pub fn chunk_text(text: &str, max_chunk_size: usize, overlap: usize) -> Vec<String> {
        if text.len() <= max_chunk_size {
            return vec![text.to_string()];
        }
        
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < text.len() {
            let end = (start + max_chunk_size).min(text.len());
            let chunk = &text[start..end];
            chunks.push(chunk.to_string());
            
            if end >= text.len() {
                break;
            }
            
            start = end.saturating_sub(overlap);
        }
        
        chunks
    }
    
    /// Extract key phrases from text (simple implementation)
    pub fn extract_keyphrases(text: &str) -> Vec<String> {
        // Simple word-based extraction
        let words: Vec<&str> = text
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();
        
        // Return unique words as keyphrases
        let mut keyphrases: Vec<String> = words
            .into_iter()
            .map(|w| w.to_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        keyphrases.sort();
        keyphrases
    }
}

/// Memory utilities for game experiences
pub struct MemoryUtils;

impl MemoryUtils {
    /// Create a memory from game event text
    pub fn create_memory(
        text: String,
        category: MemoryCategory,
        importance: f32,
        entities: Vec<String>,
    ) -> Memory {
        Memory {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            timestamp: current_timestamp(),
            importance: importance.clamp(0.0, 1.0),
            valence: 0.0, // Neutral by default
            category,
            entities,
            context: HashMap::new(),
        }
    }
    
    /// Create a social memory
    pub fn create_social_memory(
        text: String,
        entities: Vec<String>,
        importance: f32,
        valence: f32,
    ) -> Memory {
        Memory {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            timestamp: current_timestamp(),
            importance: importance.clamp(0.0, 1.0),
            valence: valence.clamp(-1.0, 1.0),
            category: MemoryCategory::Social,
            entities,
            context: HashMap::new(),
        }
    }
    
    /// Create a combat memory
    pub fn create_combat_memory(
        text: String,
        entities: Vec<String>,
        importance: f32,
        outcome: CombatOutcome,
    ) -> Memory {
        let mut context = HashMap::new();
        context.insert("outcome".to_string(), format!("{:?}", outcome));
        
        let valence = match outcome {
            CombatOutcome::Victory => 0.5,
            CombatOutcome::Defeat => -0.5,
            CombatOutcome::Draw => 0.0,
            CombatOutcome::Retreat => -0.2,
        };
        
        Memory {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            timestamp: current_timestamp(),
            importance: importance.clamp(0.0, 1.0),
            valence,
            category: MemoryCategory::Combat,
            entities,
            context,
        }
    }
    
    /// Calculate memory decay based on time and importance
    pub fn calculate_decay(memory: &Memory, current_time: u64) -> f32 {
        let age_hours = ((current_time - memory.timestamp) as f32) / 3600.0;
        let base_decay = (-age_hours / (24.0 * 7.0)).exp(); // Week half-life
        
        // Important memories decay slower
        let importance_factor = 1.0 + memory.importance * 2.0;
        let adjusted_decay = base_decay.powf(1.0 / importance_factor);
        
        adjusted_decay.clamp(0.0, 1.0)
    }
    
    /// Determine if a memory should be forgotten
    pub fn should_forget(memory: &Memory, current_time: u64, forget_threshold: f32) -> bool {
        let decay = Self::calculate_decay(memory, current_time);
        decay < forget_threshold
    }
    
    /// Consolidate similar memories
    pub fn consolidate_memories(memories: Vec<Memory>, similarity_threshold: f32) -> Vec<Memory> {
        // Simple consolidation based on text similarity
        // In practice, this would use embedding similarity
        let mut consolidated = Vec::new();
        let mut used = vec![false; memories.len()];
        
        for i in 0..memories.len() {
            if used[i] {
                continue;
            }
            
            let mut group = vec![memories[i].clone()];
            used[i] = true;
            
            // Find similar memories
            for j in (i + 1)..memories.len() {
                if used[j] {
                    continue;
                }
                
                // Simple text similarity (could be improved with embeddings)
                let similarity = text_similarity(&memories[i].text, &memories[j].text);
                if similarity > similarity_threshold {
                    group.push(memories[j].clone());
                    used[j] = true;
                }
            }
            
            if group.len() == 1 {
                consolidated.push(group[0].clone());
            } else {
                // Merge memories in the group
                let merged = Self::merge_memory_group(group);
                consolidated.push(merged);
            }
        }
        
        consolidated
    }
    
    /// Merge a group of similar memories
    fn merge_memory_group(mut memories: Vec<Memory>) -> Memory {
        if memories.is_empty() {
            panic!("Cannot merge empty memory group");
        }
        
        if memories.len() == 1 {
            return memories.into_iter().next().unwrap();
        }
        
        // Sort by importance (highest first)
        memories.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        
        let primary = &memories[0];
        let mut merged_text = primary.text.clone();
        let mut merged_importance = primary.importance;
        let mut merged_valence = primary.valence;
        let mut merged_entities = primary.entities.clone();
        let mut merged_context = primary.context.clone();
        
        // Merge additional memories
        for memory in memories.iter().skip(1) {
            // Combine text
            if !merged_text.contains(&memory.text) {
                merged_text.push_str(&format!("; {}", memory.text));
            }
            
            // Average importance (weighted by recency)
            merged_importance = (merged_importance + memory.importance) / 2.0;
            
            // Average valence
            merged_valence = (merged_valence + memory.valence) / 2.0;
            
            // Merge entities
            for entity in &memory.entities {
                if !merged_entities.contains(entity) {
                    merged_entities.push(entity.clone());
                }
            }
            
            // Merge context
            for (key, value) in &memory.context {
                merged_context.entry(key.clone()).or_insert(value.clone());
            }
        }
        
        Memory {
            id: uuid::Uuid::new_v4().to_string(),
            text: merged_text,
            timestamp: primary.timestamp, // Keep earliest timestamp
            importance: merged_importance,
            valence: merged_valence,
            category: primary.category,
            entities: merged_entities,
            context: merged_context,
        }
    }
}

/// Combat outcome for memory creation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CombatOutcome {
    Victory,
    Defeat,
    Draw,
    Retreat,
}

/// Similarity metrics
pub struct SimilarityMetrics;

impl SimilarityMetrics {
    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
    
    /// Calculate Jaccard similarity between two sets of strings
    pub fn jaccard_similarity(a: &[String], b: &[String]) -> f32 {
        let set_a: std::collections::HashSet<_> = a.iter().collect();
        let set_b: std::collections::HashSet<_> = b.iter().collect();
        
        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

/// Batch processing utilities
pub struct BatchProcessor;

impl BatchProcessor {
    /// Process items in batches with a maximum batch size
    pub async fn process_batches<T, F, R, Fut>(
        items: Vec<T>,
        batch_size: usize,
        mut processor: F,
    ) -> Result<Vec<R>>
    where
        T: Clone,
        F: FnMut(Vec<T>) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<R>>>,
    {
        let mut results = Vec::with_capacity(items.len());
        
        for chunk in items.chunks(batch_size) {
            let batch_results = processor(chunk.to_vec()).await?;
            results.extend(batch_results);
        }
        
        Ok(results)
    }
}

/// Performance monitoring
pub struct PerformanceMonitor {
    start_time: std::time::Instant,
}

impl PerformanceMonitor {
    /// Start monitoring
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }
    
    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> u128 {
        self.start_time.elapsed().as_micros()
    }
}

/// Simple text similarity using Jaccard similarity of words
fn text_similarity(a: &str, b: &str) -> f32 {
    let words_a: Vec<String> = a
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect();
    let words_b: Vec<String> = b
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect();
    
    SimilarityMetrics::jaccard_similarity(&words_a, &words_b)
}

/// Get current Unix timestamp in seconds
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_preprocessing() {
        let text = "  Hello World!  ";
        let processed = TextPreprocessor::preprocess(text);
        assert_eq!(processed, "Hello World!");
    }
    
    #[test]
    fn test_text_chunking() {
        let text = "This is a long text that needs to be split into chunks";
        let chunks = TextPreprocessor::chunk_text(text, 20, 5);
        
        assert!(chunks.len() > 1);
        assert!(chunks[0].len() <= 20);
    }
    
    #[test]
    fn test_memory_creation() {
        let memory = MemoryUtils::create_memory(
            "Player defeated goblin".to_string(),
            MemoryCategory::Combat,
            0.8,
            vec!["player".to_string(), "goblin".to_string()],
        );
        
        assert_eq!(memory.category, MemoryCategory::Combat);
        assert_eq!(memory.importance, 0.8);
        assert_eq!(memory.entities.len(), 2);
    }
    
    #[test]
    fn test_memory_decay() {
        let memory = Memory {
            id: "test".to_string(),
            text: "test memory".to_string(),
            timestamp: 1000,
            importance: 0.5,
            valence: 0.0,
            category: MemoryCategory::Social,
            entities: vec![],
            context: HashMap::new(),
        };
        
        let current_time = 1000 + 24 * 3600; // 24 hours later
        let decay = MemoryUtils::calculate_decay(&memory, current_time);
        
        assert!(decay < 1.0);
        assert!(decay > 0.0);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        
        let sim_identical = SimilarityMetrics::cosine_similarity(&a, &b);
        let sim_orthogonal = SimilarityMetrics::cosine_similarity(&a, &c);
        
        assert_eq!(sim_identical, 1.0);
        assert_eq!(sim_orthogonal, 0.0);
    }
    
    #[test]
    fn test_jaccard_similarity() {
        let a = vec!["hello".to_string(), "world".to_string()];
        let b = vec!["hello".to_string(), "world".to_string()];
        let c = vec!["foo".to_string(), "bar".to_string()];
        
        let sim_identical = SimilarityMetrics::jaccard_similarity(&a, &b);
        let sim_different = SimilarityMetrics::jaccard_similarity(&a, &c);
        
        assert_eq!(sim_identical, 1.0);
        assert_eq!(sim_different, 0.0);
    }
    
    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = monitor.elapsed_ms();
        
        assert!(elapsed >= 10);
    }
}