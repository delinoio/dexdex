//! Remote client for making API calls to the main server.
//!
//! This module provides a client for making HTTP requests to the main server
//! when the app is running in remote mode.

use rpc_protocol::{
    requests::*, responses::*, AiAgentType as RpcAiAgentType,
    CompositeTaskStatus as RpcCompositeTaskStatus, UnitTaskStatus as RpcUnitTaskStatus,
};
use serde::de::DeserializeOwned;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Remote client for making API calls to the main server.
///
/// # Authentication
///
/// The remote client supports JWT authentication via the `with_auth_token`
/// method. According to the design docs, JWT authentication is required when
/// connecting to a remote DeliDev server in production. The token is obtained
/// after successful OIDC authentication (see `docs/design.md` for details).
///
/// Currently, the auth token is not automatically injected because:
/// 1. The OIDC authentication flow is not yet implemented in the Tauri client
/// 2. Development/testing often uses servers with authentication disabled
///
/// Once OIDC authentication is implemented, the AppState should store the JWT
/// token after login and pass it when creating the RemoteClient:
/// ```ignore
/// let client = RemoteClient::new(http_client, base_url)
///     .with_auth_token(state.auth_token.clone());
/// ```
///
/// TODO: Implement OIDC authentication flow and automatic token injection
pub struct RemoteClient {
    http_client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
}

impl RemoteClient {
    /// Creates a new remote client without authentication.
    ///
    /// For servers requiring authentication, use `with_auth_token` to set the
    /// JWT token.
    pub fn new(http_client: reqwest::Client, base_url: String) -> Self {
        Self {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: None,
        }
    }

    /// Sets the authentication token (JWT) for authenticated requests.
    ///
    /// The token will be sent as a Bearer token in the Authorization header
    /// for all subsequent API requests.
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

    /// Updates the plan for a composite task with new feedback.
    pub async fn update_plan(&self, request: UpdatePlanRequest) -> AppResult<UpdatePlanResponse> {
        self.post("/api/task/update-plan", &request).await
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

    /// Gets a repository group by ID.
    pub async fn get_repository_group(
        &self,
        request: GetRepositoryGroupRequest,
    ) -> AppResult<GetRepositoryGroupResponse> {
        self.post("/api/repository-group/get", &request).await
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
// Input validation helpers
// ============================================================================

/// Maximum allowed length for text input fields.
const MAX_TEXT_LENGTH: usize = 10000;
/// Maximum allowed length for name fields.
const MAX_NAME_LENGTH: usize = 255;

/// Validates that a required string field is not empty after trimming.
pub fn validate_required_string(value: &str, field_name: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::InvalidRequest(format!(
            "{} cannot be empty",
            field_name
        )));
    }
    Ok(())
}

/// Validates that a string field does not exceed the maximum length.
pub fn validate_string_length(value: &str, field_name: &str, max_length: usize) -> AppResult<()> {
    if value.len() > max_length {
        return Err(AppError::InvalidRequest(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_length
        )));
    }
    Ok(())
}

/// Validates a name field (required, max 255 chars).
pub fn validate_name(value: &str, field_name: &str) -> AppResult<()> {
    validate_required_string(value, field_name)?;
    validate_string_length(value, field_name, MAX_NAME_LENGTH)
}

/// Validates an optional name field (max 255 chars if present).
pub fn validate_optional_name(value: Option<&str>, field_name: &str) -> AppResult<()> {
    if let Some(v) = value {
        if !v.trim().is_empty() {
            validate_string_length(v, field_name, MAX_NAME_LENGTH)?;
        }
    }
    Ok(())
}

/// Validates a text field (required, max 10000 chars).
pub fn validate_text(value: &str, field_name: &str) -> AppResult<()> {
    validate_required_string(value, field_name)?;
    validate_string_length(value, field_name, MAX_TEXT_LENGTH)
}

/// Validates an optional text field (max 10000 chars if present).
pub fn validate_optional_text(value: Option<&str>, field_name: &str) -> AppResult<()> {
    if let Some(v) = value {
        if !v.trim().is_empty() {
            validate_string_length(v, field_name, MAX_TEXT_LENGTH)?;
        }
    }
    Ok(())
}

/// Validates that a UUID string is well-formed (for remote mode where we send
/// it as string).
pub fn validate_uuid_string(value: &str, field_name: &str) -> AppResult<()> {
    validate_required_string(value, field_name)?;
    value
        .parse::<Uuid>()
        .map_err(|_| AppError::InvalidRequest(format!("Invalid {} format", field_name)))?;
    Ok(())
}

/// Validates an optional UUID string.
pub fn validate_optional_uuid_string(value: Option<&str>, field_name: &str) -> AppResult<()> {
    if let Some(v) = value {
        if !v.trim().is_empty() {
            validate_uuid_string(v, field_name)?;
        }
    }
    Ok(())
}

