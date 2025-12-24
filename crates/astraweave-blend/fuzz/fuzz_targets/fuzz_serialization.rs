#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use astraweave_blend::version::BlenderVersion;
use astraweave_blend::options::{
    ConversionOptions, OutputFormat,
};
use astraweave_blend::cache::CacheEntry;

use std::path::PathBuf;

/// Test serialization invariants across all serializable types
#[derive(Debug, Arbitrary)]
struct FuzzSerializationInput {
    // Version fields
    major: u32,
    minor: u32,
    patch: u32,
    
    // Options fields
    output_format: u8,
    export_animations: bool,
    export_materials: bool,
    apply_modifiers: bool,
    
    // Cache fields
    source_hash_bytes: Vec<u8>,
    options_hash_bytes: Vec<u8>,
    output_path: String,
    source_path: String,
    file_size: u64,
    
    // Malformed serialization attempts
    malformed_ron: String,
    malformed_json: String,
}

fuzz_target!(|input: FuzzSerializationInput| {
    // ========================================================================
    // VERSION SERIALIZATION
    // ========================================================================
    
    let version = BlenderVersion::new(input.major, input.minor, input.patch);
    
    // RON roundtrip
    if let Ok(ron_str) = ron::to_string(&version) {
        // Valid serialization should always deserialize
        let deserialized: BlenderVersion = ron::from_str(&ron_str)
            .expect("Valid RON should deserialize");
        assert_eq!(version, deserialized);
        
        // Check that re-serialization produces same result
        let re_ron = ron::to_string(&deserialized).expect("Re-serialization should work");
        assert_eq!(ron_str, re_ron, "Re-serialization should be identical");
    }
    
    // JSON roundtrip
    if let Ok(json_str) = serde_json::to_string(&version) {
        let deserialized: BlenderVersion = serde_json::from_str(&json_str)
            .expect("Valid JSON should deserialize");
        assert_eq!(version, deserialized);
    }
    
    // Pretty JSON roundtrip
    if let Ok(json_pretty) = serde_json::to_string_pretty(&version) {
        let deserialized: BlenderVersion = serde_json::from_str(&json_pretty)
            .expect("Pretty JSON should deserialize");
        assert_eq!(version, deserialized);
    }
    
    // ========================================================================
    // OPTIONS SERIALIZATION
    // ========================================================================
    
    let format = match input.output_format % 3 {
        0 => OutputFormat::Gltf,
        1 => OutputFormat::Glb,
        _ => OutputFormat::Both,
    };
    
    let options = ConversionOptions::builder()
        .output_format(format)
        .export_animations(input.export_animations)
        .export_materials(input.export_materials)
        .apply_modifiers(input.apply_modifiers)
        .build();
    
    // ConversionOptions RON roundtrip
    if let Ok(ron_str) = ron::to_string(&options) {
        if let Ok(deserialized) = ron::from_str::<ConversionOptions>(&ron_str) {
            assert_eq!(options.export_animations, deserialized.export_animations);
            assert_eq!(options.export_materials, deserialized.export_materials);
        }
    }
    
    // ConversionOptions JSON roundtrip
    if let Ok(json_str) = serde_json::to_string(&options) {
        if let Ok(deserialized) = serde_json::from_str::<ConversionOptions>(&json_str) {
            assert_eq!(options.apply_modifiers, deserialized.apply_modifiers);
        }
    }
    
    // ========================================================================
    // CACHE ENTRY SERIALIZATION
    // ========================================================================
    
    let source_hash = hex::encode(&input.source_hash_bytes[..input.source_hash_bytes.len().min(64)]);
    let options_hash = hex::encode(&input.options_hash_bytes[..input.options_hash_bytes.len().min(64)]);
    
    let entry = CacheEntry::new(
        source_hash.clone(),
        options_hash.clone(),
        version.clone(),
        PathBuf::from(&input.output_path[..input.output_path.len().min(200)]),
        PathBuf::from(&input.source_path[..input.source_path.len().min(200)]),
        input.file_size,
        100, // conversion duration
    );
    
    // CacheEntry RON roundtrip
    if let Ok(ron_str) = ron::to_string(&entry) {
        if let Ok(deserialized) = ron::from_str::<CacheEntry>(&ron_str) {
            assert_eq!(entry.source_hash, deserialized.source_hash);
            assert_eq!(entry.options_hash, deserialized.options_hash);
            assert_eq!(entry.output_size, deserialized.output_size);
        }
    }
    
    // CacheEntry JSON roundtrip
    if let Ok(json_str) = serde_json::to_string(&entry) {
        if let Ok(deserialized) = serde_json::from_str::<CacheEntry>(&json_str) {
            assert_eq!(entry.source_hash, deserialized.source_hash);
        }
    }
    
    // ========================================================================
    // MALFORMED INPUT HANDLING
    // ========================================================================
    
    // These should not panic, just return errors
    
    // Malformed RON
    let _ = ron::from_str::<BlenderVersion>(&input.malformed_ron);
    let _ = ron::from_str::<ConversionOptions>(&input.malformed_ron);
    let _ = ron::from_str::<CacheEntry>(&input.malformed_ron);
    
    // Malformed JSON
    let _ = serde_json::from_str::<BlenderVersion>(&input.malformed_json);
    let _ = serde_json::from_str::<ConversionOptions>(&input.malformed_json);
    let _ = serde_json::from_str::<CacheEntry>(&input.malformed_json);
    
    // Empty strings
    let _ = ron::from_str::<BlenderVersion>("");
    let _ = serde_json::from_str::<BlenderVersion>("");
    
    // Null bytes
    let _ = ron::from_str::<BlenderVersion>("\0\0\0");
    let _ = serde_json::from_str::<BlenderVersion>("\0\0\0");
    
    // Unicode edge cases
    let _ = ron::from_str::<BlenderVersion>("\u{FEFF}"); // BOM
    let _ = ron::from_str::<BlenderVersion>("\u{202E}"); // RTL override
    
    // Very long strings should not cause OOM (truncate test)
    if input.malformed_ron.len() < 10000 {
        let _ = ron::from_str::<ConversionOptions>(&input.malformed_ron);
    }
    
    // ========================================================================
    // BINCODE ROUNDTRIP
    // ========================================================================
    
    // Binary format testing
    if let Ok(bytes) = bincode::serialize(&version) {
        if let Ok(deserialized) = bincode::deserialize::<BlenderVersion>(&bytes) {
            assert_eq!(version, deserialized);
        }
    }
    
    if let Ok(bytes) = bincode::serialize(&entry) {
        if let Ok(deserialized) = bincode::deserialize::<CacheEntry>(&bytes) {
            assert_eq!(entry.source_hash, deserialized.source_hash);
        }
    }
    
    // ========================================================================
    // CROSS-FORMAT CONSISTENCY
    // ========================================================================
    
    // Deserialize from one format, serialize to another, should preserve data
    if let Ok(json_str) = serde_json::to_string(&version) {
        if let Ok(from_json) = serde_json::from_str::<BlenderVersion>(&json_str) {
            if let Ok(to_ron) = ron::to_string(&from_json) {
                if let Ok(from_ron) = ron::from_str::<BlenderVersion>(&to_ron) {
                    assert_eq!(version, from_ron, "Cross-format roundtrip failed");
                }
            }
        }
    }
});
