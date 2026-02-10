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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_cell_not_found() {
        let e = SceneError::CellNotFound("chunk_42".into());
        assert_eq!(e.to_string(), "cell not found: chunk_42");
    }

    #[test]
    fn display_streaming() {
        let e = SceneError::Streaming("timeout".into());
        assert_eq!(e.to_string(), "streaming error: timeout");
    }

    #[test]
    fn display_io_via_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let e = SceneError::from(io_err);
        assert!(e.to_string().starts_with("I/O error:"));
    }

    #[test]
    fn display_serialization() {
        let e = SceneError::Serialization("bad json".into());
        assert_eq!(e.to_string(), "serialization error: bad json");
    }

    #[test]
    fn display_partition() {
        let e = SceneError::Partition("overlap detected".into());
        assert_eq!(e.to_string(), "partition error: overlap detected");
    }

    #[test]
    fn display_other_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("something broke");
        let e = SceneError::from(anyhow_err);
        assert_eq!(e.to_string(), "something broke");
    }

    #[test]
    fn scene_result_ok_and_err() {
        let ok: SceneResult<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: SceneResult<i32> = Err(SceneError::CellNotFound("x".into()));
        assert!(err.is_err());
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // SceneError uses anyhow internally which is Send+Sync
        assert_send_sync::<SceneError>();
    }
}
