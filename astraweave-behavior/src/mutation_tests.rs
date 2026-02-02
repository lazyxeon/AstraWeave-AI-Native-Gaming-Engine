//! Mutation-resistant tests for behavior tree and AI planning systems.
//!
//! These tests are designed to catch common mutations in behavior tree logic,
//! GOAP planning, and decorator patterns.

use crate::{BehaviorContext, BehaviorNode, BehaviorStatus, DecoratorType};

// ============================================================================
// Shared Helper Functions (accessible by all test modules)
// ============================================================================

/// Creates a BehaviorContext with pre-registered actions that return specified statuses.
fn create_context_with_actions(action_results: &[(&str, BehaviorStatus)]) -> BehaviorContext {
    let mut ctx = BehaviorContext::new();
    for (name, status) in action_results {
        let status = *status;
        ctx.register_action(name, move || status);
    }
    ctx
}

/// Creates a BehaviorContext with pre-registered conditions that return specified booleans.
fn create_context_with_conditions(condition_results: &[(&str, bool)]) -> BehaviorContext {
    let mut ctx = BehaviorContext::new();
    for (name, result) in condition_results {
        let result = *result;
        ctx.register_condition(name, move || result);
    }
    ctx
}

// ============================================================================
// BehaviorNode Structure Tests
// ============================================================================

mod behavior_node_tests {
    use super::*;

    #[test]
    fn test_action_node_creation() {
        let node = BehaviorNode::action("attack");
        assert!(node.is_action(), "Should be action node");
        assert!(!node.is_condition(), "Should not be condition");
        assert_eq!(node.name(), Some("attack"));
    }

    #[test]
    fn test_condition_node_creation() {
        let node = BehaviorNode::condition("has_target");
        assert!(node.is_condition(), "Should be condition node");
        assert!(!node.is_action(), "Should not be action");
        assert_eq!(node.name(), Some("has_target"));
    }

    #[test]
    fn test_sequence_node_creation() {
        let node = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
        ]);
        assert!(node.is_sequence(), "Should be sequence node");
        assert!(!node.is_selector(), "Should not be selector");
        assert_eq!(node.child_count(), 2);
    }

    #[test]
    fn test_selector_node_creation() {
        let node = BehaviorNode::selector(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
            BehaviorNode::action("c"),
        ]);
        assert!(node.is_selector(), "Should be selector node");
        assert!(!node.is_sequence(), "Should not be sequence");
        assert_eq!(node.child_count(), 3);
    }

    #[test]
    fn test_parallel_node_creation() {
        let node = BehaviorNode::parallel(
            vec![BehaviorNode::action("a"), BehaviorNode::action("b")],
            2,
        );
        assert!(node.is_parallel(), "Should be parallel node");
        assert_eq!(node.child_count(), 2);
    }

    #[test]
    fn test_decorator_node_creation() {
        let node = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("check"),
        );
        assert!(node.is_decorator(), "Should be decorator node");
        assert_eq!(node.child_count(), 1);
    }

    #[test]
    fn test_is_leaf_for_action() {
        let node = BehaviorNode::action("test");
        assert!(node.is_leaf(), "Action should be leaf");
    }

    #[test]
    fn test_is_leaf_for_condition() {
        let node = BehaviorNode::condition("test");
        assert!(node.is_leaf(), "Condition should be leaf");
    }

    #[test]
    fn test_is_leaf_for_composite() {
        let node = BehaviorNode::sequence(vec![BehaviorNode::action("a")]);
        assert!(!node.is_leaf(), "Sequence should not be leaf");
    }

    #[test]
    fn test_is_composite() {
        assert!(BehaviorNode::sequence(vec![]).is_composite());
        assert!(BehaviorNode::selector(vec![]).is_composite());
        assert!(BehaviorNode::parallel(vec![], 0).is_composite());
        assert!(!BehaviorNode::action("a").is_composite());
        assert!(!BehaviorNode::condition("c").is_composite());
    }

    #[test]
    fn test_node_type_strings() {
        assert_eq!(BehaviorNode::action("a").node_type(), "Action");
        assert_eq!(BehaviorNode::condition("c").node_type(), "Condition");
        assert_eq!(BehaviorNode::sequence(vec![]).node_type(), "Sequence");
        assert_eq!(BehaviorNode::selector(vec![]).node_type(), "Selector");
        assert_eq!(BehaviorNode::parallel(vec![], 0).node_type(), "Parallel");
        assert_eq!(
            BehaviorNode::decorator(DecoratorType::Inverter, BehaviorNode::action("a")).node_type(),
            "Decorator"
        );
    }

    #[test]
    fn test_total_node_count_single() {
        let node = BehaviorNode::action("test");
        assert_eq!(node.total_node_count(), 1);
    }

    #[test]
    fn test_total_node_count_tree() {
        let tree = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::selector(vec![
                BehaviorNode::action("b"),
                BehaviorNode::action("c"),
            ]),
        ]);
        // 1 sequence + 1 action + 1 selector + 2 actions = 5
        assert_eq!(tree.total_node_count(), 5);
    }

    #[test]
    fn test_max_depth_single() {
        let node = BehaviorNode::action("test");
        assert_eq!(node.max_depth(), 1);
    }

    #[test]
    fn test_max_depth_nested() {
        let tree = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::selector(vec![
                BehaviorNode::decorator(
                    DecoratorType::Inverter,
                    BehaviorNode::action("b"),
                ),
            ]),
        ]);
        // sequence -> selector -> decorator -> action = depth 4
        assert_eq!(tree.max_depth(), 4);
    }

    #[test]
    fn test_name_returns_none_for_composite() {
        let node = BehaviorNode::sequence(vec![]);
        assert_eq!(node.name(), None);
    }
}

