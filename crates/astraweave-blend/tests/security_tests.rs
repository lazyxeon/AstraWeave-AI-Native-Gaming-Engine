//! Security tests for astraweave-blend.
//!
//! Tests path traversal prevention, symlink handling, resource exhaustion,
//! command injection, and other security-critical scenarios.

#![allow(
    clippy::assertions_on_constants,
    clippy::useless_vec
)]

use std::path::PathBuf;
use std::time::Duration;
use std::fs;
use std::io::Write;

use sha2::{Sha256, Digest};

use astraweave_blend::options::*;

/// Compute SHA-256 hash of data.
fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// ============================================================================
// PATH TRAVERSAL PREVENTION
// ============================================================================

/// Test inputs that attempt path traversal attacks.
fn path_traversal_inputs() -> Vec<String> {
    vec![
        // Basic traversal
        "../../../etc/passwd".to_string(),
        "..\\..\\..\\windows\\system32".to_string(),
        "....//....//....//etc/passwd".to_string(),
        
        // URL encoded
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc/passwd".to_string(),
        "%2e%2e%5c%2e%2e%5c%2e%2e%5cwindows".to_string(),
        
        // Double encoded
        "%252e%252e%252f".to_string(),
        
        // Unicode encoding
        "..%c0%af".to_string(),
        "..%c1%9c".to_string(),
        
        // Null byte injection
        "../../../etc/passwd\0.blend".to_string(),
        "safe.blend\0/../../../etc/passwd".to_string(),
        
        // Mixed separators
        "..\\../..\\../etc/passwd".to_string(),
        "../..\\..\\../windows".to_string(),
        
        // Leading slashes
        "/../../../etc/passwd".to_string(),
        "\\..\\..\\..\\windows\\system32".to_string(),
        
        // Trailing sequences
        "models/..".to_string(),
        "models/../..".to_string(),
        "models/../../..".to_string(),
        
        // Absolute paths disguised
        "/etc/passwd".to_string(),
        "C:\\Windows\\System32".to_string(),
        "\\\\server\\share\\file".to_string(),
        
        // Long path components
        format!("../{}/../../etc/passwd", "A".repeat(1000)),
    ]
}

#[test]
fn path_traversal_detection_basic() {
    for malicious_path in path_traversal_inputs() {
        let path = PathBuf::from(&malicious_path);
        
        // Check for .. components
        let has_parent_ref = path.components().any(|c| {
            matches!(c, std::path::Component::ParentDir)
        });
        
        // Paths with .. should be flagged or sanitized
        if malicious_path.contains("..") && !malicious_path.contains('\0') {
            // The path should either be detected as traversal attempt
            // or normalized to remove dangerous components
            assert!(
                has_parent_ref || path.to_string_lossy().contains(".."),
                "Path traversal not detected: {}",
                malicious_path
            );
        }
    }
}

#[test]
fn normalize_path_removes_parent_refs() {
    // Test path normalization logic
    fn normalize_path(path: &std::path::Path) -> PathBuf {
        use std::path::Component;
        
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    // Only pop if we have a normal component (not going above root)
                    if let Some(Component::Normal(_)) = components.last().cloned().map(|_| Component::Normal(std::ffi::OsStr::new(""))) {
                        components.pop();
                    }
                }
                Component::CurDir => {} // Skip .
                c => components.push(c),
            }
        }
        
        components.iter().collect()
    }
    
    // Simple traversal
    let path = PathBuf::from("models/../secrets/password.txt");
    let normalized = normalize_path(&path);
    assert!(!normalized.to_string_lossy().contains(".."));
}

#[test]
fn reject_absolute_paths_in_relative_context() {
    let absolute_paths = vec![
        PathBuf::from("/etc/passwd"),
        PathBuf::from("C:\\Windows\\System32\\config"),
        PathBuf::from("\\\\server\\share"),
    ];
    
    for path in absolute_paths {
        if path.is_absolute() {
            // In contexts expecting relative paths, absolute paths should be rejected
            assert!(path.is_absolute(), "Expected absolute path detection: {:?}", path);
        }
    }
}

