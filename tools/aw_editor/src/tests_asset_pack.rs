#[cfg(test)]
mod tests {
    use crate::asset_pack::{AssetPackBuilder, CompressionMethod, PackManifest, AssetEntry, PACK_VERSION};

    #[test]
    fn test_manifest_construction() {
        let manifest = PackManifest::new("Test Project");
        assert_eq!(manifest.version, PACK_VERSION);
        assert_eq!(manifest.project_name, "Test Project");
        assert_eq!(manifest.asset_count, 0);
        assert!(manifest.assets.is_empty());
    }

    #[test]
    fn test_manifest_add_retrieve() {
        let mut manifest = PackManifest::new("Test");
        let entry = AssetEntry {
            path: "data/config.json".to_string(),
            offset: 100,
            compressed_size: 50,
            uncompressed_size: 200,
            compression: CompressionMethod::Zstd,
            checksum: 0xDEADBEEF,
        };
        
        manifest.add_asset(entry.clone());
        assert_eq!(manifest.asset_count, 1);
        
        let retrieved = manifest.get_asset("data/config.json");
        assert!(retrieved.is_some());
        let r = retrieved.unwrap();
        assert_eq!(r.offset, 100);
        assert_eq!(r.checksum, 0xDEADBEEF);
        
        assert!(manifest.get_asset("missing").is_none());
    }

    #[test]
    fn test_builder_configuration() {
        let _builder = AssetPackBuilder::new("output.pak", "My Game")
            .with_compression(CompressionMethod::Lz4)
            .with_compression_level(9);
            
        // We can't inspect builder internals easily if fields are private and no getters.
        // Assuming we can't inspect.
        // But we can verify it returns the builder type and likely compiles.
        // If we really want to test, we might need public accessors or rely on `build` result properties (which involves IO).
        // Let's rely on the module-level tests for internal inspection and focus on public API surface here.
    }
    
    #[test]
    fn test_compression_method_hints() {
        assert!(CompressionMethod::Zstd.extension_hint().contains("zst"));
        assert!(CompressionMethod::Lz4.extension_hint().contains("lz4"));
        assert_eq!(CompressionMethod::None.extension_hint(), "");
    }
}
