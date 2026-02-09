//! Typed error types for the AstraWeave rendering pipeline.
//!
//! All public rendering APIs should return [`RenderResult<T>`] instead of
//! `anyhow::Result<T>`. Existing code can be migrated incrementally by converting
//! `anyhow::Result` → `RenderResult` and `.context("...")` → mapping to the
//! appropriate [`RenderError`] variant.

use thiserror::Error;

/// Unified error type for the AstraWeave rendering pipeline.
///
/// # Examples
///
/// ```
/// use astraweave_render::error::{RenderError, RenderResult};
///
/// fn load_texture(path: &str) -> RenderResult<()> {
///     if path.is_empty() {
///         return Err(RenderError::AssetLoad {
///             asset: "texture".into(),
///             detail: "empty path".into(),
///         });
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
#[must_use]
pub enum RenderError {
    /// GPU device creation or adapter request failed.
    #[error("GPU device error: {0}")]
    Device(String),

    /// Shader compilation or pipeline creation failed.
    #[error("shader/pipeline error: {0}")]
    Shader(String),

    /// Asset loading failed (textures, meshes, materials, HDRI, etc.).
    #[error("failed to load {asset}: {detail}")]
    AssetLoad {
        /// Kind of asset (e.g. "texture", "mesh", "material", "HDRI").
        asset: String,
        /// Human-readable detail.
        detail: String,
    },

    /// Surface configuration or presentation failed.
    #[error("surface error: {0}")]
    Surface(String),

    /// Render graph construction or execution failed.
    #[error("render graph error: {0}")]
    Graph(String),

    /// Material system error (missing arrays, invalid descriptors, etc.).
    #[error("material error: {0}")]
    Material(String),

    /// Post-processing pipeline error.
    #[error("post-processing error: {0}")]
    PostProcess(String),

    /// Shadow map or CSM error.
    #[error("shadow error: {0}")]
    Shadow(String),

    /// Animation or skinning error.
    #[error("animation error: {0}")]
    Animation(String),

    /// I/O error reading files from disk or network.
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Image decoding/encoding error.
    #[error("image error: {0}")]
    Image(String),

    /// wgpu validation or internal error.
    #[error("wgpu error: {0}")]
    Wgpu(String),

    /// Catch-all for errors migrated from `anyhow` that don't yet have a
    /// dedicated variant. New code should NOT use this variant.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenience alias for results from the rendering pipeline.
pub type RenderResult<T> = std::result::Result<T, RenderError>;
