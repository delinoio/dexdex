//! Task management API endpoints.

use std::sync::Arc;

use axum::{Json, extract::State};
use entities::{
    AgentTask, CompositeTask, CompositeTaskStatus as EntityCompositeTaskStatus, UnitTask,
    UnitTaskStatus as EntityUnitTaskStatus,
};
use plan_parser::{Plan, validate_plan};
use rpc_protocol::{CompositeTaskStatus, UnitTaskStatus, requests::*, responses::*};
use task_store::{TaskFilter, TaskStore};
use uuid::Uuid;

use crate::{
    error::{ServerError, ServerResult},
    state::AppState,
};

/// Maximum number of tasks allowed in a single composite task plan.
/// This prevents resource exhaustion from excessively large plans.
const MAX_TASKS_PER_PLAN: usize = 100;

/// Maximum value for `after_event_id` in log requests.
/// This prevents clients from sending unreasonably large values that could
/// cause unexpected behavior. 10 million lines is well beyond any realistic
/// log size.
const MAX_AFTER_EVENT_ID: i64 = 10_000_000;

/// Maximum number of log lines to return per session in a single response.
/// This prevents memory exhaustion from very large logs. Clients can use
/// `after_event_id` for incremental polling to get additional lines.
const MAX_LOG_LINES_PER_SESSION: usize = 5_000;

/// Converts RPC UnitTaskStatus to entity UnitTaskStatus.
fn to_entity_unit_status(status: UnitTaskStatus) -> EntityUnitTaskStatus {
    match status {
        UnitTaskStatus::Unspecified | UnitTaskStatus::InProgress => {
            EntityUnitTaskStatus::InProgress
        }
        UnitTaskStatus::InReview => EntityUnitTaskStatus::InReview,
        UnitTaskStatus::Approved => EntityUnitTaskStatus::Approved,
        UnitTaskStatus::PrOpen => EntityUnitTaskStatus::PrOpen,
        UnitTaskStatus::Done => EntityUnitTaskStatus::Done,
        UnitTaskStatus::Rejected => EntityUnitTaskStatus::Rejected,
        UnitTaskStatus::Failed => EntityUnitTaskStatus::Failed,
        UnitTaskStatus::Cancelled => EntityUnitTaskStatus::Cancelled,
    }
}

/// Converts entity UnitTaskStatus to RPC UnitTaskStatus.
fn to_rpc_unit_status(status: EntityUnitTaskStatus) -> UnitTaskStatus {
    match status {
        EntityUnitTaskStatus::InProgress => UnitTaskStatus::InProgress,
        EntityUnitTaskStatus::InReview => UnitTaskStatus::InReview,
        EntityUnitTaskStatus::Approved => UnitTaskStatus::Approved,
        EntityUnitTaskStatus::PrOpen => UnitTaskStatus::PrOpen,
        EntityUnitTaskStatus::Done => UnitTaskStatus::Done,
        EntityUnitTaskStatus::Rejected => UnitTaskStatus::Rejected,
        EntityUnitTaskStatus::Failed => UnitTaskStatus::Failed,
        EntityUnitTaskStatus::Cancelled => UnitTaskStatus::Cancelled,
    }
}

/// Converts RPC CompositeTaskStatus to entity CompositeTaskStatus.
fn to_entity_composite_status(status: CompositeTaskStatus) -> EntityCompositeTaskStatus {
    match status {
        CompositeTaskStatus::Unspecified | CompositeTaskStatus::Planning => {
            EntityCompositeTaskStatus::Planning
        }
        CompositeTaskStatus::PendingApproval => EntityCompositeTaskStatus::PendingApproval,
        CompositeTaskStatus::InProgress => EntityCompositeTaskStatus::InProgress,
        CompositeTaskStatus::Done => EntityCompositeTaskStatus::Done,
        CompositeTaskStatus::Rejected => EntityCompositeTaskStatus::Rejected,
        CompositeTaskStatus::Failed => EntityCompositeTaskStatus::Failed,
    }
}

/// Converts entity CompositeTaskStatus to RPC CompositeTaskStatus.
fn to_rpc_composite_status(status: EntityCompositeTaskStatus) -> CompositeTaskStatus {
    match status {
        EntityCompositeTaskStatus::Planning => CompositeTaskStatus::Planning,
        EntityCompositeTaskStatus::PendingApproval => CompositeTaskStatus::PendingApproval,
        EntityCompositeTaskStatus::InProgress => CompositeTaskStatus::InProgress,
        EntityCompositeTaskStatus::Done => CompositeTaskStatus::Done,
        EntityCompositeTaskStatus::Rejected => CompositeTaskStatus::Rejected,
        EntityCompositeTaskStatus::Failed => CompositeTaskStatus::Failed,
    }
}

/// Converts entity UnitTask to RPC UnitTask.
fn entity_to_rpc_unit_task(task: &entities::UnitTask) -> rpc_protocol::UnitTask {
    rpc_protocol::UnitTask {
        id: task.id.to_string(),
        repository_group_id: task.repository_group_id.to_string(),
        agent_task_id: task.agent_task_id.to_string(),
        prompt: task.prompt.clone(),
        title: task.title.clone(),
        branch_name: task.branch_name.clone(),
        linked_pr_url: task.linked_pr_url.clone(),
        base_commit: task.base_commit.clone(),
        end_commit: task.end_commit.clone(),
        git_patch: task.git_patch.clone(),
        auto_fix_task_ids: task
            .auto_fix_task_ids
            .iter()
            .map(|id| id.to_string())
            .collect(),
        status: to_rpc_unit_status(task.status),
        created_at: task.created_at,
        updated_at: task.updated_at,
    }
}

/// Converts entity CompositeTask to RPC CompositeTask.
fn entity_to_rpc_composite_task(task: &entities::CompositeTask) -> rpc_protocol::CompositeTask {
    rpc_protocol::CompositeTask {
        id: task.id.to_string(),
        repository_group_id: task.repository_group_id.to_string(),
        planning_task_id: task.planning_task_id.to_string(),
        prompt: task.prompt.clone(),
        title: task.title.clone(),
        plan_yaml: task.plan_yaml.clone(),
        update_plan_feedback: task.update_plan_feedback.clone(),
        node_ids: task.node_ids.iter().map(|id| id.to_string()).collect(),
        status: to_rpc_composite_status(task.status),
        execution_agent_type: task.execution_agent_type.map(|t| match t {
            entities::AiAgentType::ClaudeCode => rpc_protocol::AiAgentType::ClaudeCode,
            entities::AiAgentType::OpenCode => rpc_protocol::AiAgentType::OpenCode,
            entities::AiAgentType::GeminiCli => rpc_protocol::AiAgentType::GeminiCli,
            entities::AiAgentType::CodexCli => rpc_protocol::AiAgentType::CodexCli,
            entities::AiAgentType::Aider => rpc_protocol::AiAgentType::Aider,
            entities::AiAgentType::Amp => rpc_protocol::AiAgentType::Amp,
        }),
        created_at: task.created_at,
        updated_at: task.updated_at,
    }
}

