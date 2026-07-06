//! `cargo xtask fetch-assets` — restore release-hosted asset packs (AD.2, ratified D2).
//!
//! Contract (AD.2 Phase 2):
//! 1. Parse `assets/packs.manifest.toml`; per entry, skip when a stamp exists whose
//!    recorded sha256 matches the manifest (idempotent no-op, announced in output).
//! 2. Download to a temp path; verify sha256 of the complete file BEFORE unpacking.
//!    Mismatch → temp deleted, loud failure naming expected/actual, nonzero exit.
//! 3. Unpack under `assets/packs/<unpack_to>`. Entry paths and zip member paths that
//!    escape the packs root (absolute, `..`, drive prefixes) fail the pack.
//! 4. Stamp written only after successful unpack (unpack itself is staged through a
//!    temp dir + rename, so a torn unpack never occupies the final path).
//! 5. No resume logic (v1 simplification): a failed large download re-downloads.
//! 6. Unreachable source → the error names the URL and the local-source sideload option.
//! 7. `--force` re-fetches regardless of stamps.

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    #[serde(default, rename = "pack")]
    pub packs: Vec<PackEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackEntry {
    /// Unique pack name; also the stamp-file key.
    pub name: String,
    /// `https://…` URL (GitHub release asset), `file://…` URL, or plain local path.
    /// Local sources let AD.3 verify the whole loop against locally-built zips
    /// before anything is uploaded, and allow director sideloads.
    pub source: String,
    /// Lowercase hex sha256 of the zip. Verified before any unpack.
    pub sha256: String,
    /// Unpack directory, relative, confined under `assets/packs/`.
    pub unpack_to: String,
    /// Expected zip size in bytes — sanity/progress only, sha256 is the gate.
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stamp {
    name: String,
    sha256: String,
    unpacked_at_epoch_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackStatus {
    /// Stamp present and sha256 matches the manifest — nothing done.
    UpToDate,
    /// Downloaded, verified, unpacked, stamped.
    Fetched,
    /// This entry failed; message carries the cause. Other entries still run.
    Failed(String),
}

#[derive(Debug, Default)]
pub struct FetchOptions {
    /// `--pack <name>`: restrict to one entry.
    pub only_pack: Option<String>,
    /// `--force`: ignore stamps.
    pub force: bool,
}

pub fn load_manifest(path: &Path) -> Result<Manifest> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read manifest {}", path.display()))?;
    let manifest: Manifest =
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
    if manifest.schema_version != 1 {
        bail!(
            "unsupported packs.manifest.toml schema_version {} (this tool understands 1)",
            manifest.schema_version
        );
    }
    Ok(manifest)
}

/// Run the fetch over `manifest_path`, unpacking under `packs_root`.
/// Returns one `(pack name, status)` per processed entry; the caller decides the
/// process exit code (any `Failed` → nonzero).
pub fn run_fetch(
    manifest_path: &Path,
    packs_root: &Path,
    opts: &FetchOptions,
) -> Result<Vec<(String, PackStatus)>> {
    let manifest = load_manifest(manifest_path)?;

    let selected: Vec<&PackEntry> = match &opts.only_pack {
        Some(name) => {
            let found: Vec<&PackEntry> =
                manifest.packs.iter().filter(|p| &p.name == name).collect();
            if found.is_empty() {
                bail!(
                    "no pack named '{}' in {} (available: {})",
                    name,
                    manifest_path.display(),
                    manifest
                        .packs
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            found
        }
        None => manifest.packs.iter().collect(),
    };

    if selected.is_empty() {
        println!("packs.manifest.toml has no pack entries — nothing to fetch.");
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    for entry in selected {
        let status = match fetch_one(entry, packs_root, opts.force) {
            Ok(status) => status,
            Err(e) => {
                eprintln!("✗ {}: {:#}", entry.name, e);
                PackStatus::Failed(format!("{e:#}"))
            }
        };
        results.push((entry.name.clone(), status));
    }

    let failed = results
        .iter()
        .filter(|(_, s)| matches!(s, PackStatus::Failed(_)))
        .count();
    println!(
        "fetch-assets summary: {} up-to-date, {} fetched, {} failed (of {})",
        results
            .iter()
            .filter(|(_, s)| *s == PackStatus::UpToDate)
            .count(),
        results
            .iter()
            .filter(|(_, s)| *s == PackStatus::Fetched)
            .count(),
        failed,
        results.len()
    );
    Ok(results)
}

fn fetch_one(entry: &PackEntry, packs_root: &Path, force: bool) -> Result<PackStatus> {
    validate_entry(entry)?;

    let stamp_path = packs_root
        .join(".stamps")
        .join(format!("{}.toml", entry.name));
    if !force {
        if let Some(stamp) = read_stamp(&stamp_path) {
            if stamp.sha256.eq_ignore_ascii_case(&entry.sha256) {
                println!(
                    "= {}: up to date (stamp sha256 matches manifest) — skipping",
                    entry.name
                );
                return Ok(PackStatus::UpToDate);
            }
            println!(
                "~ {}: stamp sha256 differs from manifest — re-fetching",
                entry.name
            );
        }
    }

    let tmp_dir = packs_root.join(".tmp");
    fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("failed to create temp dir {}", tmp_dir.display()))?;
    let tmp_zip = tmp_dir.join(format!("{}.zip.partial", entry.name));

    // Any early exit below must not leave the partial download behind.
    let result = fetch_verify_unpack(entry, packs_root, &tmp_zip, &stamp_path);
    let _ = fs::remove_file(&tmp_zip);
    result
}

fn fetch_verify_unpack(
    entry: &PackEntry,
    packs_root: &Path,
    tmp_zip: &Path,
    stamp_path: &Path,
) -> Result<PackStatus> {
    println!("> {}: acquiring {}", entry.name, entry.source);
    acquire(&entry.source, tmp_zip)?;

    let actual_size = fs::metadata(tmp_zip)?.len();
    if actual_size != entry.size_bytes {
        eprintln!(
            "  warning: {} downloaded size {} B differs from manifest size_bytes {} B (sha256 is the gate)",
            entry.name, actual_size, entry.size_bytes
        );
    }

    let actual = sha256_of_file(tmp_zip)?;
    if !actual.eq_ignore_ascii_case(&entry.sha256) {
        bail!(
            "sha256 mismatch for pack '{}': expected {}, actual {} — refusing to unpack; partial download deleted",
            entry.name,
            entry.sha256,
            actual
        );
    }
    println!("  sha256 verified ({actual})");

    // Stage the unpack in a temp dir, then swap it into place.
    let dest = packs_root.join(&entry.unpack_to);
    let staged = packs_root
        .join(".tmp")
        .join(format!("unpack-{}", entry.name));
    if staged.exists() {
        fs::remove_dir_all(&staged)?;
    }
    unpack_zip(tmp_zip, &staged)
        .with_context(|| format!("unpack failed for pack '{}'", entry.name))
        .inspect_err(|_| {
            let _ = fs::remove_dir_all(&staged);
        })?;
    if dest.exists() {
        fs::remove_dir_all(&dest)
            .with_context(|| format!("failed to clear previous unpack at {}", dest.display()))?;
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&staged, &dest).with_context(|| {
        format!(
            "failed to move staged unpack into place ({} -> {})",
            staged.display(),
            dest.display()
        )
    })?;

    write_stamp(stamp_path, entry)?;
    println!("+ {}: unpacked to {}", entry.name, dest.display());
    Ok(PackStatus::Fetched)
}

fn validate_entry(entry: &PackEntry) -> Result<()> {
    if entry.name.is_empty() || !is_safe_relative(&entry.name) || entry.name.contains(['/', '\\']) {
        bail!("invalid pack name '{}'", entry.name);
    }
    if !is_safe_relative(&entry.unpack_to) {
        bail!(
            "pack '{}': unpack_to '{}' escapes assets/packs/ (must be relative, no '..', no absolute/drive paths)",
            entry.name,
            entry.unpack_to
        );
    }
    if entry.sha256.len() != 64 || !entry.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!(
            "pack '{}': sha256 must be 64 hex chars, got '{}'",
            entry.name,
            entry.sha256
        );
    }
    Ok(())
}

/// A path is safe when it is relative and contains only normal (or `.`) components —
/// no `..`, no root, no Windows drive/UNC prefixes. Applied to `unpack_to` AND to
/// every zip member path.
pub fn is_safe_relative(p: &str) -> bool {
    if p.is_empty() {
        return false;
    }
    let path = Path::new(p);
    if path.is_absolute() {
        return false;
    }
    path.components().all(|c| match c {
        Component::Normal(seg) => !seg.to_string_lossy().contains(':'),
        Component::CurDir => true,
        Component::ParentDir | Component::RootDir | Component::Prefix(_) => false,
    })
}

fn acquire(source: &str, dest: &Path) -> Result<()> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::blocking::Client::builder()
            .user_agent("astraweave-xtask-fetch-assets/0.1")
            .timeout(std::time::Duration::from_secs(600))
            .build()?;
        let mut resp = client
            .get(source)
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| {
                anyhow!(
                    "download failed from {source}: {e}. If this machine is offline or the release \
                     is unreachable, download the file manually from that URL and sideload it by \
                     setting this entry's `source` to the local file path."
                )
            })?;
        let mut out = File::create(dest)
            .with_context(|| format!("failed to create temp file {}", dest.display()))?;
        std::io::copy(&mut resp, &mut out)
            .with_context(|| format!("download interrupted from {source}"))?;
        out.flush()?;
        Ok(())
    } else {
        let local = local_source_path(source);
        fs::copy(&local, dest).map(|_| ()).map_err(|e| {
            anyhow!(
                "failed to read local source {} ({e}). Local sources may be plain paths or file:// URLs.",
                local.display()
            )
        })
    }
}

