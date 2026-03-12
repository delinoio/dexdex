//! Remote client for making API calls to the main server.
//!
//! This module provides a client for making HTTP requests to the main server
//! when the app is running in remote mode.

use rpc_protocol::{
    requests::*, responses::*, AiAgentType as RpcAiAgentType, UnitTaskStatus as RpcUnitTaskStatus,
};
use serde::de::DeserializeOwned;
use tracing::{debug, error};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Remote client for making API calls to the main server.
pub struct RemoteClient {
    http_client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
}

impl RemoteClient {
    /// Creates a new remote client without authentication.
    pub fn new(http_client: reqwest::Client, base_url: String) -> Self {
        Self {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: None,
        }
    }

    /// Sets the authentication token (JWT) for authenticated requests.
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
    pub async fn create_task(&self, request: CreateTaskRequest) -> AppResult<CreateTaskResponse> {
        self.post("/api/task/create", &request).await
    }

    /// Gets a task by ID.
    pub async fn get_task(&self, request: GetTaskRequest) -> AppResult<GetTaskResponse> {
        self.post("/api/task/get", &request).await
    }

    /// Lists tasks with filters.
    pub async fn list_tasks(&self, request: ListTasksRequest) -> AppResult<ListTasksResponse> {
        self.post("/api/task/list", &request).await
    }

    /// Cancels a task.
    pub async fn cancel_task(&self, request: CancelTaskRequest) -> AppResult<CancelTaskResponse> {
        self.post("/api/task/cancel", &request).await
    }

    /// Deletes a task.
    pub async fn delete_task(&self, request: DeleteTaskRequest) -> AppResult<DeleteTaskResponse> {
        self.post("/api/task/delete", &request).await
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

/// Converts entity AiAgentType to RPC AiAgentType.
/// Since rpc_protocol re-exports from entities, these are the same type.
pub fn entity_to_rpc_agent_type(agent_type: entities::AiAgentType) -> RpcAiAgentType {
    agent_type
}

/// Converts entity UnitTaskStatus to RPC UnitTaskStatus.
/// Since rpc_protocol re-exports from entities, these are the same type.
pub fn entity_to_rpc_unit_status(status: entities::UnitTaskStatus) -> RpcUnitTaskStatus {
    status
}

/// Converts RPC AiAgentType to entity AiAgentType.
/// Since rpc_protocol re-exports from entities, these are the same type.
pub fn rpc_to_entity_agent_type(agent_type: RpcAiAgentType) -> entities::AiAgentType {
    agent_type
}

/// Converts RPC Repository to entity Repository.
///
/// Since rpc_protocol re-exports entities, these are the same type.
pub fn rpc_to_entity_repository(rpc: rpc_protocol::Repository) -> AppResult<entities::Repository> {
    Ok(rpc)
}

/// Converts RPC RepositoryGroup to entity RepositoryGroup.
///
/// Since rpc_protocol re-exports entities, these are the same type.
pub fn rpc_to_entity_repository_group(
    rpc: rpc_protocol::RepositoryGroup,
) -> AppResult<entities::RepositoryGroup> {
    Ok(rpc)
}

/// Converts RPC Workspace to entity Workspace.
///
/// Since rpc_protocol re-exports entities, these are the same type.
pub fn rpc_to_entity_workspace(rpc: rpc_protocol::Workspace) -> AppResult<entities::Workspace> {
    Ok(rpc)
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
}
