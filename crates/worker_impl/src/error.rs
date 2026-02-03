//! Error types for the worker implementation.

use thiserror::Error;

/// Error type for worker operations.
#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Repository group not found: {0}")]
    RepositoryGroupNotFound(String),

    #[error("Agent task not found: {0}")]
    AgentTaskNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task store error: {0}")]
    TaskStore(#[from] task_store::TaskStoreError),

    #[error("Agent error: {0}")]
    Agent(#[from] coding_agents::AgentError),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for worker operations.
pub type WorkerResult<T> = Result<T, WorkerError>;
