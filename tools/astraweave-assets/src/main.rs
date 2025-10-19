// =============================================================================
// AstraWeave Multi-Provider Asset Fetcher - Main CLI
// =============================================================================

use astraweave_assets::direct_url_provider::DirectUrlProvider;
use astraweave_assets::downloader::{Downloader, DownloadResult};
use astraweave_assets::kenney_provider::KenneyProvider;
use astraweave_assets::organize::AssetOrganizer;
use astraweave_assets::polyhaven_provider::PolyHavenProvider;
use astraweave_assets::provider::{
    generate_attribution_file, AssetType, ProviderRegistry, ResolvedAsset,
};
use astraweave_assets::summary::FetchSummary;
use astraweave_assets::unified_config::{UnifiedAssetEntry, UnifiedManifest};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "astraweave-assets")]
#[command(about = "Multi-provider asset fetcher (PolyHaven, Poly Pizza, OpenGameArt, itch.io, Kenney.nl)")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch assets from manifest
    Fetch {
        /// Path to manifest file (unified format)
        #[arg(long, default_value = "assets/asset_manifest.toml")]
        manifest: PathBuf,

        /// Filter by provider (polyhaven, polypizza, opengameart, itchio, kenney)
        #[arg(long)]
        provider: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Skip progress bars
        #[arg(long)]
        quiet: bool,
    },

    /// Regenerate attribution files
    RegenerateAttributions {
        /// Path to manifest file
        #[arg(long, default_value = "assets/asset_manifest.toml")]
        manifest: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch {
            manifest,
            provider,
            json,
            quiet,
        } => {
            fetch_command(&manifest, provider.as_deref(), json, quiet).await?;
        }
        Commands::RegenerateAttributions { manifest } => {
            regenerate_attributions_command(&manifest).await?;
        }
    }

    Ok(())
}

/// Fetch assets from unified manifest
async fn fetch_command(
    manifest_path: &Path,
    provider_filter: Option<&str>,
    output_json: bool,
    quiet: bool,
) -> Result<()> {
    // Load manifest
    let manifest = UnifiedManifest::load(manifest_path)
        .context("Failed to load manifest")?;

    if !quiet {
        println!("🚀 AstraWeave Multi-Provider Asset Fetcher");
        println!("📋 Manifest: {}", manifest_path.display());
        println!("📁 Output: {}", manifest.output_dir.display());
        if let Some(filter) = provider_filter {
            println!("🔍 Provider Filter: {}", filter);
        }
        println!();
    }

    // Initialize provider registry
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(PolyHavenProvider::new()?));
    registry.register(Box::new(DirectUrlProvider::polypizza()));
    registry.register(Box::new(DirectUrlProvider::opengameart()));
    registry.register(Box::new(DirectUrlProvider::itchio()));
    registry.register(Box::new(KenneyProvider::new()));

    // Initialize downloader and organizer
    let downloader = Downloader::new()?;
    let organizer = AssetOrganizer::new(manifest.output_dir.clone(), manifest.cache_dir.clone());

    let mut summary = FetchSummary::new();
    let mut resolved_by_provider: HashMap<String, Vec<ResolvedAsset>> = HashMap::new();

    // Filter assets by provider if specified
    let assets_to_fetch: Vec<_> = if let Some(filter) = provider_filter {
        manifest
            .assets
            .iter()
            .filter(|a| a.provider == filter)
            .collect()
    } else {
        manifest.assets.iter().collect()
    };

    if assets_to_fetch.is_empty() {
        if !quiet {
            println!("⚠️  No assets to fetch (check provider filter or manifest)");
        }
        return Ok(());
    }

    // Fetch each asset
    for asset_entry in assets_to_fetch {
        let provider = registry.get(&asset_entry.provider)
            .with_context(|| format!("Unknown provider: {}", asset_entry.provider))?;

        if !quiet {
            let icon = match asset_entry.asset_type {
                AssetType::Texture => "🖼️ ",
                AssetType::Hdri => "🌄",
                AssetType::Model => "🎨",
                AssetType::Audio => "🔊",
                AssetType::Sprite => "🖌️ ",
                AssetType::Tileset => "🗺️ ",
            };
            println!(
                "{} Fetching {}: {} ({})",
                icon, asset_entry.asset_type_str(), asset_entry.handle, asset_entry.provider
            );
        }

        // Convert to ProviderConfig
        let config = UnifiedManifest::to_provider_config(asset_entry);

        // Resolve asset
        match provider.resolve(&asset_entry.handle, &config).await {
            Ok(resolved) => {
                // Check cache first
                if organizer.is_cached(&asset_entry.handle).await {
                    if !quiet {
                        println!("   💾 Using cached version");
                    }
                    // Add to resolved list for attribution
                    resolved_by_provider
                        .entry(asset_entry.provider.clone())
                        .or_default()
                        .push(resolved.clone());

                    // Load from lockfile for summary
                    if let Ok(lockfile) = organizer.load_lockfile().await {
                        if let Some(entry) = lockfile.assets.get(&asset_entry.handle) {
                            summary.add_downloaded(entry);
                        }
                    }
                    continue;
                }

                // Download files
                match download_asset(&downloader, &resolved, &manifest.cache_dir, !quiet).await {
                    Ok(downloads) => {
                        // Organize files
                        match organizer
                            .organize_v2(&asset_entry.handle, &resolved, &downloads)
                            .await
                        {
                            Ok(entry) => {
                                summary.add_downloaded(&entry);
                                resolved_by_provider
                                    .entry(asset_entry.provider.clone())
                                    .or_default()
                                    .push(resolved);

                                if !quiet {
                                    println!("   ✅ Downloaded {} files", entry.paths.len());
                                }
                            }
                            Err(e) => {
                                summary.add_failed(
                                    asset_entry.handle.clone(),
                                    asset_entry.provider.clone(),
                                    format!("{:?}", asset_entry.asset_type),
                                    e.to_string(),
                                );
                                eprintln!("   ❌ Failed to organize: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        summary.add_failed(
                            asset_entry.handle.clone(),
                            asset_entry.provider.clone(),
                            format!("{:?}", asset_entry.asset_type),
                            e.to_string(),
                        );
                        eprintln!("   ❌ Failed to download: {}", e);
                    }
                }
            }
            Err(e) => {
                summary.add_failed(
                    asset_entry.handle.clone(),
                    asset_entry.provider.clone(),
                    format!("{:?}", asset_entry.asset_type),
                    e.to_string(),
                );
                eprintln!("   ❌ Failed to resolve: {}", e);
            }
        }
    }

    // Generate attribution files
    if !quiet {
        println!();
        println!("📝 Generating attribution files...");
    }

    for (provider_name, assets) in &resolved_by_provider {
        let provider_dir = manifest.output_dir.join(provider_name);
        let attribution_path = provider_dir.join("ATTRIBUTION.txt");

        if let Err(e) = generate_attribution_file(provider_name, assets, &attribution_path) {
            eprintln!(
                "   ⚠️  Failed to generate attribution for {}: {}",
                provider_name, e
            );
        } else if !quiet {
            println!("   ✅ Generated attribution for {}", provider_name);
        }
    }

    // Output results
    if output_json {
        println!("{}", summary.to_json()?);
    } else if !quiet {
        println!();
        summary.print_table();

        // License summary
        println!();
        println!("⚖️  License Summary:");
        let license_counts = count_licenses(&resolved_by_provider);
        for (spdx_id, count) in license_counts {
            println!("   {} - {} assets", spdx_id, count);
        }
    }

    Ok(())
}

/// Download all files for an asset
async fn download_asset(
    downloader: &Downloader,
    resolved: &ResolvedAsset,
    cache_dir: &Path,
    show_progress: bool,
) -> Result<HashMap<String, DownloadResult>> {
    use astraweave_assets::downloader::DownloadTask;

    // Build tasks for parallel download
    let tasks: Vec<DownloadTask> = resolved
        .urls
        .iter()
        .map(|(key, url)| DownloadTask {
            url: url.clone(),
            dest_path: cache_dir.join(format!("_temp_{}_{}.tmp", resolved.handle, key)),
            key: key.clone(),
        })
        .collect();

    // Download in parallel
    let results = downloader.download_parallel(tasks, show_progress).await?;

    // Convert results to HashMap, propagating errors
    let mut downloads = HashMap::new();
    for (key, result) in results {
        downloads.insert(key, result?);
    }

    Ok(downloads)
}

/// Count assets by license
fn count_licenses(
    resolved_by_provider: &HashMap<String, Vec<ResolvedAsset>>,
) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for assets in resolved_by_provider.values() {
        for asset in assets {
            *counts.entry(asset.license.spdx_id.clone()).or_insert(0) += 1;
        }
    }

    counts
}

