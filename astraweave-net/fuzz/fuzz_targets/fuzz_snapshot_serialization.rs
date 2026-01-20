//! Fuzz target for snapshot serialization roundtrip.
//!
//! Ensures serialize -> deserialize produces identical data.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use astraweave_core::IVec2;
use astraweave_net::{EntityState, Snapshot};

#[derive(Debug, Arbitrary)]
struct FuzzSnapshot {
    tick: u64,
    t: f32,
    seq: u32,
    world_hash: u64,
    entities: Vec<FuzzEntity>,
}

#[derive(Debug, Clone, Arbitrary)]
struct FuzzEntity {
    id: u32,
    x: i32,
    y: i32,
    hp: i32,
    team: u8,
    ammo: i32,
}

fn to_snapshot(input: &FuzzSnapshot) -> Snapshot {
    Snapshot {
        version: 1,
        tick: input.tick,
        t: if input.t.is_finite() { input.t } else { 0.0 },
        seq: input.seq,
        world_hash: input.world_hash,
        entities: input
            .entities
            .iter()
            .map(|e| EntityState {
                id: e.id,
                pos: IVec2 { x: e.x, y: e.y },
                hp: e.hp,
                team: e.team,
                ammo: e.ammo,
            })
            .collect(),
    }
}

fuzz_target!(|input: FuzzSnapshot| {
    let snapshot = to_snapshot(&input);

    // Test bincode roundtrip
    if let Ok(encoded) = bincode::serialize(&snapshot) {
        if let Ok(decoded) = bincode::deserialize::<Snapshot>(&encoded) {
            assert_eq!(snapshot.version, decoded.version);
            assert_eq!(snapshot.tick, decoded.tick);
            assert_eq!(snapshot.seq, decoded.seq);
            assert_eq!(snapshot.world_hash, decoded.world_hash);
            assert_eq!(snapshot.entities.len(), decoded.entities.len());
            
            for (orig, dec) in snapshot.entities.iter().zip(decoded.entities.iter()) {
                assert_eq!(orig.id, dec.id);
                assert_eq!(orig.pos, dec.pos);
                assert_eq!(orig.hp, dec.hp);
                assert_eq!(orig.team, dec.team);
                assert_eq!(orig.ammo, dec.ammo);
            }
        }
    }

    // Test JSON roundtrip
    if let Ok(json) = serde_json::to_string(&snapshot) {
        if let Ok(decoded) = serde_json::from_str::<Snapshot>(&json) {
            assert_eq!(snapshot.version, decoded.version);
            assert_eq!(snapshot.tick, decoded.tick);
            assert_eq!(snapshot.entities.len(), decoded.entities.len());
        }
    }
});
