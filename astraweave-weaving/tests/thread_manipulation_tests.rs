//! Thread manipulation tests for the weaving system
//! 
//! Tests intent adjudication, budget constraints, cooldown enforcement, and
//! priority-based decision making. These tests validate the core weaving
//! system's ability to manage budget-limited emergent behaviors through
//! deterministic adjudication.

mod common;

#[test]
fn test_intent_budget_check() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();

    // Approve cheap intent
    let cheap_intent = common::create_test_intent("cheap", 0.8, 5);
    let approved = adjudicator.adjudicate(vec![cheap_intent]);
    assert_eq!(approved.len(), 1, "Cheap intent should be approved within budget");

    // Reject expensive intent (exceeds budget)
    adjudicator.begin_tick();
    let expensive_intent = common::create_test_intent("expensive", 0.9, 25);
    let approved = adjudicator.adjudicate(vec![expensive_intent]);
    assert_eq!(approved.len(), 0, "Expensive intent (25) should exceed budget (20) and be rejected");
}

#[test]
fn test_multiple_intent_budget_allocation() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();
    
    let intents = vec![
        common::create_test_intent("intent_a", 0.9, 8),
        common::create_test_intent("intent_b", 0.7, 7),
        common::create_test_intent("intent_c", 0.5, 6),
    ];
    
    let approved = adjudicator.adjudicate(intents);
    
    // Budget = 20, so can fit intent_a (8) + intent_b (7) = 15
    let total_cost: u32 = approved.iter().map(|i| i.cost).sum();
    assert!(total_cost <= 20, "Total cost should not exceed budget");
    assert!(approved.len() >= 2, "Should approve at least 2 intents within budget");
    
    // Higher priority intents should be approved first
    assert!(approved.iter().any(|i| i.kind == "intent_a"), "Highest priority intent should be approved");
}

#[test]
fn test_intent_cooldown_enforcement() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();
    
    let intent = common::create_test_intent("test_action", 0.9, 10)
        .with_cooldown("test_cooldown");
    
    // First approval should succeed
    let approved = adjudicator.adjudicate(vec![intent.clone()]);
    assert_eq!(approved.len(), 1, "First intent should be approved");
    assert!(adjudicator.is_on_cooldown("test_cooldown"), "Cooldown should be active");
    
    // Second approval should fail (on cooldown)
    adjudicator.begin_tick();
    let intent2 = common::create_test_intent("test_action_2", 0.9, 10)
        .with_cooldown("test_cooldown");
    let approved2 = adjudicator.adjudicate(vec![intent2]);
    assert_eq!(approved2.len(), 0, "Second intent should be blocked by cooldown");
}

#[test]
fn test_budget_constraints_during_combat() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();
    
    let combat_intents = vec![
        common::create_test_intent("emergency_heal", 0.95, 12),
        common::create_test_intent("attack_boost", 0.85, 8),
        common::create_test_intent("shield_buff", 0.75, 6),
    ];
    
    let approved = adjudicator.adjudicate(combat_intents);
    
    // Budget = 20: heal (12) + attack (8) = 20 (full budget)
    assert!(approved.len() >= 2, "Should approve at least emergency heal + attack boost");
    assert_eq!(adjudicator.budget_remaining(), 0, "Budget should be fully allocated");
}

#[test]
fn test_intent_priority_ordering() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();
    
    let intents = vec![
        common::create_test_intent("low_priority", 0.4, 5),
        common::create_test_intent("high_priority", 0.9, 5),
        common::create_test_intent("medium_priority", 0.6, 5),
    ];
    
    let approved = adjudicator.adjudicate(intents);
    
    // All fit within budget (3 × 5 = 15 < 20)
    assert_eq!(approved.len(), 3, "All intents should be approved within budget");
    
    // Verify priority ordering: high → medium → low
    assert_eq!(approved[0].kind, "high_priority");
    assert_eq!(approved[1].kind, "medium_priority");
    assert_eq!(approved[2].kind, "low_priority");
}

#[test]
fn test_budget_reset_per_tick() {
    let mut adjudicator = common::create_test_adjudicator();
    
    // Tick 1: Spend all budget
    adjudicator.begin_tick();
    let intent = common::create_test_intent("expensive", 0.9, 20);
    adjudicator.adjudicate(vec![intent]);
    assert_eq!(adjudicator.budget_remaining(), 0, "Budget should be exhausted");
    
    // Tick 2: Budget should reset
    adjudicator.begin_tick();
    assert_eq!(adjudicator.budget_remaining(), 20, "Budget should reset to 20 per tick");
}

#[test]
fn test_cooldown_decay_over_ticks() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();
    
    let intent = common::create_test_intent("test", 0.9, 5)
        .with_cooldown("test_cooldown");
    
    // Activate cooldown
    adjudicator.adjudicate(vec![intent]);
    let initial_cooldown = adjudicator.cooldown_remaining("test_cooldown");
    assert!(initial_cooldown > 0, "Cooldown should be active");
    
    // Advance 3 ticks
    for _ in 0..3 {
        adjudicator.begin_tick();
    }
    
    let remaining = adjudicator.cooldown_remaining("test_cooldown");
    assert!(remaining < initial_cooldown, "Cooldown should decay over ticks");
}

#[test]
fn test_multi_intent_simultaneous_processing() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();
    
    let intents = vec![
        common::create_test_intent("intent_1", 0.8, 5),
        common::create_test_intent("intent_2", 0.7, 5),
        common::create_test_intent("intent_3", 0.6, 5),
        common::create_test_intent("intent_4", 0.5, 5),
        common::create_test_intent("intent_5", 0.4, 5),
    ];
    
    let approved = adjudicator.adjudicate(intents);
    
    // Budget = 20, can fit 4 intents (4 × 5 = 20)
    assert_eq!(approved.len(), 4, "Should approve exactly 4 intents to fill budget");
    assert_eq!(adjudicator.budget_spent(), 20, "Budget should be fully utilized");
}

#[test]
fn test_min_priority_filter() {
    let mut adjudicator = common::create_test_adjudicator();
    adjudicator.begin_tick();
    
    let intents = vec![
        common::create_test_intent("above_threshold", 0.8, 5),
        common::create_test_intent("below_threshold", 0.2, 5), // Below min_priority (0.3)
    ];
    
    let approved = adjudicator.adjudicate(intents);
    
    // Only above_threshold should be approved (0.8 >= 0.3)
    assert_eq!(approved.len(), 1, "Only above-threshold intent should be approved");
    assert!(!approved.iter().any(|i| i.kind == "below_threshold"), "Below threshold intent should be filtered");
}

#[test]
fn test_deterministic_intent_ordering() {
    // Run adjudication twice with identical inputs
    let mut adj1 = common::create_test_adjudicator();
    adj1.begin_tick();
    
    let mut adj2 = common::create_test_adjudicator();
    adj2.begin_tick();
    
    let intents = vec![
        common::create_test_intent("intent_a", 0.9, 8),
        common::create_test_intent("intent_b", 0.7, 7),
        common::create_test_intent("intent_c", 0.5, 6),
    ];
    
    let approved1 = adj1.adjudicate(intents.clone());
    let approved2 = adj2.adjudicate(intents.clone());
    
    // Results should be identical
    assert_eq!(approved1.len(), approved2.len(), "Approval count should be deterministic");
    for i in 0..approved1.len() {
        assert_eq!(approved1[i].kind, approved2[i].kind, "Intent order should be deterministic");
        assert_eq!(approved1[i].priority, approved2[i].priority);
        assert_eq!(approved1[i].cost, approved2[i].cost);
    }
}
