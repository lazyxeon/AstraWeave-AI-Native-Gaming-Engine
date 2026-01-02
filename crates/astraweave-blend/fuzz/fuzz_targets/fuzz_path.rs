#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use std::path::PathBuf;

/// Fuzz target for path handling security
/// Tests path manipulation, normalization, and security checks
#[derive(Debug, Arbitrary)]
struct FuzzPathInput {
    /// Raw path string
    path: String,
    
    /// Components for building paths
    components: Vec<String>,
    
    /// Base directory (for relative path resolution)
    base_dir: String,
    
    /// Extension to append
    extension: String,
}

fuzz_target!(|input: FuzzPathInput| {
    // ========================================================================
    // BASIC PATH OPERATIONS
    // ========================================================================
    
    // Create PathBuf from string (should not panic)
    let path = PathBuf::from(&input.path);
    
    // Basic operations
    let _ = path.exists();
    let _ = path.is_file();
    let _ = path.is_dir();
    let _ = path.is_absolute();
    let _ = path.is_relative();
    let _ = path.file_name();
    let _ = path.file_stem();
    let _ = path.extension();
    let _ = path.parent();
    let _ = path.components().count();
    
    // Display
    let _ = path.display().to_string();
    let _ = format!("{:?}", path);
    
    // ========================================================================
    // PATH TRAVERSAL PREVENTION
    // ========================================================================
    
    // Check for path traversal attempts
    let has_parent_ref = input.path.contains("..")
        || input.path.contains("..\\")
        || input.path.contains("../");
    
    // Count parent directory references
    let parent_count = path.components()
        .filter(|c| matches!(c, std::path::Component::ParentDir))
        .count();
    
    // If there are parent refs, path should be treated with caution
    let _ = (has_parent_ref, parent_count);
    
    // ========================================================================
    // PATH NORMALIZATION
    // ========================================================================
    
    // Canonicalize (will fail if path doesn't exist, but shouldn't panic)
    let _ = path.canonicalize();
    
    // Strip prefix attempts
    let _ = path.strip_prefix("/");
    let _ = path.strip_prefix("C:\\");
    let _ = path.strip_prefix(&input.base_dir);
    
    // ========================================================================
    // PATH JOINING
    // ========================================================================
    
    // Join with base directory
    let base = PathBuf::from(&input.base_dir);
    let joined = base.join(&path);
    let _ = joined.is_absolute();
    
    // Join multiple components
    let mut multi_path = PathBuf::new();
    for component in input.components.iter().take(10) {
        multi_path.push(component);
    }
    
    let _ = multi_path.display().to_string();
    
    // ========================================================================
    // EXTENSION HANDLING
    // ========================================================================
    
    // Set extension
    let mut with_ext = path.clone();
    with_ext.set_extension(&input.extension);
    
    // The extension should be what we set (if path has a file name)
    if path.file_name().is_some() {
        let ext = with_ext.extension();
        // ext might be None if input.extension is empty
        let _ = ext;
    }
    
    // With .blend extension
    let mut blend_path = path.clone();
    blend_path.set_extension("blend");
    assert_eq!(blend_path.extension().map(|e| e.to_str()), Some(Some("blend")));
    
    // ========================================================================
    // SPECIAL PATH CHARACTERS
    // ========================================================================
    
    // Test paths with special characters
    let special_chars = [
        "\0", // Null byte
        " ",  // Space
        "\t", // Tab
        "\n", // Newline
        "\r", // Carriage return
        "?",  // Wildcard
        "*",  // Wildcard
        "|",  // Pipe
        "<",  // Redirect
        ">",  // Redirect
        "\"", // Quote
        "'",  // Single quote
        ";",  // Semicolon
        "&",  // Ampersand
        "$",  // Variable
        "`",  // Backtick
        "(",  // Paren
        ")",  // Paren
        "[",  // Bracket
        "]",  // Bracket
        "{",  // Brace
        "}",  // Brace
        "!",  // Exclamation
        "#",  // Hash
        "@",  // At
        "%",  // Percent
        "^",  // Caret
        "~",  // Tilde
    ];
    
    for special in special_chars.iter() {
        let test_path = format!("test{}file.blend", special);
        let p = PathBuf::from(&test_path);
        let _ = p.display().to_string();
        let _ = p.file_name();
    }
    
    // ========================================================================
    // UNICODE PATH HANDLING
    // ========================================================================
    
    // Valid unicode
    let unicode_paths = [
        "文件/测试.blend",
        "ファイル/テスト.blend",
        "файл/тест.blend",
        "αρχείο/δοκιμή.blend",
        "archivo/prueba.blend",
        "Ñoño.blend",
        "🎮.blend",
        "émoji🎯test.blend",
    ];
    
    for unicode_path in unicode_paths.iter() {
        let p = PathBuf::from(unicode_path);
        let _ = p.file_name();
        let _ = p.extension();
        let _ = p.display().to_string();
    }
    
    // Homoglyph attack paths
    let homoglyphs = [
        "tеst.blend", // Cyrillic 'е' instead of Latin 'e'
        "tëst.blend", // Latin 'ë' with diaeresis
        "ﬁle.blend",  // fi ligature
        "ﬂle.blend",  // fl ligature
    ];
    
    for homo_path in homoglyphs.iter() {
        let p = PathBuf::from(homo_path);
        let _ = p.display().to_string();
    }
    
    // ========================================================================
    // PATH LENGTH LIMITS
    // ========================================================================
    
    // Very long path components
    if input.path.len() < 10000 {
        let long_component = "a".repeat(input.path.len().min(255));
        let long_path = PathBuf::from(&long_component);
        let _ = long_path.display().to_string();
    }
    
    // Many path components
    let many_components: PathBuf = (0..100)
        .map(|i| format!("dir{}", i))
        .collect();
    let _ = many_components.components().count();
    
    // ========================================================================
    // WINDOWS-SPECIFIC PATHS (on Windows)
    // ========================================================================
    
    // UNC paths
    let unc_paths = [
        r"\\server\share\file.blend",
        r"\\?\C:\very\long\path.blend",
        r"\\.\COM1",
        r"\\.\NUL",
    ];
    
    for unc in unc_paths.iter() {
        let p = PathBuf::from(unc);
        let _ = p.is_absolute();
        let _ = p.display().to_string();
    }
    
    // Device paths
    let device_paths = [
        r"C:\test.blend",
        r"D:\test.blend",
        "NUL",
        "CON",
        "PRN",
        "AUX",
        "COM1",
        "LPT1",
    ];
    
    for device in device_paths.iter() {
        let p = PathBuf::from(device);
        let _ = p.display().to_string();
    }
    
    // ========================================================================
    // UNIX-SPECIFIC PATHS
    // ========================================================================
    
    let unix_paths = [
        "/dev/null",
        "/dev/zero",
        "/dev/random",
        "/proc/self/exe",
        "/etc/passwd",
        "~/.bashrc",
        "$HOME/.config",
    ];
    
    for unix_path in unix_paths.iter() {
        let p = PathBuf::from(unix_path);
        let _ = p.is_absolute();
        let _ = p.display().to_string();
    }
    
    // ========================================================================
    // PATH COMPARISON
    // ========================================================================
    
    // Same path should be equal
    let p1 = PathBuf::from(&input.path);
    let p2 = PathBuf::from(&input.path);
    assert_eq!(p1, p2);
    
    // Case sensitivity (platform-dependent)
    let lower = input.path.to_lowercase();
    let upper = input.path.to_uppercase();
    let p_lower = PathBuf::from(&lower);
    let p_upper = PathBuf::from(&upper);
    
    // Just compare, don't assert (platform-dependent)
    let _ = p_lower == p_upper;
    
    // ========================================================================
    // PATH ITERATION
    // ========================================================================
    
    // Iterate components
    for component in path.components() {
        match component {
            std::path::Component::Prefix(_) => {}
            std::path::Component::RootDir => {}
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {}
            std::path::Component::Normal(name) => {
                let _ = name.to_string_lossy();
            }
        }
    }
    
    // Ancestors
    for ancestor in path.ancestors() {
        let _ = ancestor.display().to_string();
    }
    
    // ========================================================================
    // RELATIVE PATH RESOLUTION
    // ========================================================================
    
    // Make relative to base
    if let Ok(relative) = path.strip_prefix(&input.base_dir) {
        let _ = relative.display().to_string();
    }
    
    // Check if path starts with base
    let starts_with = path.starts_with(&input.base_dir);
    let ends_with = path.ends_with(&input.extension);
    let _ = (starts_with, ends_with);
});
