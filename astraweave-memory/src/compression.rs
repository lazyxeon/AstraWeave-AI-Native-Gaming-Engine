//! Memory compression and summarization
//!
//! This module handles compressing and summarizing memories to reduce storage
//! while preserving important information.

use crate::memory_types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for memory compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Minimum age in days before compression
    pub min_age_days: f32,
    /// Minimum importance threshold for compression
    pub importance_threshold: f32,
    /// Maximum compression ratio (0.0 to 1.0)
    pub max_compression_ratio: f32,
    /// Whether to preserve emotional context
    pub preserve_emotional_context: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_age_days: 30.0,
            importance_threshold: 0.3,
            max_compression_ratio: 0.5,
            preserve_emotional_context: true,
        }
    }
}

/// Result of compression operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    /// Number of memories processed
    pub memories_processed: usize,
    /// Number of memories compressed
    pub memories_compressed: usize,
    /// Total size reduction in bytes (estimated)
    pub size_reduction: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

impl Default for CompressionResult {
    fn default() -> Self {
        Self {
            memories_processed: 0,
            memories_compressed: 0,
            size_reduction: 0,
            processing_time_ms: 0,
        }
    }
}

/// Memory compression engine
#[derive(Debug)]
pub struct CompressionEngine {
    config: CompressionConfig,
}

