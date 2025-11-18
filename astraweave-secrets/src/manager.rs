use super::backend::SecretBackend;
use super::keyring_backend::KeyringBackend;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub struct SecretManager {
    backend: Arc<dyn SecretBackend>,
}

static GLOBAL: Lazy<SecretManager> = Lazy::new(|| SecretManager {
    backend: Arc::new(KeyringBackend::new()),
});

impl SecretManager {
    pub fn global() -> &'static Self {
        &GLOBAL
    }

    pub fn get(&self, key: &str) -> anyhow::Result<super::backend::SecretValue> {
        self.backend.get(key)
    }

    pub fn set(&self, key: &str, value: super::backend::SecretValue) -> anyhow::Result<()> {
        self.backend.set(key, value)
    }

    pub fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.backend.delete(key)
    }
}
