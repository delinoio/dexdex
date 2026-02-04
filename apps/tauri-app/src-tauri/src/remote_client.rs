//! Remote client for making API calls to the main server.
//!
//! This module provides a client for making HTTP requests to the main server
//! when the app is running in remote mode.

use rpc_protocol::{
    requests::*, responses::*, AiAgentType as RpcAiAgentType,
    CompositeTaskStatus as RpcCompositeTaskStatus, UnitTaskStatus as RpcUnitTaskStatus,
};
use serde::de::DeserializeOwned;
use tracing::{debug, error};

use crate::error::{AppError, AppResult};

/// Remote client for making API calls to the main server.
pub struct RemoteClient {
    http_client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
}

impl RemoteClient {
    /// Creates a new remote client.
    pub fn new(http_client: reqwest::Client, base_url: String) -> Self {
        Self {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: None,
        }
    }

    /// Sets the authentication token.
    #[allow(dead_code)]
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Makes a POST request to the API.
    async fn post<Req, Res>(&self, path: &str, request: &Req) -> AppResult<Res>
    where
        Req: serde::Serialize,
        Res: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        debug!("Making POST request to {}", url);

        let mut req_builder = self.http_client.post(&url).json(request);

        if let Some(ref token) = self.auth_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = req_builder.send().await.map_err(|e| {
            error!("Failed to send request to {}: {}", url, e);
            AppError::Remote(format!("Failed to connect to server: {}", e))
        })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Server returned error {}: {}", status, error_text);
            return Err(AppError::Remote(format!(
                "Server error ({}): {}",
                status, error_text
            )));
        }

        let result = response.json::<Res>().await.map_err(|e| {
            error!("Failed to parse response from {}: {}", url, e);
            AppError::Remote(format!("Failed to parse server response: {}", e))
        })?;

        Ok(result)
    }

    // =========================================================================
    // Task API
    // =========================================================================

    /// Creates a new unit task.
    pub async fn create_unit_task(
        &self,
        request: CreateUnitTaskRequest,
    ) -> AppResult<CreateUnitTaskResponse> {
        self.post("/api/task/create-unit", &request).await
    }

    /// Creates a new composite task.
    pub async fn create_composite_task(
        &self,
        request: CreateCompositeTaskRequest,
    ) -> AppResult<CreateCompositeTaskResponse> {
        self.post("/api/task/create-composite", &request).await
    }

    /// Gets a task by ID.
    pub async fn get_task(&self, request: GetTaskRequest) -> AppResult<GetTaskResponse> {
        self.post("/api/task/get", &request).await
    }

    /// Lists tasks with filters.
    pub async fn list_tasks(&self, request: ListTasksRequest) -> AppResult<ListTasksResponse> {
        self.post("/api/task/list", &request).await
    }

    /// Approves a task.
    pub async fn approve_task(
        &self,
        request: ApproveTaskRequest,
    ) -> AppResult<ApproveTaskResponse> {
        self.post("/api/task/approve", &request).await
    }

    /// Rejects a task.
    pub async fn reject_task(&self, request: RejectTaskRequest) -> AppResult<RejectTaskResponse> {
        self.post("/api/task/reject", &request).await
    }

    /// Requests changes on a task.
    pub async fn request_changes(
        &self,
        request: RequestChangesRequest,
    ) -> AppResult<RequestChangesResponse> {
        self.post("/api/task/request-changes", &request).await
    }

    // =========================================================================
    // Repository API
    // =========================================================================

    /// Adds a repository.
    pub async fn add_repository(
        &self,
        request: AddRepositoryRequest,
    ) -> AppResult<AddRepositoryResponse> {
        self.post("/api/repository/add", &request).await
    }

    /// Lists repositories.
    pub async fn list_repositories(
        &self,
        request: ListRepositoriesRequest,
    ) -> AppResult<ListRepositoriesResponse> {
        self.post("/api/repository/list", &request).await
    }

    /// Removes a repository.
    pub async fn remove_repository(
        &self,
        request: RemoveRepositoryRequest,
    ) -> AppResult<RemoveRepositoryResponse> {
        self.post("/api/repository/remove", &request).await
    }

    /// Creates a repository group.
    pub async fn create_repository_group(
        &self,
        request: CreateRepositoryGroupRequest,
    ) -> AppResult<CreateRepositoryGroupResponse> {
        self.post("/api/repository-group/create", &request).await
    }

    /// Lists repository groups.
    pub async fn list_repository_groups(
        &self,
        request: ListRepositoryGroupsRequest,
    ) -> AppResult<ListRepositoryGroupsResponse> {
        self.post("/api/repository-group/list", &request).await
    }

    /// Updates a repository group.
    pub async fn update_repository_group(
        &self,
        request: UpdateRepositoryGroupRequest,
    ) -> AppResult<UpdateRepositoryGroupResponse> {
        self.post("/api/repository-group/update", &request).await
    }

    /// Deletes a repository group.
    pub async fn delete_repository_group(
        &self,
        request: DeleteRepositoryGroupRequest,
    ) -> AppResult<DeleteRepositoryGroupResponse> {
        self.post("/api/repository-group/delete", &request).await
    }

    // =========================================================================
    // Workspace API
    // =========================================================================

    /// Creates a workspace.
    pub async fn create_workspace(
        &self,
        request: CreateWorkspaceRequest,
    ) -> AppResult<CreateWorkspaceResponse> {
        self.post("/api/workspace/create", &request).await
    }

    /// Lists workspaces.
    pub async fn list_workspaces(
        &self,
        request: ListWorkspacesRequest,
    ) -> AppResult<ListWorkspacesResponse> {
        self.post("/api/workspace/list", &request).await
    }

    /// Gets a workspace by ID.
    pub async fn get_workspace(
        &self,
        request: GetWorkspaceRequest,
    ) -> AppResult<GetWorkspaceResponse> {
        self.post("/api/workspace/get", &request).await
    }

    /// Updates a workspace.
    pub async fn update_workspace(
        &self,
        request: UpdateWorkspaceRequest,
    ) -> AppResult<UpdateWorkspaceResponse> {
        self.post("/api/workspace/update", &request).await
    }

    /// Deletes a workspace.
    pub async fn delete_workspace(
        &self,
        request: DeleteWorkspaceRequest,
    ) -> AppResult<DeleteWorkspaceResponse> {
        self.post("/api/workspace/delete", &request).await
    }

    // =========================================================================
    // Session API
    // =========================================================================

    /// Submits TTY input.
    pub async fn submit_tty_input(
        &self,
        request: SubmitTtyInputRequest,
    ) -> AppResult<SubmitTtyInputResponse> {
        self.post("/api/session/submit-tty-input", &request).await
    }

    /// Gets session log.
    pub async fn get_session_log(&self, request: GetLogRequest) -> AppResult<GetLogResponse> {
        self.post("/api/session/get-log", &request).await
    }
}

