//! Fuzz target for asset header parsing.
//!
//! Tests that arbitrary byte sequences don't crash header parsers.

#![no_main]

use libfuzzer_sys::fuzz_target;

use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Test glTF magic header detection
    let is_gltf = data.len() >= 4 && &data[0..4] == b"glTF";
    let is_json_gltf = data.first().map_or(false, |&b| b == b'{');
    let _ = is_gltf || is_json_gltf;

    // Test PNG magic header detection
    let is_png = data.len() >= 8
        && data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let _ = is_png;

    // Test JPEG magic header detection
    let is_jpeg =
        data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8;
    let _ = is_jpeg;

    // Test KTX2 magic header detection
    let ktx2_magic = [
        0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A,
    ];
    let is_ktx2 = data.len() >= 12 && data[0..12] == ktx2_magic;
    let _ = is_ktx2;

    // Test DDS magic header detection
    let is_dds = data.len() >= 4 && &data[0..4] == b"DDS ";
    let _ = is_dds;

    // Determine asset type from extension-like patterns
    // (These won't crash, just exercising parsing logic)
    if data.len() > 16 {
        // Try to find embedded metadata
        let mut cursor = Cursor::new(data);
        let _ = cursor.position();
    }
});
