//! Memory forgetting and decay mechanisms
//!
//! This module implements sophisticated forgetting algorithms based on
//! memory strength, importance, and access patterns.

use crate::memory_types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Configuration for forgetting mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingConfig {
    /// Base decay rate for memory strength
    pub base_decay_rate: f32,
    /// Minimum strength for memory retention
    pub retention_threshold: f32,
    /// Importance multiplier for decay rate
    pub importance_factor: f32,
    /// Access frequency impact on forgetting
    pub access_factor: f32,
    /// Enable spaced repetition effects
    pub spaced_repetition: bool,
}

impl Default for ForgettingConfig {
    fn default() -> Self {
        Self {
            base_decay_rate: 0.1,
            retention_threshold: 0.15,
            importance_factor: 0.5,
            access_factor: 0.3,
            spaced_repetition: true,
        }
    }
}

/// Memory forgetting engine
#[derive(Debug)]
pub struct ForgettingEngine {
    config: ForgettingConfig,
    forgetting_curves: HashMap<MemoryType, ForgettingCurve>,
}

impl ForgettingEngine {
    /// Create a new forgetting engine
    pub fn new(config: ForgettingConfig) -> Self {
        let mut forgetting_curves = HashMap::new();

        // Default forgetting curves for different memory types
        forgetting_curves.insert(
            MemoryType::Sensory,
            ForgettingCurve {
                initial_strength: 1.0,
                decay_rate: 2.0,
                half_life: 0.25, // 6 hours
                retention_threshold: 0.1,
                immune: false,
            },
        );

        forgetting_curves.insert(
            MemoryType::Working,
            ForgettingCurve {
                initial_strength: 1.0,
                decay_rate: 1.0,
                half_life: 1.0, // 1 day
                retention_threshold: 0.2,
                immune: false,
            },
        );

        forgetting_curves.insert(
            MemoryType::Episodic,
            ForgettingCurve {
                initial_strength: 1.0,
                decay_rate: 0.2,
                half_life: 14.0, // 2 weeks
                retention_threshold: 0.15,
                immune: false,
            },
        );

        forgetting_curves.insert(
            MemoryType::Semantic,
            ForgettingCurve {
                initial_strength: 1.0,
                decay_rate: 0.05,
                half_life: 180.0, // 6 months
                retention_threshold: 0.1,
                immune: true,
            },
        );

        forgetting_curves.insert(
            MemoryType::Procedural,
            ForgettingCurve {
                initial_strength: 1.0,
                decay_rate: 0.1,
                half_life: 30.0, // 1 month
                retention_threshold: 0.12,
                immune: false,
            },
        );

        forgetting_curves.insert(
            MemoryType::Emotional,
            ForgettingCurve {
                initial_strength: 1.0,
                decay_rate: 0.15,
                half_life: 7.0, // 1 week
                retention_threshold: 0.18,
                immune: false,
            },
        );

        forgetting_curves.insert(
            MemoryType::Social,
            ForgettingCurve {
                initial_strength: 1.0,
                decay_rate: 0.12,
                half_life: 21.0, // 3 weeks
                retention_threshold: 0.15,
                immune: false,
            },
        );

        Self {
            config,
            forgetting_curves,
        }
    }

