use anyhow::{Context, Result};
use astraweave_asset::{AssetDatabase, AssetKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;
use which::which;
use flate2::write::GzEncoder;
use flate2::Compression;
use ring::signature::Ed25519KeyPair;
use ring::rand::SystemRandom;
use base64;
use base64::Engine;

#[derive(Debug, Serialize, Deserialize)]
struct PipelineCfg {
    source: String, // e.g. "assets_src"
    output: String, // e.g. "assets"
    rules: Vec<Rule>,
    compress: bool, // new: compress outputs
    validate: bool, // new: validate after cooking
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Rule {
    #[serde(rename = "texture")]
    Texture { glob: String, normal_map: bool },
    #[serde(rename = "model")]
    Model { glob: String },
    #[serde(rename = "audio")]
    Audio { glob: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ManifestEntry {
    src: String,
    out: String,
    sha256: String,
    kind: String,
    guid: String, // new
    dependencies: Vec<String>, // new
}

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SignedManifest {
    entries: Vec<ManifestEntry>,
    signature: String, // base64 encoded Ed25519 signature
}

fn main() -> Result<()> {
    let cfg_path = std::env::args().nth(1).unwrap_or("aw_pipeline.toml".into());
    let cfg_text = fs::read_to_string(&cfg_path).with_context(|| format!("read {}", cfg_path))?;
    let cfg: PipelineCfg = toml::from_str(&cfg_text)?;

    fs::create_dir_all(&cfg.output)?;

    // Initialize asset database
    let mut db = AssetDatabase::new();
    db.scan_directory(Path::new(&cfg.source))?;

    let mut manifest: Vec<ManifestEntry> = Vec::new();

    for rule in &cfg.rules {
        match rule {
            Rule::Texture {
                glob,
                normal_map: _,
            } => {
                for entry in globwalk(&cfg.source, glob)? {
                    let out = process_texture(&entry, &cfg.output, cfg.compress)?;
                    let guid = register_asset(&mut db, &entry, AssetKind::Texture)?;
                    manifest.push(record("texture", &entry, &out, &guid, &db)?);
                }
            }
            Rule::Model { glob } => {
                for entry in globwalk(&cfg.source, glob)? {
                    let out = process_model(&entry, &cfg.output, cfg.compress)?;
                    let guid = register_asset(&mut db, &entry, AssetKind::Mesh)?;
                    manifest.push(record("model", &entry, &out, &guid, &db)?);
                }
            }
            Rule::Audio { glob } => {
                for entry in globwalk(&cfg.source, glob)? {
                    let out = process_audio(&entry, &cfg.output, cfg.compress)?;
                    let guid = register_asset(&mut db, &entry, AssetKind::Audio)?;
                    manifest.push(record("audio", &entry, &out, &guid, &db)?);
                }
            }
        }
    }

    // Save asset database manifest
    let db_manifest_path = Path::new(&cfg.output).join("assets.json");
    db.save_manifest(&db_manifest_path)?;

    // Sign the manifest
    let signed_manifest = sign_manifest(&manifest)?;
    let manifest_path = Path::new(&cfg.output).join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&signed_manifest)?)?;
    println!("Wrote {}", manifest_path.display());

    if cfg.validate {
        validate_assets(&signed_manifest.entries, &db)?;
        println!("Validation passed");
    }

    Ok(())
}

fn globwalk(root: &str, pat: &str) -> Result<Vec<PathBuf>> {
    let mut v = vec![];

    // Handle brace expansion like *.{png,jpg,jpeg}
    let patterns = if pat.contains('{') && pat.contains('}') {
        let start = pat.find('{').unwrap();
        let end = pat.find('}').unwrap();
        let prefix = &pat[..start];
        let suffix = &pat[end + 1..];
        let extensions = &pat[start + 1..end];

        extensions
            .split(',')
            .map(|ext| format!("{}{}{}", prefix, ext, suffix))
            .collect::<Vec<_>>()
    } else {
        vec![pat.to_string()]
    };

    for pattern_str in patterns {
        for e in WalkDir::new(root) {
            let e = e?;
            if e.file_type().is_file() {
                let p = e.into_path();
                // Create pattern relative to root for matching
                let relative_path = p.strip_prefix(root).unwrap_or(&p);
                if glob::Pattern::new(&pattern_str)?.matches_path(relative_path) {
                    if !v.contains(&p) {
                        v.push(p);
                    }
                }
            }
        }
    }
    Ok(v)
}

fn register_asset(db: &mut AssetDatabase, src: &Path, kind: AssetKind) -> Result<String> {
    // Infer dependencies based on kind
    let dependencies = match kind {
        AssetKind::Mesh => {
            // For glTF, parse dependencies
            if src.extension().and_then(|e| e.to_str()) == Some("gltf") {
                // For now, empty; in full impl, parse glTF for textures
                vec![]
            } else {
                vec![]
            }
        }
        _ => vec![],
    };
    db.register_asset(src, kind, dependencies)
}

fn record(kind: &str, src: &Path, out: &Path, guid: &str, db: &AssetDatabase) -> Result<ManifestEntry> {
    let mut f = fs::File::open(out)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut f, &mut hasher)?;
    let sha = hex::encode(hasher.finalize());
    let dependencies = db.get_dependencies(guid).cloned().unwrap_or_default().into_iter().collect();
    Ok(ManifestEntry {
        src: src.to_string_lossy().to_string(),
        out: out.to_string_lossy().to_string(),
        sha256: sha,
        kind: kind.into(),
        guid: guid.to_string(),
        dependencies,
    })
}

fn process_texture(src: &Path, out_root: &str, compress: bool) -> Result<PathBuf> {
    fs::create_dir_all(out_root)?;
    let stem = src.file_stem().unwrap().to_string_lossy();
    let out = Path::new(out_root).join(format!("{stem}.ktx2"));
    let processed = if compress { out.with_extension("ktx2.gz") } else { out.clone() };
    // Prefer toktx; fallback basisu; fallback copy
    if let Ok(toktx) = which("toktx") {
        // BasisU UASTC KTX2 with Zstd
        let status = Command::new(toktx)
            .args([
                "--genmipmap",
                "--uastc",
                "--zcmp",
                "18",
                out.to_str().unwrap(),
                src.to_str().unwrap(),
            ])
            .status()?;
        if status.success() {
            if compress {
                compress_file(&out, &processed)?;
                fs::remove_file(&out)?;
            }
            return Ok(processed);
        }
    }
    if let Ok(basisu) = which("basisu") {
        let _tmp = out.with_extension("basis");
        let status = Command::new(basisu)
            .args(["-uastc", "-comp_level", "2", "-file", src.to_str().unwrap()])
            .status()?;
        if status.success() {
            // leave .basis or convert later; for now write .basis → .ktx2 not implemented
            fs::copy(&src, &out)?; // placeholder
            if compress {
                compress_file(&out, &processed)?;
                fs::remove_file(&out)?;
            }
            return Ok(processed);
        }
    }
    fs::copy(src, &out)?; // fallback
    if compress {
        compress_file(&out, &processed)?;
        fs::remove_file(&out)?;
    }
    Ok(processed)
}

fn process_model(src: &Path, out_root: &str, compress: bool) -> Result<PathBuf> {
    fs::create_dir_all(out_root)?;
    let stem = src.file_stem().unwrap().to_string_lossy();
    let out = Path::new(out_root).join(format!("{stem}.meshbin"));
    let processed = if compress { out.with_extension("meshbin.gz") } else { out.clone() };
    if src.extension().map(|e| e.to_string_lossy().to_lowercase()) == Some("gltf".into())
        || src.extension().map(|e| e.to_string_lossy().to_lowercase()) == Some("glb".into())
    {
        let (_doc, _buffers, _images) = gltf::import(src)?;
        // Minimal example: just copy the glTF for now; swap to a meshbin writer later
        fs::copy(src, &out)?;
        if compress {
            compress_file(&out, &processed)?;
            fs::remove_file(&out)?;
        }
        Ok(processed)
    } else {
        fs::copy(src, &out)?;
        if compress {
            compress_file(&out, &processed)?;
            fs::remove_file(&out)?;
        }
        Ok(processed)
    }
}

fn process_audio(src: &Path, out_root: &str, compress: bool) -> Result<PathBuf> {
    fs::create_dir_all(out_root)?;
    let stem = src.file_stem().unwrap().to_string_lossy();
    let out = Path::new(out_root).join(format!("{stem}.ogg"));
    let processed = if compress { out.with_extension("ogg.gz") } else { out.clone() };
    if let Ok(oggenc) = which("oggenc") {
        let status = Command::new(oggenc)
            .args([
                "-q",
                "4",
                src.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ])
            .status()?;
        if status.success() {
            if compress {
                compress_file(&out, &processed)?;
                fs::remove_file(&out)?;
            }
            return Ok(processed);
        }
    }
    // fallback: keep wav as-is
    let out_wav = Path::new(out_root).join(format!("{stem}.wav"));
    let processed = if compress { out_wav.with_extension("wav.gz") } else { out_wav.clone() };
    fs::copy(src, &out_wav)?;
    if compress {
        compress_file(&out_wav, &processed)?;
        fs::remove_file(&out_wav)?;
    }
    Ok(processed)
}

fn compress_file(input: &Path, output: &Path) -> Result<()> {
    let input_data = fs::read(input)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    std::io::Write::write_all(&mut encoder, &input_data)?;
    let compressed = encoder.finish()?;
    fs::write(output, compressed)?;
    Ok(())
}

fn validate_assets(manifest: &[ManifestEntry], db: &AssetDatabase) -> Result<()> {
    for entry in manifest {
        // Check if file exists and hash matches
        if !Path::new(&entry.out).exists() {
            anyhow::bail!("Output file {} does not exist", entry.out);
        }
        let mut f = fs::File::open(&entry.out)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut f, &mut hasher)?;
        let sha = hex::encode(hasher.finalize());
        if sha != entry.sha256 {
            anyhow::bail!("Hash mismatch for {}", entry.out);
        }
        // Check dependencies
        if let Some(meta) = db.get_asset(&entry.guid) {
            for dep in &meta.dependencies {
                if db.get_asset(dep).is_none() {
                    anyhow::bail!("Missing dependency {} for {}", dep, entry.guid);
                }
            }
        }
    }
    Ok(())
}

fn sign_manifest(manifest: &[ManifestEntry]) -> Result<SignedManifest> {
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).map_err(|_| anyhow::anyhow!("Failed to generate key"))?;
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).map_err(|_| anyhow::anyhow!("Invalid key"))?;
    let manifest_json = serde_json::to_string(manifest)?;
    let signature = key_pair.sign(manifest_json.as_bytes());
    Ok(SignedManifest {
        entries: manifest.to_vec(),
        signature: base64::engine::general_purpose::STANDARD.encode(signature.as_ref()),
    })
}