// ============================================================================
// Type conversion helpers
// ============================================================================

/// Converts entity AiAgentType to RPC AiAgentType.
pub fn entity_to_rpc_agent_type(agent_type: entities::AiAgentType) -> RpcAiAgentType {
    match agent_type {
        entities::AiAgentType::ClaudeCode => RpcAiAgentType::ClaudeCode,
        entities::AiAgentType::OpenCode => RpcAiAgentType::OpenCode,
        entities::AiAgentType::GeminiCli => RpcAiAgentType::GeminiCli,
        entities::AiAgentType::CodexCli => RpcAiAgentType::CodexCli,
        entities::AiAgentType::Aider => RpcAiAgentType::Aider,
        entities::AiAgentType::Amp => RpcAiAgentType::Amp,
    }
}

/// Converts RPC UnitTask to entity UnitTask.
pub fn rpc_to_entity_unit_task(rpc: rpc_protocol::UnitTask) -> entities::UnitTask {
    entities::UnitTask {
        id: rpc.id.parse().unwrap_or_default(),
        repository_group_id: rpc.repository_group_id.parse().unwrap_or_default(),
        agent_task_id: rpc.agent_task_id.parse().unwrap_or_default(),
        prompt: rpc.prompt,
        title: rpc.title,
        branch_name: rpc.branch_name,
        linked_pr_url: rpc.linked_pr_url,
        base_commit: rpc.base_commit,
        end_commit: rpc.end_commit,
        auto_fix_task_ids: rpc
            .auto_fix_task_ids
            .iter()
            .filter_map(|id| id.parse().ok())
            .collect(),
        status: rpc_to_entity_unit_status(rpc.status),
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    }
}

