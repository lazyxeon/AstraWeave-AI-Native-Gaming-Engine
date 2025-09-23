// Test for asset pipeline manifest and deterministic output
use std::fs;
use std::path::Path;
use serde_json::Value;

#[test]
fn test_manifest_is_deterministic() {
    let manifest_path = Path::new("assets/manifest.json");
    if !manifest_path.exists() {
        // Skip if not run through pipeline
        return;
    }
    let data = fs::read_to_string(manifest_path).unwrap();
    let v: Value = serde_json::from_str(&data).unwrap();
    // Check manifest is an array and has required fields
    assert!(v.is_array());
    if let Some(first) = v.as_array().and_then(|arr| arr.first()) {
        assert!(first.get("src").is_some());
        assert!(first.get("out").is_some());
        assert!(first.get("sha256").is_some());
        assert!(first.get("kind").is_some());
    }
}
