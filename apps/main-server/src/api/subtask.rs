//! SubTaskService handlers.

use axum::{Json, extract::State};
use entities::{SubTask, SubTaskStatus, SubTaskType};
use rpc_protocol::{StreamEvent, StreamEventType, requests::*, responses::*};

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

pub async fn list(
    State(state): State<SharedState>,
    Json(req): Json<ListSubTasksRequest>,
) -> AppResult<Json<ListSubTasksResponse>> {
    let sub_tasks = state.store.list_sub_tasks(req.unit_task_id).await?;
    Ok(Json(ListSubTasksResponse { sub_tasks }))
}

pub async fn get(
    State(state): State<SharedState>,
    Json(req): Json<GetSubTaskRequest>,
) -> AppResult<Json<GetSubTaskResponse>> {
    let sub_task = state
        .store
        .get_sub_task(req.sub_task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("SubTask {} not found", req.sub_task_id)))?;
    Ok(Json(GetSubTaskResponse { sub_task }))
}

/// Approve a completed subtask result (after human review).
pub async fn approve(
    State(state): State<SharedState>,
    Json(req): Json<ApproveSubTaskRequest>,
) -> AppResult<Json<ApproveSubTaskResponse>> {
    let mut subtask = state
        .store
        .get_sub_task(req.sub_task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("SubTask {} not found", req.sub_task_id)))?;

    subtask.status = SubTaskStatus::Completed;
    subtask.updated_at = chrono::Utc::now();
    let subtask = state.store.update_sub_task(subtask).await?;
    tracing::info!(subtask_id = %subtask.id, "SubTask approved");

    publish_subtask_event(&state, &subtask);
    Ok(Json(ApproveSubTaskResponse {}))
}

/// Approve the plan generated for a plan-mode subtask.
pub async fn approve_plan(
    State(state): State<SharedState>,
    Json(req): Json<ApprovePlanRequest>,
) -> AppResult<Json<ApprovePlanResponse>> {
    let mut subtask = state
        .store
        .get_sub_task(req.sub_task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("SubTask {} not found", req.sub_task_id)))?;

    if subtask.status != SubTaskStatus::WaitingForPlanApproval {
        return Err(AppError::InvalidRequest(format!(
            "SubTask {} is not waiting for plan approval (status: {:?})",
            req.sub_task_id, subtask.status
        )));
    }

    // Move to in-progress so the worker can continue executing the approved plan.
    subtask.status = SubTaskStatus::InProgress;
    subtask.updated_at = chrono::Utc::now();
    let subtask = state.store.update_sub_task(subtask).await?;
    tracing::info!(subtask_id = %subtask.id, "Plan approved, subtask resuming");

    publish_subtask_event(&state, &subtask);
    Ok(Json(ApprovePlanResponse {}))
}

/// Request revisions to the generated plan.
pub async fn revise_plan(
    State(state): State<SharedState>,
    Json(req): Json<RevisePlanRequest>,
) -> AppResult<Json<RevisePlanResponse>> {
    let mut original = state
        .store
        .get_sub_task(req.sub_task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("SubTask {} not found", req.sub_task_id)))?;

    if original.status != SubTaskStatus::WaitingForPlanApproval {
        return Err(AppError::InvalidRequest(format!(
            "SubTask {} is not waiting for plan approval (status: {:?})",
            req.sub_task_id, original.status
        )));
    }

    // Cancel the original subtask and create a new one with revised prompt.
    original.status = SubTaskStatus::Cancelled;
    original.updated_at = chrono::Utc::now();
    let _ = state.store.update_sub_task(original.clone()).await?;

    let revised_prompt = format!(
        "{}\n\n--- User feedback ---\n{}",
        original.prompt, req.feedback
    );
    let new_subtask = SubTask::new(
        original.unit_task_id,
        SubTaskType::InitialImplementation,
        revised_prompt,
    )
    .with_plan_mode();
    let new_subtask = state.store.create_sub_task(new_subtask).await?;
    tracing::info!(
        original_id = %original.id,
        new_id = %new_subtask.id,
        "Plan revised, new subtask created"
    );

    publish_subtask_event(&state, &new_subtask);
    Ok(Json(RevisePlanResponse {
        sub_task: new_subtask,
    }))
}

/// Manually retry a failed subtask.
pub async fn retry(
    State(state): State<SharedState>,
    Json(req): Json<RetrySubTaskRequest>,
) -> AppResult<Json<RetrySubTaskResponse>> {
    let original = state
        .store
        .get_sub_task(req.sub_task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("SubTask {} not found", req.sub_task_id)))?;

    if !matches!(
        original.status,
        SubTaskStatus::Failed | SubTaskStatus::Cancelled
    ) {
        return Err(AppError::InvalidRequest(format!(
            "SubTask {} cannot be retried (status: {:?})",
            req.sub_task_id, original.status
        )));
    }

    let mut new_subtask = SubTask::new(
        original.unit_task_id,
        SubTaskType::ManualRetry,
        original.prompt.clone(),
    );
    if original.plan_mode_enabled {
        new_subtask = new_subtask.with_plan_mode();
    }
    let new_subtask = state.store.create_sub_task(new_subtask).await?;
    tracing::info!(
        original_id = %original.id,
        new_id = %new_subtask.id,
        "SubTask retry created"
    );

    publish_subtask_event(&state, &new_subtask);
    Ok(Json(RetrySubTaskResponse {
        sub_task: new_subtask,
    }))
}

fn publish_subtask_event(state: &SharedState, subtask: &SubTask) {
    // Look up the workspace_id from the unit task — best effort.
    // We use an empty string if unavailable to avoid blocking.
    state.broker.publish(StreamEvent {
        event_type: StreamEventType::SubtaskUpdated,
        workspace_id: String::new(),
        payload: serde_json::to_value(subtask).unwrap_or_default(),
    });
}
