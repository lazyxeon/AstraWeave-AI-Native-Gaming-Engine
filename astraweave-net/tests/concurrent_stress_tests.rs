//! P2: Concurrent Stress Tests for astraweave-net
//!
//! Focus: Snapshot/delta and message serialization should be safe under
//! moderate concurrent use (no panics, no corrupted state).

#![cfg(test)]

use astraweave_core::IVec2;
use astraweave_net::{apply_delta, diff_snapshots, filter_snapshot_for_viewer, EntityState, FullInterest, Msg, Snapshot};
use std::sync::Arc;

fn snapshot_with_entities(tick: u64, n: u32) -> Snapshot {
    Snapshot {
        version: 1,
        tick,
        t: tick as f32 * 0.016,
        seq: tick as u32,
        world_hash: 0,
        entities: (0..n)
            .map(|id| EntityState {
                id,
                pos: IVec2 { x: id as i32, y: 0 },
                hp: 100,
                team: (id % 3) as u8,
                ammo: 0,
            })
            .collect(),
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
fn test_concurrent_diff_apply_and_serialize_no_panic() {
    let v = viewer();

    let base = Arc::new(snapshot_with_entities(0, 1000));
    let head = Arc::new({
        let mut s = snapshot_with_entities(1, 1000);
        for e in s.entities.iter_mut().take(50) {
            e.hp = 99;
            e.ammo = 1;
        }
        s
    });

    let mut handles = Vec::new();
    #[allow(clippy::clone_on_copy)]
    for t in 0..8 {
        let base = base.clone();
        let head = head.clone();
        let v = v.clone();
        handles.push(std::thread::spawn(move || {
            let interest = FullInterest;
            for i in 0..500 {
                let d = diff_snapshots(&base, &head, &interest, &v);
                let mut applied = (*base).clone();
                apply_delta(&mut applied, &d);
                assert_eq!(applied.tick, head.tick);

                // Exercise interest filtering too.
                let filtered = filter_snapshot_for_viewer(&applied, &interest, &v);
                assert!(filtered.entities.len() <= applied.entities.len());

                // And JSON round-trip.
                let msg = Msg::ClientHello {
                    name: format!("client-{t}-{i}"),
                    token: Some("tok".to_string()),
                    policy: Some("radius".into()),
                };
                let json = serde_json::to_string(&msg).expect("serialize");
                let decoded: Msg = serde_json::from_str(&json).expect("deserialize");
                match decoded {
                    Msg::ClientHello { name, token, policy } => {
                        assert!(!name.is_empty());
                        assert_eq!(token.as_deref(), Some("tok"));
                        assert_eq!(policy.as_deref(), Some("radius"));
                    }
                    other => panic!("unexpected decoded msg: {other:?}"),
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }
}
