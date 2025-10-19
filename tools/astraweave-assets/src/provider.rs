// =============================================================================
// Asset Provider Trait - Multi-Source Support
// =============================================================================
//
// This module defines the trait-based architecture for fetching assets from
// multiple providers (PolyHaven, Poly Pizza, OpenGameArt, etc.)
//
// Key design principles:
// - **License Compliance First**: All providers must track licenses
// - **Provider Isolation**: Assets organized by provider directory
// - **Attribution Automation**: Generate ATTRIBUTION.txt per provider
// - **Free Licenses Only**: CC0, CC-BY, CC-BY-SA supported (NO GPL)
//
// =============================================================================

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// =============================================================================
// Core Types
// =============================================================================

/// Resolved asset with all metadata and download URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedAsset {
    /// Unique handle (user-defined in manifest)
    pub handle: String,

    /// Provider name ("polyhaven", "polypizza", "opengameart")
    pub provider: String,

    /// Asset type (texture, hdri, model, audio, sprite, tileset)
    pub asset_type: AssetType,

    /// Download URLs (map name → URL)
    /// Examples:
    /// - Texture: {"albedo": "https://...", "normal": "https://..."}
    /// - Model: {"model": "https://.../model.glb"}
    /// - Audio: {"audio": "https://.../sound.ogg"}
    pub urls: HashMap<String, String>,

    /// License information (CRITICAL for compliance)
    pub license: LicenseInfo,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Asset type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    /// PBR texture maps (albedo, normal, roughness, metallic, ao, height)
    Texture,

    /// High Dynamic Range Image (environment lighting)
    Hdri,

    /// 3D model (GLB, GLTF, FBX, OBJ)
    Model,

    /// Audio file (OGG, WAV, MP3)
    Audio,

    /// 2D sprite sheet (PNG, WebP)
    Sprite,

    /// Tileset for 2D games (PNG, WebP)
    Tileset,
}

/// License information with SPDX identifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    /// SPDX license identifier (e.g., "CC0-1.0", "CC-BY-4.0", "CC-BY-SA-4.0")
    pub spdx_id: String,

    /// Human-readable license name (e.g., "Creative Commons Zero")
    pub name: String,

    /// Does this license require attribution?
    pub requires_attribution: bool,

    /// Does this license require share-alike?
    pub requires_sharealike: bool,

    /// Author/creator name (required if requires_attribution = true)
    pub author: Option<String>,

    /// Source URL (where asset was obtained)
    pub source_url: Option<String>,

    /// License URL (full license text)
    pub license_url: String,
}

impl LicenseInfo {
    /// Create CC0 license (Public Domain, no attribution required)
    pub fn cc0(author: Option<String>, source_url: Option<String>) -> Self {
        Self {
            spdx_id: "CC0-1.0".to_string(),
            name: "Creative Commons Zero v1.0 Universal".to_string(),
            requires_attribution: false,
            requires_sharealike: false,
            author,
            source_url,
            license_url: "https://creativecommons.org/publicdomain/zero/1.0/".to_string(),
        }
    }

    /// Create CC-BY license (requires attribution)
    pub fn cc_by(version: &str, author: String, source_url: Option<String>) -> Self {
        let spdx_id = format!("CC-BY-{}", version);
        let name = format!("Creative Commons Attribution {} International", version);
        let license_url = format!(
            "https://creativecommons.org/licenses/by/{}/",
            version.replace('.', "")
        );

        Self {
            spdx_id,
            name,
            requires_attribution: true,
            requires_sharealike: false,
            author: Some(author),
            source_url,
            license_url,
        }
    }

    /// Create CC-BY-SA license (requires attribution + share-alike)
    pub fn cc_by_sa(version: &str, author: String, source_url: Option<String>) -> Self {
        let spdx_id = format!("CC-BY-SA-{}", version);
        let name = format!(
            "Creative Commons Attribution-ShareAlike {} International",
            version
        );
        let license_url = format!(
            "https://creativecommons.org/licenses/by-sa/{}/",
            version.replace('.', "")
        );

        Self {
            spdx_id,
            name,
            requires_attribution: true,
            requires_sharealike: true,
            author: Some(author),
            source_url,
            license_url,
        }
    }