    /// Apply forgetting to a collection of memories
    pub fn apply_forgetting(&self, memories: &mut Vec<Memory>) -> Result<ForgettingResult> {
        let start_time = std::time::Instant::now();
        let mut result = ForgettingResult::default();
        let mut memories_to_remove = Vec::new();

        for (index, memory) in memories.iter_mut().enumerate() {
            // Skip permanent memories
            if memory.metadata.permanent {
                continue;
            }

            // Update memory strength based on decay
            let old_strength = memory.metadata.strength;
            self.update_memory_strength(memory)?;

            result.memories_processed += 1;
            result.total_strength_lost += old_strength - memory.metadata.strength;

            // Check if memory should be forgotten
            if self.should_forget(memory)? {
                memories_to_remove.push(index);
                result.memories_forgotten += 1;
            }
        }

        // Remove memories in reverse order to maintain indices
        for &index in memories_to_remove.iter().rev() {
            memories.remove(index);
        }

        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Update memory strength based on decay and usage patterns
    fn update_memory_strength(&self, memory: &mut Memory) -> Result<()> {
        let now = Utc::now();
        let age_days = (now - memory.metadata.created_at).num_days() as f32;
        let _last_access_days = (now - memory.metadata.last_accessed).num_days() as f32;

        // Get forgetting curve for this memory type
        let default_curve = ForgettingCurve {
            initial_strength: 1.0,
            decay_rate: self.config.base_decay_rate,
            half_life: 7.0,
            retention_threshold: self.config.retention_threshold,
            immune: false,
        };
        let curve = self
            .forgetting_curves
            .get(&memory.memory_type)
            .unwrap_or(&default_curve);

        // Calculate base decay
        let decay_factor = if curve.half_life > 0.0 {
            (-0.693 * age_days / curve.half_life).exp()
        } else {
            (-curve.decay_rate * age_days).exp()
        };

        // Apply importance modifier
        let importance_modifier = 1.0 + (memory.metadata.importance - 0.5) * self.config.importance_factor;

        // Apply access frequency modifier
        let access_modifier = if memory.metadata.access_count > 0 {
            let access_frequency = memory.metadata.access_count as f32 / age_days.max(1.0);
            1.0 + (access_frequency * self.config.access_factor)
        } else {
            1.0
        };

        // Apply spaced repetition effects
        let spaced_repetition_modifier = if self.config.spaced_repetition && memory.metadata.access_count > 1 {
            // Memories that are accessed multiple times decay more slowly
            let repetition_factor = (memory.metadata.access_count as f32).ln() * 0.1;
            1.0 + repetition_factor
        } else {
            1.0
        };

        // Calculate new strength
        let new_strength = curve.initial_strength * decay_factor * importance_modifier * access_modifier * spaced_repetition_modifier;

        memory.metadata.strength = new_strength.max(0.0).min(1.0);

        Ok(())
    }

    /// Determine if a memory should be forgotten
    fn should_forget(&self, memory: &Memory) -> Result<bool> {
        // Never forget permanent memories
        if memory.metadata.permanent {
            return Ok(false);
        }

        // Check against type-specific curve
        if let Some(curve) = self.forgetting_curves.get(&memory.memory_type) {
            if curve.immune {
                return Ok(false);
            }
            return Ok(memory.metadata.strength < curve.retention_threshold);
        }

        // Default threshold check
        Ok(memory.metadata.strength < self.config.retention_threshold)
    }

    /// Calculate memory half-life based on access patterns
    pub fn calculate_adaptive_half_life(&self, memory: &Memory) -> f32 {
        let base_half_life = self
            .forgetting_curves
            .get(&memory.memory_type)
            .map(|curve| curve.half_life)
            .unwrap_or(7.0);

        // Adjust based on access frequency
        let access_modifier = if memory.metadata.access_count > 1 {
            let access_factor = (memory.metadata.access_count as f32).ln();
            1.0 + (access_factor * 0.5)
        } else {
            1.0
        };

        // Adjust based on importance
        let importance_modifier = 0.5 + memory.metadata.importance;

        base_half_life * access_modifier * importance_modifier
    }

    /// Get forgetting statistics for a memory type
    pub fn get_type_statistics(&self, memory_type: &MemoryType, memories: &[Memory]) -> TypeForgettingStats {
        let type_memories: Vec<_> = memories
            .iter()
            .filter(|m| &m.memory_type == memory_type)
            .collect();

        if type_memories.is_empty() {
            return TypeForgettingStats::default();
        }

        let total_strength: f32 = type_memories.iter().map(|m| m.metadata.strength).sum();
        let avg_strength = total_strength / type_memories.len() as f32;

        let weak_memories = type_memories
            .iter()
            .filter(|m| {
                let threshold = self
                    .forgetting_curves
                    .get(memory_type)
                    .map(|curve| curve.retention_threshold)
                    .unwrap_or(self.config.retention_threshold);
                m.metadata.strength < threshold
            })
            .count();

        let avg_age_days = type_memories
            .iter()
            .map(|m| (Utc::now() - m.metadata.created_at).num_days() as f32)
            .sum::<f32>()
            / type_memories.len() as f32;

        TypeForgettingStats {
            memory_type: memory_type.clone(),
            total_memories: type_memories.len(),
            average_strength: avg_strength,
            weak_memories,
            average_age_days: avg_age_days,
        }
    }
}

/// Result of forgetting operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForgettingResult {
    /// Number of memories processed
    pub memories_processed: usize,
    /// Number of memories forgotten
    pub memories_forgotten: usize,
    /// Total strength lost across all memories
    pub total_strength_lost: f32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Statistics for memory forgetting by type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeForgettingStats {
    pub memory_type: MemoryType,
    pub total_memories: usize,
    pub average_strength: f32,
    pub weak_memories: usize,
    pub average_age_days: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forgetting_engine_creation() {
        let config = ForgettingConfig::default();
        let engine = ForgettingEngine::new(config);
        assert!(engine.forgetting_curves.contains_key(&MemoryType::Sensory));
    }

    #[test]
    fn test_memory_strength_decay() {
        let engine = ForgettingEngine::new(ForgettingConfig::default());
        let mut memory = Memory::sensory("Test memory".to_string(), None);

        // Artificially age the memory
        memory.metadata.created_at = Utc::now() - chrono::Duration::days(30);

        let initial_strength = memory.metadata.strength;
        engine.update_memory_strength(&mut memory).unwrap();

        assert!(memory.metadata.strength < initial_strength);
    }

    #[test]
    fn test_permanent_memory_protection() {
        let engine = ForgettingEngine::new(ForgettingConfig::default());
        let mut memory = Memory::semantic("Important fact".to_string(), "facts".to_string());
        memory.metadata.permanent = true;
        memory.metadata.strength = 0.05; // Very weak

        let should_forget = engine.should_forget(&memory).unwrap();
        assert!(!should_forget);
    }

    #[test]
    fn test_spaced_repetition_effect() {
        let config = ForgettingConfig {
            spaced_repetition: true,
            ..Default::default()
        };
        let engine = ForgettingEngine::new(config);

        let mut memory1 = Memory::episodic("Test".to_string(), vec![], None);
        let mut memory2 = Memory::episodic("Test".to_string(), vec![], None);

        // Age both memories equally
        let old_date = Utc::now() - chrono::Duration::days(10);
        memory1.metadata.created_at = old_date;
        memory2.metadata.created_at = old_date;

        // Memory2 has been accessed multiple times
        memory2.metadata.access_count = 5;

        engine.update_memory_strength(&mut memory1).unwrap();
        engine.update_memory_strength(&mut memory2).unwrap();

        // Memory2 should be stronger due to spaced repetition
        assert!(memory2.metadata.strength > memory1.metadata.strength);
    }
}