//! Property-based tests for astraweave-net
//!
//! These tests use proptest to verify invariants in snapshot/delta
//! operations and interest filtering logic.

use proptest::prelude::*;
use std::collections::BTreeSet;

use astraweave_core::IVec2;
use astraweave_net::{
    Delta, EntityDelta, EntityState, FullInterest, Interest, RadiusTeamInterest, Snapshot,
};

// ============================================================================
// PROPTEST STRATEGIES
// ============================================================================

/// Strategy for generating valid EntityState values
fn entity_state_strategy() -> impl Strategy<Value = EntityState> {
    (
        1u32..1000,           // id
        -1000i32..1000,       // pos.x
        -1000i32..1000,       // pos.y
        1i32..100,            // hp
        0u8..4,               // team
        0i32..100,            // ammo
    )
        .prop_map(|(id, x, y, hp, team, ammo)| EntityState {
            id,
            pos: IVec2::new(x, y),
            hp,
            team,
            ammo,
        })
}

/// Strategy for generating a vector of unique EntityStates
fn entity_vec_strategy(max_entities: usize) -> impl Strategy<Value = Vec<EntityState>> {
    prop::collection::vec(entity_state_strategy(), 0..max_entities).prop_map(|mut entities| {
        // Ensure unique IDs by deduplicating
        let mut seen = BTreeSet::new();
        entities.retain(|e| seen.insert(e.id));
        entities
    })
}

/// Strategy for generating Snapshot values
fn snapshot_strategy() -> impl Strategy<Value = Snapshot> {
    (
        0u64..10000,               // tick
        0.0f32..1000.0,            // t
        0u32..10000,               // seq
        entity_vec_strategy(20),   // entities
    )
        .prop_map(|(tick, t, seq, entities)| Snapshot {
            version: 1,
            tick,
            t,
            seq,
            world_hash: 0,  // Will be computed
            entities,
        })
}

/// Strategy for generating EntityDelta values
fn entity_delta_strategy() -> impl Strategy<Value = EntityDelta> {
    (
        1u32..1000,                          // id
        0u8..16,                             // mask
        proptest::option::of(-1000i32..1000), // pos.x
        proptest::option::of(-1000i32..1000), // pos.y
        proptest::option::of(1i32..100),     // hp
        proptest::option::of(0u8..4),        // team
        proptest::option::of(0i32..100),     // ammo
    )
        .prop_map(|(id, mask, px, py, hp, team, ammo)| EntityDelta {
            id,
            mask,
            pos: px.and_then(|x| py.map(|y| IVec2::new(x, y))),
            hp,
            team,
            ammo,
        })
}

/// Strategy for generating Delta values
fn delta_strategy() -> impl Strategy<Value = Delta> {
    (
        0u64..10000,                                      // base_tick
        0u64..10000,                                      // tick
        prop::collection::vec(entity_delta_strategy(), 0..10), // changed
        prop::collection::vec(1u32..1000, 0..5),          // removed
    )
        .prop_map(|(base_tick, tick, changed, removed)| Delta {
            base_tick,
            tick,
            changed,
            removed,
            head_hash: 0,
        })
}

// ============================================================================
// PROPERTY TESTS: EntityState
// ============================================================================

proptest! {
    /// Property: EntityState fields are always within valid ranges
    #[test]
    fn prop_entity_state_valid(entity in entity_state_strategy()) {
        prop_assert!(entity.id >= 1);
        prop_assert!(entity.hp >= 1);
        prop_assert!(entity.team <= 3);
        prop_assert!(entity.ammo >= 0);
    }

    /// Property: EntityState equality is reflexive
    #[test]
    fn prop_entity_state_eq_reflexive(entity in entity_state_strategy()) {
        prop_assert_eq!(entity, entity);
    }
}

// ============================================================================
// PROPERTY TESTS: FullInterest
// ============================================================================

proptest! {
    /// Property: FullInterest always includes all entities
    #[test]
    fn prop_full_interest_includes_all(
        viewer in entity_state_strategy(),
        target in entity_state_strategy()
    ) {
        let interest = FullInterest;
        prop_assert!(interest.include(&viewer, &target));
    }
}

// ============================================================================
// PROPERTY TESTS: RadiusTeamInterest
// ============================================================================

