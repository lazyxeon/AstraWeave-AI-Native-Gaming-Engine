//! P2: Boundary Condition Tests for astraweave-net
//!
//! Focuses on edge cases for snapshot/delta logic and message serialization.

#![cfg(test)]

use astraweave_net::{
    apply_delta, diff_snapshots, filter_snapshot_for_viewer, Delta, EntityState, FullInterest, Msg,
    Snapshot,
};
use astraweave_core::IVec2;

fn empty_snapshot(tick: u64) -> Snapshot {
    Snapshot {
        version: 1,
        tick,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: Vec::new(),
    }
}

fn viewer() -> EntityState {
    EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    }
}

#[test]
fn test_diff_snapshots_empty_collections_no_changes() {
    let base = empty_snapshot(0);
    let head = empty_snapshot(1);
    let v = viewer();
    let interest = FullInterest;

    let delta = diff_snapshots(&base, &head, &interest, &v);
    assert_eq!(delta.base_tick, 0);
    assert_eq!(delta.tick, 1);
    assert!(delta.changed.is_empty());
    // With empty head.entities, present is empty => removed contains base ids (none)
    assert!(delta.removed.is_empty());
}

#[test]
fn test_apply_delta_tick_mismatch_no_change() {
    let mut base = empty_snapshot(5);
    let delta = Delta {
        base_tick: 4,
        tick: 6,
        changed: Vec::new(),
        removed: Vec::new(),
        head_hash: 123,
    };

    apply_delta(&mut base, &delta);
    assert_eq!(base.tick, 5, "tick should remain unchanged on mismatch");
}

#[test]
fn test_filter_snapshot_for_viewer_empty_entities_stays_empty() {
    let head = empty_snapshot(10);
    let v = viewer();
    let interest = FullInterest;

    let filtered = filter_snapshot_for_viewer(&head, &interest, &v);
    assert!(filtered.entities.is_empty());
}

#[test]
fn test_msg_clienthello_empty_name_roundtrip() {
    let msg = Msg::ClientHello {
        name: "".to_string(),
        token: None,
        policy: None,
    };

    let json = serde_json::to_string(&msg).expect("serialize");
    let decoded: Msg = serde_json::from_str(&json).expect("deserialize");

    match decoded {
        Msg::ClientHello { name, token, policy } => {
            assert_eq!(name, "");
            assert!(token.is_none());
            assert!(policy.is_none());
        }
        other => panic!("unexpected decoded msg: {other:?}"),
    }
}

#[test]
fn test_msg_clienthello_very_long_name_roundtrip() {
    let name = "n".repeat(100_000);
    let msg = Msg::ClientHello {
        name: name.clone(),
        token: Some("t".repeat(128)),
        policy: Some("radius".into()),
    };

    let json = serde_json::to_string(&msg).expect("serialize");
    let decoded: Msg = serde_json::from_str(&json).expect("deserialize");

    match decoded {
        Msg::ClientHello { name: n2, token, policy } => {
            assert_eq!(n2.len(), name.len());
            assert_eq!(token.unwrap().len(), 128);
            assert_eq!(policy.unwrap(), "radius");
        }
        other => panic!("unexpected decoded msg: {other:?}"),
    }
}

#[test]
fn test_diff_and_apply_delta_large_collection_does_not_panic() {
    let v = viewer();
    let interest = FullInterest;

    let mut base = empty_snapshot(0);
    base.entities = (0..1000)
        .map(|id| EntityState {
            id,
            pos: IVec2 { x: id as i32, y: 0 },
            hp: 100,
            team: (id % 3) as u8,
            ammo: 0,
        })
        .collect();

    let mut head = base.clone();
    head.tick = 1;
    // Change a few entities
    for e in head.entities.iter_mut().take(10) {
        e.hp = i32::MAX;
        e.ammo = i32::MIN;
    }

    let delta = diff_snapshots(&base, &head, &interest, &v);
    assert!(!delta.changed.is_empty(), "expected some deltas");

    let mut applied = base.clone();
    apply_delta(&mut applied, &delta);
    assert_eq!(applied.tick, head.tick);
}
