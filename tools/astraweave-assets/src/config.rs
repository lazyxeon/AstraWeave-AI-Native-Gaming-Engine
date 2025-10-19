use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root configuration loaded from polyhaven_manifest.toml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetManifest {
    /// Where to write downloaded assets (default: "assets/_downloaded")
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Where to cache temp files and lockfile (default: ".asset_cache")
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    /// Texture materials (PBR maps)
    #[serde(default)]
    pub textures: HashMap<String, TextureAsset>,

    /// HDRIs for environment lighting
    #[serde(default)]
    pub hdris: HashMap<String, HdriAsset>,

    /// 3D models (GLB/GLTF)
    #[serde(default)]
    pub models: HashMap<String, ModelAsset>,
}

/// Texture asset definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextureAsset {
    /// PolyHaven asset ID (e.g., "forest_leaves", "metal_plate")
    pub id: String,

    /// Asset type (must be "texture")
    pub kind: String,

    /// Desired resolution ("1k", "2k", "4k", "8k")
    pub res: String,

    /// Maps to download (albedo, normal, roughness, metallic, ao, height, etc.)
    pub maps: Vec<String>,

    /// Optional tags for organization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// HDRI asset definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HdriAsset {
    /// PolyHaven asset ID (e.g., "kloppenheim_06_puresky")
    pub id: String,

    /// Asset type (must be "hdri")
    pub kind: String,

    /// Desired resolution ("1k", "2k", "4k", "8k")
    pub res: String,

    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Model asset definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelAsset {
    /// PolyHaven asset ID (e.g., "rock_collection_a")
    pub id: String,

    /// Asset type (must be "model")
    pub kind: String,

    /// Desired resolution for textures ("1k", "2k", "4k", "8k")
    pub res: String,

    /// Preferred format ("glb", "blend", "fbx")
    #[serde(default = "default_model_format")]
    pub format: String,

    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Lockfile entry (tracks downloaded assets)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LockEntry {
    /// Asset handle from manifest (e.g., "forest_ground")
    pub handle: String,

    /// PolyHaven asset ID
    pub id: String,

    /// Asset kind (texture/hdri/model)
    pub kind: String,

    /// Resolved download URLs
    pub urls: HashMap<String, String>,

    /// Local file paths
    pub paths: HashMap<String, PathBuf>,

    /// SHA256 hashes for integrity
    pub hashes: HashMap<String, String>,

    /// Download timestamp
    pub timestamp: String,

    /// Resolved resolution (may differ from requested if fallback used)
    pub resolved_res: String,
}

/// Lockfile structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Lockfile {
    /// Lockfile version
    pub version: u32,

    /// Locked assets
    pub assets: HashMap<String, LockEntry>,
}

// Default functions
fn default_output_dir() -> PathBuf {
    PathBuf::from("assets/_downloaded")
}

fn default_cache_dir() -> PathBuf {
    PathBuf::from(".asset_cache")
}

fn default_model_format() -> String {
    "glb".to_string()
}

