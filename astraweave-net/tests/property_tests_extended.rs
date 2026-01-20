use astraweave_core::*;
use astraweave_net::*;
use proptest::prelude::*;

/// Property-based tests for astraweave-net using proptest
/// These tests verify invariants hold across arbitrary inputs

// Strategy for generating valid EntityState
fn entity_state_strategy() -> impl Strategy<Value = EntityState> {
    (any::<u32>(), any::<i32>(), any::<i32>(), 0u8..=10, any::<i32>())
        .prop_map(|(id, x, y, team, ammo)| EntityState {
            id,
            pos: IVec2 { x: x % 1000, y: y % 1000 }, // Keep positions reasonable
            hp: 0.max(100.min(ammo.abs())), // HP between 0-100
            team,
            ammo: 0.max(ammo.abs() % 100), // Ammo between 0-99
        })
}

// Strategy for generating snapshots
fn snapshot_strategy() -> impl Strategy<Value = Snapshot> {
    (
        any::<u64>(),
        any::<f32>(),
        any::<u32>(),
        prop::collection::vec(entity_state_strategy(), 0..50),
    )
        .prop_map(|(tick, t, seq, entities)| {
            let world_hash = entities.iter().map(|e| e.id as u64).sum();
            Snapshot {
                version: 1,
                tick,
                t: t.abs() % 1000.0,
                seq,
                world_hash,
                entities,
            }
        })
}

proptest! {
    /// Property: Applying a delta to a snapshot and then diffing should be lossless
    /// Invariant: apply(diff(A, B)) on A should yield B (roundtrip)
    #[test]
    fn prop_delta_roundtrip(
        snap1 in snapshot_strategy(),
        snap2 in snapshot_strategy()
    ) {
        let viewer = EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 10,
        };
        
        let interest = FullInterest;
        // Create delta from snap1 to snap2
        let delta = diff_snapshots(&snap1, &snap2, &interest, &viewer);
        
        // Apply delta to snap1
        let mut reconstructed = snap1.clone();
        apply_delta(&mut reconstructed, &delta);
        
        // Verify tick and hash are updated
        prop_assert_eq!(reconstructed.tick, snap2.tick);
        prop_assert_eq!(reconstructed.world_hash, delta.head_hash);
        
        // Entity count should match (accounting for removals)
        let expected_count = snap2.entities.len();
        prop_assert_eq!(reconstructed.entities.len(), expected_count);
    }

    /// Property: Delta with no changes should produce empty changed/removed lists
    #[test]
    fn prop_identical_snapshots_empty_delta(snap in snapshot_strategy()) {
        let viewer = EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 10,
        };
        
        let interest = FullInterest;
        let delta = diff_snapshots(&snap, &snap, &interest, &viewer);
        
        // Identical snapshots should produce no changes
        prop_assert!(delta.changed.is_empty() || delta.changed.iter().all(|d| d.mask == 0));
        prop_assert!(delta.removed.is_empty());
    }

    /// Property: RadiusTeamInterest is symmetric for teammates
    /// Invariant: If A and B are on same team, include(A, B) == include(B, A)
    #[test]
    fn prop_radius_team_interest_teammate_symmetry(
        e1 in entity_state_strategy(),
        e2 in entity_state_strategy(),
        radius in 1i32..100
    ) {
        let interest = RadiusTeamInterest { radius };
        
        let mut teammate1 = e1;
        let mut teammate2 = e2;
        teammate1.team = 1;
        teammate2.team = 1;
        
        let include_1_2 = interest.include(&teammate1, &teammate2);
        let include_2_1 = interest.include(&teammate2, &teammate1);
        
        // Symmetry: both directions should be true for teammates
        prop_assert_eq!(include_1_2, true);
        prop_assert_eq!(include_2_1, true);
    }

    /// Property: FovInterest respects radius constraint
    /// Invariant: Entity outside radius is NEVER visible (regardless of angle)
    #[test]
    fn prop_fov_interest_radius_constraint(
        viewer in entity_state_strategy(),
        target in entity_state_strategy(),
        radius in 1i32..50
    ) {
        let interest = FovInterest {
            radius,
            half_angle_deg: 180.0, // Full 360° FOV
            facing: IVec2 { x: 1, y: 0 },
        };
        
        let mut enemy_viewer = viewer;
        let mut enemy_target = target;
        enemy_viewer.team = 1;
        enemy_target.team = 2;
        
        let dx = enemy_target.pos.x - enemy_viewer.pos.x;
        let dy = enemy_target.pos.y - enemy_viewer.pos.y;
        let dist_sq = dx * dx + dy * dy;
        
        let included = interest.include(&enemy_viewer, &enemy_target);
        
        // If outside radius, must NOT be included
        if dist_sq > radius * radius {
            prop_assert!(!included);
        }
    }

    /// Property: FullInterest always includes everything
    /// Invariant: FullInterest.include() always returns true
    #[test]
    fn prop_full_interest_always_true(
        viewer in entity_state_strategy(),
        target in entity_state_strategy()
    ) {
        let interest = FullInterest;
        prop_assert!(interest.include(&viewer, &target));
    }

    /// Property: Delta base_tick and tick are ordered
    /// Invariant: delta.base_tick <= delta.tick (time moves forward)
    #[test]
    fn prop_delta_tick_ordering(
        snap1 in snapshot_strategy(),
        snap2 in snapshot_strategy()
    ) {
        let viewer = EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 10,
        };
        
        let delta = diff_snapshots(&snap1, &snap2, &FullInterest, &viewer);
        
        prop_assert_eq!(delta.base_tick, snap1.tick);
        prop_assert_eq!(delta.tick, snap2.tick);
    }

    /// Property: Removing entities decreases entity count
    /// Invariant: After apply_delta with removals, count decreases
    #[test]
    fn prop_remove_decreases_count(
        mut snap in snapshot_strategy(),
        remove_ids in prop::collection::vec(any::<u32>(), 0..10)
    ) {
        let initial_count = snap.entities.len();
        
        let delta = Delta {
            base_tick: snap.tick,
            tick: snap.tick + 1,
            changed: vec![],
            removed: remove_ids,
            head_hash: 12345,
        };
        
        apply_delta(&mut snap, &delta);
        
        let final_count = snap.entities.len();
        
        // Count should decrease or stay same (if IDs don't exist)
        prop_assert!(final_count <= initial_count);
    }

    /// Property: Entity updates preserve ID
    /// Invariant: Updating an entity never changes its ID
    #[test]
    fn prop_entity_update_preserves_id(
        mut snap in snapshot_strategy(),
        target_id in any::<u32>(),
        new_pos_x in any::<i32>(),
        new_pos_y in any::<i32>()
    ) {
        // Only test if entity exists
        if !snap.entities.iter().any(|e| e.id == target_id) {
            return Ok(());
        }
        
        let delta = Delta {
            base_tick: snap.tick,
            tick: snap.tick + 1,
            changed: vec![EntityDelta {
                id: target_id,
                mask: 0b0001, // Only POS
                pos: Some(IVec2 { x: new_pos_x % 1000, y: new_pos_y % 1000 }),
                hp: None,
                team: None,
                ammo: None,
            }],
            removed: vec![],
            head_hash: 12345,
        };
        
        apply_delta(&mut snap, &delta);
        
        // Entity should still exist with same ID
        let entity = snap.entities.iter().find(|e| e.id == target_id);
        prop_assert!(entity.is_some());
        if let Some(e) = entity {
            prop_assert_eq!(e.id, target_id);
        }
    }

    /// Property: filter_snapshot_for_viewer respects interest
    /// Invariant: Filtered snapshot never contains entities excluded by interest
    #[test]
    fn prop_filter_respects_interest(
        snap in snapshot_strategy(),
        viewer in entity_state_strategy(),
        radius in 1i32..50
    ) {
        let interest = RadiusTeamInterest { radius };
        
        let filtered = filter_snapshot_for_viewer(&snap, &interest, &viewer);
        
        // All entities in filtered snapshot must pass interest check
        for entity in &filtered.entities {
            let included = interest.include(&viewer, entity);
            prop_assert!(included, "Entity {:?} should not be in filtered snapshot", entity.id);
        }
    }

    /// Property: Snapshot version is preserved
    /// Invariant: Delta application doesn't change snapshot version
    #[test]
    fn prop_delta_preserves_version(
        mut snap in snapshot_strategy(),
        delta_entities in prop::collection::vec(any::<u32>(), 0..5)
    ) {
        let original_version = snap.version;
        
        let delta = Delta {
            base_tick: snap.tick,
            tick: snap.tick + 1,
            changed: vec![],
            removed: delta_entities,
            head_hash: 99999,
        };
        
        apply_delta(&mut snap, &delta);
        
        prop_assert_eq!(snap.version, original_version);
    }
}

