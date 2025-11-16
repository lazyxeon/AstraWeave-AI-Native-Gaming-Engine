use anyhow::Result;
use zeroize::Zeroize;

pub struct SecretValue(Vec<u8>);

impl SecretValue {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn from_str(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }

    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.0).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SecretValue {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub trait SecretBackend: Send + Sync {
    fn get(&self, key: &str) -> Result<SecretValue>;
    fn set(&self, key: &str, value: SecretValue) -> Result<()>;
    fn delete(&self, key: &str) -> Result<()>;
    fn list_keys(&self) -> Result<Vec<String>>;
}
