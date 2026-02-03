//! Desktop notification system.
//!
//! This module provides platform-specific notification implementations
//! with click handling support.

use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, warn};

use crate::events::{event_names, NotificationShownEvent, TaskType};

/// Notification trigger types.
#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    /// AI agent is asking a question (TTY input).
    TtyInputRequest,
    /// Task is ready for review.
    TaskReviewReady,
    /// Plan is ready for approval.
    PlanApproval,
    /// Task has failed.
    TaskFailed,
}

impl NotificationType {
    /// Gets the title for the notification.
    pub fn title(&self) -> &'static str {
        match self {
            NotificationType::TtyInputRequest => "Agent Question",
            NotificationType::TaskReviewReady => "Task Ready for Review",
            NotificationType::PlanApproval => "Plan Ready for Approval",
            NotificationType::TaskFailed => "Task Failed",
        }
    }
}

/// Sends a desktop notification.
pub fn send_notification(
    app: &AppHandle,
    notification_type: NotificationType,
    message: &str,
    task_type: TaskType,
    task_id: &str,
) {
    let title = notification_type.title();

    info!("Sending notification: {} - {}", title, message);

    // Use the notification plugin
    if let Err(e) = app
        .notification()
        .builder()
        .title(title)
        .body(message)
        .show()
    {
        warn!("Failed to show notification: {}", e);
        return;
    }

    // Emit notification shown event
    // Note: Actual click handling requires platform-specific implementation.
    // This event indicates the notification was shown, allowing the frontend
    // to track which notifications are active for potential navigation.
    let event = NotificationShownEvent {
        task_type,
        task_id: task_id.to_string(),
    };

    if let Err(e) = app.emit(event_names::NOTIFICATION_SHOWN, event) {
        warn!("Failed to emit notification shown event: {}", e);
    }
}

/// Sends a TTY input request notification.
///
/// This function is part of the public notification API and will be called
/// by task handlers when a TTY input is requested from the user.
pub fn send_tty_input_notification(
    app: &AppHandle,
    task_type: TaskType,
    task_id: &str,
    question: &str,
) {
    let message = if question.len() > 100 {
        format!("{}...", &question[..97])
    } else {
        question.to_string()
    };

    send_notification(
        app,
        NotificationType::TtyInputRequest,
        &message,
        task_type,
        task_id,
    );
}

/// Sends a task review ready notification.
///
/// This function is part of the public notification API and will be called
/// when a task transitions to a reviewable state.
pub fn send_task_review_notification(
    app: &AppHandle,
    task_type: TaskType,
    task_id: &str,
    task_title: Option<&str>,
) {
    let message = task_title
        .map(|t| format!("\"{}\" is ready for your review", t))
        .unwrap_or_else(|| "A task is ready for your review".to_string());

    send_notification(
        app,
        NotificationType::TaskReviewReady,
        &message,
        task_type,
        task_id,
    );
}

/// Sends a plan approval notification.
///
/// This function is part of the public notification API and will be called
/// when a composite task plan is ready for user approval.
pub fn send_plan_approval_notification(app: &AppHandle, task_id: &str, task_title: Option<&str>) {
    let message = task_title
        .map(|t| format!("Plan for \"{}\" is ready for approval", t))
        .unwrap_or_else(|| "A plan is ready for approval".to_string());

    send_notification(
        app,
        NotificationType::PlanApproval,
        &message,
        TaskType::CompositeTask,
        task_id,
    );
}

/// Sends a task failed notification.
///
/// This function is part of the public notification API and will be called
/// when a task fails during execution.
pub fn send_task_failed_notification(
    app: &AppHandle,
    task_type: TaskType,
    task_id: &str,
    error: Option<&str>,
) {
    let message = error
        .map(|e| format!("Task failed: {}", e))
        .unwrap_or_else(|| "A task has failed".to_string());

    send_notification(
        app,
        NotificationType::TaskFailed,
        &message,
        task_type,
        task_id,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_type_tty_input_title() {
        let notification_type = NotificationType::TtyInputRequest;
        assert_eq!(notification_type.title(), "Agent Question");
    }

    #[test]
    fn test_notification_type_task_review_title() {
        let notification_type = NotificationType::TaskReviewReady;
        assert_eq!(notification_type.title(), "Task Ready for Review");
    }

    #[test]
    fn test_notification_type_plan_approval_title() {
        let notification_type = NotificationType::PlanApproval;
        assert_eq!(notification_type.title(), "Plan Ready for Approval");
    }

    #[test]
    fn test_notification_type_task_failed_title() {
        let notification_type = NotificationType::TaskFailed;
        assert_eq!(notification_type.title(), "Task Failed");
    }
}
