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
fn test_capture_and_replay_stub() {
    let res = capture_replay::capture_state(0, "test.snap");
    assert!(res.is_err());
    let res = capture_replay::replay_state("test.snap");
    assert!(res.is_err());
}
