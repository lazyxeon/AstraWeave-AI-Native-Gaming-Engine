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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_error_display() {
        let e = RenderError::Device("lost".into());
        assert_eq!(format!("{e}"), "GPU device error: lost");
    }

    #[test]
    fn shader_error_display() {
        let e = RenderError::Shader("compile fail".into());
        assert_eq!(format!("{e}"), "shader/pipeline error: compile fail");
    }

    #[test]
    fn asset_load_error_display() {
        let e = RenderError::AssetLoad {
            asset: "texture".into(),
            detail: "file not found".into(),
        };
        assert_eq!(format!("{e}"), "failed to load texture: file not found");
    }

    #[test]
    fn surface_error_display() {
        let e = RenderError::Surface("timeout".into());
        assert_eq!(format!("{e}"), "surface error: timeout");
    }

    #[test]
    fn graph_error_display() {
        let e = RenderError::Graph("cycle".into());
        assert_eq!(format!("{e}"), "render graph error: cycle");
    }

    #[test]
    fn material_error_display() {
        let e = RenderError::Material("missing array".into());
        assert_eq!(format!("{e}"), "material error: missing array");
    }

    #[test]
    fn post_process_error_display() {
        let e = RenderError::PostProcess("bad shader".into());
        assert_eq!(format!("{e}"), "post-processing error: bad shader");
    }

    #[test]
    fn shadow_error_display() {
        let e = RenderError::Shadow("cascade overflow".into());
        assert_eq!(format!("{e}"), "shadow error: cascade overflow");
    }

    #[test]
    fn animation_error_display() {
        let e = RenderError::Animation("bad joint".into());
        assert_eq!(format!("{e}"), "animation error: bad joint");
    }

    #[test]
    fn io_error_from_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let e: RenderError = io_err.into();
        let msg = format!("{e}");
        assert!(
            msg.contains("I/O error"),
            "should display I/O prefix, got: {msg}"
        );
        assert!(msg.contains("missing"), "should contain original message");
    }

    #[test]
    fn image_error_display() {
        let e = RenderError::Image("bad png".into());
        assert_eq!(format!("{e}"), "image error: bad png");
    }

    #[test]
    fn wgpu_error_display() {
        let e = RenderError::Wgpu("validation".into());
        assert_eq!(format!("{e}"), "wgpu error: validation");
    }

    #[test]
    fn other_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("something else went wrong");
        let e: RenderError = anyhow_err.into();
        let msg = format!("{e}");
        assert!(msg.contains("something else went wrong"));
    }

    #[test]
    #[allow(clippy::unnecessary_literal_unwrap)]
    fn render_result_ok_works() {
        let r: RenderResult<u32> = Ok(42);
        assert_eq!(r.expect("test"), 42);
    }

    #[test]
    fn render_result_with_question_mark() -> RenderResult<()> {
        let inner: RenderResult<i32> = Ok(10);
        let _v = inner?;
        Ok(())
    }

    #[test]
    fn error_is_debug() {
        let e = RenderError::Device("test".into());
        let debug = format!("{e:?}");
        assert!(debug.contains("Device"));
    }

    #[test]
    fn asset_load_fields_distinct() {
        let e = RenderError::AssetLoad {
            asset: "mesh".into(),
            detail: "corrupt data".into(),
        };
        let msg = format!("{e}");
        assert!(msg.contains("mesh"));
        assert!(msg.contains("corrupt data"));
    }
}
