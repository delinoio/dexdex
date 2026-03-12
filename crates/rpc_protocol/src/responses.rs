//! RPC response types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::*;

// ============================================================================
// WorkspaceService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceResponse {
    pub workspace: Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkspacesResponse {
    pub workspaces: Vec<Workspace>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWorkspaceResponse {
    pub workspace: Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkspaceResponse {
    pub workspace: Workspace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWorkspaceResponse {}

// ============================================================================
// RepositoryService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRepositoryResponse {
    pub repository: Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoriesResponse {
    pub repositories: Vec<Repository>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryResponse {
    pub repository: Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRepositoryResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepositoryGroupResponse {
    pub group: RepositoryGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositoryGroupsResponse {
    pub groups: Vec<RepositoryGroup>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRepositoryGroupResponse {
    pub group: RepositoryGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRepositoryGroupResponse {
    pub group: RepositoryGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRepositoryGroupResponse {}

// ============================================================================
// TaskService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskResponse {
    pub task: UnitTask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskResponse {
    pub task: UnitTask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksResponse {
    pub tasks: Vec<UnitTask>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTaskResponse {}

// ============================================================================
// SubTaskService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSubTaskResponse {
    pub sub_task: SubTask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSubTasksResponse {
    pub sub_tasks: Vec<SubTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveSubTaskResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovePlanResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisePlanResponse {
    pub sub_task: SubTask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySubTaskResponse {
    pub sub_task: SubTask,
}

// ============================================================================
// SessionService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionResponse {
    pub session: AgentSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<AgentSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSessionOutputResponse {
    pub events: Vec<SessionOutputEvent>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopSessionResponse {}

// ============================================================================
// PrManagementService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrTrackingResponse {
    pub pr_tracking: PullRequestTracking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPrTrackingResponse {
    pub pr_tracking: PullRequestTracking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPrTrackingsResponse {
    pub pr_trackings: Vec<PullRequestTracking>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAutoFixResponse {}

// ============================================================================
// ReviewAssistService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListReviewAssistItemsResponse {
    pub items: Vec<ReviewAssistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcknowledgeReviewAssistItemResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissReviewAssistItemResponse {}

// ============================================================================
// ReviewCommentService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListReviewInlineCommentsResponse {
    pub comments: Vec<ReviewInlineComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewInlineCommentResponse {
    pub comment: ReviewInlineComment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveReviewInlineCommentResponse {}

// ============================================================================
// BadgeThemeService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBadgeThemesResponse {
    pub themes: Vec<BadgeTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertBadgeThemeResponse {
    pub theme: BadgeTheme,
}

// ============================================================================
// NotificationService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNotificationsResponse {
    pub notifications: Vec<Notification>,
    pub total_count: i32,
    pub unread_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkNotificationReadResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkAllNotificationsReadResponse {
    pub marked_count: i32,
}

// ============================================================================
// EventStreamService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeResponse {
    pub event: StreamEvent,
}

// ============================================================================
// WorkerService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerResponse {
    pub worker_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterWorkerResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNextSubTaskResponse {
    pub sub_task: Option<SubTask>,
    pub unit_task: Option<UnitTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSubTaskStatusResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitSessionEventResponse {}

// ============================================================================
// SecretsService Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSecretsResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearSecretsResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSecretsResponse {
    pub secrets: Vec<Secret>,
}
