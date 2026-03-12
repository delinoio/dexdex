//! SessionService handlers.

use axum::{Json, extract::State};
use rpc_protocol::{requests::*, responses::*};

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

pub async fn list(
    State(state): State<SharedState>,
    Json(req): Json<ListSessionsRequest>,
) -> AppResult<Json<ListSessionsResponse>> {
    let sessions = state.store.list_agent_sessions(req.sub_task_id).await?;
    Ok(Json(ListSessionsResponse { sessions }))
}

pub async fn get(
    State(state): State<SharedState>,
    Json(req): Json<GetSessionRequest>,
) -> AppResult<Json<GetSessionResponse>> {
    let session = state
        .store
        .get_agent_session(req.session_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Session {} not found", req.session_id)))?;
    Ok(Json(GetSessionResponse { session }))
}

pub async fn get_output(
    State(state): State<SharedState>,
    Json(req): Json<GetSessionOutputRequest>,
) -> AppResult<Json<GetSessionOutputResponse>> {
    let all_events = state
        .store
        .list_session_outputs(req.session_id, req.after_sequence)
        .await?;

    let limit = req.limit.unwrap_or(u32::MAX) as usize;
    let has_more = all_events.len() > limit;
    let events = all_events.into_iter().take(limit).collect();

    Ok(Json(GetSessionOutputResponse { events, has_more }))
}

pub async fn stop(
    State(state): State<SharedState>,
    Json(req): Json<StopSessionRequest>,
) -> AppResult<Json<StopSessionResponse>> {
    let mut session = state
        .store
        .get_agent_session(req.session_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Session {} not found", req.session_id)))?;

    session.status = entities::AgentSessionStatus::Cancelled;
    session.completed_at = Some(chrono::Utc::now());
    state.store.update_agent_session(session).await?;
    tracing::info!(session_id = %req.session_id, "Session stopped");

    Ok(Json(StopSessionResponse {}))
}
