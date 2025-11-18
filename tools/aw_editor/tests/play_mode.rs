use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use astraweave_core::{IVec2, Team, World};
use aw_editor_lib::runtime::{EditorRuntime, RuntimeState};

fn sample_world() -> World {
    let mut world = World::new();
    for i in 0..5 {
        world.spawn(
            &format!("Agent_{i}"),
            IVec2 { x: i * 2, y: i * 3 },
            Team { id: (i % 2) as u8 },
            100 - (i as i32),
            10 + i as i32,
        );
    }
    world
}

fn hash_world(world: &World) -> u64 {
    let mut ids = world.entities();
    ids.sort_unstable();

    let mut hasher = DefaultHasher::new();
    (world.t.to_bits()).hash(&mut hasher);
    for id in ids {
        if let Some(pose) = world.pose(id) {
            pose.pos.x.hash(&mut hasher);
            pose.pos.y.hash(&mut hasher);
        }
        if let Some(team) = world.team(id) {
            team.id.hash(&mut hasher);
        }
        if let Some(ammo) = world.ammo(id) {
            ammo.rounds.hash(&mut hasher);
        }
        if let Some(health) = world.health(id) {
            health.hp.hash(&mut hasher);
        }
    }
    hasher.finish()
}

#[test]
fn test_play_preserves_scene() {
    let world = sample_world();
    let mut runtime = EditorRuntime::new();

    runtime.enter_play(&world).expect("enter play");
    let sim_world = runtime
        .sim_world()
        .expect("simulation world should exist while playing");
    assert_eq!(sim_world.entities().len(), world.entities().len());
}

#[test]
fn test_stop_restores_edits() {
    let mut runtime = EditorRuntime::new();
    let mut world = sample_world();
    let baseline_hash = hash_world(&world);

    runtime.enter_play(&world).expect("enter play");
    {
        let sim_world = runtime.sim_world_mut().expect("sim world available");
        let e = sim_world.entities()[0];
        if let Some(pose) = sim_world.pose_mut(e) {
            pose.pos.x += 99;
        }
    }

    let restored = runtime.exit_play().expect("exit play");
    assert!(runtime.sim_world().is_none());

    let restored_world = restored.expect("world restored");
    assert_eq!(hash_world(&restored_world), baseline_hash);
}

#[test]
fn test_step_frame_advances_one_tick() {
    let world = sample_world();
    let mut runtime = EditorRuntime::new();

    runtime.enter_play(&world).expect("enter play");
    runtime.pause();
    runtime.step_frame().expect("step frame");
    assert_eq!(runtime.stats().tick_count, 1);
    assert_eq!(runtime.state(), RuntimeState::Paused);
}

#[test]
fn test_deterministic_replay() {
    let world = sample_world();
    let mut runtime = EditorRuntime::new();

    runtime.enter_play(&world).expect("enter play");
    for _ in 0..100 {
        runtime.tick(1.0 / 60.0).expect("tick simulation");
    }
    let hash_a = hash_world(runtime.sim_world().expect("sim world"));

    runtime.exit_play().expect("exit play");
    runtime.enter_play(&world).expect("enter play again");
    for _ in 0..100 {
        runtime.tick(1.0 / 60.0).expect("tick simulation");
    }
    let hash_b = hash_world(runtime.sim_world().expect("sim world"));

    assert_eq!(hash_a, hash_b);
}

#[test]
fn test_runtime_stats_accuracy() {
    let mut world = sample_world();
    world.spawn("Extra", IVec2 { x: 10, y: 10 }, Team { id: 0 }, 120, 40);

    let mut runtime = EditorRuntime::new();
    runtime.enter_play(&world).expect("enter play");

    runtime.tick(1.0 / 60.0).expect("tick once");
    let stats = runtime.stats();
    assert_eq!(stats.entity_count, world.entities().len());
    assert_eq!(stats.tick_count, 1);
    assert!(stats.frame_time_ms >= 0.0);
}
