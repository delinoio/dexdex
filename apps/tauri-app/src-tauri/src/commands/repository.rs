//! Repository-related Tauri commands.
//!
//! Operations are routed based on the workspace's `kind`:
//! - For local workspaces, operations use the embedded local runtime.
//! - For remote workspaces, operations make RPC calls to the workspace's server
//!   URL.

use std::sync::Arc;

use entities::{Repository, RepositoryGroup, VcsProviderType, WorkspaceKind};
use rpc_protocol::requests;
use serde::{Deserialize, Serialize};
use task_store::{RepositoryFilter, RepositoryGroupFilter, TaskStore};
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    remote_client::{rpc_to_entity_repository, rpc_to_entity_repository_group},
    state::AppState,
};

/// Parameters for adding a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRepositoryParams {
    pub workspace_id: Option<String>,
    pub remote_url: String,
    pub name: Option<String>,
    pub default_branch: Option<String>,
}

/// Parameters for listing repositories.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRepositoriesParams {
    pub workspace_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response for list_repositories command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRepositoriesResult {
    pub repositories: Vec<Repository>,
    pub total_count: i32,
}

/// Resolves the workspace kind and server URL for the given workspace ID.
/// If no workspace_id is provided, uses the default workspace.
#[cfg(desktop)]
async fn resolve_workspace(
    state: &AppState,
    workspace_id: Option<&str>,
) -> AppResult<(Uuid, WorkspaceKind, Option<String>)> {
    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let ws_id = match workspace_id {
        Some(id) => Uuid::parse_str(id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?,
        None => runtime.default_workspace_id(),
    };

    let workspace = runtime
        .task_store()
        .get_workspace(ws_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", ws_id)))?;

    Ok((ws_id, workspace.kind, workspace.server_url))
}

/// Adds a repository.
#[tauri::command]
pub async fn add_repository(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: AddRepositoryParams,
) -> AppResult<Repository> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let (workspace_id, kind, server_url) =
            resolve_workspace(&state, params.workspace_id.as_deref()).await?;

        if kind == WorkspaceKind::Remote {
            let url = server_url.ok_or_else(|| {
                AppError::Config("Remote workspace has no server URL".to_string())
            })?;
            let client = state.get_remote_client_for_url(&url)?;

            let request = requests::AddRepositoryRequest {
                workspace_id: workspace_id.to_string(),
                remote_url: params.remote_url,
                name: params.name,
                default_branch: params.default_branch,
            };

            let response = client.add_repository(request).await?;
            let repository = rpc_to_entity_repository(response.repository)?;
            info!(
                "Added repository via remote: {} ({})",
                repository.name, repository.id
            );
            return Ok(repository);
        }

        // Local workspace
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        // Validate the remote URL
        validate_repository_url(&params.remote_url)?;

        // Parse the remote URL to extract repository info
        let name = params.name.unwrap_or_else(|| {
            extract_repo_name(&params.remote_url).unwrap_or_else(|| "Unknown".to_string())
        });

        let vcs_provider =
            Repository::detect_provider(&params.remote_url).unwrap_or(VcsProviderType::Github);

        let mut repository = Repository::new(workspace_id, &name, &params.remote_url, vcs_provider);
        if let Some(branch) = params.default_branch.clone() {
            repository = repository.with_default_branch(branch);
        }

        let created = runtime
            .task_store_arc()
            .create_repository(repository)
            .await?;
        info!("Added repository: {} ({})", created.name, created.id);
        Ok(created)
    }

    #[cfg(not(desktop))]
    {
        let _ = &params;
        Err(AppError::InvalidRequest(
            "Repository operations not supported on this platform".to_string(),
        ))
    }
}