/// Creates a new UnitTask.
pub async fn create_unit_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<CreateUnitTaskRequest>,
) -> ServerResult<Json<CreateUnitTaskResponse>> {
    let repository_group_id: Uuid = request
        .repository_group_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid repository_group_id".to_string()))?;

    // Verify repository group exists
    state
        .store
        .get_repository_group(repository_group_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Repository group not found".to_string()))?;

    // Create agent task first
    let mut agent_task = AgentTask::new();
    if let Some(agent_type) = request.ai_agent_type {
        agent_task.ai_agent_type = Some(match agent_type {
            rpc_protocol::AiAgentType::ClaudeCode => entities::AiAgentType::ClaudeCode,
            rpc_protocol::AiAgentType::OpenCode => entities::AiAgentType::OpenCode,
            rpc_protocol::AiAgentType::GeminiCli => entities::AiAgentType::GeminiCli,
            rpc_protocol::AiAgentType::CodexCli => entities::AiAgentType::CodexCli,
            rpc_protocol::AiAgentType::Aider => entities::AiAgentType::Aider,
            rpc_protocol::AiAgentType::Amp => entities::AiAgentType::Amp,
            _ => entities::AiAgentType::ClaudeCode,
        });
    }
    agent_task.ai_agent_model = request.ai_agent_model;
    let agent_task = state.store.create_agent_task(agent_task).await?;

    // Create unit task
    let mut unit_task = UnitTask::new(repository_group_id, agent_task.id, request.prompt);
    if let Some(title) = request.title {
        unit_task = unit_task.with_title(title);
    }
    if let Some(branch_name) = request.branch_name {
        unit_task = unit_task.with_branch_name(branch_name);
    }

    let task = state.store.create_unit_task(unit_task).await?;

    tracing::info!(task_id = %task.id, "UnitTask created");

    Ok(Json(CreateUnitTaskResponse {
        task: entity_to_rpc_unit_task(&task),
    }))
}

/// Creates a new CompositeTask.
pub async fn create_composite_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<CreateCompositeTaskRequest>,
) -> ServerResult<Json<CreateCompositeTaskResponse>> {
    let repository_group_id: Uuid = request
        .repository_group_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid repository_group_id".to_string()))?;

    // Verify repository group exists
    state
        .store
        .get_repository_group(repository_group_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Repository group not found".to_string()))?;

    // Create planning agent task
    let planning_task = AgentTask::new();
    let planning_task = state.store.create_agent_task(planning_task).await?;

    // Create composite task
    let mut composite_task =
        CompositeTask::new(repository_group_id, planning_task.id, request.prompt);
    if let Some(title) = request.title {
        composite_task = composite_task.with_title(title);
    }
    if let Some(agent_type) = request.execution_agent_type {
        composite_task.execution_agent_type = Some(match agent_type {
            rpc_protocol::AiAgentType::ClaudeCode => entities::AiAgentType::ClaudeCode,
            rpc_protocol::AiAgentType::OpenCode => entities::AiAgentType::OpenCode,
            rpc_protocol::AiAgentType::GeminiCli => entities::AiAgentType::GeminiCli,
            rpc_protocol::AiAgentType::CodexCli => entities::AiAgentType::CodexCli,
            rpc_protocol::AiAgentType::Aider => entities::AiAgentType::Aider,
            rpc_protocol::AiAgentType::Amp => entities::AiAgentType::Amp,
            _ => entities::AiAgentType::ClaudeCode,
        });
    }

    let task = state.store.create_composite_task(composite_task).await?;

    tracing::info!(task_id = %task.id, "CompositeTask created");

    Ok(Json(CreateCompositeTaskResponse {
        task: entity_to_rpc_composite_task(&task),
    }))
}

/// Gets a task by ID.
pub async fn get_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<GetTaskRequest>,
) -> ServerResult<Json<GetTaskResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    // Try to find as unit task first
    if let Some(task) = state.store.get_unit_task(task_id).await? {
        return Ok(Json(GetTaskResponse::UnitTask {
            unit_task: entity_to_rpc_unit_task(&task),
        }));
    }

    // Try composite task
    if let Some(task) = state.store.get_composite_task(task_id).await? {
        return Ok(Json(GetTaskResponse::CompositeTask {
            composite_task: entity_to_rpc_composite_task(&task),
        }));
    }

    Err(ServerError::NotFound("Task not found".to_string()))
}

/// Lists tasks with filters.
pub async fn list_tasks<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<ListTasksRequest>,
) -> ServerResult<Json<ListTasksResponse>> {
    let filter = TaskFilter {
        repository_group_id: request
            .repository_group_id
            .as_ref()
            .and_then(|id| id.parse().ok()),
        unit_status: request.unit_status.map(to_entity_unit_status),
        composite_status: request.composite_status.map(to_entity_composite_status),
        limit: Some(request.limit as u32),
        offset: Some(request.offset as u32),
    };

    let (unit_tasks, unit_count) = state.store.list_unit_tasks(filter.clone()).await?;
    let (composite_tasks, composite_count) = state.store.list_composite_tasks(filter).await?;

    Ok(Json(ListTasksResponse {
        unit_tasks: unit_tasks.iter().map(entity_to_rpc_unit_task).collect(),
        composite_tasks: composite_tasks
            .iter()
            .map(entity_to_rpc_composite_task)
            .collect(),
        total_count: (unit_count + composite_count) as i32,
    }))
}

/// Updates a task's status.
pub async fn update_task_status<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<UpdateTaskStatusRequest>,
) -> ServerResult<Json<UpdateTaskStatusResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    // Try unit task first
    if let Some(mut task) = state.store.get_unit_task(task_id).await?
        && let Some(status) = request.unit_status
    {
        task.status = to_entity_unit_status(status);
        task.updated_at = chrono::Utc::now();
        let task = state.store.update_unit_task(task).await?;
        return Ok(Json(UpdateTaskStatusResponse::UnitTask {
            unit_task: entity_to_rpc_unit_task(&task),
        }));
    }

    // Try composite task
    if let Some(mut task) = state.store.get_composite_task(task_id).await?
        && let Some(status) = request.composite_status
    {
        task.status = to_entity_composite_status(status);
        task.updated_at = chrono::Utc::now();
        let task = state.store.update_composite_task(task).await?;
        return Ok(Json(UpdateTaskStatusResponse::CompositeTask {
            composite_task: entity_to_rpc_composite_task(&task),
        }));
    }

    Err(ServerError::NotFound("Task not found".to_string()))
}

