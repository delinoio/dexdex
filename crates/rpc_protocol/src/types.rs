//! RPC type definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Normalized Event Types
// ============================================================================

/// Type of file change made by an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeType {
    /// File was created.
    Create,
    /// File was modified.
    Modify,
    /// File was deleted.
    Delete,
    /// File was renamed.
    Rename {
        /// Original file path.
        from: String,
    },
}

/// Normalized event types from AI coding agents.
///
/// This enum represents all possible events that can be emitted by any
/// supported AI coding agent, normalized to a common format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NormalizedEvent {
    /// Text output from the agent.
    TextOutput {
        /// The text content.
        content: String,
        /// Whether this is streaming output (partial).
        stream: bool,
    },

    /// Error output from the agent.
    ErrorOutput {
        /// The error message.
        content: String,
    },

    /// Agent is using a tool.
    ToolUse {
        /// Name of the tool being used.
        tool_name: String,
        /// Input to the tool (JSON value).
        input: serde_json::Value,
    },

    /// Result from a tool use.
    ToolResult {
        /// Name of the tool that was used.
        tool_name: String,
        /// Output from the tool (JSON value).
        output: serde_json::Value,
        /// Whether the tool returned an error.
        is_error: bool,
    },

    /// File was changed by the agent.
    FileChange {
        /// Path to the file.
        path: String,
        /// Type of change.
        change_type: FileChangeType,
        /// New file content (if applicable).
        content: Option<String>,
    },

    /// Command was executed by the agent.
    CommandExecution {
        /// The command that was executed.
        command: String,
        /// Exit code of the command.
        exit_code: Option<i32>,
        /// Output from the command.
        output: Option<String>,
    },

    /// Agent is asking the user a question.
    AskUserQuestion {
        /// The question being asked.
        question: String,
        /// Available options (if multiple choice).
        options: Option<Vec<String>>,
    },

    /// User responded to a question.
    UserResponse {
        /// The user's response.
        response: String,
    },

    /// Agent session started.
    SessionStart {
        /// Type of agent.
        agent_type: String,
        /// Model being used (if applicable).
        model: Option<String>,
    },

    /// Agent session ended.
    SessionEnd {
        /// Whether the session completed successfully.
        success: bool,
        /// Error message if the session failed.
        error: Option<String>,
    },

    /// Agent is thinking/reasoning.
    Thinking {
        /// The thinking content.
        content: String,
    },

    /// Raw/unparsed output (fallback for unknown formats).
    Raw {
        /// Raw content from the agent.
        content: String,
    },

    /// Token usage report from the agent.
    UsageReport {
        /// Number of input tokens (prompt tokens).
        input_tokens: u64,
        /// Number of output tokens (completion tokens).
        output_tokens: u64,
        /// Number of cache read tokens (if applicable).
        cache_read_tokens: u64,
        /// Number of cache write tokens (if applicable).
        cache_write_tokens: u64,
    },
}

impl NormalizedEvent {
    /// Creates a new text output event.
    pub fn text(content: impl Into<String>, stream: bool) -> Self {
        Self::TextOutput {
            content: content.into(),
            stream,
        }
    }

    /// Creates a new error output event.
    pub fn error(content: impl Into<String>) -> Self {
        Self::ErrorOutput {
            content: content.into(),
        }
    }

    /// Creates a new session start event.
    pub fn session_start(agent_type: impl Into<String>, model: Option<String>) -> Self {
        Self::SessionStart {
            agent_type: agent_type.into(),
            model,
        }
    }

    /// Creates a new session end event.
    pub fn session_end(success: bool, error: Option<String>) -> Self {
        Self::SessionEnd { success, error }
    }

    /// Creates a new thinking event.
    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking {
            content: content.into(),
        }
    }

    /// Creates a new file change event.
    pub fn file_change(
        path: impl Into<String>,
        change_type: FileChangeType,
        content: Option<String>,
    ) -> Self {
        Self::FileChange {
            path: path.into(),
            change_type,
            content,
        }
    }

    /// Creates a new command execution event.
    pub fn command(
        command: impl Into<String>,
        exit_code: Option<i32>,
        output: Option<String>,
    ) -> Self {
        Self::CommandExecution {
            command: command.into(),
            exit_code,
            output,
        }
    }

    /// Creates a new tool use event.
    pub fn tool_use(tool_name: impl Into<String>, input: serde_json::Value) -> Self {
        Self::ToolUse {
            tool_name: tool_name.into(),
            input,
        }
    }

    /// Creates a new tool result event.
    pub fn tool_result(
        tool_name: impl Into<String>,
        output: serde_json::Value,
        is_error: bool,
    ) -> Self {
        Self::ToolResult {
            tool_name: tool_name.into(),
            output,
            is_error,
        }
    }

    /// Creates a new ask user question event.
    pub fn ask_user(question: impl Into<String>, options: Option<Vec<String>>) -> Self {
        Self::AskUserQuestion {
            question: question.into(),
            options,
        }
    }

    /// Creates a new user response event.
    pub fn user_response(response: impl Into<String>) -> Self {
        Self::UserResponse {
            response: response.into(),
        }
    }

    /// Creates a new raw output event.
    pub fn raw(content: impl Into<String>) -> Self {
        Self::Raw {
            content: content.into(),
        }
    }

    /// Returns true if this event is an ask-user question (TTY input required).
    pub fn is_tty_input_required(&self) -> bool {
        matches!(self, Self::AskUserQuestion { .. })
    }

    /// Creates a new usage report event.
    pub fn usage_report(
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> Self {
        Self::UsageReport {
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_write_tokens,
        }
    }

    /// Returns true if this event is a usage report.
    pub fn is_usage_report(&self) -> bool {
        matches!(self, Self::UsageReport { .. })
    }
}

