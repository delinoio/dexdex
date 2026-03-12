//! RPC request types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::*;

// ============================================================================
// WorkspaceService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub endpoint_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkspacesRequest {
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkspaceRequest {
    pub workspace_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub workspace_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint_url: Option<String>,
    pub auth_profile_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWorkspaceRequest {
    pub workspace_id: Uuid,
}

// ============================================================================
// RepositoryService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRepositoryRequest {
    pub workspace_id: Uuid,
    pub remote_url: String,
    pub name: Option<String>,
    pub default_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoriesRequest {
    pub workspace_id: Option<Uuid>,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryRequest {
    pub repository_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRepositoryRequest {
    pub repository_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepositoryGroupRequest {
    pub workspace_id: Uuid,
    pub name: Option<String>,
    pub repository_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoryGroupsRequest {
    pub workspace_id: Option<Uuid>,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryGroupRequest {
    pub group_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRepositoryGroupRequest {
    pub group_id: Uuid,
    pub name: Option<String>,
    pub repository_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRepositoryGroupRequest {
    pub group_id: Uuid,
}

// ============================================================================
// TaskService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub workspace_id: Uuid,
    pub repository_group_id: Uuid,
    pub title: String,
    pub prompt: String,
    pub branch_name: Option<String>,
    pub agent_type: Option<AiAgentType>,
    pub model: Option<String>,
    pub plan_mode_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskRequest {
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksRequest {
    pub workspace_id: Option<Uuid>,
    pub repository_group_id: Option<Uuid>,
    pub status: Option<UnitTaskStatus>,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskRequest {
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTaskRequest {
    pub task_id: Uuid,
}

// ============================================================================
// SubTaskService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSubTaskRequest {
    pub sub_task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSubTasksRequest {
    pub unit_task_id: Uuid,
}

/// Approve a subtask (e.g., after reviewing the plan).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveSubTaskRequest {
    pub sub_task_id: Uuid,
}

/// Approve the generated plan for a subtask in plan mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovePlanRequest {
    pub sub_task_id: Uuid,
}

/// Reject the generated plan and provide feedback for revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisePlanRequest {
    pub sub_task_id: Uuid,
    pub feedback: String,
}

/// Manually retry a failed subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySubTaskRequest {
    pub sub_task_id: Uuid,
}

// ============================================================================
// SessionService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsRequest {
    pub sub_task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionOutputRequest {
    pub session_id: Uuid,
    pub after_sequence: Option<u64>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopSessionRequest {
    pub session_id: Uuid,
}

// ============================================================================
// PrManagementService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrTrackingRequest {
    pub unit_task_id: Uuid,
    pub provider: VcsProviderType,
    pub repository_id: String,
    pub pr_number: u64,
    pub pr_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPrTrackingRequest {
    pub pr_tracking_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPrTrackingsRequest {
    pub unit_task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAutoFixRequest {
    pub pr_tracking_id: Uuid,
}

// ============================================================================
// ReviewAssistService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListReviewAssistItemsRequest {
    pub unit_task_id: Uuid,
    pub status: Option<ReviewAssistItemStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcknowledgeReviewAssistItemRequest {
    pub item_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissReviewAssistItemRequest {
    pub item_id: Uuid,
}

// ============================================================================
// ReviewCommentService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListReviewInlineCommentsRequest {
    pub unit_task_id: Uuid,
    pub status: Option<ReviewInlineCommentStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewInlineCommentRequest {
    pub unit_task_id: Uuid,
    pub sub_task_id: Option<Uuid>,
    pub file_path: String,
    pub side: String,
    pub line_number: u32,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveReviewInlineCommentRequest {
    pub comment_id: Uuid,
}

// ============================================================================
// BadgeThemeService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBadgeThemesRequest {
    pub workspace_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertBadgeThemeRequest {
    pub workspace_id: Uuid,
    pub action_type: ActionType,
    pub color_key: BadgeColorKey,
}

// ============================================================================
// NotificationService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNotificationsRequest {
    pub workspace_id: Uuid,
    pub unread_only: bool,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkNotificationReadRequest {
    pub notification_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkAllNotificationsReadRequest {
    pub workspace_id: Uuid,
}

// ============================================================================
// EventStreamService Requests
// ============================================================================

/// Subscribe to the event stream for a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub workspace_id: Uuid,
}

// ============================================================================
// WorkerService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerRequest {
    pub name: String,
    pub endpoint_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub worker_id: Uuid,
    pub status: WorkerStatus,
    pub current_sub_task_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterWorkerRequest {
    pub worker_id: Uuid,
}

/// Request the next available subtask for a worker to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNextSubTaskRequest {
    pub worker_id: Uuid,
}

/// Report the completion status of a subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSubTaskStatusRequest {
    pub worker_id: Uuid,
    pub sub_task_id: Uuid,
    pub status: SubTaskStatus,
    pub generated_commits: Vec<GeneratedCommit>,
    pub error: Option<String>,
}

/// Emit a session output event from a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitSessionEventRequest {
    pub worker_id: Uuid,
    pub event: SessionOutputEvent,
}

// ============================================================================
// SecretsService Requests
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSecretsRequest {
    pub sub_task_id: Uuid,
    pub secrets: Vec<Secret>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearSecretsRequest {
    pub sub_task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSecretsRequest {
    pub worker_id: Uuid,
    pub sub_task_id: Uuid,
}
