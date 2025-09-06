use astraweave_persona::load_persona_zip;
fn main() -> anyhow::Result<()> {
    let p = load_persona_zip("sniper_persona.zip")?;
    println!(
        "Loaded persona: tone={}, verify={}",
        p.persona.tone,
        p.verify()
    );
    Ok(())
}
