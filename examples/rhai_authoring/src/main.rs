use astraweave_author::{run_author_script, MapMeta};

fn main() -> anyhow::Result<()> {
    let meta = MapMeta {
        width: 40,
        height: 20,
        enemy_count: 18,
        difficulty: 3,
    };
    let (budget, hints) = run_author_script("author_example.rhai", &meta)?;
    println!(
        "Budget: traps={}, terrain={}, spawns={}",
        budget.traps, budget.terrain_edits, budget.spawns
    );
    println!("Hints: {}", serde_json::to_string_pretty(&hints)?);
    Ok(())
}
