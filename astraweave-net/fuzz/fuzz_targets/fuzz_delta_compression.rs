//! Fuzz target for delta compression/decompression.
//!
//! Tests that diff_snapshots and apply_delta are robust with arbitrary inputs.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use astraweave_core::IVec2;
use astraweave_net::{apply_delta, diff_snapshots, EntityState, Snapshot};

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    base_tick: u64,
    target_tick: u64,
    entities_base: Vec<FuzzEntity>,
    entities_target: Vec<FuzzEntity>,
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

impl From<&FuzzEntity> for EntityState {
    fn from(e: &FuzzEntity) -> Self {
        EntityState {
            id: e.id,
            pos: IVec2 { x: e.x, y: e.y },
            hp: e.hp,
            team: e.team,
            ammo: e.ammo,
        }
    }
}

fn build_snapshot(tick: u64, entities: &[FuzzEntity]) -> Snapshot {
    Snapshot {
        version: 1,
        tick,
        t: tick as f32 / 60.0,
        seq: tick as u32,
        world_hash: 0,
        entities: entities.iter().map(EntityState::from).collect(),
    }
}

fuzz_target!(|input: FuzzInput| {
    // Build base and target snapshots
    let base = build_snapshot(input.base_tick, &input.entities_base);
    let target = build_snapshot(input.target_tick, &input.entities_target);

    // Generate delta between snapshots
    let delta = diff_snapshots(&base, &target);

    // Apply delta back to base
    let reconstructed = apply_delta(&base, &delta);

    // Verify reconstruction matches target for entities that exist in both
    // Note: IDs that don't exist in base can't be fully reconstructed
    for target_entity in &target.entities {
        if let Some(recon_entity) = reconstructed.entities.iter().find(|e| e.id == target_entity.id)
        {
            if input.entities_base.iter().any(|e| e.id == target_entity.id) {
                assert_eq!(
                    recon_entity.pos, target_entity.pos,
                    "Position mismatch for entity {}", target_entity.id
                );
                assert_eq!(
                    recon_entity.hp, target_entity.hp,
                    "HP mismatch for entity {}", target_entity.id
                );
            }
        }
    }
});
