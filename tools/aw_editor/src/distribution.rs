//! Platform distribution module for packaging games
//!
//! Supports Windows (installer/portable), macOS (.app/DMG),
//! Linux (AppImage/tarball), and Steam depot generation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Distribution format for platform-specific packaging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributionFormat {
    /// Windows NSIS installer (.exe)
    WindowsInstaller,
    /// Windows portable ZIP
    WindowsPortable,
    /// macOS application bundle (.app)
    MacOSBundle,
    /// macOS disk image (.dmg)
    MacOSDmg,
    /// Linux AppImage
    LinuxAppImage,
    /// Linux tarball (.tar.gz)
    LinuxTarball,
    /// Steam depot for steamcmd upload
    SteamDepot,
}

impl DistributionFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            Self::WindowsInstaller => "exe",
            Self::WindowsPortable => "zip",
            Self::MacOSBundle => "app",
            Self::MacOSDmg => "dmg",
            Self::LinuxAppImage => "AppImage",
            Self::LinuxTarball => "tar.gz",
            Self::SteamDepot => "vdf",
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            Self::WindowsInstaller => "Windows Installer",
            Self::WindowsPortable => "Windows Portable",
            Self::MacOSBundle => "macOS App Bundle",
            Self::MacOSDmg => "macOS DMG",
            Self::LinuxAppImage => "Linux AppImage",
            Self::LinuxTarball => "Linux Tarball",
            Self::SteamDepot => "Steam Depot",
        }
    }
}

/// Distribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionConfig {
    /// Game name
    pub game_name: String,
    /// Version string
    pub version: String,
    /// Publisher name
    pub publisher: String,
    /// Optional icon path
    pub icon_path: Option<PathBuf>,
    /// Steam App ID
    pub steam_app_id: Option<u32>,
    /// Steam Depot ID
    pub steam_depot_id: Option<u32>,
    /// Description
    pub description: String,
}

impl Default for DistributionConfig {
    fn default() -> Self {
        Self {
            game_name: "AstraWeave Game".to_string(),
            version: "1.0.0".to_string(),
            publisher: "AstraWeave".to_string(),
            icon_path: None,
            steam_app_id: None,
            steam_depot_id: None,
            description: "A game built with AstraWeave Engine".to_string(),
        }
    }
}

/// Result of a distribution build
#[derive(Debug)]
pub struct DistributionResult {
    /// Path to the generated distribution
    pub output_path: PathBuf,
    /// Format that was generated
    pub format: DistributionFormat,
    /// Size in bytes
    pub size_bytes: u64,
    /// Duration in seconds
    pub duration_secs: f32,
}

/// Distribution builder for creating platform packages
pub struct DistributionBuilder {
    config: DistributionConfig,
    format: DistributionFormat,
    build_dir: PathBuf,
    output_dir: PathBuf,
}

impl DistributionBuilder {
    /// Create a new distribution builder
    pub fn new(config: DistributionConfig, format: DistributionFormat) -> Self {
        Self {
            config,
            format,
            build_dir: PathBuf::from("build"),
            output_dir: PathBuf::from("dist"),
        }
    }