/// Converts RPC CompositeTask to entity CompositeTask.
pub fn rpc_to_entity_composite_task(rpc: rpc_protocol::CompositeTask) -> entities::CompositeTask {
    entities::CompositeTask {
        id: rpc.id.parse().unwrap_or_default(),
        repository_group_id: rpc.repository_group_id.parse().unwrap_or_default(),
        planning_task_id: rpc.planning_task_id.parse().unwrap_or_default(),
        prompt: rpc.prompt,
        title: rpc.title,
        node_ids: rpc
            .node_ids
            .iter()
            .filter_map(|id| id.parse().ok())
            .collect(),
        status: rpc_to_entity_composite_status(rpc.status),
        execution_agent_type: rpc.execution_agent_type.map(rpc_to_entity_agent_type),
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    }
}

/// Converts RPC UnitTaskStatus to entity UnitTaskStatus.
pub fn rpc_to_entity_unit_status(status: RpcUnitTaskStatus) -> entities::UnitTaskStatus {
    match status {
        RpcUnitTaskStatus::Unspecified | RpcUnitTaskStatus::InProgress => {
            entities::UnitTaskStatus::InProgress
        }
        RpcUnitTaskStatus::InReview => entities::UnitTaskStatus::InReview,
        RpcUnitTaskStatus::Approved => entities::UnitTaskStatus::Approved,
        RpcUnitTaskStatus::PrOpen => entities::UnitTaskStatus::PrOpen,
        RpcUnitTaskStatus::Done => entities::UnitTaskStatus::Done,
        RpcUnitTaskStatus::Rejected => entities::UnitTaskStatus::Rejected,
        RpcUnitTaskStatus::Failed => entities::UnitTaskStatus::Failed,
    }
}

/// Converts entity UnitTaskStatus to RPC UnitTaskStatus.
pub fn entity_to_rpc_unit_status(status: entities::UnitTaskStatus) -> RpcUnitTaskStatus {
    match status {
        entities::UnitTaskStatus::InProgress => RpcUnitTaskStatus::InProgress,
        entities::UnitTaskStatus::InReview => RpcUnitTaskStatus::InReview,
        entities::UnitTaskStatus::Approved => RpcUnitTaskStatus::Approved,
        entities::UnitTaskStatus::PrOpen => RpcUnitTaskStatus::PrOpen,
        entities::UnitTaskStatus::Done => RpcUnitTaskStatus::Done,
        entities::UnitTaskStatus::Rejected => RpcUnitTaskStatus::Rejected,
        entities::UnitTaskStatus::Failed => RpcUnitTaskStatus::Failed,
    }
}

/// Converts RPC CompositeTaskStatus to entity CompositeTaskStatus.
pub fn rpc_to_entity_composite_status(
    status: RpcCompositeTaskStatus,
) -> entities::CompositeTaskStatus {
    match status {
        RpcCompositeTaskStatus::Unspecified | RpcCompositeTaskStatus::Planning => {
            entities::CompositeTaskStatus::Planning
        }
        RpcCompositeTaskStatus::PendingApproval => entities::CompositeTaskStatus::PendingApproval,
        RpcCompositeTaskStatus::InProgress => entities::CompositeTaskStatus::InProgress,
        RpcCompositeTaskStatus::Done => entities::CompositeTaskStatus::Done,
        RpcCompositeTaskStatus::Rejected => entities::CompositeTaskStatus::Rejected,
    }
}

