//! Notification and badge theme entity definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ActionType;

/// Color key for badge themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeColorKey {
    /// Blue badge color.
    Blue,
    /// Green badge color.
    Green,
    /// Yellow badge color.
    Yellow,
    /// Orange badge color.
    Orange,
    /// Red badge color.
    Red,
    /// Gray badge color.
    Gray,
}

/// Type of notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// A task requires user action.
    TaskActionRequired,
    /// A plan requires user approval.
    PlanActionRequired,
    /// There is activity on a PR review.
    PrReviewActivity,
    /// CI has failed on a PR.
    PrCiFailure,
    /// An agent session has failed.
    AgentSessionFailed,
}

/// Type of event sent over the server-sent event stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventType {
    /// A unit task was updated.
    TaskUpdated,
    /// A subtask was updated.
    SubtaskUpdated,
    /// An agent session produced output.
    SessionOutput,
    /// An agent session changed state.
    SessionStateChanged,
    /// A pull request was updated.
    PrUpdated,
    /// A review assist item was updated.
    ReviewAssistUpdated,
    /// An inline comment was updated.
    InlineCommentUpdated,
    /// A notification was created.
    NotificationCreated,
}

/// Theme configuration for a badge associated with an action type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgeTheme {
    /// Unique identifier.
    pub id: Uuid,
    /// The workspace this theme belongs to.
    pub workspace_id: Uuid,
    /// The action type this theme applies to.
    pub action_type: ActionType,
    /// The color to use for the badge.
    pub color_key: BadgeColorKey,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl BadgeTheme {
    /// Creates a new badge theme.
    pub fn new(workspace_id: Uuid, action_type: ActionType, color_key: BadgeColorKey) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            action_type,
            color_key,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A notification delivered to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    /// Unique identifier.
    pub id: Uuid,
    /// The workspace this notification belongs to.
    pub workspace_id: Uuid,
    /// The type of notification.
    pub notification_type: NotificationType,
    /// Short title for the notification.
    pub title: String,
    /// Notification body text.
    pub body: String,
    /// Optional deep link URL to navigate to when the notification is clicked.
    pub deep_link: Option<String>,
    /// When the user read this notification (None if unread).
    pub read_at: Option<DateTime<Utc>>,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
}

impl Notification {
    /// Creates a new notification.
    pub fn new(
        workspace_id: Uuid,
        notification_type: NotificationType,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            notification_type,
            title: title.into(),
            body: body.into(),
            deep_link: None,
            read_at: None,
            created_at: Utc::now(),
        }
    }

    /// Sets the deep link for this notification.
    pub fn with_deep_link(mut self, deep_link: impl Into<String>) -> Self {
        self.deep_link = Some(deep_link.into());
        self
    }

    /// Returns whether this notification has been read.
    pub fn is_read(&self) -> bool {
        self.read_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_theme_creation() {
        let workspace_id = Uuid::new_v4();
        let theme = BadgeTheme::new(workspace_id, ActionType::CiFailed, BadgeColorKey::Red);

        assert_eq!(theme.workspace_id, workspace_id);
        assert_eq!(theme.action_type, ActionType::CiFailed);
        assert_eq!(theme.color_key, BadgeColorKey::Red);
    }

    #[test]
    fn test_notification_creation() {
        let workspace_id = Uuid::new_v4();
        let notif = Notification::new(
            workspace_id,
            NotificationType::TaskActionRequired,
            "Action Required",
            "Your task needs attention",
        )
        .with_deep_link("dexdex://tasks/123");

        assert_eq!(notif.workspace_id, workspace_id);
        assert_eq!(
            notif.notification_type,
            NotificationType::TaskActionRequired
        );
        assert_eq!(notif.deep_link, Some("dexdex://tasks/123".to_string()));
        assert!(!notif.is_read());
    }
}
