//! Task management API endpoints.

use std::{collections::HashMap, sync::Arc};

use axum::{Json, extract::State};
use entities::{
    AgentTask, CompositeTask, CompositeTaskNode, CompositeTaskStatus as EntityCompositeTaskStatus,
    UnitTask, UnitTaskStatus as EntityUnitTaskStatus,
};
use plan_parser::Plan;
use rpc_protocol::{CompositeTaskStatus, UnitTaskStatus, requests::*, responses::*};
use task_store::{TaskFilter, TaskStore};
use uuid::Uuid;

use crate::{
    error::{ServerError, ServerResult},
    state::AppState,
};

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

        task.status = EntityCompositeTaskStatus::InProgress;
        task.updated_at = chrono::Utc::now();

        // Parse plan_yaml and create CompositeTaskNode + UnitTask records
        if let Some(ref plan_yaml) = task.plan_yaml {
            match Plan::from_yaml(plan_yaml) {
                Ok(plan) => {
                    let agent_type = task.execution_agent_type;
                    let node_ids = create_composite_task_nodes(
                        &state.store,
                        task_id,
                        task.repository_group_id,
                        agent_type,
                        &plan,
                    )
                    .await?;
                    task.node_ids = node_ids;
                    tracing::info!(
                        task_id = %task_id,
                        node_count = plan.tasks.len(),
                        "Created composite task nodes from plan"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        task_id = %task_id,
                        "Failed to parse plan YAML: {}",
                        e
                    );
                    task.status = EntityCompositeTaskStatus::Failed;
                    state.store.update_composite_task(task).await?;
                    return Err(ServerError::Internal(format!(
                        "Failed to parse plan YAML: {}",
                        e
                    )));
                }
            }
        }

        state.store.update_composite_task(task).await?;
        tracing::info!(task_id = %task_id, "CompositeTask approved and execution started");
        return Ok(Json(ApproveTaskResponse {}));
    }

    Err(ServerError::NotFound("Task not found".to_string()))
}

/// Creates CompositeTaskNode and UnitTask records from a parsed plan.
///
/// This is called during composite task approval to set up the task graph
/// for execution. Returns the list of created node IDs.
async fn create_composite_task_nodes<S: TaskStore>(
    store: &S,
    composite_task_id: Uuid,
    repository_group_id: Uuid,
    agent_type: Option<entities::AiAgentType>,
    plan: &Plan,
) -> ServerResult<Vec<Uuid>> {
    let mut plan_id_to_node_id: HashMap<String, Uuid> = HashMap::new();
    let mut node_ids: Vec<Uuid> = Vec::new();

    // First pass: Create AgentTask + UnitTask + CompositeTaskNode for each
    // plan task (without dependencies set yet)
    for plan_task in &plan.tasks {
        // Create agent task
        let mut agent_task = AgentTask::new();
        agent_task.ai_agent_type =
            agent_type.or(Some(entities::AiAgentType::ClaudeCode));
        let agent_task = store.create_agent_task(agent_task).await?;

        // Create unit task
        let mut unit_task =
            UnitTask::new(repository_group_id, agent_task.id, &plan_task.prompt);
        if let Some(ref title) = plan_task.title {
            unit_task = unit_task.with_title(title);
        }
        if let Some(ref branch_name) = plan_task.branch_name {
            unit_task = unit_task.with_branch_name(branch_name);
        }

        let unit_task = store.create_unit_task(unit_task).await?;

        // Create composite task node
        let node = CompositeTaskNode::new(composite_task_id, unit_task.id);
        let node = store.create_composite_task_node(node).await?;

        tracing::info!(
            node_id = %node.id,
            unit_task_id = %unit_task.id,
            plan_task_id = %plan_task.id,
            "Created composite task node"
        );

        plan_id_to_node_id.insert(plan_task.id.clone(), node.id);
        node_ids.push(node.id);
    }

    // Second pass: Set dependencies on nodes
    for plan_task in &plan.tasks {
        if plan_task.depends_on.is_empty() {
            continue;
        }

        let node_id = plan_id_to_node_id[&plan_task.id];
        let mut node = store
            .get_composite_task_node(node_id)
            .await?
            .ok_or_else(|| {
                ServerError::Internal(format!("Composite task node not found: {}", node_id))
            })?;

        for dep_plan_id in &plan_task.depends_on {
            let dep_node_id = plan_id_to_node_id.get(dep_plan_id).ok_or_else(|| {
                ServerError::Internal(format!(
                    "Dependency plan task not found: {}",
                    dep_plan_id
                ))
            })?;
            node.depends_on(dep_node_id.to_owned());
        }

        store.update_composite_task_node(node).await?;
    }

    Ok(node_ids)
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
