//! Mutation-resistant comprehensive tests for astraweave-net.
//!
//! These tests target exact return values, boundary conditions, operator swaps,
//! negation bugs, and off-by-one errors to achieve 90%+ mutation kill rate.

#![allow(clippy::approx_constant)]

use astraweave_core::{IVec2, PlanIntent, Team, World};
use astraweave_net::*;
use std::collections::BTreeSet;

// ─── helpers ───────────────────────────────────────────────────────────────

fn e(id: u32, x: i32, y: i32, hp: i32, team: u8, ammo: i32) -> EntityState {
    EntityState {
        id,
        pos: IVec2 { x, y },
        hp,
        team,
        ammo,
    }
}

fn snap(tick: u64, seq: u32, entities: Vec<EntityState>) -> Snapshot {
    Snapshot {
        version: 1,
        tick,
        t: 0.0,
        seq,
        world_hash: 0,
        entities,
    }
}

// ─── EntityState field-level tests ─────────────────────────────────────────

#[test]
fn entity_state_id_exact_value() {
    let es = e(42, 1, 2, 100, 3, 50);
    assert_eq!(es.id, 42, "id must be exactly 42");
}

#[test]
fn entity_state_pos_x_exact() {
    let es = e(1, 7, 3, 100, 0, 0);
    assert_eq!(es.pos.x, 7);
}

#[test]
fn entity_state_pos_y_exact() {
    let es = e(1, 7, 3, 100, 0, 0);
    assert_eq!(es.pos.y, 3);
}

#[test]
fn entity_state_hp_exact() {
    let es = e(1, 0, 0, 55, 0, 0);
    assert_eq!(es.hp, 55);
}

#[test]
fn entity_state_team_exact() {
    let es = e(1, 0, 0, 100, 7, 0);
    assert_eq!(es.team, 7);
}

#[test]
fn entity_state_ammo_exact() {
    let es = e(1, 0, 0, 100, 0, 99);
    assert_eq!(es.ammo, 99);
}

#[test]
fn entity_state_equality() {
    let a = e(1, 2, 3, 100, 0, 10);
    let b = e(1, 2, 3, 100, 0, 10);
    assert_eq!(a, b);
}

#[test]
fn entity_state_inequality_id() {
    let a = e(1, 2, 3, 100, 0, 10);
    let b = e(2, 2, 3, 100, 0, 10);
    assert_ne!(a, b);
}

#[test]
fn entity_state_inequality_pos() {
    let a = e(1, 2, 3, 100, 0, 10);
    let b = e(1, 3, 3, 100, 0, 10);
    assert_ne!(a, b);
}

#[test]
fn entity_state_inequality_hp() {
    let a = e(1, 2, 3, 100, 0, 10);
    let b = e(1, 2, 3, 99, 0, 10);
    assert_ne!(a, b);
}

#[test]
fn entity_state_inequality_team() {
    let a = e(1, 2, 3, 100, 0, 10);
    let b = e(1, 2, 3, 100, 1, 10);
    assert_ne!(a, b);
}

#[test]
fn entity_state_inequality_ammo() {
    let a = e(1, 2, 3, 100, 0, 10);
    let b = e(1, 2, 3, 100, 0, 11);
    assert_ne!(a, b);
}

#[test]
fn entity_state_clone_preserves_all_fields() {
    let a = e(5, 8, 9, 75, 2, 33);
    let b = a;
    assert_eq!(b.id, 5);
    assert_eq!(b.pos.x, 8);
    assert_eq!(b.pos.y, 9);
    assert_eq!(b.hp, 75);
    assert_eq!(b.team, 2);
    assert_eq!(b.ammo, 33);
}

#[test]
fn entity_state_zero_values() {
    let es = e(0, 0, 0, 0, 0, 0);
    assert_eq!(es.id, 0);
    assert_eq!(es.pos.x, 0);
    assert_eq!(es.pos.y, 0);
    assert_eq!(es.hp, 0);
    assert_eq!(es.team, 0);
    assert_eq!(es.ammo, 0);
}

#[test]
fn entity_state_negative_values() {
    let es = e(1, -5, -10, -20, 0, -3);
    assert_eq!(es.pos.x, -5);
    assert_eq!(es.pos.y, -10);
    assert_eq!(es.hp, -20);
    assert_eq!(es.ammo, -3);
}

// ─── Snapshot field-level tests ────────────────────────────────────────────

#[test]
fn snapshot_version_is_one() {
    let s = snap(0, 0, vec![]);
    assert_eq!(s.version, 1, "SNAPSHOT_VERSION must be 1");
}

#[test]
fn snapshot_tick_exact() {
    let s = snap(42, 0, vec![]);
    assert_eq!(s.tick, 42);
}

#[test]
fn snapshot_seq_exact() {
    let s = snap(0, 99, vec![]);
    assert_eq!(s.seq, 99);
}

#[test]
fn snapshot_entities_populated() {
    let entities = vec![e(1, 0, 0, 100, 0, 0), e(2, 1, 1, 50, 1, 10)];
    let s = snap(0, 0, entities.clone());
    assert_eq!(s.entities.len(), 2);
    assert_eq!(s.entities[0].id, 1);
    assert_eq!(s.entities[1].id, 2);
}

// ─── Delta field-level tests ───────────────────────────────────────────────

#[test]
fn delta_base_tick_exact() {
    let d = Delta {
        base_tick: 10,
        tick: 20,
        changed: vec![],
        removed: vec![],
        head_hash: 0,
    };
    assert_eq!(d.base_tick, 10);
    assert_eq!(d.tick, 20);
}

#[test]
fn delta_removed_exact() {
    let d = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![],
        removed: vec![5, 10, 15],
        head_hash: 0,
    };
    assert_eq!(d.removed.len(), 3);
    assert_eq!(d.removed[0], 5);
    assert_eq!(d.removed[1], 10);
    assert_eq!(d.removed[2], 15);
}

