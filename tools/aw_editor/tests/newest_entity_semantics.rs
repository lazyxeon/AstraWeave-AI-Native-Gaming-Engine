//! Pins the World-id invariants that the editor's `newest_entity()` helper
//! (main.rs) depends on after the entities().last() bug.
//!
//! Root cause being guarded: `World::entities()` collects HashMap keys in
//! arbitrary order, so `.last()` is NOT the most recent spawn. The editor
//! previously used it to find "the entity I just spawned" — after any delete
//! rehashed the map, every post-spawn mesh assignment and selection landed on
//! one arbitrary fixed entity. The fix identifies the newest entity as
//! `iter_entities().max()`, which is only correct while World ids are
//! monotonic and never reused. If id recycling is ever introduced, this test
//! must fail so the editor helper is redesigned alongside it.

use astraweave_core::{IVec2, Team, World};

fn spawn(w: &mut World, name: &str) -> u32 {
    w.spawn(name, IVec2 { x: 0, y: 0 }, Team { id: 0 }, 1, 0)
}

#[test]
fn spawn_ids_are_monotonic_and_never_reused_after_delete() {
    let mut w = World::new();
    let a = spawn(&mut w, "A");
    let b = spawn(&mut w, "B");
    let c = spawn(&mut w, "C");
    assert!(a < b && b < c, "spawn ids must be strictly increasing");

    // Delete an entity, then spawn again — the freed id must NOT be reused.
    assert!(w.destroy_entity(b));
    let d = spawn(&mut w, "D");
    assert!(
        d > c,
        "freed id must not be recycled (got {d} after deleting {b}, last id {c}) — \
         if id recycling is introduced, redesign newest_entity() in aw_editor main.rs"
    );

    // The invariant the editor helper relies on: max id == most recent spawn.
    let newest = w.iter_entities().max();
    assert_eq!(newest, Some(d), "iter_entities().max() must be the newest spawn");
}

#[test]
fn entities_vec_order_is_not_a_recency_signal() {
    // Documents WHY entities().last() was wrong: after a delete + respawn the
    // HashMap ordering carries no recency meaning. We can't assert a specific
    // wrong order (it's arbitrary), but we can assert the CORRECT recency
    // signal (max id) regardless of Vec order.
    let mut w = World::new();
    for i in 0..8 {
        spawn(&mut w, &format!("E{i}"));
    }
    w.destroy_entity(3);
    w.destroy_entity(5);
    let newest = spawn(&mut w, "newest");

    let via_max = w.iter_entities().max();
    assert_eq!(via_max, Some(newest));
    // entities() must contain the same set, whatever its order.
    let mut all = w.entities();
    all.sort_unstable();
    assert_eq!(all.last().copied(), Some(newest));
    assert_eq!(all.len(), 7);
}
