//! Comprehensive error types for the Blender import system.
//!
//! This module provides granular error types using `thiserror` for precise error
//! handling and meaningful error messages throughout the import pipeline.

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Result type alias for blend operations.
pub type BlendResult<T> = Result<T, BlendError>;

/// Comprehensive error types for Blender import operations.
///
/// Each variant provides context-specific information to help diagnose
/// and resolve issues during the import process.
#[derive(Error, Debug)]
pub enum BlendError {
    // ========================================================================
    // Blender Discovery Errors
    // ========================================================================

    /// Blender executable was not found on the system.
    #[error("Blender not found. Searched paths: {searched_paths:?}. Please install Blender 2.93+ from https://www.blender.org/download/")]
    BlenderNotFound {
        /// Paths that were searched for Blender.
        searched_paths: Vec<PathBuf>,
    },

    /// Blender executable at path doesn't exist or isn't accessible.
    #[error("Blender executable not found at {path}: {reason}")]
    BlenderExecutableNotFound {
        /// Path to the missing executable.
        path: PathBuf,
        /// Reason why it wasn't found.
        reason: String,
    },

    /// Blender execution failed (couldn't spawn or run process).
    #[error("Failed to execute Blender at {path}: {reason}")]
    BlenderExecutionFailed {
        /// Path to the Blender executable.
        path: PathBuf,
        /// Reason for the failure.
        reason: String,
    },

