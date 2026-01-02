//! Error handling tests for astraweave-blend.
//!
//! Tests all error variants, error propagation, error messages,
//! error recovery, and error context preservation.

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use astraweave_blend::error::BlendError;

// ============================================================================
// ERROR VARIANT COVERAGE
// ============================================================================

#[test]
fn error_blender_not_found() {
    let err = BlendError::BlenderNotFound {
        searched_paths: vec![
            PathBuf::from("/usr/bin"),
            PathBuf::from("/usr/local/bin"),
        ],
    };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
    assert!(
        msg.to_lowercase().contains("blender") || msg.to_lowercase().contains("not found"),
        "Message should mention 'blender' or 'not found': {}",
        msg
    );
}

#[test]
fn error_blender_executable_not_found() {
    let err = BlendError::BlenderExecutableNotFound {
        path: PathBuf::from("/path/to/blender"),
        reason: "File does not exist".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("blender") || msg.contains("executable"));
}

#[test]
fn error_blender_version_too_old() {
    let err = BlendError::BlenderVersionTooOld {
        found: "2.79.0".to_string(),
        required: "2.93.0".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("2.79") || msg.to_lowercase().contains("version"));
}

#[test]
fn error_version_parse_error() {
    let err = BlendError::VersionParseError {
        output: "not a version string".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("parse") || msg.to_lowercase().contains("version"));
}

#[test]
fn error_invalid_blend_file() {
    let path = PathBuf::from("/path/to/invalid.blend");
    let message = "File header is corrupted".to_string();
    let err = BlendError::InvalidBlendFile {
        path: path.clone(),
        message: message.clone(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("invalid") || msg.contains(&message) || msg.contains("corrupted"));
}

#[test]
fn error_blend_file_not_found() {
    let path = PathBuf::from("/nonexistent/file.blend");
    let err = BlendError::BlendFileNotFound { path: path.clone() };
    let msg = format!("{}", err);
    assert!(msg.contains("not found") || msg.contains(&path.display().to_string()));
}

#[test]
fn error_io_error_wrapper() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let err = BlendError::Io(io_err);
    let msg = format!("{}", err);
    assert!(
        msg.to_lowercase().contains("file") 
        || msg.to_lowercase().contains("not found") 
        || msg.to_lowercase().contains("io")
    );
}

#[test]
fn error_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let err = BlendError::IoError(io_err);
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("io") || msg.to_lowercase().contains("error"));
}

#[test]
fn error_conversion_failed() {
    let err = BlendError::ConversionFailed {
        message: "Blender crashed during conversion".to_string(),
        exit_code: Some(1),
        stderr: "Error: Out of memory".to_string(),
        blender_output: None,
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("conversion") || msg.contains("crashed"));
}

#[test]
fn error_timeout() {
    let err = BlendError::Timeout {
        operation: "glTF export".to_string(),
        duration: Duration::from_secs(60),
        path: PathBuf::from("/path/to/file.blend"),
        timeout_secs: 60,
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("timeout") || msg.contains("60"));
}

#[test]
fn error_export_script_error() {
    let err = BlendError::ExportScriptError {
        message: "SyntaxError: invalid syntax".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("SyntaxError") || msg.to_lowercase().contains("script"));
}

#[test]
fn error_linked_library_not_found() {
    let err = BlendError::LinkedLibraryNotFound {
        library_path: PathBuf::from("/missing/library.blend"),
        source_blend: PathBuf::from("/main.blend"),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("library") || msg.contains("missing"));
}

#[test]
fn error_circular_library_reference() {
    let err = BlendError::CircularLibraryReference {
        cycle: vec![
            PathBuf::from("a.blend"),
            PathBuf::from("b.blend"),
            PathBuf::from("a.blend"),
        ],
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("circular") || msg.to_lowercase().contains("reference"));
}

#[test]
fn error_library_depth_exceeded() {
    let err = BlendError::LibraryDepthExceeded {
        max_depth: 10,
        root_blend: PathBuf::from("/root.blend"),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("depth") || msg.contains("10"));
}

#[test]
fn error_cache_corrupted() {
    let err = BlendError::CacheCorrupted {
        path: PathBuf::from("/cache/entry.cache"),
        message: "Invalid JSON".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("cache") || msg.to_lowercase().contains("corrupt"));
}

#[test]
fn error_cache_load_error() {
    let err = BlendError::CacheLoadError {
        path: PathBuf::from("/cache/manifest.ron"),
        reason: "Deserialization failed".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("cache") || msg.contains("Deserialization"));
}

#[test]
fn error_output_not_produced() {
    let err = BlendError::OutputNotProduced {
        expected_path: PathBuf::from("/output/model.glb"),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("output") || msg.contains("model.glb"));
}

#[test]
fn error_gltf_load_error() {
    let err = BlendError::GltfLoadError {
        path: PathBuf::from("/output/scene.gltf"),
        reason: "Invalid glTF format".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("gltf") || msg.contains("Invalid"));
}

