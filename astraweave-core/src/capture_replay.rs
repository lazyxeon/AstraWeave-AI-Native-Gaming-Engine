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
