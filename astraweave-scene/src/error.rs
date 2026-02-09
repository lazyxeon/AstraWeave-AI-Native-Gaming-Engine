//! Typed error types for scene management and world streaming.

use thiserror::Error;

/// Error type for scene and world partition operations.
#[derive(Debug, Error)]
#[non_exhaustive]
#[must_use]
pub enum SceneError {
    /// A cell or chunk was not found.
    #[error("cell not found: {0}")]
    CellNotFound(String),

    /// Streaming or async loading failed.
    #[error("streaming error: {0}")]
    Streaming(String),

    /// Scene file I/O error.
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Serialization or deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// World partition error (bounds, overlap, etc.).
    #[error("partition error: {0}")]
    Partition(String),

    /// Catch-all for `anyhow` migration.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenience alias for scene operation results.
pub type SceneResult<T> = std::result::Result<T, SceneError>;