    /// Parse SPDX identifier from string (user input validation)
    pub fn from_spdx(
        spdx_id: &str,
        author: Option<String>,
        source_url: Option<String>,
    ) -> Result<Self> {
        match spdx_id {
            "CC0-1.0" => Ok(Self::cc0(author, source_url)),
            "CC-BY-3.0" => Ok(Self::cc_by(
                "3.0",
                author.context("CC-BY requires author")?,
                source_url,
            )),
            "CC-BY-4.0" => Ok(Self::cc_by(
                "4.0",
                author.context("CC-BY requires author")?,
                source_url,
            )),
            "CC-BY-SA-3.0" => Ok(Self::cc_by_sa(
                "3.0",
                author.context("CC-BY-SA requires author")?,
                source_url,
            )),
            "CC-BY-SA-4.0" => Ok(Self::cc_by_sa(
                "4.0",
                author.context("CC-BY-SA requires author")?,
                source_url,
            )),
            _ => anyhow::bail!(
                "Unsupported license '{}'. Only CC0, CC-BY, and CC-BY-SA are allowed.",
                spdx_id
            ),
        }
    }

    /// Validate license is permissive (no GPL, no restrictive licenses)
    pub fn validate_permissive(&self) -> Result<()> {
        if self.spdx_id.contains("GPL") {
            anyhow::bail!(
                "GPL licenses are not supported ({}). Use CC0, CC-BY, or CC-BY-SA instead.",
                self.spdx_id
            );
        }

        if self.spdx_id.contains("NC") {
            anyhow::bail!(
                "NonCommercial licenses are not supported ({}). Use CC0, CC-BY, or CC-BY-SA instead.",
                self.spdx_id
            );
        }

        if self.spdx_id.contains("ND") {
            anyhow::bail!(
                "NoDerivatives licenses are not supported ({}). Use CC0, CC-BY, or CC-BY-SA instead.",
                self.spdx_id
            );
        }

        Ok(())
    }

    /// Generate attribution text for this license
    pub fn attribution_text(&self, asset_handle: &str) -> Option<String> {
        if !self.requires_attribution {
            return None;
        }

        let author = self.author.as_ref()?;
        let source = self
            .source_url
            .as_ref()
            .map(|s| format!(" ({})", s))
            .unwrap_or_default();

        Some(format!(
            "\"{}\" by {}{}\nLicense: {} ({})",
            asset_handle, author, source, self.name, self.license_url
        ))
    }
}

// =============================================================================
// Asset Provider Trait
// =============================================================================

/// Trait for asset providers (PolyHaven, Poly Pizza, OpenGameArt, etc.)
#[async_trait]
pub trait AssetProvider: Send + Sync {
    /// Provider name (e.g., "polyhaven", "polypizza", "opengameart")
    fn name(&self) -> &str;

    /// Resolve asset metadata and download URLs
    ///
    /// # Arguments
    /// - `handle`: User-defined asset handle (e.g., "character_knight")
    /// - `config`: Provider-specific configuration
    ///
    /// # Returns
    /// - `ResolvedAsset` with all metadata and download URLs
    async fn resolve(&self, handle: &str, config: &ProviderConfig) -> Result<ResolvedAsset>;

    /// Validate asset configuration before resolution
    fn validate_config(&self, config: &ProviderConfig) -> Result<()>;