/// Deletes a task.
///
/// For unit tasks, the associated agent task, its sessions, and auto-fix agent
/// tasks are also deleted. For composite tasks, all child nodes, their unit
/// tasks, all associated agent tasks and sessions, and the planning task are
/// deleted before the composite task itself.
///
/// If a unit task is currently in progress, its status is set to Cancelled
/// before deletion so that any worker processing the task can detect the
/// cancellation.
pub async fn delete_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<DeleteTaskRequest>,
) -> ServerResult<Json<DeleteTaskResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    // Try unit task first
    if let Some(unit_task) = state.store.get_unit_task(task_id).await? {
        // If the task is currently running, cancel it first so any worker
        // processing the task can detect the cancellation.
        if unit_task.status == EntityUnitTaskStatus::InProgress {
            let mut cancelled_task = unit_task;
            cancelled_task.status = EntityUnitTaskStatus::Cancelled;
            cancelled_task.updated_at = chrono::Utc::now();
            if let Err(e) = state.store.update_unit_task(cancelled_task).await {
                tracing::warn!(
                    task_id = %task_id,
                    error = %e,
                    "Failed to set task status to Cancelled before deletion"
                );
            } else {
                tracing::info!(task_id = %task_id, "Set running task to Cancelled before deletion");
            }
        }

        // Cascade delete: unit task + agent task + sessions + auto-fix tasks
        state.store.delete_unit_task_cascade(task_id).await?;
        tracing::info!(task_id = %task_id, "UnitTask deleted with cascade");
        return Ok(Json(DeleteTaskResponse {}));
    }

    // Try composite task (fetch once and reuse to avoid TOCTOU race)
    if let Some(_composite_task) = state.store.get_composite_task(task_id).await? {
        // Cancel any in-progress child unit tasks before deletion
        let nodes = state.store.list_composite_task_nodes(task_id).await?;
        for node in &nodes {
            if let Some(child_unit_task) = state.store.get_unit_task(node.unit_task_id).await?
                && child_unit_task.status == EntityUnitTaskStatus::InProgress
            {
                let mut cancelled = child_unit_task;
                cancelled.status = EntityUnitTaskStatus::Cancelled;
                cancelled.updated_at = chrono::Utc::now();
                if let Err(e) = state.store.update_unit_task(cancelled).await {
                    tracing::warn!(
                        composite_task_id = %task_id,
                        unit_task_id = %node.unit_task_id,
                        error = %e,
                        "Failed to cancel child unit task before composite deletion"
                    );
                }
            }
        }

        // Cascade delete: composite task + all nodes + unit tasks + agent tasks +
        // sessions
        state.store.delete_composite_task_cascade(task_id).await?;
        tracing::info!(task_id = %task_id, "CompositeTask deleted with cascade");
        return Ok(Json(DeleteTaskResponse {}));
    }

    Err(ServerError::NotFound("Task not found".to_string()))
}

/// Retries a failed task.
pub async fn retry_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<RetryTaskRequest>,
) -> ServerResult<Json<RetryTaskResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    let mut task = state
        .store
        .get_unit_task(task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Task not found".to_string()))?;

    // Reset status to in_progress
    task.status = EntityUnitTaskStatus::InProgress;
    task.updated_at = chrono::Utc::now();
    let task = state.store.update_unit_task(task).await?;

    tracing::info!(task_id = %task_id, "Task retried");

    Ok(Json(RetryTaskResponse {
        task: entity_to_rpc_unit_task(&task),
    }))
}

/// Approves a task.
pub async fn approve_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<ApproveTaskRequest>,
) -> ServerResult<Json<ApproveTaskResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    // Try unit task first
    if let Some(mut task) = state.store.get_unit_task(task_id).await? {
        task.status = EntityUnitTaskStatus::Approved;
        task.updated_at = chrono::Utc::now();
        state.store.update_unit_task(task).await?;
        tracing::info!(task_id = %task_id, "UnitTask approved");
        return Ok(Json(ApproveTaskResponse {}));
    }

    // Try composite task
    if let Some(mut task) = state.store.get_composite_task(task_id).await? {
        if task.status != EntityCompositeTaskStatus::PendingApproval {
            return Err(ServerError::InvalidRequest(
                "Composite task is not in PendingApproval status".to_string(),
            ));
        }

        // Validate the plan before approving
        if let Some(ref plan_yaml) = task.plan_yaml {
            match validate_composite_task_plan(plan_yaml) {
                Ok(plan) => {
                    tracing::info!(
                        task_id = %task_id,
                        task_count = plan.tasks.len(),
                        "Plan validated successfully"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        task_id = %task_id,
                        error = %e,
                        "Plan validation failed during approval"
                    );
                    task.status = EntityCompositeTaskStatus::Failed;
                    state.store.update_composite_task(task).await?;
                    return Err(e);
                }
            }
        }

        // Only validate and change status on the server side.
        // Node creation is delegated to the executor (LocalExecutor or
        // worker) to avoid duplicate node creation.
        task.status = EntityCompositeTaskStatus::InProgress;
        task.updated_at = chrono::Utc::now();
        state.store.update_composite_task(task).await?;
        tracing::info!(task_id = %task_id, "CompositeTask approved and execution started");
        return Ok(Json(ApproveTaskResponse {}));
    }

    Err(ServerError::NotFound("Task not found".to_string()))
}

/// Rejects a task.
pub async fn reject_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<RejectTaskRequest>,
) -> ServerResult<Json<RejectTaskResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    // Try unit task first
    if let Some(mut task) = state.store.get_unit_task(task_id).await? {
        task.status = EntityUnitTaskStatus::Rejected;
        task.updated_at = chrono::Utc::now();
        state.store.update_unit_task(task).await?;
        tracing::info!(task_id = %task_id, reason = ?request.reason, "UnitTask rejected");
        return Ok(Json(RejectTaskResponse {}));
    }

    // Try composite task
    if let Some(mut task) = state.store.get_composite_task(task_id).await? {
        task.status = EntityCompositeTaskStatus::Rejected;
        task.updated_at = chrono::Utc::now();
        state.store.update_composite_task(task).await?;
        tracing::info!(task_id = %task_id, reason = ?request.reason, "CompositeTask rejected");
        return Ok(Json(RejectTaskResponse {}));
    }

    Err(ServerError::NotFound("Task not found".to_string()))
}

/// Updates the plan for a composite task by appending feedback and resetting to
/// Planning status.
pub async fn update_plan<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<UpdatePlanRequest>,
) -> ServerResult<Json<UpdatePlanResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    let mut task = state
        .store
        .get_composite_task(task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Composite task not found".to_string()))?;

    // Only allow re-planning from PendingApproval or Failed states
    let previous_status = task.status;
    if previous_status != EntityCompositeTaskStatus::PendingApproval
        && previous_status != EntityCompositeTaskStatus::Failed
    {
        return Err(ServerError::InvalidRequest(format!(
            "Cannot update plan: task is in {} status (must be PendingApproval or Failed)",
            previous_status
        )));
    }

    // Sanitize and validate the feedback prompt
    let sanitized_prompt = entities::sanitize_user_input(&request.prompt);
    if sanitized_prompt.len() > entities::MAX_FEEDBACK_LENGTH {
        return Err(ServerError::InvalidRequest(format!(
            "Feedback exceeds maximum length of {} characters",
            entities::MAX_FEEDBACK_LENGTH
        )));
    }

    // Store the feedback for re-planning. The executor will use the existing
    // plan_yaml together with this feedback (instead of the original prompt)
    // to generate a new plan.
    task.update_plan_feedback = Some(sanitized_prompt);
    task.status = EntityCompositeTaskStatus::Planning;
    task.updated_at = chrono::Utc::now();

    // Create a new planning agent task
    let planning_agent_task = AgentTask::new();
    let planning_agent_task = state.store.create_agent_task(planning_agent_task).await?;
    task.planning_task_id = planning_agent_task.id;

    let task = state.store.update_composite_task(task).await?;

    tracing::info!(
        task_id = %task_id,
        prompt_length = request.prompt.len(),
        previous_status = %previous_status,
        "Plan updated for re-planning"
    );

    Ok(Json(UpdatePlanResponse {
        task: entity_to_rpc_composite_task(&task),
    }))
}