impl CompressionEngine {
    /// Create a new compression engine
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Compress eligible memories
    pub fn compress_memories(&self, memories: &mut Vec<Memory>) -> Result<CompressionResult> {
        let start_time = std::time::Instant::now();
        let mut result = CompressionResult::default();

        for memory in memories.iter_mut() {
            result.memories_processed += 1;

            if self.should_compress(memory)? {
                let original_size = self.estimate_memory_size(memory);
                self.compress_memory(memory)?;
                let compressed_size = self.estimate_memory_size(memory);

                result.memories_compressed += 1;
                result.size_reduction += original_size.saturating_sub(compressed_size);
            }
        }

        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Check if a memory should be compressed
    fn should_compress(&self, memory: &Memory) -> Result<bool> {
        // Don't compress permanent memories
        if memory.metadata.permanent {
            return Ok(false);
        }

        // Check age
        let age_days = (chrono::Utc::now() - memory.metadata.created_at).num_days() as f32;
        if age_days < self.config.min_age_days {
            return Ok(false);
        }

        // Check importance
        if memory.metadata.importance > self.config.importance_threshold {
            return Ok(false);
        }

        // Don't compress if already compressed
        if self.is_compressed(memory) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Compress a single memory
    fn compress_memory(&self, memory: &mut Memory) -> Result<()> {
        // Compress text content
        memory.content.text = self.compress_text(&memory.content.text);

        // Reduce sensory data if not preserving emotional context
        if !self.config.preserve_emotional_context {
            if let Some(ref mut sensory) = memory.content.sensory_data {
                sensory.visual = sensory.visual.as_ref().map(|v| self.compress_text(v));
                sensory.auditory = sensory.auditory.as_ref().map(|a| self.compress_text(a));
                sensory.tactile = sensory.tactile.as_ref().map(|t| self.compress_text(t));
                sensory.environmental = sensory
                    .environmental
                    .as_ref()
                    .map(|e| self.compress_text(e));
            }
        }

        // Reduce context information
        memory.content.context.related_events.truncate(3);
        memory.content.context.participants.truncate(5);

        // Mark as compressed
        memory.metadata.tags.push("compressed".to_string());

        Ok(())
    }

    /// Compress text content using simple summarization
    fn compress_text(&self, text: &str) -> String {
        if text.len() <= 50 {
            return text.to_string();
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        let target_length = (words.len() as f32 * self.config.max_compression_ratio) as usize;
        let compressed_length = target_length.max(10); // Minimum 10 words

        if compressed_length >= words.len() {
            return text.to_string();
        }

        // Simple compression: take first part and last part
        let first_part = words.len() / 3;
        let last_part = compressed_length - first_part;

        let mut compressed_words = Vec::new();
        compressed_words.extend_from_slice(&words[..first_part]);
        compressed_words.push("[...]");
        if last_part > 0 && words.len() > last_part {
            compressed_words.extend_from_slice(&words[words.len() - last_part..]);
        }

        compressed_words.join(" ")
    }

    /// Check if memory is already compressed
    fn is_compressed(&self, memory: &Memory) -> bool {
        memory.metadata.tags.contains(&"compressed".to_string())
    }

    /// Estimate memory size in bytes
    fn estimate_memory_size(&self, memory: &Memory) -> usize {
        let mut size = 0;

        // Text content
        size += memory.content.text.len();

        // Sensory data
        if let Some(ref sensory) = memory.content.sensory_data {
            size += sensory.visual.as_ref().map_or(0, |v| v.len());
            size += sensory.auditory.as_ref().map_or(0, |a| a.len());
            size += sensory.tactile.as_ref().map_or(0, |t| t.len());
            size += sensory.environmental.as_ref().map_or(0, |e| e.len());
        }

        // Context
        size += memory
            .content
            .context
            .location
            .as_ref()
            .map_or(0, |l| l.len());
        size += memory
            .content
            .context
            .time_period
            .as_ref()
            .map_or(0, |t| t.len());

        // Participants and events
        for participant in &memory.content.context.participants {
            size += participant.len();
        }
        for event in &memory.content.context.related_events {
            size += event.len();
        }

        // Tags
        for tag in &memory.metadata.tags {
            size += tag.len();
        }

        // Associations
        size += memory.associations.len() * 64; // Rough estimate per association

        // Embedding
        if let Some(ref embedding) = memory.embedding {
            size += embedding.len() * 4; // f32 = 4 bytes
        }

        size
    }

    /// Get compression statistics
    pub fn get_compression_stats(&self, memories: &[Memory]) -> CompressionStats {
        let total_memories = memories.len();
        let compressed_memories = memories.iter().filter(|m| self.is_compressed(m)).count();

        let total_size: usize = memories.iter().map(|m| self.estimate_memory_size(m)).sum();

        let avg_size = if total_memories > 0 {
            total_size / total_memories
        } else {
            0
        };

        let compression_ratio = if total_memories > 0 {
            compressed_memories as f32 / total_memories as f32
        } else {
            0.0
        };

        CompressionStats {
            total_memories,
            compressed_memories,
            total_size_bytes: total_size,
            average_size_bytes: avg_size,
            compression_ratio,
        }
    }
}

/// Statistics about memory compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub total_memories: usize,
    pub compressed_memories: usize,
    pub total_size_bytes: usize,
    pub average_size_bytes: usize,
    pub compression_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_compression_engine_creation() {
        let config = CompressionConfig::default();
        let engine = CompressionEngine::new(config);
        assert_eq!(engine.config.min_age_days, 30.0);
    }

    #[test]
    fn test_memory_compression() {
        let engine = CompressionEngine::new(CompressionConfig::default());
        let mut memory = Memory::sensory(
            "This is a very long memory text that should be compressed when it meets the criteria for compression".to_string(),
            None,
        );

        // Make memory old enough for compression
        memory.metadata.created_at = Utc::now() - chrono::Duration::days(35);
        memory.metadata.importance = 0.2; // Low importance

        let original_length = memory.content.text.len();
        engine.compress_memory(&mut memory).unwrap();

        assert!(memory.content.text.len() < original_length);
        assert!(memory.metadata.tags.contains(&"compressed".to_string()));
    }

    #[test]
    fn test_should_not_compress_permanent() {
        let engine = CompressionEngine::new(CompressionConfig::default());
        let mut memory = Memory::semantic("Important fact".to_string(), "facts".to_string());
        memory.metadata.permanent = true;

        let should_compress = engine.should_compress(&memory).unwrap();
        assert!(!should_compress);
    }

    #[test]
    fn test_compression_stats() {
        let engine = CompressionEngine::new(CompressionConfig::default());
        let memories = vec![
            Memory::sensory("Test memory 1".to_string(), None),
            Memory::sensory("Test memory 2".to_string(), None),
        ];

        let stats = engine.get_compression_stats(&memories);
        assert_eq!(stats.total_memories, 2);
        assert_eq!(stats.compressed_memories, 0);
    }
}
