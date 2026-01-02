//! Blender version parsing and validation.
//!
//! This module handles parsing Blender version strings from the `--version` output
//! and validates that the installed version meets minimum requirements.

use crate::error::{BlendError, BlendResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

/// Minimum supported Blender version.
///
/// Blender 2.93 introduced significant improvements to the glTF exporter,
/// including better material handling and Draco compression support.
pub const MINIMUM_BLENDER_VERSION: BlenderVersion = BlenderVersion {
    major: 2,
    minor: 93,
    patch: 0,
};

/// Recommended Blender version for optimal compatibility.
pub const RECOMMENDED_BLENDER_VERSION: BlenderVersion = BlenderVersion {
    major: 4,
    minor: 0,
    patch: 0,
};

/// Regex for parsing Blender version output.
///
/// Matches patterns like:
/// - "Blender 3.6.0"
/// - "Blender 4.0.2"  
/// - "Blender 2.93.18"
static VERSION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Blender\s+(\d+)\.(\d+)(?:\.(\d+))?").expect("Invalid version regex")
});

/// Represents a Blender version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlenderVersion {
    /// Major version (e.g., 4 in 4.0.2)
    pub major: u32,
    /// Minor version (e.g., 0 in 4.0.2)
    pub minor: u32,
    /// Patch version (e.g., 2 in 4.0.2)
    pub patch: u32,
}

impl BlenderVersion {
    /// Creates a new BlenderVersion.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Parses a BlenderVersion from Blender's `--version` output.
    ///
    /// # Example Output Formats
    ///
    /// ```text
    /// Blender 4.0.2
    ///   build date: 2024-01-15
    ///   build time: 00:00:00
    ///   ...
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `BlendError::VersionParseError` if the output cannot be parsed.
    pub fn from_version_output(output: &str) -> BlendResult<Self> {
        let captures = VERSION_REGEX.captures(output).ok_or_else(|| {
            BlendError::VersionParseError {
                output: output.lines().next().unwrap_or(output).to_string(),
            }
        })?;

        let major: u32 = captures
            .get(1)
            .and_then(|m| m.as_str().parse().ok())
            .ok_or_else(|| BlendError::VersionParseError {
                output: output.to_string(),
            })?;

        let minor: u32 = captures
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .ok_or_else(|| BlendError::VersionParseError {
                output: output.to_string(),
            })?;

        let patch: u32 = captures
            .get(3)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        Ok(Self { major, minor, patch })
    }

    /// Checks if this version meets the minimum requirements.
    pub fn meets_minimum(&self) -> bool {
        *self >= MINIMUM_BLENDER_VERSION
    }

    /// Checks if this version is at least the recommended version.
    pub fn is_recommended(&self) -> bool {
        *self >= RECOMMENDED_BLENDER_VERSION
    }

    /// Validates this version meets minimum requirements.
    ///
    /// # Errors
    ///
    /// Returns `BlendError::BlenderVersionTooOld` if the version is below minimum.
    pub fn validate(&self) -> BlendResult<()> {
        if self.meets_minimum() {
            Ok(())
        } else {
            Err(BlendError::BlenderVersionTooOld {
                found: self.to_string(),
                required: MINIMUM_BLENDER_VERSION.to_string(),
            })
        }
    }

    /// Returns capability flags based on version.
    pub fn capabilities(&self) -> BlenderCapabilities {
        BlenderCapabilities {
            draco_compression: *self >= BlenderVersion::new(2, 93, 0),
            gltf_materials_variants: *self >= BlenderVersion::new(3, 3, 0),
            geometry_nodes_export: *self >= BlenderVersion::new(3, 5, 0),
            usd_export: *self >= BlenderVersion::new(3, 0, 0),
            webp_textures: *self >= BlenderVersion::new(3, 4, 0),
            ktx2_textures: *self >= BlenderVersion::new(4, 0, 0),
        }
    }

    /// Returns the version as a tuple.
    pub fn as_tuple(&self) -> (u32, u32, u32) {
        (self.major, self.minor, self.patch)
    }
}

impl fmt::Display for BlenderVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for BlenderVersion {
    type Err = BlendError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() < 2 {
            return Err(BlendError::VersionParseError {
                output: s.to_string(),
            });
        }

