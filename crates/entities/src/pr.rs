//! Pull request tracking entity definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of VCS provider hosting the pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcsProviderType {
    /// GitHub
    Github,
    /// GitLab
    Gitlab,
    /// Bitbucket
    Bitbucket,
}

/// Status of a pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrStatus {
    /// PR is open and awaiting review.
    Open,
    /// PR has been approved by reviewers.
    Approved,
    /// Reviewers have requested changes.
    ChangesRequested,
    /// PR has been merged into the target branch.
    Merged,
    /// PR has been closed without merging.
    Closed,
    /// CI checks on the PR have failed.
    CiFailed,
}

/// Status of a review assist item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAssistItemStatus {
    /// Item is awaiting user attention.
    Pending,
    /// User has acknowledged the item.
    Acknowledged,
    /// User has dismissed the item.
    Dismissed,
}

/// Status of a review inline comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewInlineCommentStatus {
    /// Comment is open and unresolved.
    Open,
    /// Comment has been resolved.
    Resolved,
}

/// Tracks a pull request associated with a unit task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestTracking {
    /// Unique identifier.
    pub id: Uuid,
    /// The unit task this PR is associated with.
    pub unit_task_id: Uuid,
    /// The VCS provider hosting this PR.
    pub provider: VcsProviderType,
    /// Provider-specific repository identifier.
    pub repository_id: String,
    /// PR number on the provider.
    pub pr_number: u64,
    /// Direct URL to the pull request.
    pub pr_url: String,
    /// Current status of the pull request.
    pub status: PrStatus,
    /// When this PR was last polled for updates.
    pub last_polled_at: Option<DateTime<Utc>>,
    /// Whether to automatically fix review comments and CI failures.
    pub auto_fix_enabled: bool,
    /// Maximum number of auto-fix attempts allowed.
    pub max_auto_fix_attempts: u32,
    /// Number of auto-fix attempts used so far.
    pub auto_fix_attempts_used: u32,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl PullRequestTracking {
    /// Creates a new pull request tracking record.
    pub fn new(
        unit_task_id: Uuid,
        provider: VcsProviderType,
        repository_id: impl Into<String>,
        pr_number: u64,
        pr_url: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            unit_task_id,
            provider,
            repository_id: repository_id.into(),
            pr_number,
            pr_url: pr_url.into(),
            status: PrStatus::Open,
            last_polled_at: None,
            auto_fix_enabled: false,
            max_auto_fix_attempts: 3,
            auto_fix_attempts_used: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Enables auto-fix with the given maximum attempts.
    pub fn with_auto_fix(mut self, max_attempts: u32) -> Self {
        self.auto_fix_enabled = true;
        self.max_auto_fix_attempts = max_attempts;
        self
    }
}

/// An AI-generated item surfaced for review assistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewAssistItem {
    /// Unique identifier.
    pub id: Uuid,
    /// The unit task this item belongs to.
    pub unit_task_id: Uuid,
    /// The PR tracking record this item is associated with.
    pub pr_tracking_id: Uuid,
    /// The source type identifier (e.g., "security_scan", "code_smell").
    pub source_type: String,
    /// Short title describing the item.
    pub title: String,
    /// Detailed description of the finding.
    pub details: String,
    /// Current status of this item.
    pub status: ReviewAssistItemStatus,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ReviewAssistItem {
    /// Creates a new review assist item.
    pub fn new(
        unit_task_id: Uuid,
        pr_tracking_id: Uuid,
        source_type: impl Into<String>,
        title: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            unit_task_id,
            pr_tracking_id,
            source_type: source_type.into(),
            title: title.into(),
            details: details.into(),
            status: ReviewAssistItemStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }
}

/// An inline comment on a specific line of a PR review.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewInlineComment {
    /// Unique identifier.
    pub id: Uuid,
    /// The unit task this comment belongs to.
    pub unit_task_id: Uuid,
    /// The subtask that generated this comment, if any.
    pub sub_task_id: Option<Uuid>,
    /// The file path this comment is on.
    pub file_path: String,
    /// Which side of the diff this comment is on ("left" or "right").
    pub side: String,
    /// The line number in the file.
    pub line_number: u32,
    /// The comment body text.
    pub body: String,
    /// Current status of this comment.
    pub status: ReviewInlineCommentStatus,
    /// The user ID of the comment author.
    pub author_user_id: Option<Uuid>,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ReviewInlineComment {
    /// Creates a new review inline comment.
    pub fn new(
        unit_task_id: Uuid,
        file_path: impl Into<String>,
        side: impl Into<String>,
        line_number: u32,
        body: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            unit_task_id,
            sub_task_id: None,
            file_path: file_path.into(),
            side: side.into(),
            line_number,
            body: body.into(),
            status: ReviewInlineCommentStatus::Open,
            author_user_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_request_tracking_creation() {
        let unit_task_id = Uuid::new_v4();
        let pr = PullRequestTracking::new(
            unit_task_id,
            VcsProviderType::Github,
            "user/repo",
            42,
            "https://github.com/user/repo/pull/42",
        )
        .with_auto_fix(5);

        assert_eq!(pr.unit_task_id, unit_task_id);
        assert_eq!(pr.provider, VcsProviderType::Github);
        assert_eq!(pr.pr_number, 42);
        assert_eq!(pr.status, PrStatus::Open);
        assert!(pr.auto_fix_enabled);
        assert_eq!(pr.max_auto_fix_attempts, 5);
    }

    #[test]
    fn test_review_assist_item_creation() {
        let unit_task_id = Uuid::new_v4();
        let pr_tracking_id = Uuid::new_v4();
        let item = ReviewAssistItem::new(
            unit_task_id,
            pr_tracking_id,
            "security_scan",
            "SQL Injection risk",
            "Line 42 may be vulnerable to SQL injection",
        );

        assert_eq!(item.unit_task_id, unit_task_id);
        assert_eq!(item.source_type, "security_scan");
        assert_eq!(item.status, ReviewAssistItemStatus::Pending);
    }
}
