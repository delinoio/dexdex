//! SecretsService handlers.

use axum::{Json, extract::State};
use rpc_protocol::{requests::*, responses::*};

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

/// Store secrets for a subtask (sent from the client before execution).
pub async fn send(
    State(state): State<SharedState>,
    Json(req): Json<SendSecretsRequest>,
) -> AppResult<Json<SendSecretsResponse>> {
    let mut cache = state.secrets_cache.write().await;
    cache.store(req.sub_task_id, req.secrets);
    tracing::info!(sub_task_id = %req.sub_task_id, "Secrets stored");
    Ok(Json(SendSecretsResponse {}))
}

/// Clear secrets for a subtask.
pub async fn clear(
    State(state): State<SharedState>,
    Json(req): Json<ClearSecretsRequest>,
) -> AppResult<Json<ClearSecretsResponse>> {
    let mut cache = state.secrets_cache.write().await;
    cache.clear(&req.sub_task_id);
    tracing::info!(sub_task_id = %req.sub_task_id, "Secrets cleared");
    Ok(Json(ClearSecretsResponse {}))
}

/// Get secrets for a subtask (called by the worker during execution).
pub async fn get(
    State(state): State<SharedState>,
    Json(req): Json<GetSecretsRequest>,
) -> AppResult<Json<GetSecretsResponse>> {
    // Verify the worker is registered.
    {
        let registry = state.worker_registry.read().await;
        if registry.get(&req.worker_id).is_none() {
            return Err(AppError::NotFound(format!(
                "Worker {} not found",
                req.worker_id
            )));
        }
    }

    let cache = state.secrets_cache.read().await;
    let secrets = cache.get(&req.sub_task_id).cloned().unwrap_or_default();

    Ok(Json(GetSecretsResponse { secrets }))
}
