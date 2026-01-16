//! Asset Packaging Module
//!
//! Provides functionality for packaging game assets into distributable archives.
//! Supports compression, encryption (optional), and asset manifests for efficient
//! runtime loading.
//!
//! # Features
//!
//! - **Archive Creation**: Pack multiple assets into a single .awpack file
//! - **Compression**: zstd compression with configurable levels
//! - **Manifest Generation**: JSON manifest for fast asset lookup
//! - **Streaming Support**: Assets can be loaded on-demand from archives

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Magic bytes for AstraWeave asset pack files
pub const PACK_MAGIC: [u8; 4] = *b"AWPK";

/// Current pack format version
pub const PACK_VERSION: u32 = 1;

/// Progress callback type for asset packing operations.
/// Takes (progress: f32, message: &str) where progress is 0.0..1.0.
pub type ProgressCallback = Box<dyn Fn(f32, &str) + Send>;

/// Compression method for assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CompressionMethod {
    /// No compression
    None,
    /// Zstandard compression (default)
    #[default]
    Zstd,
    /// LZ4 fast compression
    Lz4,
}

impl CompressionMethod {
    /// Get the file extension hint for this compression
    pub fn extension_hint(&self) -> &str {
        match self {
            CompressionMethod::None => "",
            CompressionMethod::Zstd => ".zst",
            CompressionMethod::Lz4 => ".lz4",
        }
    }
}

/// An entry in the asset pack manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    /// Original asset path (relative to project root)
    pub path: String,
    /// Offset in the pack file (after header)
    pub offset: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Original uncompressed size
    pub uncompressed_size: u64,
    /// Compression method used
    pub compression: CompressionMethod,
    /// CRC32 checksum of uncompressed data
    pub checksum: u32,
}

/// Asset pack manifest - stored at the start of the pack file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    /// Pack format version
    pub version: u32,
    /// Total number of assets
    pub asset_count: u32,
    /// Map of asset paths to entries
    pub assets: HashMap<String, AssetEntry>,
    /// Pack creation timestamp
    pub created_at: u64,
    /// Project name
    pub project_name: String,
}

impl PackManifest {
    /// Create a new empty manifest
    pub fn new(project_name: &str) -> Self {
        Self {
            version: PACK_VERSION,
            asset_count: 0,
            assets: HashMap::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            project_name: project_name.to_string(),
        }
    }

    /// Add an asset entry to the manifest
    pub fn add_asset(&mut self, entry: AssetEntry) {
        self.assets.insert(entry.path.clone(), entry);
        self.asset_count = self.assets.len() as u32;
    }

    /// Get an asset entry by path
    pub fn get_asset(&self, path: &str) -> Option<&AssetEntry> {
        self.assets.get(path)
    }

    /// List all asset paths
    pub fn asset_paths(&self) -> Vec<&str> {
        self.assets.keys().map(|s| s.as_str()).collect()
    }
}

/// Asset pack builder for creating .awpack files
pub struct AssetPackBuilder {
    /// Output file path
    output_path: PathBuf,
    /// Project name for manifest
    project_name: String,
    /// Compression method to use
    compression: CompressionMethod,
    /// Compression level (1-22 for zstd, 1-12 for lz4)
    compression_level: i32,
    /// Assets to pack (source path, archive path)
    assets: Vec<(PathBuf, String)>,
    /// Progress callback
    progress_callback: Option<ProgressCallback>,
}

impl AssetPackBuilder {
    /// Create a new builder
    pub fn new(output_path: impl AsRef<Path>, project_name: &str) -> Self {
        Self {
            output_path: output_path.as_ref().to_path_buf(),
            project_name: project_name.to_string(),
            compression: CompressionMethod::Zstd,
            compression_level: 3,
            assets: Vec::new(),
            progress_callback: None,
        }
    }

    /// Set the compression method
    pub fn with_compression(mut self, method: CompressionMethod) -> Self {
        self.compression = method;
        self
    }

    /// Set the compression level
    pub fn with_compression_level(mut self, level: i32) -> Self {
        self.compression_level = level;
        self
    }

    /// Add an asset to the pack
    pub fn add_asset(mut self, source: impl AsRef<Path>, archive_path: &str) -> Self {
        self.assets
            .push((source.as_ref().to_path_buf(), archive_path.to_string()));
        self
    }