/// `file:///D:/x/y.zip` → `D:/x/y.zip`; `file:///home/x.zip` → `/home/x.zip`;
/// anything without the scheme is returned as-is.
fn local_source_path(source: &str) -> PathBuf {
    match source.strip_prefix("file://") {
        None => PathBuf::from(source),
        Some(rest) => {
            let bytes = rest.as_bytes();
            // Windows drive form: "/D:/…" — drop the leading slash.
            if bytes.len() >= 3 && bytes[0] == b'/' && bytes[2] == b':' {
                PathBuf::from(&rest[1..])
            } else {
                PathBuf::from(rest)
            }
        }
    }
}

fn sha256_of_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn unpack_zip(zip_path: &Path, dest: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(zip_path)?)
        .with_context(|| format!("not a readable zip: {}", zip_path.display()))?;
    fs::create_dir_all(dest)?;
    for i in 0..archive.len() {
        let mut member = archive.by_index(i)?;
        let raw_name = member.name().to_string();
        if !is_safe_relative(&raw_name) {
            bail!("path traversal rejected: zip member '{raw_name}'");
        }
        let out_path = dest.join(&raw_name);
        if member.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = File::create(&out_path)
                .with_context(|| format!("failed to create {}", out_path.display()))?;
            std::io::copy(&mut member, &mut out)?;
        }
    }
    Ok(())
}

fn read_stamp(path: &Path) -> Option<Stamp> {
    let text = fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

fn write_stamp(path: &Path, entry: &PackEntry) -> Result<()> {
    let stamp = Stamp {
        name: entry.name.clone(),
        sha256: entry.sha256.to_ascii_lowercase(),
        unpacked_at_epoch_secs: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, toml::to_string_pretty(&stamp)?)
        .with_context(|| format!("failed to write stamp {}", path.display()))?;
    Ok(())
}