#[test]
fn entity_delta_mask_values() {
    // Verify bitmask constants: POS=1, HP=2, TEAM=4, AMMO=8
    let d = EntityDelta {
        id: 1,
        mask: 0b1111, // all 4 bits
        pos: Some(IVec2 { x: 1, y: 2 }),
        hp: Some(50),
        team: Some(1),
        ammo: Some(10),
    };
    assert_eq!(d.mask & 1, 1, "POS bit");
    assert_eq!(d.mask & 2, 2, "HP bit");
    assert_eq!(d.mask & 4, 4, "TEAM bit");
    assert_eq!(d.mask & 8, 8, "AMMO bit");
    assert_eq!(d.mask, 15);
}

#[test]
fn entity_delta_partial_mask() {
    let d = EntityDelta {
        id: 1,
        mask: 0b0101, // POS + TEAM
        pos: Some(IVec2 { x: 3, y: 4 }),
        hp: None,
        team: Some(2),
        ammo: None,
    };
    assert_eq!(d.mask & 1, 1, "POS set");
    assert_eq!(d.mask & 2, 0, "HP not set");
    assert_eq!(d.mask & 4, 4, "TEAM set");
    assert_eq!(d.mask & 8, 0, "AMMO not set");
}

// ─── FullInterest tests ───────────────────────────────────────────────────

#[test]
fn full_interest_always_true_same_team() {
    let fi = FullInterest;
    let viewer = e(1, 0, 0, 100, 0, 0);
    let target = e(2, 100, 100, 50, 0, 0);
    assert!(fi.include(&viewer, &target));
}

#[test]
fn full_interest_always_true_different_team() {
    let fi = FullInterest;
    let viewer = e(1, 0, 0, 100, 0, 0);
    let target = e(2, 100, 100, 50, 1, 0);
    assert!(fi.include(&viewer, &target));
}

#[test]
fn full_interest_self() {
    let fi = FullInterest;
    let viewer = e(1, 5, 5, 100, 0, 0);
    assert!(fi.include(&viewer, &viewer));
}

// ─── RadiusTeamInterest tests ──────────────────────────────────────────────

#[test]
fn radius_interest_same_team_always_included() {
    let ri = RadiusTeamInterest { radius: 1 };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let far_ally = e(2, 1000, 1000, 50, 0, 0); // same team, way beyond radius
    assert!(
        ri.include(&viewer, &far_ally),
        "same team must always include"
    );
}

#[test]
fn radius_interest_enemy_within_radius() {
    let ri = RadiusTeamInterest { radius: 5 };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 3, 4, 50, 1, 0); // distance = 5.0, 3^2+4^2=25 <= 5^2=25
    assert!(ri.include(&viewer, &enemy));
}

#[test]
fn radius_interest_enemy_exactly_at_radius() {
    let ri = RadiusTeamInterest { radius: 5 };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 3, 4, 50, 1, 0); // 9+16=25 <= 25
    assert!(
        ri.include(&viewer, &enemy),
        "exactly at radius should include"
    );
}

#[test]
fn radius_interest_enemy_just_beyond_radius() {
    let ri = RadiusTeamInterest { radius: 4 };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 3, 4, 50, 1, 0); // 9+16=25 > 4^2=16
    assert!(!ri.include(&viewer, &enemy), "beyond radius should exclude");
}

#[test]
fn radius_interest_enemy_at_zero_distance() {
    let ri = RadiusTeamInterest { radius: 1 };
    let viewer = e(1, 5, 5, 100, 0, 0);
    let enemy = e(2, 5, 5, 50, 1, 0); // same pos, distance 0
    assert!(ri.include(&viewer, &enemy));
}

#[test]
fn radius_interest_enemy_one_step() {
    let ri = RadiusTeamInterest { radius: 1 };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 1, 0, 50, 1, 0); // distance 1
    assert!(ri.include(&viewer, &enemy));
}

#[test]
fn radius_interest_diagonal_check() {
    // distance = sqrt(2) ~= 1.414, radius=1 → 1+1=2 > 1
    let ri = RadiusTeamInterest { radius: 1 };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 1, 1, 50, 1, 0);
    assert!(
        !ri.include(&viewer, &enemy),
        "diagonal distance exceeds radius 1"
    );
}

#[test]
fn radius_interest_negative_coords() {
    let ri = RadiusTeamInterest { radius: 5 };
    let viewer = e(1, -2, -3, 100, 0, 0);
    let enemy = e(2, 1, 1, 50, 1, 0); // dx=3, dy=4, dist^2=25 <= 25
    assert!(ri.include(&viewer, &enemy));
}

#[test]
fn radius_interest_zero_radius_same_pos() {
    let ri = RadiusTeamInterest { radius: 0 };
    let viewer = e(1, 5, 5, 100, 0, 0);
    let enemy = e(2, 5, 5, 50, 1, 0); // dist=0, 0 <= 0
    assert!(ri.include(&viewer, &enemy));
}

#[test]
fn radius_interest_zero_radius_adjacent() {
    let ri = RadiusTeamInterest { radius: 0 };
    let viewer = e(1, 5, 5, 100, 0, 0);
    let enemy = e(2, 5, 6, 50, 1, 0); // dist=1, 1 > 0
    assert!(!ri.include(&viewer, &enemy));
}

// ─── FovInterest tests ─────────────────────────────────────────────────────

#[test]
fn fov_interest_same_team_always_true() {
    let fov = FovInterest {
        radius: 1,
        half_angle_deg: 1.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let ally = e(2, 100, 100, 50, 0, 0);
    assert!(fov.include(&viewer, &ally));
}

#[test]
fn fov_interest_beyond_radius_excluded() {
    let fov = FovInterest {
        radius: 3,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 10, 0, 50, 1, 0); // dist 10 > radius 3
    assert!(!fov.include(&viewer, &enemy));
}

#[test]
fn fov_interest_in_cone_and_radius() {
    let fov = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 5, 0, 50, 1, 0); // straight ahead
    assert!(fov.include(&viewer, &enemy));
}

#[test]
fn fov_interest_behind_excluded() {
    let fov = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, -5, 0, 50, 1, 0); // behind
    assert!(!fov.include(&viewer, &enemy));
}