/// Lists repositories.
#[tauri::command]
pub async fn list_repositories(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListRepositoriesParams,
) -> AppResult<ListRepositoriesResult> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let (workspace_id, kind, server_url) =
            resolve_workspace(&state, params.workspace_id.as_deref()).await?;

        if kind == WorkspaceKind::Remote {
            let url = server_url.ok_or_else(|| {
                AppError::Config("Remote workspace has no server URL".to_string())
            })?;
            let client = state.get_remote_client_for_url(&url)?;

            let request = requests::ListRepositoriesRequest {
                workspace_id: Some(workspace_id.to_string()),
                limit: params.limit.unwrap_or(100),
                offset: params.offset.unwrap_or(0),
            };

            let response = client.list_repositories(request).await?;
            let repositories: crate::error::AppResult<Vec<_>> = response
                .repositories
                .into_iter()
                .map(rpc_to_entity_repository)
                .collect();
            return Ok(ListRepositoriesResult {
                repositories: repositories?,
                total_count: response.total_count,
            });
        }

        // Local workspace
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let limit = params
            .limit
            .map(|l| {
                u32::try_from(l)
                    .map_err(|_| AppError::InvalidRequest("limit must be non-negative".to_string()))
            })
            .transpose()?;
        let offset = params
            .offset
            .map(|o| {
                u32::try_from(o).map_err(|_| {
                    AppError::InvalidRequest("offset must be non-negative".to_string())
                })
            })
            .transpose()?;

        let filter = RepositoryFilter {
            workspace_id: Some(workspace_id),
            limit,
            offset,
        };

        let (repositories, total) = runtime.task_store_arc().list_repositories(filter).await?;

        let total_count = i32::try_from(total).unwrap_or(i32::MAX);
        Ok(ListRepositoriesResult {
            repositories,
            total_count,
        })
    }

    #[cfg(not(desktop))]
    {
        let _ = &params;
        Err(AppError::InvalidRequest(
            "Repository operations not supported on this platform".to_string(),
        ))
    }
}

/// Removes a repository.
#[tauri::command]
pub async fn remove_repository(
    state: State<'_, Arc<RwLock<AppState>>>,
    repository_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let id = Uuid::parse_str(&repository_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid repository ID: {}", e)))?;

        // Try to find the repository to determine workspace
        let repo = runtime.task_store_arc().get_repository(id).await?;

        if let Some(repo) = repo {
            let workspace = runtime
                .task_store()
                .get_workspace(repo.workspace_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            if let Some(ws) = workspace {
                if ws.kind == WorkspaceKind::Remote {
                    let url = ws.server_url.ok_or_else(|| {
                        AppError::Config("Remote workspace has no server URL".to_string())
                    })?;
                    let client = state.get_remote_client_for_url(&url)?;

                    let request = requests::RemoveRepositoryRequest {
                        repository_id: repository_id.clone(),
                    };

                    client.remove_repository(request).await?;
                    info!("Removed repository via remote: {}", repository_id);
                    return Ok(());
                }
            }
        }

        // Local workspace or fallback
        runtime.task_store_arc().delete_repository(id).await?;
        info!("Removed repository: {}", id);
        Ok(())
    }

    #[cfg(not(desktop))]
    {
        let _ = &repository_id;
        Err(AppError::InvalidRequest(
            "Repository operations not supported on this platform".to_string(),
        ))
    }
}

// Helper functions

fn validate_repository_url(url: &str) -> AppResult<()> {
    let url = url.trim();

    if url.is_empty() {
        return Err(AppError::InvalidRequest(
            "Repository URL cannot be empty".to_string(),
        ));
    }

    let is_valid = url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with("git://")
        || url.starts_with("ssh://")
        || (url.starts_with("git@") && url.contains(':'));

    if !is_valid {
        return Err(AppError::InvalidRequest(format!(
            "Invalid repository URL format: '{}'. Expected formats: https://host/path, \
             git@host:path, git://host/path, or ssh://host/path",
            url
        )));
    }

    if url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with("git://")
        || url.starts_with("ssh://")
    {
        let without_scheme = url.split("://").nth(1).unwrap_or("");
        if !without_scheme.contains('/') {
            return Err(AppError::InvalidRequest(format!(
                "Invalid repository URL: missing path component in '{}'",
                url
            )));
        }
        let host = without_scheme.split('/').next().unwrap_or("");
        if host.is_empty() || !host.contains('.') {
            return Err(AppError::InvalidRequest(format!(
                "Invalid repository URL: invalid host in '{}'",
                url
            )));
        }
    }

    if let Some(without_prefix) = url.strip_prefix("git@") {
        let parts: Vec<&str> = without_prefix.splitn(2, ':').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(AppError::InvalidRequest(format!(
                "Invalid repository URL: expected format git@host:path, got '{}'",
                url
            )));
        }
        if !parts[0].contains('.') {
            return Err(AppError::InvalidRequest(format!(
                "Invalid repository URL: invalid host in '{}'",
                url
            )));
        }
    }

    Ok(())
}

