//! Task store trait definitions.

use async_trait::async_trait;
use entities::{
    AgentSession, BadgeTheme, Notification, PullRequestTracking, Repository, RepositoryGroup,
    ReviewAssistItem, ReviewInlineComment, SessionOutputEvent, SubTask, UnitTask, UnitTaskStatus,
    Workspace,
};
use uuid::Uuid;

use crate::TaskStoreResult;

/// Filter options for listing workspaces.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceFilter {
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

/// Filter options for listing repositories.
#[derive(Debug, Clone, Default)]
pub struct RepositoryFilter {
    /// Filter by workspace ID.
    pub workspace_id: Option<Uuid>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

/// Filter options for listing repository groups.
#[derive(Debug, Clone, Default)]
pub struct RepositoryGroupFilter {
    /// Filter by workspace ID.
    pub workspace_id: Option<Uuid>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

/// Filter options for listing unit tasks.
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    /// Filter by workspace ID.
    pub workspace_id: Option<Uuid>,
    /// Filter by repository group ID.
    pub repository_group_id: Option<Uuid>,
    /// Filter by unit task status.
    pub status: Option<UnitTaskStatus>,
    /// Maximum number of results.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

/// Trait for task storage operations.
#[async_trait]
pub trait TaskStore: Send + Sync {
    // =========================================================================
    // Workspace operations
    // =========================================================================

    /// Creates a new workspace.
    async fn create_workspace(&self, workspace: Workspace) -> TaskStoreResult<Workspace>;

    /// Gets a workspace by ID.
    async fn get_workspace(&self, id: Uuid) -> TaskStoreResult<Option<Workspace>>;

    /// Lists workspaces with optional filters.
    async fn list_workspaces(
        &self,
        filter: WorkspaceFilter,
    ) -> TaskStoreResult<(Vec<Workspace>, u32)>;

    /// Updates a workspace.
    async fn update_workspace(&self, workspace: Workspace) -> TaskStoreResult<Workspace>;

