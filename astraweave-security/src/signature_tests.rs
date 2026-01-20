//! Comprehensive test suite for cryptographic signature verification
//!
//! This module provides extensive testing for asset signature verification,
//! which is critical for preventing asset tampering and ensuring game integrity.
//!
//! **Coverage Goal**: Test all signature operations, edge cases, and attack scenarios

#![allow(unused_imports)]

use super::*;
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};

// ============================================================================
// Test Suite 1: Basic Signature Operations (5 tests)
// ============================================================================

#[test]
fn test_generate_keypair_produces_valid_keys() {
    let (signing_key, verifying_key) = generate_keypair();

    // Keys should be usable for signing and verification
    let data = b"test data";
    let signature = generate_signature(data, &signing_key);
    assert!(verify_signature(data, &signature, &verifying_key));
}

#[test]
fn test_sign_and_verify_basic_workflow() {
    let (signing_key, verifying_key) = generate_keypair();
    let data = b"Hello, world!";

    let signature = generate_signature(data, &signing_key);
    let is_valid = verify_signature(data, &signature, &verifying_key);

    assert!(is_valid, "Valid signature should verify successfully");
}

#[test]
fn test_signature_is_deterministic() {
    let (signing_key, verifying_key) = generate_keypair();
    let data = b"deterministic test";

    let signature1 = generate_signature(data, &signing_key);
    let signature2 = generate_signature(data, &signing_key);

    // ed25519 signatures are deterministic
    assert_eq!(
        signature1.to_bytes(),
        signature2.to_bytes(),
        "Same data with same key should produce identical signatures"
    );

    assert!(verify_signature(data, &signature1, &verifying_key));
    assert!(verify_signature(data, &signature2, &verifying_key));
}

#[test]
fn test_signature_different_data_different_signature() {
    let (signing_key, verifying_key) = generate_keypair();

    let data1 = b"data1";
    let data2 = b"data2";

    let signature1 = generate_signature(data1, &signing_key);
    let signature2 = generate_signature(data2, &signing_key);

    // Different data should produce different signatures
    assert_ne!(
        signature1.to_bytes(),
        signature2.to_bytes(),
        "Different data should produce different signatures"
    );

    // Cross-verification should fail
    assert!(!verify_signature(data1, &signature2, &verifying_key));
    assert!(!verify_signature(data2, &signature1, &verifying_key));
}

#[test]
fn test_signature_empty_data() {
    let (signing_key, verifying_key) = generate_keypair();
    let empty_data: &[u8] = &[];

    let signature = generate_signature(empty_data, &signing_key);
    assert!(
        verify_signature(empty_data, &signature, &verifying_key),
        "Should be able to sign and verify empty data"
    );
}

// ============================================================================
// Test Suite 2: Data Tampering Detection (5 tests)
// ============================================================================

#[test]
fn test_tampered_data_fails_verification() {
    let (signing_key, verifying_key) = generate_keypair();
    let original_data = b"original asset data";
    let signature = generate_signature(original_data, &signing_key);

    // Tamper with data
    let tampered_data = b"tampered asset data";

    assert!(
        !verify_signature(tampered_data, &signature, &verifying_key),
        "Tampered data should fail verification"
    );
}

#[test]
fn test_single_byte_modification_detected() {
    let (signing_key, verifying_key) = generate_keypair();
    let original_data = b"asset_v1.0.bin";
    let signature = generate_signature(original_data, &signing_key);

    // Modify a single byte
    let mut tampered = original_data.to_vec();
    tampered[0] ^= 0x01; // Flip one bit

    assert!(
        !verify_signature(&tampered, &signature, &verifying_key),
        "Single byte modification should be detected"
    );
}

#[test]
fn test_trailing_byte_addition_detected() {
    let (signing_key, verifying_key) = generate_keypair();
    let original_data = b"asset data";
    let signature = generate_signature(original_data, &signing_key);

    // Add trailing byte
    let mut extended = original_data.to_vec();
    extended.push(0x00);

    assert!(
        !verify_signature(&extended, &signature, &verifying_key),
        "Trailing byte addition should be detected"
    );
}

#[test]
fn test_truncated_data_fails_verification() {
    let (signing_key, verifying_key) = generate_keypair();
    let original_data = b"full asset data";
    let signature = generate_signature(original_data, &signing_key);

    // Truncate data
    let truncated = &original_data[..original_data.len() - 1];

    assert!(
        !verify_signature(truncated, &signature, &verifying_key),
        "Truncated data should fail verification"
    );
}

#[test]
fn test_reordered_bytes_detected() {
    let (signing_key, verifying_key) = generate_keypair();
    let original_data = b"ABCDEF";
    let signature = generate_signature(original_data, &signing_key);

    // Reorder bytes
    let reordered = b"FEDCBA";

    assert!(
        !verify_signature(reordered, &signature, &verifying_key),
        "Reordered bytes should be detected"
    );
}

// ============================================================================
// Test Suite 3: Wrong Key Detection (3 tests)
// ============================================================================

#[test]
fn test_wrong_verifying_key_fails() {
    let (signing_key1, _) = generate_keypair();
    let (_, verifying_key2) = generate_keypair();

    let data = b"asset data";
    let signature = generate_signature(data, &signing_key1);

    // Verify with wrong key
    assert!(
        !verify_signature(data, &signature, &verifying_key2),
        "Verification with wrong key should fail"
    );
}

#[test]
fn test_signature_replay_attack_prevented() {
    let (signing_key1, verifying_key1) = generate_keypair();
    let (_, verifying_key2) = generate_keypair();

    let data = b"legitimate asset";
    let signature = generate_signature(data, &signing_key1);

    // Valid with correct key
    assert!(verify_signature(data, &signature, &verifying_key1));

    // Replay to different key should fail
    assert!(!verify_signature(data, &signature, &verifying_key2));
}