// ============================================================================
// Type conversion helpers
// ============================================================================

/// Parses a UUID from a string, returning an error with context if parsing
/// fails.
fn parse_uuid(id_str: &str, field_name: &str) -> AppResult<Uuid> {
    id_str.parse().map_err(|e| {
        warn!("Failed to parse {} UUID '{}': {}", field_name, id_str, e);
        AppError::Remote(format!(
            "Server returned invalid {}: '{}'",
            field_name, id_str
        ))
    })
}

/// Parses a list of UUIDs, logging errors for invalid entries and collecting
/// valid ones.
///
/// This is used for list fields where partial results are acceptable (e.g.,
/// non-critical relationships). Invalid UUIDs are logged at error level to help
/// identify potential server bugs or data corruption issues.
///
/// # Arguments
///
/// * `ids` - The list of UUID strings to parse
/// * `field_name` - The name of the field being parsed (for logging context)
///
/// # Returns
///
/// A vector of successfully parsed UUIDs. Invalid UUIDs are excluded but
/// logged.
fn parse_uuid_list(ids: &[String], field_name: &str) -> Vec<Uuid> {
    let mut valid_uuids = Vec::with_capacity(ids.len());
    let mut invalid_count = 0;

    for id in ids {
        match id.parse() {
            Ok(uuid) => valid_uuids.push(uuid),
            Err(e) => {
                error!(
                    "Invalid {} UUID '{}': {} - this may indicate a server bug or data corruption",
                    field_name, id, e
                );
                invalid_count += 1;
            }
        }
    }

    if invalid_count > 0 {
        error!(
            "Skipped {} invalid {} UUID(s) out of {} total",
            invalid_count,
            field_name,
            ids.len()
        );
    }

    valid_uuids
}

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
///
/// Returns an error if required UUID fields cannot be parsed.
pub fn rpc_to_entity_unit_task(rpc: rpc_protocol::UnitTask) -> AppResult<entities::UnitTask> {
    Ok(entities::UnitTask {
        id: parse_uuid(&rpc.id, "task id")?,
        repository_group_id: parse_uuid(&rpc.repository_group_id, "repository group id")?,
        agent_task_id: parse_uuid(&rpc.agent_task_id, "agent task id")?,
        prompt: rpc.prompt,
        title: rpc.title,
        branch_name: rpc.branch_name,
        linked_pr_url: rpc.linked_pr_url,
        base_commit: rpc.base_commit,
        end_commit: rpc.end_commit,
        auto_fix_task_ids: parse_uuid_list(&rpc.auto_fix_task_ids, "auto fix task id"),
        status: rpc_to_entity_unit_status(rpc.status),
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    })
}

/// Converts RPC CompositeTask to entity CompositeTask.
///
/// Returns an error if required UUID fields cannot be parsed.
pub fn rpc_to_entity_composite_task(
    rpc: rpc_protocol::CompositeTask,
) -> AppResult<entities::CompositeTask> {
    Ok(entities::CompositeTask {
        id: parse_uuid(&rpc.id, "composite task id")?,
        repository_group_id: parse_uuid(&rpc.repository_group_id, "repository group id")?,
        planning_task_id: parse_uuid(&rpc.planning_task_id, "planning task id")?,
        prompt: rpc.prompt,
        title: rpc.title,
        plan_yaml: rpc.plan_yaml,
        update_plan_feedback: rpc.update_plan_feedback,
        node_ids: parse_uuid_list(&rpc.node_ids, "node id"),
        status: rpc_to_entity_composite_status(rpc.status),
        execution_agent_type: rpc.execution_agent_type.map(rpc_to_entity_agent_type),
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    })
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
        RpcUnitTaskStatus::Cancelled => entities::UnitTaskStatus::Cancelled,
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
        entities::UnitTaskStatus::Cancelled => RpcUnitTaskStatus::Cancelled,
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
        RpcCompositeTaskStatus::Failed => entities::CompositeTaskStatus::Failed,
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
        entities::CompositeTaskStatus::Failed => RpcCompositeTaskStatus::Failed,
    }
}

/// Converts RPC AiAgentType to entity AiAgentType.
pub fn rpc_to_entity_agent_type(agent_type: RpcAiAgentType) -> entities::AiAgentType {
    match agent_type {
        RpcAiAgentType::Unspecified | RpcAiAgentType::ClaudeCode => {
            entities::AiAgentType::ClaudeCode
        }
        RpcAiAgentType::OpenCode => entities::AiAgentType::OpenCode,
        RpcAiAgentType::GeminiCli => entities::AiAgentType::GeminiCli,
        RpcAiAgentType::CodexCli => entities::AiAgentType::CodexCli,
        RpcAiAgentType::Aider => entities::AiAgentType::Aider,
        RpcAiAgentType::Amp => entities::AiAgentType::Amp,
    }
}