fn extract_repo_name(url: &str) -> Option<String> {
    url.rsplit('/')
        .next()
        .or_else(|| url.rsplit(':').next())
        .map(|s| s.trim_end_matches(".git").to_string())
        .filter(|s| !s.is_empty())
}

// =========================================================================
// Repository Group commands
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepositoryGroupParams {
    pub workspace_id: Option<String>,
    pub name: Option<String>,
    pub repository_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRepositoryGroupsParams {
    pub workspace_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRepositoryGroupsResult {
    pub groups: Vec<RepositoryGroup>,
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRepositoryGroupParams {
    pub name: Option<String>,
    pub repository_ids: Vec<String>,
}

#[tauri::command]
pub async fn create_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateRepositoryGroupParams,
) -> AppResult<RepositoryGroup> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let (workspace_id, kind, server_url) =
            resolve_workspace(&state, params.workspace_id.as_deref()).await?;

        if kind == WorkspaceKind::Remote {
            let url = server_url.ok_or_else(|| {
                AppError::Config("Remote workspace has no server URL".to_string())
            })?;
            let client = state.get_remote_client_for_url(&url)?;

            let request = requests::CreateRepositoryGroupRequest {
                workspace_id: workspace_id.to_string(),
                name: params.name,
                repository_ids: params.repository_ids,
            };

            let response = client.create_repository_group(request).await?;
            let group = rpc_to_entity_repository_group(response.group)?;
            info!(
                "Created repository group via remote: {} ({})",
                group.name.as_deref().unwrap_or("unnamed"),
                group.id
            );
            return Ok(group);
        }

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        if params.repository_ids.is_empty() {
            return Err(AppError::InvalidRequest(
                "At least one repository is required".to_string(),
            ));
        }

        let name = params
            .name
            .clone()
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty());

        let repository_ids: Vec<Uuid> = params
            .repository_ids
            .iter()
            .map(|s| Uuid::parse_str(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::InvalidRequest(format!("Invalid repository ID: {}", e)))?;

        for repo_id in &repository_ids {
            runtime
                .task_store_arc()
                .get_repository(*repo_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;
        }

        let mut group = RepositoryGroup::new(workspace_id);
        if let Some(name) = name {
            group = group.with_name(name);
        }
        for repo_id in repository_ids {
            group.add_repository(repo_id);
        }

        let created = runtime
            .task_store_arc()
            .create_repository_group(group)
            .await?;
        info!(
            "Created repository group: {} ({})",
            created.name.as_deref().unwrap_or("unnamed"),
            created.id
        );
        Ok(created)
    }

    #[cfg(not(desktop))]
    {
        let _ = &params;
        Err(AppError::InvalidRequest(
            "Repository group operations not supported on this platform".to_string(),
        ))
    }
}

#[tauri::command]
pub async fn list_repository_groups(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListRepositoryGroupsParams,
) -> AppResult<ListRepositoryGroupsResult> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let (workspace_id, kind, server_url) =
            resolve_workspace(&state, params.workspace_id.as_deref()).await?;

        if kind == WorkspaceKind::Remote {
            let url = server_url.ok_or_else(|| {
                AppError::Config("Remote workspace has no server URL".to_string())
            })?;
            let client = state.get_remote_client_for_url(&url)?;

            let request = requests::ListRepositoryGroupsRequest {
                workspace_id: Some(workspace_id.to_string()),
                limit: params.limit.unwrap_or(100),
                offset: params.offset.unwrap_or(0),
            };

            let response = client.list_repository_groups(request).await?;
            let groups: crate::error::AppResult<Vec<_>> = response
                .groups
                .into_iter()
                .map(rpc_to_entity_repository_group)
                .collect();
            return Ok(ListRepositoryGroupsResult {
                groups: groups?,
                total_count: response.total_count,
            });
        }

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let limit = params
            .limit
            .map(|l| {
                u32::try_from(l)
                    .map_err(|_| AppError::InvalidRequest("limit must be non-negative".to_string()))
            })
            .transpose()?;
        let offset = params
            .offset
            .map(|o| {
                u32::try_from(o).map_err(|_| {
                    AppError::InvalidRequest("offset must be non-negative".to_string())
                })
            })
            .transpose()?;

        let filter = RepositoryGroupFilter {
            workspace_id: Some(workspace_id),
            limit,
            offset,
        };

        let (groups, total) = runtime
            .task_store_arc()
            .list_repository_groups(filter)
            .await?;

        let total_count = i32::try_from(total).unwrap_or(i32::MAX);
        Ok(ListRepositoryGroupsResult {
            groups,
            total_count,
        })
    }

    #[cfg(not(desktop))]
    {
        let _ = &params;
        Err(AppError::InvalidRequest(
            "Repository group operations not supported on this platform".to_string(),
        ))
    }
}

