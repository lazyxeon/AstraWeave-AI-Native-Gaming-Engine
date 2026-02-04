//! Kani formal verification proofs for SDK C ABI functions
//!
//! These proofs verify FFI safety properties:
//! - Buffer overflow protection
//! - Null pointer handling
//! - Version string encoding correctness
//!
//! Run with: `cargo kani --package astraweave-sdk`

// Import the FFI functions
use crate::{aw_version, aw_version_string, AWVersion};

/// Verify aw_version returns valid version struct
#[kani::proof]
fn version_returns_valid_struct() {
    let version = aw_version();

    // Version numbers should be reasonable (not garbage)
    // The actual values depend on Cargo.toml, but they should parse correctly
    kani::assert(
        version.major <= 100,
        "Major version should be reasonable",
    );
    kani::assert(
        version.minor <= 100,
        "Minor version should be reasonable",
    );
    kani::assert(
        version.patch <= 1000,
        "Patch version should be reasonable",
    );
}

/// Verify aw_version_string with null buffer returns required size
#[kani::proof]
fn version_string_null_buffer_returns_size() {
    let required = unsafe { aw_version_string(std::ptr::null_mut(), 0) };

    // Version string should be at least "0.0.0\0" = 6 bytes
    kani::assert(required >= 6, "Version string requires at least 6 bytes");

    // Version string should be reasonable length
    kani::assert(required <= 50, "Version string should be < 50 bytes");
}

/// Verify aw_version_string never writes past buffer end
#[kani::proof]
#[kani::unwind(32)]
fn version_string_no_buffer_overflow() {
    let len: usize = kani::any();
    kani::assume(len <= 30); // Bound for tractability

    if len == 0 {
        // Zero-length buffer: should return required size without writing
        let required = unsafe { aw_version_string(std::ptr::null_mut(), 0) };
        kani::assert(required > 0, "Should return required size");
    } else {
        // Create buffer and verify no overflow
        let mut buf = vec![0xFFu8; len];
        let written = unsafe { aw_version_string(buf.as_mut_ptr(), len) };

        // Check that null terminator is within bounds
        if len > 0 {
            // Find null terminator
            let null_pos = buf.iter().position(|&b| b == 0);

            if let Some(pos) = null_pos {
                kani::assert(
                    pos < len,
                    "Null terminator must be within buffer bounds",
                );
            }
        }

        kani::assert(written > 0, "Should return required size");
    }
}

/// Verify aw_version_string with sufficient buffer writes valid string
#[kani::proof]
fn version_string_writes_valid_utf8() {
    // Use a buffer large enough for any reasonable version
    let mut buf = vec![0u8; 50];

    let written = unsafe { aw_version_string(buf.as_mut_ptr(), buf.len()) };

    // Find the null terminator
    let null_pos = buf.iter().position(|&b| b == 0);
    kani::assert(null_pos.is_some(), "Must write null terminator");

    if let Some(pos) = null_pos {
        // Check the string before null terminator is valid UTF-8
        let string_bytes = &buf[..pos];

        // Each byte should be valid ASCII (version strings are ASCII)
        for &byte in string_bytes {
            kani::assert(
                byte < 128,
                "Version string must be ASCII",
            );
        }
    }
}

/// Verify version string contains dots (x.y.z format)
#[kani::proof]
fn version_string_format() {
    let mut buf = vec![0u8; 50];

    unsafe { aw_version_string(buf.as_mut_ptr(), buf.len()) };

    // Count dots in the string
    let mut dot_count = 0;
    for &byte in buf.iter() {
        if byte == 0 {
            break;
        }
        if byte == b'.' {
            dot_count += 1;
        }
    }

    // Semantic versioning has at least 2 dots: x.y.z
    kani::assert(
        dot_count >= 2,
        "Version string must have at least 2 dots (x.y.z)",
    );
}

/// Verify AWVersion struct layout is stable
#[kani::proof]
fn aw_version_struct_layout() {
    // Verify struct is the expected size (3 × u16 = 6 bytes)
    kani::assert(
        std::mem::size_of::<AWVersion>() == 6,
        "AWVersion must be 6 bytes",
    );

    // Verify alignment (u16 = 2 byte alignment)
    kani::assert(
        std::mem::align_of::<AWVersion>() == 2,
        "AWVersion must be 2-byte aligned",
    );
}

/// Verify aw_version returns valid version
#[kani::proof]
fn version_consistency() {
    let version = aw_version();

    // Verify version fields are in valid ranges
    // Major version should be reasonable (0-100 for practical use)
    kani::assert(version.major <= 100, "Major version must be reasonable");
    // Minor and patch can be any u16 value, but verify they're set
    kani::assert(
        version.major > 0 || version.minor > 0 || version.patch > 0,
        "Version must have at least one non-zero component",
    );
}

/// Verify minimum buffer sizes work correctly
/// Uses fixed-size arrays since Kani struggles with Vec allocations
#[kani::proof]
#[kani::unwind(10)]
fn version_string_minimum_buffers() {
    // Test size 1 buffer
    let mut buf1 = [0xFFu8; 1];
    let _ = unsafe { aw_version_string(buf1.as_mut_ptr(), 1) };
    kani::assert(buf1.iter().any(|&b| b == 0), "Size 1: must have null terminator");

    // Test size 4 buffer
    let mut buf4 = [0xFFu8; 4];
    let _ = unsafe { aw_version_string(buf4.as_mut_ptr(), 4) };
    kani::assert(buf4.iter().any(|&b| b == 0), "Size 4: must have null terminator");

    // Test size 8 buffer
    let mut buf8 = [0xFFu8; 8];
    let _ = unsafe { aw_version_string(buf8.as_mut_ptr(), 8) };
    kani::assert(buf8.iter().any(|&b| b == 0), "Size 8: must have null terminator");
}
