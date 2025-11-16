use super::backend::{SecretBackend, SecretValue};
use keyring::Entry;

const SERVICE_NAME: &str = "astraweave.secrets";

pub struct KeyringBackend;

impl KeyringBackend {
    pub fn new() -> Self {
        Self
    }
}

impl SecretBackend for KeyringBackend {
    fn get(&self, key: &str) -> anyhow::Result<SecretValue> {
        let entry = Entry::new(SERVICE_NAME, key)?;
        let password = entry.get_password()?;
        Ok(SecretValue::from_str(&password))
    }

    fn set(&self, key: &str, value: SecretValue) -> anyhow::Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)?;
        entry.set_password(value.as_str()?)?;
        Ok(())
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)?;
        entry.delete_credential()?;
        Ok(())
    }

    fn list_keys(&self) -> anyhow::Result<Vec<String>> {
        // Note: keyring doesn't support listing, return empty for now
        // Real implementation would track keys in separate metadata
        Ok(Vec::new())
    }
}
