//! WorkerService handlers (internal — used by worker-server).

use axum::{Json, extract::State};
use entities::{SubTaskStatus, UnitTaskStatus};
use rpc_protocol::{StreamEvent, StreamEventType, requests::*, responses::*};

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

pub async fn register(
    State(state): State<SharedState>,
    Json(req): Json<RegisterWorkerRequest>,
) -> AppResult<Json<RegisterWorkerResponse>> {
    let worker_id = state
        .worker_registry
        .write()
        .await
        .register(req.name.clone(), req.endpoint_url.clone());

    tracing::info!(
        worker_id = %worker_id,
        name = %req.name,
        endpoint = %req.endpoint_url,
        "Worker registered"
    );

    Ok(Json(RegisterWorkerResponse { worker_id }))
}

pub async fn heartbeat(
    State(state): State<SharedState>,
    Json(req): Json<HeartbeatRequest>,
) -> AppResult<Json<HeartbeatResponse>> {
    let found = state
        .worker_registry
        .write()
        .await
        .heartbeat(req.worker_id, req.current_sub_task_id);

    if !found {
        return Err(AppError::NotFound(format!(
            "Worker {} not found",
            req.worker_id
        )));
    }

    Ok(Json(HeartbeatResponse {}))
}

pub async fn unregister(
    State(state): State<SharedState>,
    Json(req): Json<UnregisterWorkerRequest>,
) -> AppResult<Json<UnregisterWorkerResponse>> {
    state
        .worker_registry
        .write()
        .await
        .unregister(req.worker_id);
    tracing::info!(worker_id = %req.worker_id, "Worker unregistered");
    Ok(Json(UnregisterWorkerResponse {}))
}

pub async fn get_next_sub_task(
    State(state): State<SharedState>,
    Json(_req): Json<GetNextSubTaskRequest>,
) -> AppResult<Json<GetNextSubTaskResponse>> {
    let maybe_subtask = state.store.get_next_queued_sub_task().await?;

    if let Some(ref subtask) = maybe_subtask {
        // Mark it as in-progress immediately.
        let mut subtask_mut = subtask.clone();
        subtask_mut.status = SubTaskStatus::InProgress;
        subtask_mut.updated_at = chrono::Utc::now();
        let subtask_mut = state.store.update_sub_task(subtask_mut).await?;

        // Fetch the parent unit task.
        let unit_task = state
            .store
            .get_unit_task(subtask_mut.unit_task_id)
            .await?
            .map(|mut t| {
                t.status = UnitTaskStatus::InProgress;
                t.updated_at = chrono::Utc::now();
                t
            });

        // Update unit task status if found.
        if let Some(ref task) = unit_task {
            let updated_task = state.store.update_unit_task(task.clone()).await?;
            state.broker.publish(StreamEvent {
                event_type: StreamEventType::TaskUpdated,
                workspace_id: updated_task.workspace_id.to_string(),
                payload: serde_json::to_value(&updated_task).unwrap_or_default(),
            });
        }

        state.broker.publish(StreamEvent {
            event_type: StreamEventType::SubtaskUpdated,
            workspace_id: unit_task
                .as_ref()
                .map(|t| t.workspace_id.to_string())
                .unwrap_or_default(),
            payload: serde_json::to_value(&subtask_mut).unwrap_or_default(),
        });

        return Ok(Json(GetNextSubTaskResponse {
            sub_task: Some(subtask_mut),
            unit_task,
        }));
    }

    Ok(Json(GetNextSubTaskResponse {
        sub_task: None,
        unit_task: None,
    }))
}

pub async fn report_sub_task_status(
    State(state): State<SharedState>,
    Json(req): Json<ReportSubTaskStatusRequest>,
) -> AppResult<Json<ReportSubTaskStatusResponse>> {
    let mut subtask = state
        .store
        .get_sub_task(req.sub_task_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("SubTask {} not found", req.sub_task_id)))?;

    subtask.status = req.status;
    subtask.generated_commits = req.generated_commits;
    subtask.updated_at = chrono::Utc::now();

    // Update head commit if there are generated commits.
    if let Some(last_commit) = subtask.generated_commits.last() {
        subtask.head_commit_sha = Some(last_commit.sha.clone());
    }

    let subtask = state.store.update_sub_task(subtask).await?;
    tracing::info!(
        subtask_id = %subtask.id,
        status = ?subtask.status,
        "SubTask status reported"
    );

    // Update the parent unit task status based on subtask outcome.
    let unit_task_update = state.store.get_unit_task(subtask.unit_task_id).await?;
    if let Some(mut task) = unit_task_update {
        let new_task_status = match subtask.status {
            SubTaskStatus::Completed => UnitTaskStatus::Completed,
            SubTaskStatus::Failed => UnitTaskStatus::Failed,
            SubTaskStatus::Cancelled => UnitTaskStatus::Cancelled,
            SubTaskStatus::WaitingForPlanApproval | SubTaskStatus::WaitingForUserInput => {
                UnitTaskStatus::ActionRequired
            }
            _ => task.status,
        };

        if task.status != new_task_status {
            task.status = new_task_status;
            task.updated_at = chrono::Utc::now();

            // Update the latest commit SHA on the task.
            if let Some(sha) = &subtask.head_commit_sha {
                task.latest_commit_sha = Some(sha.clone());
                task.generated_commit_count += subtask.generated_commits.len() as u32;
            }

            let task = state.store.update_unit_task(task).await?;
            state.broker.publish(StreamEvent {
                event_type: StreamEventType::TaskUpdated,
                workspace_id: task.workspace_id.to_string(),
                payload: serde_json::to_value(&task).unwrap_or_default(),
            });
        }
    }

    state.broker.publish(StreamEvent {
        event_type: StreamEventType::SubtaskUpdated,
        workspace_id: String::new(),
        payload: serde_json::to_value(&subtask).unwrap_or_default(),
    });

    Ok(Json(ReportSubTaskStatusResponse {}))
}

pub async fn emit_session_event(
    State(state): State<SharedState>,
    Json(req): Json<EmitSessionEventRequest>,
) -> AppResult<Json<EmitSessionEventResponse>> {
    // Ensure the session exists; if not, create it lazily.
    let session_exists = state
        .store
        .get_agent_session(req.event.session_id)
        .await?
        .is_some();

    if !session_exists {
        // This should not normally happen — worker should have registered the session
        // first.
        tracing::warn!(
            session_id = %req.event.session_id,
            "Received event for unknown session — ignoring"
        );
        return Ok(Json(EmitSessionEventResponse {}));
    }

    let event = state.store.append_session_output(req.event).await?;
    tracing::debug!(
        session_id = %event.session_id,
        sequence = %event.sequence,
        "Session output event stored"
    );

    state.broker.publish(StreamEvent {
        event_type: StreamEventType::SessionOutput,
        workspace_id: String::new(),
        payload: serde_json::to_value(&event).unwrap_or_default(),
    });

    Ok(Json(EmitSessionEventResponse {}))
}
