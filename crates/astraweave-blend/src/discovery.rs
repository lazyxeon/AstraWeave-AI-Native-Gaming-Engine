//! Cross-platform Blender executable discovery.
//!
//! This module provides automatic detection of Blender installations across
//! Windows, macOS, and Linux, with support for user-configured paths.

use crate::error::{BlendError, BlendResult};
use crate::version::BlenderVersion;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use tracing::{debug, info, warn};

/// Configuration for Blender discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderDiscoveryConfig {
    /// User-specified Blender executable path (highest priority).
    pub user_path: Option<PathBuf>,
    /// Whether to search environment PATH.
    pub search_path: bool,
    /// Whether to search common installation directories.
    pub search_common_dirs: bool,
    /// Whether to use platform-specific discovery (registry on Windows, mdfind on macOS).
    pub use_platform_discovery: bool,
    /// Additional custom search paths.
    pub custom_search_paths: Vec<PathBuf>,
}

impl Default for BlenderDiscoveryConfig {
    fn default() -> Self {
        Self {
            user_path: None,
            search_path: true,
            search_common_dirs: true,
            use_platform_discovery: true,
            custom_search_paths: Vec::new(),
        }
    }
}

/// Result of a successful Blender discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderInstallation {
    /// Path to the Blender executable.
    pub executable_path: PathBuf,
    /// Detected Blender version.
    pub version: BlenderVersion,
    /// How the installation was discovered.
    pub discovery_method: DiscoveryMethod,
    /// Installation directory (parent of executable).
    pub install_dir: PathBuf,
}

impl BlenderInstallation {
    /// Checks if this installation meets minimum requirements.
    pub fn is_valid(&self) -> bool {
        self.version.meets_minimum()
    }

    /// Returns capability information for this installation.
    pub fn capabilities(&self) -> crate::version::BlenderCapabilities {
        self.version.capabilities()
    }
}

/// Method used to discover Blender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// User explicitly configured the path.
    UserConfigured,
    /// Found in system PATH.
    SystemPath,
    /// Found in common installation directory.
    CommonDirectory,
    /// Found via Windows Registry.
    WindowsRegistry,
    /// Found via macOS mdfind/Spotlight.
    MacOsSpotlight,
    /// Found in custom search path.
    CustomSearchPath,
}

impl std::fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::UserConfigured => write!(f, "user configured"),
            DiscoveryMethod::SystemPath => write!(f, "system PATH"),
            DiscoveryMethod::CommonDirectory => write!(f, "common installation directory"),
            DiscoveryMethod::WindowsRegistry => write!(f, "Windows Registry"),
            DiscoveryMethod::MacOsSpotlight => write!(f, "macOS Spotlight"),
            DiscoveryMethod::CustomSearchPath => write!(f, "custom search path"),
        }
    }
}

/// Discovers and manages Blender installations.
#[derive(Debug)]
pub struct BlenderDiscovery {
    config: BlenderDiscoveryConfig,
    cached_installation: Option<BlenderInstallation>,
}

impl BlenderDiscovery {
    /// Creates a new BlenderDiscovery with default configuration.
    pub fn new() -> Self {
        Self {
            config: BlenderDiscoveryConfig::default(),
            cached_installation: None,
        }
    }

    /// Creates a new BlenderDiscovery with custom configuration.
    pub fn with_config(config: BlenderDiscoveryConfig) -> Self {
        Self {
            config,
            cached_installation: None,
        }
    }

    /// Sets the user-configured Blender path.
    pub fn set_user_path(&mut self, path: impl Into<PathBuf>) {
        self.config.user_path = Some(path.into());
        self.cached_installation = None; // Invalidate cache
    }