/// A timestamped event for storage in logs.
///
/// This wrapper adds a timestamp to normalized events so that historical
/// events can be displayed with their actual occurrence time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEvent {
    /// The timestamp when the event occurred.
    pub timestamp: DateTime<Utc>,
    /// The normalized event.
    pub event: NormalizedEvent,
}

// ============================================================================
// RPC Types
// ============================================================================

/// VCS type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcsType {
    Unspecified,
    Git,
}

/// VCS provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcsProviderType {
    Unspecified,
    Github,
    Gitlab,
    Bitbucket,
}

/// AI agent type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiAgentType {
    Unspecified,
    ClaudeCode,
    OpenCode,
    GeminiCli,
    CodexCli,
    Aider,
    Amp,
}

/// Unit task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitTaskStatus {
    Unspecified,
    InProgress,
    InReview,
    Approved,
    PrOpen,
    Done,
    Rejected,
    Failed,
    Cancelled,
}

/// Composite task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositeTaskStatus {
    Unspecified,
    Planning,
    PendingApproval,
    InProgress,
    Done,
    Rejected,
}

/// TTY input type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtyInputType {
    Unspecified,
    Text,
    Select,
    Confirm,
    Password,
}

/// TTY input status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtyInputStatus {
    Unspecified,
    Pending,
    Responded,
    Timeout,
    Cancelled,
}

/// Worker status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Unspecified,
    Idle,
    Busy,
    Unhealthy,
}

/// Todo item type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoItemType {
    Unspecified,
    IssueTriage,
    PrReview,
}

/// Todo item status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoItemStatus {
    Unspecified,
    Pending,
    InProgress,
    Completed,
    Dismissed,
}

/// Base remote information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRemote {
    pub git_remote_url: String,
    pub git_branch_name: String,
}

/// Token usage information for an AI agent session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input tokens (prompt tokens).
    pub input_tokens: u64,
    /// Number of output tokens (completion tokens).
    pub output_tokens: u64,
    /// Number of cache read tokens (if applicable).
    pub cache_read_tokens: u64,
    /// Number of cache write tokens (if applicable).
    pub cache_write_tokens: u64,
}

/// Agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub agent_task_id: String,
    pub ai_agent_type: AiAgentType,
    pub ai_agent_model: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_log: Option<String>,
    pub token_usage: Option<TokenUsage>,
    pub created_at: DateTime<Utc>,
}

/// Agent task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub base_remotes: Vec<BaseRemote>,
    pub agent_sessions: Vec<AgentSession>,
    pub ai_agent_type: Option<AiAgentType>,
    pub ai_agent_model: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Unit task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitTask {
    pub id: String,
    pub repository_group_id: String,
    pub agent_task_id: String,
    pub prompt: String,
    pub title: Option<String>,
    pub branch_name: Option<String>,
    pub linked_pr_url: Option<String>,
    pub base_commit: Option<String>,
    pub end_commit: Option<String>,
    pub auto_fix_task_ids: Vec<String>,
    pub status: UnitTaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Composite task node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeTaskNode {
    pub id: String,
    pub composite_task_id: String,
    pub unit_task_id: String,
    pub depends_on_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Composite task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeTask {
    pub id: String,
    pub repository_group_id: String,
    pub planning_task_id: String,
    pub prompt: String,
    pub title: Option<String>,
    pub node_ids: Vec<String>,
    pub status: CompositeTaskStatus,
    pub execution_agent_type: Option<AiAgentType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub remote_url: String,
    pub default_branch: String,
    pub vcs_type: VcsType,
    pub vcs_provider_type: VcsProviderType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Repository group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryGroup {
    pub id: String,
    pub workspace_id: String,
    pub name: Option<String>,
    pub repository_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// TTY input request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtyInputRequest {
    pub id: String,
    pub task_id: String,
    pub session_id: String,
    pub prompt: String,
    pub input_type: TtyInputType,
    pub options: Vec<String>,
    pub status: TtyInputStatus,
    pub response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

/// Worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub id: String,
    pub name: String,
    pub endpoint_url: String,
    pub status: WorkerStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub current_task_id: Option<String>,
    pub registered_at: DateTime<Utc>,
}

/// Issue triage data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTriageData {
    pub issue_url: String,
    pub issue_title: String,
    pub suggested_labels: Vec<String>,
    pub suggested_assignees: Vec<String>,
}

/// PR review data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReviewData {
    pub pr_url: String,
    pub pr_title: String,
    pub changed_files_count: i32,
    pub ai_summary: Option<String>,
}

/// Todo item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub item_type: TodoItemType,
    pub status: TodoItemStatus,
    pub repository_id: String,
    pub issue_triage: Option<IssueTriageData>,
    pub pr_review: Option<PrReviewData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Secret key-value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub key: String,
    pub value: String,
}
