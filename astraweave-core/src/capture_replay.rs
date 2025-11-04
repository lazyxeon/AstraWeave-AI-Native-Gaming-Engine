// ECS/AI/Physics State Capture & Replay (Phase 0 minimal)
// JSON snapshot of World and tick for smoke tests and determinism checks.

use crate::{sim::step, World};
use anyhow::{Context, Result};

#[derive(serde::Serialize, serde::Deserialize)]
struct Snapshot {
    tick: u64,
    world: WorldSerde,
}

// We can't serialize World directly due to HashMaps with non-serializable keys;
// provide a stable serde wrapper. For Phase 0, we capture only fields we need
// to rehydrate a minimal world state deterministically.
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct WorldSerde {
    t: f32,
    next_id: u32,
    obstacles: Vec<(i32, i32)>,
}

impl From<&World> for WorldSerde {
    fn from(w: &World) -> Self {
        let mut obstacles: Vec<(i32, i32)> = w.obstacles.iter().copied().collect();
        obstacles.sort_unstable();
        WorldSerde {
            t: w.t,
            next_id: w.next_id,
            obstacles,
        }
    }
}

impl World {
    fn from_serde(ws: &WorldSerde) -> Self {
        let mut w = World::new();
        w.t = ws.t;
        w.next_id = ws.next_id;
        w.obstacles = ws.obstacles.iter().copied().collect();
        w
    }
}

pub fn capture_state(tick: u64, path: &str, world: &World) -> Result<()> {
    let snap = Snapshot {
        tick,
        world: WorldSerde::from(world),
    };
    let data = serde_json::to_vec_pretty(&snap).context("serializing snapshot")?;
    std::fs::write(path, data).context(format!("writing snapshot to {}", path))?;
    Ok(())
}

