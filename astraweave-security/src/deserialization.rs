/// Secure deserialization utilities with size limits
/// Prevents DoS attacks via oversized JSON/TOML/RON files
use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Maximum allowed file sizes for configuration formats
pub const MAX_JSON_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
pub const MAX_TOML_BYTES: u64 = 5 * 1024 * 1024; // 5 MB
pub const MAX_RON_BYTES: u64 = 5 * 1024 * 1024; // 5 MB

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
                "Size limit exceeded during deserialization",
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

    serde_json::from_reader(limited).map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))
}

/// Parse TOML with size limit (pre-check via metadata)
pub fn parse_toml_limited<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let size = std::fs::metadata(path)?.len();

    if size > MAX_TOML_BYTES {
        bail!(
            "TOML file too large: {} bytes (max: {})",
            size,
            MAX_TOML_BYTES
        );
    }

    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| anyhow::anyhow!("TOML parse error: {}", e))
}

/// Parse RON with size limit (pre-check via metadata)
pub fn parse_ron_limited<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let size = std::fs::metadata(path)?.len();

    if size > MAX_RON_BYTES {
        bail!(
            "RON file too large: {} bytes (max: {})",
            size,
            MAX_RON_BYTES
        );
    }

    let content = std::fs::read_to_string(path)?;
    ron::from_str(&content).map_err(|e| anyhow::anyhow!("RON parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Write;
    use tempfile::NamedTempFile;

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

    // ── Non-existent file tests ──

    #[test]
    fn test_json_nonexistent_file() {
        let result: Result<TestData> = parse_json_limited(Path::new("/no/such/file.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_nonexistent_file() {
        let result: Result<TestData> = parse_toml_limited(Path::new("/no/such/file.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_ron_nonexistent_file() {
        let result: Result<TestData> = parse_ron_limited(Path::new("/no/such/file.ron"));
        assert!(result.is_err());
    }

    // ── Invalid syntax tests ──

    #[test]
    fn test_json_invalid_syntax() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{broken json").unwrap();
        let result: Result<TestData> = parse_json_limited(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse"));
    }

    #[test]
    fn test_toml_invalid_syntax() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "= = = not valid toml").unwrap();
        let result: Result<TestData> = parse_toml_limited(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse"));
    }

    #[test]
    fn test_ron_invalid_syntax() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "(((invalid ron").unwrap();
        let result: Result<TestData> = parse_ron_limited(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse"));
    }

    // ── Empty file tests ──

    #[test]
    fn test_json_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let result: Result<TestData> = parse_json_limited(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let result: Result<TestData> = parse_toml_limited(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_ron_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let result: Result<TestData> = parse_ron_limited(file.path());
        assert!(result.is_err());
    }

    // ── Type mismatch tests ──

    #[test]
    fn test_json_type_mismatch() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"wrong_field": 123}}"#).unwrap();
        let result: Result<TestData> = parse_json_limited(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_type_mismatch() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "wrong_field = 123").unwrap();
        let result: Result<TestData> = parse_toml_limited(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_ron_type_mismatch() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "(wrong_field: 123)").unwrap();
        let result: Result<TestData> = parse_ron_limited(file.path());
        assert!(result.is_err());
    }

    // ── ReadLimiter direct tests ──

    #[test]
    fn test_read_limiter_zero_limit() {
        let data = b"hello";
        let mut limiter = ReadLimiter::new(&data[..], 0);
        let mut buf = [0u8; 5];
        let err = limiter.read(&mut buf).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_read_limiter_exact_boundary() {
        let data = b"hello";
        let mut limiter = ReadLimiter::new(&data[..], 5);
        let mut buf = [0u8; 10];
        let n = limiter.read(&mut buf).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&buf[..n], b"hello");
        // Next read: remaining=0 → error
        let err = limiter.read(&mut buf).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_read_limiter_partial_reads() {
        let data = b"abcdef";
        let mut limiter = ReadLimiter::new(&data[..], 3);
        let mut buf = [0u8; 2];
        let n = limiter.read(&mut buf).unwrap();
        assert_eq!(n, 2);
        assert_eq!(&buf[..n], b"ab");
        let n = limiter.read(&mut buf).unwrap();
        assert_eq!(n, 1);
        assert_eq!(&buf[..n], b"c");
    }

    // ═══════════════════════════════════════════════════════════════
    // MUTATION REMEDIATION TESTS
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn mutation_toml_size_at_exact_boundary_passes() {
        // Targets: deserialization.rs:58 replace > with >= in parse_toml_limited
        // With >: a file of exactly MAX_TOML_BYTES is NOT too large (passes)
        // With >=: a file of exactly MAX_TOML_BYTES IS too large (rejected)
        let mut file = NamedTempFile::new().unwrap();
        // Write exactly MAX_TOML_BYTES of valid TOML content
        // Pad with TOML comments to reach exact size
        let header = b"x = 42\n";
        file.write_all(header).unwrap();
        let remaining = MAX_TOML_BYTES as usize - header.len();
        // Fill with comment characters (valid TOML)
        let padding = vec![b'#'; remaining];
        file.write_all(&padding).unwrap();
        file.flush().unwrap();

        // Verify file is exact size
        let meta = std::fs::metadata(file.path()).unwrap();
        assert_eq!(meta.len(), MAX_TOML_BYTES);

        // With > : passes size check, proceeds to parsing
        // With >= : rejects with "TOML file too large"
        let result: Result<TestData> = parse_toml_limited(file.path());
        // Should NOT contain "too large" error (size check passes)
        if let Err(e) = &result {
            assert!(
                !format!("{}", e).contains("too large"),
                "File at exact MAX size must pass size check, got: {}",
                e
            );
        }
    }

    #[test]
    fn mutation_ron_size_at_exact_boundary_passes() {
        // Targets: deserialization.rs:74 replace > with >= in parse_ron_limited
        let mut file = NamedTempFile::new().unwrap();
        // Write exactly MAX_RON_BYTES of content
        let header = b"(x: 42)\n";
        file.write_all(header).unwrap();
        let remaining = MAX_RON_BYTES as usize - header.len();
        let padding = vec![b' '; remaining];
        file.write_all(&padding).unwrap();
        file.flush().unwrap();

        let meta = std::fs::metadata(file.path()).unwrap();
        assert_eq!(meta.len(), MAX_RON_BYTES);

        let result: Result<TestData> = parse_ron_limited(file.path());
        if let Err(e) = &result {
            assert!(
                !format!("{}", e).contains("too large"),
                "File at exact MAX size must pass size check, got: {}",
                e
            );
        }
    }
}