#[tauri::command]
pub async fn get_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    group_id: String,
) -> AppResult<RepositoryGroup> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&group_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid group ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        return runtime
            .task_store_arc()
            .get_repository_group(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Repository group not found: {}", id)));
    }

    #[cfg(not(desktop))]
    {
        let _ = &group_id;
        Err(AppError::InvalidRequest(
            "Repository group operations not supported on this platform".to_string(),
        ))
    }
}

#[tauri::command]
pub async fn update_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    group_id: String,
    params: UpdateRepositoryGroupParams,
) -> AppResult<RepositoryGroup> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&group_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid group ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let group = runtime
            .task_store_arc()
            .get_repository_group(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Repository group not found: {}", id)))?;

        let workspace = runtime
            .task_store()
            .get_workspace(group.workspace_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(ws) = workspace {
            if ws.kind == WorkspaceKind::Remote {
                let url = ws.server_url.ok_or_else(|| {
                    AppError::Config("Remote workspace has no server URL".to_string())
                })?;
                let client = state.get_remote_client_for_url(&url)?;

                let request = requests::UpdateRepositoryGroupRequest {
                    group_id: group_id.clone(),
                    name: params.name,
                    repository_ids: params.repository_ids,
                };

                let response = client.update_repository_group(request).await?;
                let group = rpc_to_entity_repository_group(response.group)?;
                info!(
                    "Updated repository group via remote: {} ({})",
                    group.name.as_deref().unwrap_or("unnamed"),
                    group.id
                );
                return Ok(group);
            }
        }

        if params.repository_ids.is_empty() {
            return Err(AppError::InvalidRequest(
                "At least one repository is required".to_string(),
            ));
        }

        let name = params
            .name
            .clone()
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty());

        let repository_ids: Vec<Uuid> = params
            .repository_ids
            .iter()
            .map(|s| Uuid::parse_str(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::InvalidRequest(format!("Invalid repository ID: {}", e)))?;

        for repo_id in &repository_ids {
            runtime
                .task_store_arc()
                .get_repository(*repo_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;
        }

        let mut group = runtime
            .task_store_arc()
            .get_repository_group(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Repository group not found: {}", id)))?;

        group.name = name;
        group.repository_ids = repository_ids;
        group.updated_at = chrono::Utc::now();

        let updated = runtime
            .task_store_arc()
            .update_repository_group(group)
            .await?;
        info!(
            "Updated repository group: {} ({})",
            updated.name.as_deref().unwrap_or("unnamed"),
            updated.id
        );
        Ok(updated)
    }

    #[cfg(not(desktop))]
    {
        let _ = (&group_id, &params);
        Err(AppError::InvalidRequest(
            "Repository group operations not supported on this platform".to_string(),
        ))
    }
}

#[tauri::command]
pub async fn delete_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    group_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&group_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid group ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let group = runtime
            .task_store_arc()
            .get_repository_group(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Repository group not found: {}", id)))?;

        let workspace = runtime
            .task_store()
            .get_workspace(group.workspace_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(ws) = workspace {
            if ws.kind == WorkspaceKind::Remote {
                let url = ws.server_url.ok_or_else(|| {
                    AppError::Config("Remote workspace has no server URL".to_string())
                })?;
                let client = state.get_remote_client_for_url(&url)?;

                let request = requests::DeleteRepositoryGroupRequest {
                    group_id: group_id.clone(),
                };

                client.delete_repository_group(request).await?;
                info!("Deleted repository group via remote: {}", group_id);
                return Ok(());
            }
        }

        runtime.task_store_arc().delete_repository_group(id).await?;
        info!("Deleted repository group: {}", id);
        Ok(())
    }

    #[cfg(not(desktop))]
    {
        let _ = &group_id;
        Err(AppError::InvalidRequest(
            "Repository group operations not supported on this platform".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_repository_url_valid_https() {
        assert!(validate_repository_url("https://github.com/user/repo.git").is_ok());
        assert!(validate_repository_url("https://github.com/user/repo").is_ok());
        assert!(validate_repository_url("https://gitlab.com/group/subgroup/repo.git").is_ok());
    }

    #[test]
    fn test_validate_repository_url_valid_ssh() {
        assert!(validate_repository_url("git@github.com:user/repo.git").is_ok());
        assert!(validate_repository_url("git@gitlab.com:group/repo.git").is_ok());
        assert!(validate_repository_url("git@bitbucket.org:team/repo.git").is_ok());
    }

    #[test]
    fn test_validate_repository_url_valid_git_protocol() {
        assert!(validate_repository_url("git://github.com/user/repo.git").is_ok());
    }

    #[test]
    fn test_validate_repository_url_valid_ssh_scheme() {
        assert!(validate_repository_url("ssh://git@github.com/user/repo.git").is_ok());
    }

    #[test]
    fn test_validate_repository_url_empty() {
        let result = validate_repository_url("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_repository_url_invalid_format() {
        let result = validate_repository_url("not-a-valid-url");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid repository URL format"));
    }

    #[test]
    fn test_validate_repository_url_missing_path() {
        let result = validate_repository_url("https://github.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing path"));
    }

    #[test]
    fn test_validate_repository_url_invalid_host() {
        let result = validate_repository_url("https://localhost/repo.git");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid host"));
    }

    #[test]
    fn test_validate_repository_url_git_ssh_invalid() {
        let result = validate_repository_url("git@:user/repo.git");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_repo_name_https() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git"),
            Some("repo".to_string())
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/repo"),
            Some("repo".to_string())
        );
    }

    #[test]
    fn test_extract_repo_name_ssh() {
        assert_eq!(
            extract_repo_name("git@github.com:user/repo.git"),
            Some("repo".to_string())
        );
    }

    #[test]
    fn test_extract_repo_name_edge_cases() {
        assert_eq!(extract_repo_name(""), None);
        assert_eq!(extract_repo_name("/"), None);
    }
}
