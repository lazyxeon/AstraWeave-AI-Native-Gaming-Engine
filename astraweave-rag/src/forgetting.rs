//! Memory forgetting and decay mechanisms
//!
//! This module handles memory decay, forgetting, and cleanup processes.

use anyhow::Result;
use astraweave_embeddings::{Memory, MemoryCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Forgetting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingConfig {
    /// Enable memory forgetting
    pub enabled: bool,

    /// Base decay rate per hour (0.0 to 1.0)
    pub base_decay_rate: f32,

    /// Importance factor (important memories decay slower)
    pub importance_factor: f32,

    /// Minimum importance threshold for retention
    pub min_importance_threshold: f32,

    /// Maximum age in seconds before forced forgetting
    pub max_memory_age: u64,

    /// Memory cleanup interval (in seconds)
    pub cleanup_interval: u64,

    /// Categories that should never be forgotten
    pub protected_categories: Vec<MemoryCategory>,
}

impl Default for ForgettingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_decay_rate: 0.1,
            importance_factor: 2.0,
            min_importance_threshold: 0.2,
            max_memory_age: 2592000, // 30 days
            cleanup_interval: 86400, // Daily
            protected_categories: vec![MemoryCategory::Quest],
        }
    }
}

/// Memory strength and decay information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStrength {
    /// Current strength (0.0 to 1.0)
    pub current_strength: f32,
    /// Initial strength
    pub initial_strength: f32,
    /// Last access timestamp
    pub last_access: i64,
    /// Number of times accessed
    pub access_count: u32,
    /// Whether this memory should be protected from forgetting
    pub protected: bool,
}

impl Default for MemoryStrength {
    fn default() -> Self {
        Self {
            current_strength: 1.0,
            initial_strength: 1.0,
            last_access: chrono::Utc::now().timestamp(),
            access_count: 0,
            protected: false,
        }
    }
}

/// Result of a forgetting operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingResult {
    /// Number of memories processed
    pub processed_count: usize,
    /// Number of memories forgotten (removed)
    pub forgotten_count: usize,
    /// Number of memories with updated strength
    pub updated_count: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Memory forgetting engine
#[derive(Debug)]
pub struct ForgettingEngine {
    /// Configuration
    config: ForgettingConfig,
    /// Memory strength tracking
    memory_strengths: HashMap<String, MemoryStrength>,
}

impl ForgettingEngine {
    /// Create a new forgetting engine
    pub fn new(config: ForgettingConfig) -> Self {
        Self {
            config,
            memory_strengths: HashMap::new(),
        }
    }

    /// Process memory forgetting and decay
    pub fn process_forgetting(
        &mut self,
        memories: Vec<Memory>,
    ) -> Result<(Vec<Memory>, ForgettingResult)> {
        let start_time = std::time::Instant::now();
        let processed_count = memories.len();
        let mut retained_memories = Vec::new();
        let mut forgotten_count = 0;
        let mut updated_count = 0;

        let current_time = chrono::Utc::now().timestamp();

        for memory in memories {
            // Get or create strength tracking for this memory
            let memory_id = memory.id.clone();
            let should_forget = {
                let strength = self
                    .memory_strengths
                    .entry(memory_id.clone())
                    .or_insert_with(|| {
                        let mut strength = MemoryStrength::default();
                        strength.protected =
                            self.config.protected_categories.contains(&memory.category);
                        // Use memory timestamp as initial last_access
                        strength.last_access = memory.timestamp as i64;
                        strength
                    });

                // Calculate current strength based on time decay
                let hours_since_access = (current_time - strength.last_access) as f32 / 3600.0;
                
                // Apply importance factor to decay
                // Higher importance factor means slower decay for important memories
                // We assume memory.importance is 0.0-1.0
                let decay_modifier = 1.0 / (1.0 + self.config.importance_factor * memory.importance);
                let effective_decay = self.config.base_decay_rate * decay_modifier;

                strength.current_strength = (strength.initial_strength
                    * (-effective_decay * hours_since_access).exp())
                .max(0.0);

                // Check if memory should be forgotten
                Self::should_forget_static(&self.config, &memory, strength, current_time)
            };

            if should_forget {
                forgotten_count += 1;
                self.memory_strengths.remove(&memory_id);
            } else {
                retained_memories.push(memory);
                updated_count += 1;
            }
        }

        let result = ForgettingResult {
            processed_count,
            forgotten_count,
            updated_count,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        };

        Ok((retained_memories, result))
    }