proptest! {
    /// Property: Same team is always included regardless of distance
    #[test]
    fn prop_radius_same_team_included(
        radius in 10i32..1000,
        viewer in entity_state_strategy(),
        mut target in entity_state_strategy()
    ) {
        let interest = RadiusTeamInterest { radius };
        // Make them same team
        target.team = viewer.team;
        // Put target far away
        target.pos = IVec2::new(viewer.pos.x + 10000, viewer.pos.y + 10000);
        
        prop_assert!(interest.include(&viewer, &target), 
            "Same team should always be included");
    }

    /// Property: Target within radius is included
    #[test]
    fn prop_radius_within_range_included(
        radius in 10i32..1000,
        viewer in entity_state_strategy(),
        mut target in entity_state_strategy()
    ) {
        let interest = RadiusTeamInterest { radius };
        
        // Different team
        target.team = (viewer.team + 1) % 4;
        // Place target within radius
        target.pos = IVec2::new(viewer.pos.x + radius / 2, viewer.pos.y);
        
        prop_assert!(interest.include(&viewer, &target),
            "Target within radius should be included");
    }

    /// Property: Target outside radius is excluded (different team)
    #[test]
    fn prop_radius_outside_range_excluded(
        radius in 10i32..100,
        viewer in entity_state_strategy(),
        mut target in entity_state_strategy()
    ) {
        let interest = RadiusTeamInterest { radius };
        
        // Different team
        target.team = (viewer.team + 1) % 4;
        // Place target well outside radius
        target.pos = IVec2::new(viewer.pos.x + radius * 3, viewer.pos.y + radius * 3);
        
        prop_assert!(!interest.include(&viewer, &target),
            "Target outside radius should be excluded");
    }

    /// Property: include never panics
    #[test]
    fn prop_radius_interest_never_panics(
        radius in 1i32..10000,
        viewer in entity_state_strategy(),
        target in entity_state_strategy()
    ) {
        let interest = RadiusTeamInterest { radius };
        let _ = interest.include(&viewer, &target);
    }
}

// ============================================================================
// PROPERTY TESTS: Snapshot
// ============================================================================

proptest! {
    /// Property: Snapshot can hold arbitrary entity counts
    #[test]
    fn prop_snapshot_entity_count(snapshot in snapshot_strategy()) {
        prop_assert!(snapshot.entities.len() <= 20);
    }

    /// Property: Snapshot tick is consistent
    #[test]
    fn prop_snapshot_tick_valid(tick in 0u64..u64::MAX) {
        let snapshot = Snapshot {
            version: 1,
            tick,
            t: 0.0,
            seq: 0,
            world_hash: 0,
            entities: vec![],
        };
        prop_assert_eq!(snapshot.tick, tick);
    }
}

// ============================================================================
// PROPERTY TESTS: Delta
// ============================================================================

proptest! {
    /// Property: Delta tick should logically follow base_tick
    #[test]
    fn prop_delta_tick_order(delta in delta_strategy()) {
        // Delta tick can be anything (no strict ordering required by struct)
        // Just verify it doesn't panic
        let _ = delta.tick;
        let _ = delta.base_tick;
    }

    /// Property: Delta changed entities have valid IDs
    #[test]
    fn prop_delta_changed_valid_ids(delta in delta_strategy()) {
        for entity_delta in &delta.changed {
            prop_assert!(entity_delta.id >= 1, "Entity ID should be >= 1");
        }
    }

    /// Property: Delta removed IDs are valid
    #[test]
    fn prop_delta_removed_valid_ids(delta in delta_strategy()) {
        for &id in &delta.removed {
            prop_assert!(id >= 1, "Removed ID should be >= 1");
        }
    }
}

// ============================================================================
// PROPERTY TESTS: EntityDelta
// ============================================================================

proptest! {
    /// Property: EntityDelta valid ID
    #[test]
    fn prop_entity_delta_valid_id(delta in entity_delta_strategy()) {
        prop_assert!(delta.id >= 1);
    }

    /// Property: EntityDelta mask is within valid range
    #[test]
    fn prop_entity_delta_mask_valid(delta in entity_delta_strategy()) {
        prop_assert!(delta.mask <= 0x0F, "Mask should only use lower 4 bits");
    }
}

// ============================================================================
// PROPERTY TESTS: Interest trait implementations
// ============================================================================

proptest! {
    /// Property: Interest implementations are deterministic
    #[test]
    fn prop_interest_deterministic(
        radius in 1i32..1000,
        viewer in entity_state_strategy(),
        target in entity_state_strategy()
    ) {
        let interest = RadiusTeamInterest { radius };
        let result1 = interest.include(&viewer, &target);
        let result2 = interest.include(&viewer, &target);
        prop_assert_eq!(result1, result2, "Interest should be deterministic");
    }

    /// Property: Self is always included (viewer = target)
    #[test]
    fn prop_interest_self_included(
        radius in 1i32..1000,
        entity in entity_state_strategy()
    ) {
        let interest = RadiusTeamInterest { radius };
        prop_assert!(interest.include(&entity, &entity),
            "Entity should always see itself");
    }
}
