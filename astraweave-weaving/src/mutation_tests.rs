//! Mutation-resistant tests for the weaving/fate-weaving system.
//!
//! These tests are designed to catch common mutations in pattern detection,
//! echo currency, anchors, and intent processing.

use crate::patterns::{PatternStrength, WorldMetrics};
use crate::echo_currency::{EchoCurrency, TransactionReason};
use crate::anchor::{Anchor, AbilityType, AnchorVfxState};
use crate::{CWeaveAgent, CWeaveSignal};

// ============================================================================
// PatternStrength Tests
// ============================================================================

mod pattern_strength_tests {
    use super::*;

    #[test]
    fn test_weak_threshold() {
        assert_eq!(PatternStrength::from_value(0.0), PatternStrength::Weak);
        assert_eq!(PatternStrength::from_value(0.1), PatternStrength::Weak);
        assert_eq!(PatternStrength::from_value(0.29), PatternStrength::Weak);
    }

    #[test]
    fn test_moderate_threshold() {
        assert_eq!(PatternStrength::from_value(0.3), PatternStrength::Moderate);
        assert_eq!(PatternStrength::from_value(0.5), PatternStrength::Moderate);
        assert_eq!(PatternStrength::from_value(0.69), PatternStrength::Moderate);
    }

    #[test]
    fn test_strong_threshold() {
        assert_eq!(PatternStrength::from_value(0.7), PatternStrength::Strong);
        assert_eq!(PatternStrength::from_value(0.9), PatternStrength::Strong);
        assert_eq!(PatternStrength::from_value(1.0), PatternStrength::Strong);
    }

    #[test]
    fn test_threshold_values() {
        assert_eq!(PatternStrength::Weak.threshold(), 0.0);
        assert_eq!(PatternStrength::Moderate.threshold(), 0.3);
        assert_eq!(PatternStrength::Strong.threshold(), 0.7);
    }

    #[test]
    fn test_pattern_strength_equality() {
        assert_eq!(PatternStrength::Weak, PatternStrength::Weak);
        assert_ne!(PatternStrength::Weak, PatternStrength::Moderate);
        assert_ne!(PatternStrength::Moderate, PatternStrength::Strong);
    }
}

// ============================================================================
// WorldMetrics Tests
// ============================================================================

mod world_metrics_tests {
    use super::*;

    #[test]
    fn test_world_metrics_default() {
        let metrics = WorldMetrics::default();
        
        assert_eq!(metrics.avg_health, 0.0);
        assert_eq!(metrics.critical_health_count, 0);
        assert!(metrics.resource_scarcity.is_empty());
        assert!(metrics.faction_tensions.is_empty());
    }

    #[test]
    fn test_world_metrics_with_health() {
        let mut metrics = WorldMetrics::default();
        metrics.avg_health = 0.75;
        metrics.critical_health_count = 3;
        
        assert_eq!(metrics.avg_health, 0.75);
        assert_eq!(metrics.critical_health_count, 3);
    }

    #[test]
    fn test_world_metrics_with_resources() {
        let mut metrics = WorldMetrics::default();
        metrics.resource_scarcity.insert("wood".to_string(), 0.8);
        metrics.resource_scarcity.insert("stone".to_string(), 0.2);
        
        assert_eq!(metrics.resource_scarcity.len(), 2);
        assert_eq!(*metrics.resource_scarcity.get("wood").unwrap(), 0.8);
    }
}

// ============================================================================
// EchoCurrency Tests
// ============================================================================

mod echo_currency_tests {
    use super::*;

    #[test]
    fn test_echo_new_starts_zero() {
        let currency = EchoCurrency::new();
        assert_eq!(currency.count(), 0);
    }

    #[test]
    fn test_echo_with_balance() {
        let currency = EchoCurrency::with_balance(10);
        assert_eq!(currency.count(), 10);
    }

    #[test]
    fn test_echo_add() {
        let mut currency = EchoCurrency::new();
        currency.add(5, TransactionReason::TutorialReward);
        assert_eq!(currency.count(), 5);
    }

    #[test]
    fn test_echo_add_multiple() {
        let mut currency = EchoCurrency::new();
        currency.add(2, TransactionReason::TutorialReward);
        currency.add(3, TransactionReason::KillRiftStalker);
        assert_eq!(currency.count(), 5);
    }

    #[test]
    fn test_echo_spend_success() {
        let mut currency = EchoCurrency::with_balance(10);
        let success = currency.spend(3, TransactionReason::RepairAnchor("test".into()));
        
        assert!(success);
        assert_eq!(currency.count(), 7);
    }

    #[test]
    fn test_echo_spend_insufficient() {
        let mut currency = EchoCurrency::with_balance(2);
        let success = currency.spend(5, TransactionReason::RepairAnchor("test".into()));
        
        assert!(!success);
        assert_eq!(currency.count(), 2, "Balance should not change on failed spend");
    }

