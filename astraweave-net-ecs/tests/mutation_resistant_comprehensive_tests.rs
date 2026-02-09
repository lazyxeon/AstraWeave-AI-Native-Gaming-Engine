//! Mutation-resistant comprehensive tests for astraweave-net-ecs.

use astraweave_ecs::Plugin;
use astraweave_net_ecs::*;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// CNetworkClient field-level tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn network_client_fields() {
    let nc = CNetworkClient {
        player_id: "player_42".into(),
        last_acknowledged_input: 100,
        pending_inputs: vec![101, 102, 103],
    };
    assert_eq!(nc.player_id, "player_42");
    assert_eq!(nc.last_acknowledged_input, 100);
    assert_eq!(nc.pending_inputs.len(), 3);
    assert_eq!(nc.pending_inputs[0], 101);
    assert_eq!(nc.pending_inputs[1], 102);
    assert_eq!(nc.pending_inputs[2], 103);
}

#[test]
fn network_client_empty_pending() {
    let nc = CNetworkClient {
        player_id: "p".into(),
        last_acknowledged_input: 0,
        pending_inputs: vec![],
    };
    assert!(nc.pending_inputs.is_empty());
    assert_eq!(nc.last_acknowledged_input, 0);
}

#[test]
fn network_client_clone() {
    let nc = CNetworkClient {
        player_id: "hero".into(),
        last_acknowledged_input: 50,
        pending_inputs: vec![51],
    };
    let nc2 = nc.clone();
    assert_eq!(nc2.player_id, "hero");
    assert_eq!(nc2.last_acknowledged_input, 50);
    assert_eq!(nc2.pending_inputs, vec![51]);
}

#[test]
fn network_client_debug() {
    let nc = CNetworkClient {
        player_id: "dbg".into(),
        last_acknowledged_input: 0,
        pending_inputs: vec![],
    };
    let dbg = format!("{nc:?}");
    assert!(dbg.contains("CNetworkClient"));
    assert!(dbg.contains("dbg"));
}

// ═══════════════════════════════════════════════════════════════════════════
// CClientPrediction field-level tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn client_prediction_fields() {
    let cp = CClientPrediction {
        predicted_position: glam::Vec3::new(1.0, 2.0, 3.0),
        prediction_error: glam::Vec3::new(0.1, 0.2, 0.3),
    };
    assert!((cp.predicted_position.x - 1.0).abs() < f32::EPSILON);
    assert!((cp.predicted_position.y - 2.0).abs() < f32::EPSILON);
    assert!((cp.predicted_position.z - 3.0).abs() < f32::EPSILON);
    assert!((cp.prediction_error.x - 0.1).abs() < f32::EPSILON);
    assert!((cp.prediction_error.y - 0.2).abs() < f32::EPSILON);
    assert!((cp.prediction_error.z - 0.3).abs() < f32::EPSILON);
}

#[test]
fn client_prediction_zero() {
    let cp = CClientPrediction {
        predicted_position: glam::Vec3::ZERO,
        prediction_error: glam::Vec3::ZERO,
    };
    assert_eq!(cp.predicted_position, glam::Vec3::ZERO);
    assert_eq!(cp.prediction_error, glam::Vec3::ZERO);
}

#[test]
fn client_prediction_clone() {
    let cp = CClientPrediction {
        predicted_position: glam::Vec3::new(5.0, 10.0, 15.0),
        prediction_error: glam::Vec3::new(0.5, 0.5, 0.5),
    };
    let cp2 = cp.clone();
    assert_eq!(cp2.predicted_position, cp.predicted_position);
    assert_eq!(cp2.prediction_error, cp.prediction_error);
}

// ═══════════════════════════════════════════════════════════════════════════
// CNetworkAuthority field-level tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn network_authority_fields() {
    let na = CNetworkAuthority {
        authoritative_tick: 999,
        connected_clients: HashMap::new(),
    };
    assert_eq!(na.authoritative_tick, 999);
    assert!(na.connected_clients.is_empty());
}

