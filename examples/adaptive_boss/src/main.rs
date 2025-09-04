use astraweave_core::*;
use astraweave_director::BossDirector;

fn main() -> anyhow::Result<()> {
    let mut w = World::new();
    // simple arena
    let player = w.spawn("Player", IVec2{x:2,y:2}, Team{ id:0 }, 100, 0);
    let comp   = w.spawn("Comp",   IVec2{x:3,y:2}, Team{ id:1 }, 80,  30);
    let boss   = w.spawn("Boss",   IVec2{x:14,y:2}, Team{ id:2 }, 400, 0);
    // snapshot for director (pretend boss sees one enemy)
    let snap = WorldSnapshot {
        t: w.t,
        player: PlayerState{ hp:100, pos:w.pos_of(player).unwrap(), stance:"stand".into(), orders:vec![] },
        me: CompanionState{ ammo:30, cooldowns:Default::default(), morale:0.8, pos:w.pos_of(comp).unwrap() },
        enemies: vec![ EnemyState{ id: boss, pos: w.pos_of(boss).unwrap(), hp:400, cover:"high".into(), last_seen:w.t } ],
        pois: vec![],
        objective: Some("defeat_boss".into()),
    };

    let director = BossDirector;
    let mut budget = DirectorBudget{ traps: 2, terrain_edits: 2, spawns: 2 };
    let plan = director.plan(&snap, &budget);

    let mut log = |s:String| println!("{}", s);
    println!("Director plan: {}", serde_json::to_string_pretty(&plan)?);
    apply_director_plan(&mut w, &mut budget, &plan, &mut log);

    println!("Remaining budget: traps={}, terrain_edits={}, spawns={}",
        budget.traps, budget.terrain_edits, budget.spawns);
    Ok(())
}
