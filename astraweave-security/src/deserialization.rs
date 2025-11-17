/// Secure deserialization utilities with size limits
/// Prevents DoS attacks via oversized JSON/TOML/RON files
use anyhow::{Result, bail};
use std::path::Path;
use std::io::{Read, BufReader};
use std::fs::File;
use serde::de::DeserializeOwned;

/// Maximum allowed file sizes for configuration formats
pub const MAX_JSON_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
pub const MAX_TOML_BYTES: u64 = 5 * 1024 * 1024;  // 5 MB
pub const MAX_RON_BYTES: u64 = 5 * 1024 * 1024;   // 5 MB

/// Read limiter that enforces maximum bytes read
struct ReadLimiter<R: Read> {
    inner: R,
    remaining: u64,
}

impl<R: Read> ReadLimiter<R> {
    fn new(inner: R, limit: u64) -> Self {
        Self {
            inner,
            remaining: limit,
        }
    }
}

impl<R: Read> Read for ReadLimiter<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.remaining == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Size limit exceeded during deserialization"
            ));
        }
        
        let to_read = buf.len().min(self.remaining as usize);
        let n = self.inner.read(&mut buf[..to_read])?;
        self.remaining -= n as u64;
        Ok(n)
    }
}

/// Parse JSON with size limit using streaming deserializer
pub fn parse_json_limited<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let limited = ReadLimiter::new(reader, MAX_JSON_BYTES);
    
    serde_json::from_reader(limited)
        .map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))
}

/// Parse TOML with size limit (pre-check via metadata)
pub fn parse_toml_limited<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let size = std::fs::metadata(path)?.len();
    
    if size > MAX_TOML_BYTES {
        bail!("TOML file too large: {} bytes (max: {})", size, MAX_TOML_BYTES);
    }
    
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("TOML parse error: {}", e))
}

/// Parse RON with size limit (pre-check via metadata)
pub fn parse_ron_limited<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let size = std::fs::metadata(path)?.len();
    
    if size > MAX_RON_BYTES {
        bail!("RON file too large: {} bytes (max: {})", size, MAX_RON_BYTES);
    }
    
    let content = std::fs::read_to_string(path)?;
    ron::from_str(&content)
        .map_err(|e| anyhow::anyhow!("RON parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[derive(Deserialize, Debug)]
    struct TestData {
        value: String,
    }
    
    #[test]
    fn test_json_within_limit() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"value": "test"}}"#).unwrap();
        
        let result: TestData = parse_json_limited(file.path()).unwrap();
        assert_eq!(result.value, "test");
    }
    
    #[test]
    fn test_json_exceeds_limit() {
        let mut file = NamedTempFile::new().unwrap();
        
        // Create large JSON (> 10 MB)
        write!(file, r#"{{"value": ""#).unwrap();
        let large_string = "x".repeat(11 * 1024 * 1024);
        write!(file, r#""{}"}}"#, large_string).unwrap();
        file.flush().unwrap();
        
        let result: Result<TestData> = parse_json_limited(file.path());
        assert!(result.is_err());
        // Error message might be from JSON parser or size limiter
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Size limit") || err_msg.contains("parse"));
    }
    
    #[test]
    fn test_toml_within_limit() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"value = "test""#).unwrap();
        
        let result: TestData = parse_toml_limited(file.path()).unwrap();
        assert_eq!(result.value, "test");
    }
    
    #[test]
    fn test_toml_exceeds_limit() {
        let mut file = NamedTempFile::new().unwrap();
        
        // Create large TOML (> 5 MB)
        write!(file, "value = \"").unwrap();
        let large_string = "x".repeat(6 * 1024 * 1024);
        writeln!(file, "{}\"", large_string).unwrap();
        file.flush().unwrap();
        
        let result: Result<TestData> = parse_toml_limited(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }
    
    #[test]
    fn test_ron_within_limit() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"(value: "test")"#).unwrap();
        
        let result: TestData = parse_ron_limited(file.path()).unwrap();
        assert_eq!(result.value, "test");
    }
    
    #[test]
    fn test_ron_exceeds_limit() {
        let mut file = NamedTempFile::new().unwrap();
        
        // Create large RON (> 5 MB)
        write!(file, "(value: \"").unwrap();
        let large_string = "x".repeat(6 * 1024 * 1024);
        writeln!(file, "{}\")", large_string).unwrap();
        file.flush().unwrap();
        
        let result: Result<TestData> = parse_ron_limited(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }
}
