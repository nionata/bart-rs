//! Error types for the BART client.

use thiserror::Error;

/// Errors that can occur when using the BART client.
#[derive(Debug, Error)]
pub enum Error {
    /// An HTTP or network-level error from `reqwest`.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A JSON deserialization error (unexpected response shape).
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An error returned by the BART API itself.
    #[error("API error: {0}")]
    Api(String),
}

/// Convenience alias for `Result<T, bart::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
