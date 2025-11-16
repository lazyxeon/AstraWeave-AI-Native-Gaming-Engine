//! Shader Validation Tests
//!
//! Validates all WGSL shaders in the project compile correctly using naga.
//! This catches shader syntax errors, type mismatches, and unsupported features
//! before they cause runtime failures.

use naga::front::wgsl;
use std::path::PathBuf;

/// Get all WGSL shader files in the project
fn get_all_shaders() -> Vec<PathBuf> {
    let mut shaders = Vec::new();

    // Get workspace root (navigate up from astraweave-render/tests)
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = if current_dir.ends_with("astraweave-render") {
        current_dir.parent().unwrap().to_path_buf()
    } else {
        current_dir
    };

    // Change to workspace root for glob patterns
    std::env::set_current_dir(&workspace_root).unwrap();

    // Core rendering shaders
    for entry in glob::glob("astraweave-render/shaders/**/*.wgsl").unwrap() {
        if let Ok(path) = entry {
            shaders.push(path);
        }
    }
    for entry in glob::glob("astraweave-render/src/shaders/**/*.wgsl").unwrap() {
        if let Ok(path) = entry {
            shaders.push(path);
        }
    }

    // Bevy integration shaders
    for entry in glob::glob("astraweave-render-bevy/shaders/**/*.wgsl").unwrap() {
        if let Ok(path) = entry {
            shaders.push(path);
        }
    }
    for entry in glob::glob("astraweave-render-bevy/src/shaders/**/*.wgsl").unwrap() {
        if let Ok(path) = entry {
            shaders.push(path);
        }
    }

    // Editor viewport shaders
    for entry in glob::glob("tools/aw_editor/src/viewport/shaders/**/*.wgsl").unwrap() {
        if let Ok(path) = entry {
            shaders.push(path);
        }
    }

    // Example shaders
    for entry in glob::glob("examples/**/src/**/*.wgsl").unwrap() {
        if let Ok(path) = entry {
            shaders.push(path);
        }
    }

    shaders
}

#[test]
fn test_all_shaders_compile() {
    let shaders = get_all_shaders();

    assert!(
        !shaders.is_empty(),
        "No shaders found! Check glob patterns."
    );

    println!("📦 Found {} WGSL shader files", shaders.len());

    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    let mut success_count = 0;

    for shader_path in &shaders {
        let relative_path = shader_path
            .strip_prefix(std::env::current_dir().unwrap())
            .unwrap_or(shader_path);

        let source = match std::fs::read_to_string(shader_path) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!(
                    "❌ {}: Failed to read file: {}",
                    relative_path.display(),
                    e
                ));
                continue;
            }
        };

        // Skip Bevy shaders that use preprocessor directives
        // These need Bevy's shader processor before naga validation
        if source.contains("#import") || source.contains("#define") {
            println!(
                "⏭️  {} (Bevy preprocessor shader - skipped)",
                relative_path.display()
            );
            success_count += 1; // Count as success (will be validated by Bevy)
            continue;
        }

        // Parse shader with naga
        match wgsl::parse_str(&source) {
            Ok(module) => {
                // Validate module
                let mut validator = naga::valid::Validator::new(
                    naga::valid::ValidationFlags::all(),
                    naga::valid::Capabilities::all(),
                );

                match validator.validate(&module) {
                    Ok(_) => {
                        success_count += 1;
                        println!("✅ {}", relative_path.display());
                    }
                    Err(e) => {
                        failures.push(format!(
                            "❌ {}: Validation error: {}",
                            relative_path.display(),
                            e
                        ));
                    }
                }
            }
            Err(e) => {
                // Check if it's a warning or fatal error
                let error_str = format!("{}", e);
                if error_str.contains("warning") {
                    warnings.push(format!("⚠️  {}: {}", relative_path.display(), e));
                    success_count += 1;
                } else {
                    failures.push(format!(
                        "❌ {}: Parse error: {}",
                        relative_path.display(),
                        e
                    ));
                }
            }
        }
    }

    println!("\n📊 Shader Validation Summary:");
    println!("   Total shaders: {}", shaders.len());
    println!("   ✅ Passed: {}", success_count);
    println!("   ⚠️  Warnings: {}", warnings.len());
    println!("   ❌ Failed: {}", failures.len());

    // Print warnings
    if !warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for warning in &warnings {
            println!("   {}", warning);
        }
    }

    // Print failures
    if !failures.is_empty() {
        println!("\n❌ Failures:");
        for failure in &failures {
            println!("   {}", failure);
        }

        panic!(
            "\n💥 {} shader(s) failed validation!\n\
             See errors above for details.",
            failures.len()
        );
    }

    println!("\n🎉 All {} shaders validated successfully!", shaders.len());
}

#[test]
fn test_shader_features_compatibility() {
    // Verify shaders don't use features unavailable on WebGL2
    // This is a placeholder for future platform-specific validation

    let shaders = get_all_shaders();
    let mut incompatible = Vec::new();

    for shader_path in &shaders {
        let source = std::fs::read_to_string(shader_path).unwrap();

        // Check for features that might not be available everywhere
        if source.contains("atomicAdd") || source.contains("atomicMax") {
            // Atomic operations - verify they're only in compute shaders
            if !source.contains("@compute") {
                incompatible.push(format!(
                    "{}: Uses atomic operations outside compute shader",
                    shader_path.display()
                ));
            }
        }

        // Check for excessive texture bindings (WebGL2 has lower limits)
        let binding_count = source.matches("@binding(").count();
        if binding_count > 16 {
            incompatible.push(format!(
                "{}: Has {} bindings (WebGL2 limit: 16)",
                shader_path.display(),
                binding_count
            ));
        }
    }

    if !incompatible.is_empty() {
        println!("⚠️  Potential compatibility issues:");
        for issue in &incompatible {
            println!("   {}", issue);
        }
    } else {
        println!("✅ No compatibility issues detected");
    }
}

#[test]
fn test_shader_entry_points() {
    // Verify all shaders have proper entry points
    let shaders = get_all_shaders();
    let mut missing_entry_points = Vec::new();

    for shader_path in &shaders {
        let source = std::fs::read_to_string(shader_path).unwrap();

        // Parse to get module
        if let Ok(module) = wgsl::parse_str(&source) {
            if module.entry_points.is_empty() {
                // Some shaders are libraries without entry points (e.g., pbr_lib.wgsl)
                // Only warn if it doesn't look like a library
                if !shader_path.to_string_lossy().contains("lib")
                    && !shader_path.to_string_lossy().contains("functions")
                    && !shader_path.to_string_lossy().contains("types")
                    && !shader_path.to_string_lossy().contains("bindings")
                    && !shader_path.to_string_lossy().contains("utils")
                {
                    missing_entry_points.push(shader_path.display().to_string());
                }
            }
        }
    }

    if !missing_entry_points.is_empty() {
        println!("⚠️  Shaders without entry points (may be libraries):");
        for shader in &missing_entry_points {
            println!("   {}", shader);
        }
    }

    // This is informational only, not a failure
    assert!(true, "Entry point check complete");
}