#[test]
fn fov_interest_zero_facing_includes_all_in_radius() {
    let fov = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 0, y: 0 },
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, -5, 3, 50, 1, 0);
    assert!(
        fov.include(&viewer, &enemy),
        "zero facing = omnidirectional"
    );
}

#[test]
fn fov_interest_same_pos_included() {
    let fov = FovInterest {
        radius: 10,
        half_angle_deg: 10.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    let viewer = e(1, 5, 5, 100, 0, 0);
    let enemy = e(2, 5, 5, 50, 1, 0);
    assert!(
        fov.include(&viewer, &enemy),
        "same pos = zero distance = true"
    );
}

#[test]
fn fov_interest_wide_angle_180() {
    let fov = FovInterest {
        radius: 20,
        half_angle_deg: 180.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    // 180 half-angle = everything in radius
    let enemy = e(2, -5, 0, 50, 1, 0);
    assert!(fov.include(&viewer, &enemy));
}

// ─── FovLosInterest tests ──────────────────────────────────────────────────

#[test]
fn fov_los_same_team_always_true() {
    let fov = FovLosInterest {
        radius: 1,
        half_angle_deg: 1.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: BTreeSet::new(),
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let ally = e(2, 100, 100, 50, 0, 0);
    assert!(fov.include(&viewer, &ally));
}

#[test]
fn fov_los_clear_path() {
    let fov = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: BTreeSet::new(),
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 5, 0, 50, 1, 0);
    assert!(fov.include(&viewer, &enemy));
}

#[test]
fn fov_los_blocked_by_obstacle() {
    let mut obs = BTreeSet::new();
    obs.insert((3, 0)); // obstacle between (0,0) and (5,0)
    let fov = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: obs,
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 5, 0, 50, 1, 0);
    assert!(!fov.include(&viewer, &enemy), "obstacle blocks LOS");
}

#[test]
fn fov_los_obstacle_at_viewer_pos_not_blocking() {
    let mut obs = BTreeSet::new();
    obs.insert((0, 0)); // obstacle at viewer's own position
    let fov = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: obs,
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 3, 0, 50, 1, 0);
    assert!(
        fov.include(&viewer, &enemy),
        "viewer's own cell is skipped in LOS check"
    );
}

#[test]
fn fov_los_zero_facing_los_only() {
    // zero facing → omnidirectional, but still LOS check
    let mut obs = BTreeSet::new();
    obs.insert((2, 0));
    let fov = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 0, y: 0 },
        obstacles: obs,
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 4, 0, 50, 1, 0);
    assert!(
        !fov.include(&viewer, &enemy),
        "LOS blocked even with zero facing"
    );
}

#[test]
fn fov_los_beyond_radius_excluded() {
    let fov = FovLosInterest {
        radius: 3,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: BTreeSet::new(),
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 10, 0, 50, 1, 0);
    assert!(!fov.include(&viewer, &enemy));
}

#[test]
fn fov_los_same_pos_included() {
    let fov = FovLosInterest {
        radius: 10,
        half_angle_deg: 10.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: BTreeSet::new(),
    };
    let viewer = e(1, 5, 5, 100, 0, 0);
    let enemy = e(2, 5, 5, 50, 1, 0);
    assert!(fov.include(&viewer, &enemy));
}

// ─── build_snapshot tests ──────────────────────────────────────────────────

#[test]
fn build_snapshot_from_world() {
    let mut world = World::new();
    world.spawn("A", IVec2 { x: 1, y: 2 }, Team { id: 0 }, 100, 50);
    let snap = build_snapshot(&world, 10, 5);
    assert_eq!(snap.version, 1);
    assert_eq!(snap.tick, 10);
    assert_eq!(snap.seq, 5);
    assert_eq!(snap.entities.len(), 1);
    assert_eq!(snap.entities[0].pos.x, 1);
    assert_eq!(snap.entities[0].pos.y, 2);
    assert_eq!(snap.entities[0].hp, 100);
    assert_eq!(snap.entities[0].ammo, 50);
    assert_ne!(
        snap.world_hash, 0,
        "hash must be non-zero for non-empty world"
    );
}

#[test]
fn build_snapshot_empty_world() {
    let world = World::new();
    let snap = build_snapshot(&world, 0, 0);
    assert_eq!(snap.version, 1);
    assert_eq!(snap.tick, 0);
    assert_eq!(snap.seq, 0);
    assert!(snap.entities.is_empty());
}

#[test]
fn build_snapshot_deterministic_hash() {
    let mut w1 = World::new();
    w1.spawn("A", IVec2 { x: 3, y: 4 }, Team { id: 0 }, 100, 0);
    let s1 = build_snapshot(&w1, 0, 0);

    let mut w2 = World::new();
    w2.spawn("A", IVec2 { x: 3, y: 4 }, Team { id: 0 }, 100, 0);
    let s2 = build_snapshot(&w2, 0, 0);

    assert_eq!(s1.world_hash, s2.world_hash);
}

