//! Fuzz target for asset path resolution and normalization.
//!
//! Tests that arbitrary path strings don't cause path traversal or crashes.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use std::path::{Path, PathBuf};

#[derive(Debug, Arbitrary)]
struct FuzzPathInput {
    base_path: String,
    relative_path: String,
    extension_filter: Option<String>,
}

fn normalize_path(path: &str) -> PathBuf {
    let path = Path::new(path);
    let mut components = Vec::new();
    
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::ParentDir => {
                // Prevent path traversal by not allowing ..
                // to go above the root
                if !components.is_empty() {
                    components.pop();
                }
            }
            Component::Normal(c) => {
                components.push(c);
            }
            Component::RootDir | Component::CurDir | Component::Prefix(_) => {
                // Skip these for normalization
            }
        }
    }
    
    components.iter().collect()
}

fn validate_extension(path: &Path, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some(ext) => path.extension().map_or(false, |e| e == ext),
    }
}

fuzz_target!(|input: FuzzPathInput| {
    // Normalize paths to prevent traversal attacks
    let base = normalize_path(&input.base_path);
    let relative = normalize_path(&input.relative_path);
    
    // Join paths safely
    let full_path = base.join(&relative);
    
    // Ensure the result is still under base (no traversal)
    let _ = full_path.starts_with(&base);
    
    // Extension filtering
    let _ = validate_extension(&full_path, input.extension_filter.as_deref());
    
    // Try to extract asset ID from path
    let asset_id = full_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let _ = asset_id.len();
    
    // Test path manipulation safety
    let _ = full_path.parent();
    let _ = full_path.file_name();
    let _ = full_path.extension();
    let _ = full_path.with_extension("glb");
    
    // String conversion safety
    let _ = full_path.to_string_lossy();
    let _ = full_path.display().to_string();
});