pub fn replay_state(path: &str, steps: u32, cfg: &crate::SimConfig) -> anyhow::Result<World> {
    let data = std::fs::read(path).context(format!("failed to read snapshot file: {}", path))?;
    let snap: Snapshot =
        serde_json::from_slice(&data).context("failed to deserialize snapshot JSON")?;
    let mut w = World::from_serde(&snap.world);
    for _ in 0..steps {
        step(&mut w, cfg);
    }
    Ok(w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IVec2, SimConfig, Team};
    use std::fs;

    #[test]
    fn test_world_serde_default() {
        let ws = WorldSerde::default();
        assert_eq!(ws.t, 0.0);
        assert_eq!(ws.next_id, 0);
        assert!(ws.obstacles.is_empty());
    }

    #[test]
    fn test_world_serde_from_world_empty() {
        let w = World::new();
        let ws = WorldSerde::from(&w);

        assert_eq!(ws.t, 0.0);
        assert_eq!(ws.next_id, 1);
        assert!(ws.obstacles.is_empty());
    }

    #[test]
    fn test_world_serde_from_world_with_obstacles() {
        let mut w = World::new();
        w.obstacles.insert((5, 10));
        w.obstacles.insert((0, 0));
        w.obstacles.insert((15, 20));
        w.t = 1.5;
        w.next_id = 42;

        let ws = WorldSerde::from(&w);

        assert_eq!(ws.t, 1.5);
        assert_eq!(ws.next_id, 42);
        assert_eq!(ws.obstacles.len(), 3);
        // Obstacles should be sorted
        assert!(ws.obstacles.contains(&(0, 0)));
        assert!(ws.obstacles.contains(&(5, 10)));
        assert!(ws.obstacles.contains(&(15, 20)));
    }

    #[test]
    fn test_world_serde_obstacles_sorted() {
        let mut w = World::new();
        w.obstacles.insert((10, 10));
        w.obstacles.insert((5, 5));
        w.obstacles.insert((15, 15));

        let ws = WorldSerde::from(&w);

        // Check that obstacles are sorted (stable serialization)
        let mut prev = ws.obstacles[0];
        for &obs in ws.obstacles.iter().skip(1) {
            assert!(obs >= prev, "Obstacles should be sorted");
            prev = obs;
        }
    }

    #[test]
    fn test_world_from_serde_empty() {
        let ws = WorldSerde::default();
        let w = World::from_serde(&ws);

        assert_eq!(w.t, 0.0);
        assert_eq!(w.next_id, 0);
        assert!(w.obstacles.is_empty());
    }

    #[test]
    fn test_world_from_serde_with_data() {
        let ws = WorldSerde {
            t: 2.5,
            next_id: 100,
            obstacles: vec![(0, 0), (5, 5), (10, 10)],
        };

        let w = World::from_serde(&ws);

        assert_eq!(w.t, 2.5);
        assert_eq!(w.next_id, 100);
        assert_eq!(w.obstacles.len(), 3);
        assert!(w.obstacles.contains(&(0, 0)));
        assert!(w.obstacles.contains(&(5, 5)));
        assert!(w.obstacles.contains(&(10, 10)));
    }

    #[test]
    fn test_world_serde_roundtrip() {
        let mut w1 = World::new();
        w1.t = 3.14;
        w1.next_id = 999;
        w1.obstacles.insert((1, 2));
        w1.obstacles.insert((3, 4));

        let ws = WorldSerde::from(&w1);
        let w2 = World::from_serde(&ws);

        assert_eq!(w2.t, w1.t);
        assert_eq!(w2.next_id, w1.next_id);
        assert_eq!(w2.obstacles, w1.obstacles);
    }

    #[test]
    fn test_snapshot_serialization() {
        let snap = Snapshot {
            tick: 42,
            world: WorldSerde {
                t: 1.5,
                next_id: 10,
                obstacles: vec![(0, 0), (5, 5)],
            },
        };

        let json = serde_json::to_vec_pretty(&snap).unwrap();
        let deserialized: Snapshot = serde_json::from_slice(&json).unwrap();

        assert_eq!(deserialized.tick, 42);
        assert_eq!(deserialized.world.t, 1.5);
        assert_eq!(deserialized.world.next_id, 10);
        assert_eq!(deserialized.world.obstacles.len(), 2);
    }

    #[test]
    fn test_capture_state_creates_file() {
        let temp_path = "test_capture_state.json";
        let mut w = World::new();
        w.t = 5.0;
        w.next_id = 50;
        w.obstacles.insert((10, 20));

        let result = capture_state(100, temp_path, &w);
        assert!(result.is_ok(), "capture_state should succeed");

        // Verify file was created
        assert!(fs::metadata(temp_path).is_ok(), "File should exist");

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_capture_state_file_content() {
        let temp_path = "test_capture_content.json";
        let mut w = World::new();
        w.t = 7.5;
        w.next_id = 75;
        w.obstacles.insert((1, 2));

        capture_state(200, temp_path, &w).unwrap();

        // Read and verify content
        let data = fs::read(temp_path).unwrap();
        let snap: Snapshot = serde_json::from_slice(&data).unwrap();

        assert_eq!(snap.tick, 200);
        assert_eq!(snap.world.t, 7.5);
        assert_eq!(snap.world.next_id, 75);
        assert_eq!(snap.world.obstacles.len(), 1);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_capture_state_overwrites_existing() {
        let temp_path = "test_capture_overwrite.json";

        // First capture
        let mut w1 = World::new();
        w1.t = 1.0;
        capture_state(1, temp_path, &w1).unwrap();

        // Second capture (should overwrite)
        let mut w2 = World::new();
        w2.t = 2.0;
        capture_state(2, temp_path, &w2).unwrap();

        // Verify only second capture exists
        let data = fs::read(temp_path).unwrap();
        let snap: Snapshot = serde_json::from_slice(&data).unwrap();
        assert_eq!(snap.tick, 2);
        assert_eq!(snap.world.t, 2.0);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_replay_state_loads_file() {
        let temp_path = "test_replay_load.json";
        let mut w = World::new();
        w.t = 3.0;
        w.next_id = 30;
        w.obstacles.insert((5, 5));

        capture_state(50, temp_path, &w).unwrap();

        let cfg = SimConfig { dt: 0.1 };
        let result = replay_state(temp_path, 0, &cfg);

        assert!(result.is_ok(), "replay_state should succeed");
        let loaded = result.unwrap();
        assert_eq!(loaded.t, 3.0);
        assert_eq!(loaded.next_id, 30);
        assert!(loaded.obstacles.contains(&(5, 5)));

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_replay_state_with_steps() {
        let temp_path = "test_replay_steps.json";
        let mut w = World::new();
        w.t = 0.0;
        w.obstacles.insert((0, 0));

        capture_state(0, temp_path, &w).unwrap();

        let cfg = SimConfig { dt: 0.5 };
        let replayed = replay_state(temp_path, 5, &cfg).unwrap();

        // After 5 steps with dt=0.5, time should be 2.5
        assert!((replayed.t - 2.5).abs() < 1e-6);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_replay_state_zero_steps() {
        let temp_path = "test_replay_zero.json";
        let mut w = World::new();
        w.t = 10.0;

        capture_state(0, temp_path, &w).unwrap();

        let cfg = SimConfig { dt: 0.1 };
        let replayed = replay_state(temp_path, 0, &cfg).unwrap();

        // Time should remain unchanged with 0 steps
        assert_eq!(replayed.t, 10.0);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_replay_state_nonexistent_file() {
        let cfg = SimConfig { dt: 0.1 };
        let result = replay_state("nonexistent_file_12345.json", 0, &cfg);

        assert!(result.is_err(), "Should fail on nonexistent file");
    }

    #[test]
    fn test_replay_state_invalid_json() {
        let temp_path = "test_replay_invalid.json";
        fs::write(temp_path, b"{ invalid json ").unwrap();

        let cfg = SimConfig { dt: 0.1 };
        let result = replay_state(temp_path, 0, &cfg);

        assert!(result.is_err(), "Should fail on invalid JSON");

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_replay_state_wrong_structure() {
        let temp_path = "test_replay_wrong.json";
        fs::write(temp_path, br#"{"tick": 1}"#).unwrap();

        let cfg = SimConfig { dt: 0.1 };
        let result = replay_state(temp_path, 0, &cfg);

        assert!(result.is_err(), "Should fail on wrong JSON structure");

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_capture_replay_roundtrip_with_entities() {
        let temp_path = "test_roundtrip_entities.json";

        // Create world with entities
        let mut w1 = World::new();
        w1.spawn("player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
        w1.spawn("enemy", IVec2 { x: 10, y: 10 }, Team { id: 2 }, 50, 15);
        w1.obstacles.insert((5, 5));
        w1.obstacles.insert((15, 15));
        let original_time = w1.t;
        let original_next_id = w1.next_id;

        // Capture
        capture_state(42, temp_path, &w1).unwrap();

        // Replay with steps
        let cfg = SimConfig { dt: 0.2 };
        let w2 = replay_state(temp_path, 10, &cfg).unwrap();

        // Verify time advanced
        assert!((w2.t - (original_time + 2.0)).abs() < 1e-6); // 10 steps * 0.2 dt

        // Verify next_id preserved (entities not re-spawned)
        assert_eq!(w2.next_id, original_next_id);

        // Verify obstacles preserved
        assert_eq!(w2.obstacles.len(), 2);
        assert!(w2.obstacles.contains(&(5, 5)));
        assert!(w2.obstacles.contains(&(15, 15)));

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_determinism_verification() {
        let temp_path = "test_determinism.json";

        // Create initial state
        let mut w = World::new();
        w.spawn("agent", IVec2 { x: 0, y: 0 }, Team { id: 1 }, 80, 20);
        w.obstacles.insert((3, 3));

        capture_state(0, temp_path, &w).unwrap();

        // Replay twice with same config
        let cfg = SimConfig { dt: 0.1 };
        let w1 = replay_state(temp_path, 50, &cfg).unwrap();
        let w2 = replay_state(temp_path, 50, &cfg).unwrap();

        // Results should be identical (determinism)
        assert_eq!(w1.t, w2.t);
        assert_eq!(w1.next_id, w2.next_id);
        assert_eq!(w1.obstacles, w2.obstacles);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_capture_state_with_empty_world() {
        let temp_path = "test_empty_world.json";
        let w = World::new();

        let result = capture_state(0, temp_path, &w);
        assert!(result.is_ok());

        // Verify we can replay it
        let cfg = SimConfig { dt: 0.1 };
        let replayed = replay_state(temp_path, 0, &cfg).unwrap();
        assert_eq!(replayed.t, 0.0);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_capture_state_with_many_obstacles() {
        let temp_path = "test_many_obstacles.json";
        let mut w = World::new();

        // Add 100 obstacles
        for i in 0..100 {
            w.obstacles.insert((i, i * 2));
        }

        capture_state(999, temp_path, &w).unwrap();

        let cfg = SimConfig { dt: 0.1 };
        let replayed = replay_state(temp_path, 0, &cfg).unwrap();

        assert_eq!(replayed.obstacles.len(), 100);
        assert!(replayed.obstacles.contains(&(50, 100)));

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }
}
