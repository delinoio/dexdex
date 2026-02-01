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

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task store error: {0}")]
    TaskStore(#[from] task_store::TaskStoreError),

    #[error("Secrets keychain error: {0}")]
    SecretsKeychain(#[from] secrets::SecretsError),

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
