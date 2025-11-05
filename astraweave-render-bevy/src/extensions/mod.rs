//! AstraWeave Extensions for Bevy Renderer
//! 
//! This module contains AstraWeave-specific rendering extensions
//! that enhance the Bevy renderer with AI-native optimizations.
//!
//! # Extensions
//! 
//! - **MegaLights**: GPU-accelerated light culling for 100k+ lights (68× speedup)
//! - **Nanite**: Virtualized geometry for 10M+ polygons @ 60 FPS

#[cfg(feature = "megalights")]
pub mod megalights;

#[cfg(feature = "megalights")]
pub use megalights::{MegaLightsRenderer, GpuLight, ClusterBounds};

#[cfg(feature = "nanite")]
pub mod nanite;

#[cfg(feature = "nanite")]
pub use nanite::{
    NaniteCullingPipeline, GpuMeshlet, GpuCamera, CullStats,
    Frustum, LODSelector, VisibilityBuffer,
};
