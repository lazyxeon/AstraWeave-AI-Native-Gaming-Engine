// Re-export main types for library usage
pub use config::{AssetManifest, HdriAsset, LockEntry, Lockfile, ModelAsset, TextureAsset};
pub use direct_url_provider::DirectUrlProvider;
pub use downloader::{DownloadResult, DownloadTask, Downloader};
pub use kenney_provider::KenneyProvider;
pub use organize::AssetOrganizer;
pub use polyhaven::{PolyHavenClient, ResolvedAsset};
pub use provider::{
    AssetProvider, AssetType, LicenseInfo, ProviderConfig, ProviderRegistry,
    ResolvedAsset as ResolvedAssetV2,
};
pub use summary::FetchSummary;

use std::path::Path;

pub mod config;
pub mod direct_url_provider;
pub mod downloader;
pub mod kenney_provider;
pub mod organize;
pub mod polyhaven;
pub mod polyhaven_provider;
pub mod provider;
pub mod summary;
pub mod unified_config;

/// Runtime asset fetcher for on-demand loading
pub mod ensure_asset {
    use super::*;
    use anyhow::{Context, Result};
    use std::path::PathBuf;

    /// Ensure asset exists locally, fetching if necessary
    pub async fn ensure_asset(manifest_path: &Path, handle: &str) -> Result<Vec<PathBuf>> {
        // Load manifest
        let manifest = AssetManifest::load(manifest_path).context("Failed to load manifest")?;

        let organizer =
            AssetOrganizer::new(manifest.output_dir.clone(), manifest.cache_dir.clone());

        // Check if cached
        if organizer.is_cached(handle).await {
            // Return cached paths
            let lockfile = organizer.load_lockfile().await?;
            if let Some(entry) = lockfile.assets.get(handle) {
                return Ok(entry.paths.values().cloned().collect());
            }
        }

        // Fetch asset
        // Allow test injection via environment variable
        let client = if let Ok(base_url) = std::env::var("POLYHAVEN_BASE_URL") {
            PolyHavenClient::new_with_base_url(&base_url)?
        } else {
            PolyHavenClient::new()?
        };
        let downloader = Downloader::new()?;

        // Try texture
        if let Some(texture) = manifest.textures.get(handle) {
            let resolved = client
                .resolve_texture(&texture.id, &texture.res, &texture.maps)
                .await?;

            let mut downloads = std::collections::HashMap::new();

            for (map_name, url) in &resolved.urls {
                let temp_path = manifest
                    .cache_dir
                    .join(format!("_temp_{}_{}.tmp", handle, map_name));
                let result = downloader.download(url, &temp_path, false).await?;
                downloads.insert(map_name.clone(), result);
            }

            let entry = organizer.organize(handle, &resolved, &downloads).await?;
            return Ok(entry.paths.values().cloned().collect());
        }

        // Try HDRI
        if let Some(hdri) = manifest.hdris.get(handle) {
            let resolved = client.resolve_hdri(&hdri.id, &hdri.res).await?;

            let mut downloads = std::collections::HashMap::new();

            if let Some(url) = resolved.urls.get("hdri") {
                let temp_path = manifest
                    .cache_dir
                    .join(format!("_temp_{}_hdri.tmp", handle));
                let result = downloader.download(url, &temp_path, false).await?;
                downloads.insert("hdri".to_string(), result);
            }

            let entry = organizer.organize(handle, &resolved, &downloads).await?;
            return Ok(entry.paths.values().cloned().collect());
        }

        // Try model
        if let Some(model) = manifest.models.get(handle) {
            let resolved = client
                .resolve_model(&model.id, &model.res, &model.format)
                .await?;

            let mut downloads = std::collections::HashMap::new();

            if let Some(url) = resolved.urls.get("model") {
                let temp_path = manifest
                    .cache_dir
                    .join(format!("_temp_{}_model.tmp", handle));
                let result = downloader.download(url, &temp_path, false).await?;
                downloads.insert("model".to_string(), result);
            }

            let entry = organizer.organize(handle, &resolved, &downloads).await?;
            return Ok(entry.paths.values().cloned().collect());
        }

        Err(anyhow::anyhow!("Asset '{}' not found in manifest", handle))
    }

    /// Check if asset is available locally (without fetching)
    pub async fn is_available(manifest_path: &Path, handle: &str) -> Result<bool> {
        let manifest = AssetManifest::load(manifest_path)?;
        let organizer = AssetOrganizer::new(manifest.output_dir, manifest.cache_dir);

        Ok(organizer.is_cached(handle).await)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_library_exports() {
        // Ensure all types are re-exported
        let _manifest: super::AssetManifest;
        let _client: super::PolyHavenClient;
        let _downloader: super::Downloader;
    }
}
