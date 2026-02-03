//! Event emission traits for platform-agnostic task execution.

use async_trait::async_trait;

use crate::{AgentResult, NormalizedEvent};

/// Type of task (unit or composite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    /// A single unit task.
    UnitTask,
    /// A composite task containing multiple unit tasks.
    CompositeTask,
}

/// Event emitted when a task status changes.
#[derive(Debug, Clone)]
pub struct TaskStatusChangedEvent {
    /// The task ID.
    pub task_id: String,
    /// The task type.
    pub task_type: TaskType,
    /// The previous status.
    pub old_status: String,
    /// The new status.
    pub new_status: String,
}

/// Event emitted when an agent produces output.
#[derive(Debug, Clone)]
pub struct AgentOutputEvent {
    /// The task ID.
    pub task_id: String,
    /// The session ID.
    pub session_id: String,
    /// The normalized event.
    pub event: NormalizedEvent,
}

/// Event emitted when a TTY input is requested.
#[derive(Debug, Clone)]
pub struct TtyInputRequestEvent {
    /// The request ID.
    pub request_id: String,
    /// The task ID.
    pub task_id: String,
    /// The session ID.
    pub session_id: String,
    /// The question being asked.
    pub question: String,
    /// Available options (if multiple choice).
    pub options: Option<Vec<String>>,
}

/// Event emitted when a task execution completes.
#[derive(Debug, Clone)]
pub struct TaskCompletedEvent {
    /// The task ID.
    pub task_id: String,
    /// The task type.
    pub task_type: TaskType,
    /// Whether the task completed successfully.
    pub success: bool,
    /// Error message if the task failed.
    pub error: Option<String>,
}

/// Trait for emitting events during task execution.
///
/// This trait abstracts the platform-specific event emission mechanism,
/// allowing the executor to be used in different environments (Tauri, CLI,
/// etc.).
#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// Emits a task status changed event.
    fn emit_task_status_changed(&self, event: TaskStatusChangedEvent) -> AgentResult<()>;

    /// Emits an agent output event.
    fn emit_agent_output(&self, event: AgentOutputEvent) -> AgentResult<()>;

    /// Emits a TTY input request event.
    fn emit_tty_input_request(&self, event: TtyInputRequestEvent) -> AgentResult<()>;

    /// Emits a task completed event.
    fn emit_task_completed(&self, event: TaskCompletedEvent) -> AgentResult<()>;
}

/// A no-op event emitter that discards all events.
///
/// Useful for testing or when events are not needed.
pub struct NoOpEventEmitter;

impl NoOpEventEmitter {
    /// Creates a new no-op event emitter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpEventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventEmitter for NoOpEventEmitter {
    fn emit_task_status_changed(&self, _event: TaskStatusChangedEvent) -> AgentResult<()> {
        Ok(())
    }

    fn emit_agent_output(&self, _event: AgentOutputEvent) -> AgentResult<()> {
        Ok(())
    }

    fn emit_tty_input_request(&self, _event: TtyInputRequestEvent) -> AgentResult<()> {
        Ok(())
    }

    fn emit_task_completed(&self, _event: TaskCompletedEvent) -> AgentResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_emitter() {
        let emitter = NoOpEventEmitter::new();

        assert!(
            emitter
                .emit_task_status_changed(TaskStatusChangedEvent {
                    task_id: "test".to_string(),
                    task_type: TaskType::UnitTask,
                    old_status: "pending".to_string(),
                    new_status: "running".to_string(),
                })
                .is_ok()
        );

        assert!(
            emitter
                .emit_task_completed(TaskCompletedEvent {
                    task_id: "test".to_string(),
                    task_type: TaskType::UnitTask,
                    success: true,
                    error: None,
                })
                .is_ok()
        );
    }
}