/// Regenerate attribution files from lockfile
async fn regenerate_attributions_command(manifest_path: &Path) -> Result<()> {
    println!("📝 Regenerating attribution files...");

    let manifest = UnifiedManifest::load(manifest_path)?;
    let organizer = AssetOrganizer::new(manifest.output_dir.clone(), manifest.cache_dir.clone());

    // Load lockfile
    let lockfile = organizer.load_lockfile().await
        .context("Failed to load lockfile. Run 'fetch' first.")?;

    // Group assets by provider
    let mut by_provider: HashMap<String, Vec<ResolvedAsset>> = HashMap::new();

    // Initialize provider registry to resolve cached assets
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(PolyHavenProvider::new()?));
    registry.register(Box::new(DirectUrlProvider::polypizza()));
    registry.register(Box::new(DirectUrlProvider::opengameart()));
    registry.register(Box::new(DirectUrlProvider::itchio()));
    registry.register(Box::new(KenneyProvider::new()));

    // Re-resolve each asset from manifest
    for asset_entry in &manifest.assets {
        if lockfile.assets.contains_key(&asset_entry.handle) {
            let provider = registry.get(&asset_entry.provider)?;
            let config = UnifiedManifest::to_provider_config(asset_entry);

            if let Ok(resolved) = provider.resolve(&asset_entry.handle, &config).await {
                by_provider
                    .entry(asset_entry.provider.clone())
                    .or_default()
                    .push(resolved);
            }
        }
    }

    // Generate attribution files
    for (provider_name, assets) in &by_provider {
        let provider_dir = manifest.output_dir.join(provider_name);
        let attribution_path = provider_dir.join("ATTRIBUTION.txt");

        generate_attribution_file(provider_name, assets, &attribution_path)?;
        println!("   ✅ Generated attribution for {}", provider_name);
    }

    println!("✅ Attribution files regenerated successfully!");
    Ok(())
}

// Helper trait for asset type display
trait AssetTypeDisplay {
    fn asset_type_str(&self) -> &str;
}

impl AssetTypeDisplay for UnifiedAssetEntry {
    fn asset_type_str(&self) -> &str {
        match self.asset_type {
            AssetType::Texture => "texture",
            AssetType::Hdri => "HDRI",
            AssetType::Model => "model",
            AssetType::Audio => "audio",
            AssetType::Sprite => "sprite",
            AssetType::Tileset => "tileset",
        }
    }
}
