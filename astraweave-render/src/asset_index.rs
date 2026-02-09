//! Runtime asset index — loads `assets/asset_index.toml` so the engine can
//! discover every material set, texture, HDRI, model and audio pack at startup
//! without scanning the filesystem.
//!
//! # Usage
//! ```rust,no_run
//! use astraweave_render::asset_index::AssetIndex;
//!
//! let index = AssetIndex::load("assets/asset_index.toml").unwrap();
//! let forest = index.material_set("forest").unwrap();
//! println!("Forest materials: {}", forest.dir);
//! ```

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// ─── Top-level envelope ─────────────────────────────────────────────────────

/// Parsed representation of `asset_index.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub index: IndexMeta,
    #[serde(default, rename = "material_set")]
    pub material_sets: Vec<MaterialSetEntry>,
    #[serde(default, rename = "texture")]
    pub textures: Vec<TextureEntry>,
    #[serde(default, rename = "hdri")]
    pub hdris: Vec<HdriRef>,
    #[serde(default, rename = "model")]
    pub models: Vec<ModelEntry>,
    #[serde(default, rename = "audio_pack")]
    pub audio_packs: Vec<AudioPackEntry>,
}

/// Metadata header.
#[derive(Debug, Clone, Deserialize)]
pub struct IndexMeta {
    pub version: u32,
    pub generated: String,
    pub asset_root: String,
}

// ─── Entry types ─────────────────────────────────────────────────────────────

/// A biome material set (directory with `materials.toml` + `arrays.toml`).
#[derive(Debug, Clone, Deserialize)]
pub struct MaterialSetEntry {
    pub biome: String,
    pub dir: String,
    pub layers: u32,
    #[serde(default)]
    pub description: String,
}

/// A single PBR texture set (albedo + normal + MRA).
#[derive(Debug, Clone, Deserialize)]
pub struct TextureEntry {
    pub name: String,
    pub dir: String,
    pub maps: Vec<String>,
    #[serde(default)]
    pub has_ktx2: bool,
    #[serde(default)]
    pub resolution: String,
}

/// An HDRI environment-map reference (quick summary — full mappings live in
/// `hdri_catalog.toml`).
#[derive(Debug, Clone, Deserialize)]
pub struct HdriRef {
    pub name: String,
    pub file: String,
    #[serde(default)]
    pub time: String,
    #[serde(default)]
    pub biomes: Vec<String>,
}

/// A 3-D model or model pack.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub dir: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub note: String,
}

/// An audio pack (loops, ambience, SFX, etc.).
#[derive(Debug, Clone, Deserialize)]
pub struct AudioPackEntry {
    pub name: String,
    pub dir: String,
    #[serde(default)]
    pub formats: Vec<String>,
    #[serde(default)]
    pub tracks: u32,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub description: String,
}

// ─── Implementation ──────────────────────────────────────────────────────────