impl AssetManifest {
    /// Load manifest from TOML file
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: AssetManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Save manifest to TOML file
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Lockfile {
    /// Load lockfile from TOML file
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Lockfile {
                version: 1,
                assets: HashMap::new(),
            });
        }

        let content = std::fs::read_to_string(path)?;
        let lockfile: Lockfile = toml::from_str(&content)?;
        Ok(lockfile)
    }

    /// Save lockfile to TOML file
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Check if asset exists and is valid
    pub fn is_valid(&self, handle: &str, _paths: &HashMap<String, PathBuf>) -> bool {
        if let Some(entry) = self.assets.get(handle) {
            // Check all paths exist
            for path in entry.paths.values() {
                if !path.exists() {
                    return false;
                }

                // Optionally verify hash (expensive)
                // if let Some(expected_hash) = entry.hashes.get(map) {
                //     if !verify_hash(path, expected_hash) {
                //         return false;
                //     }
                // }
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_manifest_defaults() {
        let manifest = AssetManifest {
            output_dir: default_output_dir(),
            cache_dir: default_cache_dir(),
            textures: HashMap::new(),
            hdris: HashMap::new(),
            models: HashMap::new(),
        };

        assert_eq!(manifest.output_dir, PathBuf::from("assets/_downloaded"));
        assert_eq!(manifest.cache_dir, PathBuf::from(".asset_cache"));
    }

    #[test]
    fn test_parse_texture_asset() {
        let toml = r#"
            [textures."forest_ground"]
            id = "forest_leaves"
            kind = "texture"
            res = "2k"
            maps = ["albedo", "normal", "roughness"]
            tags = ["biome:forest"]
        "#;

        let manifest: AssetManifest = toml::from_str(toml).unwrap();
        let texture = manifest.textures.get("forest_ground").unwrap();

        assert_eq!(texture.id, "forest_leaves");
        assert_eq!(texture.res, "2k");
        assert_eq!(texture.maps.len(), 3);
        assert_eq!(texture.tags, vec!["biome:forest"]);
    }

    #[test]
    fn test_parse_hdri_asset() {
        let toml = r#"
            [hdris."studio"]
            id = "studio_small_09"
            kind = "hdri"
            res = "4k"
            tags = ["indoor", "neutral"]
        "#;

        let manifest: AssetManifest = toml::from_str(toml).unwrap();
        
        assert_eq!(manifest.hdris.len(), 1);
        let hdri = manifest.hdris.get("studio").unwrap();
        assert_eq!(hdri.id, "studio_small_09");
        assert_eq!(hdri.res, "4k");
        assert_eq!(hdri.tags, vec!["indoor", "neutral"]);
    }

    #[test]
    fn test_parse_model_asset() {
        let toml = r#"
            [models."rock_set"]
            id = "rocks_collection_a"
            kind = "model"
            res = "2k"
            format = "glb"
            tags = ["nature"]
        "#;

        let manifest: AssetManifest = toml::from_str(toml).unwrap();
        
        assert_eq!(manifest.models.len(), 1);
        let model = manifest.models.get("rock_set").unwrap();
        assert_eq!(model.id, "rocks_collection_a");
        assert_eq!(model.res, "2k");
        assert_eq!(model.format, "glb");
    }

    #[test]
    fn test_manifest_with_custom_dirs() {
        let toml = r#"
            output_dir = "custom/output"
            cache_dir = "custom/cache"
            
            [textures."test"]
            id = "test_id"
            kind = "texture"
            res = "1k"
            maps = ["albedo"]
        "#;

        let manifest: AssetManifest = toml::from_str(toml).unwrap();
        
        assert_eq!(manifest.output_dir, PathBuf::from("custom/output"));
        assert_eq!(manifest.cache_dir, PathBuf::from("custom/cache"));
    }

    #[test]
    fn test_manifest_missing_required_field() {
        let toml = r#"
            [textures."broken"]
            kind = "texture"
            res = "2k"
            maps = ["albedo"]
        "#;

        let result: Result<AssetManifest, _> = toml::from_str(toml);
        assert!(result.is_err(), "Should fail when 'id' field is missing");
    }

    #[test]
    fn test_lockfile_serialization() {
        let mut lockfile = Lockfile {
            version: 1,
            assets: HashMap::new(),
        };

        let entry = LockEntry {
            handle: "test".to_string(),
            id: "test_asset".to_string(),
            kind: "texture".to_string(),
            urls: HashMap::new(),
            paths: HashMap::new(),
            hashes: HashMap::new(),
            timestamp: "2025-10-17T00:00:00Z".to_string(),
            resolved_res: "2k".to_string(),
        };

        lockfile.assets.insert("test".to_string(), entry);

        let toml = toml::to_string_pretty(&lockfile).unwrap();
        assert!(toml.contains("version = 1"));
        assert!(toml.contains("test_asset"));
    }

    #[test]
    fn test_lockfile_roundtrip() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let lockfile_path = temp.path().join("test.lock");

        // Create lockfile
        let mut lockfile = Lockfile {
            version: 1,
            assets: HashMap::new(),
        };

        let entry = LockEntry {
            handle: "test_asset".to_string(),
            id: "test_id".to_string(),
            kind: "texture".to_string(),
            urls: [("albedo".to_string(), "https://example.com/albedo.png".to_string())]
                .iter()
                .cloned()
                .collect(),
            paths: [("albedo".to_string(), PathBuf::from("test/albedo.png"))]
                .iter()
                .cloned()
                .collect(),
            hashes: [("albedo".to_string(), "abc123".to_string())]
                .iter()
                .cloned()
                .collect(),
            timestamp: "2025-10-17T12:00:00Z".to_string(),
            resolved_res: "2k".to_string(),
        };

        lockfile.assets.insert("test_asset".to_string(), entry);

        // Save
        lockfile.save(&lockfile_path).unwrap();
        assert!(lockfile_path.exists());

        // Load
        let loaded = Lockfile::load(&lockfile_path).unwrap();
        
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.assets.len(), 1);
        
        let loaded_entry = loaded.assets.get("test_asset").unwrap();
        assert_eq!(loaded_entry.id, "test_id");
        assert_eq!(loaded_entry.kind, "texture");
        assert_eq!(loaded_entry.resolved_res, "2k");
        assert_eq!(loaded_entry.urls.len(), 1);
        assert_eq!(loaded_entry.paths.len(), 1);
        assert_eq!(loaded_entry.hashes.len(), 1);
    }

    #[test]
    fn test_lockfile_load_nonexistent() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let lockfile_path = temp.path().join("nonexistent.lock");

        let lockfile = Lockfile::load(&lockfile_path).unwrap();
        
        assert_eq!(lockfile.version, 1);
        assert_eq!(lockfile.assets.len(), 0);
    }

    #[test]
    fn test_lockfile_is_valid_all_paths_exist() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        
        // Create test file
        let test_file = temp.path().join("test.png");
        fs::write(&test_file, b"test data").unwrap();

        let mut lockfile = Lockfile {
            version: 1,
            assets: HashMap::new(),
        };

        let entry = LockEntry {
            handle: "test".to_string(),
            id: "test_id".to_string(),
            kind: "texture".to_string(),
            urls: HashMap::new(),
            paths: [("albedo".to_string(), test_file.clone())]
                .iter()
                .cloned()
                .collect(),
            hashes: HashMap::new(),
            timestamp: "2025-10-17T12:00:00Z".to_string(),
            resolved_res: "2k".to_string(),
        };

        lockfile.assets.insert("test".to_string(), entry);

        assert!(lockfile.is_valid("test", &HashMap::new()));
    }

    #[test]
    fn test_lockfile_is_valid_missing_path() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let nonexistent_file = temp.path().join("nonexistent.png");

        let mut lockfile = Lockfile {
            version: 1,
            assets: HashMap::new(),
        };

        let entry = LockEntry {
            handle: "test".to_string(),
            id: "test_id".to_string(),
            kind: "texture".to_string(),
            urls: HashMap::new(),
            paths: [("albedo".to_string(), nonexistent_file)]
                .iter()
                .cloned()
                .collect(),
            hashes: HashMap::new(),
            timestamp: "2025-10-17T12:00:00Z".to_string(),
            resolved_res: "2k".to_string(),
        };

        lockfile.assets.insert("test".to_string(), entry);

        assert!(!lockfile.is_valid("test", &HashMap::new()));
    }

    #[test]
    fn test_lockfile_is_valid_missing_handle() {
        let lockfile = Lockfile {
            version: 1,
            assets: HashMap::new(),
        };

        assert!(!lockfile.is_valid("nonexistent", &HashMap::new()));
    }

    #[test]
    fn test_manifest_load_save_roundtrip() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("manifest.toml");

        // Create manifest
        let mut textures = HashMap::new();
        textures.insert(
            "test_texture".to_string(),
            TextureAsset {
                id: "test_id".to_string(),
                kind: "texture".to_string(),
                res: "2k".to_string(),
                maps: vec!["albedo".to_string()],
                tags: vec!["test".to_string()],
            },
        );

        let manifest = AssetManifest {
            output_dir: PathBuf::from("custom/output"),
            cache_dir: PathBuf::from("custom/cache"),
            textures,
            hdris: HashMap::new(),
            models: HashMap::new(),
        };

        // Save
        manifest.save(&manifest_path).unwrap();
        assert!(manifest_path.exists());

        // Load
        let loaded = AssetManifest::load(&manifest_path).unwrap();
        
        assert_eq!(loaded.output_dir, PathBuf::from("custom/output"));
        assert_eq!(loaded.cache_dir, PathBuf::from("custom/cache"));
        assert_eq!(loaded.textures.len(), 1);
        
        let texture = loaded.textures.get("test_texture").unwrap();
        assert_eq!(texture.id, "test_id");
        assert_eq!(texture.maps, vec!["albedo"]);
    }

    #[test]
    fn test_manifest_load_invalid_toml() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let manifest_path = temp.path().join("invalid.toml");
        
        // Write invalid TOML
        fs::write(&manifest_path, "this is not valid toml {[}]").unwrap();

        let result = AssetManifest::load(&manifest_path);
        assert!(result.is_err(), "Should fail on invalid TOML");
    }

    #[test]
    fn test_lockfile_save_creates_valid_toml() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let lockfile_path = temp.path().join("test.lock");

        let mut lockfile = Lockfile {
            version: 1,
            assets: HashMap::new(),
        };

        let entry = LockEntry {
            handle: "test".to_string(),
            id: "test_id".to_string(),
            kind: "texture".to_string(),
            urls: HashMap::new(),
            paths: HashMap::new(),
            hashes: HashMap::new(),
            timestamp: "2025-10-18T00:00:00Z".to_string(),
            resolved_res: "2k".to_string(),
        };

        lockfile.assets.insert("test".to_string(), entry);

        // Save
        lockfile.save(&lockfile_path).unwrap();

        // Read and verify it's valid TOML
        let content = fs::read_to_string(&lockfile_path).unwrap();
        let parsed: Lockfile = toml::from_str(&content).unwrap();
        
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.assets.len(), 1);
    }
}
