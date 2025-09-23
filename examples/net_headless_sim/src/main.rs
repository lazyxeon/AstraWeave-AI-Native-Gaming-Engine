use anyhow::Result;
use astraweave_core::{ActionStep, IVec2, PlanIntent, Team, World};
use astraweave_net::{build_snapshot, replay_from, ReplayEvent};

#[tokio::main]
async fn main() -> Result<()> {
    // Build a tiny world and a short replay log, then verify deterministic hash.
    let mut w = World::new();
    let p = w.spawn("P", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let c = w.spawn("C", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 10);
    let _e = w.spawn("E", IVec2 { x: 7, y: 2 }, Team { id: 2 }, 60, 0);

    let baseline = build_snapshot(&w, 0, 0).world_hash;

    let evs = vec![
        ReplayEvent {
            tick: 5,
            seq: 0,
            actor_id: c,
            intent: PlanIntent { plan_id: "mv".into(), steps: vec![ActionStep::MoveTo { x: 4, y: 2 }] },
            world_hash: 0,
        },
        ReplayEvent {
            tick: 10,
            seq: 0,
            actor_id: p,
            intent: PlanIntent { plan_id: "mv".into(), steps: vec![ActionStep::MoveTo { x: 3, y: 2 }] },
            world_hash: 0,
        },
    ];
    let hash = replay_from(w, &evs)?;
    println!("baseline={}, final={}", baseline, hash);
    Ok(())
}