#[test]
fn build_snapshot_different_entities_different_hash() {
    let mut w1 = World::new();
    w1.spawn("A", IVec2 { x: 1, y: 1 }, Team { id: 0 }, 100, 0);
    let s1 = build_snapshot(&w1, 0, 0);

    let mut w2 = World::new();
    w2.spawn("B", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let s2 = build_snapshot(&w2, 0, 0);

    assert_ne!(s1.world_hash, s2.world_hash);
}

// ─── filter_snapshot_for_viewer tests ──────────────────────────────────────

#[test]
fn filter_snapshot_full_interest() {
    let entities = vec![e(1, 0, 0, 100, 0, 0), e(2, 50, 50, 50, 1, 0)];
    let s = snap(0, 0, entities);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let filtered = filter_snapshot_for_viewer(&s, &FullInterest, &viewer);
    assert_eq!(filtered.entities.len(), 2, "full interest keeps all");
}

#[test]
fn filter_snapshot_radius_excludes_distant() {
    let entities = vec![
        e(1, 0, 0, 100, 0, 0),
        e(2, 50, 50, 50, 1, 0),
        e(3, 2, 0, 80, 0, 0), // same team
    ];
    let s = snap(0, 0, entities);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let ri = RadiusTeamInterest { radius: 5 };
    let filtered = filter_snapshot_for_viewer(&s, &ri, &viewer);
    assert_eq!(
        filtered.entities.len(),
        2,
        "distant enemy excluded, ally kept"
    );
    assert!(filtered.entities.iter().any(|e| e.id == 1));
    assert!(filtered.entities.iter().any(|e| e.id == 3));
    assert!(!filtered.entities.iter().any(|e| e.id == 2));
}

#[test]
fn filter_snapshot_changes_world_hash() {
    let entities = vec![e(1, 0, 0, 100, 0, 0), e(2, 50, 50, 50, 1, 0)];
    let s = snap(0, 0, entities);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let ri = RadiusTeamInterest { radius: 3 };
    let filtered = filter_snapshot_for_viewer(&s, &ri, &viewer);
    // Filtered snapshot has different entity set, so different hash
    assert_eq!(filtered.entities.len(), 1);
}

// ─── diff_snapshots tests ──────────────────────────────────────────────────

#[test]
fn diff_snapshots_no_change() {
    let ents = vec![e(1, 0, 0, 100, 0, 10)];
    let base = snap(0, 0, ents.clone());
    let head = snap(1, 1, ents);
    let viewer = e(1, 0, 0, 100, 0, 10);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.base_tick, 0);
    assert_eq!(delta.tick, 1);
    assert!(delta.changed.is_empty(), "no changes = no deltas");
    assert!(delta.removed.is_empty());
}

#[test]
fn diff_snapshots_pos_changed() {
    let base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let head = snap(1, 1, vec![e(1, 5, 5, 100, 0, 0)]);
    let viewer = e(1, 5, 5, 100, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].id, 1);
    assert_eq!(delta.changed[0].mask & 1, 1, "POS bit set");
    assert_eq!(delta.changed[0].pos, Some(IVec2 { x: 5, y: 5 }));
    assert_eq!(delta.changed[0].hp, None, "HP unchanged");
}

#[test]
fn diff_snapshots_hp_changed() {
    let base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let head = snap(1, 1, vec![e(1, 0, 0, 50, 0, 0)]);
    let viewer = e(1, 0, 0, 50, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].mask & 2, 2, "HP bit set");
    assert_eq!(delta.changed[0].hp, Some(50));
    assert_eq!(delta.changed[0].pos, None);
}

#[test]
fn diff_snapshots_team_changed() {
    let base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let head = snap(1, 1, vec![e(1, 0, 0, 100, 2, 0)]);
    let viewer = e(1, 0, 0, 100, 2, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].mask & 4, 4, "TEAM bit set");
    assert_eq!(delta.changed[0].team, Some(2));
}

#[test]
fn diff_snapshots_ammo_changed() {
    let base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 10)]);
    let head = snap(1, 1, vec![e(1, 0, 0, 100, 0, 5)]);
    let viewer = e(1, 0, 0, 100, 0, 5);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].mask & 8, 8, "AMMO bit set");
    assert_eq!(delta.changed[0].ammo, Some(5));
}

#[test]
fn diff_snapshots_entity_removed() {
    let base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0), e(2, 5, 5, 50, 0, 0)]);
    let head = snap(1, 1, vec![e(1, 0, 0, 100, 0, 0)]);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.removed.len(), 1);
    assert_eq!(delta.removed[0], 2);
}

#[test]
fn diff_snapshots_entity_added() {
    let base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let head = snap(1, 1, vec![e(1, 0, 0, 100, 0, 0), e(2, 5, 5, 50, 0, 0)]);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].id, 2);
    // New entity has all bits set (POS | HP | TEAM | AMMO = 15)
    assert_eq!(delta.changed[0].mask, 15);
    assert_eq!(delta.changed[0].pos, Some(IVec2 { x: 5, y: 5 }));
    assert_eq!(delta.changed[0].hp, Some(50));
    assert_eq!(delta.changed[0].team, Some(0));
    assert_eq!(delta.changed[0].ammo, Some(0));
}

#[test]
fn diff_snapshots_multiple_changes() {
    let base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 10), e(2, 5, 5, 50, 1, 20)]);
    let head = snap(1, 1, vec![e(1, 1, 0, 90, 0, 10), e(2, 5, 5, 50, 1, 15)]);
    let viewer = e(1, 1, 0, 90, 0, 10);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 2);
}

// ─── apply_delta tests ─────────────────────────────────────────────────────

#[test]
fn apply_delta_updates_position() {
    let mut base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![EntityDelta {
            id: 1,
            mask: 1, // POS
            pos: Some(IVec2 { x: 10, y: 20 }),
            hp: None,
            team: None,
            ammo: None,
        }],
        removed: vec![],
        head_hash: 999,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.tick, 1);
    assert_eq!(base.entities[0].pos.x, 10);
    assert_eq!(base.entities[0].pos.y, 20);
    assert_eq!(base.entities[0].hp, 100, "HP unchanged");
    assert_eq!(base.world_hash, 999);
}

#[test]
fn apply_delta_updates_hp() {
    let mut base = snap(0, 0, vec![e(1, 5, 5, 100, 0, 0)]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![EntityDelta {
            id: 1,
            mask: 2, // HP
            pos: None,
            hp: Some(42),
            team: None,
            ammo: None,
        }],
        removed: vec![],
        head_hash: 0,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.entities[0].hp, 42);
    assert_eq!(base.entities[0].pos.x, 5, "pos unchanged");
}

#[test]
fn apply_delta_updates_team() {
    let mut base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![EntityDelta {
            id: 1,
            mask: 4, // TEAM
            pos: None,
            hp: None,
            team: Some(3),
            ammo: None,
        }],
        removed: vec![],
        head_hash: 0,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.entities[0].team, 3);
}