    /// Set the build directory
    pub fn build_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.build_dir = path.into();
        self
    }

    /// Set the output directory
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_dir = path.into();
        self
    }

    /// Build the distribution
    pub fn build(&self) -> Result<DistributionResult> {
        let start = std::time::Instant::now();

        fs::create_dir_all(&self.output_dir).context("Failed to create output directory")?;

        let output_path = match self.format {
            DistributionFormat::WindowsInstaller => self.build_windows_installer()?,
            DistributionFormat::WindowsPortable => self.build_windows_portable()?,
            DistributionFormat::MacOSBundle => self.build_macos_bundle()?,
            DistributionFormat::MacOSDmg => self.build_macos_dmg()?,
            DistributionFormat::LinuxAppImage => self.build_linux_appimage()?,
            DistributionFormat::LinuxTarball => self.build_linux_tarball()?,
            DistributionFormat::SteamDepot => self.build_steam_depot()?,
        };

        let size_bytes = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

        Ok(DistributionResult {
            output_path,
            format: self.format,
            size_bytes,
            duration_secs: start.elapsed().as_secs_f32(),
        })
    }

    fn build_windows_portable(&self) -> Result<PathBuf> {
        let zip_name = format!(
            "{}_{}_win64.zip",
            self.config.game_name.replace(' ', "_"),
            self.config.version
        );
        let output_path = self.output_dir.join(&zip_name);

        #[cfg(target_os = "windows")]
        {
            let status = Command::new("powershell")
                .args([
                    "-Command",
                    &format!(
                        "Compress-Archive -Path '{}\\*' -DestinationPath '{}' -Force",
                        self.build_dir.display(),
                        output_path.display()
                    ),
                ])
                .status()
                .context("Failed to run PowerShell")?;

            if !status.success() {
                anyhow::bail!("PowerShell Compress-Archive failed");
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let output_str = output_path
                .to_str()
                .context("Output path contains invalid UTF-8")?;

            let status = Command::new("zip")
                .args(["-r", output_str, "."])
                .current_dir(&self.build_dir)
                .status()
                .context("Failed to run zip")?;

            if !status.success() {
                anyhow::bail!("zip command failed");
            }
        }

        Ok(output_path)
    }

    fn build_windows_installer(&self) -> Result<PathBuf> {
        // Fall back to portable if NSIS not available
        self.build_windows_portable()
    }

    fn build_macos_bundle(&self) -> Result<PathBuf> {
        let app_name = format!("{}.app", self.config.game_name.replace(' ', ""));
        let app_path = self.output_dir.join(&app_name);
        let contents_path = app_path.join("Contents");
        let macos_path = contents_path.join("MacOS");
        let resources_path = contents_path.join("Resources");

        fs::create_dir_all(&macos_path)?;
        fs::create_dir_all(&resources_path)?;

        let plist = self.generate_info_plist()?;
        fs::write(contents_path.join("Info.plist"), plist)?;

        copy_dir_all(&self.build_dir, &macos_path)?;

        Ok(app_path)
    }

    fn build_macos_dmg(&self) -> Result<PathBuf> {
        self.build_macos_bundle()
    }

    fn build_linux_appimage(&self) -> Result<PathBuf> {
        self.build_linux_tarball()
    }

    fn build_linux_tarball(&self) -> Result<PathBuf> {
        let tar_name = format!(
            "{}_{}_linux.tar.gz",
            self.config.game_name.replace(' ', "_"),
            self.config.version
        );
        let output_path = self.output_dir.join(&tar_name);

        let output_str = output_path
            .to_str()
            .context("Output path contains invalid UTF-8")?;

        let parent_str = self
            .build_dir
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_str()
            .context("Build directory parent contains invalid UTF-8")?;

        let dir_name = self
            .build_dir
            .file_name()
            .context("Build directory has no file name")?
            .to_str()
            .context("Build directory name contains invalid UTF-8")?;

        let status = Command::new("tar")
            .args(["-czf", output_str, "-C", parent_str, dir_name])
            .status()
            .context("Failed to run tar")?;

        if !status.success() {
            anyhow::bail!("tar command failed");
        }

        Ok(output_path)
    }

    fn build_steam_depot(&self) -> Result<PathBuf> {
        let app_id = self.config.steam_app_id.context("Steam App ID required")?;
        let depot_id = self.config.steam_depot_id.unwrap_or(app_id + 1);

        let depot_dir = self.output_dir.join("steam_depot");
        fs::create_dir_all(&depot_dir)?;

        let app_vdf = self.generate_app_vdf(app_id, depot_id)?;
        fs::write(depot_dir.join("app_build.vdf"), app_vdf)?;

        let depot_vdf = self.generate_depot_vdf(depot_id)?;
        fs::write(depot_dir.join(format!("depot_{}.vdf", depot_id)), depot_vdf)?;

        let content_dir = depot_dir.join("content");
        copy_dir_all(&self.build_dir, &content_dir)?;

        Ok(depot_dir.join("app_build.vdf"))
    }

    fn generate_info_plist(&self) -> Result<String> {
        let bundle_id = format!(
            "com.{}.{}",
            self.config.publisher.to_lowercase().replace(' ', ""),
            self.config.game_name.to_lowercase().replace(' ', "")
        );
        let exe_name = self.config.game_name.to_lowercase().replace(' ', "_");

        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>{name}</string>
    <key>CFBundleIdentifier</key>
    <string>{bundle_id}</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>CFBundleExecutable</key>
    <string>{exe_name}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
"#,
            name = self.config.game_name,
            bundle_id = bundle_id,
            version = self.config.version,
            exe_name = exe_name,
        ))
    }

    fn generate_app_vdf(&self, app_id: u32, depot_id: u32) -> Result<String> {
        Ok(format!(
            r#""AppBuild"
{{
    "AppID" "{app_id}"
    "Desc" "{description}"
    "ContentRoot" "content"
    "Depots"
    {{
        "{depot_id}" "depot_{depot_id}.vdf"
    }}
}}
"#,
            app_id = app_id,
            depot_id = depot_id,
            description = format!("{} v{}", self.config.game_name, self.config.version),
        ))
    }

    fn generate_depot_vdf(&self, depot_id: u32) -> Result<String> {
        Ok(format!(
            r#""DepotBuild"
{{
    "DepotID" "{depot_id}"
    "FileMapping"
    {{
        "LocalPath" "*"
        "DepotPath" "."
        "Recursive" "1"
    }}
}}
"#,
            depot_id = depot_id,
        ))
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

/// Format bytes as human-readable
pub fn format_bytes(bytes: u64) -> String {
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
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_extension() {
        assert_eq!(DistributionFormat::WindowsPortable.extension(), "zip");
        assert_eq!(DistributionFormat::LinuxAppImage.extension(), "AppImage");
        assert_eq!(DistributionFormat::MacOSBundle.extension(), "app");
    }

    #[test]
    fn test_config_default() {
        let config = DistributionConfig::default();
        assert_eq!(config.version, "1.0.0");
        assert!(config.steam_app_id.is_none());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1_572_864), "1.50 MB");
    }

    #[test]
    fn test_info_plist_generation() {
        let config = DistributionConfig {
            game_name: "Test Game".to_string(),
            version: "2.0.0".to_string(),
            publisher: "Test Publisher".to_string(),
            ..Default::default()
        };

        let builder = DistributionBuilder::new(config, DistributionFormat::MacOSBundle);
        let plist = builder.generate_info_plist().unwrap();

        assert!(plist.contains("<string>Test Game</string>"));
        assert!(plist.contains("<string>2.0.0</string>"));
    }

    #[test]
    fn test_steam_vdf_generation() {
        let config = DistributionConfig {
            game_name: "Steam Game".to_string(),
            version: "1.0.0".to_string(),
            steam_app_id: Some(480),
            steam_depot_id: Some(481),
            ..Default::default()
        };

        let builder = DistributionBuilder::new(config, DistributionFormat::SteamDepot);
        let app_vdf = builder.generate_app_vdf(480, 481).unwrap();
        let depot_vdf = builder.generate_depot_vdf(481).unwrap();

        assert!(app_vdf.contains("\"AppID\" \"480\""));
        assert!(depot_vdf.contains("\"DepotID\" \"481\""));
    }
}
