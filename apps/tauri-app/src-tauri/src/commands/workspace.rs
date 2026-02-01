//! Workspace-related Tauri commands.

use std::sync::Arc;

use entities::Workspace;
use serde::{Deserialize, Serialize};
use task_store::{TaskStore, WorkspaceFilter};
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::AppMode,
    error::{AppError, AppResult},
    state::AppState,
};

/// Maximum length for workspace name.
const MAX_WORKSPACE_NAME_LENGTH: usize = 255;
/// Maximum length for workspace description.
const MAX_WORKSPACE_DESCRIPTION_LENGTH: usize = 10000;

/// Validates workspace name.
fn validate_workspace_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        return Err(AppError::InvalidRequest(
            "Workspace name cannot be empty".to_string(),
        ));
    }
    if name.len() > MAX_WORKSPACE_NAME_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Workspace name exceeds maximum length of {} characters",
            MAX_WORKSPACE_NAME_LENGTH
        )));
    }
    Ok(())
}

/// Validates workspace description.
fn validate_workspace_description(description: &str) -> Result<(), AppError> {
    if description.len() > MAX_WORKSPACE_DESCRIPTION_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Workspace description exceeds maximum length of {} characters",
            MAX_WORKSPACE_DESCRIPTION_LENGTH
        )));
    }
    Ok(())
}

/// Parameters for creating a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceParams {
    pub name: String,
    pub description: Option<String>,
}

/// Parameters for updating a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceParams {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Parameters for listing workspaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspacesParams {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response for list_workspaces command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWorkspacesResult {
    pub workspaces: Vec<Workspace>,
    pub total_count: i32,
}

/// Creates a new workspace.
#[tauri::command]
pub async fn create_workspace(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateWorkspaceParams,
) -> AppResult<Workspace> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    // Validate input
    validate_workspace_name(&params.name)?;
    if let Some(ref description) = params.description {
        validate_workspace_description(description)?;
    }

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let workspace = Workspace {
        id: Uuid::new_v4(),
        user_id: None,
        name: params.name.clone(),
        description: params.description,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = runtime.task_store_arc().create_workspace(workspace).await?;
    info!("Created workspace: {} ({})", created.name, created.id);
    Ok(created)
}

/// Lists workspaces.
#[tauri::command]
pub async fn list_workspaces(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListWorkspacesParams,
) -> AppResult<ListWorkspacesResult> {
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

    let filter = WorkspaceFilter {
        user_id: None,
        limit: params.limit.map(|l| l as u32),
        offset: params.offset.map(|o| o as u32),
    };

    let (workspaces, total) = runtime.task_store_arc().list_workspaces(filter).await?;

    Ok(ListWorkspacesResult {
        workspaces,
        total_count: total as i32,
    })
}

/// Gets a workspace by ID.
#[tauri::command]
pub async fn get_workspace(
    state: State<'_, Arc<RwLock<AppState>>>,
    workspace_id: String,
) -> AppResult<Workspace> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&workspace_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let workspace = runtime
        .task_store_arc()
        .get_workspace(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", id)))?;

    Ok(workspace)
}

/// Updates a workspace.
#[tauri::command]
pub async fn update_workspace(
    state: State<'_, Arc<RwLock<AppState>>>,
    workspace_id: String,
    params: UpdateWorkspaceParams,
) -> AppResult<Workspace> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&workspace_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let mut workspace = runtime
        .task_store_arc()
        .get_workspace(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", id)))?;

    if let Some(ref name) = params.name {
        validate_workspace_name(name)?;
        workspace.name = name.clone();
    }
    if let Some(ref description) = params.description {
        validate_workspace_description(description)?;
        workspace.description = Some(description.clone());
    }
    workspace.updated_at = chrono::Utc::now();

    let updated = runtime.task_store_arc().update_workspace(workspace).await?;
    info!("Updated workspace: {} ({})", updated.name, updated.id);
    Ok(updated)
}

/// Deletes a workspace.
#[tauri::command]
pub async fn delete_workspace(
    state: State<'_, Arc<RwLock<AppState>>>,
    workspace_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&workspace_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    runtime.task_store_arc().delete_workspace(id).await?;
    info!("Deleted workspace: {}", id);
    Ok(())
}

/// Gets the default workspace ID.
#[tauri::command]
pub async fn get_default_workspace_id(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<String> {
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

    Ok(runtime.default_workspace_id().to_string())
}
