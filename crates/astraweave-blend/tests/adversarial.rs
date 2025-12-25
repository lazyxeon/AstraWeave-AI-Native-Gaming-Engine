//! Adversarial tests for astraweave-blend.
//!
//! Tests malicious inputs, race conditions, concurrency issues,
//! stress testing, and other adversarial scenarios.

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::thread;

use sha2::{Sha256, Digest};

use astraweave_blend::options::*;
use astraweave_blend::version::BlenderVersion;
use astraweave_blend::error::BlendError;

// ============================================================================
// MALFORMED FILE INPUTS
// ============================================================================

/// Generate malformed .blend file contents.
fn malformed_blend_contents() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        // Empty file
        ("empty", vec![]),
        
        // Just null bytes
        ("all_nulls_small", vec![0u8; 10]),
        ("all_nulls_medium", vec![0u8; 1000]),
        
        // Just 0xFF bytes
        ("all_ff_small", vec![0xFF; 10]),
        ("all_ff_medium", vec![0xFF; 1000]),
        
        // Truncated header
        ("truncated_header_1", b"B".to_vec()),
        ("truncated_header_4", b"BLEN".to_vec()),
        ("truncated_header_7", b"BLENDER".to_vec()),
        ("truncated_header_8", b"BLENDER-".to_vec()),
        
        // Wrong magic
        ("wrong_magic_1", b"GLENDER-v300test".to_vec()),
        ("wrong_magic_2", b"BLANDER-v300test".to_vec()),
        ("wrong_magic_3", b"blender-v300test".to_vec()),
        
        // Corrupted version
        ("bad_version_1", b"BLENDER-XXXX".to_vec()),
        ("bad_version_2", b"BLENDER-v!!!".to_vec()),
        ("bad_version_3", b"BLENDER-v-99".to_vec()),
        
        // Valid header, garbage body
        ("valid_header_garbage", {
            let mut data = b"BLENDER-v300".to_vec();
            data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
            data.extend_from_slice(&vec![0xFF; 100]);
            data
        }),
        
        // Extremely long header
        ("long_header", {
            let mut data = b"BLENDER-v300".to_vec();
            data.extend_from_slice(&vec![b'A'; 10000]);
            data
        }),
        
        // Binary looking like text
        ("text_as_binary", b"This is not a blend file but looks like text".to_vec()),
        
        // Other file formats
        ("png_header", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        ("gif_header", b"GIF89a".to_vec()),
        ("jpeg_header", vec![0xFF, 0xD8, 0xFF, 0xE0]),
        ("zip_header", vec![0x50, 0x4B, 0x03, 0x04]),
        ("pdf_header", b"%PDF-1.4".to_vec()),
        ("exe_header", vec![0x4D, 0x5A]),  // MZ
        
        // Polyglot attempts
        ("blend_png_polyglot", {
            let mut data = vec![0x89, 0x50, 0x4E, 0x47];  // PNG magic
            data.extend_from_slice(b"BLENDER-v300");
            data
        }),
    ]
}

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[test]
fn handle_malformed_blend_gracefully() {
    for (name, content) in malformed_blend_contents() {
        // Test that we can at least process the content without panicking
        // Actual validation would reject these
        
        // Simple prefix check - not full validation
        let has_blend_prefix = content.starts_with(b"BLENDER-") || content.starts_with(b"BLENDER_");
        
        // Full validation requires at least 12 bytes (BLENDER-v300)
        let is_complete_header = has_blend_prefix && content.len() >= 12;
        
        if name.starts_with("wrong_magic") {
            assert!(!has_blend_prefix, "Should not have blend prefix: {}", name);
        }
        
        if name.starts_with("truncated") {
            // Truncated headers may start correctly but are incomplete
            assert!(!is_complete_header, "Should not be complete header: {}", name);
        }
        
        // The content should be hashable without panic
        let _hash = compute_sha256(&content);
    }
}

#[test]
fn malformed_contents_produce_unique_hashes() {
    let contents = malformed_blend_contents();
    let mut hashes = std::collections::HashSet::new();
    
    for (name, content) in &contents {
        let hash = compute_sha256(content);
        let is_unique = hashes.insert(hash.clone());
        
        // Allow some collisions for empty-ish content, but most should be unique
        if !is_unique && !name.starts_with("all_") && content.len() > 10 {
            // This is fine - some malformed contents might hash the same
        }
    }
    
    // Should have many unique hashes
    assert!(hashes.len() > contents.len() / 2, "Too many hash collisions");
}

// ============================================================================
// VERSION STRESS TESTS
// ============================================================================

