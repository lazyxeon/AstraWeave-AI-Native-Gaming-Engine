//! Typed error types for the AstraWeave networking layer.

use thiserror::Error;

/// Error type for networking operations.
#[derive(Debug, Error)]
#[non_exhaustive]
#[must_use]
pub enum NetError {
    /// Connection or handshake failed.
    #[error("connection error: {0}")]
    Connection(String),

    /// TLS / certificate error.
    #[error("TLS error: {0}")]
    Tls(String),

    /// Protocol-level error (invalid message, version mismatch, etc.).
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Rate limiting or backpressure rejected the request.
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// Authentication or HMAC verification failed.
    #[error("authentication error: {0}")]
    Auth(String),

    /// I/O error.
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Catch-all for `anyhow` migration.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenience alias for networking results.
pub type NetResult<T> = std::result::Result<T, NetError>;