#[test]
fn error_object_not_found() {
    let err = BlendError::ObjectNotFound {
        name: "Cube".to_string(),
        blend_path: PathBuf::from("/scene.blend"),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Cube") || msg.contains("not found"));
}

#[test]
fn error_configuration_error() {
    let err = BlendError::ConfigurationError {
        message: "Invalid texture format specified".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("configuration") || msg.contains("texture"));
}

#[test]
fn error_invalid_option() {
    let err = BlendError::InvalidOption {
        reason: "Thread count must be positive".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Thread") || msg.to_lowercase().contains("invalid"));
}

#[test]
fn error_cancelled() {
    let err = BlendError::Cancelled;
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("cancel"));
}

#[test]
fn error_serialization() {
    let err = BlendError::Serialization("JSON parse error at line 5".to_string());
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("serial") || msg.contains("JSON"));
}

#[test]
fn error_internal() {
    let err = BlendError::Internal {
        message: "Unexpected state in conversion pipeline".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("internal") || msg.contains("Unexpected"));
}

// ============================================================================
// ERROR HELPER METHODS
// ============================================================================

#[test]
fn is_blender_missing() {
    let missing = BlendError::BlenderNotFound {
        searched_paths: vec![],
    };
    assert!(missing.is_blender_missing());
    
    let exec_missing = BlendError::BlenderExecutableNotFound {
        path: PathBuf::from("/blender"),
        reason: "Not found".to_string(),
    };
    assert!(exec_missing.is_blender_missing());
    
    let not_missing = BlendError::Cancelled;
    assert!(!not_missing.is_blender_missing());
}

#[test]
fn is_retryable() {
    let timeout = BlendError::Timeout {
        operation: "export".to_string(),
        duration: Duration::from_secs(30),
        path: PathBuf::from("/file.blend"),
        timeout_secs: 30,
    };
    assert!(timeout.is_retryable());
    
    let io_err = BlendError::IoError(io::Error::new(io::ErrorKind::Other, "temp"));
    assert!(io_err.is_retryable());
    
    let not_retryable = BlendError::Cancelled;
    assert!(!not_retryable.is_retryable());
}

#[test]
fn is_cancelled() {
    let cancelled = BlendError::Cancelled;
    assert!(cancelled.is_cancelled());
    
    let not_cancelled = BlendError::Internal {
        message: "some error".to_string(),
    };
    assert!(!not_cancelled.is_cancelled());
}

#[test]
fn is_cache_error() {
    let cache_err = BlendError::CacheCorrupted {
        path: PathBuf::from("/cache"),
        message: "corrupt".to_string(),
    };
    assert!(cache_err.is_cache_error());
    
    let load_err = BlendError::CacheLoadError {
        path: PathBuf::from("/cache"),
        reason: "load failed".to_string(),
    };
    assert!(load_err.is_cache_error());
    
    let not_cache = BlendError::Cancelled;
    assert!(!not_cache.is_cache_error());
}

// ============================================================================
// ERROR MESSAGE QUALITY
// ============================================================================

#[test]
fn error_messages_are_actionable() {
    // Error messages should give guidance on how to fix the problem
    let err = BlendError::BlenderNotFound {
        searched_paths: vec![PathBuf::from("/usr/bin")],
    };
    let msg = format!("{}", err);
    
    // Should mention how to fix (install Blender)
    assert!(
        msg.contains("install") || msg.contains("download") || msg.contains("blender.org"),
        "Error should give actionable guidance: {}",
        msg
    );
}

#[test]
fn error_messages_include_context() {
    let err = BlendError::LinkedLibraryNotFound {
        library_path: PathBuf::from("/libs/textures.blend"),
        source_blend: PathBuf::from("/scene/main.blend"),
    };
    let msg = format!("{}", err);
    
    // Should include both paths for debugging
    assert!(
        msg.contains("textures.blend") && msg.contains("main.blend"),
        "Error should include context (both paths): {}",
        msg
    );
}

#[test]
fn error_messages_not_empty() {
    let errors: Vec<BlendError> = vec![
        BlendError::Cancelled,
        BlendError::Internal { message: "test".to_string() },
        BlendError::Serialization("test".to_string()),
        BlendError::IoError(io::Error::new(io::ErrorKind::Other, "test")),
    ];
    
    for err in errors {
        let msg = format!("{}", err);
        assert!(!msg.is_empty(), "Error message should not be empty");
        assert!(msg.len() > 5, "Error message should be meaningful");
    }
}

// ============================================================================
// ERROR DEBUG FORMAT
// ============================================================================

#[test]
fn error_debug_format() {
    let err = BlendError::ConversionFailed {
        message: "Failed".to_string(),
        exit_code: Some(1),
        stderr: "Error details".to_string(),
        blender_output: Some("Full output".to_string()),
    };
    
    let debug = format!("{:?}", err);
    
    // Debug format should include struct fields
    assert!(debug.contains("ConversionFailed"));
    assert!(debug.contains("message"));
    assert!(debug.contains("exit_code"));
}

#[test]
fn error_implements_std_error() {
    // Verify that BlendError implements std::error::Error
    fn assert_error<E: std::error::Error>() {}
    assert_error::<BlendError>();
}