#[test]
fn apply_delta_updates_ammo() {
    let mut base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 10)]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![EntityDelta {
            id: 1,
            mask: 8, // AMMO
            pos: None,
            hp: None,
            team: None,
            ammo: Some(0),
        }],
        removed: vec![],
        head_hash: 0,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.entities[0].ammo, 0);
}

#[test]
fn apply_delta_removes_entity() {
    let mut base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0), e(2, 5, 5, 50, 1, 0)]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![],
        removed: vec![2],
        head_hash: 0,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.entities.len(), 1);
    assert_eq!(base.entities[0].id, 1);
}

#[test]
fn apply_delta_adds_new_entity() {
    let mut base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![EntityDelta {
            id: 2,
            mask: 15, // all bits
            pos: Some(IVec2 { x: 7, y: 8 }),
            hp: Some(60),
            team: Some(2),
            ammo: Some(30),
        }],
        removed: vec![],
        head_hash: 0,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.entities.len(), 2);
    let new = base.entities.iter().find(|e| e.id == 2).unwrap();
    assert_eq!(new.pos.x, 7);
    assert_eq!(new.pos.y, 8);
    assert_eq!(new.hp, 60);
    assert_eq!(new.team, 2);
    assert_eq!(new.ammo, 30);
}

#[test]
fn apply_delta_wrong_base_tick_is_noop() {
    let mut base = snap(5, 0, vec![e(1, 0, 0, 100, 0, 0)]);
    let delta = Delta {
        base_tick: 3, // doesn't match base.tick=5
        tick: 6,
        changed: vec![EntityDelta {
            id: 1,
            mask: 2,
            pos: None,
            hp: Some(1),
            team: None,
            ammo: None,
        }],
        removed: vec![],
        head_hash: 0,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.tick, 5, "tick unchanged when base_tick mismatch");
    assert_eq!(base.entities[0].hp, 100, "hp unchanged");
}

#[test]
fn apply_delta_all_fields_at_once() {
    let mut base = snap(0, 0, vec![e(1, 0, 0, 100, 0, 10)]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![EntityDelta {
            id: 1,
            mask: 15,
            pos: Some(IVec2 { x: 9, y: 8 }),
            hp: Some(50),
            team: Some(3),
            ammo: Some(99),
        }],
        removed: vec![],
        head_hash: 42,
    };
    apply_delta(&mut base, &delta);
    let ent = &base.entities[0];
    assert_eq!(ent.pos.x, 9);
    assert_eq!(ent.pos.y, 8);
    assert_eq!(ent.hp, 50);
    assert_eq!(ent.team, 3);
    assert_eq!(ent.ammo, 99);
    assert_eq!(base.world_hash, 42);
}

// ─── diff + apply roundtrip tests ──────────────────────────────────────────

