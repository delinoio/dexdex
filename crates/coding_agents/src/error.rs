//! Error types for the coding_agents crate.

use thiserror::Error;

/// Errors that can occur when running AI coding agents.
#[derive(Debug, Error)]
pub enum AgentError {
    /// Agent process failed to start.
    #[error("Failed to start agent process: {0}")]
    ProcessStart(#[from] std::io::Error),

    /// Agent process exited with an error.
    #[error("Agent process exited with code {code}: {message}")]
    ProcessExit { code: i32, message: String },

    /// Failed to parse agent output.
    #[error("Failed to parse agent output: {0}")]
    OutputParse(String),

    /// Agent timed out.
    #[error("Agent timed out after {0} seconds")]
    Timeout(u64),

    /// Agent was cancelled.
    #[error("Agent execution was cancelled")]
    Cancelled,

    /// TTY input required but no handler provided.
    #[error("TTY input required: {0}")]
    TtyInputRequired(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Other error.
    #[error("{0}")]
    Other(String),
}

/// Result type for agent operations.
pub type AgentResult<T> = Result<T, AgentError>;
