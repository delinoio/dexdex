//! PrManagementService handlers.

use axum::{Json, extract::State};
use entities::PullRequestTracking;
use rpc_protocol::{requests::*, responses::*};

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

pub async fn create_tracking(
    State(state): State<SharedState>,
    Json(req): Json<CreatePrTrackingRequest>,
) -> AppResult<Json<CreatePrTrackingResponse>> {
    let pr = PullRequestTracking::new(
        req.unit_task_id,
        req.provider,
        req.repository_id,
        req.pr_number,
        req.pr_url,
    );
    let pr_tracking = state.store.create_pr_tracking(pr).await?;
    tracing::info!(pr_id = %pr_tracking.id, "PR tracking created");
    Ok(Json(CreatePrTrackingResponse { pr_tracking }))
}

pub async fn get_tracking(
    State(state): State<SharedState>,
    Json(req): Json<GetPrTrackingRequest>,
) -> AppResult<Json<GetPrTrackingResponse>> {
    let pr_tracking = state
        .store
        .get_pr_tracking(req.pr_tracking_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("PR tracking {} not found", req.pr_tracking_id))
        })?;
    Ok(Json(GetPrTrackingResponse { pr_tracking }))
}

pub async fn list_trackings(
    State(state): State<SharedState>,
    Json(req): Json<ListPrTrackingsRequest>,
) -> AppResult<Json<ListPrTrackingsResponse>> {
    let pr_trackings = state
        .store
        .list_pr_trackings(Some(req.unit_task_id))
        .await?;
    Ok(Json(ListPrTrackingsResponse { pr_trackings }))
}

pub async fn trigger_auto_fix(
    State(state): State<SharedState>,
    Json(req): Json<TriggerAutoFixRequest>,
) -> AppResult<Json<TriggerAutoFixResponse>> {
    let mut pr = state
        .store
        .get_pr_tracking(req.pr_tracking_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("PR tracking {} not found", req.pr_tracking_id))
        })?;

    if pr.auto_fix_attempts_used >= pr.max_auto_fix_attempts {
        return Err(AppError::InvalidRequest(
            "Maximum auto-fix attempts reached".to_string(),
        ));
    }

    pr.auto_fix_enabled = true;
    pr.auto_fix_attempts_used += 1;
    pr.updated_at = chrono::Utc::now();
    state.store.update_pr_tracking(pr).await?;
    tracing::info!(pr_id = %req.pr_tracking_id, "Auto-fix triggered");

    Ok(Json(TriggerAutoFixResponse {}))
}
