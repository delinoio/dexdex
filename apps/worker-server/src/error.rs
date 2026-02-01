//! Worker server error types.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
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

    /// Docker operation failed.
    #[error("Docker operation failed: {0}")]
    Docker(#[from] bollard::errors::Error),

    /// Task execution failed.
    #[error("Task execution failed: {0}")]
    TaskExecution(String),

    /// Git operation failed.
    #[error("Git operation failed: {0}")]
    Git(String),

    /// Agent error.
    #[error("Agent error: {0}")]
    Agent(#[from] coding_agents::AgentError),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Task not found.
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Worker is busy.
    #[error("Worker is busy")]
    Busy,

    /// Task was cancelled.
    #[error("Task was cancelled")]
    Cancelled,
}

impl IntoResponse for WorkerError {
    fn into_response(self) -> Response {
        let status = match &self {
            WorkerError::TaskNotFound(_) => StatusCode::NOT_FOUND,
            WorkerError::Busy => StatusCode::SERVICE_UNAVAILABLE,
            WorkerError::Cancelled => StatusCode::CONFLICT,
            WorkerError::Config(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = serde_json::json!({
            "error": self.to_string(),
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Result type for worker operations.
pub type WorkerResult<T> = Result<T, WorkerError>;
