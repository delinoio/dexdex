//! Worker server error types.

use thiserror::Error;

/// Errors that can occur in the worker server.
#[derive(Debug, Error)]
pub enum WorkerError {
    /// Failed to connect to main server.
    #[error("Failed to connect to main server: {0}")]
    MainServerConnection(String),

    /// Failed to register with main server.
    #[error("Failed to register with main server: {0}")]
    Registration(String),

    /// Task execution failed.
    #[error("Task execution failed: {0}")]
    TaskExecution(String),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type for worker operations.
pub type WorkerResult<T> = Result<T, WorkerError>;
