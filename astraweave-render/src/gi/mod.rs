//! Global Illumination Module
//!
//! This module provides various GI solutions including VXGI and integration
//! with the existing DDGI system.

pub mod vxgi;

pub use vxgi::{VxgiConfig, VxgiRenderer, VoxelRadiance, CONE_TRACING_SHADER};

/// Hybrid GI configuration combining VXGI and DDGI
#[derive(Debug, Clone, Copy)]
pub struct HybridGiConfig {
    /// Use VXGI for voxel terrain
    pub use_vxgi: bool,
    /// Use DDGI for polygonal assets
    pub use_ddgi: bool,
    /// VXGI configuration
    pub vxgi_config: VxgiConfig,
}

impl Default for HybridGiConfig {
    fn default() -> Self {
        Self {
            use_vxgi: true,
            use_ddgi: true,
            vxgi_config: VxgiConfig::default(),
        }
    }
}