    /// Clears the user-configured path.
    pub fn clear_user_path(&mut self) {
        self.config.user_path = None;
        self.cached_installation = None;
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &BlenderDiscoveryConfig {
        &self.config
    }

    /// Discovers the best available Blender installation.
    ///
    /// Discovery order (highest priority first):
    /// 1. User-configured path
    /// 2. Custom search paths
    /// 3. System PATH
    /// 4. Platform-specific discovery (Registry/mdfind)
    /// 5. Common installation directories
    ///
    /// # Errors
    ///
    /// Returns `BlendError::BlenderNotFound` if no valid installation is found.
    pub async fn discover(&mut self) -> BlendResult<&BlenderInstallation> {
        // Return cached result if available
        if self.cached_installation.is_some() {
            return Ok(self.cached_installation.as_ref().unwrap());
        }

        info!("Starting Blender discovery...");

        // 1. Try user-configured path first
        if let Some(ref user_path) = self.config.user_path {
            debug!("Checking user-configured path: {:?}", user_path);
            match self.validate_executable(user_path, DiscoveryMethod::UserConfigured).await {
                Ok(installation) => {
                    info!("Found Blender at user-configured path: {} (v{})", 
                          installation.executable_path.display(), installation.version);
                    self.cached_installation = Some(installation);
                    return Ok(self.cached_installation.as_ref().unwrap());
                }
                Err(e) => {
                    warn!("User-configured Blender path invalid: {}", e);
                    // Continue with auto-discovery
                }
            }
        }

        // 2. Try custom search paths
        for custom_path in &self.config.custom_search_paths.clone() {
            debug!("Checking custom search path: {:?}", custom_path);
            if let Some(exe) = self.find_executable_in_dir(custom_path) {
                if let Ok(installation) = self.validate_executable(&exe, DiscoveryMethod::CustomSearchPath).await {
                    info!("Found Blender in custom search path: {} (v{})", 
                          installation.executable_path.display(), installation.version);
                    self.cached_installation = Some(installation);
                    return Ok(self.cached_installation.as_ref().unwrap());
                }
            }
        }

        // 3. Try system PATH
        if self.config.search_path {
            debug!("Searching system PATH...");
            if let Some(installation) = self.discover_from_path().await {
                info!("Found Blender in system PATH: {} (v{})", 
                      installation.executable_path.display(), installation.version);
                self.cached_installation = Some(installation);
                return Ok(self.cached_installation.as_ref().unwrap());
            }
        }

        // 4. Try platform-specific discovery
        if self.config.use_platform_discovery {
            debug!("Trying platform-specific discovery...");
            if let Some(installation) = self.discover_platform_specific().await {
                info!("Found Blender via platform discovery: {} (v{})", 
                      installation.executable_path.display(), installation.version);
                self.cached_installation = Some(installation);
                return Ok(self.cached_installation.as_ref().unwrap());
            }
        }

        // 5. Try common installation directories
        if self.config.search_common_dirs {
            debug!("Searching common installation directories...");
            if let Some(installation) = self.discover_common_dirs().await {
                info!("Found Blender in common directory: {} (v{})", 
                      installation.executable_path.display(), installation.version);
                self.cached_installation = Some(installation);
                return Ok(self.cached_installation.as_ref().unwrap());
            }
        }

        // No valid installation found
        Err(BlendError::BlenderNotFound {
            searched_paths: self.collect_searched_paths(),
        })
    }

    /// Discovers Blender synchronously (blocking).
    pub fn discover_sync(&mut self) -> BlendResult<&BlenderInstallation> {
        // Return cached result if available
        if self.cached_installation.is_some() {
            return Ok(self.cached_installation.as_ref().unwrap());
        }

        // Create a runtime if needed, then call discover
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We have a current runtime, use block_in_place to run the async code
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        self.discover().await
                    })
                })?;
            }
            Err(_) => {
                // No runtime, create one
                let rt = tokio::runtime::Runtime::new()
                    .map_err(BlendError::IoError)?;
                rt.block_on(self.discover())?;
            }
        };

        // Re-borrow from cache after discover has populated it
        Ok(self.cached_installation.as_ref().unwrap())
    }

    /// Invalidates the cached installation.
    pub fn invalidate_cache(&mut self) {
        self.cached_installation = None;
    }

    /// Returns the cached installation if available.
    pub fn cached(&self) -> Option<&BlenderInstallation> {
        self.cached_installation.as_ref()
    }

    /// Validates an executable path and returns installation info.
    async fn validate_executable(&self, path: &Path, method: DiscoveryMethod) -> BlendResult<BlenderInstallation> {
        // Check file exists
        if !path.exists() {
            return Err(BlendError::BlenderExecutableNotFound {
                path: path.to_path_buf(),
                reason: "File does not exist".to_string(),
            });
        }

        // Get version
        let version = self.get_version(path).await?;

        // Validate minimum version
        version.validate()?;

        let install_dir = path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(BlenderInstallation {
            executable_path: path.to_path_buf(),
            version,
            discovery_method: method,
            install_dir,
        })
    }

    /// Gets Blender version from executable.
    async fn get_version(&self, executable: &Path) -> BlendResult<BlenderVersion> {
        let output = AsyncCommand::new(executable)
            .arg("--version")
            .output()
            .await
            .map_err(|e| BlendError::BlenderExecutionFailed {
                path: executable.to_path_buf(),
                reason: format!("Failed to run --version: {}", e),
            })?;

        if !output.status.success() {
            return Err(BlendError::BlenderExecutionFailed {
                path: executable.to_path_buf(),
                reason: format!("--version returned non-zero exit code: {}", output.status),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        BlenderVersion::from_version_output(&stdout)
    }

    /// Discovers Blender from system PATH.
    async fn discover_from_path(&self) -> Option<BlenderInstallation> {
        let exe_name = if cfg!(windows) { "blender.exe" } else { "blender" };
        
        let output = Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(exe_name)
            .output()
            .ok()?;

        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path = PathBuf::from(path_str.lines().next()?.trim());
            self.validate_executable(&path, DiscoveryMethod::SystemPath).await.ok()
        } else {
            None
        }
    }

    /// Platform-specific discovery.
    async fn discover_platform_specific(&self) -> Option<BlenderInstallation> {
        #[cfg(target_os = "windows")]
        {
            self.discover_windows_registry().await
        }
        #[cfg(target_os = "macos")]
        {
            self.discover_macos_spotlight().await
        }
        #[cfg(target_os = "linux")]
        {
            None // Linux uses common dirs discovery
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }

    /// Windows Registry discovery.
    #[cfg(target_os = "windows")]
    async fn discover_windows_registry(&self) -> Option<BlenderInstallation> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        // Try different registry locations
        let registry_paths = [
            r"SOFTWARE\BlenderFoundation\Blender",
            r"SOFTWARE\WOW6432Node\BlenderFoundation\Blender",
        ];

        for reg_path in registry_paths {
            if let Ok(blender_key) = hklm.open_subkey(reg_path) {
                // Try to get the install location
                if let Ok(install_path) = blender_key.get_value::<String, _>("") {
                    let exe_path = PathBuf::from(&install_path).join("blender.exe");
                    if let Ok(installation) = self.validate_executable(&exe_path, DiscoveryMethod::WindowsRegistry).await {
                        return Some(installation);
                    }
                }

                // Try versioned subkeys
                for subkey_name in blender_key.enum_keys().filter_map(|k| k.ok()) {
                    if let Ok(version_key) = blender_key.open_subkey(&subkey_name) {
                        if let Ok(install_path) = version_key.get_value::<String, _>("") {
                            let exe_path = PathBuf::from(&install_path).join("blender.exe");
                            if let Ok(installation) = self.validate_executable(&exe_path, DiscoveryMethod::WindowsRegistry).await {
                                return Some(installation);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// macOS Spotlight discovery.
    #[cfg(target_os = "macos")]
    async fn discover_macos_spotlight(&self) -> Option<BlenderInstallation> {
        let output = AsyncCommand::new("mdfind")
            .args(["kMDItemCFBundleIdentifier", "=", "org.blenderfoundation.blender"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let apps = String::from_utf8_lossy(&output.stdout);
            for app_path in apps.lines() {
                let exe_path = PathBuf::from(app_path)
                    .join("Contents")
                    .join("MacOS")
                    .join("Blender");
                if let Ok(installation) = self.validate_executable(&exe_path, DiscoveryMethod::MacOsSpotlight).await {
                    return Some(installation);
                }
            }
        }

        None
    }

    /// Discovers Blender in common installation directories.
    async fn discover_common_dirs(&self) -> Option<BlenderInstallation> {
        let common_paths = self.get_common_paths();

        for dir in common_paths {
            if let Some(exe) = self.find_executable_in_dir(&dir) {
                if let Ok(installation) = self.validate_executable(&exe, DiscoveryMethod::CommonDirectory).await {
                    return Some(installation);
                }
            }
        }

        None
    }

    /// Gets common Blender installation paths for the current platform.
    fn get_common_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // Steam installation
            paths.push(PathBuf::from(r"C:\Program Files\Steam\steamapps\common\Blender"));
            // Blender Foundation installation
            paths.push(PathBuf::from(r"C:\Program Files\Blender Foundation"));
            paths.push(PathBuf::from(r"C:\Program Files (x86)\Blender Foundation"));
            // Portable versions often in user profile
            if let Ok(home) = std::env::var("USERPROFILE") {
                paths.push(PathBuf::from(&home).join("blender"));
                paths.push(PathBuf::from(&home).join("Downloads").join("blender"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/Applications/Blender.app/Contents/MacOS"));
            paths.push(PathBuf::from("/Applications/Blender/Blender.app/Contents/MacOS"));
            if let Ok(home) = std::env::var("HOME") {
                paths.push(PathBuf::from(&home).join("Applications/Blender.app/Contents/MacOS"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            paths.push(PathBuf::from("/usr/bin"));
            paths.push(PathBuf::from("/usr/local/bin"));
            paths.push(PathBuf::from("/snap/bin"));
            paths.push(PathBuf::from("/var/lib/flatpak/exports/bin"));
            if let Ok(home) = std::env::var("HOME") {
                paths.push(PathBuf::from(&home).join(".local/bin"));
                paths.push(PathBuf::from(&home).join(".local/share/flatpak/exports/bin"));
                paths.push(PathBuf::from(&home).join("blender"));
            }
            // Steam on Linux
            if let Ok(home) = std::env::var("HOME") {
                paths.push(PathBuf::from(&home).join(".steam/steam/steamapps/common/Blender"));
            }
        }

        paths
    }

    /// Finds Blender executable in a directory (handles versioned subdirs).
    fn find_executable_in_dir(&self, dir: &Path) -> Option<PathBuf> {
        let exe_name = if cfg!(windows) { "blender.exe" } else { "blender" };

        // Direct executable
        let direct = dir.join(exe_name);
        if direct.exists() {
            return Some(direct);
        }

        // Check versioned subdirectories (e.g., "Blender 4.0", "blender-3.6.0")
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut candidates: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .to_lowercase()
                        .contains("blender")
                })
                .collect();

            // Sort by name (descending) to prefer newer versions
            candidates.sort_by_key(|b| std::cmp::Reverse(b.file_name()));

            for entry in candidates {
                let subdir_exe = entry.path().join(exe_name);
                if subdir_exe.exists() {
                    return Some(subdir_exe);
                }
            }
        }

        None
    }

    /// Collects all searched paths for error reporting.
    fn collect_searched_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(ref user_path) = self.config.user_path {
            paths.push(user_path.clone());
        }

        paths.extend(self.config.custom_search_paths.clone());
        
        if self.config.search_path {
            paths.push(PathBuf::from("$PATH"));
        }

        if self.config.search_common_dirs {
            paths.extend(self.get_common_paths());
        }

        paths
    }
}

impl Default for BlenderDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = BlenderDiscoveryConfig::default();
        assert!(config.user_path.is_none());
        assert!(config.search_path);
        assert!(config.search_common_dirs);
        assert!(config.use_platform_discovery);
    }

    #[test]
    fn test_discovery_method_display() {
        assert_eq!(DiscoveryMethod::UserConfigured.to_string(), "user configured");
        assert_eq!(DiscoveryMethod::SystemPath.to_string(), "system PATH");
        assert_eq!(DiscoveryMethod::WindowsRegistry.to_string(), "Windows Registry");
    }

    #[test]
    fn test_common_paths_not_empty() {
        let discovery = BlenderDiscovery::new();
        let paths = discovery.get_common_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_set_user_path_invalidates_cache() {
        let mut discovery = BlenderDiscovery::new();
        discovery.set_user_path("/fake/path/blender");
        assert!(discovery.cached().is_none());
    }
}