/// Validates a composite task's plan YAML, checking for parse errors,
/// validation errors (cycles, invalid deps, etc.), and resource limits.
///
/// Returns `Ok(plan)` if valid, or an appropriate `ServerError`.
///
/// NOTE: This validation is intentionally duplicated here (server API boundary)
/// and in `LocalExecutor::execute_composite_task_graph` (executor). The server
/// validates for immediate user feedback when approving via the remote API,
/// while the executor validates for the desktop (Tauri) code path where
/// approval bypasses the server. Both paths must reject invalid plans.
fn validate_composite_task_plan(plan_yaml: &str) -> ServerResult<Plan> {
    let plan = Plan::from_yaml(plan_yaml)
        .map_err(|e| ServerError::Internal(format!("Failed to parse plan YAML: {}", e)))?;

    let validation = validate_plan(&plan);
    if !validation.is_valid() {
        return Err(ServerError::InvalidRequest(format!(
            "Invalid plan: {:?}",
            validation.errors
        )));
    }

    if plan.tasks.len() > MAX_TASKS_PER_PLAN {
        return Err(ServerError::InvalidRequest(format!(
            "Plan has {} tasks, exceeding the maximum of {}",
            plan.tasks.len(),
            MAX_TASKS_PER_PLAN
        )));
    }

    Ok(plan)
}

/// Cancels a running task.
///
/// This signals the worker executing the task (if any) to stop via the
/// worker's `/cancel` endpoint, and then updates the task status to
/// Cancelled in the database. By signaling the worker first, we avoid a
/// race window where the DB says "Cancelled" but the worker is still
/// running unaware.
pub async fn cancel_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<CancelTaskRequest>,
) -> ServerResult<Json<CancelTaskResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    let mut task = state
        .store
        .get_unit_task(task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Task not found".to_string()))?;

    if task.status != EntityUnitTaskStatus::InProgress {
        return Err(ServerError::InvalidRequest(format!(
            "Task {} is not in InProgress status (current: {:?})",
            task_id, task.status
        )));
    }

    // Signal the worker to stop execution BEFORE updating the database.
    // This avoids a race window where the DB says "Cancelled" but the
    // worker hasn't been notified yet and continues executing.
    let worker_endpoint = {
        let registry = state.worker_registry.read().await;
        registry
            .find_worker_by_task_id(task_id)
            .map(|w| w.endpoint_url.clone())
    };

    if let Some(endpoint_url) = worker_endpoint {
        // SECURITY: Defense-in-depth URL validation. The URL was validated at
        // registration time, but we re-validate before making outbound HTTP
        // requests to guard against any data corruption or bypass.
        if let Err(e) = crate::services::worker_registry::validate_worker_endpoint_url(&endpoint_url) {
            tracing::error!(
                task_id = %task_id,
                endpoint_url = %endpoint_url,
                error = %e,
                "Refusing to send cancel request to invalid worker endpoint URL"
            );
            return Err(ServerError::Internal(format!(
                "Worker has invalid endpoint URL: {}",
                e
            )));
        }

        let cancel_url = format!("{}/cancel", endpoint_url.trim_end_matches('/'));
        tracing::info!(task_id = %task_id, cancel_url = %cancel_url, "Signaling worker to cancel task");

        let payload = serde_json::json!({ "task_id": task_id });
        match state.http_client.post(&cancel_url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!(task_id = %task_id, "Worker acknowledged cancellation");
            }
            Ok(resp) => {
                // Worker actively rejected the cancellation request. Do NOT
                // mark the task as cancelled in the DB because the worker may
                // still be running. Return an error so the client can retry.
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::error!(
                    task_id = %task_id,
                    status = %status,
                    body = %body,
                    "Worker rejected cancellation request"
                );
                return Err(ServerError::Internal(format!(
                    "Worker rejected cancellation (HTTP {}): {}",
                    status, body
                )));
            }
            Err(e) => {
                // Worker is unreachable (may have crashed or lost network).
                // Proceed with cancellation in the DB since the worker won't
                // be able to continue the task anyway.
                tracing::warn!(
                    task_id = %task_id,
                    error = %e,
                    "Failed to reach worker for cancellation, proceeding with DB update"
                );
            }
        }
    } else {
        tracing::debug!(task_id = %task_id, "No worker found for task, skipping worker signal");
    }

    // Update DB status to Cancelled after signaling the worker.
    task.status = EntityUnitTaskStatus::Cancelled;
    task.updated_at = chrono::Utc::now();
    state.store.update_unit_task(task).await?;

    tracing::info!(task_id = %task_id, "Task cancelled");

    Ok(Json(CancelTaskResponse {}))
}

/// Dismisses approval for a task, moving it back to InReview.
pub async fn dismiss_approval<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<DismissApprovalRequest>,
) -> ServerResult<Json<DismissApprovalResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    let mut task = state
        .store
        .get_unit_task(task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Task not found".to_string()))?;

    if task.status != EntityUnitTaskStatus::Approved {
        return Err(ServerError::InvalidRequest(format!(
            "Task {} is not in Approved status (current: {:?})",
            task_id, task.status
        )));
    }

    task.status = EntityUnitTaskStatus::InReview;
    task.updated_at = chrono::Utc::now();
    state.store.update_unit_task(task).await?;

    tracing::info!(task_id = %task_id, "Approval dismissed");

    Ok(Json(DismissApprovalResponse {}))
}

/// Creates a pull request for an approved task.
///
/// In the server context, PR creation is delegated to the worker. The server
/// transitions the task status to InProgress with a PR creation prompt so the
/// worker picks it up.
pub async fn create_pr<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<CreatePrRequest>,
) -> ServerResult<Json<CreatePrResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    let mut task = state
        .store
        .get_unit_task(task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Task not found".to_string()))?;

    if task.status != EntityUnitTaskStatus::Approved
        && task.status != EntityUnitTaskStatus::InReview
    {
        return Err(ServerError::InvalidRequest(format!(
            "Task {} is not in Approved or InReview status (current: {:?})",
            task_id, task.status
        )));
    }

    // Reset to InProgress so the worker picks it up for PR creation.
    // Guard against duplicate appends: if the user retries after a failure,
    // the PR creation instructions may already be present in the prompt.
    const PR_CREATION_MARKER: &str = "--- Create PR ---";
    task.status = EntityUnitTaskStatus::InProgress;
    if !task.prompt.contains(PR_CREATION_MARKER) {
        task.prompt = format!(
            "{}\n\n{}\nCreate a pull request with the changes from this task. Push the \
             current branch to the remote and create a PR using the available tools (e.g. `gh pr \
             create`). Output the PR URL.",
            task.prompt, PR_CREATION_MARKER
        );
    }
    task.updated_at = chrono::Utc::now();
    state.store.update_unit_task(task).await?;

    tracing::info!(task_id = %task_id, "PR creation requested, task reset to InProgress for worker");

    // The actual PR URL will be available after the worker completes
    Ok(Json(CreatePrResponse {
        pr_url: String::new(),
    }))
}

