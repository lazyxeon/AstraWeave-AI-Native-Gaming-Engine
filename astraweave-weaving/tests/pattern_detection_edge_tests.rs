//! Pattern detection edge case tests for astraweave-weaving
//!
//! These tests validate pattern detection behavior under edge conditions,
//! performance under load, and determinism.

mod common;

use astraweave_weaving::patterns::*;
use common::*;
use std::collections::BTreeMap;

#[test]
fn test_low_health_cluster_boundary_conditions() {
    // Test entities at exactly threshold (25% health)
    let metrics = WorldMetrics {
        avg_health: 0.25,
        critical_health_count: 3, // exactly at min_cluster_size threshold
        resource_scarcity: BTreeMap::new(),
        faction_tensions: BTreeMap::new(),
        recent_damage_events: 0,
        time_since_event: 0.0,
    };

    let detector = create_low_health_detector();
    let patterns = detector.detect(&metrics);

    // Should detect pattern with Moderate strength
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].0, "low_health_cluster");
    assert!(patterns[0].1 > 0.0 && patterns[0].1 <= 1.0);
}

#[test]
fn test_resource_scarcity_gradual_depletion() {
    // Test resource depletion increasing pattern strength over time
    let mut resource_scarcity = BTreeMap::new();
    resource_scarcity.insert("food".to_string(), 0.3); // Below threshold

    let metrics1 = WorldMetrics {
        avg_health: 0.8,
        critical_health_count: 0,
        resource_scarcity: resource_scarcity.clone(),
        faction_tensions: BTreeMap::new(),
        recent_damage_events: 0,
        time_since_event: 0.0,
    };

    let detector = create_resource_scarcity_detector();
    let patterns1 = detector.detect(&metrics1);
    assert_eq!(patterns1.len(), 0, "Should not detect below threshold");

    // Increase scarcity above threshold
    resource_scarcity.insert("food".to_string(), 0.6);
    let metrics2 = WorldMetrics {
        resource_scarcity: resource_scarcity.clone(),
        ..metrics1.clone()
    };

    let patterns2 = detector.detect(&metrics2);
    assert_eq!(patterns2.len(), 1, "Should detect above threshold");
    assert_eq!(patterns2[0].0, "resource_scarce_food");

    // Further increase
    resource_scarcity.insert("food".to_string(), 0.9);
    let metrics3 = WorldMetrics {
        resource_scarcity,
        ..metrics1
    };

    let patterns3 = detector.detect(&metrics3);
    assert_eq!(patterns3.len(), 1);
    assert!(
        patterns3[0].1 > patterns2[0].1,
        "Strength should increase with scarcity"
    );
}

#[test]
fn test_faction_conflict_multi_faction() {
    // Test detection of conflicts between multiple factions
    let mut faction_tensions = BTreeMap::new();
    faction_tensions.insert("rebels".to_string(), 0.7); // Above threshold (0.6)
    faction_tensions.insert("empire".to_string(), 0.8);
    faction_tensions.insert("neutral".to_string(), 0.4); // Below threshold

    let metrics = WorldMetrics {
        avg_health: 0.8,
        critical_health_count: 0,
        resource_scarcity: BTreeMap::new(),
        faction_tensions,
        recent_damage_events: 0,
        time_since_event: 0.0,
    };

    let detector = create_faction_conflict_detector();
    let patterns = detector.detect(&metrics);

    // Should detect 2 patterns (rebels and empire, but not neutral)
    assert_eq!(patterns.len(), 2);
    assert!(patterns
        .iter()
        .any(|(id, _)| id == "faction_conflict_rebels"));
    assert!(patterns
        .iter()
        .any(|(id, _)| id == "faction_conflict_empire"));
    assert!(!patterns.iter().any(|(id, _)| id.contains("neutral")));
}

#[test]
fn test_combat_intensity_spike() {
    // Test rapid intensity increase
    let metrics_low = WorldMetrics {
        avg_health: 0.8,
        critical_health_count: 0,
        resource_scarcity: BTreeMap::new(),
        faction_tensions: BTreeMap::new(),
        recent_damage_events: 5, // Below threshold (10)
        time_since_event: 2.0,
    };

    let detector = create_combat_intensity_detector();
    let patterns_low = detector.detect(&metrics_low);
    assert_eq!(patterns_low.len(), 0, "Should not detect below threshold");

    // Spike to high intensity
    let metrics_high = WorldMetrics {
        recent_damage_events: 15, // Above threshold
        time_since_event: 0.5,    // Recent events
        ..metrics_low
    };

    let patterns_high = detector.detect(&metrics_high);
    assert_eq!(patterns_high.len(), 1, "Should detect intensity spike");
    assert_eq!(patterns_high[0].0, "high_combat_intensity");
}

