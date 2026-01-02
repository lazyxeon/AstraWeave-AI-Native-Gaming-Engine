#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use astraweave_blend::cache::CacheEntry;
use astraweave_blend::version::BlenderVersion;
use std::path::PathBuf;

#[derive(Debug, Arbitrary)]
struct FuzzCacheEntryInput {
    // Hash strings
    source_hash_bytes: Vec<u8>,
    options_hash_bytes: Vec<u8>,
    
    // Version
    major: u32,
    minor: u32,
    patch: u32,
    
    // Paths
    output_path: String,
    source_path: String,
    texture_paths: Vec<String>,
    linked_libs: Vec<String>,
    
    // Metrics
    output_size: u64,
    conversion_duration_ms: u64,
}

fuzz_target!(|input: FuzzCacheEntryInput| {
    // Create version
    let version = BlenderVersion::new(input.major, input.minor, input.patch);
    
    // Convert hash bytes to hex strings (simulating actual hash output)
    let source_hash = hex::encode(&input.source_hash_bytes);
    let options_hash = hex::encode(&input.options_hash_bytes);
    
    // Create paths (limit length to prevent memory issues)
    let output_path = PathBuf::from(&input.output_path[..input.output_path.len().min(1000)]);
    let source_path = PathBuf::from(&input.source_path[..input.source_path.len().min(1000)]);
    
    // Create cache entry
    let mut entry = CacheEntry::new(
        source_hash.clone(),
        options_hash.clone(),
        version.clone(),
        output_path.clone(),
        source_path.clone(),
        input.output_size,
        input.conversion_duration_ms.min(u64::MAX / 2), // Prevent overflow
    );
    
    // Test basic accessors
    assert_eq!(entry.source_hash, source_hash);
    assert_eq!(entry.options_hash, options_hash);
    assert_eq!(entry.blender_version, version);
    assert_eq!(entry.output_path, output_path);
    assert_eq!(entry.source_path, source_path);
    assert_eq!(entry.output_size, input.output_size);
    
    // Add texture files (limited)
    for tex in input.texture_paths.iter().take(10) {
        entry.texture_files.push(PathBuf::from(&tex[..tex.len().min(200)]));
    }
    
    // Add linked libraries (limited)
    for lib in input.linked_libs.iter().take(10) {
        entry.linked_libraries.push(PathBuf::from(&lib[..lib.len().min(200)]));
    }
    
    // Test touch() - updates last_accessed
    let before_access = entry.last_accessed;
    entry.touch();
    // last_accessed should be >= before (or same if very fast)
    assert!(entry.last_accessed >= before_access);
    
    // Test age() - should return a Duration
    let age = entry.age();
    // Age should be non-negative (Duration is always non-negative)
    let _ = age.as_secs();
    
    // Test time_since_access()
    let since_access = entry.time_since_access();
    let _ = since_access.as_secs();
    
    // Test Clone
    let entry_clone = entry.clone();
    assert_eq!(entry.source_hash, entry_clone.source_hash);
    assert_eq!(entry.options_hash, entry_clone.options_hash);
    assert_eq!(entry.blender_version, entry_clone.blender_version);
    assert_eq!(entry.output_size, entry_clone.output_size);
    
    // Test Debug format (should not panic)
    let _ = format!("{:?}", entry);
    
    // Test serialization roundtrip - RON
    if let Ok(serialized) = ron::to_string(&entry) {
        if let Ok(deserialized) = ron::from_str::<CacheEntry>(&serialized) {
            assert_eq!(entry.source_hash, deserialized.source_hash);
            assert_eq!(entry.options_hash, deserialized.options_hash);
            assert_eq!(entry.blender_version, deserialized.blender_version);
            assert_eq!(entry.output_size, deserialized.output_size);
        }
    }
    
    // Test serialization roundtrip - JSON
    if let Ok(serialized) = serde_json::to_string(&entry) {
        if let Ok(deserialized) = serde_json::from_str::<CacheEntry>(&serialized) {
            assert_eq!(entry.source_hash, deserialized.source_hash);
            assert_eq!(entry.options_hash, deserialized.options_hash);
        }
    }
    
    // Test serialization roundtrip - bincode
    if let Ok(serialized) = bincode::serialize(&entry) {
        if let Ok(deserialized) = bincode::deserialize::<CacheEntry>(&serialized) {
            assert_eq!(entry.source_hash, deserialized.source_hash);
        }
    }
    
    // Test with various timestamp values
    let timestamps = [0u64, 1, u32::MAX as u64, u64::MAX / 2];
    for ts in timestamps {
        let mut test_entry = entry.clone();
        test_entry.created_at = ts;
        test_entry.last_accessed = ts;
        let _ = test_entry.age();
        let _ = test_entry.time_since_access();
    }
});