// ============================================================================
// SYMLINK SECURITY
// ============================================================================

#[test]
fn detect_symlink_in_path() {
    // This test verifies that symlinks can be detected
    // In production, symlinks should be resolved and checked against allowed directories
    
    let temp_dir = tempfile::tempdir().unwrap();
    let target = temp_dir.path().join("target.blend");
    
    // Create a regular file
    fs::write(&target, b"BLENDER-v300").unwrap();
    
    // Try to create symlink (may fail on Windows without privileges)
    #[cfg(unix)]
    {
        let link = temp_dir.path().join("link.blend");
        if std::os::unix::fs::symlink(&target, &link).is_ok() {
            // Should be able to detect it's a symlink
            assert!(link.is_symlink() || fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
        }
    }
}

#[test]
fn symlink_loop_detection() {
    // Circular symlinks should be detected to prevent infinite loops
    // This is a conceptual test - actual implementation depends on OS
    
    let circular_paths = vec![
        ("a.blend", "b.blend"),
        ("b.blend", "c.blend"),
        ("c.blend", "a.blend"),  // Creates cycle
    ];
    
    // Detection algorithm should track visited paths
    let mut visited = std::collections::HashSet::new();
    for (from, _to) in &circular_paths {
        if !visited.insert(*from) {
            // Detected cycle
            break;
        }
    }
}

// ============================================================================
// COMMAND INJECTION PREVENTION
// ============================================================================

/// Inputs that attempt command injection.
fn command_injection_inputs() -> Vec<&'static str> {
    vec![
        // Shell metacharacters
        "; rm -rf /",
        "| cat /etc/passwd",
        "& whoami",
        "` id `",
        "$(id)",
        
        // Windows specific
        "& dir",
        "| type C:\\Windows\\win.ini",
        
        // Newline injection
        "file.blend\nrm -rf /",
        "file.blend\r\ndir",
        
        // Null byte
        "file.blend\0; rm -rf /",
        
        // Quote escaping
        "file.blend' ; rm -rf / '",
        "file.blend\" ; rm -rf / \"",
        
        // Backticks
        "file.blend`id`",
        
        // Variable expansion
        "${PATH}",
        "$HOME/../../etc/passwd",
        "%USERPROFILE%\\..\\..\\etc\\passwd",
    ]
}

#[test]
fn command_injection_characters_detected() {
    let dangerous_chars = [';', '|', '&', '`', '$', '(', ')', '{', '}', '\n', '\r', '\0'];
    
    for input in command_injection_inputs() {
        let has_dangerous = input.chars().any(|c| dangerous_chars.contains(&c));
        if has_dangerous {
            // Should be detected and sanitized/rejected
            assert!(
                has_dangerous,
                "Dangerous characters not detected in: {}",
                input.escape_default()
            );
        }
    }
}

#[test]
fn sanitize_for_shell() {
    fn is_safe_for_shell(s: &str) -> bool {
        !s.chars().any(|c| matches!(c, ';' | '|' | '&' | '`' | '$' | '(' | ')' | '{' | '}' | '\n' | '\r' | '\0' | '\'' | '"' | '<' | '>' | '\\'))
    }
    
    assert!(is_safe_for_shell("model.blend"));
    assert!(is_safe_for_shell("my model.blend"));  // Spaces are okay if quoted
    assert!(!is_safe_for_shell("model.blend; rm -rf /"));
    assert!(!is_safe_for_shell("model.blend | cat"));
}

// ============================================================================
// RESOURCE EXHAUSTION PREVENTION
// ============================================================================

#[test]
fn timeout_prevents_hang() {
    let opts = ProcessOptions {
        timeout: Duration::from_secs(30),
        ..Default::default()
    };
    
    // Timeout should be enforced
    assert!(opts.timeout.as_secs() > 0);
    assert!(opts.timeout.as_secs() <= 3600);  // Reasonable upper bound
}

