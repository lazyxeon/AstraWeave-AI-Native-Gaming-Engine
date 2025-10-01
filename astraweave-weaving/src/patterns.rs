//! Pattern detection for emergent behavior

use std::collections::BTreeMap;

/// A detected pattern in the world
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: String,
    pub strength: f32,
    pub metadata: BTreeMap<String, String>,
}

/// Pattern strength categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternStrength {
    Weak,
    Moderate,
    Strong,
}

impl PatternStrength {
    pub fn from_value(value: f32) -> Self {
        if value < 0.3 {
            PatternStrength::Weak
        } else if value < 0.7 {
            PatternStrength::Moderate
        } else {
            PatternStrength::Strong
        }
    }

    pub fn threshold(&self) -> f32 {
        match self {
            PatternStrength::Weak => 0.0,
            PatternStrength::Moderate => 0.3,
            PatternStrength::Strong => 0.7,
        }
    }
}

/// Trait for pattern detectors
pub trait PatternDetector: Send + Sync {
    /// Detect patterns in the world state
    /// Returns list of (pattern_id, strength) tuples
    fn detect(&self, metrics: &WorldMetrics) -> Vec<(String, f32)>;

    /// Get the name of this detector
    fn name(&self) -> &str;
}

/// World metrics used for pattern detection
#[derive(Debug, Clone, Default)]
pub struct WorldMetrics {
    /// Average health across all entities (0.0 to 1.0)
    pub avg_health: f32,
    /// Number of entities with critical health (<= 20%)
    pub critical_health_count: usize,
    /// Resource scarcity metrics by type
    pub resource_scarcity: BTreeMap<String, f32>,
    /// Faction tension levels
    pub faction_tensions: BTreeMap<String, f32>,
    /// Recent damage events count
    pub recent_damage_events: usize,
    /// Time since last major event
    pub time_since_event: f32,
}

/// Detector for low health clusters
pub struct LowHealthClusterDetector {
    pub threshold: f32,
    pub min_cluster_size: usize,
}

impl PatternDetector for LowHealthClusterDetector {
    fn detect(&self, metrics: &WorldMetrics) -> Vec<(String, f32)> {
        let mut patterns = Vec::new();

        if metrics.critical_health_count >= self.min_cluster_size {
            let strength = (metrics.critical_health_count as f32 / 10.0).min(1.0);
            patterns.push(("low_health_cluster".to_string(), strength));
        }

        patterns
    }

    fn name(&self) -> &str {
        "low_health_cluster"
    }
}

/// Detector for resource scarcity
pub struct ResourceScarcityDetector {
    pub threshold: f32,
}

impl PatternDetector for ResourceScarcityDetector {
    fn detect(&self, metrics: &WorldMetrics) -> Vec<(String, f32)> {
        let mut patterns = Vec::new();

        for (resource, scarcity) in &metrics.resource_scarcity {
            if *scarcity >= self.threshold {
                patterns.push((format!("resource_scarce_{}", resource), *scarcity));
            }
        }

        patterns
    }

    fn name(&self) -> &str {
        "resource_scarcity"
    }
}

/// Detector for faction conflicts
pub struct FactionConflictDetector {
    pub threshold: f32,
}

impl PatternDetector for FactionConflictDetector {
    fn detect(&self, metrics: &WorldMetrics) -> Vec<(String, f32)> {
        let mut patterns = Vec::new();

        for (faction, tension) in &metrics.faction_tensions {
            if *tension >= self.threshold {
                patterns.push((format!("faction_conflict_{}", faction), *tension));
            }
        }

        patterns
    }

    fn name(&self) -> &str {
        "faction_conflict"
    }
}

/// Detector for combat intensity
pub struct CombatIntensityDetector {
    pub events_threshold: usize,
    pub time_window: f32,
}

impl PatternDetector for CombatIntensityDetector {
    fn detect(&self, metrics: &WorldMetrics) -> Vec<(String, f32)> {
        let mut patterns = Vec::new();

        if metrics.recent_damage_events >= self.events_threshold {
            let strength = (metrics.recent_damage_events as f32
                / (self.events_threshold as f32 * 2.0))
                .min(1.0);
            patterns.push(("high_combat_intensity".to_string(), strength));
        }

        patterns
    }

    fn name(&self) -> &str {
        "combat_intensity"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_strength_from_value() {
        assert_eq!(PatternStrength::from_value(0.1), PatternStrength::Weak);
        assert_eq!(PatternStrength::from_value(0.5), PatternStrength::Moderate);
        assert_eq!(PatternStrength::from_value(0.9), PatternStrength::Strong);
    }

    #[test]
    fn test_low_health_cluster_detection() {
        let detector = LowHealthClusterDetector {
            threshold: 0.2,
            min_cluster_size: 3,
        };

        let mut metrics = WorldMetrics::default();
        metrics.critical_health_count = 5;

        let patterns = detector.detect(&metrics);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].0, "low_health_cluster");
        assert!(patterns[0].1 > 0.0 && patterns[0].1 <= 1.0);
    }

    #[test]
    fn test_low_health_below_threshold() {
        let detector = LowHealthClusterDetector {
            threshold: 0.2,
            min_cluster_size: 3,
        };

        let mut metrics = WorldMetrics::default();
        metrics.critical_health_count = 2; // Below min_cluster_size

        let patterns = detector.detect(&metrics);
        assert_eq!(patterns.len(), 0);
    }

    #[test]
    fn test_resource_scarcity_detection() {
        let detector = ResourceScarcityDetector { threshold: 0.5 };

        let mut metrics = WorldMetrics::default();
        metrics.resource_scarcity.insert("food".to_string(), 0.8);
        metrics.resource_scarcity.insert("water".to_string(), 0.3);

        let patterns = detector.detect(&metrics);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].0, "resource_scarce_food");
        assert_eq!(patterns[0].1, 0.8);
    }

    #[test]
    fn test_faction_conflict_detection() {
        let detector = FactionConflictDetector { threshold: 0.6 };

        let mut metrics = WorldMetrics::default();
        metrics.faction_tensions.insert("red".to_string(), 0.9);
        metrics.faction_tensions.insert("blue".to_string(), 0.4);

        let patterns = detector.detect(&metrics);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].0, "faction_conflict_red");
        assert_eq!(patterns[0].1, 0.9);
    }

    #[test]
    fn test_combat_intensity_detection() {
        let detector = CombatIntensityDetector {
            events_threshold: 10,
            time_window: 5.0,
        };

        let mut metrics = WorldMetrics::default();
        metrics.recent_damage_events = 15;

        let patterns = detector.detect(&metrics);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].0, "high_combat_intensity");
        assert!(patterns[0].1 > 0.0);
    }

    #[test]
    fn test_multiple_detectors() {
        let detectors: Vec<Box<dyn PatternDetector>> = vec![
            Box::new(LowHealthClusterDetector {
                threshold: 0.2,
                min_cluster_size: 3,
            }),
            Box::new(ResourceScarcityDetector { threshold: 0.5 }),
        ];

        let mut metrics = WorldMetrics::default();
        metrics.critical_health_count = 5;
        metrics.resource_scarcity.insert("food".to_string(), 0.8);

        let mut all_patterns = Vec::new();
        for detector in &detectors {
            all_patterns.extend(detector.detect(&metrics));
        }

        assert_eq!(all_patterns.len(), 2);
    }
}
