use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::{Path, PathBuf}, process::Command};
use walkdir::WalkDir;
use which::which;

#[derive(Debug, Serialize, Deserialize)]
struct PipelineCfg {
    source: String,         // e.g. "assets_src"
    output: String,         // e.g. "assets"
    rules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="kind")]
enum Rule {
    #[serde(rename="texture")] Texture { glob: String, normal_map: bool },
    #[serde(rename="model")]   Model   { glob: String },
    #[serde(rename="audio")]   Audio   { glob: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestEntry {
    src: String,
    out: String,
    sha256: String,
    kind: String,
}

fn main() -> Result<()> {
    let cfg_path = std::env::args().nth(1).unwrap_or("aw_pipeline.toml".into());
    let cfg_text = fs::read_to_string(&cfg_path)
        .with_context(|| format!("read {}", cfg_path))?;
    let cfg: PipelineCfg = toml::from_str(&cfg_text)?;

    fs::create_dir_all(&cfg.output)?;
    let mut manifest: Vec<ManifestEntry> = Vec::new();

    for rule in &cfg.rules {
        match rule {
            Rule::Texture { glob, normal_map:_ } => {
                for entry in globwalk(&cfg.source, glob)? {
                    let out = process_texture(&entry, &cfg.output)?;
                    manifest.push(record("texture", &entry, &out)?);
                }
            }
            Rule::Model { glob } => {
                for entry in globwalk(&cfg.source, glob)? {
                    let out = process_model(&entry, &cfg.output)?;
                    manifest.push(record("model", &entry, &out)?);
                }
            }
            Rule::Audio { glob } => {
                for entry in globwalk(&cfg.source, glob)? {
                    let out = process_audio(&entry, &cfg.output)?;
                    manifest.push(record("audio", &entry, &out)?);
                }
            }
        }
    }

    let manifest_path = Path::new(&cfg.output).join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    println!("Wrote {}", manifest_path.display());
    Ok(())
}

fn globwalk(root:&str, pat:&str) -> Result<Vec<PathBuf>> {
    let mut v = vec![];
    for e in WalkDir::new(root) {
        let e = e?;
        if e.file_type().is_file() {
            let p = e.into_path();
            if glob::Pattern::new(pat)?.matches_path(&p) {
                v.push(p);
            }
        }
    }
    Ok(v)
}

fn record(kind:&str, src:&Path, out:&Path) -> Result<ManifestEntry> {
    let mut f = fs::File::open(out)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut f, &mut hasher)?;
    let sha = hex::encode(hasher.finalize());
    Ok(ManifestEntry{
        src: src.to_string_lossy().to_string(),
        out: out.to_string_lossy().to_string(),
        sha256: sha,
        kind: kind.into(),
    })
}

fn process_texture(src:&Path, out_root:&str) -> Result<PathBuf> {
    fs::create_dir_all(out_root)?;
    let stem = src.file_stem().unwrap().to_string_lossy();
    let out = Path::new(out_root).join(format!("{stem}.ktx2"));
    // Prefer toktx; fallback basisu; fallback copy
    if let Ok(toktx) = which("toktx") {
        // BasisU UASTC KTX2 with Zstd
        let status = Command::new(toktx).args([
            "--genmipmap", "--uastc", "--zcmp", "18",
            out.to_str().unwrap(),
            src.to_str().unwrap(),
        ]).status()?;
        if status.success() { return Ok(out) }
    }
    if let Ok(basisu) = which("basisu") {
        let tmp = out.with_extension("basis");
        let status = Command::new(basisu).args([
            "-uastc", "-comp_level", "2", "-file",
            src.to_str().unwrap(),
        ]).status()?;
        if status.success() {
            // leave .basis or convert later; for now write .basis → .ktx2 not implemented
            fs::copy(&src, &out)?; // placeholder
            return Ok(out)
        }
    }
    fs::copy(src, &out)?; // fallback
    Ok(out)
}

fn process_model(src:&Path, out_root:&str) -> Result<PathBuf> {
    fs::create_dir_all(out_root)?;
    let stem = src.file_stem().unwrap().to_string_lossy();
    let out = Path::new(out_root).join(format!("{stem}.meshbin"));
    if src.extension().map(|e| e.to_string_lossy().to_lowercase()) == Some("gltf".into()) ||
       src.extension().map(|e| e.to_string_lossy().to_lowercase()) == Some("glb".into()) {
        let (doc, _buffers, _images) = gltf::import(src)?;
        // Minimal example: just copy the glTF for now; swap to a meshbin writer later
        fs::copy(src, &out)?;
        Ok(out)
    } else {
        fs::copy(src, &out)?;
        Ok(out)
    }
}

fn process_audio(src:&Path, out_root:&str) -> Result<PathBuf> {
    fs::create_dir_all(out_root)?;
    let stem = src.file_stem().unwrap().to_string_lossy();
    let out = Path::new(out_root).join(format!("{stem}.ogg"));
    if let Ok(oggenc) = which("oggenc") {
        let status = Command::new(oggenc).args([
            "-q","4",
            src.to_str().unwrap(),
            "-o", out.to_str().unwrap(),
        ]).status()?;
        if status.success() { return Ok(out) }
    }
    // fallback: keep wav as-is
    let out_wav = Path::new(out_root).join(format!("{stem}.wav"));
    fs::copy(src, &out_wav)?;
    Ok(out_wav)
}