//! Worker server API endpoints.
//!
//! These endpoints are called by the main server for task management
//! and TTY input responses.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{WorkerError, WorkerResult},
    state::AppState,
};

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Worker status.
    pub status: String,
    /// Worker ID.
    pub worker_id: Option<Uuid>,
    /// Current task ID.
    pub current_task_id: Option<Uuid>,
}

/// Submit TTY input request.
#[derive(Debug, Deserialize)]
pub struct SubmitTtyInputRequest {
    /// Request ID.
    pub request_id: Uuid,
    /// User's response.
    pub response: String,
}

/// Submit TTY input response.
#[derive(Debug, Serialize)]
pub struct SubmitTtyInputResponse {
    /// Whether the response was accepted.
    pub accepted: bool,
}

/// Cancel task request.
#[derive(Debug, Deserialize)]
pub struct CancelTaskRequest {
    /// Task ID to cancel.
    pub task_id: Uuid,
}

/// Cancel task response.
#[derive(Debug, Serialize)]
pub struct CancelTaskResponse {
    /// Whether the task was cancelled.
    pub cancelled: bool,
}

/// Health check endpoint.
async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let status = state.get_status().await;
    let worker_id = state.get_worker_id().await;
    let current_task_id = state.get_current_task_id().await;

    Json(HealthResponse {
        status: status.as_str().to_string(),
        worker_id,
        current_task_id,
    })
}

/// Submit TTY input response endpoint.
async fn submit_tty_input(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SubmitTtyInputRequest>,
) -> WorkerResult<Json<SubmitTtyInputResponse>> {
    let accepted = state
        .submit_tty_response(request.request_id, request.response)
        .await;

    Ok(Json(SubmitTtyInputResponse { accepted }))
}

/// Cancel task endpoint.
async fn cancel_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CancelTaskRequest>,
) -> WorkerResult<Json<CancelTaskResponse>> {
    // Verify the task ID matches the current task
    let current_task_id = state.get_current_task_id().await;

    if current_task_id != Some(request.task_id) {
        return Err(WorkerError::TaskNotFound(request.task_id.to_string()));
    }

    let cancelled = state.cancel_current_task().await;

    Ok(Json(CancelTaskResponse { cancelled }))
}

/// Get current task output.
async fn get_task_output(State(state): State<Arc<AppState>>) -> WorkerResult<String> {
    state
        .get_task_output()
        .await
        .ok_or_else(|| WorkerError::TaskNotFound("No current task".to_string()))
}

/// Creates the API router.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/tty-input", post(submit_tty_input))
        .route("/cancel", post(cancel_task))
        .route("/output", get(get_task_output))
}
