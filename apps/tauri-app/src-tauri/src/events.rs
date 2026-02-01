//! Tauri event definitions.
//!
//! This module defines the events that can be emitted from the Rust backend
//! to the frontend.

use serde::{Deserialize, Serialize};

/// Event emitted when a task status changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusChangedEvent {
    pub task_id: String,
    pub task_type: TaskType,
    pub old_status: String,
    pub new_status: String,
}

/// Type of task (unit or composite).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    UnitTask,
    CompositeTask,
}

/// Event emitted when a TTY input is requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtyInputRequestEvent {
    pub request_id: String,
    pub task_id: String,
    pub session_id: String,
    pub question: String,
    pub options: Option<Vec<String>>,
}

/// Event emitted when a notification is clicked.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationClickedEvent {
    pub task_type: TaskType,
    pub task_id: String,
}

/// Event names as constants.
pub mod event_names {
    pub const TASK_STATUS_CHANGED: &str = "task-status-changed";
    pub const TTY_INPUT_REQUEST: &str = "tty-input-request";
    pub const NOTIFICATION_CLICKED: &str = "notification-clicked";
}