        let major = parts[0].parse().map_err(|_| BlendError::VersionParseError {
            output: s.to_string(),
        })?;
        let minor = parts[1].parse().map_err(|_| BlendError::VersionParseError {
            output: s.to_string(),
        })?;
        let patch = parts
            .get(2)
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);

        Ok(Self { major, minor, patch })
    }
}

impl PartialOrd for BlenderVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlenderVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

impl Default for BlenderVersion {
    fn default() -> Self {
        MINIMUM_BLENDER_VERSION
    }
}

/// Capabilities available based on Blender version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlenderCapabilities {
    /// Draco mesh compression in glTF export.
    pub draco_compression: bool,
    /// glTF material variants extension.
    pub gltf_materials_variants: bool,
    /// Geometry nodes evaluated mesh export.
    pub geometry_nodes_export: bool,
    /// USD format export support.
    pub usd_export: bool,
    /// WebP texture export.
    pub webp_textures: bool,
    /// KTX2/Basis Universal texture compression.
    pub ktx2_textures: bool,
}

impl BlenderCapabilities {
    /// Returns a list of capability descriptions.
    pub fn describe(&self) -> Vec<(&'static str, bool)> {
        vec![
            ("Draco compression", self.draco_compression),
            ("Material variants", self.gltf_materials_variants),
            ("Geometry nodes export", self.geometry_nodes_export),
            ("USD export", self.usd_export),
            ("WebP textures", self.webp_textures),
            ("KTX2 textures", self.ktx2_textures),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        // Standard format
        let v = BlenderVersion::from_version_output("Blender 4.0.2").unwrap();
        assert_eq!(v, BlenderVersion::new(4, 0, 2));

        // With build info
        let v = BlenderVersion::from_version_output(
            "Blender 3.6.0\n  build date: 2024-01-15\n  build time: 00:00:00",
        )
        .unwrap();
        assert_eq!(v, BlenderVersion::new(3, 6, 0));

        // Without patch version
        let v = BlenderVersion::from_version_output("Blender 2.93").unwrap();
        assert_eq!(v, BlenderVersion::new(2, 93, 0));

        // Old version format
        let v = BlenderVersion::from_version_output("Blender 2.80.0").unwrap();
        assert_eq!(v, BlenderVersion::new(2, 80, 0));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = BlenderVersion::new(2, 93, 0);
        let v2 = BlenderVersion::new(3, 0, 0);
        let v3 = BlenderVersion::new(3, 0, 1);
        let v4 = BlenderVersion::new(3, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v2 == v4);
        assert!(v3 > v1);
    }

    #[test]
    fn test_minimum_version() {
        let old = BlenderVersion::new(2, 80, 0);
        let minimum = BlenderVersion::new(2, 93, 0);
        let newer = BlenderVersion::new(4, 0, 0);

        assert!(!old.meets_minimum());
        assert!(minimum.meets_minimum());
        assert!(newer.meets_minimum());
    }

    #[test]
    fn test_version_validation() {
        let old = BlenderVersion::new(2, 80, 0);
        assert!(old.validate().is_err());

        let valid = BlenderVersion::new(3, 6, 0);
        assert!(valid.validate().is_ok());
    }

    #[test]
    fn test_from_str() {
        let v: BlenderVersion = "3.6.2".parse().unwrap();
        assert_eq!(v, BlenderVersion::new(3, 6, 2));

        let v: BlenderVersion = "4.0".parse().unwrap();
        assert_eq!(v, BlenderVersion::new(4, 0, 0));
    }

    #[test]
    fn test_display() {
        let v = BlenderVersion::new(4, 0, 2);
        assert_eq!(v.to_string(), "4.0.2");
    }

    #[test]
    fn test_capabilities() {
        let old = BlenderVersion::new(2, 80, 0);
        let caps = old.capabilities();
        assert!(!caps.draco_compression);
        assert!(!caps.ktx2_textures);

        let new = BlenderVersion::new(4, 1, 0);
        let caps = new.capabilities();
        assert!(caps.draco_compression);
        assert!(caps.ktx2_textures);
        assert!(caps.geometry_nodes_export);
    }

    #[test]
    fn test_invalid_version_output() {
        assert!(BlenderVersion::from_version_output("not blender").is_err());
        assert!(BlenderVersion::from_version_output("version 1.0").is_err());
    }
}