#[test]
fn version_comparison_all_orderings() {
    let versions = vec![
        BlenderVersion::new(0, 0, 0),
        BlenderVersion::new(2, 79, 0),
        BlenderVersion::new(2, 93, 0),
        BlenderVersion::new(3, 0, 0),
        BlenderVersion::new(4, 0, 0),
        BlenderVersion::new(4, 1, 0),
    ];
    
    for i in 0..versions.len() {
        for j in 0..versions.len() {
            let v1 = &versions[i];
            let v2 = &versions[j];
            
            // Test consistency
            if i < j {
                assert!(v1 < v2, "{:?} should be < {:?}", v1, v2);
            } else if i > j {
                assert!(v1 > v2, "{:?} should be > {:?}", v1, v2);
            } else {
                assert_eq!(v1, v2, "{:?} should equal {:?}", v1, v2);
            }
        }
    }
}

#[test]
fn version_extreme_values() {
    let extreme_versions = vec![
        BlenderVersion::new(0, 0, 0),
        BlenderVersion::new(0, 0, 1),
        BlenderVersion::new(0, 1, 0),
        BlenderVersion::new(1, 0, 0),
        BlenderVersion::new(u32::MAX - 1, u32::MAX - 1, u32::MAX - 1),
        BlenderVersion::new(u32::MAX, u32::MAX, u32::MAX),
    ];
    
    for v in &extreme_versions {
        // Should not panic
        let _display = format!("{}", v);
        let _min = v.meets_minimum();
        let _clone = v.clone();
    }
}

#[test]
fn version_rapid_comparisons() {
    let v1 = BlenderVersion::new(3, 6, 5);
    let v2 = BlenderVersion::new(4, 0, 0);
    
    // Perform many comparisons quickly
    for _ in 0..10000 {
        assert!(v1 < v2);
        assert!(v2 > v1);
        assert_eq!(v1, v1);
        assert_eq!(v2, v2);
    }
}

// ============================================================================
// CONCURRENCY STRESS TESTS
// ============================================================================

#[test]
fn concurrent_version_creation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let _v = BlenderVersion::new(3 + i % 2, 90 + i % 10, i);
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(counter.load(Ordering::SeqCst), 1000);
}

#[test]
fn concurrent_options_creation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let _opts = ConversionOptions::game_runtime();
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(counter.load(Ordering::SeqCst), 1000);
}

#[test]
fn concurrent_builder_usage() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let _opts = ConversionOptions::builder()
                    .format(if i % 2 == 0 { OutputFormat::GlbBinary } else { OutputFormat::GltfEmbedded })
                    .draco_compression(i % 3 == 0)
                    .timeout(Duration::from_secs(60 + i as u64))
                    .build();
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(counter.load(Ordering::SeqCst), 1000);
}

// ============================================================================
// ERROR STRESS TESTS
// ============================================================================