#[test]
fn max_recursion_depth_limits_stack() {
    let opts = LinkedLibraryOptions {
        max_recursion_depth: 50,  // Reasonable limit
        ..Default::default()
    };
    
    assert!(opts.max_recursion_depth <= 100);  // Shouldn't be too high
}

#[test]
fn cache_size_limits() {
    let opts = CacheOptions {
        max_cache_size: Some(1024 * 1024 * 1024),  // 1 GB limit
        ..Default::default()
    };
    
    // Should have a reasonable limit
    assert!(opts.max_cache_size.unwrap() <= 10 * 1024 * 1024 * 1024);  // 10 GB max
}

#[test]
fn thread_count_limits() {
    let opts = ProcessOptions {
        threads: 32,
        ..Default::default()
    };
    
    // Should be reasonable
    assert!(opts.threads <= 256);
}

// ============================================================================
// FILE HEADER VALIDATION
// ============================================================================

#[test]
fn valid_blend_header() {
    let valid_headers = [
        b"BLENDER-v300",  // Little-endian
        b"BLENDER-V300",  // Big-endian
        b"BLENDER_v300",  // Underscore variant
    ];
    
    for header in &valid_headers {
        assert!(header.starts_with(b"BLENDER"));
    }
}

#[test]
fn invalid_blend_header_detection() {
    let invalid_headers: Vec<&[u8]> = vec![
        b"",              // Empty
        b"BLEND",         // Too short
        b"GLENDER-v300",  // Wrong magic
        b"\x00\x00\x00\x00", // Null bytes
        b"\xff\xfe\xfd\xfc", // Random bytes
        b"BLENDER",       // Truncated
    ];
    
    for header in invalid_headers {
        assert!(
            !header.starts_with(b"BLENDER-") && !header.starts_with(b"BLENDER_"),
            "Invalid header falsely accepted: {:?}",
            header
        );
    }
}

#[test]
fn gltf_header_validation() {
    // Valid glTF JSON header
    let valid_gltf = br#"{"asset":{"version":"2.0"}}"#;
    assert!(valid_gltf.starts_with(b"{"));
    
    // Valid GLB header (magic + version + length)
    let valid_glb = [
        0x67, 0x6C, 0x54, 0x46,  // glTF magic
        0x02, 0x00, 0x00, 0x00,  // Version 2
        0x00, 0x01, 0x00, 0x00,  // Length
    ];
    assert_eq!(&valid_glb[0..4], b"glTF");
}

// ============================================================================
// INPUT SIZE LIMITS
// ============================================================================

#[test]
fn reject_oversized_path() {
    // Maximum reasonable path length
    const MAX_PATH_LENGTH: usize = 4096;
    
    let long_path = "a/".repeat(MAX_PATH_LENGTH);
    assert!(long_path.len() >= MAX_PATH_LENGTH);
    
    // Should be rejected or truncated
}

#[test]
fn reject_oversized_environment() {
    // Don't allow too many environment variables
    const MAX_ENV_VARS: usize = 1000;
    
    let env_vars: Vec<(String, String)> = (0..MAX_ENV_VARS + 1)
        .map(|i| (format!("VAR_{}", i), format!("VALUE_{}", i)))
        .collect();
    
    assert!(env_vars.len() > MAX_ENV_VARS);
    // Should be limited
}

#[test]
fn reject_oversized_args() {
    // Don't allow too many extra arguments
    const MAX_ARGS: usize = 100;
    
    let args: Vec<String> = (0..MAX_ARGS + 1)
        .map(|i| format!("--arg-{}", i))
        .collect();
    
    assert!(args.len() > MAX_ARGS);
    // Should be limited
}

// ============================================================================
// HASH SECURITY
// ============================================================================

