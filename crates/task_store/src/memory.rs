//! In-memory task store implementation for testing.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use entities::{
    ActionType, AgentSession, BadgeTheme, Notification, PullRequestTracking, Repository,
    RepositoryGroup, ReviewAssistItem, ReviewInlineComment, SessionOutputEvent, SubTask,
    SubTaskStatus, UnitTask, Workspace,
};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    RepositoryFilter, RepositoryGroupFilter, TaskFilter, TaskStore, TaskStoreError,
    TaskStoreResult, WorkspaceFilter,
};

/// Internal storage for the in-memory task store.
#[derive(Debug, Default)]
struct StoreInner {
    workspaces: HashMap<Uuid, Workspace>,
    repositories: HashMap<Uuid, Repository>,
    repository_groups: HashMap<Uuid, RepositoryGroup>,
    unit_tasks: HashMap<Uuid, UnitTask>,
    sub_tasks: HashMap<Uuid, SubTask>,
    agent_sessions: HashMap<Uuid, AgentSession>,
    session_outputs: Vec<SessionOutputEvent>,
    pr_trackings: HashMap<Uuid, PullRequestTracking>,
    review_assist_items: HashMap<Uuid, ReviewAssistItem>,
    review_inline_comments: HashMap<Uuid, ReviewInlineComment>,
    /// Key: (workspace_id, action_type discriminant string)
    badge_themes: HashMap<(Uuid, String), BadgeTheme>,
    notifications: HashMap<Uuid, Notification>,
}

fn action_type_key(action_type: ActionType) -> String {
    format!("{:?}", action_type)
}

/// In-memory task store for testing purposes.
#[derive(Debug, Default, Clone)]
pub struct MemoryTaskStore {
    inner: Arc<RwLock<StoreInner>>,
}

impl MemoryTaskStore {
    /// Creates a new in-memory task store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TaskStore for MemoryTaskStore {
    // =========================================================================
    // Workspace operations
    // =========================================================================

    async fn create_workspace(&self, workspace: Workspace) -> TaskStoreResult<Workspace> {
        let mut inner = self.inner.write().await;
        if inner.workspaces.contains_key(&workspace.id) {
            return Err(TaskStoreError::already_exists(
                "Workspace",
                workspace.id.to_string(),
            ));
        }
        inner.workspaces.insert(workspace.id, workspace.clone());
        Ok(workspace)
    }

    async fn get_workspace(&self, id: Uuid) -> TaskStoreResult<Option<Workspace>> {
        let inner = self.inner.read().await;
        Ok(inner.workspaces.get(&id).cloned())
    }

    async fn list_workspaces(
        &self,
        filter: WorkspaceFilter,
    ) -> TaskStoreResult<(Vec<Workspace>, u32)> {
        let inner = self.inner.read().await;
        let mut result: Vec<Workspace> = inner.workspaces.values().cloned().collect();

        let total = result.len() as u32;

        if let Some(offset) = filter.offset {
            result = result.into_iter().skip(offset as usize).collect();
        }
        if let Some(limit) = filter.limit {
            result = result.into_iter().take(limit as usize).collect();
        }

        Ok((result, total))
    }

    async fn update_workspace(&self, workspace: Workspace) -> TaskStoreResult<Workspace> {
        let mut inner = self.inner.write().await;
        if !inner.workspaces.contains_key(&workspace.id) {
            return Err(TaskStoreError::not_found(
                "Workspace",
                workspace.id.to_string(),
            ));
        }
        inner.workspaces.insert(workspace.id, workspace.clone());
        Ok(workspace)
    }