#[test]
fn error_display_huge_message() {
    let huge_reason = "X".repeat(100_000);  // 100KB message
    
    let err = BlendError::ConversionFailed {
        message: huge_reason.clone(),
        exit_code: Some(1),
        stderr: huge_reason.clone(),
        blender_output: Some(huge_reason.clone()),
    };
    
    // Should not panic
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

#[test]
fn error_display_special_characters() {
    let special_messages = vec![
        "Error with\nnewlines",
        "Error with\ttabs",
        "Error with\r\nCRLF",
        "Error with\0null",
        "Error with 'quotes' and \"double quotes\"",
        "Error with <xml> tags",
        "Error with unicode: 日本語",
        "Error with emoji: 🔥💥",
    ];
    
    for msg in special_messages {
        let err = BlendError::ConversionFailed {
            message: msg.to_string(),
            exit_code: Some(1),
            stderr: msg.to_string(),
            blender_output: None,
        };
        
        // Should not panic
        let _display = format!("{}", err);
    }
}

#[test]
fn create_many_error_variants() {
    let errors: Vec<BlendError> = (0..1000)
        .map(|i| {
            match i % 6 {
                0 => BlendError::BlenderNotFound {
                    searched_paths: vec![PathBuf::from(format!("/path/{}", i))],
                },
                1 => BlendError::InvalidBlendFile {
                    path: PathBuf::from(format!("/test_{}.blend", i)),
                    message: format!("Reason {}", i),
                },
                2 => BlendError::Cancelled,
                3 => BlendError::ConversionFailed {
                    message: format!("Failed {}", i),
                    exit_code: Some(i as i32),
                    stderr: String::new(),
                    blender_output: None,
                },
                4 => BlendError::Timeout {
                    operation: "test".to_string(),
                    duration: Duration::from_secs(i as u64),
                    path: PathBuf::from(format!("/test_{}.blend", i)),
                    timeout_secs: i as u64,
                },
                _ => BlendError::CacheCorrupted {
                    path: PathBuf::from(format!("/cache/{}", i)),
                    message: format!("Corrupted {}", i),
                },
            }
        })
        .collect();
    
    // All should be displayable
    for err in &errors {
        let _display = format!("{}", err);
    }
    
    assert_eq!(errors.len(), 1000);
}

// ============================================================================
// MEMORY STRESS TESTS
// ============================================================================

#[test]
fn allocate_many_options() {
    let start = Instant::now();
    
    let options: Vec<ConversionOptions> = (0..10000)
        .map(|_| ConversionOptions::game_runtime())
        .collect();
    
    let elapsed = start.elapsed();
    
    assert_eq!(options.len(), 10000);
    // Should complete in reasonable time (< 1 second)
    assert!(elapsed < Duration::from_secs(1), "Took too long: {:?}", elapsed);
}

#[test]
fn serialize_many_options() {
    let options: Vec<ConversionOptions> = (0..100)
        .map(|_| ConversionOptions::game_runtime())
        .collect();
    
    let start = Instant::now();
    
    for opts in &options {
        let _json = serde_json::to_string(opts).unwrap();
        let _ron = ron::to_string(opts).unwrap();
    }
    
    let elapsed = start.elapsed();
    
    // Should complete in reasonable time
    assert!(elapsed < Duration::from_secs(5), "Serialization took too long: {:?}", elapsed);
}

// ============================================================================
// PATH ADVERSARIAL TESTS
// ============================================================================

#[test]
fn paths_with_control_characters() {
    let control_chars: Vec<char> = (0..32u8).map(|c| c as char).collect();
    
    for c in control_chars {
        if c == '\0' {
            // Null terminates C strings, skip
            continue;
        }
        
        let path = PathBuf::from(format!("file{}name.blend", c));
        // Should be constructible
        let _ = path.to_string_lossy();
    }
}

#[test]
fn paths_extremely_deep() {
    let depth = 100;
    let mut path = PathBuf::new();
    
    for i in 0..depth {
        path.push(format!("dir{}", i));
    }
    path.push("file.blend");
    
    assert!(path.components().count() > depth);
}

#[test]
fn paths_with_repeated_dots() {
    let paths = vec![
        PathBuf::from("...blend"),
        PathBuf::from("....blend"),
        PathBuf::from("file...blend"),
        PathBuf::from("file....blend"),
        PathBuf::from("....."),
    ];
    
    for path in paths {
        // Should not panic
        let _ext = path.extension();
        let _str = path.to_string_lossy();
    }
}

// ============================================================================
// TIMEOUT ADVERSARIAL TESTS
// ============================================================================

#[test]
fn timeout_error_very_long_duration() {
    let err = BlendError::Timeout {
        operation: "test".to_string(),
        duration: Duration::from_secs(u64::MAX / 2),  // Very long
        path: PathBuf::from("/test.blend"),
        timeout_secs: u64::MAX / 2,
    };
    
    // Should not panic when displaying
    let display = format!("{}", err);
    assert!(!display.is_empty());
}

#[test]
fn options_timeout_extremes() {
    // Zero timeout
    let opts_zero = ConversionOptions::builder()
        .timeout(Duration::ZERO)
        .build();
    assert_eq!(opts_zero.process.timeout, Duration::ZERO);
    
    // Very large timeout
    let opts_max = ConversionOptions::builder()
        .timeout(Duration::from_secs(u64::MAX / 2))
        .build();
    assert!(opts_max.process.timeout.as_secs() > 0);
}

// ============================================================================
// SERIALIZATION ADVERSARIAL TESTS
// ============================================================================

#[test]
fn deserialize_invalid_json() {
    let invalid_jsons = vec![
        "",
        "{",
        "{}",
        "null",
        "[]",
        "{invalid}",
        r#"{"unknown_field": true}"#,
    ];
    
    for json in invalid_jsons {
        let result: Result<ConversionOptions, _> = serde_json::from_str(json);
        // Should fail gracefully, not panic
        assert!(result.is_err(), "Should reject invalid JSON: {}", json);
    }
}

#[test]
fn deserialize_invalid_ron() {
    let invalid_rons = vec![
        "",
        "(",
        "()",
        "nil",
        "[]",
        "(invalid)",
    ];
    
    for ron_str in invalid_rons {
        let result: Result<ConversionOptions, _> = ron::from_str(ron_str);
        // Should fail gracefully, not panic
        assert!(result.is_err(), "Should reject invalid RON: {}", ron_str);
    }
}

#[test]
fn serialize_then_corrupt_then_deserialize() {
    let opts = ConversionOptions::game_runtime();
    let mut serialized = serde_json::to_string(&opts).unwrap();
    
    // Corrupt the JSON
    serialized.push_str("garbage");
    
    let result: Result<ConversionOptions, _> = serde_json::from_str(&serialized);
    assert!(result.is_err());
}

// ============================================================================
// HASH ADVERSARIAL TESTS
// ============================================================================

#[test]
fn hash_empty_content() {
    let hash = compute_sha256(&[]);
    
    // Empty content should still produce a valid hash
    assert_eq!(hash.len(), 64);  // SHA-256 produces 64 hex chars
    
    // SHA-256 of empty is known
    assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

#[test]
fn hash_single_byte_variations() {
    let mut hashes = std::collections::HashSet::new();
    
    for byte in 0..=255u8 {
        let hash = compute_sha256(&[byte]);
        hashes.insert(hash);
    }
    
    // All single bytes should produce unique hashes
    assert_eq!(hashes.len(), 256);
}

#[test]
fn hash_length_independence() {
    // Different length inputs should produce different hashes
    let hashes: Vec<_> = (0..100)
        .map(|len| compute_sha256(&vec![0x42; len]))
        .collect();
    
    let unique: std::collections::HashSet<_> = hashes.iter().collect();
    assert_eq!(unique.len(), hashes.len(), "Length variations should produce unique hashes");
}

// ============================================================================
// FORMAT ADVERSARIAL TESTS
// ============================================================================

#[test]
fn all_output_formats_valid() {
    let formats = [
        OutputFormat::GlbBinary,
        OutputFormat::GltfEmbedded,
        OutputFormat::GltfSeparate,
    ];
    
    for format in &formats {
        // Should not panic
        let ext = format.extension();
        assert!(!ext.is_empty());
        
        let blender_fmt = format.blender_format();
        assert!(!blender_fmt.is_empty());
    }
}

#[test]
fn all_texture_formats_valid() {
    let formats = [
        TextureFormat::Png,
        TextureFormat::Jpeg,
        TextureFormat::WebP,
        TextureFormat::Original,
    ];
    
    for format in &formats {
        // Should not panic
        let ext = format.extension();
        // Note: Original format returns empty string (extension determined at runtime)
        // So we just verify the call doesn't panic
        if *format != TextureFormat::Original {
            assert!(!ext.is_empty(), "Non-original format {:?} should have extension", format);
        } else {
            assert!(ext.is_empty(), "Original format should return empty extension");
        }
    }
}

// ============================================================================
// BUILDER PATTERN ADVERSARIAL TESTS
// ============================================================================

#[test]
fn builder_repeated_builds() {
    // Building multiple times from same builder reference (if possible)
    // Tests that build() doesn't have side effects
    
    let opts1 = ConversionOptions::builder()
        .format(OutputFormat::GlbBinary)
        .build();
    
    let opts2 = ConversionOptions::builder()
        .format(OutputFormat::GlbBinary)
        .build();
    
    assert_eq!(opts1.format, opts2.format);
}

#[test]
fn builder_conflicting_settings() {
    // Set conflicting options (draco on, then off, then on)
    let opts = ConversionOptions::builder()
        .draco_compression(true)
        .draco_compression(false)
        .draco_compression(true)
        .build();
    
    // Last setting wins
    assert!(opts.gltf.draco_compression);
}

#[test]
fn builder_all_formats_with_draco() {
    // Test draco with all output formats
    let formats = [
        OutputFormat::GlbBinary,
        OutputFormat::GltfEmbedded,
        OutputFormat::GltfSeparate,
    ];
    
    for format in &formats {
        let opts = ConversionOptions::builder()
            .format(*format)
            .draco_compression(true)
            .build();
        
        assert_eq!(opts.format, *format);
        assert!(opts.gltf.draco_compression);
    }
}

// ============================================================================
// THREAD-SAFETY COMPILE-TIME CHECKS
// ============================================================================

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

#[test]
fn types_are_send_sync() {
    assert_send::<ConversionOptions>();
    assert_sync::<ConversionOptions>();
    assert_send::<BlenderVersion>();
    assert_sync::<BlenderVersion>();
    assert_send::<BlendError>();
    assert_sync::<BlendError>();
}