    /// Deletes a workspace.
    async fn delete_workspace(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // Repository operations
    // =========================================================================

    /// Creates a new repository.
    async fn create_repository(&self, repo: Repository) -> TaskStoreResult<Repository>;

    /// Gets a repository by ID.
    async fn get_repository(&self, id: Uuid) -> TaskStoreResult<Option<Repository>>;

    /// Lists repositories with optional filters.
    async fn list_repositories(
        &self,
        filter: RepositoryFilter,
    ) -> TaskStoreResult<(Vec<Repository>, u32)>;

    /// Updates a repository.
    async fn update_repository(&self, repo: Repository) -> TaskStoreResult<Repository>;

    /// Deletes a repository.
    async fn delete_repository(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // RepositoryGroup operations
    // =========================================================================

    /// Creates a new repository group.
    async fn create_repository_group(
        &self,
        group: RepositoryGroup,
    ) -> TaskStoreResult<RepositoryGroup>;

    /// Gets a repository group by ID.
    async fn get_repository_group(&self, id: Uuid) -> TaskStoreResult<Option<RepositoryGroup>>;

    /// Lists repository groups with optional filters.
    async fn list_repository_groups(
        &self,
        filter: RepositoryGroupFilter,
    ) -> TaskStoreResult<(Vec<RepositoryGroup>, u32)>;

    /// Updates a repository group.
    async fn update_repository_group(
        &self,
        group: RepositoryGroup,
    ) -> TaskStoreResult<RepositoryGroup>;

    /// Deletes a repository group.
    async fn delete_repository_group(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // UnitTask operations
    // =========================================================================

    /// Creates a new unit task.
    async fn create_unit_task(&self, task: UnitTask) -> TaskStoreResult<UnitTask>;

    /// Gets a unit task by ID.
    async fn get_unit_task(&self, id: Uuid) -> TaskStoreResult<Option<UnitTask>>;

    /// Lists unit tasks with optional filters.
    async fn list_unit_tasks(&self, filter: TaskFilter) -> TaskStoreResult<(Vec<UnitTask>, u32)>;

    /// Updates a unit task.
    async fn update_unit_task(&self, task: UnitTask) -> TaskStoreResult<UnitTask>;

    /// Deletes a unit task.
    async fn delete_unit_task(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // SubTask operations
    // =========================================================================

    /// Creates a new subtask.
    async fn create_sub_task(&self, sub_task: SubTask) -> TaskStoreResult<SubTask>;

    /// Gets a subtask by ID.
    async fn get_sub_task(&self, id: Uuid) -> TaskStoreResult<Option<SubTask>>;

    /// Lists subtasks belonging to a unit task.
    async fn list_sub_tasks(&self, unit_task_id: Uuid) -> TaskStoreResult<Vec<SubTask>>;

    /// Updates a subtask.
    async fn update_sub_task(&self, sub_task: SubTask) -> TaskStoreResult<SubTask>;

    /// Deletes a subtask.
    async fn delete_sub_task(&self, id: Uuid) -> TaskStoreResult<()>;

    /// Returns the oldest queued subtask, if any.
    async fn get_next_queued_sub_task(&self) -> TaskStoreResult<Option<SubTask>>;

    // =========================================================================
    // AgentSession operations
    // =========================================================================

    /// Creates a new agent session.
    async fn create_agent_session(&self, session: AgentSession) -> TaskStoreResult<AgentSession>;

    /// Gets an agent session by ID.
    async fn get_agent_session(&self, id: Uuid) -> TaskStoreResult<Option<AgentSession>>;

    /// Lists agent sessions belonging to a subtask.
    async fn list_agent_sessions(&self, sub_task_id: Uuid) -> TaskStoreResult<Vec<AgentSession>>;

    /// Updates an agent session.
    async fn update_agent_session(&self, session: AgentSession) -> TaskStoreResult<AgentSession>;

    /// Deletes an agent session.
    async fn delete_agent_session(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // SessionOutputEvent operations
    // =========================================================================

    /// Appends a session output event.
    async fn append_session_output(
        &self,
        event: SessionOutputEvent,
    ) -> TaskStoreResult<SessionOutputEvent>;

    /// Lists session output events for a session, optionally filtered by
    /// sequence number.
    async fn list_session_outputs(
        &self,
        session_id: Uuid,
        since_sequence: Option<u64>,
    ) -> TaskStoreResult<Vec<SessionOutputEvent>>;

    // =========================================================================
    // PullRequestTracking operations
    // =========================================================================

    /// Creates a new pull request tracking record.
    async fn create_pr_tracking(
        &self,
        pr: PullRequestTracking,
    ) -> TaskStoreResult<PullRequestTracking>;

    /// Gets a pull request tracking record by ID.
    async fn get_pr_tracking(&self, id: Uuid) -> TaskStoreResult<Option<PullRequestTracking>>;

    /// Lists pull request tracking records, optionally filtered by unit task
    /// ID.
    async fn list_pr_trackings(
        &self,
        unit_task_id: Option<Uuid>,
    ) -> TaskStoreResult<Vec<PullRequestTracking>>;

    /// Updates a pull request tracking record.
    async fn update_pr_tracking(
        &self,
        pr: PullRequestTracking,
    ) -> TaskStoreResult<PullRequestTracking>;

    /// Deletes a pull request tracking record.
    async fn delete_pr_tracking(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // ReviewAssistItem operations
    // =========================================================================

    /// Creates a new review assist item.
    async fn create_review_assist_item(
        &self,
        item: ReviewAssistItem,
    ) -> TaskStoreResult<ReviewAssistItem>;

    /// Gets a review assist item by ID.
    async fn get_review_assist_item(&self, id: Uuid) -> TaskStoreResult<Option<ReviewAssistItem>>;

    /// Lists review assist items, optionally filtered by unit task ID.
    async fn list_review_assist_items(
        &self,
        unit_task_id: Option<Uuid>,
    ) -> TaskStoreResult<Vec<ReviewAssistItem>>;

    /// Updates a review assist item.
    async fn update_review_assist_item(
        &self,
        item: ReviewAssistItem,
    ) -> TaskStoreResult<ReviewAssistItem>;

    /// Deletes a review assist item.
    async fn delete_review_assist_item(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // ReviewInlineComment operations
    // =========================================================================

    /// Creates a new review inline comment.
    async fn create_review_inline_comment(
        &self,
        comment: ReviewInlineComment,
    ) -> TaskStoreResult<ReviewInlineComment>;

    /// Gets a review inline comment by ID.
    async fn get_review_inline_comment(
        &self,
        id: Uuid,
    ) -> TaskStoreResult<Option<ReviewInlineComment>>;

    /// Lists review inline comments for a unit task.
    async fn list_review_inline_comments(
        &self,
        unit_task_id: Uuid,
    ) -> TaskStoreResult<Vec<ReviewInlineComment>>;

    /// Updates a review inline comment.
    async fn update_review_inline_comment(
        &self,
        comment: ReviewInlineComment,
    ) -> TaskStoreResult<ReviewInlineComment>;

    /// Deletes a review inline comment.
    async fn delete_review_inline_comment(&self, id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // BadgeTheme operations
    // =========================================================================

    /// Lists badge themes for a workspace.
    async fn list_badge_themes(&self, workspace_id: Uuid) -> TaskStoreResult<Vec<BadgeTheme>>;

    /// Upserts a badge theme (insert or update by workspace_id + action_type).
    async fn upsert_badge_theme(&self, theme: BadgeTheme) -> TaskStoreResult<BadgeTheme>;

    // =========================================================================
    // Notification operations
    // =========================================================================

    /// Creates a new notification.
    async fn create_notification(
        &self,
        notification: Notification,
    ) -> TaskStoreResult<Notification>;

    /// Gets a notification by ID.
    async fn get_notification(&self, id: Uuid) -> TaskStoreResult<Option<Notification>>;

    /// Lists notifications, optionally filtered by workspace ID and read
    /// status.
    async fn list_notifications(
        &self,
        workspace_id: Option<Uuid>,
        unread_only: bool,
    ) -> TaskStoreResult<Vec<Notification>>;

    /// Marks a notification as read.
    async fn mark_notification_read(&self, id: Uuid) -> TaskStoreResult<()>;

    /// Marks all notifications in a workspace as read.
    async fn mark_all_notifications_read(&self, workspace_id: Uuid) -> TaskStoreResult<()>;

    // =========================================================================
    // Cascade delete operations
    //
    // NOTE: These cascade operations are NOT transactional. If the process
    // crashes mid-deletion, orphaned child resources may remain. This is a
    // known limitation acceptable for single-user desktop apps. For
    // multi-user deployments, consider wrapping these in a database
    // transaction or adding a periodic cleanup job.
    // =========================================================================

    /// Deletes a unit task and all associated subtasks and their sessions.
    ///
    /// This is a best-effort cascade: child deletion failures are logged
    /// but do not prevent the unit task from being deleted.
    async fn delete_unit_task_cascade(&self, id: Uuid) -> TaskStoreResult<()> {
        // Delete all subtasks (each cascades to sessions)
        let sub_tasks = self.list_sub_tasks(id).await.unwrap_or_default();
        for sub_task in &sub_tasks {
            // Delete agent sessions belonging to this subtask
            let sessions = self
                .list_agent_sessions(sub_task.id)
                .await
                .unwrap_or_default();
            for session in &sessions {
                if let Err(e) = self.delete_agent_session(session.id).await {
                    tracing::warn!(
                        sub_task_id = %sub_task.id,
                        session_id = %session.id,
                        error = %e,
                        "Failed to delete agent session during cascade delete"
                    );
                }
            }

            if let Err(e) = self.delete_sub_task(sub_task.id).await {
                tracing::warn!(
                    unit_task_id = %id,
                    sub_task_id = %sub_task.id,
                    error = %e,
                    "Failed to delete subtask during cascade delete"
                );
            }
        }

        self.delete_unit_task(id).await
    }
}
