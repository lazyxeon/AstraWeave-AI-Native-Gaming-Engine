//! Kani formal verification proofs for SDK C ABI functions
//!
//! These proofs verify FFI safety properties:
//! - Buffer overflow protection
//! - Null pointer handling
//! - Version string encoding correctness
//!
//! Run with: `cargo kani --package astraweave-sdk`

#![cfg(kani)]

use std::ffi::CStr;

// Import the FFI functions
use super::{aw_version, aw_version_string, AWVersion};

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

/// Verify aw_version is consistent with aw_version_string
#[kani::proof]
fn version_consistency() {
    let version = aw_version();
    let mut buf = vec![0u8; 50];

    unsafe { aw_version_string(buf.as_mut_ptr(), buf.len()) };

    // Parse major version from string
    let null_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    let string = std::str::from_utf8(&buf[..null_pos]).ok();

    if let Some(s) = string {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() >= 3 {
            if let Ok(major) = parts[0].parse::<u16>() {
                kani::assert(
                    major == version.major,
                    "String major must match struct major",
                );
            }
        }
    }
}

/// Verify minimum buffer sizes work correctly
#[kani::proof]
fn version_string_minimum_buffers() {
    // Test various small buffer sizes
    let sizes = [1usize, 2, 3, 4, 5, 6, 7, 8];

    for &size in sizes.iter() {
        let mut buf = vec![0xFFu8; size];
        let _written = unsafe { aw_version_string(buf.as_mut_ptr(), size) };

        // Regardless of buffer size, the function should write a null terminator
        // or not write at all (for size 0, which we don't test here)
        let has_null = buf.iter().any(|&b| b == 0);
        kani::assert(has_null, "Must always write null terminator");
    }
}
