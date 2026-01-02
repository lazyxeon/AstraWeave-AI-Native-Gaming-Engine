#![cfg(test)]

use crate::Interest;
use crate::{
    apply_delta, build_snapshot, diff_snapshots, filter_snapshot_for_viewer, FullInterest,
};
use astraweave_core::{IVec2, Team, World};

fn make_world() -> World {
    let mut w = World::new();
    w.obstacles.insert((1, 1));
    let _p = w.spawn("P", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 10);
    let _c = w.spawn("C", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 5);
    let _e = w.spawn("E", IVec2 { x: 5, y: 2 }, Team { id: 2 }, 60, 0);
    w
}

#[test]
fn snapshot_hash_stable() {
    let w1 = make_world();
    let s1 = build_snapshot(&w1, 1, 0);
    let w2 = make_world();
    let s2 = build_snapshot(&w2, 1, 0);
    assert_eq!(s1.world_hash, s2.world_hash);
}

#[test]
fn delta_roundtrip() {
    let mut w = make_world();
    let a = build_snapshot(&w, 1, 0);
    // mutate world
    if let Some(p) = w.pose_mut(1) {
        p.pos.x += 1;
    }
    if let Some(h) = w.health_mut(2) {
        h.hp -= 5;
    }
    if let Some(a) = w.ammo_mut(3) {
        a.rounds += 2;
    }
    let b = build_snapshot(&w, 2, 1);
    let viewer = a.entities.first().cloned().unwrap();
    let d = diff_snapshots(&a, &b, &FullInterest, &viewer);
    let mut x = a.clone();
    apply_delta(&mut x, &d);
    assert_eq!(x.tick, b.tick);
    assert_eq!(x.entities.len(), b.entities.len());
    // Compare maps
    let map_x: std::collections::BTreeMap<_, _> = x.entities.iter().map(|e| (e.id, e)).collect();
    let map_b: std::collections::BTreeMap<_, _> = b.entities.iter().map(|e| (e.id, e)).collect();
    for (id, ex) in map_x.iter() {
        let eb = map_b.get(id).unwrap();
        assert_eq!(ex, eb);
    }
}

#[test]
fn delta_removal_on_interest_change() {
    use crate::RadiusTeamInterest;
    let mut w = make_world();
    let a = build_snapshot(&w, 1, 0);
    // viewer = team 0 entity
    let viewer = a.entities.iter().find(|e| e.team == 0).cloned().unwrap();
    // Identify an enemy and move it far away so it drops out of interest
    let enemy_id = a.entities.iter().find(|e| e.team == 2).unwrap().id;
    if let Some(p) = w.pose_mut(enemy_id) {
        p.pos = IVec2 { x: 50, y: 50 };
    }
    let b = build_snapshot(&w, 2, 1);
    let policy = RadiusTeamInterest { radius: 5 };
    let base_f = filter_snapshot_for_viewer(&a, &policy, &viewer);
    let head_f = filter_snapshot_for_viewer(&b, &policy, &viewer);
    let d = diff_snapshots(&base_f, &head_f, &FullInterest, &viewer);
    assert!(
        d.removed.contains(&enemy_id),
        "enemy should be removed when it leaves interest"
    );
    let mut x = base_f.clone();
    apply_delta(&mut x, &d);
    assert_eq!(x.tick, head_f.tick);
    assert_eq!(x.entities.len(), head_f.entities.len());
    assert!(x.entities.iter().all(|e| e.id != enemy_id));
}

