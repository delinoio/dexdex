//! Repository-related Tauri commands.

use std::sync::Arc;

use entities::{Repository, RepositoryGroup, VcsProviderType};
use serde::{Deserialize, Serialize};
use task_store::{RepositoryFilter, RepositoryGroupFilter, TaskStore};
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::AppMode,
    error::{AppError, AppResult},
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

/// Adds a repository.
#[tauri::command]
pub async fn add_repository(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: AddRepositoryParams,
) -> AppResult<Repository> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let workspace_id = params
        .workspace_id
        .as_ref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?
        .unwrap_or_else(|| runtime.default_workspace_id());

    // Validate the remote URL
    validate_repository_url(&params.remote_url)?;

    // Parse the remote URL to extract repository info
    let name = params.name.unwrap_or_else(|| {
        extract_repo_name(&params.remote_url).unwrap_or_else(|| "Unknown".to_string())
    });

    let vcs_provider =
        Repository::detect_provider(&params.remote_url).unwrap_or(VcsProviderType::Github);

    let mut repository = Repository::new(workspace_id, &name, &params.remote_url, vcs_provider);
    if let Some(branch) = params.default_branch {
        repository = repository.with_default_branch(branch);
    }

    let created = runtime
        .task_store_arc()
        .create_repository(repository)
        .await?;
    info!("Added repository: {} ({})", created.name, created.id);
    Ok(created)
}

/// Lists repositories.
#[tauri::command]
pub async fn list_repositories(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListRepositoriesParams,
) -> AppResult<ListRepositoriesResult> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
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
            u32::try_from(o)
                .map_err(|_| AppError::InvalidRequest("offset must be non-negative".to_string()))
        })
        .transpose()?;

    let filter = RepositoryFilter {
        workspace_id: params
            .workspace_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok()),
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

/// Removes a repository.
#[tauri::command]
pub async fn remove_repository(
    state: State<'_, Arc<RwLock<AppState>>>,
    repository_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&repository_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid repository ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    runtime.task_store_arc().delete_repository(id).await?;
    info!("Removed repository: {}", id);
    Ok(())
}

// Helper functions

/// Validates that a repository URL is a valid git remote URL.
///
/// Accepts the following formats:
/// - HTTPS: `https://github.com/user/repo.git`
/// - SSH: `git@github.com:user/repo.git`
/// - Git protocol: `git://github.com/user/repo.git`
/// - SSH with scheme: `ssh://git@github.com/user/repo.git`
fn validate_repository_url(url: &str) -> AppResult<()> {
    let url = url.trim();

    if url.is_empty() {
        return Err(AppError::InvalidRequest(
            "Repository URL cannot be empty".to_string(),
        ));
    }

    // Check for common git URL patterns
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

    // For URL-style addresses, do basic validation
    if url.starts_with("https://")
        || url.starts_with("http://")
        || url.starts_with("git://")
        || url.starts_with("ssh://")
    {
        // Check for host and path components
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

    // For git@host:path style
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
        .or_else(|| {
            // Handle git@host:path format
            url.rsplit(':').next()
        })
        .map(|s| s.trim_end_matches(".git").to_string())
        .filter(|s| !s.is_empty())
}

// =========================================================================
// Repository Group commands
// =========================================================================

/// Parameters for creating a repository group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepositoryGroupParams {
    pub workspace_id: Option<String>,
    pub name: Option<String>,
    pub repository_ids: Vec<String>,
}

/// Parameters for listing repository groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRepositoryGroupsParams {
    pub workspace_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response for list_repository_groups command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRepositoryGroupsResult {
    pub groups: Vec<RepositoryGroup>,
    pub total_count: i32,
}

/// Parameters for updating a repository group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRepositoryGroupParams {
    pub name: Option<String>,
    pub repository_ids: Vec<String>,
}

/// Creates a repository group.
#[tauri::command]
pub async fn create_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateRepositoryGroupParams,
) -> AppResult<RepositoryGroup> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let workspace_id = params
        .workspace_id
        .as_ref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?
        .unwrap_or_else(|| runtime.default_workspace_id());

    // Validate that at least one repository is provided
    if params.repository_ids.is_empty() {
        return Err(AppError::InvalidRequest(
            "At least one repository is required".to_string(),
        ));
    }

    // Validate and sanitize name if provided
    let name = params
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());

    // Parse repository IDs
    let repository_ids: Vec<Uuid> = params
        .repository_ids
        .iter()
        .map(|s| Uuid::parse_str(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::InvalidRequest(format!("Invalid repository ID: {}", e)))?;

    // Verify all repositories exist
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

/// Lists repository groups.
#[tauri::command]
pub async fn list_repository_groups(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListRepositoryGroupsParams,
) -> AppResult<ListRepositoryGroupsResult> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
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
            u32::try_from(o)
                .map_err(|_| AppError::InvalidRequest("offset must be non-negative".to_string()))
        })
        .transpose()?;

    let filter = RepositoryGroupFilter {
        workspace_id: params
            .workspace_id
            .as_ref()
            .map(|s| Uuid::parse_str(s))
            .transpose()
            .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?,
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

/// Gets a repository group by ID.
#[tauri::command]
pub async fn get_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    group_id: String,
) -> AppResult<RepositoryGroup> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&group_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid group ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    runtime
        .task_store_arc()
        .get_repository_group(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository group not found: {}", id)))
}

/// Updates a repository group.
#[tauri::command]
pub async fn update_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    group_id: String,
    params: UpdateRepositoryGroupParams,
) -> AppResult<RepositoryGroup> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&group_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid group ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Validate that at least one repository is provided
    if params.repository_ids.is_empty() {
        return Err(AppError::InvalidRequest(
            "At least one repository is required".to_string(),
        ));
    }

    // Validate and sanitize name if provided
    let name = params
        .name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());

    // Parse repository IDs
    let repository_ids: Vec<Uuid> = params
        .repository_ids
        .iter()
        .map(|s| Uuid::parse_str(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::InvalidRequest(format!("Invalid repository ID: {}", e)))?;

    // Verify all repositories exist
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

/// Deletes a repository group.
#[tauri::command]
pub async fn delete_repository_group(
    state: State<'_, Arc<RwLock<AppState>>>,
    group_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&group_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid group ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    runtime.task_store_arc().delete_repository_group(id).await?;
    info!("Deleted repository group: {}", id);
    Ok(())
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