/// Converts entity CompositeTaskStatus to RPC CompositeTaskStatus.
pub fn entity_to_rpc_composite_status(
    status: entities::CompositeTaskStatus,
) -> RpcCompositeTaskStatus {
    match status {
        entities::CompositeTaskStatus::Planning => RpcCompositeTaskStatus::Planning,
        entities::CompositeTaskStatus::PendingApproval => RpcCompositeTaskStatus::PendingApproval,
        entities::CompositeTaskStatus::InProgress => RpcCompositeTaskStatus::InProgress,
        entities::CompositeTaskStatus::Done => RpcCompositeTaskStatus::Done,
        entities::CompositeTaskStatus::Rejected => RpcCompositeTaskStatus::Rejected,
    }
}

/// Converts RPC AiAgentType to entity AiAgentType.
pub fn rpc_to_entity_agent_type(agent_type: RpcAiAgentType) -> entities::AiAgentType {
    match agent_type {
        RpcAiAgentType::Unspecified | RpcAiAgentType::ClaudeCode => entities::AiAgentType::ClaudeCode,
        RpcAiAgentType::OpenCode => entities::AiAgentType::OpenCode,
        RpcAiAgentType::GeminiCli => entities::AiAgentType::GeminiCli,
        RpcAiAgentType::CodexCli => entities::AiAgentType::CodexCli,
        RpcAiAgentType::Aider => entities::AiAgentType::Aider,
        RpcAiAgentType::Amp => entities::AiAgentType::Amp,
    }
}

/// Converts RPC Repository to entity Repository.
pub fn rpc_to_entity_repository(rpc: rpc_protocol::Repository) -> entities::Repository {
    entities::Repository {
        id: rpc.id.parse().unwrap_or_default(),
        workspace_id: rpc.workspace_id.parse().unwrap_or_default(),
        name: rpc.name,
        remote_url: rpc.remote_url,
        default_branch: rpc.default_branch,
        vcs_type: match rpc.vcs_type {
            rpc_protocol::VcsType::Git => entities::VcsType::Git,
            _ => entities::VcsType::Git,
        },
        vcs_provider_type: match rpc.vcs_provider_type {
            rpc_protocol::VcsProviderType::Github => entities::VcsProviderType::Github,
            rpc_protocol::VcsProviderType::Gitlab => entities::VcsProviderType::Gitlab,
            rpc_protocol::VcsProviderType::Bitbucket => entities::VcsProviderType::Bitbucket,
            _ => entities::VcsProviderType::Github,
        },
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    }
}

/// Converts RPC RepositoryGroup to entity RepositoryGroup.
pub fn rpc_to_entity_repository_group(
    rpc: rpc_protocol::RepositoryGroup,
) -> entities::RepositoryGroup {
    entities::RepositoryGroup {
        id: rpc.id.parse().unwrap_or_default(),
        workspace_id: rpc.workspace_id.parse().unwrap_or_default(),
        name: rpc.name,
        repository_ids: rpc
            .repository_ids
            .iter()
            .filter_map(|id| id.parse().ok())
            .collect(),
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    }
}

/// Converts RPC Workspace to entity Workspace.
pub fn rpc_to_entity_workspace(rpc: rpc_protocol::Workspace) -> entities::Workspace {
    entities::Workspace {
        id: rpc.id.parse().unwrap_or_default(),
        user_id: rpc.user_id.as_ref().and_then(|id| id.parse().ok()),
        name: rpc.name,
        description: rpc.description,
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    }
}
