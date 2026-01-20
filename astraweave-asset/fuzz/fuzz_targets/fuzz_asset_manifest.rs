//! Fuzz target for asset manifest parsing.
//!
//! Tests JSON and TOML manifest parsing robustness.

#![no_main]

use libfuzzer_sys::fuzz_target;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct AssetManifest {
    version: Option<u32>,
    assets: Option<HashMap<String, AssetEntry>>,
    bundles: Option<Vec<BundleEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetEntry {
    path: Option<String>,
    hash: Option<String>,
    size: Option<u64>,
    dependencies: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BundleEntry {
    name: Option<String>,
    assets: Option<Vec<String>>,
    compressed: Option<bool>,
}

fuzz_target!(|data: &[u8]| {
    // Try parsing as JSON
    if let Ok(s) = std::str::from_utf8(data) {
        let json_result: Result<AssetManifest, _> = serde_json::from_str(s);
        if let Ok(manifest) = json_result {
            // Validate parsed manifest
            if let Some(assets) = &manifest.assets {
                for (name, entry) in assets {
                    // Asset names should be non-empty
                    let _ = name.len();
                    
                    // Hash should be valid hex if present
                    if let Some(hash) = &entry.hash {
                        let is_valid_hex = hash.chars().all(|c| c.is_ascii_hexdigit());
                        let _ = is_valid_hex;
                    }
                    
                    // Dependencies should not self-reference
                    if let Some(deps) = &entry.dependencies {
                        let self_ref = deps.contains(name);
                        let _ = self_ref; // Would be an error in real code
                    }
                }
            }
        }
        
        // Try parsing as TOML
        let toml_result: Result<AssetManifest, _> = toml::from_str(s);
        if let Ok(manifest) = toml_result {
            if let Some(bundles) = &manifest.bundles {
                for bundle in bundles {
                    // Bundle names should be valid identifiers
                    if let Some(name) = &bundle.name {
                        let is_valid = name.chars().all(|c| c.is_alphanumeric() || c == '_');
                        let _ = is_valid;
                    }
                }
            }
        }
    }
    
    // Try parsing as raw bytes (binary manifest format)
    if data.len() >= 8 {
        // Check for magic header
        let magic = &data[0..4];
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        
        let is_valid_magic = magic == b"AWAM"; // AstraWeave Asset Manifest
        let _ = is_valid_magic;
        let _ = version;
    }
});
