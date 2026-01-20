// Integration test for simulation determinism and replay
use astraweave_core::{capture_replay, step, SimConfig, World};

#[test]
fn test_simulation_determinism() {
    let mut world1 = World::default();
    let mut world2 = World::default();
    let cfg = SimConfig { dt: 0.016 };
    for _ in 0..100 {
        step(&mut world1, &cfg);
        step(&mut world2, &cfg);
    }
    // Compare world state hashes or key fields
    assert_eq!(world1.t, world2.t);
    // TODO: Compare more fields for full determinism
}

#[test]
fn test_capture_and_replay_minimal() {
    let mut world = World::default();
    world.obstacles.insert((1, 2));
    world.obstacles.insert((2, 3));
    let path = "test.snap";
    capture_replay::capture_state(5, path, &world).unwrap_or_else(|e| {
        panic!("failed to capture state (seed: 5, path: {}): {e}", path)
    });
    let cfg = SimConfig { dt: 0.016 };
    let w2 = capture_replay::replay_state(path, 3, &cfg).unwrap_or_else(|e| {
        panic!("failed to replay state (seed: 3, path: {}): {e}", path)
    });
    // Basic invariants: replay doesn't drop obstacles order-independently
    assert!(
        w2.obstacles.contains(&(1, 2)),
        "replayed state missing obstacle (1,2): {:?}",
        w2.obstacles
    );
    assert!(
        w2.obstacles.contains(&(2, 3)),
        "replayed state missing obstacle (2,3): {:?}",
        w2.obstacles
    );
    std::fs::remove_file(path).ok();
}
