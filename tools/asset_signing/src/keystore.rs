//! Persistent key storage using OS keyring for asset signing
//!
//! Security features:
//! - Keys stored in OS credential manager (Windows Credential Manager, macOS Keychain, Linux Secret Service)
//! - Ed25519 signatures over SHA-256 hashes
//! - Base64 encoding for safe storage
//! - Automatic key generation with OsRng

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use keyring::Entry;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// KeyStore manages persistent signing keys using the OS keyring
pub struct KeyStore {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyStore {
    /// Load existing key from OS keyring or generate a new one
    ///
    /// Keys are stored with the service name "AstraWeave-AssetSigning"
    /// and the provided key_name as the account/username.
    ///
    /// # Arguments
    /// * `key_name` - Identifier for the key (e.g., "developer_signing_key")
    ///
    /// # Errors
    /// Returns error if keyring is unavailable or key operations fail
    pub fn load_or_generate(key_name: &str) -> Result<Self, String> {
        let service = "AstraWeave-AssetSigning";
        let entry = Entry::new(service, key_name)
            .map_err(|e| format!("Failed to access OS keyring: {}", e))?;

        let signing_key = match entry.get_password() {
            Ok(encoded_key) => {
                // Decode existing key from base64
                let key_bytes = BASE64
                    .decode(encoded_key)
                    .map_err(|e| format!("Failed to decode stored key: {}", e))?;

                if key_bytes.len() != SECRET_KEY_LENGTH {
                    return Err(format!(
                        "Stored key has invalid length: {} (expected {})",
                        key_bytes.len(),
                        SECRET_KEY_LENGTH
                    ));
                }

                let mut key_array = [0u8; SECRET_KEY_LENGTH];
                key_array.copy_from_slice(&key_bytes);
                SigningKey::from_bytes(&key_array)
            }
            Err(_) => {
                // Generate new key and store it
                let new_key = SigningKey::generate(&mut OsRng);
                let encoded = BASE64.encode(new_key.to_bytes());

                entry
                    .set_password(&encoded)
                    .map_err(|e| format!("Failed to store key in OS keyring: {}", e))?;

                eprintln!("Generated new signing key: {}", key_name);
                new_key
            }
        };

        let verifying_key = VerifyingKey::from(&signing_key);

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Export the public key to PEM format file
    ///
    /// # Arguments
    /// * `path` - Output file path for the PEM-encoded public key
    pub fn export_public_key(&self, path: &str) -> Result<(), String> {
        let public_key_bytes = self.verifying_key.as_bytes();
        let encoded = BASE64.encode(public_key_bytes);

        let pem_content = format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
            encoded
        );

        std::fs::write(path, pem_content).map_err(|e| format!("Failed to write public key: {}", e))
    }

    /// Import a public key from PEM format file
    ///
    /// # Arguments
    /// * `path` - Path to PEM-encoded public key file
    pub fn import_public_key(path: &str) -> Result<VerifyingKey, String> {
        let pem_content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read public key: {}", e))?;

        // Strip PEM header/footer and whitespace
        let key_data: String = pem_content
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("");

        let key_bytes = BASE64
            .decode(key_data.trim())
            .map_err(|e| format!("Failed to decode public key: {}", e))?;

        if key_bytes.len() != 32 {
            return Err(format!(
                "Invalid public key length: {} (expected 32)",
                key_bytes.len()
            ));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);

        VerifyingKey::from_bytes(&key_array)
            .map_err(|e| format!("Invalid Ed25519 public key: {}", e))
    }

    /// Sign JSON manifest data
    ///
    /// # Arguments
    /// * `manifest_json` - The JSON string to sign
    ///
    /// # Returns
    /// Base64-encoded signature
    pub fn sign(&self, manifest_json: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(manifest_json.as_bytes());
        let hash = hasher.finalize();

        let signature: Signature = self.signing_key.sign(&hash);
        BASE64.encode(signature.to_bytes())
    }

    /// Verify a signature on JSON manifest data
    ///
    /// # Arguments
    /// * `public_key` - The verifying key to use
    /// * `manifest_json` - The JSON string that was signed
    /// * `signature_b64` - Base64-encoded signature
    ///
    /// # Returns
    /// `Ok(true)` if signature is valid, `Ok(false)` if invalid, `Err` on parse errors
    pub fn verify(
        public_key: &VerifyingKey,
        manifest_json: &str,
        signature_b64: &str,
    ) -> Result<bool, String> {
        let sig_bytes = BASE64
            .decode(signature_b64)
            .map_err(|e| format!("Failed to decode signature: {}", e))?;

        if sig_bytes.len() != 64 {
            return Err(format!(
                "Invalid signature length: {} (expected 64)",
                sig_bytes.len()
            ));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_array);

        let mut hasher = Sha256::new();
        hasher.update(manifest_json.as_bytes());
        let hash = hasher.finalize();

        Ok(public_key.verify_strict(&hash, &signature).is_ok())
    }

    /// Get the verifying (public) key
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_sign_verify() {
        let keystore = KeyStore::load_or_generate("test_key_roundtrip").unwrap();
        let message = r#"{"entries":[{"src":"test.png","out":"test.ktx2"}]}"#;

        let signature = keystore.sign(message);
        let verified = KeyStore::verify(keystore.verifying_key(), message, &signature).unwrap();

        assert!(verified, "Signature verification should succeed");
    }

    #[test]
    fn test_keystore_tamper_detection() {
        let keystore = KeyStore::load_or_generate("test_key_tamper").unwrap();
        let message = r#"{"entries":[{"src":"test.png","out":"test.ktx2"}]}"#;
        let tampered = r#"{"entries":[{"src":"evil.png","out":"test.ktx2"}]}"#;

        let signature = keystore.sign(message);
        let verified = KeyStore::verify(keystore.verifying_key(), tampered, &signature).unwrap();

        assert!(!verified, "Tampered message should fail verification");
    }

    #[test]
    fn test_export_import_public_key() {
        let keystore = KeyStore::load_or_generate("test_key_export").unwrap();
        let temp_path = "test_pubkey.pem";

        keystore.export_public_key(temp_path).unwrap();
        let imported_key = KeyStore::import_public_key(temp_path).unwrap();

        assert_eq!(
            keystore.verifying_key().as_bytes(),
            imported_key.as_bytes(),
            "Exported and imported keys should match"
        );

        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_key_persistence() {
        let key_name = "test_key_persistence";

        // First load - generates key
        let keystore1 = KeyStore::load_or_generate(key_name).unwrap();
        let pubkey1 = keystore1.verifying_key().as_bytes();

        // Second load - should retrieve same key
        let keystore2 = KeyStore::load_or_generate(key_name).unwrap();
        let pubkey2 = keystore2.verifying_key().as_bytes();

        assert_eq!(pubkey1, pubkey2, "Keys should persist across loads");
    }
}