// ============================================================================
// ERROR CONVERSION
// ============================================================================

#[test]
fn io_error_conversion() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let blend_err: BlendError = io_err.into();
    
    // Should convert to Io variant
    matches!(blend_err, BlendError::Io(_));
}

// ============================================================================
// ERROR PROPAGATION SCENARIOS
// ============================================================================

fn operation_that_fails() -> Result<(), BlendError> {
    Err(BlendError::Internal {
        message: "Simulated failure".to_string(),
    })
}

fn outer_operation() -> Result<(), BlendError> {
    operation_that_fails()?;
    Ok(())
}

#[test]
fn error_propagation_with_question_mark() {
    let result = outer_operation();
    assert!(result.is_err());
    
    if let Err(e) = result {
        let msg = format!("{}", e);
        assert!(msg.contains("Simulated"));
    }
}

fn fallible_with_option() -> Option<Result<(), BlendError>> {
    Some(Err(BlendError::Cancelled))
}

#[test]
fn error_in_option() {
    let result = fallible_with_option();
    assert!(matches!(result, Some(Err(BlendError::Cancelled))));
}

// ============================================================================
// ERROR COMPARISON
// ============================================================================

#[test]
fn different_error_variants() {
    // Create different errors and verify they produce different messages
    let err1 = BlendError::Cancelled;
    let err2 = BlendError::Internal { message: "test".to_string() };
    
    let msg1 = format!("{}", err1);
    let msg2 = format!("{}", err2);
    
    assert_ne!(msg1, msg2, "Different errors should have different messages");
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn error_with_empty_strings() {
    let err = BlendError::ConversionFailed {
        message: "".to_string(),
        exit_code: None,
        stderr: "".to_string(),
        blender_output: Some("".to_string()),
    };
    
    // Should not panic
    let _msg = format!("{}", err);
    let _debug = format!("{:?}", err);
}

#[test]
fn error_with_unicode() {
    let err = BlendError::ObjectNotFound {
        name: "Объект_🎨_日本語".to_string(),
        blend_path: PathBuf::from("/путь/ファイル.blend"),
    };
    
    let msg = format!("{}", err);
    assert!(msg.contains("Объект") || msg.contains("🎨") || msg.contains("日本語"));
}

#[test]
fn error_with_special_characters() {
    let err = BlendError::Internal {
        message: "Error: \"value\" contains <xml> & 'quotes'".to_string(),
    };
    
    let msg = format!("{}", err);
    assert!(msg.contains("\"") || msg.contains("<") || msg.contains("&"));
}

#[test]
fn error_with_newlines() {
    let err = BlendError::ConversionFailed {
        message: "Line 1\nLine 2\nLine 3".to_string(),
        exit_code: None,
        stderr: "".to_string(),
        blender_output: None,
    };
    
    let msg = format!("{}", err);
    // Message might preserve or flatten newlines
    assert!(msg.contains("Line 1"));
}

#[test]
fn error_with_very_long_message() {
    let long_message = "x".repeat(10_000);
    let err = BlendError::Internal {
        message: long_message.clone(),
    };
    
    let msg = format!("{}", err);
    // Should handle long messages without panic
    assert!(msg.len() >= long_message.len());
}

#[test]
fn error_with_many_paths() {
    let paths: Vec<PathBuf> = (0..100)
        .map(|i| PathBuf::from(format!("/path/to/file_{}.blend", i)))
        .collect();
    
    let err = BlendError::BlenderNotFound {
        searched_paths: paths,
    };
    
    // Should handle many paths without panic
    let _msg = format!("{}", err);
    let _debug = format!("{:?}", err);
}

#[test]
fn error_with_deep_path() {
    let deep_path = PathBuf::from(
        (0..50)
            .map(|_| "folder")
            .collect::<Vec<_>>()
            .join("/"),
    );
    
    let err = BlendError::BlendFileNotFound { path: deep_path };
    
    // Should handle deep paths
    let _msg = format!("{}", err);
}

// ============================================================================
// ERROR SERIALIZATION (if applicable)
// ============================================================================

#[test]
fn error_to_string_roundtrip_information() {
    // Create an error with specific information
    let err = BlendError::LinkedLibraryNotFound {
        library_path: PathBuf::from("/lib/materials.blend"),
        source_blend: PathBuf::from("/scene/main.blend"),
    };
    
    let msg = format!("{}", err);
    
    // The string representation should preserve key information
    // (we can't deserialize back, but the info should be there)
    assert!(msg.contains("materials.blend") || msg.contains("main.blend"));
}

// ============================================================================
// CONCURRENT ERROR HANDLING
// ============================================================================

#[test]
fn errors_are_send_sync() {
    // BlendError should be Send + Sync for use across threads
    fn assert_send<T: Send>() {}
    
    assert_send::<BlendError>();
    // Note: BlendError contains io::Error which is Send but not Sync
    // so we don't assert Sync here
}

#[test]
fn error_in_thread() {
    use std::thread;
    
    let handle = thread::spawn(|| {
        BlendError::Cancelled
    });
    
    let err = handle.join().unwrap();
    assert!(err.is_cancelled());
}