    #[test]
    fn test_echo_spend_exact() {
        let mut currency = EchoCurrency::with_balance(5);
        let success = currency.spend(5, TransactionReason::RepairAnchor("test".into()));
        
        assert!(success);
        assert_eq!(currency.count(), 0);
    }

    #[test]
    fn test_echo_has() {
        let currency = EchoCurrency::with_balance(5);
        
        assert!(currency.has(5));
        assert!(currency.has(3));
        assert!(currency.has(0));
        assert!(!currency.has(6));
    }

    #[test]
    fn test_echo_transaction_log() {
        let mut currency = EchoCurrency::new();
        currency.add(2, TransactionReason::TutorialReward);
        currency.add(1, TransactionReason::KillRiftStalker);
        
        assert_eq!(currency.transaction_count(), 2);
    }
}

// ============================================================================
// Anchor Tests
// ============================================================================

mod anchor_tests {
    use super::*;

    #[test]
    fn test_anchor_creation() {
        let anchor = Anchor::new(1.0, 5, None);
        
        assert_eq!(anchor.stability(), 1.0);
        assert_eq!(anchor.repair_cost(), 5);
        assert!(anchor.unlocks_ability().is_none());
    }

    #[test]
    fn test_anchor_with_ability() {
        let anchor = Anchor::new(0.7, 2, Some(AbilityType::EchoDash));
        
        assert_eq!(anchor.unlocks_ability(), Some(AbilityType::EchoDash));
    }

