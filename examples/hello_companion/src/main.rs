use astraweave_ai::{Orchestrator, RuleOrchestrator};
use astraweave_core::{
    build_snapshot, step, validate_and_execute, IVec2, PerceptionConfig, SimConfig, Team,
    ValidateCfg, World,
};

fn main() -> anyhow::Result<()> {
    // Build a tiny grid arena 20x10 with some obstacles
    let mut w = World::new();
    for x in 6..=6 {
        for y in 1..=8 {
            w.obstacles.insert((x, y));
        }
    } // a vertical wall

    // Spawn entities
    let player = w.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let comp = w.spawn("Companion", IVec2 { x: 2, y: 3 }, Team { id: 1 }, 80, 30);
    let enemy = w.spawn("Rival", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);

    // Prime companion cooldowns
    if let Some(cd) = w.cooldowns_mut(comp) {
        cd.map.insert("throw:smoke".into(), 0.0);
    }

    let orch = RuleOrchestrator;
    let p_cfg = PerceptionConfig { los_max: 12 };
    let v_cfg = ValidateCfg {
        world_bounds: (0, 0, 19, 9),
    };
    let s_cfg = SimConfig { dt: 0.25 };

    // Build snapshot & propose plan
    let enemies = vec![enemy];
    let snap = build_snapshot(&w, player, comp, &enemies, Some("extract".into()), &p_cfg);
    let plan = orch.propose_plan(&snap);

    let mut log = |line: String| {
        println!("{}", line);
    };

    println!("--- TICK 0, world time {:.2}", w.t);
    validate_and_execute(&mut w, comp, &plan, &v_cfg, &mut log).unwrap();

    // Progress a few seconds to simulate cooldowns & time
    for _ in 0..20 {
        step(&mut w, &s_cfg);
    }

    println!("--- Post-plan world state @ t={:.2}", w.t);
    println!(
        "Companion @ {:?}, Enemy @ {:?}, Enemy HP = {:?}",
        w.pos_of(comp).unwrap(),
        w.pos_of(enemy).unwrap(),
        w.health(enemy).unwrap().hp
    );

    Ok(())
}
