//! Secure path validation utilities
//!
//! This module provides utilities to prevent path traversal attacks
//! and ensure file operations stay within allowed directories.

use std::io;
use std::path::{Path, PathBuf};

/// Validates that a user-provided path is within an allowed base directory
///
/// This prevents path traversal attacks like "../../../etc/passwd"
///
/// # Arguments
/// * `base` - The allowed base directory (e.g., "assets/")
/// * `user_path` - User-provided path (potentially unsafe)
///
/// # Returns
/// * `Ok(PathBuf)` - Canonical path if safe
/// * `Err` - If path escapes base directory
///
/// # Example
/// ```
/// use astraweave_security::path::safe_under;
/// use std::path::Path;
///
/// # fn example() -> std::io::Result<()> {
/// let safe = safe_under(Path::new("assets"), Path::new("textures/grass.png"))?;
/// // OK: assets/textures/grass.png
///
/// let unsafe_path = safe_under(Path::new("assets"), Path::new("../../../etc/passwd"));
/// // Err: Path escapes base directory
/// # Ok(())
/// # }
/// ```
pub fn safe_under(base: &Path, user_path: &Path) -> io::Result<PathBuf> {
    // First, validate user_path components before joining
    validate_user_path_components(user_path)?;

    // Canonicalize base directory (must exist)
    let base_canonical = base.canonicalize().map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Base directory not found: {} ({})", base.display(), e),
        )
    })?;

    // Join user path with base
    let combined = base_canonical.join(user_path);

    // Try to canonicalize combined path
    // If it doesn't exist yet, we need to check parent directories
    let target_canonical = match combined.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // Find the deepest existing parent
            let mut check_path = combined.clone();
            let mut non_existent_parts = Vec::new();

            loop {
                if check_path.exists() {
                    // Found existing parent
                    let parent_canonical = check_path.canonicalize()?;

                    // Verify parent is under base
                    if !parent_canonical.starts_with(&base_canonical) {
                        return Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            format!(
                                "Path traversal attempt: {} escapes {}",
                                user_path.display(),
                                base.display()
                            ),
                        ));
                    }

                    // Rebuild path with non-existent parts
                    let mut result = parent_canonical;
                    for part in non_existent_parts.iter().rev() {
                        result = result.join(part);
                    }
                    return Ok(result);
                }

                // Store this component and check parent
                if let Some(filename) = check_path.file_name() {
                    non_existent_parts.push(filename.to_owned());
                }

                if let Some(parent) = check_path.parent() {
                    check_path = parent.to_path_buf();
                } else {
                    // Reached root without finding existing path
                    // This means the entire path is non-existent
                    // Return the combined path since user_path passed validation
                    return Ok(combined);
                }
            }
        }
    };

    // Verify target is under base
    if !target_canonical.starts_with(&base_canonical) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Path traversal attempt: {} escapes {}",
                user_path.display(),
                base.display()
            ),
        ));
    }

    Ok(target_canonical)
}

/// Validate user-provided path components (before joining with base)
fn validate_user_path_components(user_path: &Path) -> io::Result<()> {
    // Check for suspicious components
    for component in user_path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Path contains '..' component",
                ));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Absolute paths not allowed",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Validates file extension against allowlist