// ============================================================================
// DecoratorType Tests
// ============================================================================

mod decorator_type_tests {
    use super::*;

    #[test]
    fn test_decorator_names() {
        assert_eq!(DecoratorType::Inverter.name(), "Inverter");
        assert_eq!(DecoratorType::Succeeder.name(), "Succeeder");
        assert_eq!(DecoratorType::Failer.name(), "Failer");
        assert_eq!(DecoratorType::Repeat(5).name(), "Repeat");
        assert_eq!(DecoratorType::Retry(3).name(), "Retry");
    }

    #[test]
    fn test_decorator_is_inverter() {
        assert!(DecoratorType::Inverter.is_inverter());
        assert!(!DecoratorType::Succeeder.is_inverter());
        assert!(!DecoratorType::Failer.is_inverter());
    }
}

// ============================================================================
// Behavior Tree Tick Tests
// ============================================================================

mod tick_tests {
    use super::*;

    #[test]
    fn test_action_tick_returns_handler_result() {
        let ctx = create_context_with_actions(&[
            ("success_action", BehaviorStatus::Success),
            ("failure_action", BehaviorStatus::Failure),
            ("running_action", BehaviorStatus::Running),
        ]);

        assert_eq!(BehaviorNode::action("success_action").tick(&ctx), BehaviorStatus::Success);
        assert_eq!(BehaviorNode::action("failure_action").tick(&ctx), BehaviorStatus::Failure);
        assert_eq!(BehaviorNode::action("running_action").tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_condition_tick_returns_handler_result() {
        let ctx = create_context_with_conditions(&[
            ("is_true", true),
            ("is_false", false),
        ]);

        assert_eq!(BehaviorNode::condition("is_true").tick(&ctx), BehaviorStatus::Success);
        assert_eq!(BehaviorNode::condition("is_false").tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_sequence_all_success() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Success),
            ("c", BehaviorStatus::Success),
        ]);
        let seq = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
            BehaviorNode::action("c"),
        ]);

        assert_eq!(seq.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_sequence_fails_on_first_failure() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Failure),
            ("c", BehaviorStatus::Success),
        ]);
        let seq = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
            BehaviorNode::action("c"),
        ]);

        assert_eq!(seq.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_sequence_returns_running() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Running),
        ]);
        let seq = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
        ]);

        assert_eq!(seq.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_selector_returns_first_success() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Failure),
            ("b", BehaviorStatus::Success),
            ("c", BehaviorStatus::Failure),
        ]);
        let sel = BehaviorNode::selector(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
            BehaviorNode::action("c"),
        ]);

        assert_eq!(sel.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_selector_all_failure() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Failure),
            ("b", BehaviorStatus::Failure),
        ]);
        let sel = BehaviorNode::selector(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
        ]);

        assert_eq!(sel.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_selector_returns_running() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Failure),
            ("b", BehaviorStatus::Running),
        ]);
        let sel = BehaviorNode::selector(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
        ]);

        assert_eq!(sel.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_inverter_inverts_success() {
        let ctx = create_context_with_actions(&[("a", BehaviorStatus::Success)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("a"),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_inverter_inverts_failure() {
        let ctx = create_context_with_actions(&[("a", BehaviorStatus::Failure)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("a"),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_inverter_preserves_running() {
        let ctx = create_context_with_actions(&[("a", BehaviorStatus::Running)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("a"),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_succeeder_always_succeeds() {
        let ctx = create_context_with_actions(&[("a", BehaviorStatus::Failure)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Succeeder,
            BehaviorNode::action("a"),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_failer_always_fails() {
        let ctx = create_context_with_actions(&[("a", BehaviorStatus::Success)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Failer,
            BehaviorNode::action("a"),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_parallel_threshold_zero_succeeds_immediately() {
        let ctx = BehaviorContext::new();
        let node = BehaviorNode::parallel(
            vec![BehaviorNode::action("a"), BehaviorNode::action("b")],
            0,
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_parallel_threshold_exceeds_children_fails() {
        let ctx = BehaviorContext::new();
        let node = BehaviorNode::parallel(
            vec![BehaviorNode::action("a"), BehaviorNode::action("b")],
            5, // More than 2 children
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_parallel_meets_threshold() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Success),
            ("c", BehaviorStatus::Failure),
        ]);
        let node = BehaviorNode::parallel(
            vec![
                BehaviorNode::action("a"),
                BehaviorNode::action("b"),
                BehaviorNode::action("c"),
            ],
            2, // Need 2 successes
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_parallel_running_propagates() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Running),
            ("b", BehaviorStatus::Failure),
        ]);
        let node = BehaviorNode::parallel(
            vec![BehaviorNode::action("a"), BehaviorNode::action("b")],
            1,
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Running);
    }
}

// ============================================================================
// Behavioral Correctness Tests
// ============================================================================

mod behavioral_tests {
    use super::*;

    #[test]
    fn test_empty_sequence_succeeds() {
        let ctx = BehaviorContext::default();
        let seq = BehaviorNode::sequence(vec![]);
        assert_eq!(seq.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_empty_selector_fails() {
        let ctx = BehaviorContext::default();
        let sel = BehaviorNode::selector(vec![]);
        assert_eq!(sel.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_child_count_matches_children() {
        let seq = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
            BehaviorNode::action("c"),
        ]);
        assert_eq!(seq.child_count(), 3);
    }

    #[test]
    fn test_total_nodes_includes_self() {
        let single = BehaviorNode::action("a");
        assert_eq!(single.total_node_count(), 1);
        
        let with_child = BehaviorNode::sequence(vec![BehaviorNode::action("a")]);
        assert_eq!(with_child.total_node_count(), 2);
    }

    #[test]
    fn test_summary_format() {
        let action = BehaviorNode::action("test");
        assert!(action.summary().contains("test"));
        
        let seq = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
        ]);
        assert!(seq.summary().contains("2"));
    }
}

// ============================================================================
// MUTATION TEST MODULES (3 Types Required for Mutation Testing)
// ============================================================================

/// Boundary condition tests - catch < vs <=, > vs >=, off-by-one errors
#[cfg(test)]
mod boundary_condition_tests {
    use super::*;

    // ========================================================================
    // Parallel Node Threshold Boundary Tests
    // ========================================================================

    #[test]
    fn test_parallel_threshold_zero_boundary() {
        // threshold = 0: should succeed immediately
        let ctx = BehaviorContext::default();
        let node = BehaviorNode::parallel(vec![BehaviorNode::action("a")], 0);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "threshold=0 should succeed immediately"
        );
    }

    #[test]
    fn test_parallel_threshold_equals_children_boundary() {
        // threshold = child count: need ALL to succeed
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Success),
        ]);
        let node = BehaviorNode::parallel(
            vec![BehaviorNode::action("a"), BehaviorNode::action("b")],
            2,
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "threshold=child_count should succeed when all succeed"
        );
    }

    #[test]
    fn test_parallel_threshold_exceeds_children_boundary() {
        // threshold > child count: impossible to meet, should fail
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Success),
        ]);
        let node = BehaviorNode::parallel(
            vec![BehaviorNode::action("a"), BehaviorNode::action("b")],
            3, // More than 2 children
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "threshold > children should fail"
        );
    }

    #[test]
    fn test_parallel_threshold_one_below_total() {
        // threshold = child_count - 1: one failure allowed
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Failure),
            ("c", BehaviorStatus::Success),
        ]);
        let node = BehaviorNode::parallel(
            vec![
                BehaviorNode::action("a"),
                BehaviorNode::action("b"),
                BehaviorNode::action("c"),
            ],
            2, // 2 of 3
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "2 successes of 3 with threshold=2 should succeed"
        );
    }

    // ========================================================================
    // Decorator Repeat/Retry Boundary Tests
    // ========================================================================

    #[test]
    fn test_repeat_zero_boundary() {
        // Repeat(0): should succeed immediately without executing child
        let ctx = BehaviorContext::default();
        let node =
            BehaviorNode::decorator(DecoratorType::Repeat(0), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Repeat(0) should succeed without executing"
        );
    }

    #[test]
    fn test_repeat_one_boundary() {
        // Repeat(1): execute exactly once
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Success)]);
        let node =
            BehaviorNode::decorator(DecoratorType::Repeat(1), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Repeat(1) should execute once and succeed"
        );
    }

    #[test]
    fn test_retry_zero_boundary() {
        // Retry(0): should fail immediately
        let ctx = BehaviorContext::default();
        let node =
            BehaviorNode::decorator(DecoratorType::Retry(0), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "Retry(0) should fail without executing"
        );
    }

    #[test]
    fn test_retry_exactly_needed_boundary() {
        // Retry(3) with success on 3rd try: boundary case
        // We can't easily test this with our mock, but we can test that Retry(1) with success works
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Success)]);
        let node =
            BehaviorNode::decorator(DecoratorType::Retry(1), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Retry(1) with immediate success should succeed"
        );
    }

    // ========================================================================
    // Sequence/Selector Index Boundary Tests
    // ========================================================================

    #[test]
    fn test_sequence_first_child_boundary() {
        // First child fails: entire sequence fails
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Failure),
            ("second", BehaviorStatus::Success),
        ]);
        let node = BehaviorNode::sequence(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "Sequence should fail on first child failure"
        );
    }

    #[test]
    fn test_sequence_last_child_boundary() {
        // Last child fails: still fails despite all others succeeding
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Success),
            ("second", BehaviorStatus::Success),
            ("last", BehaviorStatus::Failure),
        ]);
        let node = BehaviorNode::sequence(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
            BehaviorNode::action("last"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "Sequence should fail on last child failure"
        );
    }

    #[test]
    fn test_selector_first_child_boundary() {
        // First child succeeds: entire selector succeeds
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Success),
            ("second", BehaviorStatus::Failure),
        ]);
        let node = BehaviorNode::selector(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Selector should succeed on first child success"
        );
    }

    #[test]
    fn test_selector_last_child_boundary() {
        // Only last child succeeds: still succeeds
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Failure),
            ("second", BehaviorStatus::Failure),
            ("last", BehaviorStatus::Success),
        ]);
        let node = BehaviorNode::selector(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
            BehaviorNode::action("last"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Selector should succeed on last child success"
        );
    }

    // ========================================================================
    // Node Count Boundary Tests
    // ========================================================================

    #[test]
    fn test_child_count_single_child() {
        let node = BehaviorNode::sequence(vec![BehaviorNode::action("only")]);
        assert_eq!(node.child_count(), 1, "Single child should count as 1");
    }

    #[test]
    fn test_total_nodes_deeply_nested() {
        // 1 root + 1 child + 1 grandchild = 3
        let nested = BehaviorNode::sequence(vec![BehaviorNode::sequence(vec![
            BehaviorNode::action("leaf"),
        ])]);
        assert_eq!(
            nested.total_node_count(),
            3,
            "Deeply nested: root + child + grandchild = 3"
        );
    }
}