    /// Blender was found but the version is too old.
    #[error(
        "Blender version {found} is too old. Minimum required: {required}. \
         Please update Blender from https://www.blender.org/download/"
    )]
    BlenderVersionTooOld {
        /// The version that was found.
        found: String,
        /// The minimum required version.
        required: String,
    },

    /// Failed to parse the Blender version string.
    #[error("Failed to parse Blender version from output: {output}")]
    VersionParseError {
        /// The raw output that couldn't be parsed.
        output: String,
    },

    /// The discovered Blender executable is not actually Blender.
    #[error("Executable at {path} is not Blender: {reason}")]
    NotBlenderExecutable {
        /// Path to the suspicious executable.
        path: PathBuf,
        /// Reason why it's not considered Blender.
        reason: String,
    },

    /// User-configured Blender path does not exist.
    #[error("User-configured Blender path does not exist: {path}")]
    ConfiguredPathNotFound {
        /// The configured path that doesn't exist.
        path: PathBuf,
    },

    /// User-configured Blender path is not executable.
    #[error("User-configured Blender path is not executable: {path}")]
    ConfiguredPathNotExecutable {
        /// The configured path that isn't executable.
        path: PathBuf,
    },

    // ========================================================================
    // File and Path Errors
    // ========================================================================

    /// The source .blend file does not exist.
    #[error("Blend file not found: {path}")]
    BlendFileNotFound {
        /// Path to the missing file.
        path: PathBuf,
    },

    /// The file is not a valid .blend file (wrong magic bytes or extension).
    #[error("File is not a valid Blender file: {path}. {message}")]
    InvalidBlendFile {
        /// Path to the invalid file.
        path: PathBuf,
        /// Additional message about the problem.
        message: String,
    },

    /// Failed to read a file.
    #[error("Failed to read file {path}: {message}")]
    FileReadError {
        /// Path to the file that couldn't be read.
        path: PathBuf,
        /// Description of what went wrong.
        message: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to write output file.
    #[error("Failed to write output file {path}: {message}")]
    FileWriteError {
        /// Path to the file that couldn't be written.
        path: PathBuf,
        /// Description of what went wrong.
        message: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Generic I/O error wrapper.
    #[error("I/O error: {0}")]
    IoError(std::io::Error),

    // ========================================================================
    // Cache Errors
    // ========================================================================

    /// Failed to create cache directory.
    #[error("Failed to create cache directory {path}: {message}")]
    CacheDirectoryError {
        /// Path to the cache directory.
        path: PathBuf,
        /// Description of what went wrong.
        message: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to write to cache.
    #[error("Failed to write cache file {path}: {message}")]
    CacheWriteError {
        /// Path to the cache file.
        path: PathBuf,
        /// Description of what went wrong.
        message: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Cached file is corrupted or invalid.
    #[error("Cache corrupted at {path}: {message}")]
    CacheCorrupted {
        /// Path to the corrupted cache entry.
        path: PathBuf,
        /// Description of the corruption.
        message: String,
    },

    /// Failed to load cache manifest.
    #[error("Failed to load cache manifest from {path}: {reason}")]
    CacheLoadError {
        /// Path to the manifest.
        path: PathBuf,
        /// Description of what went wrong.
        reason: String,
    },

    /// Failed to save cache manifest.
    #[error("Failed to save cache manifest to {path}: {source}")]
    CacheSaveError {
        /// Path to the manifest.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to compute file hash.
    #[error("Failed to compute hash for {path}: {source}")]
    HashComputeError {
        /// Path to the file being hashed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    // ========================================================================
    // Conversion Process Errors
    // ========================================================================

    /// Failed to spawn Blender subprocess.
    #[error("Failed to start Blender process: {source}")]
    ProcessSpawnError {
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Blender process exited with non-zero status or produced errors.
    #[error("Blender conversion failed: {message}")]
    ConversionFailed {
        /// Human-readable error message.
        message: String,
        /// The exit code, if available.
        exit_code: Option<i32>,
        /// Captured stderr output from Blender.
        stderr: String,
        /// Full Blender output for debugging.
        blender_output: Option<String>,
    },

    /// Conversion was cancelled by user.
    #[error("Conversion cancelled by user")]
    Cancelled,

    /// Conversion timed out.
    #[error("Operation '{operation}' timed out after {duration:?} for {path}")]
    Timeout {
        /// Description of the operation that timed out.
        operation: String,
        /// How long we waited.
        duration: Duration,
        /// Path to the file being processed.
        path: PathBuf,
        /// Timeout in seconds (for simpler display).
        timeout_secs: u64,
    },

    /// Blender reported an error in the export script.
    #[error("Blender export script error: {message}")]
    ExportScriptError {
        /// The error message from the script.
        message: String,
    },

    /// The converted file was not produced.
    #[error("Blender did not produce output file: {expected_path}")]
    OutputNotProduced {
        /// The expected output path.
        expected_path: PathBuf,
    },

    // ========================================================================
    // Linked Library Errors
    // ========================================================================

    /// A linked library file is missing.
    #[error("Linked library not found: {library_path} (referenced from {source_blend})")]
    LinkedLibraryNotFound {
        /// Path to the missing library.
        library_path: PathBuf,
        /// The .blend file that references this library.
        source_blend: PathBuf,
    },

    /// Circular reference detected in linked libraries.
    #[error("Circular library reference detected: {cycle:?}")]
    CircularLibraryReference {
        /// The cycle of files forming the circular reference.
        cycle: Vec<PathBuf>,
    },

    /// Too many levels of library nesting.
    #[error("Library nesting depth exceeded maximum ({max_depth}) starting from {root_blend}")]
    LibraryDepthExceeded {
        /// Maximum allowed depth.
        max_depth: usize,
        /// The root file that started the chain.
        root_blend: PathBuf,
    },

    // ========================================================================
    // Post-Processing Errors
    // ========================================================================

    /// Failed to load the converted glTF file.
    #[error("Failed to load converted glTF from {path}: {reason}")]
    GltfLoadError {
        /// Path to the glTF file.
        path: PathBuf,
        /// Description of what went wrong.
        reason: String,
    },

    /// Failed to unpack embedded texture.
    #[error("Failed to unpack texture '{texture_name}' from {blend_path}: {reason}")]
    TextureUnpackError {
        /// Name of the texture.
        texture_name: String,
        /// Source .blend file.
        blend_path: PathBuf,
        /// Description of what went wrong.
        reason: String,
    },

    // ========================================================================
    // Configuration Errors
    // ========================================================================

    /// Configuration error (invalid settings, missing requirements).
    #[error("Configuration error: {message}")]
    ConfigurationError {
        /// Description of the configuration issue.
        message: String,
    },

    /// Invalid import options provided.
    #[error("Invalid import option: {reason}")]
    InvalidOption {
        /// Description of the invalid option.
        reason: String,
    },

    /// The specified collection/object to export doesn't exist.
    #[error("Object or collection '{name}' not found in {blend_path}")]
    ObjectNotFound {
        /// Name of the missing object/collection.
        name: String,
        /// The .blend file being imported.
        blend_path: PathBuf,
    },

    // ========================================================================
    // Generic/Wrapper Errors
    // ========================================================================

    /// Generic I/O error (from std::io::Error conversion).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// An unexpected internal error occurred.
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error.
        message: String,
    },
}

impl BlendError {
    /// Returns true if this error indicates a missing Blender installation.
    pub fn is_blender_missing(&self) -> bool {
        matches!(
            self,
            BlendError::BlenderNotFound { .. } | BlendError::BlenderExecutableNotFound { .. }
        )
    }

    /// Returns true if this error is recoverable by retrying.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            BlendError::Timeout { .. }
                | BlendError::ProcessSpawnError { .. }
                | BlendError::FileWriteError { .. }
                | BlendError::CacheWriteError { .. }
                | BlendError::IoError(_)
        )
    }

    /// Returns true if this error was caused by user cancellation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, BlendError::Cancelled)
    }

    /// Returns true if this error indicates a cache issue.
    pub fn is_cache_error(&self) -> bool {
        matches!(
            self,
            BlendError::CacheLoadError { .. }
                | BlendError::CacheSaveError { .. }
                | BlendError::CacheCorrupted { .. }
                | BlendError::CacheDirectoryError { .. }
                | BlendError::CacheWriteError { .. }
        )
    }

    /// Returns true if this is a configuration or setup problem.
    pub fn is_configuration_error(&self) -> bool {
        matches!(
            self,
            BlendError::ConfigurationError { .. }
                | BlendError::InvalidOption { .. }
                | BlendError::ConfiguredPathNotFound { .. }
                | BlendError::ConfiguredPathNotExecutable { .. }
        )
    }

    /// Creates a user-friendly suggestion for resolving this error.
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            BlendError::BlenderNotFound { .. } | BlendError::BlenderExecutableNotFound { .. } => {
                Some("Install Blender from https://www.blender.org/download/ and ensure it's in your PATH, or configure the path in Editor Settings.")
            }
            BlendError::BlenderVersionTooOld { .. } => {
                Some("Update Blender to version 2.93 or later for modern glTF export support.")
            }
            BlendError::Timeout { .. } => {
                Some("Try increasing the timeout in import options, or simplify the .blend file by reducing geometry/textures.")
            }
            BlendError::CacheCorrupted { .. } | BlendError::CacheWriteError { .. } => {
                Some("Clear the cache directory at .astraweave/blend_cache/ and re-import.")
            }
            BlendError::LinkedLibraryNotFound { .. } => {
                Some("Ensure all linked .blend files are accessible from the same relative paths, or use 'Make Local' in Blender to embed the data.")
            }
            BlendError::CircularLibraryReference { .. } => {
                Some("Remove circular library links in Blender before importing. Use 'Make Local' to break the cycle.")
            }
            BlendError::InvalidBlendFile { .. } => {
                Some("Ensure the file is a valid Blender file (.blend extension) and is not corrupted.")
            }
            BlendError::BlenderExecutionFailed { .. } => {
                Some("Check that Blender is properly installed and not blocked by antivirus software.")
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BlendError::BlenderNotFound {
            searched_paths: vec![PathBuf::from("/usr/bin"), PathBuf::from("/opt/blender")],
        };
        let msg = err.to_string();
        assert!(msg.contains("Blender not found"));
        assert!(msg.contains("blender.org"));
    }

    #[test]
    fn test_error_categorization() {
        assert!(BlendError::BlenderNotFound { searched_paths: vec![] }.is_blender_missing());
        assert!(!BlendError::Cancelled.is_blender_missing());

        assert!(BlendError::Cancelled.is_cancelled());
        assert!(!BlendError::BlenderNotFound { searched_paths: vec![] }.is_cancelled());

        let timeout = BlendError::Timeout {
            operation: "conversion".to_string(),
            duration: Duration::from_secs(120),
            path: PathBuf::from("test.blend"),
            timeout_secs: 120,
        };
        assert!(timeout.is_retryable());
    }

    #[test]
    fn test_error_suggestions() {
        assert!(BlendError::BlenderNotFound { searched_paths: vec![] }
            .suggestion()
            .is_some());

        let version_err = BlendError::BlenderVersionTooOld {
            found: "2.80".to_string(),
            required: "2.93".to_string(),
        };
        assert!(version_err.suggestion().is_some());

        let config_err = BlendError::ConfigurationError {
            message: "test".to_string(),
        };
        assert!(config_err.is_configuration_error());
    }

    #[test]
    fn test_cache_error_detection() {
        let cache_err = BlendError::CacheCorrupted {
            path: PathBuf::from("/cache/test"),
            message: "invalid checksum".to_string(),
        };
        assert!(cache_err.is_cache_error());
        assert!(cache_err.suggestion().is_some());
    }
}