    /// Generate attribution file content for all assets
    fn generate_attribution(&self, assets: &[ResolvedAsset]) -> String {
        let mut lines = vec![
            format!("# Attribution - {}", self.name()),
            String::new(),
            "This directory contains assets from the following sources:".to_string(),
            String::new(),
        ];

        for asset in assets {
            if let Some(attribution) = asset.license.attribution_text(&asset.handle) {
                lines.push(attribution);
                lines.push(String::new());
            }
        }

        lines.push(String::new());
        lines.push("For full license texts, see URLs in each attribution above.".to_string());

        lines.join("\n")
    }
}

// =============================================================================
// Provider Configuration
// =============================================================================

/// Provider-specific configuration (parsed from manifest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name ("polyhaven", "polypizza", "opengameart")
    pub provider: String,

    /// Asset type (texture, hdri, model, audio, sprite, tileset)
    #[serde(rename = "type")]
    pub asset_type: AssetType,

    /// Asset handle (user-defined, e.g., "character_knight")
    pub handle: String,

    // =========================================================================
    // PolyHaven-specific fields
    // =========================================================================
    /// PolyHaven asset ID (e.g., "aerial_rocks_02")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Texture resolution ("1k", "2k", "4k", "8k")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,

    /// File format ("png", "jpg", "exr", "hdr", "glb")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    // =========================================================================
    // Direct URL fields (Poly Pizza, OpenGameArt)
    // =========================================================================
    /// Direct download URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// License SPDX identifier (e.g., "CC0-1.0", "CC-BY-4.0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Author/creator name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Source URL (original page where asset was found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

// =============================================================================
// Provider Registry
// =============================================================================

/// Registry of all available providers
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn AssetProvider>>,
}

impl ProviderRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider
    pub fn register(&mut self, provider: Box<dyn AssetProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    /// Get provider by name
    pub fn get(&self, name: &str) -> Result<&dyn AssetProvider> {
        self.providers
            .get(name)
            .map(|p| p.as_ref())
            .context(format!("Unknown provider: {}", name))
    }

    /// List all registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Attribution File Generator
// =============================================================================

/// Generate ATTRIBUTION.txt for a provider directory
pub fn generate_attribution_file(
    provider_name: &str,
    assets: &[ResolvedAsset],
    output_path: &Path,
) -> Result<()> {
    use std::fs;

    let mut lines = vec![
        format!("# Attribution - {}", provider_name.to_uppercase()),
        "=".repeat(80),
        String::new(),
        format!(
            "This directory contains {} assets from {}:",
            assets.len(),
            provider_name
        ),
        String::new(),
    ];

    // Group by license type
    let mut by_license: HashMap<String, Vec<&ResolvedAsset>> = HashMap::new();
    for asset in assets {
        by_license
            .entry(asset.license.spdx_id.clone())
            .or_default()
            .push(asset);
    }

    // Write summary
    lines.push("## License Summary".to_string());
    lines.push(String::new());
    for (spdx_id, assets_list) in &by_license {
        lines.push(format!("- {}: {} assets", spdx_id, assets_list.len()));
    }
    lines.push(String::new());
    lines.push("=".repeat(80));
    lines.push(String::new());

    // Write detailed attributions
    lines.push("## Detailed Attributions".to_string());
    lines.push(String::new());

    for asset in assets {
        lines.push(format!("### {}", asset.handle));
        lines.push(String::new());

        if let Some(attribution) = asset.license.attribution_text(&asset.handle) {
            lines.push(attribution);
        } else {
            lines.push(format!("License: {} (Public Domain)", asset.license.name));
            if let Some(source) = &asset.license.source_url {
                lines.push(format!("Source: {}", source));
            }
        }

        lines.push(String::new());
        lines.push("-".repeat(80));
        lines.push(String::new());
    }

    // Write footer
    lines.push(String::new());
    lines.push("For full license texts, see URLs above.".to_string());
    lines.push(format!("Generated: {}", chrono::Utc::now().to_rfc3339()));

    let content = lines.join("\n");
    fs::write(output_path, content)
        .with_context(|| format!("Failed to write attribution file: {}", output_path.display()))?;

    Ok(())
}