#[test]
fn test_multiple_keypairs_independent() {
    let (signing_key1, verifying_key1) = generate_keypair();
    let (signing_key2, verifying_key2) = generate_keypair();

    let data = b"shared data";

    let signature1 = generate_signature(data, &signing_key1);
    let signature2 = generate_signature(data, &signing_key2);

    // Each signature only verifies with its own key
    assert!(verify_signature(data, &signature1, &verifying_key1));
    assert!(verify_signature(data, &signature2, &verifying_key2));

    assert!(!verify_signature(data, &signature1, &verifying_key2));
    assert!(!verify_signature(data, &signature2, &verifying_key1));
}

// ============================================================================
// Test Suite 4: Large Data Signing (3 tests)
// ============================================================================

#[test]
fn test_sign_large_asset_1mb() {
    let (signing_key, verifying_key) = generate_keypair();

    // 1MB of data
    let large_data = vec![0xAB; 1024 * 1024];

    let signature = generate_signature(&large_data, &signing_key);
    assert!(
        verify_signature(&large_data, &signature, &verifying_key),
        "Should handle 1MB asset signing"
    );
}

#[test]
fn test_sign_large_asset_10mb() {
    let (signing_key, verifying_key) = generate_keypair();

    // 10MB of data
    let large_data = vec![0xCD; 10 * 1024 * 1024];

    let signature = generate_signature(&large_data, &signing_key);
    assert!(
        verify_signature(&large_data, &signature, &verifying_key),
        "Should handle 10MB asset signing"
    );
}

#[test]
fn test_sign_varying_size_assets() {
    let (signing_key, verifying_key) = generate_keypair();

    let sizes = vec![0, 1, 100, 1024, 10_000, 100_000];

    for size in sizes {
        let data = vec![0xFF; size];
        let signature = generate_signature(&data, &signing_key);

        assert!(
            verify_signature(&data, &signature, &verifying_key),
            "Should handle {} byte asset",
            size
        );
    }
}

// ============================================================================
// Test Suite 5: Hash Integrity (4 tests)
// ============================================================================

#[test]
fn test_hash_data_deterministic() {
    let data = b"test asset";

    let hash1 = hash_data(data);
    let hash2 = hash_data(data);

    assert_eq!(hash1, hash2, "Hash should be deterministic");
}

#[test]
fn test_hash_different_data_different_hash() {
    let data1 = b"asset1";
    let data2 = b"asset2";

    let hash1 = hash_data(data1);
    let hash2 = hash_data(data2);

    assert_ne!(
        hash1, hash2,
        "Different data should produce different hashes"
    );
}

#[test]
fn test_hash_output_format() {
    let data = b"test data";
    let hash = hash_data(data);

    // SHA256 produces 64 hex characters (32 bytes * 2)
    assert_eq!(hash.len(), 64, "SHA256 hash should be 64 hex characters");

    // Should only contain hex characters
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash should only contain hex characters"
    );
}

#[test]
fn test_hash_empty_data() {
    let empty: &[u8] = &[];
    let hash = hash_data(empty);

    // SHA256 of empty data is known
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

// ============================================================================
// Integration Test: Asset Verification Workflow
// ============================================================================

#[test]
fn test_complete_asset_verification_workflow() {
    // Simulate asset publisher workflow
    let (publisher_signing_key, publisher_verifying_key) = generate_keypair();

    // Create asset data
    let asset_data = b"game_asset_texture_v1.0.dds";

    // Publisher signs the asset
    let asset_signature = generate_signature(asset_data, &publisher_signing_key);

    // Compute asset hash for quick integrity check
    let asset_hash = hash_data(asset_data);

    // === Client-side verification ===

    // 1. Verify hash matches (fast check)
    let received_hash = hash_data(asset_data);
    assert_eq!(asset_hash, received_hash, "Asset hash should match");

    // 2. Verify signature (cryptographic proof)
    assert!(
        verify_signature(asset_data, &asset_signature, &publisher_verifying_key),
        "Asset signature should verify with publisher's key"
    );

    // 3. Tampered asset should fail both checks
    let mut tampered_asset = asset_data.to_vec();
    tampered_asset[0] ^= 0xFF;

    let tampered_hash = hash_data(&tampered_asset);
    assert_ne!(
        asset_hash, tampered_hash,
        "Tampered asset hash should differ"
    );

    assert!(
        !verify_signature(&tampered_asset, &asset_signature, &publisher_verifying_key),
        "Tampered asset signature should fail"
    );
}

#[cfg(test)]
mod signature_edge_cases {
    use super::*;

    #[test]
    fn test_signature_with_all_zero_data() {
        let (signing_key, verifying_key) = generate_keypair();
        let zero_data = vec![0u8; 1000];

        let signature = generate_signature(&zero_data, &signing_key);
        assert!(verify_signature(&zero_data, &signature, &verifying_key));
    }

    #[test]
    fn test_signature_with_all_ones_data() {
        let (signing_key, verifying_key) = generate_keypair();
        let ones_data = vec![0xFFu8; 1000];

        let signature = generate_signature(&ones_data, &signing_key);
        assert!(verify_signature(&ones_data, &signature, &verifying_key));
    }

    #[test]
    fn test_signature_with_random_data() {
        let (signing_key, verifying_key) = generate_keypair();

        // Generate random data
        let random_data: Vec<u8> = (0..1000).map(|_| rand::random()).collect();

        let signature = generate_signature(&random_data, &signing_key);
        assert!(verify_signature(&random_data, &signature, &verifying_key));
    }
}