/// Gets all composite task nodes.
pub async fn get_composite_task_nodes<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<GetCompositeTaskNodesRequest>,
) -> ServerResult<Json<GetCompositeTaskNodesResponse>> {
    let composite_task_id: Uuid = request
        .composite_task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid composite_task_id".to_string()))?;

    // Verify composite task exists
    state
        .store
        .get_composite_task(composite_task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Composite task not found".to_string()))?;

    let nodes = state
        .store
        .list_composite_task_nodes(composite_task_id)
        .await?;

    let rpc_nodes: Vec<rpc_protocol::CompositeTaskNode> = nodes
        .iter()
        .map(|node| rpc_protocol::CompositeTaskNode {
            id: node.id.to_string(),
            composite_task_id: node.composite_task_id.to_string(),
            unit_task_id: node.unit_task_id.to_string(),
            depends_on_ids: node
                .depends_on_ids
                .iter()
                .map(|id| id.to_string())
                .collect(),
            created_at: node.created_at,
        })
        .collect();

    // Fetch all unit tasks associated with the nodes to avoid N+1 queries
    // on the client side.
    let mut rpc_unit_tasks = Vec::with_capacity(nodes.len());
    for node in &nodes {
        if let Some(unit_task) = state.store.get_unit_task(node.unit_task_id).await? {
            rpc_unit_tasks.push(entity_to_rpc_unit_task(&unit_task));
        } else {
            tracing::warn!(
                node_id = %node.id,
                unit_task_id = %node.unit_task_id,
                "CompositeTaskNode references missing UnitTask"
            );
        }
    }

    Ok(Json(GetCompositeTaskNodesResponse {
        nodes: rpc_nodes,
        unit_tasks: rpc_unit_tasks,
    }))
}

/// Gets task logs (agent sessions and their events).
pub async fn get_task_logs<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<GetTaskLogsRequest>,
) -> ServerResult<Json<GetTaskLogsResponse>> {
    let agent_task_id: Uuid = request
        .agent_task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid agent_task_id".to_string()))?;

    // Validate after_event_id if provided
    if let Some(after_id) = request.after_event_id {
        if after_id < 0 {
            return Err(ServerError::InvalidRequest(
                "after_event_id must be non-negative".to_string(),
            ));
        }
        if after_id > MAX_AFTER_EVENT_ID {
            return Err(ServerError::InvalidRequest(format!(
                "after_event_id exceeds maximum value of {}",
                MAX_AFTER_EVENT_ID
            )));
        }
    }

    // Verify agent task exists
    state
        .store
        .get_agent_task(agent_task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Agent task not found".to_string()))?;

    // Get all sessions for this agent task
    let mut sessions = state.store.list_agent_sessions(agent_task_id).await?;

    // Sort by created_at for chronological order
    sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // Convert sessions to RPC format.
    // When after_event_id is provided, only include lines after that ID in the
    // output_log to avoid sending the entire log on every poll. Additionally,
    // truncate output_log to MAX_LOG_LINES_PER_SESSION lines to prevent memory
    // exhaustion from very large logs.
    let after_id = request.after_event_id;
    let rpc_sessions: Vec<rpc_protocol::AgentSession> = sessions
        .iter()
        .map(|s| {
            let truncated_log = s.output_log.as_ref().map(|log| {
                let lines: Vec<&str> = log.lines().collect();
                let total = lines.len();

                // If after_event_id is provided, skip lines up to and including
                // that ID (line index).
                let start = if let Some(aid) = after_id {
                    (aid as usize + 1).min(total)
                } else {
                    // No after_event_id: return the last MAX_LOG_LINES_PER_SESSION lines
                    total.saturating_sub(MAX_LOG_LINES_PER_SESSION)
                };

                let end = total.min(start + MAX_LOG_LINES_PER_SESSION);
                lines[start..end].join("\n")
            });

            rpc_protocol::AgentSession {
                id: s.id.to_string(),
                agent_task_id: s.agent_task_id.to_string(),
                ai_agent_type: match s.ai_agent_type {
                    entities::AiAgentType::ClaudeCode => rpc_protocol::AiAgentType::ClaudeCode,
                    entities::AiAgentType::OpenCode => rpc_protocol::AiAgentType::OpenCode,
                    entities::AiAgentType::GeminiCli => rpc_protocol::AiAgentType::GeminiCli,
                    entities::AiAgentType::CodexCli => rpc_protocol::AiAgentType::CodexCli,
                    entities::AiAgentType::Aider => rpc_protocol::AiAgentType::Aider,
                    entities::AiAgentType::Amp => rpc_protocol::AiAgentType::Amp,
                },
                ai_agent_model: s.ai_agent_model.clone(),
                started_at: s.started_at,
                completed_at: s.completed_at,
                output_log: truncated_log,
                token_usage: s.token_usage.as_ref().map(|tu| rpc_protocol::TokenUsage {
                    input_tokens: tu.input_tokens,
                    output_tokens: tu.output_tokens,
                    cache_read_input_tokens: tu.cache_read_input_tokens,
                    cache_creation_input_tokens: tu.cache_creation_input_tokens,
                    total_cost_usd: tu.total_cost_usd,
                    duration_ms: tu.duration_ms,
                    num_turns: tu.num_turns,
                }),
                created_at: s.created_at,
            }
        })
        .collect();

    // Compute last_event_id from the latest session's output log line count.
    // The client parses events directly from sessions' output_log fields,
    // so we only need to provide last_event_id for incremental polling and
    // an empty events list (the client builds events from output_log).
    let last_event_id: Option<i64> = sessions.last().and_then(|s| {
        s.output_log.as_ref().map(|log| {
            let total_lines = log.lines().count() as i64;
            if total_lines > 0 { total_lines - 1 } else { 0 }
        })
    });

    Ok(Json(GetTaskLogsResponse {
        sessions: rpc_sessions,
        events: Vec::new(),
        last_event_id,
    }))
}

/// Requests changes on a task.
pub async fn request_changes<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<RequestChangesRequest>,
) -> ServerResult<Json<RequestChangesResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    let mut task = state
        .store
        .get_unit_task(task_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Task not found".to_string()))?;

    // Reset to in_progress and append feedback to prompt
    task.status = EntityUnitTaskStatus::InProgress;
    task.prompt = format!(
        "{}\n\n--- Requested Changes ---\n{}",
        task.prompt, request.feedback
    );
    task.updated_at = chrono::Utc::now();
    let task = state.store.update_unit_task(task).await?;

    // If this unit task belongs to a composite task, ensure the composite
    // task is also marked as InProgress so the dashboard reflects ongoing work.
    if let Ok(Some(composite_task_id)) = state
        .store
        .find_composite_task_id_by_unit_task_id(task_id)
        .await
        && let Ok(Some(mut ct)) = state.store.get_composite_task(composite_task_id).await
        && ct.status != entities::CompositeTaskStatus::InProgress
    {
        tracing::info!(
            composite_task_id = %composite_task_id,
            task_id = %task_id,
            "Transitioning parent composite task to InProgress due to request_changes"
        );
        ct.status = entities::CompositeTaskStatus::InProgress;
        ct.updated_at = chrono::Utc::now();
        if let Err(e) = state.store.update_composite_task(ct).await {
            tracing::warn!(
                composite_task_id = %composite_task_id,
                error = %e,
                "Failed to update composite task status to InProgress"
            );
        }
    }

    tracing::info!(task_id = %task_id, "Changes requested on task");

    Ok(Json(RequestChangesResponse {
        task: entity_to_rpc_unit_task(&task),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_plan() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: "Task A"
  - id: "b"
    prompt: "Task B"
    dependsOn: ["a"]
"#;
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.tasks.len(), 2);
    }

    #[test]
    fn test_validate_plan_with_cycle() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: "Task A"
    dependsOn: ["b"]
  - id: "b"
    prompt: "Task B"
    dependsOn: ["a"]
"#;
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("CyclicDependency"),
            "Expected CyclicDependency error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_validate_plan_with_invalid_dependency() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: "Task A"
    dependsOn: ["non-existent"]
"#;
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("InvalidDependency"),
            "Expected InvalidDependency error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_validate_plan_with_duplicate_ids() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: "Task A"
  - id: "a"
    prompt: "Task A again"
"#;
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("DuplicateTaskId"),
            "Expected DuplicateTaskId error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_validate_plan_with_empty_prompt() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: ""
"#;
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("EmptyPrompt"),
            "Expected EmptyPrompt error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_validate_plan_invalid_yaml() {
        let yaml = "this is not valid yaml for a plan";
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_plan_exceeds_max_tasks() {
        // Create a plan with more than MAX_TASKS_PER_PLAN tasks
        let mut tasks = String::from("tasks:\n");
        for i in 0..(MAX_TASKS_PER_PLAN + 1) {
            tasks.push_str(&format!(
                "  - id: \"task-{}\"\n    prompt: \"Task {}\"\n",
                i, i
            ));
        }
        let result = validate_composite_task_plan(&tasks);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("exceeding the maximum"),
            "Expected max tasks error, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_validate_plan_at_max_tasks() {
        // Create a plan with exactly MAX_TASKS_PER_PLAN tasks (should succeed)
        let mut tasks = String::from("tasks:\n");
        for i in 0..MAX_TASKS_PER_PLAN {
            tasks.push_str(&format!(
                "  - id: \"task-{}\"\n    prompt: \"Task {}\"\n",
                i, i
            ));
        }
        let result = validate_composite_task_plan(&tasks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_plan_with_diamond_dependencies() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: "Task A"
  - id: "b"
    prompt: "Task B"
    dependsOn: ["a"]
  - id: "c"
    prompt: "Task C"
    dependsOn: ["a"]
  - id: "d"
    prompt: "Task D"
    dependsOn: ["b", "c"]
"#;
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.tasks.len(), 4);
    }

    #[test]
    fn test_validate_plan_self_dependency() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: "Task A"
    dependsOn: ["a"]
"#;
        let result = validate_composite_task_plan(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(
            err_msg.contains("CyclicDependency"),
            "Expected CyclicDependency error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_approve_composite_task_with_valid_plan() {
        use task_store::MemoryTaskStore;

        let store = MemoryTaskStore::new();
        let config = crate::config::Config {
            host: "127.0.0.1".to_string(),
            port: 54871,
            database_url: "sqlite::memory:".to_string(),
            single_user_mode: true,
            jwt_secret: None,
            jwt_expiration_hours: 24,
            oidc_issuer_url: None,
            oidc_client_id: None,
            oidc_client_secret: None,
            oidc_redirect_url: None,
            log_level: "info".to_string(),
            webhook_secret: None,
        };
        let state = Arc::new(AppState::new(config, store, None));

        // Create repository group
        let workspace = entities::Workspace::new("Test Workspace");
        let workspace = state.store.create_workspace(workspace).await.unwrap();
        let repo = entities::Repository::new(
            workspace.id,
            "test-repo",
            "https://github.com/test/repo.git",
            entities::VcsProviderType::Github,
        );
        let repo = state.store.create_repository(repo).await.unwrap();
        let mut repo_group = entities::RepositoryGroup::new(workspace.id);
        repo_group.repository_ids.push(repo.id);
        let repo_group = state
            .store
            .create_repository_group(repo_group)
            .await
            .unwrap();

        // Create a planning agent task
        let planning_task = AgentTask::new();
        let planning_task = state.store.create_agent_task(planning_task).await.unwrap();

        // Create composite task in PendingApproval status with valid plan
        let valid_plan = r#"
tasks:
  - id: "a"
    prompt: "Task A"
  - id: "b"
    prompt: "Task B"
    dependsOn: ["a"]
"#;
        let mut composite_task = CompositeTask::new(repo_group.id, planning_task.id, "Test task");
        composite_task.status = EntityCompositeTaskStatus::PendingApproval;
        composite_task.plan_yaml = Some(valid_plan.to_string());
        let composite_task = state
            .store
            .create_composite_task(composite_task)
            .await
            .unwrap();

        // Approve the task
        let request = ApproveTaskRequest {
            task_id: composite_task.id.to_string(),
        };
        let result = approve_task(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify the status was updated to InProgress
        let updated = state
            .store
            .get_composite_task(composite_task.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, EntityCompositeTaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_approve_composite_task_with_invalid_plan_fails() {
        use task_store::MemoryTaskStore;

        let store = MemoryTaskStore::new();
        let config = crate::config::Config {
            host: "127.0.0.1".to_string(),
            port: 54871,
            database_url: "sqlite::memory:".to_string(),
            single_user_mode: true,
            jwt_secret: None,
            jwt_expiration_hours: 24,
            oidc_issuer_url: None,
            oidc_client_id: None,
            oidc_client_secret: None,
            oidc_redirect_url: None,
            log_level: "info".to_string(),
            webhook_secret: None,
        };
        let state = Arc::new(AppState::new(config, store, None));

        // Create repository group
        let workspace = entities::Workspace::new("Test Workspace");
        let workspace = state.store.create_workspace(workspace).await.unwrap();
        let repo = entities::Repository::new(
            workspace.id,
            "test-repo",
            "https://github.com/test/repo.git",
            entities::VcsProviderType::Github,
        );
        let repo = state.store.create_repository(repo).await.unwrap();
        let mut repo_group = entities::RepositoryGroup::new(workspace.id);
        repo_group.repository_ids.push(repo.id);
        let repo_group = state
            .store
            .create_repository_group(repo_group)
            .await
            .unwrap();

        // Create a planning agent task
        let planning_task = AgentTask::new();
        let planning_task = state.store.create_agent_task(planning_task).await.unwrap();

        // Create composite task with cyclic plan
        let cyclic_plan = r#"
tasks:
  - id: "a"
    prompt: "Task A"
    dependsOn: ["b"]
  - id: "b"
    prompt: "Task B"
    dependsOn: ["a"]
"#;
        let mut composite_task = CompositeTask::new(repo_group.id, planning_task.id, "Test task");
        composite_task.status = EntityCompositeTaskStatus::PendingApproval;
        composite_task.plan_yaml = Some(cyclic_plan.to_string());
        let composite_task = state
            .store
            .create_composite_task(composite_task)
            .await
            .unwrap();

        // Approve the task should fail
        let request = ApproveTaskRequest {
            task_id: composite_task.id.to_string(),
        };
        let result = approve_task(State(state.clone()), Json(request)).await;
        assert!(result.is_err());

        // Verify the status was set to Failed
        let updated = state
            .store
            .get_composite_task(composite_task.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, EntityCompositeTaskStatus::Failed);
    }

    #[tokio::test]
    async fn test_approve_composite_task_no_node_creation() {
        use task_store::MemoryTaskStore;

        let store = MemoryTaskStore::new();
        let config = crate::config::Config {
            host: "127.0.0.1".to_string(),
            port: 54871,
            database_url: "sqlite::memory:".to_string(),
            single_user_mode: true,
            jwt_secret: None,
            jwt_expiration_hours: 24,
            oidc_issuer_url: None,
            oidc_client_id: None,
            oidc_client_secret: None,
            oidc_redirect_url: None,
            log_level: "info".to_string(),
            webhook_secret: None,
        };
        let state = Arc::new(AppState::new(config, store, None));

        // Create repository group
        let workspace = entities::Workspace::new("Test Workspace");
        let workspace = state.store.create_workspace(workspace).await.unwrap();
        let repo = entities::Repository::new(
            workspace.id,
            "test-repo",
            "https://github.com/test/repo.git",
            entities::VcsProviderType::Github,
        );
        let repo = state.store.create_repository(repo).await.unwrap();
        let mut repo_group = entities::RepositoryGroup::new(workspace.id);
        repo_group.repository_ids.push(repo.id);
        let repo_group = state
            .store
            .create_repository_group(repo_group)
            .await
            .unwrap();

        // Create a planning agent task
        let planning_task = AgentTask::new();
        let planning_task = state.store.create_agent_task(planning_task).await.unwrap();

        // Create composite task with valid plan
        let valid_plan = r#"
tasks:
  - id: "a"
    prompt: "Task A"
  - id: "b"
    prompt: "Task B"
    dependsOn: ["a"]
"#;
        let mut composite_task = CompositeTask::new(repo_group.id, planning_task.id, "Test task");
        composite_task.status = EntityCompositeTaskStatus::PendingApproval;
        composite_task.plan_yaml = Some(valid_plan.to_string());
        let composite_task = state
            .store
            .create_composite_task(composite_task)
            .await
            .unwrap();

        // Approve the task
        let request = ApproveTaskRequest {
            task_id: composite_task.id.to_string(),
        };
        let _result = approve_task(State(state.clone()), Json(request))
            .await
            .unwrap();

        // Verify that no CompositeTaskNodes were created
        // (node creation is delegated to the executor)
        let nodes = state
            .store
            .list_composite_task_nodes(composite_task.id)
            .await
            .unwrap();
        assert!(
            nodes.is_empty(),
            "Server approval should not create nodes; expected 0, got {}",
            nodes.len()
        );
    }

    /// Helper to create test state with a repository group ready for creating
    /// tasks.
    async fn create_test_state_with_repo_group()
    -> (Arc<AppState<task_store::MemoryTaskStore>>, Uuid) {
        use task_store::MemoryTaskStore;

        let store = MemoryTaskStore::new();
        let config = crate::config::Config {
            host: "127.0.0.1".to_string(),
            port: 54871,
            database_url: "sqlite::memory:".to_string(),
            single_user_mode: true,
            jwt_secret: None,
            jwt_expiration_hours: 24,
            oidc_issuer_url: None,
            oidc_client_id: None,
            oidc_client_secret: None,
            oidc_redirect_url: None,
            log_level: "info".to_string(),
            webhook_secret: None,
        };
        let state = Arc::new(AppState::new(config, store, None));

        let workspace = entities::Workspace::new("Test Workspace");
        let workspace = state.store.create_workspace(workspace).await.unwrap();
        let repo = entities::Repository::new(
            workspace.id,
            "test-repo",
            "https://github.com/test/repo.git",
            entities::VcsProviderType::Github,
        );
        let repo = state.store.create_repository(repo).await.unwrap();
        let mut repo_group = entities::RepositoryGroup::new(workspace.id);
        repo_group.repository_ids.push(repo.id);
        let repo_group = state
            .store
            .create_repository_group(repo_group)
            .await
            .unwrap();

        (state, repo_group.id)
    }

    #[tokio::test]
    async fn test_delete_standalone_unit_task() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        // Create agent task and unit task
        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let unit_task = UnitTask::new(repo_group_id, agent_task.id, "Test task");
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        // Verify task exists
        assert!(
            state
                .store
                .get_unit_task(unit_task.id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            state
                .store
                .get_agent_task(agent_task.id)
                .await
                .unwrap()
                .is_some()
        );

        // Delete the task
        let request = rpc_protocol::requests::DeleteTaskRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = delete_task(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify both the unit task and agent task are deleted
        assert!(
            state
                .store
                .get_unit_task(unit_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(agent_task.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_delete_unit_task_with_auto_fix_tasks() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        // Create main agent task
        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        // Create auto-fix agent tasks
        let auto_fix_1 = AgentTask::new();
        let auto_fix_1 = state.store.create_agent_task(auto_fix_1).await.unwrap();
        let auto_fix_2 = AgentTask::new();
        let auto_fix_2 = state.store.create_agent_task(auto_fix_2).await.unwrap();

        // Create unit task with auto-fix task ids
        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "Test task");
        unit_task.auto_fix_task_ids = vec![auto_fix_1.id, auto_fix_2.id];
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        // Delete the task
        let request = rpc_protocol::requests::DeleteTaskRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = delete_task(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify all resources are cleaned up
        assert!(
            state
                .store
                .get_unit_task(unit_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(agent_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(auto_fix_1.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(auto_fix_2.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_delete_unit_task_with_sessions() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        // Create agent task
        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        // Create agent sessions
        let session1 =
            entities::AgentSession::new(agent_task.id, entities::AiAgentType::ClaudeCode);
        let session1 = state.store.create_agent_session(session1).await.unwrap();
        let session2 =
            entities::AgentSession::new(agent_task.id, entities::AiAgentType::ClaudeCode);
        let session2 = state.store.create_agent_session(session2).await.unwrap();

        // Create unit task
        let unit_task = UnitTask::new(repo_group_id, agent_task.id, "Test task");
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        // Delete the task
        let request = rpc_protocol::requests::DeleteTaskRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = delete_task(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify all resources including sessions are cleaned up
        assert!(
            state
                .store
                .get_unit_task(unit_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(agent_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_session(session1.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_session(session2.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_delete_in_progress_unit_task_cancels_first() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        // Create agent task and in-progress unit task
        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "Running task");
        unit_task.status = EntityUnitTaskStatus::InProgress;
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        // Delete the task (should cancel first, then delete)
        let request = rpc_protocol::requests::DeleteTaskRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = delete_task(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify task is deleted
        assert!(
            state
                .store
                .get_unit_task(unit_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(agent_task.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_delete_composite_task_with_child_nodes() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        // Create planning agent task
        let planning_task = AgentTask::new();
        let planning_task = state.store.create_agent_task(planning_task).await.unwrap();

        // Create composite task
        let composite_task = CompositeTask::new(repo_group_id, planning_task.id, "Multi task");
        let composite_task = state
            .store
            .create_composite_task(composite_task)
            .await
            .unwrap();

        // Create child unit tasks and nodes
        let child_agent_1 = AgentTask::new();
        let child_agent_1 = state.store.create_agent_task(child_agent_1).await.unwrap();
        let child_unit_1 = UnitTask::new(repo_group_id, child_agent_1.id, "Child 1");
        let child_unit_1 = state.store.create_unit_task(child_unit_1).await.unwrap();
        let node_1 = entities::CompositeTaskNode::new(composite_task.id, child_unit_1.id);
        let node_1 = state
            .store
            .create_composite_task_node(node_1)
            .await
            .unwrap();

        let child_agent_2 = AgentTask::new();
        let child_agent_2 = state.store.create_agent_task(child_agent_2).await.unwrap();
        let child_unit_2 = UnitTask::new(repo_group_id, child_agent_2.id, "Child 2");
        let child_unit_2 = state.store.create_unit_task(child_unit_2).await.unwrap();
        let mut node_2 = entities::CompositeTaskNode::new(composite_task.id, child_unit_2.id);
        node_2.depends_on(node_1.id);
        let node_2 = state
            .store
            .create_composite_task_node(node_2)
            .await
            .unwrap();

        // Delete the composite task
        let request = rpc_protocol::requests::DeleteTaskRequest {
            task_id: composite_task.id.to_string(),
        };
        let result = delete_task(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify all resources are cleaned up
        assert!(
            state
                .store
                .get_composite_task(composite_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(planning_task.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_unit_task(child_unit_1.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_unit_task(child_unit_2.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(child_agent_1.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_agent_task(child_agent_2.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_composite_task_node(node_1.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            state
                .store
                .get_composite_task_node(node_2.id)
                .await
                .unwrap()
                .is_none()
        );
        let remaining_nodes = state
            .store
            .list_composite_task_nodes(composite_task.id)
            .await
            .unwrap();
        assert!(remaining_nodes.is_empty());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_task() {
        let (state, _) = create_test_state_with_repo_group().await;

        let request = rpc_protocol::requests::DeleteTaskRequest {
            task_id: Uuid::new_v4().to_string(),
        };
        let result = delete_task(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_task_invalid_id() {
        let (state, _) = create_test_state_with_repo_group().await;

        let request = rpc_protocol::requests::DeleteTaskRequest {
            task_id: "not-a-valid-uuid".to_string(),
        };
        let result = delete_task(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Cancel Task Tests
    // =========================================================================

    #[tokio::test]
    async fn test_cancel_in_progress_task() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "Running task");
        unit_task.status = EntityUnitTaskStatus::InProgress;
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        // Cancel the task (no worker registered, so it should proceed directly)
        let request = CancelTaskRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = cancel_task(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify the task status was updated to Cancelled
        let updated = state
            .store
            .get_unit_task(unit_task.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, EntityUnitTaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_non_in_progress_task_fails() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "Done task");
        unit_task.status = EntityUnitTaskStatus::Done;
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        let request = CancelTaskRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = cancel_task(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_nonexistent_task_fails() {
        let (state, _) = create_test_state_with_repo_group().await;

        let request = CancelTaskRequest {
            task_id: Uuid::new_v4().to_string(),
        };
        let result = cancel_task(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Dismiss Approval Tests
    // =========================================================================

    #[tokio::test]
    async fn test_dismiss_approval_approved_task() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "Approved task");
        unit_task.status = EntityUnitTaskStatus::Approved;
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        let request = DismissApprovalRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = dismiss_approval(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let updated = state
            .store
            .get_unit_task(unit_task.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, EntityUnitTaskStatus::InReview);
    }

    #[tokio::test]
    async fn test_dismiss_approval_non_approved_task_fails() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "In progress task");
        unit_task.status = EntityUnitTaskStatus::InProgress;
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        let request = DismissApprovalRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = dismiss_approval(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Create PR Tests
    // =========================================================================

    #[tokio::test]
    async fn test_create_pr_approved_task() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "Approved task");
        unit_task.status = EntityUnitTaskStatus::Approved;
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        let request = CreatePrRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = create_pr(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        // Verify the task was reset to InProgress for worker pickup
        let updated = state
            .store
            .get_unit_task(unit_task.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.status, EntityUnitTaskStatus::InProgress);
        assert!(updated.prompt.contains("Create a pull request"));
    }

    #[tokio::test]
    async fn test_create_pr_wrong_status_fails() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let mut unit_task = UnitTask::new(repo_group_id, agent_task.id, "In progress task");
        unit_task.status = EntityUnitTaskStatus::InProgress;
        let unit_task = state.store.create_unit_task(unit_task).await.unwrap();

        let request = CreatePrRequest {
            task_id: unit_task.id.to_string(),
        };
        let result = create_pr(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Get Composite Task Nodes Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_composite_task_nodes() {
        let (state, repo_group_id) = create_test_state_with_repo_group().await;

        // Create planning agent task and composite task
        let planning_task = AgentTask::new();
        let planning_task = state.store.create_agent_task(planning_task).await.unwrap();
        let composite_task = CompositeTask::new(repo_group_id, planning_task.id, "Multi task");
        let composite_task = state
            .store
            .create_composite_task(composite_task)
            .await
            .unwrap();

        // Create child unit tasks and nodes
        let child_agent = AgentTask::new();
        let child_agent = state.store.create_agent_task(child_agent).await.unwrap();
        let child_unit = UnitTask::new(repo_group_id, child_agent.id, "Child task");
        let child_unit = state.store.create_unit_task(child_unit).await.unwrap();
        let node = entities::CompositeTaskNode::new(composite_task.id, child_unit.id);
        let _node = state
            .store
            .create_composite_task_node(node)
            .await
            .unwrap();

        // Fetch nodes
        let request = GetCompositeTaskNodesRequest {
            composite_task_id: composite_task.id.to_string(),
        };
        let result = get_composite_task_nodes(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.nodes.len(), 1);
        assert_eq!(response.unit_tasks.len(), 1);
        assert_eq!(response.nodes[0].unit_task_id, child_unit.id.to_string());
        assert_eq!(response.unit_tasks[0].id, child_unit.id.to_string());
    }

    #[tokio::test]
    async fn test_get_composite_task_nodes_nonexistent_fails() {
        let (state, _) = create_test_state_with_repo_group().await;

        let request = GetCompositeTaskNodesRequest {
            composite_task_id: Uuid::new_v4().to_string(),
        };
        let result = get_composite_task_nodes(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    // =========================================================================
    // Get Task Logs Tests
    // =========================================================================

    #[tokio::test]
    async fn test_get_task_logs_empty_sessions() {
        let (state, _) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        let request = GetTaskLogsRequest {
            agent_task_id: agent_task.id.to_string(),
            after_event_id: None,
        };
        let result = get_task_logs(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.sessions.is_empty());
        assert!(response.events.is_empty());
    }

    #[tokio::test]
    async fn test_get_task_logs_with_sessions() {
        let (state, _) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        // Create a session with some output
        let mut session =
            entities::AgentSession::new(agent_task.id, entities::AiAgentType::ClaudeCode);
        session.output_log = Some("line1\nline2\nline3".to_string());
        session.completed_at = Some(chrono::Utc::now());
        let _session = state.store.create_agent_session(session).await.unwrap();

        let request = GetTaskLogsRequest {
            agent_task_id: agent_task.id.to_string(),
            after_event_id: None,
        };
        let result = get_task_logs(State(state.clone()), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.sessions.len(), 1);
        assert!(response.last_event_id.is_some());
    }

    #[tokio::test]
    async fn test_get_task_logs_nonexistent_agent_task_fails() {
        let (state, _) = create_test_state_with_repo_group().await;

        let request = GetTaskLogsRequest {
            agent_task_id: Uuid::new_v4().to_string(),
            after_event_id: None,
        };
        let result = get_task_logs(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_task_logs_invalid_after_event_id() {
        let (state, _) = create_test_state_with_repo_group().await;

        let agent_task = AgentTask::new();
        let agent_task = state.store.create_agent_task(agent_task).await.unwrap();

        // Negative after_event_id
        let request = GetTaskLogsRequest {
            agent_task_id: agent_task.id.to_string(),
            after_event_id: Some(-1),
        };
        let result = get_task_logs(State(state.clone()), Json(request)).await;
        assert!(result.is_err());

        // Exceeds max
        let request = GetTaskLogsRequest {
            agent_task_id: agent_task.id.to_string(),
            after_event_id: Some(MAX_AFTER_EVENT_ID + 1),
        };
        let result = get_task_logs(State(state.clone()), Json(request)).await;
        assert!(result.is_err());
    }
}
