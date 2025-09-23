#![cfg(test)]

use crate::{apply_delta, build_snapshot, diff_snapshots, filter_snapshot_for_viewer, FullInterest};
use crate::Interest;
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
    if let Some(p) = w.pose_mut(1) { p.pos.x += 1; }
    if let Some(h) = w.health_mut(2) { h.hp -= 5; }
    if let Some(a) = w.ammo_mut(3) { a.rounds += 2; }
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
    if let Some(p) = w.pose_mut(enemy_id) { p.pos = IVec2 { x: 50, y: 50 }; }
    let b = build_snapshot(&w, 2, 1);
    let policy = RadiusTeamInterest { radius: 5 };
    let base_f = filter_snapshot_for_viewer(&a, &policy, &viewer);
    let head_f = filter_snapshot_for_viewer(&b, &policy, &viewer);
    let d = diff_snapshots(&base_f, &head_f, &FullInterest, &viewer);
    assert!(d.removed.contains(&enemy_id), "enemy should be removed when it leaves interest");
    let mut x = base_f.clone();
    apply_delta(&mut x, &d);
    assert_eq!(x.tick, head_f.tick);
    assert_eq!(x.entities.len(), head_f.entities.len());
    assert!(x.entities.iter().all(|e| e.id != enemy_id));
}

#[test]
fn interest_filter_basic() {
    use crate::{EntityState, RadiusTeamInterest};
    let viewer = EntityState { id: 1, pos: IVec2 { x: 0, y: 0 }, hp: 100, team: 0, ammo: 0 };
    let ally = EntityState { id: 2, pos: IVec2 { x: 100, y: 100 }, hp: 100, team: 0, ammo: 0 };
    let near_enemy = EntityState { id: 3, pos: IVec2 { x: 2, y: 2 }, hp: 100, team: 1, ammo: 0 };
    let far_enemy = EntityState { id: 4, pos: IVec2 { x: 20, y: 20 }, hp: 100, team: 1, ammo: 0 };
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
        ReplayEvent { tick: 5, seq: 0, actor_id: c1, intent: PlanIntent { plan_id: "mv".into(), steps: vec![ActionStep::MoveTo { x: 4, y: 2 }] }, world_hash: 0 },
        ReplayEvent { tick: 10, seq: 0, actor_id: p1, intent: PlanIntent { plan_id: "mv".into(), steps: vec![ActionStep::MoveTo { x: 3, y: 2 }] }, world_hash: 0 },
    ];
    let h1 = replay_from(w1, &evs).unwrap();
    let h2 = replay_from(w2, &evs).unwrap();
    assert_eq!(h1, h2, "replay should be deterministic and consistent");
}

#[test]
fn multi_client_consistency_filtered_hash() {
    use crate::{RadiusTeamInterest, subset_hash};
    // Setup world and two viewers
    let mut w = make_world();
    let snap0 = build_snapshot(&w, 0, 0);
    let viewer_a = snap0.entities.iter().find(|e| e.team == 0).cloned().unwrap();
    let viewer_b = snap0.entities.iter().find(|e| e.team == 1).cloned().unwrap();
    let policy = RadiusTeamInterest { radius: 6 };

    // Each client keeps its own filtered base and hash history
    let mut a = filter_snapshot_for_viewer(&snap0, &policy, &viewer_a);
    let mut b = filter_snapshot_for_viewer(&snap0, &policy, &viewer_b);
    let mut a_hashes = vec![a.world_hash];
    let mut b_hashes = vec![b.world_hash];

    // Advance world a few ticks and broadcast filtered snapshots; apply deltas on each client
    for tick in 1..=10u64 {
        // simple world change: move team 2 enemy to the right each tick
        if let Some(enemy) = w.all_of_team(2).first().cloned() { if let Some(p) = w.pose_mut(enemy) { p.pos.x += 1; } }
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
    let viewer = EntityState { id: 1, pos: IVec2 { x: 0, y: 0 }, hp: 100, team: 0, ammo: 0 };
    let enemy_visible = EntityState { id: 2, pos: IVec2 { x: 5, y: 0 }, hp: 100, team: 1, ammo: 0 };
    let enemy_blocked = EntityState { id: 3, pos: IVec2 { x: 5, y: 2 }, hp: 100, team: 1, ammo: 0 };
    let mut obstacles = std::collections::BTreeSet::new();
    // Wall at x=3 for y in [1..3]
    obstacles.insert((3, 1));
    obstacles.insert((3, 2));
    let policy = FovLosInterest { radius: 10, half_angle_deg: 45.0, facing: IVec2 { x: 1, y: 0 }, obstacles };
    assert!(policy.include(&viewer, &enemy_visible), "enemy directly ahead with LOS should be included");
    assert!(!policy.include(&viewer, &enemy_blocked), "enemy behind wall should be excluded by LOS");
}