#[test]
fn network_authority_tick_zero() {
    let na = CNetworkAuthority {
        authoritative_tick: 0,
        connected_clients: HashMap::new(),
    };
    assert_eq!(na.authoritative_tick, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// NetworkSnapshot field-level tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn network_snapshot_empty() {
    let ns = NetworkSnapshot {
        server_tick: 100,
        entity_states: HashMap::new(),
    };
    assert_eq!(ns.server_tick, 100);
    assert!(ns.entity_states.is_empty());
}

#[test]
fn network_snapshot_with_entities() {
    let mut states = HashMap::new();
    states.insert(1, EntityState {
        position: glam::Vec3::new(1.0, 2.0, 3.0),
        health: 100,
    });
    states.insert(2, EntityState {
        position: glam::Vec3::new(4.0, 5.0, 6.0),
        health: 50,
    });
    let ns = NetworkSnapshot {
        server_tick: 42,
        entity_states: states,
    };
    assert_eq!(ns.server_tick, 42);
    assert_eq!(ns.entity_states.len(), 2);
    assert_eq!(ns.entity_states[&1].health, 100);
    assert_eq!(ns.entity_states[&2].health, 50);
}

#[test]
fn network_snapshot_clone() {
    let mut states = HashMap::new();
    states.insert(1, EntityState {
        position: glam::Vec3::ZERO,
        health: 75,
    });
    let ns = NetworkSnapshot {
        server_tick: 10,
        entity_states: states,
    };
    let ns2 = ns.clone();
    assert_eq!(ns2.server_tick, 10);
    assert_eq!(ns2.entity_states[&1].health, 75);
}

#[test]
fn network_snapshot_json_roundtrip() {
    let mut states = HashMap::new();
    states.insert(7, EntityState {
        position: glam::Vec3::new(3.14, 2.72, 1.41),
        health: 88,
    });
    let ns = NetworkSnapshot {
        server_tick: 55,
        entity_states: states,
    };
    let json = serde_json::to_string(&ns).unwrap();
    let back: NetworkSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(back.server_tick, 55);
    assert_eq!(back.entity_states[&7].health, 88);
}

// ═══════════════════════════════════════════════════════════════════════════
// EntityState (net-ecs version) field-level tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn entity_state_fields() {
    let es = EntityState {
        position: glam::Vec3::new(10.0, 20.0, 30.0),
        health: 100,
    };
    assert!((es.position.x - 10.0).abs() < f32::EPSILON);
    assert!((es.position.y - 20.0).abs() < f32::EPSILON);
    assert!((es.position.z - 30.0).abs() < f32::EPSILON);
    assert_eq!(es.health, 100);
}

#[test]
fn entity_state_negative_health() {
    let es = EntityState {
        position: glam::Vec3::ZERO,
        health: -10,
    };
    assert_eq!(es.health, -10);
}

#[test]
fn entity_state_clone() {
    let es = EntityState {
        position: glam::Vec3::new(1.0, 2.0, 3.0),
        health: 42,
    };
    let es2 = es.clone();
    assert_eq!(es2.health, 42);
    assert_eq!(es2.position, es.position);
}

#[test]
fn entity_state_json_roundtrip() {
    let es = EntityState {
        position: glam::Vec3::new(1.5, 2.5, 3.5),
        health: 50,
    };
    let json = serde_json::to_string(&es).unwrap();
    let back: EntityState = serde_json::from_str(&json).unwrap();
    assert_eq!(back.health, 50);
    assert!((back.position.x - 1.5).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════
// NetworkClientPlugin and NetworkServerPlugin
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn network_client_plugin_new() {
    let _plugin = NetworkClientPlugin::new("ws://localhost:9000".into());
    // Should not panic
}

#[test]
fn network_server_plugin_new() {
    let _plugin = NetworkServerPlugin::new("0.0.0.0:9000".into());
    // Should not panic
}

#[test]
fn network_client_plugin_build() {
    let plugin = NetworkClientPlugin::new("ws://localhost:9000".into());
    let mut app = astraweave_ecs::App::new();
    plugin.build(&mut app);
    // Should register systems without panic
}

#[test]
fn network_server_plugin_build() {
    let plugin = NetworkServerPlugin::new("0.0.0.0:9000".into());
    let mut app = astraweave_ecs::App::new();
    plugin.build(&mut app);
    // Should register systems without panic
}

// ═══════════════════════════════════════════════════════════════════════════
// System functions (basic call tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn client_input_system_on_empty_world() {
    let mut world = astraweave_ecs::World::new();
    client_input_system(&mut world);
    // Should not panic on empty world
}

#[test]
fn client_reconciliation_system_on_empty_world() {
    let mut world = astraweave_ecs::World::new();
    client_reconciliation_system(&mut world);
    // Should not panic on empty world
}

#[test]
fn server_snapshot_system_on_empty_world() {
    let mut world = astraweave_ecs::World::new();
    server_snapshot_system(&mut world);
    // Should not panic on empty world
}

// ═══════════════════════════════════════════════════════════════════════════
// Boundary conditions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn entity_state_max_health() {
    let es = EntityState {
        position: glam::Vec3::ZERO,
        health: i32::MAX,
    };
    assert_eq!(es.health, i32::MAX);
}

#[test]
fn network_snapshot_max_tick() {
    let ns = NetworkSnapshot {
        server_tick: u64::MAX,
        entity_states: HashMap::new(),
    };
    assert_eq!(ns.server_tick, u64::MAX);
}

#[test]
fn network_client_max_ack() {
    let nc = CNetworkClient {
        player_id: "".into(),
        last_acknowledged_input: u64::MAX,
        pending_inputs: vec![],
    };
    assert_eq!(nc.last_acknowledged_input, u64::MAX);
}
