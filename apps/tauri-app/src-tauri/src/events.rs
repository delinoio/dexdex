//! Tauri event definitions.
//!
//! This module defines the events that can be emitted from the Rust backend
//! to the frontend.

use coding_agents::NormalizedEvent;
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

/// Event emitted when a notification is shown.
///
/// This event is emitted when a desktop notification is displayed.
/// The frontend can use this to track which notifications were shown
/// and navigate accordingly when the user clicks the notification.
///
/// Note: Actual click handling requires platform-specific implementation.
/// This event indicates the notification was shown, not that it was clicked.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationShownEvent {
    pub task_type: TaskType,
    pub task_id: String,
}

/// Event emitted when an agent produces output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentOutputEvent {
    pub task_id: String,
    pub session_id: String,
    pub event: NormalizedEvent,
}

/// Event emitted when a task execution completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompletedEvent {
    pub task_id: String,
    pub task_type: TaskType,
    pub success: bool,
    pub error: Option<String>,
}

/// Event names as constants.
pub mod event_names {
    pub const TASK_STATUS_CHANGED: &str = "task-status-changed";
    pub const TTY_INPUT_REQUEST: &str = "tty-input-request";
    pub const NOTIFICATION_SHOWN: &str = "notification-shown";
    pub const AGENT_OUTPUT: &str = "agent-output";
    pub const TASK_COMPLETED: &str = "task-completed";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_serialization() {
        let unit = TaskType::UnitTask;
        let json = serde_json::to_string(&unit).unwrap();
        assert_eq!(json, "\"unit_task\"");

        let composite = TaskType::CompositeTask;
        let json = serde_json::to_string(&composite).unwrap();
        assert_eq!(json, "\"composite_task\"");
    }

    #[test]
    fn test_task_type_deserialization() {
        let unit: TaskType = serde_json::from_str("\"unit_task\"").unwrap();
        assert!(matches!(unit, TaskType::UnitTask));

        let composite: TaskType = serde_json::from_str("\"composite_task\"").unwrap();
        assert!(matches!(composite, TaskType::CompositeTask));
    }

    #[test]
    fn test_task_status_changed_event_serialization() {
        let event = TaskStatusChangedEvent {
            task_id: "task-123".to_string(),
            task_type: TaskType::UnitTask,
            old_status: "pending".to_string(),
            new_status: "running".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"taskId\":\"task-123\""));
        assert!(json.contains("\"taskType\":\"unit_task\""));
        assert!(json.contains("\"oldStatus\":\"pending\""));
        assert!(json.contains("\"newStatus\":\"running\""));
    }

    #[test]
    fn test_tty_input_request_event_serialization() {
        let event = TtyInputRequestEvent {
            request_id: "req-1".to_string(),
            task_id: "task-1".to_string(),
            session_id: "session-1".to_string(),
            question: "What is your name?".to_string(),
            options: Some(vec!["Option A".to_string(), "Option B".to_string()]),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"requestId\":\"req-1\""));
        assert!(json.contains("\"question\":\"What is your name?\""));
        assert!(json.contains("\"options\":[\"Option A\",\"Option B\"]"));
    }

    #[test]
    fn test_tty_input_request_event_without_options() {
        let event = TtyInputRequestEvent {
            request_id: "req-1".to_string(),
            task_id: "task-1".to_string(),
            session_id: "session-1".to_string(),
            question: "Enter your name".to_string(),
            options: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"options\":null"));
    }

    #[test]
    fn test_notification_shown_event_serialization() {
        let event = NotificationShownEvent {
            task_type: TaskType::CompositeTask,
            task_id: "task-456".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"taskType\":\"composite_task\""));
        assert!(json.contains("\"taskId\":\"task-456\""));
    }

    #[test]
    fn test_event_names_constants() {
        assert_eq!(event_names::TASK_STATUS_CHANGED, "task-status-changed");
        assert_eq!(event_names::TTY_INPUT_REQUEST, "tty-input-request");
        assert_eq!(event_names::NOTIFICATION_SHOWN, "notification-shown");
        assert_eq!(event_names::AGENT_OUTPUT, "agent-output");
        assert_eq!(event_names::TASK_COMPLETED, "task-completed");
    }

    #[test]
    fn test_agent_output_event_serialization() {
        let event = AgentOutputEvent {
            task_id: "task-123".to_string(),
            session_id: "session-456".to_string(),
            event: NormalizedEvent::text("Hello, world!", false),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"taskId\":\"task-123\""));
        assert!(json.contains("\"sessionId\":\"session-456\""));
        assert!(json.contains("\"content\":\"Hello, world!\""));
    }

    #[test]
    fn test_task_completed_event_serialization() {
        let event = TaskCompletedEvent {
            task_id: "task-123".to_string(),
            task_type: TaskType::UnitTask,
            success: true,
            error: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"taskId\":\"task-123\""));
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"error\":null"));
    }

    #[test]
    fn test_task_completed_event_with_error() {
        let event = TaskCompletedEvent {
            task_id: "task-123".to_string(),
            task_type: TaskType::UnitTask,
            success: false,
            error: Some("Something went wrong".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Something went wrong\""));
    }
}
