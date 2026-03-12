//! RepositoryService handlers.

use axum::{Json, extract::State};
use entities::{Repository, RepositoryGroup, VcsProviderType};
use rpc_protocol::{requests::*, responses::*};
use task_store::{RepositoryFilter, RepositoryGroupFilter};

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

/// Extracts repository name from remote URL.
fn extract_repo_name(remote_url: &str) -> String {
    remote_url
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .trim_end_matches(".git")
        .to_string()
}

pub async fn add(
    State(state): State<SharedState>,
    Json(req): Json<AddRepositoryRequest>,
) -> AppResult<Json<AddRepositoryResponse>> {
    let name = req
        .name
        .unwrap_or_else(|| extract_repo_name(&req.remote_url));
    let provider = Repository::detect_provider(&req.remote_url).unwrap_or(VcsProviderType::Github);

    let mut repo = Repository::new(req.workspace_id, name, req.remote_url, provider);
    if let Some(branch) = req.default_branch {
        repo = repo.with_default_branch(branch);
    }

    let repo = state.store.create_repository(repo).await?;
    tracing::info!(repo_id = %repo.id, "Repository added");
    Ok(Json(AddRepositoryResponse { repository: repo }))
}

pub async fn list(
    State(state): State<SharedState>,
    Json(req): Json<ListRepositoriesRequest>,
) -> AppResult<Json<ListRepositoriesResponse>> {
    let filter = RepositoryFilter {
        workspace_id: req.workspace_id,
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
    let (repositories, total_count) = state.store.list_repositories(filter).await?;
    Ok(Json(ListRepositoriesResponse {
        repositories,
        total_count: total_count as i32,
    }))
}

pub async fn get(
    State(state): State<SharedState>,
    Json(req): Json<GetRepositoryRequest>,
) -> AppResult<Json<GetRepositoryResponse>> {
    let repo = state
        .store
        .get_repository(req.repository_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository {} not found", req.repository_id)))?;
    Ok(Json(GetRepositoryResponse { repository: repo }))
}

pub async fn remove(
    State(state): State<SharedState>,
    Json(req): Json<RemoveRepositoryRequest>,
) -> AppResult<Json<RemoveRepositoryResponse>> {
    state.store.delete_repository(req.repository_id).await?;
    tracing::info!(repo_id = %req.repository_id, "Repository removed");
    Ok(Json(RemoveRepositoryResponse {}))
}

pub async fn create_group(
    State(state): State<SharedState>,
    Json(req): Json<CreateRepositoryGroupRequest>,
) -> AppResult<Json<CreateRepositoryGroupResponse>> {
    let mut group = RepositoryGroup::new(req.workspace_id);
    if let Some(name) = req.name {
        group = group.with_name(name);
    }
    group.repository_ids = req.repository_ids;

    let group = state.store.create_repository_group(group).await?;
    tracing::info!(group_id = %group.id, "Repository group created");
    Ok(Json(CreateRepositoryGroupResponse { group }))
}

pub async fn list_groups(
    State(state): State<SharedState>,
    Json(req): Json<ListRepositoryGroupsRequest>,
) -> AppResult<Json<ListRepositoryGroupsResponse>> {
    let filter = RepositoryGroupFilter {
        workspace_id: req.workspace_id,
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
    let (groups, total_count) = state.store.list_repository_groups(filter).await?;
    Ok(Json(ListRepositoryGroupsResponse {
        groups,
        total_count: total_count as i32,
    }))
}

pub async fn update_group(
    State(state): State<SharedState>,
    Json(req): Json<UpdateRepositoryGroupRequest>,
) -> AppResult<Json<UpdateRepositoryGroupResponse>> {
    let mut group = state
        .store
        .get_repository_group(req.group_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Repository group {} not found", req.group_id))
        })?;

    if let Some(name) = req.name {
        group.name = Some(name);
    }
    group.repository_ids = req.repository_ids;
    group.updated_at = chrono::Utc::now();

    let group = state.store.update_repository_group(group).await?;
    tracing::info!(group_id = %group.id, "Repository group updated");
    Ok(Json(UpdateRepositoryGroupResponse { group }))
}

pub async fn delete_group(
    State(state): State<SharedState>,
    Json(req): Json<DeleteRepositoryGroupRequest>,
) -> AppResult<Json<DeleteRepositoryGroupResponse>> {
    state.store.delete_repository_group(req.group_id).await?;
    tracing::info!(group_id = %req.group_id, "Repository group deleted");
    Ok(Json(DeleteRepositoryGroupResponse {}))
}
