//! Server error types.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Server error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Invalid request parameters.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Store error.
    #[error("Store error: {0}")]
    Store(#[from] task_store::TaskStoreError),

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST,
                rpc_protocol::error_codes::INVALID_REQUEST,
                msg.clone(),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                rpc_protocol::error_codes::RESOURCE_NOT_FOUND,
                msg.clone(),
            ),
            AppError::Store(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                rpc_protocol::error_codes::INTERNAL_ERROR,
                e.to_string(),
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                rpc_protocol::error_codes::INTERNAL_ERROR,
                msg.clone(),
            ),
        };

        let body = json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

/// Result type alias for server operations.
pub type AppResult<T> = Result<T, AppError>;
