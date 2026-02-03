//! Error types for the Tauri app.

use serde::Serialize;
use thiserror::Error;

/// Error type for Tauri commands.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Secrets error: {0}")]
    Secrets(String),

    #[error("Remote error: {0}")]
    Remote(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task store error: {0}")]
    TaskStore(#[from] task_store::TaskStoreError),

    #[error("Secrets keychain error: {0}")]
    SecretsKeychain(#[from] secrets::SecretsError),

    #[error("Worker error: {0}")]
    Worker(#[from] worker_impl::error::WorkerError),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result type for Tauri commands.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_not_found() {
        let err = AppError::NotFound("Task".to_string());
        assert_eq!(err.to_string(), "Not found: Task");
    }

    #[test]
    fn test_error_display_invalid_request() {
        let err = AppError::InvalidRequest("Missing field".to_string());
        assert_eq!(err.to_string(), "Invalid request: Missing field");
    }

    #[test]
    fn test_error_display_config() {
        let err = AppError::Config("Invalid TOML".to_string());
        assert_eq!(err.to_string(), "Configuration error: Invalid TOML");
    }

    #[test]
    fn test_error_display_storage() {
        let err = AppError::Storage("Database error".to_string());
        assert_eq!(err.to_string(), "Storage error: Database error");
    }

    #[test]
    fn test_error_display_secrets() {
        let err = AppError::Secrets("Keychain unavailable".to_string());
        assert_eq!(err.to_string(), "Secrets error: Keychain unavailable");
    }

    #[test]
    fn test_error_display_remote() {
        let err = AppError::Remote("Connection failed".to_string());
        assert_eq!(err.to_string(), "Remote error: Connection failed");
    }

    #[test]
    fn test_error_display_internal() {
        let err = AppError::Internal("Unexpected state".to_string());
        assert_eq!(err.to_string(), "Internal error: Unexpected state");
    }

    #[test]
    fn test_error_serialize() {
        let err = AppError::NotFound("Test".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"Not found: Test\"");
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_err: AppError = io_err.into();
        assert!(app_err.to_string().contains("File not found"));
    }
}
