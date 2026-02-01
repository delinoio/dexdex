//! Repository-related Tauri commands.

use std::sync::Arc;

use entities::{Repository, VcsProviderType};
use serde::{Deserialize, Serialize};
use task_store::{RepositoryFilter, TaskStore};
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

    let filter = RepositoryFilter {
        workspace_id: params
            .workspace_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok()),
        limit: params.limit.map(|l| l as u32),
        offset: params.offset.map(|o| o as u32),
    };

    let (repositories, total) = runtime.task_store_arc().list_repositories(filter).await?;

    Ok(ListRepositoriesResult {
        repositories,
        total_count: total as i32,
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

fn extract_repo_name(url: &str) -> Option<String> {
    url.rsplit('/')
        .next()
        .map(|s| s.trim_end_matches(".git").to_string())
        .filter(|s| !s.is_empty())
}