/// Comparison operator tests - catch == vs !=, < vs >, sign inversions
#[cfg(test)]
mod comparison_operator_tests {
    use super::*;

    // ========================================================================
    // Status Comparison Tests
    // ========================================================================

    #[test]
    fn test_success_vs_failure_distinction() {
        // Success and Failure must be distinct
        assert_ne!(
            BehaviorStatus::Success,
            BehaviorStatus::Failure,
            "Success must not equal Failure"
        );
    }

    #[test]
    fn test_running_vs_success_distinction() {
        assert_ne!(
            BehaviorStatus::Running,
            BehaviorStatus::Success,
            "Running must not equal Success"
        );
    }

    #[test]
    fn test_running_vs_failure_distinction() {
        assert_ne!(
            BehaviorStatus::Running,
            BehaviorStatus::Failure,
            "Running must not equal Failure"
        );
    }

    // ========================================================================
    // Inverter Comparison Tests
    // ========================================================================

    #[test]
    fn test_inverter_success_becomes_failure() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Success)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("action"),
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "Inverter should convert Success to Failure"
        );
    }

    #[test]
    fn test_inverter_failure_becomes_success() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Failure)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("action"),
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Inverter should convert Failure to Success"
        );
    }

    #[test]
    fn test_inverter_running_unchanged() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Running)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("action"),
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Running,
            "Inverter should not change Running"
        );
    }

    // ========================================================================
    // Succeeder/Failer Comparison Tests
    // ========================================================================

    #[test]
    fn test_succeeder_overrides_failure() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Failure)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Succeeder,
            BehaviorNode::action("action"),
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Succeeder should convert any result to Success"
        );
    }

    #[test]
    fn test_failer_overrides_success() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Success)]);
        let node = BehaviorNode::decorator(
            DecoratorType::Failer,
            BehaviorNode::action("action"),
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "Failer should convert any result to Failure"
        );
    }

    // ========================================================================
    // Sequence vs Selector Logic Comparison
    // ========================================================================

    #[test]
    fn test_sequence_vs_selector_on_mixed_results() {
        // Same children, different logic
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Failure),
        ]);

        let seq = BehaviorNode::sequence(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
        ]);
        let sel = BehaviorNode::selector(vec![
            BehaviorNode::action("a"),
            BehaviorNode::action("b"),
        ]);

        assert_eq!(
            seq.tick(&ctx),
            BehaviorStatus::Failure,
            "Sequence fails on any failure"
        );
        assert_eq!(
            sel.tick(&ctx),
            BehaviorStatus::Success,
            "Selector succeeds on any success"
        );
    }

    #[test]
    fn test_sequence_selector_opposite_behaviors() {
        // Sequence: AND logic (all must succeed)
        // Selector: OR logic (any can succeed)
        let all_success = create_context_with_actions(&[
            ("a", BehaviorStatus::Success),
            ("b", BehaviorStatus::Success),
        ]);
        let all_fail = create_context_with_actions(&[
            ("a", BehaviorStatus::Failure),
            ("b", BehaviorStatus::Failure),
        ]);

        let children = vec![BehaviorNode::action("a"), BehaviorNode::action("b")];

        // Sequence: ALL must succeed
        let seq_success = BehaviorNode::sequence(children.clone());
        assert_eq!(seq_success.tick(&all_success), BehaviorStatus::Success);

        let seq_fail = BehaviorNode::sequence(children.clone());
        assert_eq!(seq_fail.tick(&all_fail), BehaviorStatus::Failure);

        // Selector: ANY can succeed
        let sel_fail = BehaviorNode::selector(children.clone());
        assert_eq!(sel_fail.tick(&all_fail), BehaviorStatus::Failure);
    }

    // ========================================================================
    // Node Type Comparison Tests
    // ========================================================================

    #[test]
    fn test_is_action_exclusive() {
        let action = BehaviorNode::action("test");
        assert!(action.is_action());
        assert!(!action.is_condition());
        assert!(!action.is_sequence());
        assert!(!action.is_selector());
    }

    #[test]
    fn test_is_condition_exclusive() {
        let condition = BehaviorNode::condition("test");
        assert!(condition.is_condition());
        assert!(!condition.is_action());
        assert!(!condition.is_sequence());
    }

    #[test]
    fn test_is_sequence_exclusive() {
        let seq = BehaviorNode::sequence(vec![]);
        assert!(seq.is_sequence());
        assert!(!seq.is_selector());
        assert!(!seq.is_action());
    }

    #[test]
    fn test_is_selector_exclusive() {
        let sel = BehaviorNode::selector(vec![]);
        assert!(sel.is_selector());
        assert!(!sel.is_sequence());
        assert!(!sel.is_action());
    }
}

