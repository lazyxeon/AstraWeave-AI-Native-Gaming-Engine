use anyhow::{bail, Result};
use std::path::Path;
use xtask::fetch_assets::{
    ci_guard, gen_ignore, gen_keeplist, run_fetch, verify_assets_cmd, FetchOptions, PackStatus,
    Selection,
};

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  fetch-assets [--profile <name> | --all | --pack <name>] [--force|--repair] [--adopt]
      Transactionally restore release-hosted asset packs (assets/packs.manifest.toml)
      to their ORIGINAL repo paths. Default selection: the `starter` profile.
      Idempotent via stamps; sha256-verified; journaled crash recovery.
      --profile <name>  fetch the named manifest profile (default: starter)
      --all             fetch every pack
      --pack <name>     fetch only the named pack
      --force|--repair  reinstall the current version to pristine
      --adopt           take ownership of unknown preexisting files at destinations

  verify-assets
      Step-12 existence pass: installed-pack ledgers + consumer manifests
      (biome material tomls) must all resolve on disk.

  gen-ignore [--check]
      (Re)generate the ignore surfaces from the manifest roots: assets/.gitignore
      (fully generated) + the AW-PACK-IGNORE marked block in the root .gitignore
      (only for roots outside assets/, e.g. materials-src at assets_src/).
      --check: fail on drift from the generated form (CI mode).

  gen-keeplist
      Regenerate assets/packlists/_tracked_keep.txt — tracked cohabitants under
      managed roots that are NOT pack members. ci-guard uses subset semantics.

  ci-guard
      The anti-re-bloat rail (file-exact): fail if any packlist member is
      git-tracked, if a stray tracked file appears under a managed root (neither
      member nor keeplist), or if an ignore surface drifts from its generated form.
";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("fetch-assets") => fetch_assets_cmd(&args[1..]),
        Some("verify-assets") => verify_assets_entry(),
        Some("gen-ignore") => gen_ignore_cmd(&args[1..]),
        Some("gen-keeplist") => {
            let root = workspace_root();
            gen_keeplist(&root.join("assets").join("packs.manifest.toml"), root)
        }
        Some("ci-guard") => ci_guard_entry(),
        Some(other) => bail!("unknown task '{other}'\n{USAGE}"),
        None => {
            print!("{USAGE}");
            Ok(())
        }
    }
}

fn workspace_root() -> &'static Path {
    // tools/xtask -> tools -> workspace root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask lives two levels below the workspace root")
}

fn fetch_assets_cmd(args: &[String]) -> Result<()> {
    let mut opts = FetchOptions::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--force" | "--repair" => opts.force = true,
            "--adopt" => opts.adopt = true,
            "--all" => opts.selection = Selection::All,
            "--profile" => match it.next() {
                Some(name) => opts.selection = Selection::Profile(name.clone()),
                None => bail!("--profile requires a profile name\n{USAGE}"),
            },
            "--pack" => match it.next() {
                Some(name) => opts.selection = Selection::Pack(name.clone()),
                None => bail!("--pack requires a pack name\n{USAGE}"),
            },
            other => bail!("unknown argument '{other}' for fetch-assets\n{USAGE}"),
        }
    }

    let root = workspace_root();
    let manifest = root.join("assets").join("packs.manifest.toml");

    let results = run_fetch(&manifest, root, &opts)?;
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

fn verify_assets_entry() -> Result<()> {
    let root = workspace_root();
    verify_assets_cmd(&root.join("assets").join("packs.manifest.toml"), root)
}

fn gen_ignore_cmd(args: &[String]) -> Result<()> {
    let check = match args {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => bail!("gen-ignore takes at most `--check`\n{USAGE}"),
    };
    let root = workspace_root();
    gen_ignore(
        &root.join("assets").join("packs.manifest.toml"),
        root,
        check,
    )
}

fn ci_guard_entry() -> Result<()> {
    let root = workspace_root();
    ci_guard(&root.join("assets").join("packs.manifest.toml"), root)
}