    /// Add all files from a directory
    pub fn add_directory(mut self, source_dir: impl AsRef<Path>, prefix: &str) -> Self {
        let source_dir = source_dir.as_ref();

        let entries = walkdir::WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in entries {
            if entry.file_type().is_file() {
                if let Ok(relative) = entry.path().strip_prefix(source_dir) {
                    let archive_path = if prefix.is_empty() {
                        relative.to_string_lossy().to_string()
                    } else {
                        format!("{}/{}", prefix, relative.to_string_lossy())
                    };
                    // Normalize path separators
                    let archive_path = archive_path.replace('\\', "/");
                    self.assets.push((entry.path().to_path_buf(), archive_path));
                }
            }
        }

        self
    }

    /// Set a progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(f32, &str) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Build the asset pack
    pub fn build(self) -> Result<PackResult, PackError> {
        use std::fs::File;
        use std::io::BufWriter;

        let start_time = std::time::Instant::now();
        let total_assets = self.assets.len();

        if total_assets == 0 {
            return Err(PackError::NoAssets);
        }

        // Create output file
        let file =
            File::create(&self.output_path).map_err(|e| PackError::Io(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        // Write magic and placeholder for manifest offset
        writer
            .write_all(&PACK_MAGIC)
            .map_err(|e| PackError::Io(e.to_string()))?;
        writer
            .write_all(&[0u8; 8]) // Placeholder for manifest offset
            .map_err(|e| PackError::Io(e.to_string()))?;

        let data_start_offset = 12u64; // 4 bytes magic + 8 bytes manifest offset
        let mut current_offset = 0u64;
        let mut manifest = PackManifest::new(&self.project_name);
        let mut total_uncompressed = 0u64;
        let mut total_compressed = 0u64;

        // Process each asset
        for (i, (source_path, archive_path)) in self.assets.iter().enumerate() {
            if let Some(ref callback) = self.progress_callback {
                let progress = (i as f32) / (total_assets as f32);
                callback(progress, &format!("Packing: {}", archive_path));
            }

            // Read source file
            let data = std::fs::read(source_path).map_err(|e| {
                PackError::Io(format!("Failed to read {}: {}", source_path.display(), e))
            })?;

            let uncompressed_size = data.len() as u64;
            let checksum = crc32fast::hash(&data);

            // Compress data
            let compressed_data = match self.compression {
                CompressionMethod::None => data,
                CompressionMethod::Zstd => zstd::encode_all(&data[..], self.compression_level)
                    .map_err(|e| PackError::Compression(e.to_string()))?,
                CompressionMethod::Lz4 => lz4_flex::compress_prepend_size(&data),
            };

            let compressed_size = compressed_data.len() as u64;

            // Write data
            writer
                .write_all(&compressed_data)
                .map_err(|e| PackError::Io(e.to_string()))?;

            // Add to manifest
            manifest.add_asset(AssetEntry {
                path: archive_path.clone(),
                offset: current_offset,
                compressed_size,
                uncompressed_size,
                compression: self.compression,
                checksum,
            });

            current_offset += compressed_size;
            total_uncompressed += uncompressed_size;
            total_compressed += compressed_size;
        }

        // Write manifest at the end
        let manifest_offset = data_start_offset + current_offset;
        let manifest_json =
            serde_json::to_vec(&manifest).map_err(|e| PackError::Io(e.to_string()))?;
        writer
            .write_all(&manifest_json)
            .map_err(|e| PackError::Io(e.to_string()))?;

        // Go back and write manifest offset
        use std::io::Seek;
        writer
            .seek(std::io::SeekFrom::Start(4))
            .map_err(|e| PackError::Io(e.to_string()))?;
        writer
            .write_all(&manifest_offset.to_le_bytes())
            .map_err(|e| PackError::Io(e.to_string()))?;

        writer
            .flush()
            .map_err(|e| PackError::Io(e.to_string()))?;

        if let Some(ref callback) = self.progress_callback {
            callback(1.0, "Pack complete!");
        }

        let duration = start_time.elapsed();
        let compression_ratio = if total_uncompressed > 0 {
            (total_compressed as f64) / (total_uncompressed as f64)
        } else {
            1.0
        };

        Ok(PackResult {
            output_path: self.output_path,
            asset_count: total_assets,
            total_uncompressed_size: total_uncompressed,
            total_compressed_size: total_compressed,
            compression_ratio,
            duration_secs: duration.as_secs_f32(),
        })
    }
}

/// Result of a successful pack operation
#[derive(Debug)]
pub struct PackResult {
    /// Path to the created pack file
    pub output_path: PathBuf,
    /// Number of assets packed
    pub asset_count: usize,
    /// Total size before compression
    pub total_uncompressed_size: u64,
    /// Total size after compression
    pub total_compressed_size: u64,
    /// Compression ratio (compressed/uncompressed)
    pub compression_ratio: f64,
    /// Time taken to create pack
    pub duration_secs: f32,
}

impl PackResult {
    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        let ratio_percent = (1.0 - self.compression_ratio) * 100.0;
        format!(
            "Packed {} assets: {} → {} ({:.1}% reduction) in {:.2}s",
            self.asset_count,
            format_bytes(self.total_uncompressed_size),
            format_bytes(self.total_compressed_size),
            ratio_percent,
            self.duration_secs
        )
    }
}

/// Errors that can occur during packing
#[derive(Debug)]
pub enum PackError {
    NoAssets,
    Io(String),
    Compression(String),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAssets => write!(f, "No assets to pack"),
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Compression(e) => write!(f, "Compression error: {}", e),
        }
    }
}

