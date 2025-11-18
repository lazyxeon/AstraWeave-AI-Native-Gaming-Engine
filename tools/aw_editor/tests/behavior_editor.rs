use astraweave_behavior::BehaviorNode;
use astraweave_core::{IVec2, Team, World};
use aw_editor_lib::behavior_graph::{BehaviorGraphDocument, BehaviorGraphNodeKind};
use tempfile::NamedTempFile;

#[test]
fn document_converts_to_runtime_sequence() {
    let mut doc = BehaviorGraphDocument::new_default();
    let root_id = doc.root_id();
    // Transform root into a sequence with two children
    if let Some(root) = doc.node_mut(root_id) {
        root.label = "Root Sequence".into();
        root.kind = BehaviorGraphNodeKind::Sequence {
            children: Vec::new(),
        };
    }

    let condition_id = doc
        .add_child_node(
            root_id,
            "Check Target",
            BehaviorGraphNodeKind::Condition {
                name: "has_target".into(),
            },
        )
        .expect("should add condition child");
    let action_id = doc
        .add_child_node(
            root_id,
            "Fire",
            BehaviorGraphNodeKind::Action {
                name: "attack".into(),
            },
        )
        .expect("should add action child");

    let runtime = doc.to_runtime().expect("document converts to runtime");
    match &runtime.root {
        BehaviorNode::Sequence(children) => {
            assert_eq!(children.len(), 2);
            match &children[0] {
                BehaviorNode::Condition(name) => assert_eq!(name, "has_target"),
                other => panic!("unexpected first child: {:?}", other),
            }
            match &children[1] {
                BehaviorNode::Action(name) => assert_eq!(name, "attack"),
                other => panic!("unexpected second child: {:?}", other),
            }
        }
        other => panic!("unexpected root node: {:?}", other),
    }

    // Ensure round-trip retains structure
    let rebuilt = BehaviorGraphDocument::from_runtime(&runtime);
    let round_trip = rebuilt.to_runtime().expect("round-trip conversion");
    assert!(matches!(round_trip.root, BehaviorNode::Sequence(_)));
    assert!(doc.node(condition_id).is_some());
    assert!(doc.node(action_id).is_some());
}

#[test]
fn document_save_and_load_preserves_data() {
    let mut doc = BehaviorGraphDocument::new_default();
    let path = NamedTempFile::new().expect("temp file");
    doc.save_to_path(path.path()).expect("save document");

    let loaded = BehaviorGraphDocument::load_from_path(path.path()).expect("load document");
    assert!(!loaded.is_dirty());
    let runtime = loaded.to_runtime().expect("runtime conversion");
    match runtime.root {
        BehaviorNode::Action(name) => assert_eq!(name, "idle"),
        other => panic!("unexpected node after load: {:?}", other),
    }
}

#[test]
fn document_assignment_binds_world_component() {
    let doc = BehaviorGraphDocument::new_default();
    let runtime = doc.to_runtime().expect("runtime conversion");
    let mut world = World::new();
    let entity = world.spawn("test", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 10);
    world.set_behavior_graph(entity, runtime);
    assert!(world.behavior_graph(entity).is_some());
}