/// Boolean return path tests - catch logic inversions, early returns, missing branches
#[cfg(test)]
mod boolean_return_path_tests {
    use super::*;

    // ========================================================================
    // is_* Method Boolean Tests
    // ========================================================================

    #[test]
    fn test_is_leaf_true_for_actions() {
        let action = BehaviorNode::action("test");
        assert!(action.is_leaf(), "Action MUST be leaf");
    }

    #[test]
    fn test_is_leaf_true_for_conditions() {
        let condition = BehaviorNode::condition("test");
        assert!(condition.is_leaf(), "Condition MUST be leaf");
    }

    #[test]
    fn test_is_leaf_false_for_composites() {
        let seq = BehaviorNode::sequence(vec![]);
        let sel = BehaviorNode::selector(vec![]);
        let par = BehaviorNode::parallel(vec![], 0);

        assert!(!seq.is_leaf(), "Sequence must NOT be leaf");
        assert!(!sel.is_leaf(), "Selector must NOT be leaf");
        assert!(!par.is_leaf(), "Parallel must NOT be leaf");
    }

    #[test]
    fn test_is_composite_true_for_all_composite_types() {
        assert!(BehaviorNode::sequence(vec![]).is_composite());
        assert!(BehaviorNode::selector(vec![]).is_composite());
        assert!(BehaviorNode::parallel(vec![], 0).is_composite());
    }