    async fn delete_workspace(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.workspaces.remove(&id).is_none() {
            return Err(TaskStoreError::not_found("Workspace", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Repository operations
    // =========================================================================

    async fn create_repository(&self, repo: Repository) -> TaskStoreResult<Repository> {
        let mut inner = self.inner.write().await;
        if inner.repositories.contains_key(&repo.id) {
            return Err(TaskStoreError::already_exists(
                "Repository",
                repo.id.to_string(),
            ));
        }
        inner.repositories.insert(repo.id, repo.clone());
        Ok(repo)
    }

    async fn get_repository(&self, id: Uuid) -> TaskStoreResult<Option<Repository>> {
        let inner = self.inner.read().await;
        Ok(inner.repositories.get(&id).cloned())
    }

    async fn list_repositories(
        &self,
        filter: RepositoryFilter,
    ) -> TaskStoreResult<(Vec<Repository>, u32)> {
        let inner = self.inner.read().await;
        let mut result: Vec<Repository> = inner
            .repositories
            .values()
            .filter(|r| {
                if let Some(workspace_id) = filter.workspace_id {
                    r.workspace_id == workspace_id
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        let total = result.len() as u32;

        if let Some(offset) = filter.offset {
            result = result.into_iter().skip(offset as usize).collect();
        }
        if let Some(limit) = filter.limit {
            result = result.into_iter().take(limit as usize).collect();
        }

        Ok((result, total))
    }

    async fn update_repository(&self, repo: Repository) -> TaskStoreResult<Repository> {
        let mut inner = self.inner.write().await;
        if !inner.repositories.contains_key(&repo.id) {
            return Err(TaskStoreError::not_found("Repository", repo.id.to_string()));
        }
        inner.repositories.insert(repo.id, repo.clone());
        Ok(repo)
    }

    async fn delete_repository(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.repositories.remove(&id).is_none() {
            return Err(TaskStoreError::not_found("Repository", id.to_string()));
        }

        // Remove the deleted repository from all repository groups
        for group in inner.repository_groups.values_mut() {
            if group.repository_ids.contains(&id) {
                group.repository_ids.retain(|&r| r != id);
                group.updated_at = Utc::now();
            }
        }

        Ok(())
    }

    // =========================================================================
    // RepositoryGroup operations
    // =========================================================================

    async fn create_repository_group(
        &self,
        group: RepositoryGroup,
    ) -> TaskStoreResult<RepositoryGroup> {
        let mut inner = self.inner.write().await;
        if inner.repository_groups.contains_key(&group.id) {
            return Err(TaskStoreError::already_exists(
                "RepositoryGroup",
                group.id.to_string(),
            ));
        }
        inner.repository_groups.insert(group.id, group.clone());
        Ok(group)
    }

    async fn get_repository_group(&self, id: Uuid) -> TaskStoreResult<Option<RepositoryGroup>> {
        let inner = self.inner.read().await;
        Ok(inner.repository_groups.get(&id).cloned())
    }

    async fn list_repository_groups(
        &self,
        filter: RepositoryGroupFilter,
    ) -> TaskStoreResult<(Vec<RepositoryGroup>, u32)> {
        let inner = self.inner.read().await;
        let mut result: Vec<RepositoryGroup> = inner
            .repository_groups
            .values()
            .filter(|g| {
                if let Some(ws_id) = filter.workspace_id {
                    g.workspace_id == ws_id
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        let total = result.len() as u32;

        if let Some(offset) = filter.offset {
            result = result.into_iter().skip(offset as usize).collect();
        }
        if let Some(limit) = filter.limit {
            result = result.into_iter().take(limit as usize).collect();
        }

        Ok((result, total))
    }

    async fn update_repository_group(
        &self,
        group: RepositoryGroup,
    ) -> TaskStoreResult<RepositoryGroup> {
        let mut inner = self.inner.write().await;
        if !inner.repository_groups.contains_key(&group.id) {
            return Err(TaskStoreError::not_found(
                "RepositoryGroup",
                group.id.to_string(),
            ));
        }
        inner.repository_groups.insert(group.id, group.clone());
        Ok(group)
    }

    async fn delete_repository_group(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.repository_groups.remove(&id).is_none() {
            return Err(TaskStoreError::not_found("RepositoryGroup", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // UnitTask operations
    // =========================================================================

    async fn create_unit_task(&self, task: UnitTask) -> TaskStoreResult<UnitTask> {
        let mut inner = self.inner.write().await;
        if inner.unit_tasks.contains_key(&task.id) {
            return Err(TaskStoreError::already_exists(
                "UnitTask",
                task.id.to_string(),
            ));
        }
        inner.unit_tasks.insert(task.id, task.clone());
        Ok(task)
    }

    async fn get_unit_task(&self, id: Uuid) -> TaskStoreResult<Option<UnitTask>> {
        let inner = self.inner.read().await;
        Ok(inner.unit_tasks.get(&id).cloned())
    }

    async fn list_unit_tasks(&self, filter: TaskFilter) -> TaskStoreResult<(Vec<UnitTask>, u32)> {
        let inner = self.inner.read().await;
        let mut result: Vec<UnitTask> = inner
            .unit_tasks
            .values()
            .filter(|t| {
                let mut matches = true;
                if let Some(ws_id) = filter.workspace_id {
                    matches = matches && t.workspace_id == ws_id;
                }
                if let Some(group_id) = filter.repository_group_id {
                    matches = matches && t.repository_group_id == group_id;
                }
                if let Some(status) = filter.status {
                    matches = matches && t.status == status;
                }
                matches
            })
            .cloned()
            .collect();

        let total = result.len() as u32;

        if let Some(offset) = filter.offset {
            result = result.into_iter().skip(offset as usize).collect();
        }
        if let Some(limit) = filter.limit {
            result = result.into_iter().take(limit as usize).collect();
        }

        Ok((result, total))
    }

    async fn update_unit_task(&self, task: UnitTask) -> TaskStoreResult<UnitTask> {
        let mut inner = self.inner.write().await;
        if !inner.unit_tasks.contains_key(&task.id) {
            return Err(TaskStoreError::not_found("UnitTask", task.id.to_string()));
        }
        inner.unit_tasks.insert(task.id, task.clone());
        Ok(task)
    }

    async fn delete_unit_task(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.unit_tasks.remove(&id).is_none() {
            return Err(TaskStoreError::not_found("UnitTask", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // SubTask operations
    // =========================================================================

    async fn create_sub_task(&self, sub_task: SubTask) -> TaskStoreResult<SubTask> {
        let mut inner = self.inner.write().await;
        if inner.sub_tasks.contains_key(&sub_task.id) {
            return Err(TaskStoreError::already_exists(
                "SubTask",
                sub_task.id.to_string(),
            ));
        }
        inner.sub_tasks.insert(sub_task.id, sub_task.clone());
        Ok(sub_task)
    }

    async fn get_sub_task(&self, id: Uuid) -> TaskStoreResult<Option<SubTask>> {
        let inner = self.inner.read().await;
        Ok(inner.sub_tasks.get(&id).cloned())
    }

    async fn list_sub_tasks(&self, unit_task_id: Uuid) -> TaskStoreResult<Vec<SubTask>> {
        let inner = self.inner.read().await;
        Ok(inner
            .sub_tasks
            .values()
            .filter(|s| s.unit_task_id == unit_task_id)
            .cloned()
            .collect())
    }

    async fn update_sub_task(&self, sub_task: SubTask) -> TaskStoreResult<SubTask> {
        let mut inner = self.inner.write().await;
        if !inner.sub_tasks.contains_key(&sub_task.id) {
            return Err(TaskStoreError::not_found(
                "SubTask",
                sub_task.id.to_string(),
            ));
        }
        inner.sub_tasks.insert(sub_task.id, sub_task.clone());
        Ok(sub_task)
    }

    async fn delete_sub_task(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.sub_tasks.remove(&id).is_none() {
            return Err(TaskStoreError::not_found("SubTask", id.to_string()));
        }
        Ok(())
    }

    async fn get_next_queued_sub_task(&self) -> TaskStoreResult<Option<SubTask>> {
        let inner = self.inner.read().await;
        // Return the oldest subtask with status=Queued
        let mut queued: Vec<&SubTask> = inner
            .sub_tasks
            .values()
            .filter(|s| s.status == SubTaskStatus::Queued)
            .collect();
        queued.sort_by_key(|s| s.created_at);
        Ok(queued.into_iter().next().cloned())
    }

    // =========================================================================
    // AgentSession operations
    // =========================================================================

    async fn create_agent_session(&self, session: AgentSession) -> TaskStoreResult<AgentSession> {
        let mut inner = self.inner.write().await;
        if inner.agent_sessions.contains_key(&session.id) {
            return Err(TaskStoreError::already_exists(
                "AgentSession",
                session.id.to_string(),
            ));
        }
        inner.agent_sessions.insert(session.id, session.clone());
        Ok(session)
    }

    async fn get_agent_session(&self, id: Uuid) -> TaskStoreResult<Option<AgentSession>> {
        let inner = self.inner.read().await;
        Ok(inner.agent_sessions.get(&id).cloned())
    }

    async fn list_agent_sessions(&self, sub_task_id: Uuid) -> TaskStoreResult<Vec<AgentSession>> {
        let inner = self.inner.read().await;
        Ok(inner
            .agent_sessions
            .values()
            .filter(|s| s.sub_task_id == sub_task_id)
            .cloned()
            .collect())
    }

    async fn update_agent_session(&self, session: AgentSession) -> TaskStoreResult<AgentSession> {
        let mut inner = self.inner.write().await;
        if !inner.agent_sessions.contains_key(&session.id) {
            return Err(TaskStoreError::not_found(
                "AgentSession",
                session.id.to_string(),
            ));
        }
        inner.agent_sessions.insert(session.id, session.clone());
        Ok(session)
    }

    async fn delete_agent_session(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.agent_sessions.remove(&id).is_none() {
            return Err(TaskStoreError::not_found("AgentSession", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // SessionOutputEvent operations
    // =========================================================================

    async fn append_session_output(
        &self,
        event: SessionOutputEvent,
    ) -> TaskStoreResult<SessionOutputEvent> {
        let mut inner = self.inner.write().await;
        inner.session_outputs.push(event.clone());
        Ok(event)
    }

    async fn list_session_outputs(
        &self,
        session_id: Uuid,
        since_sequence: Option<u64>,
    ) -> TaskStoreResult<Vec<SessionOutputEvent>> {
        let inner = self.inner.read().await;
        let result = inner
            .session_outputs
            .iter()
            .filter(|e| {
                e.session_id == session_id && since_sequence.map_or(true, |seq| e.sequence > seq)
            })
            .cloned()
            .collect();
        Ok(result)
    }

    // =========================================================================
    // PullRequestTracking operations
    // =========================================================================

    async fn create_pr_tracking(
        &self,
        pr: PullRequestTracking,
    ) -> TaskStoreResult<PullRequestTracking> {
        let mut inner = self.inner.write().await;
        if inner.pr_trackings.contains_key(&pr.id) {
            return Err(TaskStoreError::already_exists(
                "PullRequestTracking",
                pr.id.to_string(),
            ));
        }
        inner.pr_trackings.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn get_pr_tracking(&self, id: Uuid) -> TaskStoreResult<Option<PullRequestTracking>> {
        let inner = self.inner.read().await;
        Ok(inner.pr_trackings.get(&id).cloned())
    }

    async fn list_pr_trackings(
        &self,
        unit_task_id: Option<Uuid>,
    ) -> TaskStoreResult<Vec<PullRequestTracking>> {
        let inner = self.inner.read().await;
        Ok(inner
            .pr_trackings
            .values()
            .filter(|p| {
                if let Some(task_id) = unit_task_id {
                    p.unit_task_id == task_id
                } else {
                    true
                }
            })
            .cloned()
            .collect())
    }

    async fn update_pr_tracking(
        &self,
        pr: PullRequestTracking,
    ) -> TaskStoreResult<PullRequestTracking> {
        let mut inner = self.inner.write().await;
        if !inner.pr_trackings.contains_key(&pr.id) {
            return Err(TaskStoreError::not_found(
                "PullRequestTracking",
                pr.id.to_string(),
            ));
        }
        inner.pr_trackings.insert(pr.id, pr.clone());
        Ok(pr)
    }

    async fn delete_pr_tracking(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.pr_trackings.remove(&id).is_none() {
            return Err(TaskStoreError::not_found(
                "PullRequestTracking",
                id.to_string(),
            ));
        }
        Ok(())
    }

    // =========================================================================
    // ReviewAssistItem operations
    // =========================================================================

    async fn create_review_assist_item(
        &self,
        item: ReviewAssistItem,
    ) -> TaskStoreResult<ReviewAssistItem> {
        let mut inner = self.inner.write().await;
        if inner.review_assist_items.contains_key(&item.id) {
            return Err(TaskStoreError::already_exists(
                "ReviewAssistItem",
                item.id.to_string(),
            ));
        }
        inner.review_assist_items.insert(item.id, item.clone());
        Ok(item)
    }

    async fn get_review_assist_item(&self, id: Uuid) -> TaskStoreResult<Option<ReviewAssistItem>> {
        let inner = self.inner.read().await;
        Ok(inner.review_assist_items.get(&id).cloned())
    }

    async fn list_review_assist_items(
        &self,
        unit_task_id: Option<Uuid>,
    ) -> TaskStoreResult<Vec<ReviewAssistItem>> {
        let inner = self.inner.read().await;
        Ok(inner
            .review_assist_items
            .values()
            .filter(|i| {
                if let Some(task_id) = unit_task_id {
                    i.unit_task_id == task_id
                } else {
                    true
                }
            })
            .cloned()
            .collect())
    }

    async fn update_review_assist_item(
        &self,
        item: ReviewAssistItem,
    ) -> TaskStoreResult<ReviewAssistItem> {
        let mut inner = self.inner.write().await;
        if !inner.review_assist_items.contains_key(&item.id) {
            return Err(TaskStoreError::not_found(
                "ReviewAssistItem",
                item.id.to_string(),
            ));
        }
        inner.review_assist_items.insert(item.id, item.clone());
        Ok(item)
    }

    async fn delete_review_assist_item(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.review_assist_items.remove(&id).is_none() {
            return Err(TaskStoreError::not_found(
                "ReviewAssistItem",
                id.to_string(),
            ));
        }
        Ok(())
    }

    // =========================================================================
    // ReviewInlineComment operations
    // =========================================================================

    async fn create_review_inline_comment(
        &self,
        comment: ReviewInlineComment,
    ) -> TaskStoreResult<ReviewInlineComment> {
        let mut inner = self.inner.write().await;
        if inner.review_inline_comments.contains_key(&comment.id) {
            return Err(TaskStoreError::already_exists(
                "ReviewInlineComment",
                comment.id.to_string(),
            ));
        }
        inner
            .review_inline_comments
            .insert(comment.id, comment.clone());
        Ok(comment)
    }

    async fn get_review_inline_comment(
        &self,
        id: Uuid,
    ) -> TaskStoreResult<Option<ReviewInlineComment>> {
        let inner = self.inner.read().await;
        Ok(inner.review_inline_comments.get(&id).cloned())
    }

    async fn list_review_inline_comments(
        &self,
        unit_task_id: Uuid,
    ) -> TaskStoreResult<Vec<ReviewInlineComment>> {
        let inner = self.inner.read().await;
        Ok(inner
            .review_inline_comments
            .values()
            .filter(|c| c.unit_task_id == unit_task_id)
            .cloned()
            .collect())
    }

    async fn update_review_inline_comment(
        &self,
        comment: ReviewInlineComment,
    ) -> TaskStoreResult<ReviewInlineComment> {
        let mut inner = self.inner.write().await;
        if !inner.review_inline_comments.contains_key(&comment.id) {
            return Err(TaskStoreError::not_found(
                "ReviewInlineComment",
                comment.id.to_string(),
            ));
        }
        inner
            .review_inline_comments
            .insert(comment.id, comment.clone());
        Ok(comment)
    }

    async fn delete_review_inline_comment(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        if inner.review_inline_comments.remove(&id).is_none() {
            return Err(TaskStoreError::not_found(
                "ReviewInlineComment",
                id.to_string(),
            ));
        }
        Ok(())
    }

    // =========================================================================
    // BadgeTheme operations
    // =========================================================================

    async fn list_badge_themes(&self, workspace_id: Uuid) -> TaskStoreResult<Vec<BadgeTheme>> {
        let inner = self.inner.read().await;
        Ok(inner
            .badge_themes
            .iter()
            .filter(|((ws_id, _), _)| *ws_id == workspace_id)
            .map(|(_, theme)| theme.clone())
            .collect())
    }

    async fn upsert_badge_theme(&self, theme: BadgeTheme) -> TaskStoreResult<BadgeTheme> {
        let mut inner = self.inner.write().await;
        let key = (theme.workspace_id, action_type_key(theme.action_type));
        inner.badge_themes.insert(key, theme.clone());
        Ok(theme)
    }

    // =========================================================================
    // Notification operations
    // =========================================================================

    async fn create_notification(
        &self,
        notification: Notification,
    ) -> TaskStoreResult<Notification> {
        let mut inner = self.inner.write().await;
        if inner.notifications.contains_key(&notification.id) {
            return Err(TaskStoreError::already_exists(
                "Notification",
                notification.id.to_string(),
            ));
        }
        inner
            .notifications
            .insert(notification.id, notification.clone());
        Ok(notification)
    }

    async fn get_notification(&self, id: Uuid) -> TaskStoreResult<Option<Notification>> {
        let inner = self.inner.read().await;
        Ok(inner.notifications.get(&id).cloned())
    }

    async fn list_notifications(
        &self,
        workspace_id: Option<Uuid>,
        unread_only: bool,
    ) -> TaskStoreResult<Vec<Notification>> {
        let inner = self.inner.read().await;
        Ok(inner
            .notifications
            .values()
            .filter(|n| {
                let ws_match = workspace_id.map_or(true, |ws_id| n.workspace_id == ws_id);
                let read_match = !unread_only || n.read_at.is_none();
                ws_match && read_match
            })
            .cloned()
            .collect())
    }

    async fn mark_notification_read(&self, id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        let notification = inner
            .notifications
            .get_mut(&id)
            .ok_or_else(|| TaskStoreError::not_found("Notification", id.to_string()))?;
        notification.read_at = Some(Utc::now());
        Ok(())
    }

    async fn mark_all_notifications_read(&self, workspace_id: Uuid) -> TaskStoreResult<()> {
        let mut inner = self.inner.write().await;
        let now = Utc::now();
        for notification in inner.notifications.values_mut() {
            if notification.workspace_id == workspace_id && notification.read_at.is_none() {
                notification.read_at = Some(now);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use entities::{
        ActionType, AiAgentType, BadgeColorKey, NotificationType, Repository, RepositoryGroup,
        SubTaskType, UnitTask, VcsProviderType, Workspace,
    };

    use super::*;

    #[tokio::test]
    async fn test_workspace_crud() {
        let store = MemoryTaskStore::new();

        let workspace = Workspace::new("Test Workspace");
        let created = store.create_workspace(workspace.clone()).await.unwrap();
        assert_eq!(created.name, "Test Workspace");

        let fetched = store.get_workspace(created.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Test Workspace");

        let (workspaces, count) = store
            .list_workspaces(WorkspaceFilter::default())
            .await
            .unwrap();
        assert_eq!(count, 1);
        assert_eq!(workspaces.len(), 1);

        store.delete_workspace(created.id).await.unwrap();
        assert!(store.get_workspace(created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_repository_crud() {
        let store = MemoryTaskStore::new();

        let workspace = Workspace::new("Test Workspace");
        let workspace = store.create_workspace(workspace).await.unwrap();

        let repo = Repository::new(
            workspace.id,
            "test-repo",
            "https://github.com/test/test-repo",
            VcsProviderType::Github,
        );
        let created = store.create_repository(repo).await.unwrap();
        assert_eq!(created.name, "test-repo");

        let fetched = store.get_repository(created.id).await.unwrap().unwrap();
        assert_eq!(fetched.remote_url, "https://github.com/test/test-repo");

        let filter = RepositoryFilter {
            workspace_id: Some(workspace.id),
            ..Default::default()
        };
        let (repos, count) = store.list_repositories(filter).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(repos.len(), 1);

        store.delete_repository(created.id).await.unwrap();
        assert!(store.get_repository(created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_unit_task_crud() {
        let store = MemoryTaskStore::new();

        let workspace = Workspace::new("Test Workspace");
        let workspace = store.create_workspace(workspace).await.unwrap();
        let group = RepositoryGroup::new(workspace.id);
        let group = store.create_repository_group(group).await.unwrap();

        let task = UnitTask::new(workspace.id, group.id, "Fix the bug", "Fix auth bug");
        let created = store.create_unit_task(task).await.unwrap();
        assert_eq!(created.title, "Fix the bug");

        let fetched = store.get_unit_task(created.id).await.unwrap().unwrap();
        assert_eq!(fetched.prompt, "Fix auth bug");

        let (tasks, count) = store.list_unit_tasks(TaskFilter::default()).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(tasks.len(), 1);

        store.delete_unit_task(created.id).await.unwrap();
        assert!(store.get_unit_task(created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_sub_task_crud() {
        let store = MemoryTaskStore::new();

        let workspace = Workspace::new("Test");
        let workspace = store.create_workspace(workspace).await.unwrap();
        let group = RepositoryGroup::new(workspace.id);
        let group = store.create_repository_group(group).await.unwrap();
        let task = UnitTask::new(workspace.id, group.id, "Task", "Do something");
        let task = store.create_unit_task(task).await.unwrap();

        let sub_task = SubTask::new(task.id, SubTaskType::InitialImplementation, "Implement it");
        let created = store.create_sub_task(sub_task).await.unwrap();
        assert_eq!(created.unit_task_id, task.id);

        let sub_tasks = store.list_sub_tasks(task.id).await.unwrap();
        assert_eq!(sub_tasks.len(), 1);

        // Test get_next_queued_sub_task
        let next = store.get_next_queued_sub_task().await.unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, created.id);

        store.delete_sub_task(created.id).await.unwrap();
        assert!(store.get_sub_task(created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_agent_session_crud() {
        let store = MemoryTaskStore::new();

        let workspace = Workspace::new("Test");
        let workspace = store.create_workspace(workspace).await.unwrap();
        let group = RepositoryGroup::new(workspace.id);
        let group = store.create_repository_group(group).await.unwrap();
        let task = UnitTask::new(workspace.id, group.id, "Task", "Do something");
        let task = store.create_unit_task(task).await.unwrap();
        let sub_task = SubTask::new(task.id, SubTaskType::InitialImplementation, "Implement");
        let sub_task = store.create_sub_task(sub_task).await.unwrap();

        let session = AgentSession::new(sub_task.id, AiAgentType::ClaudeCode);
        let created = store.create_agent_session(session).await.unwrap();
        assert_eq!(created.sub_task_id, sub_task.id);

        let sessions = store.list_agent_sessions(sub_task.id).await.unwrap();
        assert_eq!(sessions.len(), 1);

        store.delete_agent_session(created.id).await.unwrap();
        assert!(store.get_agent_session(created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_session_output_events() {
        let store = MemoryTaskStore::new();
        let session_id = Uuid::new_v4();

        let event1 = SessionOutputEvent::new(
            session_id,
            1,
            entities::SessionOutputKind::Text,
            "First output",
        );
        let event2 = SessionOutputEvent::new(
            session_id,
            2,
            entities::SessionOutputKind::Text,
            "Second output",
        );

        store.append_session_output(event1).await.unwrap();
        store.append_session_output(event2).await.unwrap();

        let all = store.list_session_outputs(session_id, None).await.unwrap();
        assert_eq!(all.len(), 2);

        let after_first = store
            .list_session_outputs(session_id, Some(1))
            .await
            .unwrap();
        assert_eq!(after_first.len(), 1);
        assert_eq!(after_first[0].sequence, 2);
    }

    #[tokio::test]
    async fn test_badge_theme_upsert() {
        let store = MemoryTaskStore::new();
        let workspace_id = Uuid::new_v4();

        let theme = BadgeTheme::new(workspace_id, ActionType::CiFailed, BadgeColorKey::Red);
        store.upsert_badge_theme(theme).await.unwrap();

        let themes = store.list_badge_themes(workspace_id).await.unwrap();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].color_key, BadgeColorKey::Red);

        // Upsert with a different color
        let updated_theme =
            BadgeTheme::new(workspace_id, ActionType::CiFailed, BadgeColorKey::Orange);
        store.upsert_badge_theme(updated_theme).await.unwrap();

        let themes = store.list_badge_themes(workspace_id).await.unwrap();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].color_key, BadgeColorKey::Orange);
    }

    #[tokio::test]
    async fn test_notifications() {
        let store = MemoryTaskStore::new();
        let workspace_id = Uuid::new_v4();

        let notif = Notification::new(
            workspace_id,
            NotificationType::TaskActionRequired,
            "Action Required",
            "Your task needs attention",
        );
        let created = store.create_notification(notif).await.unwrap();
        assert!(!created.is_read());

        let unread = store
            .list_notifications(Some(workspace_id), true)
            .await
            .unwrap();
        assert_eq!(unread.len(), 1);

        store.mark_notification_read(created.id).await.unwrap();

        let unread_after = store
            .list_notifications(Some(workspace_id), true)
            .await
            .unwrap();
        assert_eq!(unread_after.len(), 0);

        let fetched = store.get_notification(created.id).await.unwrap().unwrap();
        assert!(fetched.is_read());
    }

    #[tokio::test]
    async fn test_delete_repository_removes_from_groups() {
        let store = MemoryTaskStore::new();

        let workspace = Workspace::new("Test Workspace");
        let workspace = store.create_workspace(workspace).await.unwrap();

        let repo1 = Repository::new(
            workspace.id,
            "repo1",
            "https://github.com/test/repo1",
            VcsProviderType::Github,
        );
        let repo1 = store.create_repository(repo1).await.unwrap();

        let repo2 = Repository::new(
            workspace.id,
            "repo2",
            "https://github.com/test/repo2",
            VcsProviderType::Github,
        );
        let repo2 = store.create_repository(repo2).await.unwrap();

        let mut group = RepositoryGroup::new(workspace.id);
        group.add_repository(repo1.id);
        group.add_repository(repo2.id);
        let group = store.create_repository_group(group).await.unwrap();

        let fetched_group = store.get_repository_group(group.id).await.unwrap().unwrap();
        assert_eq!(fetched_group.repository_ids.len(), 2);

        store.delete_repository(repo1.id).await.unwrap();

        let fetched_group = store.get_repository_group(group.id).await.unwrap().unwrap();
        assert_eq!(fetched_group.repository_ids.len(), 1);
        assert!(!fetched_group.repository_ids.contains(&repo1.id));
        assert!(fetched_group.repository_ids.contains(&repo2.id));
    }
}