impl std::error::Error for PackError {}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = PackManifest::new("TestGame");
        assert_eq!(manifest.project_name, "TestGame");
        assert_eq!(manifest.version, PACK_VERSION);
        assert_eq!(manifest.asset_count, 0);
    }

    #[test]
    fn test_manifest_add_asset() {
        let mut manifest = PackManifest::new("TestGame");
        manifest.add_asset(AssetEntry {
            path: "textures/test.png".to_string(),
            offset: 0,
            compressed_size: 100,
            uncompressed_size: 200,
            compression: CompressionMethod::Zstd,
            checksum: 12345,
        });

        assert_eq!(manifest.asset_count, 1);
        assert!(manifest.get_asset("textures/test.png").is_some());
    }

    #[test]
    fn test_compression_method_default() {
        let method = CompressionMethod::default();
        assert_eq!(method, CompressionMethod::Zstd);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_compression_extension_hint() {
        assert_eq!(CompressionMethod::None.extension_hint(), "");
        assert_eq!(CompressionMethod::Zstd.extension_hint(), ".zst");
        assert_eq!(CompressionMethod::Lz4.extension_hint(), ".lz4");
    }

    #[test]
    fn test_manifest_asset_paths() {
        let mut manifest = PackManifest::new("Test");
        manifest.add_asset(AssetEntry {
            path: "a.txt".into(),
            offset: 0,
            compressed_size: 10,
            uncompressed_size: 10,
            compression: CompressionMethod::None,
            checksum: 0,
        });
        manifest.add_asset(AssetEntry {
            path: "b.txt".into(),
            offset: 0,
            compressed_size: 10,
            uncompressed_size: 10,
            compression: CompressionMethod::None,
            checksum: 0,
        });

        let paths = manifest.asset_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"a.txt"));
        assert!(paths.contains(&"b.txt"));
    }

    #[test]
    fn test_pack_error_display() {
        assert_eq!(format!("{}", PackError::NoAssets), "No assets to pack");
        assert_eq!(format!("{}", PackError::Io("foo".into())), "IO error: foo");
        assert_eq!(format!("{}", PackError::Compression("bar".into())), "Compression error: bar");
    }

    #[test]
    fn test_pack_result_summary() {
        let res = PackResult {
            output_path: PathBuf::from("out.pak"),
            asset_count: 5,
            total_uncompressed_size: 200,
            total_compressed_size: 100,
            compression_ratio: 0.5,
            duration_secs: 1.5,
        };
        let summary = res.summary();
        assert!(summary.contains("Packed 5 assets"));
        assert!(summary.contains("50.0% reduction"));
        assert!(summary.contains("200 bytes"));
        assert!(summary.contains("1.50s"));
    }

    #[test]
    fn test_builder_defaults() {
        let builder = AssetPackBuilder::new("out.pak", "Proj");
        assert_eq!(builder.project_name, "Proj");
        assert_eq!(builder.compression, CompressionMethod::Zstd);
        assert_eq!(builder.assets.len(), 0);
    }
    
    #[test]
    fn test_builder_chaining() {
        let builder = AssetPackBuilder::new("out.pak", "Proj")
            .with_compression(CompressionMethod::Lz4)
            .with_compression_level(5)
            .add_asset("src.txt", "arch.txt");
            
        assert_eq!(builder.compression, CompressionMethod::Lz4);
        assert_eq!(builder.compression_level, 5);
        assert_eq!(builder.assets.len(), 1);
        assert_eq!(builder.assets[0].1, "arch.txt");
    }
}
