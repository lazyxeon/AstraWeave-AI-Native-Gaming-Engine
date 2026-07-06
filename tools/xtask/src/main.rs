use anyhow::{bail, Result};
use std::path::Path;
use xtask::fetch_assets::{run_fetch, FetchOptions, PackStatus};

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  fetch-assets [--pack <name>] [--force]
      Restore release-hosted asset packs listed in assets/packs.manifest.toml
      into assets/packs/. Idempotent via stamps; sha256-verified before unpack.
      --pack <name>   fetch only the named pack
      --force         re-fetch even when the stamp matches
";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("fetch-assets") => fetch_assets_cmd(&args[1..]),
        Some(other) => bail!("unknown task '{other}'\n{USAGE}"),
        None => {
            print!("{USAGE}");
            Ok(())
        }
    }
}

fn fetch_assets_cmd(args: &[String]) -> Result<()> {
    let mut opts = FetchOptions::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--force" => opts.force = true,
            "--pack" => match it.next() {
                Some(name) => opts.only_pack = Some(name.clone()),
                None => bail!("--pack requires a pack name\n{USAGE}"),
            },
            other => bail!("unknown argument '{other}' for fetch-assets\n{USAGE}"),
        }
    }

    // tools/xtask -> tools -> workspace root
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask lives two levels below the workspace root");
    let manifest = root.join("assets").join("packs.manifest.toml");
    let packs_root = root.join("assets").join("packs");

    let results = run_fetch(&manifest, &packs_root, &opts)?;
    let failures: Vec<&(String, PackStatus)> = results
        .iter()
        .filter(|(_, s)| matches!(s, PackStatus::Failed(_)))
        .collect();
    if !failures.is_empty() {
        bail!(
            "{} pack(s) failed: {}",
            failures.len(),
            failures
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}
