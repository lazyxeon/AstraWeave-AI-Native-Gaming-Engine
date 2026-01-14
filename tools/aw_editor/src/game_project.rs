//! Game Project Configuration
//!
//! This module defines the game project manifest structure that describes
//! a complete game project. The manifest is stored as `game.toml` in the
//! project root and contains all metadata needed for building and shipping.
//!
//! # Example game.toml
//!
//! ```toml
//! [project]
//! name = "My Awesome Game"
//! version = "1.0.0"
//! author = "Game Studio"
//! description = "An epic adventure game"
//!
//! [build]
//! entry_scene = "scenes/main_menu.scene"
//! default_target = "windows"
//!
//! [assets]
//! include = ["assets/**/*", "content/**/*"]
//! exclude = ["**/*.psd", "**/*.blend"]
//! ```

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Game project manifest - the central configuration for a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProject {
    /// Project metadata
    pub project: ProjectMetadata,
    /// Build configuration
    pub build: BuildSettings,
    /// Asset configuration
    #[serde(default)]
    pub assets: AssetSettings,
    /// Platform-specific overrides
    #[serde(default)]
    pub platforms: PlatformOverrides,
}

/// Project metadata (name, version, author, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Game name (displayed in window title, store pages)
    pub name: String,
    /// Semantic version (e.g., "1.0.0")
    pub version: String,
    /// Author or studio name
    #[serde(default)]
    pub author: String,
    /// Short description
    #[serde(default)]
    pub description: String,
    /// Icon path (relative to project root)
    #[serde(default)]
    pub icon: Option<PathBuf>,
    /// Unique identifier for the game (e.g., com.studio.gamename)
    #[serde(default)]
    pub identifier: Option<String>,
}

/// Build configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSettings {
    /// Entry scene to load on game start
    pub entry_scene: PathBuf,
    /// Default build target platform
    #[serde(default = "default_target")]
    pub default_target: String,
    /// Default build profile
    #[serde(default = "default_profile")]
    pub default_profile: String,
    /// Output directory for builds
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    /// Additional cargo features to enable
    #[serde(default)]
    pub features: Vec<String>,
}

fn default_target() -> String {
    "windows".to_string()
}

fn default_profile() -> String {
    "release".to_string()
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("builds")
}

/// Asset packaging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSettings {
    /// Glob patterns for assets to include
    #[serde(default = "default_include_patterns")]
    pub include: Vec<String>,
    /// Glob patterns for assets to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Enable asset compression
    #[serde(default = "default_true")]
    pub compress: bool,
    /// Compression level (1-22 for zstd)
    #[serde(default = "default_compression_level")]
    pub compression_level: u8,
}

fn default_include_patterns() -> Vec<String> {
    vec![
        "assets/**/*".to_string(),
        "content/**/*".to_string(),
        "scenes/**/*".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

fn default_compression_level() -> u8 {
    3
}

/// Platform-specific build overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformOverrides {
    #[serde(default)]
    pub windows: Option<PlatformConfig>,
    #[serde(default)]
    pub linux: Option<PlatformConfig>,
    #[serde(default)]
    pub macos: Option<PlatformConfig>,
    #[serde(default)]
    pub web: Option<PlatformConfig>,
}

/// Configuration for a specific platform
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformConfig {
    /// Platform-specific icon
    pub icon: Option<PathBuf>,
    /// Additional features for this platform
    #[serde(default)]
    pub features: Vec<String>,
    /// Platform-specific asset overrides
    #[serde(default)]
    pub asset_overrides: Vec<AssetOverride>,
}

/// Override for specific assets on a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetOverride {
    /// Original asset path pattern
    pub pattern: String,
    /// Replacement path or processing action
    pub replacement: String,
}

impl GameProject {
    /// Load a game project from a TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, GameProjectError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| GameProjectError::IoError(e.to_string()))?;

        toml::from_str(&content).map_err(|e| GameProjectError::ParseError(e.to_string()))
    }

    /// Save the game project to a TOML file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), GameProjectError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| GameProjectError::SerializeError(e.to_string()))?;

        std::fs::write(path.as_ref(), content).map_err(|e| GameProjectError::IoError(e.to_string()))
    }

    /// Create a new game project with default settings
    pub fn new(name: &str, entry_scene: &str) -> Self {
        Self {
            project: ProjectMetadata {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                author: String::new(),
                description: String::new(),
                icon: None,
                identifier: None,
            },
            build: BuildSettings {
                entry_scene: PathBuf::from(entry_scene),
                default_target: default_target(),
                default_profile: default_profile(),
                output_dir: default_output_dir(),
                features: Vec::new(),
            },
            assets: AssetSettings::default(),
            platforms: PlatformOverrides::default(),
        }
    }

    /// Find game.toml in the current directory or parent directories
    pub fn find_project_file() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        loop {
            let manifest = current.join("game.toml");
            if manifest.exists() {
                return Some(manifest);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Get the project root directory (parent of game.toml)
    pub fn project_root(&self, manifest_path: &Path) -> PathBuf {
        manifest_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Validate the project configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.project.name.is_empty() {
            errors.push("Project name is required".to_string());
        }

        if self.build.entry_scene.as_os_str().is_empty() {
            errors.push("Entry scene is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for GameProject {
    fn default() -> Self {
        Self::new("Untitled Game", "scenes/main.scene")
    }
}

impl Default for AssetSettings {
    fn default() -> Self {
        Self {
            include: default_include_patterns(),
            exclude: Vec::new(),
            compress: true,
            compression_level: default_compression_level(),
        }
    }
}

/// Errors that can occur when working with game projects
#[derive(Debug, Clone)]
pub enum GameProjectError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    ValidationError(Vec<String>),
}

impl std::fmt::Display for GameProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::SerializeError(e) => write!(f, "Serialize error: {}", e),
            Self::ValidationError(errors) => write!(f, "Validation errors: {:?}", errors),
        }
    }
}

impl std::error::Error for GameProjectError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_project() {
        let project = GameProject::default();
        assert_eq!(project.project.name, "Untitled Game");
        assert_eq!(
            project.build.entry_scene,
            PathBuf::from("scenes/main.scene")
        );
    }

    #[test]
    fn test_new_project() {
        let project = GameProject::new("My Game", "levels/intro.scene");
        assert_eq!(project.project.name, "My Game");
        assert_eq!(
            project.build.entry_scene,
            PathBuf::from("levels/intro.scene")
        );
    }

    #[test]
    fn test_validate_empty_name() {
        let mut project = GameProject::default();
        project.project.name = String::new();

        let result = project.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains(&"Project name is required".to_string()));
    }

    #[test]
    fn test_serialize_deserialize() {
        let project = GameProject::new("Test Game", "test.scene");
        let toml = toml::to_string(&project).unwrap();
        let parsed: GameProject = toml::from_str(&toml).unwrap();

        assert_eq!(parsed.project.name, project.project.name);
        assert_eq!(parsed.build.entry_scene, project.build.entry_scene);
    }

    #[test]
    fn test_default_asset_settings() {
        let settings = AssetSettings::default();
        assert!(settings.compress);
        assert_eq!(settings.compression_level, 3);
        assert!(!settings.include.is_empty());
    }
}