/// Deterministic tests for edge cases found during property testing
#[cfg(test)]
mod deterministic_edge_cases {
    use super::*;

    #[test]
    fn test_delta_with_zero_entities() {
        let empty_snap = Snapshot {
            version: 1,
            tick: 100,
            t: 10.0,
            seq: 1,
            world_hash: 0,
            entities: vec![],
        };

        let viewer = EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 10,
        };

        let interest = FullInterest;
        let delta = diff_snapshots(&empty_snap, &empty_snap, &interest, &viewer);

        assert_eq!(delta.changed.len(), 0);
        assert_eq!(delta.removed.len(), 0);
    }

    #[test]
    fn test_filter_with_no_matching_entities() {
        let snap = Snapshot {
            version: 1,
            tick: 100,
            t: 10.0,
            seq: 1,
            world_hash: 12345,
            entities: vec![
                EntityState {
                    id: 1,
                    pos: IVec2 { x: 1000, y: 1000 }, // Very far away
                    hp: 100,
                    team: 2,
                    ammo: 10,
                },
            ],
        };

        let viewer = EntityState {
            id: 2,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 1,
            ammo: 10,
        };

        let interest = RadiusTeamInterest { radius: 10 };
        let filtered = filter_snapshot_for_viewer(&snap, &interest, &viewer);

        // No entities should match (far away, different team)
        assert_eq!(filtered.entities.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_entity() {
        let mut snap = Snapshot {
            version: 1,
            tick: 100,
            t: 10.0,
            seq: 1,
            world_hash: 12345,
            entities: vec![
                EntityState {
                    id: 1,
                    pos: IVec2 { x: 0, y: 0 },
                    hp: 100,
                    team: 1,
                    ammo: 10,
                },
            ],
        };

        let delta = Delta {
            base_tick: 100,
            tick: 101,
            changed: vec![],
            removed: vec![999], // Non-existent ID
            head_hash: 12345,
        };

        apply_delta(&mut snap, &delta);

        // Should not panic, entity count unchanged
        assert_eq!(snap.entities.len(), 1);
        assert_eq!(snap.entities[0].id, 1);
    }
}
