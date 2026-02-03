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

/// Sanitizes a string by removing control characters and normalizing
/// whitespace. This prevents potential XSS attacks and ensures clean data
/// storage.
fn sanitize_string(input: &str) -> String {
    input
        .chars()
        // Remove control characters except newlines and tabs
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect::<String>()
        // Normalize multiple spaces to single space
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Validates and sanitizes workspace name.
fn validate_and_sanitize_name(name: &str) -> Result<String, AppError> {
    let sanitized = sanitize_string(name);
    if sanitized.is_empty() {
        return Err(AppError::InvalidRequest(
            "Workspace name cannot be empty".to_string(),
        ));
    }
    if sanitized.len() > MAX_WORKSPACE_NAME_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Workspace name exceeds maximum length of {} characters",
            MAX_WORKSPACE_NAME_LENGTH
        )));
    }
    Ok(sanitized)
}

/// Validates and sanitizes workspace description.
fn validate_and_sanitize_description(description: &str) -> Result<String, AppError> {
    // For descriptions, preserve newlines but remove other control characters
    let sanitized: String = description
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect();
    let sanitized = sanitized.trim().to_string();

    if sanitized.len() > MAX_WORKSPACE_DESCRIPTION_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Workspace description exceeds maximum length of {} characters",
            MAX_WORKSPACE_DESCRIPTION_LENGTH
        )));
    }
    Ok(sanitized)
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

    #[cfg(desktop)]
    if state.mode == AppMode::Local {
        // Validate and sanitize input
        let sanitized_name = validate_and_sanitize_name(&params.name)?;
        let sanitized_description = match params.description {
            Some(ref desc) => Some(validate_and_sanitize_description(desc)?),
            None => None,
        };

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let workspace = Workspace {
            id: Uuid::new_v4(),
            user_id: None,
            name: sanitized_name,
            description: sanitized_description,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created = runtime.task_store_arc().create_workspace(workspace).await?;
        info!("Created workspace: {} ({})", created.name, created.id);
        return Ok(created);
    }

    let _ = &params;

    Err(AppError::InvalidRequest(
        "Remote mode not yet implemented".to_string(),
    ))
}

/// Lists workspaces.
#[tauri::command]
pub async fn list_workspaces(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListWorkspacesParams,
) -> AppResult<ListWorkspacesResult> {
    let state = state.read().await;

    #[cfg(desktop)]
    if state.mode == AppMode::Local {
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

        return Ok(ListWorkspacesResult {
            workspaces,
            total_count: total as i32,
        });
    }

    let _ = &params;

    Err(AppError::InvalidRequest(
        "Remote mode not yet implemented".to_string(),
    ))
}

/// Gets a workspace by ID.
#[tauri::command]
pub async fn get_workspace(
    state: State<'_, Arc<RwLock<AppState>>>,
    workspace_id: String,
) -> AppResult<Workspace> {
    let state = state.read().await;

    #[cfg(desktop)]
    if state.mode == AppMode::Local {
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

        return Ok(workspace);
    }

    let _ = &workspace_id;

    Err(AppError::InvalidRequest(
        "Remote mode not yet implemented".to_string(),
    ))
}

/// Updates a workspace.
#[tauri::command]
pub async fn update_workspace(
    state: State<'_, Arc<RwLock<AppState>>>,
    workspace_id: String,
    params: UpdateWorkspaceParams,
) -> AppResult<Workspace> {
    let state = state.read().await;

    #[cfg(desktop)]
    if state.mode == AppMode::Local {
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
            workspace.name = validate_and_sanitize_name(name)?;
        }
        if let Some(ref description) = params.description {
            workspace.description = Some(validate_and_sanitize_description(description)?);
        }
        workspace.updated_at = chrono::Utc::now();

        let updated = runtime.task_store_arc().update_workspace(workspace).await?;
        info!("Updated workspace: {} ({})", updated.name, updated.id);
        return Ok(updated);
    }

    let _ = (&workspace_id, &params);

    Err(AppError::InvalidRequest(
        "Remote mode not yet implemented".to_string(),
    ))
}

/// Deletes a workspace.
#[tauri::command]
pub async fn delete_workspace(
    state: State<'_, Arc<RwLock<AppState>>>,
    workspace_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    #[cfg(desktop)]
    if state.mode == AppMode::Local {
        let id = Uuid::parse_str(&workspace_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        runtime.task_store_arc().delete_workspace(id).await?;
        info!("Deleted workspace: {}", id);
        return Ok(());
    }

    let _ = &workspace_id;

    Err(AppError::InvalidRequest(
        "Remote mode not yet implemented".to_string(),
    ))
}

/// Gets the default workspace ID.
#[tauri::command]
pub async fn get_default_workspace_id(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<String> {
    let state = state.read().await;

    #[cfg(desktop)]
    if state.mode == AppMode::Local {
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        return Ok(runtime.default_workspace_id().to_string());
    }

    Err(AppError::InvalidRequest(
        "Remote mode not yet implemented".to_string(),
    ))
}
