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
pub async fn delete_task<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<DeleteTaskRequest>,
) -> ServerResult<Json<DeleteTaskResponse>> {
    let task_id: Uuid = request
        .task_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid task_id".to_string()))?;

    // Try to delete unit task first
    if state.store.get_unit_task(task_id).await?.is_some() {
        state.store.delete_unit_task(task_id).await?;
        tracing::info!(task_id = %task_id, "UnitTask deleted");
        return Ok(Json(DeleteTaskResponse {}));
    }

    // Try composite task
    if state.store.get_composite_task(task_id).await?.is_some() {
        state.store.delete_composite_task(task_id).await?;
        tracing::info!(task_id = %task_id, "CompositeTask deleted");
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
}
