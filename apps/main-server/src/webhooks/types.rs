//! Webhook payload types.

use serde::{Deserialize, Serialize};

/// GitHub webhook event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GitHubEventType {
    /// Pull request event.
    PullRequest,
    /// Pull request review event.
    PullRequestReview,
    /// Pull request review comment event.
    PullRequestReviewComment,
    /// Check run event.
    CheckRun,
    /// Check suite event.
    CheckSuite,
    /// Status event.
    Status,
    /// Push event.
    Push,
    /// Unknown event type.
    #[serde(other)]
    Unknown,
}

impl GitHubEventType {
    /// Parses an event type from the X-GitHub-Event header.
    pub fn from_header(header: &str) -> Self {
        match header {
            "pull_request" => Self::PullRequest,
            "pull_request_review" => Self::PullRequestReview,
            "pull_request_review_comment" => Self::PullRequestReviewComment,
            "check_run" => Self::CheckRun,
            "check_suite" => Self::CheckSuite,
            "status" => Self::Status,
            "push" => Self::Push,
            _ => Self::Unknown,
        }
    }
}

/// Pull request review action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAction {
    Submitted,
    Edited,
    Dismissed,
    #[serde(other)]
    Other,
}

/// Pull request review state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
    Pending,
    #[serde(other)]
    Unknown,
}

/// Check run status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckRunStatus {
    Queued,
    InProgress,
    Completed,
    #[serde(other)]
    Unknown,
}

/// Check run conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckRunConclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
    #[serde(other)]
    Unknown,
}

/// GitHub user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub user_type: Option<String>,
}

/// GitHub repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
    #[serde(default)]
    pub private: bool,
}

/// Pull request head/base reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    pub repo: Option<GitHubRepository>,
}

/// GitHub pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub html_url: String,
    pub state: String,
    pub user: GitHubUser,
    pub head: GitHubRef,
    pub base: GitHubRef,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub draft: bool,
}

/// GitHub pull request review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReview {
    pub id: i64,
    pub user: GitHubUser,
    pub state: ReviewState,
    #[serde(default)]
    pub body: Option<String>,
    pub submitted_at: Option<String>,
    pub html_url: String,
}

/// GitHub pull request review comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReviewComment {
    pub id: i64,
    pub user: GitHubUser,
    pub body: String,
    pub path: String,
    #[serde(default)]
    pub position: Option<i32>,
    #[serde(default)]
    pub line: Option<i32>,
    pub html_url: String,
    pub created_at: String,
}

/// GitHub check run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCheckRun {
    pub id: i64,
    pub name: String,
    pub status: CheckRunStatus,
    #[serde(default)]
    pub conclusion: Option<CheckRunConclusion>,
    pub html_url: String,
    #[serde(default)]
    pub output: Option<CheckRunOutput>,
    pub head_sha: String,
}

/// Check run output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRunOutput {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

/// Pull request review event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestReviewPayload {
    pub action: ReviewAction,
    pub review: GitHubReview,
    pub pull_request: GitHubPullRequest,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

/// Pull request review comment event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestReviewCommentPayload {
    pub action: String,
    pub comment: GitHubReviewComment,
    pub pull_request: GitHubPullRequest,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

/// Check run event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRunPayload {
    pub action: String,
    pub check_run: GitHubCheckRun,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

/// Result of processing a webhook.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookResult {
    /// Whether the webhook was processed successfully.
    pub success: bool,
    /// Description of the action taken.
    pub message: String,
    /// ID of created auto-fix task, if any.
    pub auto_fix_task_id: Option<uuid::Uuid>,
}

impl WebhookResult {
    /// Creates a success result.
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            auto_fix_task_id: None,
        }
    }

    /// Creates a success result with an auto-fix task ID.
    pub fn with_task(message: impl Into<String>, task_id: uuid::Uuid) -> Self {
        Self {
            success: true,
            message: message.into(),
            auto_fix_task_id: Some(task_id),
        }
    }

    /// Creates a skipped result (no action needed).
    pub fn skipped(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            auto_fix_task_id: None,
        }
    }

    /// Creates an error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            auto_fix_task_id: None,
        }
    }
}