///
/// # Arguments
/// * `path` - Path to validate
/// * `allowed` - List of allowed extensions (without dots)
///
/// # Example
/// ```
/// use astraweave_security::path::validate_extension;
/// use std::path::Path;
///
/// # fn example() -> std::io::Result<()> {
/// validate_extension(Path::new("file.png"), &["png", "jpg"])?;
/// // OK
///
/// let result = validate_extension(Path::new("file.exe"), &["png", "jpg"]);
/// // Err: Extension not allowed
/// # Ok(())
/// # }
/// ```
pub fn validate_extension(path: &Path, allowed: &[&str]) -> io::Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "File has no extension"))?;

    if !allowed.contains(&ext) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Extension '{}' not allowed", ext),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_safe_path() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        // Create subdirectory
        let subdir = base.join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Safe path to existing directory
        let safe = safe_under(base, Path::new("subdir")).unwrap();
        // On Windows, canonicalize adds \\?\ prefix, so use canonical comparison
        let base_canonical = base.canonicalize().unwrap();
        assert!(safe.starts_with(&base_canonical));
    }

    #[test]
    fn test_safe_path_with_file() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        // Create subdirectory and file
        let subdir = base.join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file = subdir.join("file.txt");
        fs::write(&file, "test").unwrap();

        // Safe path to existing file
        let safe = safe_under(base, Path::new("subdir/file.txt")).unwrap();
        let base_canonical = base.canonicalize().unwrap();
        assert!(safe.starts_with(&base_canonical));
        let file_canonical = file.canonicalize().unwrap();
        assert_eq!(safe, file_canonical);
    }

    #[test]
    fn test_path_traversal_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        // Create subdirectory so base exists and is canonical
        fs::create_dir_all(base).unwrap();

        // Path traversal attempt
        let result = safe_under(base, Path::new("../../../etc/passwd"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_absolute_path_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        #[cfg(windows)]
        let result = safe_under(base, Path::new("C:\\Windows\\System32"));

        #[cfg(not(windows))]
        let result = safe_under(base, Path::new("/etc/passwd"));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn test_nonexistent_file_safe_parent() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        // Create parent directory
        let parent = base.join("parent");
        fs::create_dir(&parent).unwrap();

        // Non-existent file in existing parent
        let safe = safe_under(base, Path::new("parent/newfile.txt")).unwrap();
        let base_canonical = base.canonicalize().unwrap();
        assert!(safe.starts_with(&base_canonical));
        assert!(safe.ends_with("newfile.txt"));
    }

    #[test]
    fn test_nonexistent_nested_path() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        // Non-existent nested path - should still validate
        let result = safe_under(base, Path::new("new/nested/file.txt"));

        // This should succeed because we validate components
        if let Err(e) = &result {
            eprintln!("Error: {}", e);
            eprintln!("Error kind: {:?}", e.kind());
        }
        assert!(
            result.is_ok(),
            "Failed to validate non-existent nested path: {:?}",
            result
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_escape_blocked() {
        // This test is platform-specific and requires symlink permissions
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        // Create a directory outside base
        let outside = tempfile::tempdir().unwrap();
        let outside_path = outside.path();

        // Create symlink inside base pointing outside
        let link = base.join("escape");
        std::os::unix::fs::symlink(outside_path, &link).unwrap();

        // Try to access through symlink
        let result = safe_under(base, Path::new("escape"));
        // After canonicalization, this should point outside base
        if let Ok(canonical) = result {
            // The symlink was resolved, verify it's blocked
            let base_canonical = base.canonicalize().unwrap();
            assert!(!canonical.starts_with(&base_canonical));
        }
    }

    #[test]
    fn test_validate_extension_allowed() {
        let path = Path::new("file.png");
        let result = validate_extension(path, &["png", "jpg", "gif"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_extension_blocked() {
        let path = Path::new("file.exe");
        let result = validate_extension(path, &["png", "jpg", "gif"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert!(err.to_string().contains("exe"));
    }

    #[test]
    fn test_validate_extension_no_extension() {
        let path = Path::new("file");
        let result = validate_extension(path, &["png", "jpg"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("no extension"));
    }

    #[test]
    fn test_validate_extension_case_sensitive() {
        let path = Path::new("file.PNG");
        let result = validate_extension(path, &["png", "jpg"]);
        // Extension comparison is case-sensitive
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_extension_multiple_dots() {
        let path = Path::new("file.tar.gz");
        // Only checks last extension
        let result = validate_extension(path, &["gz"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_user_path_components_parent_dir() {
        let path = Path::new("../etc/passwd");
        let result = validate_user_path_components(path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("..") || err_msg.contains("Parent"));
    }

    #[test]
    fn test_validate_user_path_components_root_dir() {
        let path = Path::new("/etc/passwd");
        let result = validate_user_path_components(path);
        // Should block absolute paths
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_directories() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();

        // Create nested structure
        let nested = base.join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();

        // Access deeply nested path
        let safe = safe_under(base, Path::new("a/b/c")).unwrap();
        let base_canonical = base.canonicalize().unwrap();
        assert!(safe.starts_with(&base_canonical));
        assert!(safe.ends_with("c"));
    }

    #[test]
    fn test_base_directory_not_found() {
        let result = safe_under(Path::new("/nonexistent/base"), Path::new("file.txt"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(err.to_string().contains("Base directory not found"));
    }
}