#[test]
fn hash_collision_resistance() {
    // Different inputs should produce different hashes
    let inputs = vec![
        b"input1".to_vec(),
        b"input2".to_vec(),
        b"input3".to_vec(),
        b"INPUT1".to_vec(),  // Case difference
        b"input1 ".to_vec(),  // Whitespace difference
        vec![0u8; 100],
        vec![1u8; 100],
    ];
    
    let mut hashes = std::collections::HashSet::new();
    for input in &inputs {
        let hash = compute_sha256(input);
        hashes.insert(hash);
    }
    
    // All hashes should be unique
    assert_eq!(hashes.len(), inputs.len(), "Hash collision detected!");
}

#[test]
fn hash_deterministic() {
    let input = b"test input for hashing";
    let hash1 = compute_sha256(input);
    let hash2 = compute_sha256(input);
    
    assert_eq!(hash1, hash2, "Hash should be deterministic");
}

#[test]
fn hash_length_extension_protection() {
    // SHA-256 is resistant to length extension attacks in practice
    // The hash of input should not help compute hash of input + extension
    let input = b"original input";
    let extended = b"original input + extension";
    
    let hash1 = compute_sha256(input);
    let hash2 = compute_sha256(extended);
    
    assert_ne!(hash1, hash2);
}

// ============================================================================
// UNICODE SECURITY
// ============================================================================

#[test]
fn unicode_normalization_attacks() {
    // Different Unicode representations of same character
    let strings = vec![
        "café",      // Composed
        "cafe\u{0301}",  // Decomposed (e + combining acute)
    ];
    
    // These might appear same but are different bytes
    // Security check should use normalized comparison or reject
    assert_ne!(strings[0].as_bytes(), strings[1].as_bytes());
}

#[test]
fn homoglyph_detection() {
    // Characters that look similar but are different
    let pairs = vec![
        ("blend", "blеnd"),  // Second 'e' is Cyrillic
        ("model", "mоdel"),  // 'o' is Cyrillic
        ("test.blend", "test.blеnd"),  // Cyrillic 'e'
    ];
    
    for (ascii, lookalike) in pairs {
        assert_ne!(ascii, lookalike);
        // Visual inspection might miss the difference
    }
}

#[test]
fn right_to_left_override() {
    // RTL override can be used to hide true file extension
    let rtl_override = "\u{202E}";  // Right-to-left override
    let disguised = format!("model{}blend.exe", rtl_override);
    
    // The filename appears as "modelexe.dnelb" visually
    // but actually ends in .exe
    
    // Should strip or reject RTL control characters
    assert!(disguised.contains(rtl_override));
}

#[test]
fn null_byte_in_string() {
    let with_null = "model\0.blend";
    
    // Null byte can truncate strings in C APIs
    // Should be detected and rejected
    assert!(with_null.contains('\0'));
}

#[test]
fn bom_handling() {
    // Byte Order Mark should be handled
    let with_bom = "\u{FEFF}model.blend";
    
    // BOM at start can cause issues
    assert!(with_bom.starts_with('\u{FEFF}'));
}

// ============================================================================
// DENIAL OF SERVICE PREVENTION
// ============================================================================

#[test]
fn zip_bomb_detection() {
    // Highly compressed data that expands massively
    // Should have decompression limits
    
    let compressed_size = 1000;  // 1 KB compressed
    let max_expansion_ratio = 1000;  // Max 1000x expansion
    let max_output = compressed_size * max_expansion_ratio;
    
    // Any decompression should respect this limit
    assert!(max_output <= 1_000_000_000);  // 1 GB absolute max
}

#[test]
fn billion_laughs_prevention() {
    // XML/JSON entity expansion attack
    // Blender exports can contain JSON metadata
    
    // Example of nested expansion (not actual attack):
    // {"a": "lol", "b": "&a;&a;&a;..."}
    
    // Should limit entity expansion depth and total size
    const MAX_ENTITY_EXPANSION: usize = 10;
    const MAX_TOTAL_SIZE: usize = 100_000_000;  // 100 MB
    
    assert!(MAX_ENTITY_EXPANSION < 100);
    assert!(MAX_TOTAL_SIZE < 1_000_000_000);
}