#[test]
fn test_multi_pattern_simultaneous_detection() {
    // Test multiple patterns detected at once
    let mut resource_scarcity = BTreeMap::new();
    resource_scarcity.insert("water".to_string(), 0.8);

    let mut faction_tensions = BTreeMap::new();
    faction_tensions.insert("faction_a".to_string(), 0.7);

    let metrics = WorldMetrics {
        avg_health: 0.2,
        critical_health_count: 5, // Above min_cluster_size (3)
        resource_scarcity,
        faction_tensions,
        recent_damage_events: 12, // Above threshold (10)
        time_since_event: 1.0,
    };

    // Run all detectors
    let low_health = create_low_health_detector().detect(&metrics);
    let resource = create_resource_scarcity_detector().detect(&metrics);
    let faction = create_faction_conflict_detector().detect(&metrics);
    let combat = create_combat_intensity_detector().detect(&metrics);

    // All patterns should be detected
    assert_eq!(low_health.len(), 1, "Low health pattern");
    assert_eq!(resource.len(), 1, "Resource scarcity pattern");
    assert_eq!(faction.len(), 1, "Faction conflict pattern");
    assert_eq!(combat.len(), 1, "Combat intensity pattern");

    // Total 4 patterns
    let total_patterns = low_health.len() + resource.len() + faction.len() + combat.len();
    assert_eq!(total_patterns, 4);
}

#[test]
fn test_pattern_detection_performance_100_entities() {
    // Simulate metrics from 100 entities
    let mut resource_scarcity = BTreeMap::new();
    for i in 0..10 {
        resource_scarcity.insert(format!("resource_{}", i), 0.6);
    }

    let metrics = WorldMetrics {
        avg_health: 0.5,
        critical_health_count: 10,
        resource_scarcity,
        faction_tensions: BTreeMap::new(),
        recent_damage_events: 20,
        time_since_event: 1.0,
    };

    // Measure detection time (simplified - just verify it completes quickly)
    let start = std::time::Instant::now();

    let detector = create_low_health_detector();
    for _ in 0..100 {
        detector.detect(&metrics);
    }

    let elapsed = start.elapsed();

    // Should complete 100 detections in <10ms (100µs per detection target)
    assert!(
        elapsed.as_millis() < 10,
        "Detection too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_pattern_detection_performance_1000_entities() {
    // Simulate metrics from 1000 entities
    let metrics = WorldMetrics {
        avg_health: 0.4,
        critical_health_count: 100,
        resource_scarcity: BTreeMap::new(),
        faction_tensions: BTreeMap::new(),
        recent_damage_events: 200,
        time_since_event: 0.5,
    };

    let start = std::time::Instant::now();

    let detector = create_low_health_detector();
    for _ in 0..1000 {
        detector.detect(&metrics);
    }

    let elapsed = start.elapsed();

    // Should complete 1000 detections in <100ms (100µs per detection target)
    assert!(
        elapsed.as_millis() < 100,
        "Detection too slow at scale: {:?}",
        elapsed
    );
}

#[test]
fn test_pattern_strength_threshold_exact_match() {
    // Test PatternStrength enum boundaries
    let weak = PatternStrength::from_value(0.29);
    assert_eq!(weak, PatternStrength::Weak);

    let moderate = PatternStrength::from_value(0.30);
    assert_eq!(moderate, PatternStrength::Moderate);

    let moderate_high = PatternStrength::from_value(0.69);
    assert_eq!(moderate_high, PatternStrength::Moderate);

    let strong = PatternStrength::from_value(0.70);
    assert_eq!(strong, PatternStrength::Strong);

    let strong_max = PatternStrength::from_value(1.0);
    assert_eq!(strong_max, PatternStrength::Strong);
}

#[test]
fn test_pattern_strength_thresholds() {
    // Test threshold values
    assert_eq!(PatternStrength::Weak.threshold(), 0.0);
    assert_eq!(PatternStrength::Moderate.threshold(), 0.3);
    assert_eq!(PatternStrength::Strong.threshold(), 0.7);
}

#[test]
fn test_low_health_detector_name() {
    let detector = create_low_health_detector();
    assert_eq!(detector.name(), "low_health_cluster");
}

#[test]
fn test_resource_scarcity_detector_name() {
    let detector = create_resource_scarcity_detector();
    assert_eq!(detector.name(), "resource_scarcity");
}

#[test]
fn test_faction_conflict_detector_name() {
    let detector = create_faction_conflict_detector();
    assert_eq!(detector.name(), "faction_conflict");
}

#[test]
fn test_combat_intensity_detector_name() {
    let detector = create_combat_intensity_detector();
    assert_eq!(detector.name(), "combat_intensity");
}

#[test]
fn test_pattern_detection_determinism() {
    // Pattern detection should be deterministic (no RNG involved)
    let metrics = create_test_metrics(5, 0.3, 15);

    let detector = create_low_health_detector();

    let patterns1 = detector.detect(&metrics);
    let patterns2 = detector.detect(&metrics);
    let patterns3 = detector.detect(&metrics);

    // All runs should produce identical results
    assert_eq!(patterns1.len(), patterns2.len());
    assert_eq!(patterns2.len(), patterns3.len());

    for i in 0..patterns1.len() {
        assert_eq!(patterns1[i].0, patterns2[i].0);
        assert_eq!(patterns1[i].1, patterns2[i].1);
        assert_eq!(patterns2[i].0, patterns3[i].0);
        assert_eq!(patterns2[i].1, patterns3[i].1);
    }
}
