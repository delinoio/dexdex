//! TaskService handlers.

use axum::{Json, extract::State};
use entities::{SubTask, SubTaskType, UnitTask, UnitTaskStatus};
use rpc_protocol::{StreamEvent, StreamEventType, requests::*, responses::*};
use task_store::TaskFilter;

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

pub async fn create(
    State(state): State<SharedState>,
    Json(req): Json<CreateTaskRequest>,
) -> AppResult<Json<CreateTaskResponse>> {
    // Create the unit task.
    let mut task = UnitTask::new(
        req.workspace_id,
        req.repository_group_id,
        req.title,
        req.prompt.clone(),
    );
    if let Some(branch) = req.branch_name {
        task = task.with_branch_name(branch);
    }

    let task = state.store.create_unit_task(task).await?;
    tracing::info!(task_id = %task.id, "Unit task created");

    // Create the initial subtask.
    let mut subtask = SubTask::new(task.id, SubTaskType::InitialImplementation, req.prompt);
    if req.plan_mode_enabled {
        subtask = subtask.with_plan_mode();
    }
    let subtask = state.store.create_sub_task(subtask).await?;
    tracing::info!(subtask_id = %subtask.id, "Initial subtask created");

    // Publish TASK_UPDATED event.
    state.broker.publish(StreamEvent {
        event_type: StreamEventType::TaskUpdated,
        workspace_id: task.workspace_id.to_string(),
        payload: serde_json::to_value(&task).unwrap_or_default(),
    });

    Ok(Json(CreateTaskResponse { task }))
}

pub async fn list(
    State(state): State<SharedState>,
    Json(req): Json<ListTasksRequest>,
) -> AppResult<Json<ListTasksResponse>> {
    let filter = TaskFilter {
        workspace_id: req.workspace_id,
        repository_group_id: req.repository_group_id,
        status: req.status,
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
    let (tasks, total_count) = state.store.list_unit_tasks(filter).await?;
    Ok(Json(ListTasksResponse {
        tasks,
        total_count: total_count as i32,
    }))
}

pub async fn get(
    State(state): State<SharedState>,
    Json(req): Json<GetTaskRequest>,
) -> AppResult<Json<GetTaskResponse>> {
    let task = state
        .store
        .get_unit_task(req.task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", req.task_id)))?;
    Ok(Json(GetTaskResponse { task }))
}

pub async fn cancel(
    State(state): State<SharedState>,
    Json(req): Json<CancelTaskRequest>,
) -> AppResult<Json<CancelTaskResponse>> {
    let mut task = state
        .store
        .get_unit_task(req.task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", req.task_id)))?;

    task.status = UnitTaskStatus::Cancelled;
    task.updated_at = chrono::Utc::now();
    let task = state.store.update_unit_task(task).await?;
    tracing::info!(task_id = %task.id, "Task cancelled");

    // Cancel all in-progress subtasks.
    let subtasks = state.store.list_sub_tasks(task.id).await?;
    for mut subtask in subtasks {
        if matches!(
            subtask.status,
            entities::SubTaskStatus::Queued
                | entities::SubTaskStatus::InProgress
                | entities::SubTaskStatus::WaitingForPlanApproval
                | entities::SubTaskStatus::WaitingForUserInput
        ) {
            subtask.status = entities::SubTaskStatus::Cancelled;
            subtask.updated_at = chrono::Utc::now();
            let _ = state.store.update_sub_task(subtask).await;
        }
    }

    state.broker.publish(StreamEvent {
        event_type: StreamEventType::TaskUpdated,
        workspace_id: task.workspace_id.to_string(),
        payload: serde_json::to_value(&task).unwrap_or_default(),
    });

    Ok(Json(CancelTaskResponse {}))
}

pub async fn delete(
    State(state): State<SharedState>,
    Json(req): Json<DeleteTaskRequest>,
) -> AppResult<Json<DeleteTaskResponse>> {
    state.store.delete_unit_task_cascade(req.task_id).await?;
    tracing::info!(task_id = %req.task_id, "Task deleted");
    Ok(Json(DeleteTaskResponse {}))
}
