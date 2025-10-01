use astraweave_behavior::{
    BehaviorContext, BehaviorGraph, BehaviorNode, BehaviorStatus, DecoratorType,
};

#[test]
fn test_sequence_success() {
    let mut context = BehaviorContext::new();
    context.register_action("action1", || BehaviorStatus::Success);
    context.register_action("action2", || BehaviorStatus::Success);

    let graph = BehaviorGraph::new(BehaviorNode::Sequence(vec![
        BehaviorNode::Action("action1".to_string()),
        BehaviorNode::Action("action2".to_string()),
    ]));

    assert_eq!(graph.tick(&context), BehaviorStatus::Success);
}

#[test]
fn test_sequence_failure() {
    let mut context = BehaviorContext::new();
    context.register_action("action1", || BehaviorStatus::Success);
    context.register_action("action2", || BehaviorStatus::Failure);

    let graph = BehaviorGraph::new(BehaviorNode::Sequence(vec![
        BehaviorNode::Action("action1".to_string()),
        BehaviorNode::Action("action2".to_string()),
    ]));

    assert_eq!(graph.tick(&context), BehaviorStatus::Failure);
}

#[test]
fn test_selector_success() {
    let mut context = BehaviorContext::new();
    context.register_action("action1", || BehaviorStatus::Failure);
    context.register_action("action2", || BehaviorStatus::Success);

    let graph = BehaviorGraph::new(BehaviorNode::Selector(vec![
        BehaviorNode::Action("action1".to_string()),
        BehaviorNode::Action("action2".to_string()),
    ]));

    assert_eq!(graph.tick(&context), BehaviorStatus::Success);
}

#[test]
fn test_decorator_inverter() {
    let mut context = BehaviorContext::new();
    context.register_action("action", || BehaviorStatus::Success);

    let graph = BehaviorGraph::new(BehaviorNode::Decorator(
        DecoratorType::Inverter,
        Box::new(BehaviorNode::Action("action".to_string())),
    ));

    assert_eq!(graph.tick(&context), BehaviorStatus::Failure);
}

#[test]
fn test_condition() {
    let mut context = BehaviorContext::new();
    context.register_condition("cond", || true);

    let graph = BehaviorGraph::new(BehaviorNode::Condition("cond".to_string()));

    assert_eq!(graph.tick(&context), BehaviorStatus::Success);
}

#[test]
fn test_condition_false() {
    let mut context = BehaviorContext::new();
    context.register_condition("cond_false", || false);

    let graph = BehaviorGraph::new(BehaviorNode::Condition("cond_false".to_string()));

    assert_eq!(graph.tick(&context), BehaviorStatus::Failure);
}
