//! WorkspaceService handlers.

use axum::{Json, extract::State};
use entities::Workspace;
use rpc_protocol::{requests::*, responses::*};
use task_store::WorkspaceFilter;

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

pub async fn create(
    State(state): State<SharedState>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> AppResult<Json<CreateWorkspaceResponse>> {
    let mut workspace = Workspace::new(req.name);
    if let Some(desc) = req.description {
        workspace = workspace.with_description(desc);
    }
    if let Some(url) = req.endpoint_url {
        workspace = workspace.with_endpoint_url(url);
    }
    let workspace = state.store.create_workspace(workspace).await?;
    tracing::info!(workspace_id = %workspace.id, "Workspace created");
    Ok(Json(CreateWorkspaceResponse { workspace }))
}

pub async fn list(
    State(state): State<SharedState>,
    Json(req): Json<ListWorkspacesRequest>,
) -> AppResult<Json<ListWorkspacesResponse>> {
    let filter = WorkspaceFilter {
        limit: if req.limit > 0 {
            Some(req.limit as u32)
        } else {
            None
        },
        offset: if req.offset > 0 {
            Some(req.offset as u32)
        } else {
            None
        },
    };
    let (workspaces, total_count) = state.store.list_workspaces(filter).await?;
    Ok(Json(ListWorkspacesResponse {
        workspaces,
        total_count: total_count as i32,
    }))
}

pub async fn get(
    State(state): State<SharedState>,
    Json(req): Json<GetWorkspaceRequest>,
) -> AppResult<Json<GetWorkspaceResponse>> {
    let workspace = state
        .store
        .get_workspace(req.workspace_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", req.workspace_id)))?;
    Ok(Json(GetWorkspaceResponse { workspace }))
}

pub async fn update(
    State(state): State<SharedState>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> AppResult<Json<UpdateWorkspaceResponse>> {
    let mut workspace = state
        .store
        .get_workspace(req.workspace_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", req.workspace_id)))?;

    if let Some(name) = req.name {
        workspace.name = name;
    }
    if let Some(desc) = req.description {
        workspace.description = Some(desc);
    }
    if let Some(url) = req.endpoint_url {
        workspace.endpoint_url = Some(url);
    }
    if let Some(auth_id) = req.auth_profile_id {
        workspace.auth_profile_id = Some(auth_id);
    }
    workspace.updated_at = chrono::Utc::now();

    let workspace = state.store.update_workspace(workspace).await?;
    tracing::info!(workspace_id = %workspace.id, "Workspace updated");
    Ok(Json(UpdateWorkspaceResponse { workspace }))
}

pub async fn delete(
    State(state): State<SharedState>,
    Json(req): Json<DeleteWorkspaceRequest>,
) -> AppResult<Json<DeleteWorkspaceResponse>> {
    state.store.delete_workspace(req.workspace_id).await?;
    tracing::info!(workspace_id = %req.workspace_id, "Workspace deleted");
    Ok(Json(DeleteWorkspaceResponse {}))
}