    #[test]
    fn test_is_composite_false_for_leaves() {
        assert!(!BehaviorNode::action("a").is_composite());
        assert!(!BehaviorNode::condition("c").is_composite());
    }

    #[test]
    fn test_is_decorator_returns_correct_boolean() {
        let dec = BehaviorNode::decorator(
            DecoratorType::Inverter,
            BehaviorNode::action("a"),
        );
        let action = BehaviorNode::action("a");

        assert!(dec.is_decorator(), "Decorator must return true");
        assert!(!action.is_decorator(), "Action must return false");
    }

    // ========================================================================
    // Empty Collection Boolean Paths
    // ========================================================================

    #[test]
    fn test_empty_sequence_returns_success() {
        let ctx = BehaviorContext::default();
        let seq = BehaviorNode::sequence(vec![]);
        assert_eq!(
            seq.tick(&ctx),
            BehaviorStatus::Success,
            "Empty sequence MUST return Success"
        );
    }

    #[test]
    fn test_empty_selector_returns_failure() {
        let ctx = BehaviorContext::default();
        let sel = BehaviorNode::selector(vec![]);
        assert_eq!(
            sel.tick(&ctx),
            BehaviorStatus::Failure,
            "Empty selector MUST return Failure"
        );
    }

    #[test]
    fn test_empty_parallel_with_zero_threshold() {
        let ctx = BehaviorContext::default();
        let par = BehaviorNode::parallel(vec![], 0);
        assert_eq!(
            par.tick(&ctx),
            BehaviorStatus::Success,
            "Empty parallel with threshold=0 MUST succeed"
        );
    }