    /// Check if a memory should be forgotten
    #[allow(dead_code)]
    fn should_forget(&self, memory: &Memory, strength: &MemoryStrength, current_time: i64) -> bool {
        Self::should_forget_static(&self.config, memory, strength, current_time)
    }

    /// Static version of should_forget to avoid borrowing issues
    fn should_forget_static(
        config: &ForgettingConfig,
        memory: &Memory,
        strength: &MemoryStrength,
        current_time: i64,
    ) -> bool {
        // Protected memories are never forgotten
        if strength.protected {
            return false;
        }

        // Check strength threshold
        if strength.current_strength < config.min_importance_threshold {
            return true;
        }

        // Check maximum age
        let age_seconds = (current_time - memory.timestamp as i64) as u64;
        if age_seconds > config.max_memory_age {
            return true;
        }

        false
    }

    /// Strengthen a memory (called when accessed)
    pub fn strengthen_memory(&mut self, memory_id: &str, boost_factor: f32) -> Result<()> {
        if let Some(strength) = self.memory_strengths.get_mut(memory_id) {
            strength.current_strength = (strength.current_strength + boost_factor).min(1.0);
            strength.last_access = chrono::Utc::now().timestamp();
            strength.access_count += 1;
        }
        Ok(())
    }

    /// Get memory strength information
    pub fn get_memory_strength(&self, memory_id: &str) -> Option<&MemoryStrength> {
        self.memory_strengths.get(memory_id)
    }

    /// Set memory as protected from forgetting
    pub fn protect_memory(&mut self, memory_id: &str) -> Result<()> {
        if let Some(strength) = self.memory_strengths.get_mut(memory_id) {
            strength.protected = true;
        }
        Ok(())
    }

    /// Remove protection from a memory
    pub fn unprotect_memory(&mut self, memory_id: &str) -> Result<()> {
        if let Some(strength) = self.memory_strengths.get_mut(memory_id) {
            strength.protected = false;
        }
        Ok(())
    }

    /// Get statistics about memory strengths
    pub fn get_statistics(&self) -> ForgettingStatistics {
        let total_memories = self.memory_strengths.len();
        let protected_memories = self
            .memory_strengths
            .values()
            .filter(|s| s.protected)
            .count();

        let mut strength_sum = 0.0;
        let mut weak_memories = 0;

        for strength in self.memory_strengths.values() {
            strength_sum += strength.current_strength;
            if strength.current_strength < self.config.min_importance_threshold {
                weak_memories += 1;
            }
        }

        let average_strength = if total_memories > 0 {
            strength_sum / total_memories as f32
        } else {
            0.0
        };

        ForgettingStatistics {
            total_memories,
            protected_memories,
            weak_memories,
            average_strength,
        }
    }
}

/// Statistics about memory forgetting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingStatistics {
    /// Total number of tracked memories
    pub total_memories: usize,
    /// Number of protected memories
    pub protected_memories: usize,
    /// Number of weak memories (below retention threshold)
    pub weak_memories: usize,
    /// Average memory strength
    pub average_strength: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forgetting_engine() {
        let config = ForgettingConfig::default();
        let mut engine = ForgettingEngine::new(config);

        let memories = vec![
            Memory {
                id: "1".to_string(),
                text: "Recent memory".to_string(),
                category: MemoryCategory::Social,
                timestamp: chrono::Utc::now().timestamp() as u64,
                importance: 0.5,
                valence: 0.0,
                entities: vec![],
                context: HashMap::new(),
            },
            Memory {
                id: "2".to_string(),
                text: "Old memory".to_string(),
                category: MemoryCategory::Social,
                timestamp: (chrono::Utc::now().timestamp() - 3600) as u64, // 1 hour old
                importance: 0.5,
                valence: 0.0,
                entities: vec![],
                context: HashMap::new(),
            },
        ];

        let (retained, result) = engine.process_forgetting(memories).unwrap();
        assert_eq!(result.processed_count, 2);
        assert!(!retained.is_empty());
    }

    #[test]
    fn test_memory_strengthening() {
        let config = ForgettingConfig::default();
        let mut engine = ForgettingEngine::new(config);

        engine.strengthen_memory("test_id", 0.1).unwrap();
        // Should not panic even if memory doesn't exist
    }
}