#[test]
fn interest_filter_basic() {
    use crate::{EntityState, RadiusTeamInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let ally = EntityState {
        id: 2,
        pos: IVec2 { x: 100, y: 100 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let near_enemy = EntityState {
        id: 3,
        pos: IVec2 { x: 2, y: 2 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let far_enemy = EntityState {
        id: 4,
        pos: IVec2 { x: 20, y: 20 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = RadiusTeamInterest { radius: 5 };
    assert!(policy.include(&viewer, &ally), "allies always included");
    assert!(policy.include(&viewer, &near_enemy), "near enemy included");
    assert!(!policy.include(&viewer, &far_enemy), "far enemy excluded");
}

#[test]
fn replay_determinism() {
    use crate::{replay_from, ReplayEvent};
    use astraweave_core::{ActionStep, PlanIntent};
    // Build two identical worlds
    let mut w1 = World::new();
    let p1 = w1.spawn("P", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let c1 = w1.spawn("C", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 10);
    let _e1 = w1.spawn("E", IVec2 { x: 7, y: 2 }, Team { id: 2 }, 60, 0);

    let mut w2 = World::new();
    let _p2 = w2.spawn("P", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let _c2 = w2.spawn("C", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 10);
    let _e2 = w2.spawn("E", IVec2 { x: 7, y: 2 }, Team { id: 2 }, 60, 0);

    let evs = vec![
        ReplayEvent {
            tick: 5,
            seq: 0,
            actor_id: c1,
            intent: PlanIntent {
                plan_id: "mv".into(),
                steps: vec![ActionStep::MoveTo {
                    x: 4,
                    y: 2,
                    speed: None,
                }],
            },
            world_hash: 0,
        },
        ReplayEvent {
            tick: 10,
            seq: 0,
            actor_id: p1,
            intent: PlanIntent {
                plan_id: "mv".into(),
                steps: vec![ActionStep::MoveTo {
                    x: 3,
                    y: 2,
                    speed: None,
                }],
            },
            world_hash: 0,
        },
    ];
    let h1 = replay_from(w1, &evs).unwrap();
    let h2 = replay_from(w2, &evs).unwrap();
    assert_eq!(h1, h2, "replay should be deterministic and consistent");
}

#[test]
fn multi_client_consistency_filtered_hash() {
    use crate::{subset_hash, RadiusTeamInterest};
    // Setup world and two viewers
    let mut w = make_world();
    let snap0 = build_snapshot(&w, 0, 0);
    let viewer_a = snap0
        .entities
        .iter()
        .find(|e| e.team == 0)
        .cloned()
        .unwrap();
    let viewer_b = snap0
        .entities
        .iter()
        .find(|e| e.team == 1)
        .cloned()
        .unwrap();
    let policy = RadiusTeamInterest { radius: 6 };

    // Each client keeps its own filtered base and hash history
    let mut a = filter_snapshot_for_viewer(&snap0, &policy, &viewer_a);
    let mut b = filter_snapshot_for_viewer(&snap0, &policy, &viewer_b);
    let mut a_hashes = vec![a.world_hash];
    let mut b_hashes = vec![b.world_hash];

    // Advance world a few ticks and broadcast filtered snapshots; apply deltas on each client
    for tick in 1..=10u64 {
        // simple world change: move team 2 enemy to the right each tick
        if let Some(enemy) = w.all_of_team(2).first().cloned() {
            if let Some(p) = w.pose_mut(enemy) {
                p.pos.x += 1;
            }
        }
        let snap = build_snapshot(&w, tick, tick as u32);
        let head_a = filter_snapshot_for_viewer(&snap, &policy, &viewer_a);
        let head_b = filter_snapshot_for_viewer(&snap, &policy, &viewer_b);
        let d_a = diff_snapshots(&a, &head_a, &FullInterest, &viewer_a);
        let d_b = diff_snapshots(&b, &head_b, &FullInterest, &viewer_b);
        apply_delta(&mut a, &d_a);
        apply_delta(&mut b, &d_b);
        a_hashes.push(a.world_hash);
        b_hashes.push(b.world_hash);
    }

    // We don't expect a_hashes == b_hashes because views differ, but they should each be consistent
    assert_eq!(a.world_hash, subset_hash(&a.entities));
    assert_eq!(b.world_hash, subset_hash(&b.entities));
}

#[test]
fn fov_los_interest_blocks_through_wall() {
    use crate::{EntityState, FovLosInterest};
    // Viewer at (0,0) facing +X with 45deg half-angle and radius 10
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let enemy_visible = EntityState {
        id: 2,
        pos: IVec2 { x: 5, y: 0 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let enemy_blocked = EntityState {
        id: 3,
        pos: IVec2 { x: 5, y: 2 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let mut obstacles = std::collections::BTreeSet::new();
    // Wall at x=3 for y in [1..3]
    obstacles.insert((3, 1));
    obstacles.insert((3, 2));
    let policy = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles,
    };
    assert!(
        policy.include(&viewer, &enemy_visible),
        "enemy directly ahead with LOS should be included"
    );
    assert!(
        !policy.include(&viewer, &enemy_blocked),
        "enemy behind wall should be excluded by LOS"
    );
}

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn test_entity_state_equality() {
    use crate::EntityState;
    let e1 = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 10 },
        hp: 100,
        team: 0,
        ammo: 30,
    };
    let e2 = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 10 },
        hp: 100,
        team: 0,
        ammo: 30,
    };
    let e3 = EntityState {
        id: 1,
        pos: IVec2 { x: 6, y: 10 },
        hp: 100,
        team: 0,
        ammo: 30,
    };
    assert_eq!(e1, e2);
    assert_ne!(e1, e3);
}

#[test]
fn test_entity_state_clone() {
    use crate::EntityState;
    let e1 = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 10 },
        hp: 100,
        team: 0,
        ammo: 30,
    };
    let e2 = e1.clone();
    assert_eq!(e1.id, e2.id);
    assert_eq!(e1.pos, e2.pos);
}

#[test]
fn test_entity_delta_mask_flags() {
    use crate::EntityDeltaMask;
    assert_eq!(EntityDeltaMask::POS, 1);
    assert_eq!(EntityDeltaMask::HP, 2);
    assert_eq!(EntityDeltaMask::TEAM, 4);
    assert_eq!(EntityDeltaMask::AMMO, 8);
}

#[test]
fn test_entity_delta_equality() {
    use crate::EntityDelta;
    let d1 = EntityDelta {
        id: 1,
        mask: 3, // POS | HP
        pos: Some(IVec2 { x: 5, y: 5 }),
        hp: Some(80),
        team: None,
        ammo: None,
    };
    let d2 = d1.clone();
    assert_eq!(d1, d2);
}

#[test]
fn test_delta_equality() {
    use crate::Delta;
    let d1 = Delta {
        base_tick: 0,
        tick: 1,
        changed: vec![],
        removed: vec![],
        head_hash: 12345,
    };
    let d2 = d1.clone();
    assert_eq!(d1.base_tick, d2.base_tick);
    assert_eq!(d1.tick, d2.tick);
    assert_eq!(d1.head_hash, d2.head_hash);
}

#[test]
fn test_snapshot_clone() {
    let w = make_world();
    let s1 = build_snapshot(&w, 1, 0);
    let s2 = s1.clone();
    assert_eq!(s1.tick, s2.tick);
    assert_eq!(s1.world_hash, s2.world_hash);
    assert_eq!(s1.entities.len(), s2.entities.len());
}

#[test]
fn test_full_interest_includes_all() {
    use crate::{EntityState, FullInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let far_entity = EntityState {
        id: 2,
        pos: IVec2 { x: 1000, y: 1000 },
        hp: 100,
        team: 2,
        ammo: 0,
    };
    let policy = FullInterest;
    assert!(policy.include(&viewer, &far_entity));
}

#[test]
fn test_radius_interest_same_team_always_included() {
    use crate::{EntityState, RadiusTeamInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let far_ally = EntityState {
        id: 2,
        pos: IVec2 { x: 1000, y: 1000 },
        hp: 100,
        team: 0, // Same team
        ammo: 0,
    };
    let policy = RadiusTeamInterest { radius: 5 };
    assert!(policy.include(&viewer, &far_ally)); // Same team always included
}

#[test]
fn test_fov_interest_basic() {
    use crate::{EntityState, FovInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    // Enemy directly in front
    let enemy_ahead = EntityState {
        id: 2,
        pos: IVec2 { x: 5, y: 0 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    // Enemy behind
    let enemy_behind = EntityState {
        id: 3,
        pos: IVec2 { x: -5, y: 0 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    assert!(policy.include(&viewer, &enemy_ahead));
    assert!(!policy.include(&viewer, &enemy_behind));
}

#[test]
fn test_fov_interest_same_position() {
    use crate::{EntityState, FovInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    // Entity at same position
    let same_pos = EntityState {
        id: 2,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    assert!(policy.include(&viewer, &same_pos)); // Same position should be included
}

#[test]
fn test_fov_interest_zero_facing() {
    use crate::{EntityState, FovInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let enemy = EntityState {
        id: 2,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 0, y: 0 }, // Zero facing
    };
    assert!(policy.include(&viewer, &enemy)); // Should include when facing is zero
}

#[test]
fn test_fov_interest_out_of_range() {
    use crate::{EntityState, FovInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let far_enemy = EntityState {
        id: 2,
        pos: IVec2 { x: 100, y: 0 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = FovInterest {
        radius: 10,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    assert!(!policy.include(&viewer, &far_enemy)); // Out of range
}

#[test]
fn test_fov_los_interest_same_team() {
    use crate::{EntityState, FovLosInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let ally = EntityState {
        id: 2,
        pos: IVec2 { x: 50, y: 50 },
        hp: 100,
        team: 0, // Same team
        ammo: 0,
    };
    let policy = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: std::collections::BTreeSet::new(),
    };
    assert!(policy.include(&viewer, &ally)); // Same team always included
}

#[test]
fn test_fov_los_interest_zero_facing() {
    use crate::{EntityState, FovLosInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let enemy = EntityState {
        id: 2,
        pos: IVec2 { x: 3, y: 3 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 0, y: 0 }, // Zero facing
        obstacles: std::collections::BTreeSet::new(),
    };
    assert!(policy.include(&viewer, &enemy)); // Should use LOS only when facing is zero
}

#[test]
fn test_interest_policy_variants() {
    use crate::InterestPolicy;
    let policy1 = InterestPolicy::Radius { radius: 10 };
    let policy2 = InterestPolicy::Fov {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    let policy3 = InterestPolicy::FovLos {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    // Just test they can be created and cloned
    let _p1 = policy1.clone();
    let _p2 = policy2.clone();
    let _p3 = policy3.clone();
}

#[test]
fn test_apply_delta_mismatched_tick() {
    use crate::{Delta, Snapshot};
    let mut snap = Snapshot {
        version: 1,
        tick: 5,
        t: 0.5,
        seq: 0,
        world_hash: 0,
        entities: vec![],
    };
    let delta = Delta {
        base_tick: 10, // Mismatched
        tick: 11,
        changed: vec![],
        removed: vec![],
        head_hash: 0,
    };
    apply_delta(&mut snap, &delta);
    // Delta should not be applied due to tick mismatch
    assert_eq!(snap.tick, 5);
}

#[test]
fn test_apply_delta_creates_new_entity() {
    use crate::{Delta, EntityDelta, EntityDeltaMask, Snapshot};
    let mut snap = Snapshot {
        version: 1,
        tick: 1,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![],
    };
    let delta = Delta {
        base_tick: 1,
        tick: 2,
        changed: vec![EntityDelta {
            id: 100,
            mask: EntityDeltaMask::POS | EntityDeltaMask::HP | EntityDeltaMask::TEAM | EntityDeltaMask::AMMO,
            pos: Some(IVec2 { x: 10, y: 20 }),
            hp: Some(75),
            team: Some(1),
            ammo: Some(25),
        }],
        removed: vec![],
        head_hash: 12345,
    };
    apply_delta(&mut snap, &delta);
    assert_eq!(snap.tick, 2);
    assert_eq!(snap.entities.len(), 1);
    let entity = &snap.entities[0];
    assert_eq!(entity.id, 100);
    assert_eq!(entity.pos.x, 10);
    assert_eq!(entity.hp, 75);
}

#[test]
fn test_diff_snapshots_new_entity() {
    use crate::{EntityState, Snapshot};
    let base = Snapshot {
        version: 1,
        tick: 1,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![],
    };
    let head = Snapshot {
        version: 1,
        tick: 2,
        t: 0.0,
        seq: 1,
        world_hash: 0,
        entities: vec![EntityState {
            id: 1,
            pos: IVec2 { x: 5, y: 5 },
            hp: 100,
            team: 0,
            ammo: 10,
        }],
    };
    let viewer = EntityState {
        id: 0,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 1);
    assert!(delta.removed.is_empty());
    let change = &delta.changed[0];
    assert_eq!(change.id, 1);
    // All fields should be set for a new entity
    assert!(change.pos.is_some());
    assert!(change.hp.is_some());
    assert!(change.team.is_some());
    assert!(change.ammo.is_some());
}

#[test]
fn test_diff_snapshots_no_change() {
    use crate::EntityState;
    let state = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 0,
        ammo: 10,
    };
    let snap = crate::Snapshot {
        version: 1,
        tick: 1,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![state.clone()],
    };
    let viewer = state.clone();
    let delta = diff_snapshots(&snap, &snap, &FullInterest, &viewer);
    assert!(delta.changed.is_empty());
    assert!(delta.removed.is_empty());
}

#[test]
fn test_msg_serialization() {
    use crate::Msg;
    let msg = Msg::ClientHello {
        name: "test".to_string(),
        token: Some("dev".to_string()),
        policy: Some("radius".to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: Msg = serde_json::from_str(&json).unwrap();
    match decoded {
        Msg::ClientHello { name, token, policy } => {
            assert_eq!(name, "test");
            assert_eq!(token, Some("dev".to_string()));
            assert_eq!(policy, Some("radius".to_string()));
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_msg_welcome_serialization() {
    use crate::Msg;
    let msg = Msg::ServerWelcome { id: 42 };
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: Msg = serde_json::from_str(&json).unwrap();
    match decoded {
        Msg::ServerWelcome { id } => assert_eq!(id, 42),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_msg_apply_result_serialization() {
    use crate::Msg;
    let msg = Msg::ServerApplyResult {
        ok: false,
        err: Some("Out of bounds".to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: Msg = serde_json::from_str(&json).unwrap();
    match decoded {
        Msg::ServerApplyResult { ok, err } => {
            assert!(!ok);
            assert_eq!(err, Some("Out of bounds".to_string()));
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_msg_ack_serialization() {
    use crate::Msg;
    let msg = Msg::ServerAck {
        seq: 5,
        tick_applied: 100,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: Msg = serde_json::from_str(&json).unwrap();
    match decoded {
        Msg::ServerAck { seq, tick_applied } => {
            assert_eq!(seq, 5);
            assert_eq!(tick_applied, 100);
        }
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_replay_event_serialization() {
    use crate::ReplayEvent;
    use astraweave_core::{ActionStep, PlanIntent};
    let event = ReplayEvent {
        tick: 50,
        seq: 1,
        actor_id: 5,
        intent: PlanIntent {
            plan_id: "test".to_string(),
            steps: vec![ActionStep::Scan { radius: 10.0 }],
        },
        world_hash: 12345,
    };
    let json = serde_json::to_string(&event).unwrap();
    let decoded: ReplayEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.tick, 50);
    assert_eq!(decoded.seq, 1);
    assert_eq!(decoded.actor_id, 5);
    assert_eq!(decoded.world_hash, 12345);
}

#[test]
fn test_game_server_default() {
    use crate::GameServer;
    let server = GameServer::default();
    assert!(server.player_id > 0);
    assert!(server.companion_id > 0);
    assert!(server.enemy_id > 0);
}

#[test]
fn test_server_event_clone() {
    use crate::{ServerEvent, Snapshot};
    let event = ServerEvent::Snapshot(Snapshot {
        version: 1,
        tick: 1,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![],
    });
    let _cloned = event.clone();

    let result = ServerEvent::ApplyResult {
        ok: true,
        err: None,
    };
    let _cloned_result = result.clone();

    let ack = ServerEvent::Ack {
        seq: 1,
        tick_applied: 10,
    };
    let _cloned_ack = ack.clone();
}

#[test]
fn test_has_los_negative_sx() {
    use crate::has_los;
    let mut obstacles = std::collections::BTreeSet::new();
    let a = IVec2 { x: 5, y: 0 };
    let b = IVec2 { x: 0, y: 0 };
    // x0 > x1, so sx should be -1
    assert!(has_los(a, b, &obstacles));
    
    obstacles.insert((2, 0));
    assert!(!has_los(a, b, &obstacles));
}

#[test]
fn test_fov_los_interest_out_of_range() {
    use crate::{EntityState, FovLosInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let far_enemy = EntityState {
        id: 2,
        pos: IVec2 { x: 20, y: 0 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: std::collections::BTreeSet::new(),
    };
    assert!(!policy.include(&viewer, &far_enemy));
}

#[test]
fn test_fov_los_interest_same_position_enemy() {
    use crate::{EntityState, FovLosInterest};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let enemy = EntityState {
        id: 2,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let policy = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles: std::collections::BTreeSet::new(),
    };
    assert!(policy.include(&viewer, &enemy));
}

#[test]
fn test_diff_snapshots_team_change() {
    use crate::{EntityState, Snapshot, FullInterest, diff_snapshots, EntityDeltaMask};
    let base_state = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 10,
    };
    let head_state = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1, // Team changed
        ammo: 10,
    };
    let base = Snapshot {
        version: 1,
        tick: 1,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![base_state.clone()],
    };
    let head = Snapshot {
        version: 1,
        tick: 2,
        t: 0.0,
        seq: 1,
        world_hash: 0,
        entities: vec![head_state],
    };
    let viewer = base_state;
    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);
    assert_eq!(delta.changed.len(), 1);
    assert_eq!(delta.changed[0].mask & EntityDeltaMask::TEAM, EntityDeltaMask::TEAM);
    assert_eq!(delta.changed[0].team, Some(1));
}

#[test]
fn test_diff_snapshots_with_exclusion() {
    use crate::{EntityState, Snapshot, RadiusTeamInterest, diff_snapshots};
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 0,
    };
    let near_enemy = EntityState {
        id: 2,
        pos: IVec2 { x: 1, y: 1 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let far_enemy = EntityState {
        id: 3,
        pos: IVec2 { x: 100, y: 100 },
        hp: 100,
        team: 1,
        ammo: 0,
    };
    let base = Snapshot {
        version: 1,
        tick: 1,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![viewer.clone(), near_enemy.clone()],
    };
    let head = Snapshot {
        version: 1,
        tick: 2,
        t: 0.0,
        seq: 1,
        world_hash: 0,
        entities: vec![viewer.clone(), near_enemy, far_enemy],
    };
    let policy = RadiusTeamInterest { radius: 10 };
    let delta = diff_snapshots(&base, &head, &policy, &viewer);
    // far_enemy should be excluded by policy, so it shouldn't appear in changed
    assert!(delta.changed.is_empty());
}