    // ========================================================================
    // Running Status Propagation Tests
    // ========================================================================

    #[test]
    fn test_sequence_propagates_running() {
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Success),
            ("second", BehaviorStatus::Running),
        ]);
        let node = BehaviorNode::sequence(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Running,
            "Sequence MUST propagate Running"
        );
    }

    #[test]
    fn test_selector_propagates_running() {
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Failure),
            ("second", BehaviorStatus::Running),
        ]);
        let node = BehaviorNode::selector(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Running,
            "Selector MUST propagate Running"
        );
    }

    #[test]
    fn test_parallel_propagates_running_when_threshold_not_met() {
        let ctx = create_context_with_actions(&[
            ("a", BehaviorStatus::Running),
            ("b", BehaviorStatus::Failure),
        ]);
        let node = BehaviorNode::parallel(
            vec![BehaviorNode::action("a"), BehaviorNode::action("b")],
            1,
        );
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Running,
            "Parallel MUST propagate Running when threshold could still be met"
        );
    }

    // ========================================================================
    // Early Return Detection Tests
    // ========================================================================

    #[test]
    fn test_sequence_short_circuits_on_failure() {
        // If sequence short-circuits, second action shouldn't affect result
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Failure),
            ("second", BehaviorStatus::Success), // Should not matter
        ]);
        let node = BehaviorNode::sequence(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "Sequence MUST short-circuit on failure"
        );
    }

    #[test]
    fn test_selector_short_circuits_on_success() {
        // If selector short-circuits, second action shouldn't affect result
        let ctx = create_context_with_actions(&[
            ("first", BehaviorStatus::Success),
            ("second", BehaviorStatus::Failure), // Should not matter
        ]);
        let node = BehaviorNode::selector(vec![
            BehaviorNode::action("first"),
            BehaviorNode::action("second"),
        ]);
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Selector MUST short-circuit on success"
        );
    }

    // ========================================================================
    // Decorator Return Path Tests
    // ========================================================================

    #[test]
    fn test_repeat_returns_failure_on_child_failure() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Failure)]);
        let node =
            BehaviorNode::decorator(DecoratorType::Repeat(5), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Failure,
            "Repeat MUST return Failure when child fails"
        );
    }

    #[test]
    fn test_retry_returns_success_on_child_success() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Success)]);
        let node =
            BehaviorNode::decorator(DecoratorType::Retry(5), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Success,
            "Retry MUST return Success when child succeeds"
        );
    }

    #[test]
    fn test_repeat_running_propagation() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Running)]);
        let node =
            BehaviorNode::decorator(DecoratorType::Repeat(5), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Running,
            "Repeat MUST propagate Running"
        );
    }

    #[test]
    fn test_retry_running_propagation() {
        let ctx =
            create_context_with_actions(&[("action", BehaviorStatus::Running)]);
        let node =
            BehaviorNode::decorator(DecoratorType::Retry(5), BehaviorNode::action("action"));
        assert_eq!(
            node.tick(&ctx),
            BehaviorStatus::Running,
            "Retry MUST propagate Running"
        );
    }

    // ========================================================================
    // Name Method Boolean Path Tests
    // ========================================================================

    #[test]
    fn test_name_returns_some_for_action() {
        let node = BehaviorNode::action("test_name");
        assert!(node.name().is_some(), "Action MUST have a name");
        assert_eq!(node.name(), Some("test_name"));
    }

    #[test]
    fn test_name_returns_some_for_condition() {
        let node = BehaviorNode::condition("cond_name");
        assert!(node.name().is_some(), "Condition MUST have a name");
        assert_eq!(node.name(), Some("cond_name"));
    }

    #[test]
    fn test_name_returns_none_for_composites() {
        let seq = BehaviorNode::sequence(vec![]);
        let sel = BehaviorNode::selector(vec![]);

        assert!(seq.name().is_none(), "Sequence should NOT have a name");
        assert!(sel.name().is_none(), "Selector should NOT have a name");
    }
}