    #[test]
    fn test_anchor_vfx_perfect() {
        let anchor = Anchor::new(1.0, 5, None);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Perfect);
    }

    #[test]
    fn test_anchor_vfx_stable() {
        let anchor = Anchor::new(0.8, 5, None);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
    }

    #[test]
    fn test_anchor_vfx_unstable() {
        let anchor = Anchor::new(0.5, 5, None);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Unstable);
    }

    #[test]
    fn test_anchor_vfx_critical() {
        let anchor = Anchor::new(0.2, 5, None);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Critical);
    }

    #[test]
    fn test_anchor_vfx_broken() {
        let anchor = Anchor::new(0.0, 5, None);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Broken);
    }

    #[test]
    fn test_anchor_decay() {
        let mut anchor = Anchor::new(1.0, 5, None);
        
        // Simulate 60 seconds of decay (call apply_decay 60 times)
        for _ in 0..60 {
            anchor.apply_decay(1.0);
        }
        
        // Should have decayed by ~0.01
        assert!(anchor.stability() < 1.0);
        assert!(anchor.stability() > 0.98);
    }

    #[test]
    fn test_anchor_repair() {
        let mut anchor = Anchor::new(0.5, 5, None);
        
        anchor.repair();
        
        // Should increase by REPAIR_BONUS (0.3)
        assert!((anchor.stability() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_anchor_repair_clamped() {
        let mut anchor = Anchor::new(0.9, 5, None);
        
        anchor.repair();
        
        // Should not exceed 1.0
        assert!((anchor.stability() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_anchor_combat_stress() {
        let mut anchor = Anchor::new(1.0, 5, None);
        
        anchor.apply_combat_stress();
        
        // Should decrease by COMBAT_STRESS_DECAY (0.05)
        assert!((anchor.stability() - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_anchor_stability_boundary() {
        let broken = Anchor::new(0.0, 5, None);
        let perfect = Anchor::new(1.0, 5, None);
        let damaged = Anchor::new(0.5, 5, None);
        
        assert_eq!(broken.stability(), 0.0, "Broken anchor has 0 stability");
        assert_eq!(perfect.stability(), 1.0, "Perfect anchor has 1.0 stability");
        assert_eq!(damaged.stability(), 0.5, "Damaged anchor has 0.5 stability");
    }
}

// ============================================================================
// AbilityType Tests
// ============================================================================

mod ability_type_tests {
    use super::*;

    #[test]
    fn test_ability_type_echo_dash() {
        let ability = AbilityType::EchoDash;
        assert_eq!(format!("{}", ability), "Echo Dash");
    }

    #[test]
    fn test_ability_type_barricade_deploy() {
        let ability = AbilityType::BarricadeDeploy;
        assert_eq!(format!("{}", ability), "Barricade Deploy");
    }

    #[test]
    fn test_ability_type_equality() {
        assert_eq!(AbilityType::EchoDash, AbilityType::EchoDash);
        assert_ne!(AbilityType::EchoDash, AbilityType::BarricadeDeploy);
    }

    #[test]
    fn test_ability_type_copy_clone() {
        let original = AbilityType::EchoDash;
        let copied = original;
        let cloned = original.clone();
        
        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }
}

// ============================================================================
// CWeaveAgent Tests
// ============================================================================

mod weave_agent_tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = CWeaveAgent::new(1.0);
        
        assert!(agent.patterns_detected.is_empty());
        assert_eq!(agent.scan_interval, 1.0);
        assert_eq!(agent.last_scan, 0.0);
    }

    #[test]
    fn test_agent_should_scan_initial() {
        let agent = CWeaveAgent::new(1.0);
        
        // At time 0, should NOT scan since 0.0 - 0.0 = 0.0 < 1.0 interval
        assert!(!agent.should_scan(0.0), "Should not scan at time 0 with interval 1.0");
        // At time 1.0, should scan since 1.0 - 0.0 >= 1.0
        assert!(agent.should_scan(1.0), "Should scan when interval elapsed");
        // At time 2.0, should also scan
        assert!(agent.should_scan(2.0), "Should scan after interval");
    }

    #[test]
    fn test_agent_should_scan_after_interval() {
        let mut agent = CWeaveAgent::new(2.0);
        agent.last_scan = 5.0;
        
        assert!(!agent.should_scan(6.0), "Should not scan before interval");
        assert!(agent.should_scan(7.0), "Should scan at interval");
        assert!(agent.should_scan(10.0), "Should scan after interval");
    }
}

// ============================================================================
// CWeaveSignal Tests
// ============================================================================

mod weave_signal_tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let signal = CWeaveSignal {
            kind: "test_signal".to_string(),
            strength: 0.75,
            seed: 12345,
            metadata: std::collections::BTreeMap::new(),
        };
        
        assert_eq!(signal.kind, "test_signal");
        assert_eq!(signal.strength, 0.75);
        assert_eq!(signal.seed, 12345);
    }

    #[test]
    fn test_signal_with_metadata() {
        let mut metadata = std::collections::BTreeMap::new();
        metadata.insert("source".to_string(), "combat".to_string());
        
        let signal = CWeaveSignal {
            kind: "event".to_string(),
            strength: 1.0,
            seed: 42,
            metadata,
        };
        
        assert_eq!(signal.metadata.get("source"), Some(&"combat".to_string()));
    }
}

// ============================================================================
// Behavioral Correctness Tests
// ============================================================================

mod behavioral_tests {
    use super::*;

    #[test]
    fn test_echo_earn_and_spend_cycle() {
        let mut currency = EchoCurrency::new();
        
        // Earn from combat
        currency.add(4, TransactionReason::KillRiftStalker);
        currency.add(2, TransactionReason::KillSentinel);
        assert_eq!(currency.count(), 6);
        
        // Spend on anchor
        assert!(currency.spend(2, TransactionReason::RepairAnchor("test".into())));
        assert_eq!(currency.count(), 4);
        
        // Spend on ability
        assert!(currency.spend(1, TransactionReason::UseEchoDash));
        assert_eq!(currency.count(), 3);
    }

    #[test]
    fn test_anchor_decay_and_repair_cycle() {
        let mut anchor = Anchor::new(0.7, 2, Some(AbilityType::EchoDash));
        
        // Decay over time (simulate 120 seconds)
        for _ in 0..120 {
            anchor.apply_decay(1.0);
        }
        let decayed_stability = anchor.stability();
        assert!(decayed_stability < 0.7);
        
        // Repair
        anchor.repair();
        assert!(anchor.stability() > decayed_stability);
    }

    #[test]
    fn test_pattern_strength_boundary_values() {
        // Test exact boundary values
        assert_eq!(PatternStrength::from_value(0.29999), PatternStrength::Weak);
        assert_eq!(PatternStrength::from_value(0.30), PatternStrength::Moderate);
        assert_eq!(PatternStrength::from_value(0.69999), PatternStrength::Moderate);
        assert_eq!(PatternStrength::from_value(0.70), PatternStrength::Strong);
    }

    #[test]
    fn test_anchor_stability_clamping() {
        let mut anchor = Anchor::new(0.05, 1, None);
        
        // Multiple repairs should not exceed 1.0
        anchor.repair();
        anchor.repair();
        anchor.repair();
        anchor.repair();
        
        assert!(anchor.stability() <= 1.0);
    }

    #[test]
    fn test_echo_transaction_count_increments() {
        let mut currency = EchoCurrency::new();
        
        assert_eq!(currency.transaction_count(), 0);
        
        currency.add(1, TransactionReason::TutorialReward);
        assert_eq!(currency.transaction_count(), 1);
        
        currency.add(1, TransactionReason::KillRiftStalker);
        assert_eq!(currency.transaction_count(), 2);
        
        currency.spend(1, TransactionReason::RepairAnchor("test".into()));
        assert_eq!(currency.transaction_count(), 3);
    }
}