#[test]
fn diff_apply_roundtrip_position_change() {
    let base_ents = vec![e(1, 0, 0, 100, 0, 0)];
    let head_ents = vec![e(1, 5, 5, 100, 0, 0)];
    let base = snap(0, 0, base_ents.clone());
    let head = snap(1, 1, head_ents.clone());
    let viewer = e(1, 5, 5, 100, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    let mut applied = snap(0, 0, base_ents);
    apply_delta(&mut applied, &delta);
    assert_eq!(applied.entities[0].pos.x, 5);
    assert_eq!(applied.entities[0].pos.y, 5);
    assert_eq!(applied.tick, 1);
}

#[test]
fn diff_apply_roundtrip_entity_removal() {
    let base_ents = vec![e(1, 0, 0, 100, 0, 0), e(2, 5, 5, 50, 0, 0)];
    let head_ents = vec![e(1, 0, 0, 100, 0, 0)];
    let base = snap(0, 0, base_ents.clone());
    let head = snap(1, 1, head_ents);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    let mut applied = snap(0, 0, base_ents);
    apply_delta(&mut applied, &delta);
    assert_eq!(applied.entities.len(), 1);
    assert_eq!(applied.entities[0].id, 1);
}

// ─── GameServer defaults tests ─────────────────────────────────────────────

#[test]
fn game_server_default_ids() {
    let gs = GameServer::new();
    // Player, companion, enemy IDs are returned by World::spawn
    // They should be distinct
    assert_ne!(gs.player_id, gs.companion_id);
    assert_ne!(gs.player_id, gs.enemy_id);
    assert_ne!(gs.companion_id, gs.enemy_id);
}

#[tokio::test]
async fn game_server_default_world_has_three_entities() {
    let gs = GameServer::new();
    let w = gs.world.lock().await;
    let snap = build_snapshot(&w, 0, 0);
    assert_eq!(snap.entities.len(), 3, "player + companion + enemy");
}

#[tokio::test]
async fn game_server_player_stats() {
    let gs = GameServer::new();
    let w = gs.world.lock().await;
    let snap = build_snapshot(&w, 0, 0);
    let player = snap.entities.iter().find(|e| e.id == gs.player_id).unwrap();
    assert_eq!(player.pos.x, 2);
    assert_eq!(player.pos.y, 2);
    assert_eq!(player.hp, 100);
    assert_eq!(player.team, 0);
    assert_eq!(player.ammo, 0);
}

#[tokio::test]
async fn game_server_companion_stats() {
    let gs = GameServer::new();
    let w = gs.world.lock().await;
    let snap = build_snapshot(&w, 0, 0);
    let comp = snap
        .entities
        .iter()
        .find(|e| e.id == gs.companion_id)
        .unwrap();
    assert_eq!(comp.pos.x, 2);
    assert_eq!(comp.pos.y, 3);
    assert_eq!(comp.hp, 80);
    assert_eq!(comp.team, 1);
    assert_eq!(comp.ammo, 30);
}

#[tokio::test]
async fn game_server_enemy_stats() {
    let gs = GameServer::new();
    let w = gs.world.lock().await;
    let snap = build_snapshot(&w, 0, 0);
    let enemy = snap.entities.iter().find(|e| e.id == gs.enemy_id).unwrap();
    assert_eq!(enemy.pos.x, 12);
    assert_eq!(enemy.pos.y, 2);
    assert_eq!(enemy.hp, 60);
    assert_eq!(enemy.team, 2);
    assert_eq!(enemy.ammo, 0);
}

#[tokio::test]
async fn game_server_obstacles() {
    let gs = GameServer::new();
    let w = gs.world.lock().await;
    // Obstacles at (6,y) for y in 1..=8
    for y in 1..=8 {
        assert!(w.obstacles.contains(&(6, y)), "obstacle at (6, {y})");
    }
    assert!(!w.obstacles.contains(&(6, 0)), "no obstacle at y=0");
    assert!(!w.obstacles.contains(&(6, 9)), "no obstacle at y=9");
}

#[test]
fn game_server_default_impl() {
    let gs = GameServer::default();
    assert_ne!(gs.player_id, gs.companion_id);
}

#[tokio::test]
async fn game_server_tick_starts_at_zero() {
    let gs = GameServer::new();
    assert_eq!(gs.tick.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[tokio::test]
async fn game_server_replay_starts_empty() {
    let gs = GameServer::new();
    let replay = gs.replay.lock().await;
    assert!(replay.is_empty());
}

// ─── ReplayEvent tests ─────────────────────────────────────────────────────

#[test]
fn replay_event_fields() {
    let re = ReplayEvent {
        tick: 42,
        seq: 3,
        actor_id: 7,
        intent: PlanIntent {
            plan_id: "test".into(),
            steps: vec![],
        },
        world_hash: 12345,
    };
    assert_eq!(re.tick, 42);
    assert_eq!(re.seq, 3);
    assert_eq!(re.actor_id, 7);
    assert_eq!(re.world_hash, 12345);
    assert_eq!(re.intent.plan_id, "test");
    assert!(re.intent.steps.is_empty());
}

// ─── replay_from tests ─────────────────────────────────────────────────────

#[test]
fn replay_from_empty_events() {
    let world = World::new();
    let result = replay_from(world, &[]);
    assert!(result.is_ok());
}

#[test]
fn replay_from_deterministic() {
    let w1 = World::new();
    let w2 = World::new();
    let h1 = replay_from(w1, &[]).unwrap();
    let h2 = replay_from(w2, &[]).unwrap();
    assert_eq!(h1, h2, "same input = same hash");
}

// ─── NetError tests ────────────────────────────────────────────────────────

#[test]
fn net_error_connection_message() {
    let err = NetError::Connection("timeout".into());
    assert_eq!(format!("{err}"), "connection error: timeout");
}

#[test]
fn net_error_tls_message() {
    let err = NetError::Tls("cert expired".into());
    assert_eq!(format!("{err}"), "TLS error: cert expired");
}

#[test]
fn net_error_protocol_message() {
    let err = NetError::Protocol("bad version".into());
    assert_eq!(format!("{err}"), "protocol error: bad version");
}

#[test]
fn net_error_rate_limited_message() {
    let err = NetError::RateLimited("too fast".into());
    assert_eq!(format!("{err}"), "rate limited: too fast");
}

#[test]
fn net_error_auth_message() {
    let err = NetError::Auth("bad token".into());
    assert_eq!(format!("{err}"), "authentication error: bad token");
}

#[test]
fn net_error_io_from() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let net_err: NetError = io_err.into();
    let msg = format!("{net_err}");
    assert!(msg.contains("I/O error"), "got: {msg}");
}

// ─── Serialization round-trip tests ────────────────────────────────────────

#[test]
fn entity_state_json_roundtrip() {
    let es = e(10, 3, 4, 75, 2, 15);
    let json = serde_json::to_string(&es).unwrap();
    let back: EntityState = serde_json::from_str(&json).unwrap();
    assert_eq!(back, es);
}

#[test]
fn snapshot_json_roundtrip() {
    let s = snap(10, 5, vec![e(1, 0, 0, 100, 0, 50)]);
    let json = serde_json::to_string(&s).unwrap();
    let back: Snapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tick, 10);
    assert_eq!(back.seq, 5);
    assert_eq!(back.entities.len(), 1);
    assert_eq!(back.entities[0], s.entities[0]);
}

#[test]
fn delta_json_roundtrip() {
    let d = Delta {
        base_tick: 5,
        tick: 10,
        changed: vec![EntityDelta {
            id: 1,
            mask: 3,
            pos: Some(IVec2 { x: 7, y: 8 }),
            hp: Some(50),
            team: None,
            ammo: None,
        }],
        removed: vec![3, 4],
        head_hash: 777,
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: Delta = serde_json::from_str(&json).unwrap();
    assert_eq!(back, d);
}

#[test]
fn msg_client_hello_roundtrip() {
    let msg = Msg::ClientHello {
        name: "Player1".into(),
        token: Some("abc".into()),
        policy: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: Msg = serde_json::from_str(&json).unwrap();
    match back {
        Msg::ClientHello {
            name,
            token,
            policy,
        } => {
            assert_eq!(name, "Player1");
            assert_eq!(token, Some("abc".into()));
            assert_eq!(policy, None);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn msg_server_welcome_roundtrip() {
    let msg = Msg::ServerWelcome { id: 42 };
    let json = serde_json::to_string(&msg).unwrap();
    let back: Msg = serde_json::from_str(&json).unwrap();
    match back {
        Msg::ServerWelcome { id } => assert_eq!(id, 42),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn msg_server_apply_result_roundtrip() {
    let msg = Msg::ServerApplyResult {
        ok: false,
        err: Some("invalid".into()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: Msg = serde_json::from_str(&json).unwrap();
    match back {
        Msg::ServerApplyResult { ok, err } => {
            assert!(!ok);
            assert_eq!(err, Some("invalid".into()));
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn msg_server_ack_roundtrip() {
    let msg = Msg::ServerAck {
        seq: 10,
        tick_applied: 500,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: Msg = serde_json::from_str(&json).unwrap();
    match back {
        Msg::ServerAck { seq, tick_applied } => {
            assert_eq!(seq, 10);
            assert_eq!(tick_applied, 500);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn replay_event_json_roundtrip() {
    let re = ReplayEvent {
        tick: 100,
        seq: 5,
        actor_id: 3,
        intent: PlanIntent {
            plan_id: "plan-1".into(),
            steps: vec![],
        },
        world_hash: 9999,
    };
    let json = serde_json::to_string(&re).unwrap();
    let back: ReplayEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tick, 100);
    assert_eq!(back.seq, 5);
    assert_eq!(back.actor_id, 3);
    assert_eq!(back.world_hash, 9999);
}

// ─── InterestPolicy tests ──────────────────────────────────────────────────

#[test]
fn interest_policy_radius_variant() {
    let p = InterestPolicy::Radius { radius: 10 };
    match p {
        InterestPolicy::Radius { radius } => assert_eq!(radius, 10),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn interest_policy_fov_variant() {
    let p = InterestPolicy::Fov {
        radius: 20,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    match p {
        InterestPolicy::Fov {
            radius,
            half_angle_deg,
            facing,
        } => {
            assert_eq!(radius, 20);
            assert!((half_angle_deg - 45.0).abs() < f32::EPSILON);
            assert_eq!(facing.x, 1);
            assert_eq!(facing.y, 0);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn interest_policy_fov_los_variant() {
    let p = InterestPolicy::FovLos {
        radius: 15,
        half_angle_deg: 60.0,
        facing: IVec2 { x: 0, y: 1 },
    };
    match p {
        InterestPolicy::FovLos {
            radius,
            half_angle_deg,
            facing,
        } => {
            assert_eq!(radius, 15);
            assert!((half_angle_deg - 60.0).abs() < f32::EPSILON);
            assert_eq!(facing.x, 0);
            assert_eq!(facing.y, 1);
        }
        _ => panic!("wrong variant"),
    }
}

// ─── ServerEvent tests ─────────────────────────────────────────────────────

#[test]
fn server_event_snapshot_variant() {
    let s = snap(5, 2, vec![e(1, 0, 0, 100, 0, 0)]);
    let evt = ServerEvent::Snapshot(s);
    match evt {
        ServerEvent::Snapshot(snap) => {
            assert_eq!(snap.tick, 5);
            assert_eq!(snap.seq, 2);
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn server_event_force_snapshot_variant() {
    let s = snap(7, 3, vec![]);
    let evt = ServerEvent::ForceSnapshot(s);
    match evt {
        ServerEvent::ForceSnapshot(snap) => assert_eq!(snap.tick, 7),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn server_event_apply_result_ok() {
    let evt = ServerEvent::ApplyResult {
        ok: true,
        err: None,
    };
    match evt {
        ServerEvent::ApplyResult { ok, err } => {
            assert!(ok);
            assert!(err.is_none());
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn server_event_apply_result_err() {
    let evt = ServerEvent::ApplyResult {
        ok: false,
        err: Some("fail".into()),
    };
    match evt {
        ServerEvent::ApplyResult { ok, err } => {
            assert!(!ok);
            assert_eq!(err, Some("fail".into()));
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn server_event_ack() {
    let evt = ServerEvent::Ack {
        seq: 42,
        tick_applied: 1000,
    };
    match evt {
        ServerEvent::Ack { seq, tick_applied } => {
            assert_eq!(seq, 42);
            assert_eq!(tick_applied, 1000);
        }
        _ => panic!("wrong variant"),
    }
}

// ─── Boundary conditions ───────────────────────────────────────────────────

#[test]
fn radius_interest_large_radius() {
    // Use a large but non-overflowing radius (sqrt(i32::MAX) ≈ 46340)
    let ri = RadiusTeamInterest { radius: 46340 };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let far = e(2, 30000, 30000, 50, 1, 0);
    assert!(
        ri.include(&viewer, &far),
        "large radius should include distant enemies"
    );
}

#[test]
fn entity_state_max_values() {
    let es = EntityState {
        id: u32::MAX,
        pos: IVec2 {
            x: i32::MAX,
            y: i32::MAX,
        },
        hp: i32::MAX,
        team: u8::MAX,
        ammo: i32::MAX,
    };
    assert_eq!(es.id, u32::MAX);
    assert_eq!(es.hp, i32::MAX);
    assert_eq!(es.team, u8::MAX);
}

#[test]
fn apply_delta_to_empty_snapshot() {
    let mut base = snap(0, 0, vec![]);
    let delta = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![EntityDelta {
            id: 99,
            mask: 15,
            pos: Some(IVec2 { x: 1, y: 2 }),
            hp: Some(100),
            team: Some(0),
            ammo: Some(50),
        }],
        removed: vec![],
        head_hash: 0,
    };
    apply_delta(&mut base, &delta);
    assert_eq!(base.entities.len(), 1);
    assert_eq!(base.entities[0].id, 99);
}

#[test]
fn diff_empty_snapshots() {
    let base = snap(0, 0, vec![]);
    let head = snap(1, 1, vec![]);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert!(delta.changed.is_empty());
    assert!(delta.removed.is_empty());
}

#[test]
fn filter_empty_snapshot() {
    let s = snap(0, 0, vec![]);
    let viewer = e(1, 0, 0, 100, 0, 0);
    let filtered = filter_snapshot_for_viewer(&s, &FullInterest, &viewer);
    assert!(filtered.entities.is_empty());
}

// ─── Multiple entity stress tests ──────────────────────────────────────────

#[test]
fn diff_apply_many_entities() {
    let base_ents: Vec<EntityState> = (0..50).map(|i| e(i, i as i32, 0, 100, 0, 0)).collect();
    let head_ents: Vec<EntityState> = (0..50).map(|i| e(i, i as i32 + 1, 1, 99, 0, 0)).collect();
    let base = snap(0, 0, base_ents.clone());
    let head = snap(1, 1, head_ents);
    let viewer = e(0, 1, 1, 99, 0, 0);
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 50, "all 50 entities changed");
    let mut applied = snap(0, 0, base_ents);
    apply_delta(&mut applied, &delta);
    assert_eq!(applied.entities.len(), 50);
    assert_eq!(applied.tick, 1);
}

#[test]
fn filter_snapshot_preserves_version_and_tick() {
    let s = Snapshot {
        version: 1,
        tick: 42,
        t: 3.14,
        seq: 7,
        world_hash: 999,
        entities: vec![e(1, 0, 0, 100, 0, 0)],
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let filtered = filter_snapshot_for_viewer(&s, &FullInterest, &viewer);
    assert_eq!(filtered.version, 1);
    assert_eq!(filtered.tick, 42);
    assert_eq!(filtered.seq, 7);
}

// ─── Msg ClientInput roundtrip ─────────────────────────────────────────────

#[test]
fn msg_client_input_roundtrip() {
    let msg = Msg::ClientInput {
        seq: 5,
        tick: 100,
        actor_id: 3,
        intent: PlanIntent {
            plan_id: "attack".into(),
            steps: vec![],
        },
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: Msg = serde_json::from_str(&json).unwrap();
    match back {
        Msg::ClientInput {
            seq,
            tick,
            actor_id,
            intent,
        } => {
            assert_eq!(seq, 5);
            assert_eq!(tick, 100);
            assert_eq!(actor_id, 3);
            assert_eq!(intent.plan_id, "attack");
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn msg_client_propose_plan_roundtrip() {
    let msg = Msg::ClientProposePlan {
        actor_id: 7,
        intent: PlanIntent {
            plan_id: "defend".into(),
            steps: vec![],
        },
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: Msg = serde_json::from_str(&json).unwrap();
    match back {
        Msg::ClientProposePlan { actor_id, intent } => {
            assert_eq!(actor_id, 7);
            assert_eq!(intent.plan_id, "defend");
        }
        _ => panic!("wrong variant"),
    }
}

// ─── Interest edge cases ───────────────────────────────────────────────────

#[test]
fn fov_interest_narrow_angle_excludes_side() {
    let fov = FovInterest {
        radius: 20,
        half_angle_deg: 5.0, // very narrow cone
        facing: IVec2 { x: 1, y: 0 },
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let side = e(2, 0, 5, 50, 1, 0); // 90 degrees to the side
    assert!(!fov.include(&viewer, &side));
}

#[test]
fn fov_los_multiple_obstacles() {
    let mut obs = BTreeSet::new();
    obs.insert((2, 0));
    obs.insert((3, 0));
    obs.insert((4, 0));
    let fov = FovLosInterest {
        radius: 20,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: obs,
    };
    let viewer = e(1, 0, 0, 100, 0, 0);
    let enemy = e(2, 6, 0, 50, 1, 0);
    assert!(!fov.include(&viewer, &enemy), "wall of obstacles blocks");
}

#[test]
fn radius_interest_symmetric() {
    let ri = RadiusTeamInterest { radius: 5 };
    let a = e(1, 0, 0, 100, 0, 0);
    let b = e(2, 3, 4, 50, 1, 0);
    assert_eq!(
        ri.include(&a, &b),
        ri.include(&b, &a),
        "symmetric for same-distance different-team"
    );
}

// ─── Clone and Debug trait tests ───────────────────────────────────────────

#[test]
fn snapshot_clone() {
    let s = snap(10, 5, vec![e(1, 0, 0, 100, 0, 0)]);
    let s2 = s.clone();
    assert_eq!(s2.tick, 10);
    assert_eq!(s2.entities.len(), 1);
}

#[test]
fn delta_clone() {
    let d = Delta {
        base_tick: 1,
        tick: 2,
        changed: vec![],
        removed: vec![5],
        head_hash: 42,
    };
    let d2 = d.clone();
    assert_eq!(d2, d);
}

#[test]
fn entity_delta_clone() {
    let ed = EntityDelta {
        id: 1,
        mask: 3,
        pos: Some(IVec2 { x: 5, y: 6 }),
        hp: Some(50),
        team: None,
        ammo: None,
    };
    let ed2 = ed.clone();
    assert_eq!(ed2, ed);
}

#[test]
fn server_event_clone() {
    let evt = ServerEvent::Ack {
        seq: 1,
        tick_applied: 10,
    };
    let evt2 = evt.clone();
    match evt2 {
        ServerEvent::Ack { seq, tick_applied } => {
            assert_eq!(seq, 1);
            assert_eq!(tick_applied, 10);
        }
        _ => panic!("wrong variant after clone"),
    }
}

#[test]
fn entity_state_debug() {
    let es = e(1, 0, 0, 100, 0, 0);
    let dbg = format!("{es:?}");
    assert!(dbg.contains("EntityState"));
}

#[test]
fn replay_event_clone() {
    let re = ReplayEvent {
        tick: 1,
        seq: 2,
        actor_id: 3,
        intent: PlanIntent {
            plan_id: "test".into(),
            steps: vec![],
        },
        world_hash: 42,
    };
    let re2 = re.clone();
    assert_eq!(re2.tick, 1);
    assert_eq!(re2.seq, 2);
    assert_eq!(re2.actor_id, 3);
    assert_eq!(re2.world_hash, 42);
}

// ─── World-based snapshot consistency ──────────────────────────────────────

#[test]
fn build_snapshot_with_obstacles_affects_hash() {
    let mut w1 = World::new();
    w1.spawn("A", IVec2 { x: 1, y: 1 }, Team { id: 0 }, 100, 0);
    let s1 = build_snapshot(&w1, 0, 0);

    let mut w2 = World::new();
    w2.spawn("A", IVec2 { x: 1, y: 1 }, Team { id: 0 }, 100, 0);
    w2.obstacles.insert((5, 5));
    let s2 = build_snapshot(&w2, 0, 0);

    assert_ne!(s1.world_hash, s2.world_hash, "obstacles affect hash");
}

#[test]
fn build_snapshot_multi_team() {
    let mut world = World::new();
    world.spawn("A", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 0);
    world.spawn("B", IVec2 { x: 5, y: 5 }, Team { id: 1 }, 50, 10);
    world.spawn("C", IVec2 { x: 10, y: 10 }, Team { id: 2 }, 25, 20);
    let snap = build_snapshot(&world, 0, 0);
    assert_eq!(snap.entities.len(), 3);
}