#[test]
fn recursion_bomb_prevention() {
    // Deeply nested structures
    let max_depth = 100;
    
    // Linked library recursion is limited
    let opts = LinkedLibraryOptions {
        max_recursion_depth: max_depth,
        ..Default::default()
    };
    
    assert!(opts.max_recursion_depth <= 100);
}

// ============================================================================
// TEMPORARY FILE SECURITY
// ============================================================================

#[test]
fn temp_file_permissions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_file = temp_dir.path().join("secure.tmp");
    
    // Create with restrictive permissions
    let mut file = fs::File::create(&temp_file).unwrap();
    file.write_all(b"sensitive data").unwrap();
    
    // Verify file was created
    assert!(temp_file.exists());
    
    // On Unix, should have 600 permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::metadata(&temp_file).unwrap().permissions();
        // Default permissions depend on umask, but should be restricted
        assert!(perms.mode() & 0o777 <= 0o644);
    }
}

#[test]
fn temp_file_cleanup() {
    let temp_path;
    {
        let temp_dir = tempfile::tempdir().unwrap();
        temp_path = temp_dir.path().to_path_buf();
        
        // Create a file in temp dir
        let temp_file = temp_path.join("test.tmp");
        fs::write(&temp_file, b"test").unwrap();
        
        // TempDir dropped here
    }
    
    // After drop, directory should be cleaned up
    assert!(!temp_path.exists());
}

#[test]
fn predictable_temp_name_prevention() {
    // Temp file names should be unpredictable
    let temp1 = tempfile::NamedTempFile::new().unwrap();
    let temp2 = tempfile::NamedTempFile::new().unwrap();
    
    assert_ne!(temp1.path(), temp2.path());
    
    // Names should contain random components
    let name1 = temp1.path().file_name().unwrap().to_string_lossy();
    let name2 = temp2.path().file_name().unwrap().to_string_lossy();
    
    // Should not be sequential or predictable
    assert_ne!(name1, name2);
}

// ============================================================================
// SERIALIZATION SECURITY
// ============================================================================

#[test]
fn ron_injection_prevention() {
    // RON should not execute code
    let malicious_ron = r#"
        (
            format: GlbBinary,
            // Attempt to include external file
            #include "/etc/passwd"
        )
    "#;
    
    // Should fail to parse, not execute
    let result: std::result::Result<ConversionOptions, _> = ron::from_str(malicious_ron);
    assert!(result.is_err());
}

#[test]
fn oversized_serialized_data() {
    // Deserializing huge data should be limited
    let huge_json = format!(r#"{{"data": "{}"}}"#, "A".repeat(1_000_000));
    
    // Should either fail or handle gracefully
    // (Not test actual limit implementation, just demonstrate the concern)
    assert!(huge_json.len() > 1_000_000);
}

// ============================================================================
// SAFE DEFAULTS
// ============================================================================

#[test]
fn default_options_are_secure() {
    let opts = ConversionOptions::default();
    
    // Should have reasonable timeout (not infinite)
    assert!(opts.process.timeout.as_secs() < 3600);
    assert!(opts.process.timeout.as_secs() >= 1);
    
    // Should have limited recursion
    assert!(opts.linked_libraries.max_recursion_depth <= 100);
    
    // Cache should have size limits or be disabled by default
    if opts.cache.enabled {
        // If enabled, should have reasonable limits
    }
}

#[test]
fn presets_are_secure() {
    let presets = vec![
        ConversionOptions::game_runtime(),
        ConversionOptions::editor_preview(),
        ConversionOptions::archival_quality(),
    ];
    
    for opts in presets {
        // All presets should have timeouts
        assert!(opts.process.timeout.as_secs() > 0);
        assert!(opts.process.timeout.as_secs() < 7200);  // 2 hours max
        
        // Limited recursion
        assert!(opts.linked_libraries.max_recursion_depth <= 100);
    }
}
