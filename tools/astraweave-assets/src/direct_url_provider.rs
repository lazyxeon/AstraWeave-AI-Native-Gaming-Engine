// =============================================================================
// Direct URL Provider - Poly Pizza & OpenGameArt Support
// =============================================================================
//
// This provider handles assets from sources that don't have well-documented
// APIs. Instead, users provide direct download URLs with license metadata.
//
// Supported providers:
// - Poly Pizza (poly.pizza) - CC0 3D models
// - OpenGameArt (opengameart.org) - Mixed license audio/sprites/models
// - itch.io (itch.io) - Indie games, assets, tools (CC0, CC-BY, CC-BY-SA)
//
// Key features:
// - Manual URL configuration (user provides direct download link)
// - Strict license validation (CC0, CC-BY, CC-BY-SA only)
// - Attribution generation (ATTRIBUTION.txt per provider)
// - No web scraping (ToS compliant)
//
// =============================================================================

use crate::provider::{
    AssetProvider, AssetType, LicenseInfo, ProviderConfig, ResolvedAsset as ResolvedAssetV2,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;

/// Direct URL provider for manually configured assets
pub struct DirectUrlProvider {
    provider_name: String,
}

impl DirectUrlProvider {
    /// Create provider for Poly Pizza
    pub fn polypizza() -> Self {
        Self {
            provider_name: "polypizza".to_string(),
        }
    }

    /// Create provider for OpenGameArt
    pub fn opengameart() -> Self {
        Self {
            provider_name: "opengameart".to_string(),
        }
    }

    /// Create provider for itch.io
    pub fn itchio() -> Self {
        Self {
            provider_name: "itchio".to_string(),
        }
    }

    /// Validate URL belongs to expected domain
    fn validate_domain(&self, url: &str) -> Result<()> {
        match self.provider_name.as_str() {
            "polypizza" => {
                if !url.contains("poly.pizza") {
                    anyhow::bail!(
                        "Invalid Poly Pizza URL '{}'. Expected domain: poly.pizza",
                        url
                    );
                }
            }
            "opengameart" => {
                if !url.contains("opengameart.org") {
                    anyhow::bail!(
                        "Invalid OpenGameArt URL '{}'. Expected domain: opengameart.org",
                        url
                    );
                }
            }
            "itchio" => {
                // Accept both main domain and CDN
                if !url.contains("itch.io") && !url.contains("img.itch.zone") {
                    anyhow::bail!(
                        "Invalid itch.io URL '{}'. Expected domain: itch.io or img.itch.zone",
                        url
                    );
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Infer asset type from URL and format
    #[allow(dead_code)]
    fn infer_asset_type(format: &str, url: &str) -> Result<AssetType> {
        match format.to_lowercase().as_str() {
            // 3D models
            "glb" | "gltf" | "fbx" | "obj" => Ok(AssetType::Model),

            // Audio
            "ogg" | "wav" | "mp3" | "flac" => Ok(AssetType::Audio),

            // Sprites/Textures
            "png" | "webp" | "jpg" | "jpeg" => {
                // Heuristic: if URL contains "sprite" or "tileset", classify accordingly
                if url.to_lowercase().contains("sprite") {
                    Ok(AssetType::Sprite)
                } else if url.to_lowercase().contains("tileset")
                    || url.to_lowercase().contains("tile")
                {
                    Ok(AssetType::Tileset)
                } else {
                    Ok(AssetType::Texture)
                }
            }

            // HDRIs
            "exr" | "hdr" => Ok(AssetType::Hdri),

            _ => anyhow::bail!("Unsupported file format: {}", format),
        }
    }
}

#[async_trait]
impl AssetProvider for DirectUrlProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }

    async fn resolve(&self, handle: &str, config: &ProviderConfig) -> Result<ResolvedAssetV2> {
        // Validate configuration
        self.validate_config(config)?;

        let url = config
            .url
            .as_ref()
            .context("Missing 'url' field for direct URL provider")?;
        let license_spdx = config
            .license
            .as_ref()
            .context("Missing 'license' field for direct URL provider")?;
        let author = config.author.clone();
        let source_url = config.source_url.clone();
        let format = config
            .format
            .as_ref()
            .context("Missing 'format' field for direct URL provider")?;

        // Validate domain
        self.validate_domain(url)?;

        // Parse license
        let license = LicenseInfo::from_spdx(license_spdx, author, source_url)?;
        license.validate_permissive()?;

        // Infer asset type
        let asset_type = config.asset_type;

        // Build download URLs map
        let mut urls = HashMap::new();
        let key = match asset_type {
            AssetType::Model => "model",
            AssetType::Audio => "audio",
            AssetType::Sprite => "sprite",
            AssetType::Tileset => "tileset",
            AssetType::Texture => "texture",
            AssetType::Hdri => "hdri",
        };
        urls.insert(key.to_string(), url.clone());

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), format.clone());
        if let Some(author) = &license.author {
            metadata.insert("author".to_string(), author.clone());
        }
        if let Some(source) = &license.source_url {
            metadata.insert("source_url".to_string(), source.clone());
        }

        Ok(ResolvedAssetV2 {
            handle: handle.to_string(),
            provider: self.provider_name.clone(),
            asset_type,
            urls,
            license,
            metadata,
        })
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<()> {
        // Check required fields
        if config.url.is_none() {
            anyhow::bail!(
                "Missing required field 'url' for {} asset '{}'",
                self.provider_name,
                config.handle
            );
        }
        if config.license.is_none() {
            anyhow::bail!(
                "Missing required field 'license' for {} asset '{}'",
                self.provider_name,
                config.handle
            );
        }
        if config.format.is_none() {
            anyhow::bail!(
                "Missing required field 'format' for {} asset '{}'",
                self.provider_name,
                config.handle
            );
        }

        // Validate license
        let license_spdx = config.license.as_ref().unwrap();
        let author = config.author.clone();
        let source_url = config.source_url.clone();

        let license =
            LicenseInfo::from_spdx(license_spdx, author, source_url).with_context(|| {
                format!(
                    "Invalid license '{}' for {} asset '{}'",
                    license_spdx, self.provider_name, config.handle
                )
            })?;

        license.validate_permissive().with_context(|| {
            format!(
                "Restrictive license '{}' not allowed for {} asset '{}'",
                license_spdx, self.provider_name, config.handle
            )
        })?;

        // Validate author for attribution licenses
        if license.requires_attribution && config.author.is_none() {
            anyhow::bail!(
                "License '{}' requires 'author' field for {} asset '{}'",
                license_spdx,
                self.provider_name,
                config.handle
            );
        }

        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_polypizza_cc0_model() {
        let provider = DirectUrlProvider::polypizza();

        let config = ProviderConfig {
            provider: "polypizza".to_string(),
            asset_type: AssetType::Model,
            handle: "test_knight".to_string(),
            id: None,
            resolution: None,
            format: Some("glb".to_string()),
            url: Some("https://poly.pizza/files/Low_poly_Knight-9zzjdYXlcwJ.glb".to_string()),
            license: Some("CC0-1.0".to_string()),
            author: Some("Quaternius".to_string()),
            source_url: Some("https://poly.pizza/m/Low_poly_Knight-9zzjdYXlcwJ".to_string()),
        };

        let resolved = provider.resolve("test_knight", &config).await.unwrap();

        assert_eq!(resolved.handle, "test_knight");
        assert_eq!(resolved.provider, "polypizza");
        assert_eq!(resolved.asset_type, AssetType::Model);
        assert_eq!(resolved.license.spdx_id, "CC0-1.0");
        assert_eq!(resolved.license.requires_attribution, false);
        assert!(resolved.urls.contains_key("model"));
    }

    #[tokio::test]
    async fn test_opengameart_cc_by_audio() {
        let provider = DirectUrlProvider::opengameart();

        let config = ProviderConfig {
            provider: "opengameart".to_string(),
            asset_type: AssetType::Audio,
            handle: "test_music".to_string(),
            id: None,
            resolution: None,
            format: Some("ogg".to_string()),
            url: Some("https://opengameart.org/sites/default/files/music.ogg".to_string()),
            license: Some("CC-BY-4.0".to_string()),
            author: Some("TestArtist".to_string()),
            source_url: Some("https://opengameart.org/content/test-music".to_string()),
        };

        let resolved = provider.resolve("test_music", &config).await.unwrap();

        assert_eq!(resolved.handle, "test_music");
        assert_eq!(resolved.provider, "opengameart");
        assert_eq!(resolved.asset_type, AssetType::Audio);
        assert_eq!(resolved.license.spdx_id, "CC-BY-4.0");
        assert_eq!(resolved.license.requires_attribution, true);
        assert_eq!(resolved.license.author, Some("TestArtist".to_string()));
    }

    #[tokio::test]
    async fn test_missing_author_for_attribution_license() {
        let provider = DirectUrlProvider::polypizza();

        let config = ProviderConfig {
            provider: "polypizza".to_string(),
            asset_type: AssetType::Model,
            handle: "test_model".to_string(),
            id: None,
            resolution: None,
            format: Some("glb".to_string()),
            url: Some("https://poly.pizza/files/test.glb".to_string()),
            license: Some("CC-BY-4.0".to_string()),
            author: None, // Missing author for CC-BY!
            source_url: Some("https://poly.pizza/m/test".to_string()),
        };

        // Should fail during validate_config
        let result = provider.validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Check for multiple possible error messages
        assert!(
            err_msg.contains("author") || err_msg.contains("CC-BY") || err_msg.contains("requires"),
            "Expected author-related error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_reject_gpl_license() {
        let provider = DirectUrlProvider::opengameart();

        let config = ProviderConfig {
            provider: "opengameart".to_string(),
            asset_type: AssetType::Audio,
            handle: "test_gpl".to_string(),
            id: None,
            resolution: None,
            format: Some("ogg".to_string()),
            url: Some("https://opengameart.org/sites/default/files/gpl.ogg".to_string()),
            license: Some("GPL-3.0".to_string()),
            author: Some("TestArtist".to_string()),
            source_url: Some("https://opengameart.org/content/test-gpl".to_string()),
        };

        let result = provider.validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Check for license rejection (can be "Invalid license" or "Unsupported" or "not allowed")
        assert!(
            err_msg.contains("license") && err_msg.contains("GPL"),
            "Expected GPL rejection, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_invalid_domain() {
        let provider = DirectUrlProvider::polypizza();

        let config = ProviderConfig {
            provider: "polypizza".to_string(),
            asset_type: AssetType::Model,
            handle: "test_invalid".to_string(),
            id: None,
            resolution: None,
            format: Some("glb".to_string()),
            url: Some("https://example.com/model.glb".to_string()), // Wrong domain!
            license: Some("CC0-1.0".to_string()),
            author: Some("TestArtist".to_string()),
            source_url: Some("https://example.com/model".to_string()),
        };

        let result = provider.resolve("test_invalid", &config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid Poly Pizza URL"));
    }

    // =============================================================================
    // itch.io Tests
    // =============================================================================

    #[tokio::test]
    async fn test_itchio_cc0_sprite() {
        let provider = DirectUrlProvider::itchio();

        let config = ProviderConfig {
            provider: "itchio".to_string(),
            asset_type: AssetType::Sprite,
            handle: "pixel_adventure".to_string(),
            id: None,
            resolution: None,
            format: Some("png".to_string()),
            url: Some("https://img.itch.zone/aW1hZ2UvMTI4L2FydGlzdC5wbmc=".to_string()), // CDN
            license: Some("CC0-1.0".to_string()),
            author: Some("PixelArtist".to_string()),
            source_url: Some("https://pixelartist.itch.io/pixel-adventure".to_string()),
        };

        let resolved = provider.resolve("pixel_adventure", &config).await.unwrap();

        assert_eq!(resolved.handle, "pixel_adventure");
        assert_eq!(resolved.provider, "itchio");
        assert_eq!(resolved.asset_type, AssetType::Sprite);
        assert_eq!(resolved.license.spdx_id, "CC0-1.0");
        assert_eq!(resolved.license.requires_attribution, false);
        assert!(resolved.urls.contains_key("sprite"));
    }

    #[tokio::test]
    async fn test_itchio_cc_by_audio() {
        let provider = DirectUrlProvider::itchio();

        let config = ProviderConfig {
            provider: "itchio".to_string(),
            asset_type: AssetType::Audio,
            handle: "fantasy_music".to_string(),
            id: None,
            resolution: None,
            format: Some("ogg".to_string()),
            url: Some("https://musicartist.itch.io/downloads/fantasy-music-pack.zip".to_string()),
            license: Some("CC-BY-4.0".to_string()),
            author: Some("MusicArtist".to_string()),
            source_url: Some("https://musicartist.itch.io/fantasy-music-pack".to_string()),
        };

        let resolved = provider.resolve("fantasy_music", &config).await.unwrap();

        assert_eq!(resolved.handle, "fantasy_music");
        assert_eq!(resolved.provider, "itchio");
        assert_eq!(resolved.asset_type, AssetType::Audio);
        assert_eq!(resolved.license.spdx_id, "CC-BY-4.0");
        assert_eq!(resolved.license.requires_attribution, true);
        assert_eq!(resolved.license.author, Some("MusicArtist".to_string()));
        assert!(resolved.urls.contains_key("audio"));
    }

    #[tokio::test]
    async fn test_itchio_missing_author_for_cc_by() {
        let provider = DirectUrlProvider::itchio();

        let config = ProviderConfig {
            provider: "itchio".to_string(),
            asset_type: AssetType::Model,
            handle: "test_model".to_string(),
            id: None,
            resolution: None,
            format: Some("glb".to_string()),
            url: Some("https://artist.itch.io/downloads/model.glb".to_string()),
            license: Some("CC-BY-4.0".to_string()),
            author: None, // Missing author for CC-BY!
            source_url: Some("https://artist.itch.io/3d-model".to_string()),
        };

        // Should fail during validate_config
        let result = provider.validate_config(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("author") || err_msg.contains("CC-BY") || err_msg.contains("requires"),
            "Expected author-related error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_itchio_invalid_domain() {
        let provider = DirectUrlProvider::itchio();

        let config = ProviderConfig {
            provider: "itchio".to_string(),
            asset_type: AssetType::Sprite,
            handle: "test_invalid".to_string(),
            id: None,
            resolution: None,
            format: Some("png".to_string()),
            url: Some("https://example.com/sprite.png".to_string()), // Wrong domain!
            license: Some("CC0-1.0".to_string()),
            author: Some("TestArtist".to_string()),
            source_url: Some("https://example.com/sprite".to_string()),
        };

        let result = provider.resolve("test_invalid", &config).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid itch.io URL") || err_msg.contains("itch.io"),
            "Expected domain validation error, got: {}",
            err_msg
        );
    }
}