/// Converts RPC Repository to entity Repository.
///
/// Returns an error if required UUID fields cannot be parsed.
pub fn rpc_to_entity_repository(rpc: rpc_protocol::Repository) -> AppResult<entities::Repository> {
    Ok(entities::Repository {
        id: parse_uuid(&rpc.id, "repository id")?,
        workspace_id: parse_uuid(&rpc.workspace_id, "workspace id")?,
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
    })
}

/// Converts RPC RepositoryGroup to entity RepositoryGroup.
///
/// Returns an error if required UUID fields cannot be parsed.
pub fn rpc_to_entity_repository_group(
    rpc: rpc_protocol::RepositoryGroup,
) -> AppResult<entities::RepositoryGroup> {
    Ok(entities::RepositoryGroup {
        id: parse_uuid(&rpc.id, "repository group id")?,
        workspace_id: parse_uuid(&rpc.workspace_id, "workspace id")?,
        name: rpc.name,
        repository_ids: parse_uuid_list(&rpc.repository_ids, "repository id"),
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    })
}

/// Converts RPC Workspace to entity Workspace.
///
/// Returns an error if required UUID fields cannot be parsed.
pub fn rpc_to_entity_workspace(rpc: rpc_protocol::Workspace) -> AppResult<entities::Workspace> {
    // For user_id, log a warning if parsing fails but don't fail the entire
    // conversion since user_id is optional and may legitimately be missing or
    // invalid in some contexts
    let user_id = rpc.user_id.as_ref().and_then(|id| match id.parse() {
        Ok(uuid) => Some(uuid),
        Err(e) => {
            warn!("Failed to parse user_id UUID '{}': {}", id, e);
            None
        }
    });

    Ok(entities::Workspace {
        id: parse_uuid(&rpc.id, "workspace id")?,
        user_id,
        name: rpc.name,
        description: rpc.description,
        created_at: rpc.created_at,
        updated_at: rpc.updated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Input Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_required_string_valid() {
        assert!(validate_required_string("hello", "field").is_ok());
        assert!(validate_required_string("  hello  ", "field").is_ok());
    }

    #[test]
    fn test_validate_required_string_empty() {
        assert!(validate_required_string("", "field").is_err());
        assert!(validate_required_string("   ", "field").is_err());
        assert!(validate_required_string("\t\n", "field").is_err());
    }

    #[test]
    fn test_validate_string_length_within_limit() {
        assert!(validate_string_length("hello", "field", 10).is_ok());
        assert!(validate_string_length("hello", "field", 5).is_ok());
    }

    #[test]
    fn test_validate_string_length_exceeds_limit() {
        let result = validate_string_length("hello world", "field", 5);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds maximum length"));
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("My Workspace", "name").is_ok());
        assert!(validate_name("test", "name").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("", "name").is_err());
        assert!(validate_name("   ", "name").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(300);
        let result = validate_name(&long_name, "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_optional_name_none() {
        assert!(validate_optional_name(None, "name").is_ok());
    }

    #[test]
    fn test_validate_optional_name_some_valid() {
        assert!(validate_optional_name(Some("test"), "name").is_ok());
    }

    #[test]
    fn test_validate_optional_name_some_too_long() {
        let long_name = "a".repeat(300);
        let result = validate_optional_name(Some(&long_name), "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_text_valid() {
        assert!(validate_text("This is a prompt", "prompt").is_ok());
    }

    #[test]
    fn test_validate_text_empty() {
        assert!(validate_text("", "prompt").is_err());
    }

    #[test]
    fn test_validate_text_too_long() {
        let long_text = "a".repeat(20000);
        let result = validate_text(&long_text, "prompt");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_uuid_string_valid() {
        assert!(validate_uuid_string("550e8400-e29b-41d4-a716-446655440000", "id").is_ok());
    }

    #[test]
    fn test_validate_uuid_string_invalid() {
        assert!(validate_uuid_string("not-a-uuid", "id").is_err());
        assert!(validate_uuid_string("", "id").is_err());
    }

    #[test]
    fn test_validate_optional_uuid_string_none() {
        assert!(validate_optional_uuid_string(None, "id").is_ok());
    }

    #[test]
    fn test_validate_optional_uuid_string_valid() {
        assert!(
            validate_optional_uuid_string(Some("550e8400-e29b-41d4-a716-446655440000"), "id")
                .is_ok()
        );
    }

    #[test]
    fn test_validate_optional_uuid_string_invalid() {
        assert!(validate_optional_uuid_string(Some("not-a-uuid"), "id").is_err());
    }

    // =========================================================================
    // UUID Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_uuid_valid() {
        let result = parse_uuid("550e8400-e29b-41d4-a716-446655440000", "test");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().to_string(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("not-a-uuid", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid"));
    }

    #[test]
    fn test_parse_uuid_list_all_valid() {
        let ids = vec![
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
        ];
        let result = parse_uuid_list(&ids, "test");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_uuid_list_some_invalid() {
        let ids = vec![
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "invalid".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
        ];
        let result = parse_uuid_list(&ids, "test");
        // Invalid UUIDs are skipped
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_uuid_list_empty() {
        let ids: Vec<String> = vec![];
        let result = parse_uuid_list(&ids, "test");
        assert!(result.is_empty());
    }

    // =========================================================================
    // Type Conversion Tests
    // =========================================================================

    #[test]
    fn test_entity_to_rpc_agent_type() {
        assert!(matches!(
            entity_to_rpc_agent_type(entities::AiAgentType::ClaudeCode),
            RpcAiAgentType::ClaudeCode
        ));
        assert!(matches!(
            entity_to_rpc_agent_type(entities::AiAgentType::OpenCode),
            RpcAiAgentType::OpenCode
        ));
        assert!(matches!(
            entity_to_rpc_agent_type(entities::AiAgentType::Aider),
            RpcAiAgentType::Aider
        ));
    }

    #[test]
    fn test_rpc_to_entity_agent_type() {
        assert!(matches!(
            rpc_to_entity_agent_type(RpcAiAgentType::ClaudeCode),
            entities::AiAgentType::ClaudeCode
        ));
        assert!(matches!(
            rpc_to_entity_agent_type(RpcAiAgentType::OpenCode),
            entities::AiAgentType::OpenCode
        ));
        // Unspecified defaults to ClaudeCode
        assert!(matches!(
            rpc_to_entity_agent_type(RpcAiAgentType::Unspecified),
            entities::AiAgentType::ClaudeCode
        ));
    }

    #[test]
    fn test_rpc_to_entity_unit_status() {
        assert!(matches!(
            rpc_to_entity_unit_status(RpcUnitTaskStatus::InProgress),
            entities::UnitTaskStatus::InProgress
        ));
        assert!(matches!(
            rpc_to_entity_unit_status(RpcUnitTaskStatus::InReview),
            entities::UnitTaskStatus::InReview
        ));
        assert!(matches!(
            rpc_to_entity_unit_status(RpcUnitTaskStatus::Done),
            entities::UnitTaskStatus::Done
        ));
        // Unspecified defaults to InProgress
        assert!(matches!(
            rpc_to_entity_unit_status(RpcUnitTaskStatus::Unspecified),
            entities::UnitTaskStatus::InProgress
        ));
    }

    #[test]
    fn test_entity_to_rpc_unit_status() {
        assert!(matches!(
            entity_to_rpc_unit_status(entities::UnitTaskStatus::InProgress),
            RpcUnitTaskStatus::InProgress
        ));
        assert!(matches!(
            entity_to_rpc_unit_status(entities::UnitTaskStatus::Approved),
            RpcUnitTaskStatus::Approved
        ));
        assert!(matches!(
            entity_to_rpc_unit_status(entities::UnitTaskStatus::Failed),
            RpcUnitTaskStatus::Failed
        ));
    }

    #[test]
    fn test_rpc_to_entity_composite_status() {
        assert!(matches!(
            rpc_to_entity_composite_status(RpcCompositeTaskStatus::Planning),
            entities::CompositeTaskStatus::Planning
        ));
        assert!(matches!(
            rpc_to_entity_composite_status(RpcCompositeTaskStatus::InProgress),
            entities::CompositeTaskStatus::InProgress
        ));
        assert!(matches!(
            rpc_to_entity_composite_status(RpcCompositeTaskStatus::Failed),
            entities::CompositeTaskStatus::Failed
        ));
        // Unspecified defaults to Planning
        assert!(matches!(
            rpc_to_entity_composite_status(RpcCompositeTaskStatus::Unspecified),
            entities::CompositeTaskStatus::Planning
        ));
    }

    #[test]
    fn test_entity_to_rpc_composite_status() {
        assert!(matches!(
            entity_to_rpc_composite_status(entities::CompositeTaskStatus::Planning),
            RpcCompositeTaskStatus::Planning
        ));
        assert!(matches!(
            entity_to_rpc_composite_status(entities::CompositeTaskStatus::Done),
            RpcCompositeTaskStatus::Done
        ));
        assert!(matches!(
            entity_to_rpc_composite_status(entities::CompositeTaskStatus::Rejected),
            RpcCompositeTaskStatus::Rejected
        ));
        assert!(matches!(
            entity_to_rpc_composite_status(entities::CompositeTaskStatus::Failed),
            RpcCompositeTaskStatus::Failed
        ));
    }
}