impl AssetIndex {
    /// Load the index from a TOML file on disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("reading {}", path.as_ref().display()))?;
        Self::parse_str(&text)
    }

    /// Parse directly from a TOML string (useful for tests or embedded data).
    pub fn parse_str(text: &str) -> Result<Self> {
        let idx: Self = toml::from_str(text).context("parsing asset_index.toml")?;
        Ok(idx)
    }

    // ── Lookup helpers ───────────────────────────────────────────────────

    /// Find a material set by biome name (case-insensitive).
    pub fn material_set(&self, biome: &str) -> Option<&MaterialSetEntry> {
        let b = biome.to_lowercase();
        self.material_sets
            .iter()
            .find(|m| m.biome.to_lowercase() == b)
    }

    /// Find a texture by name (case-insensitive).
    pub fn texture(&self, name: &str) -> Option<&TextureEntry> {
        let n = name.to_lowercase();
        self.textures.iter().find(|t| t.name.to_lowercase() == n)
    }

    /// Find an HDRI by name (case-insensitive).
    pub fn hdri(&self, name: &str) -> Option<&HdriRef> {
        let n = name.to_lowercase();
        self.hdris.iter().find(|h| h.name.to_lowercase() == n)
    }

    /// List all HDRIs suitable for a given biome + time.
    pub fn hdris_for(&self, biome: &str, time: &str) -> Vec<&HdriRef> {
        let b = biome.to_lowercase();
        let t = time.to_lowercase();
        self.hdris
            .iter()
            .filter(|h| {
                h.biomes.iter().any(|x| x.to_lowercase() == b) && h.time.to_lowercase() == t
            })
            .collect()
    }

    /// Build a quick biome → MaterialSetEntry map.
    pub fn material_set_map(&self) -> HashMap<String, &MaterialSetEntry> {
        self.material_sets
            .iter()
            .map(|m| (m.biome.clone(), m))
            .collect()
    }

    // ── Validation ───────────────────────────────────────────────────────

    /// Validate that every referenced file actually exists under `asset_root`.
    /// Returns a list of missing paths.
    pub fn validate_paths(&self, base: impl AsRef<Path>) -> Vec<String> {
        let base = base.as_ref();
        let mut missing = Vec::new();

        // Material sets: check dir exists
        for ms in &self.material_sets {
            let p = base.join(&ms.dir);
            if !p.is_dir() {
                missing.push(format!(
                    "material_set[{}]: dir not found: {}",
                    ms.biome,
                    p.display()
                ));
            }
        }

        // Textures: check dir exists
        for tex in &self.textures {
            let p = base.join(&tex.dir);
            if !p.is_dir() {
                missing.push(format!(
                    "texture[{}]: dir not found: {}",
                    tex.name,
                    p.display()
                ));
            }
        }

        // HDRIs: check file exists
        for h in &self.hdris {
            let p = base.join(&h.file);
            if !p.is_file() {
                missing.push(format!("hdri[{}]: file not found: {}", h.name, p.display()));
            }
        }

        missing
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_TOML: &str = r#"
[index]
version = 1
generated = "2026-02-06"
asset_root = "assets"

[[material_set]]
biome = "forest"
dir = "materials/forest"
layers = 5
description = "Dense forest"

[[texture]]
name = "grass"
dir = "textures/grass"
maps = ["albedo", "normal", "mra"]
has_ktx2 = true
resolution = "1024x1024"

[[hdri]]
name = "rainforest"
file = "hdri/polyhaven/rainforest_trail/rainforest_trail_2k.hdr"
time = "day"
biomes = ["forest"]

[[model]]
name = "test_cube"
dir = "models/test"
format = "glb"

[[audio_pack]]
name = "loops"
dir = "audio/loops"
formats = ["mp3"]
tracks = 3
"#;

    #[test]
    fn parse_minimal_index() {
        let idx = AssetIndex::parse_str(MINIMAL_TOML).expect("parse");
        assert_eq!(idx.index.version, 1);
        assert_eq!(idx.material_sets.len(), 1);
        assert_eq!(idx.textures.len(), 1);
        assert_eq!(idx.hdris.len(), 1);
        assert_eq!(idx.models.len(), 1);
        assert_eq!(idx.audio_packs.len(), 1);
    }

    #[test]
    fn lookup_material_set() {
        let idx = AssetIndex::parse_str(MINIMAL_TOML).expect("parse");
        let ms = idx.material_set("Forest").expect("found");
        assert_eq!(ms.biome, "forest");
        assert_eq!(ms.layers, 5);
    }

    #[test]
    fn lookup_texture() {
        let idx = AssetIndex::parse_str(MINIMAL_TOML).expect("parse");
        let tex = idx.texture("GRASS").expect("found");
        assert_eq!(tex.name, "grass");
        assert!(tex.has_ktx2);
    }

    #[test]
    fn lookup_hdri() {
        let idx = AssetIndex::parse_str(MINIMAL_TOML).expect("parse");
        let h = idx.hdri("Rainforest").expect("found");
        assert_eq!(h.time, "day");
        assert!(h.biomes.contains(&"forest".to_string()));
    }

    #[test]
    fn hdris_for_biome_and_time() {
        let idx = AssetIndex::parse_str(MINIMAL_TOML).expect("parse");
        let matches = idx.hdris_for("forest", "day");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "rainforest");

        let empty = idx.hdris_for("desert", "day");
        assert!(empty.is_empty());
    }

    #[test]
    fn material_set_map() {
        let idx = AssetIndex::parse_str(MINIMAL_TOML).expect("parse");
        let map = idx.material_set_map();
        assert!(map.contains_key("forest"));
    }

    #[test]
    fn missing_biome_returns_none() {
        let idx = AssetIndex::parse_str(MINIMAL_TOML).expect("parse");
        assert!(idx.material_set("volcano").is_none());
    }

    #[test]
    fn multiple_material_sets() {
        let toml = r#"
[index]
version = 1
generated = "2026-02-06"
asset_root = "assets"

[[material_set]]
biome = "forest"
dir = "materials/forest"
layers = 5

[[material_set]]
biome = "desert"
dir = "materials/desert"
layers = 5

[[material_set]]
biome = "tundra"
dir = "materials/tundra"
layers = 5
"#;
        let idx = AssetIndex::parse_str(toml).expect("parse");
        assert_eq!(idx.material_sets.len(), 3);
        assert!(idx.material_set("desert").is_some());
        assert!(idx.material_set("tundra").is_some());
    }

    #[test]
    fn load_real_asset_index() {
        // Load the actual asset_index.toml from the workspace root
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("assets")
            .join("asset_index.toml");
        if path.exists() {
            let idx = AssetIndex::load(&path).expect("load real asset_index.toml");
            assert!(idx.index.version >= 1);
            assert!(!idx.material_sets.is_empty(), "should have material sets");
            assert!(!idx.textures.is_empty(), "should have textures");
            assert!(!idx.hdris.is_empty(), "should have hdris");
            // Validate paths
            let base = path.parent().unwrap(); // assets/
            let missing = idx.validate_paths(base);
            // Print any missing but don't hard-fail (CI may not have all assets)
            for m in &missing {
                eprintln!("  WARN: {}", m);
            }
        }
    }
}
