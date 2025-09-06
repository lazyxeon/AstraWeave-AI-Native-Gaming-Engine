use astraweave_memory::{CompanionProfile, Episode};

fn main() -> anyhow::Result<()> {
    let mut p = CompanionProfile::new_default();
    p.episodes.push(Episode {
        title: "rescue_echo".into(),
        summary: "flanked north; clutch revive".into(),
        tags: vec!["flank".into(), "revive".into()],
        ts: "2025-09-04T12:00:00Z".into(),
    });
    p.distill();
    p.sign();
    p.save_to_file("companion.cprof")?;
    let loaded = CompanionProfile::load_from_file("companion.cprof")?;
    println!("Loaded profile OK? verify={}", loaded.verify());
    Ok(())
}
