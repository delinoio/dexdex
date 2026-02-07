//! Task-related Tauri commands.

#[cfg(desktop)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(desktop)]
use coding_agents::{NormalizedEvent, TimestampedEvent};
use entities::{
    AgentTask, AiAgentType, CompositeTask, CompositeTaskNode, CompositeTaskStatus, UnitTask,
    UnitTaskStatus,
};
use rpc_protocol::requests;
use serde::{Deserialize, Serialize};
#[cfg(desktop)]
use task_store::{TaskFilter, TaskStore};
use tauri::{Emitter, State};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[cfg(not(desktop))]
use crate::state::ERR_LOCAL_MODE_NOT_SUPPORTED;
use crate::{
    config::AppMode,
    error::{AppError, AppResult},
    events::{event_names, TaskStatusChangedEvent, TaskType},
    remote_client::{
        entity_to_rpc_agent_type, entity_to_rpc_composite_status, entity_to_rpc_unit_status,
        rpc_to_entity_composite_task, rpc_to_entity_unit_task, validate_optional_name,
        validate_text, validate_uuid_string,
    },
    state::AppState,
};

// =============================================================================
// Security Constants
// =============================================================================

/// Maximum allowed prompt length in characters.
/// This prevents memory exhaustion and potential DoS attacks from very large
/// prompts.
const MAX_PROMPT_LENGTH: usize = 100_000;

/// Minimum prompt length to be useful.
const MIN_PROMPT_LENGTH: usize = 1;

/// Maximum allowed title length in characters.
const MAX_TITLE_LENGTH: usize = 500;

/// Minimum time between task creations in milliseconds.
/// This provides basic rate limiting to prevent resource exhaustion.
///
/// # Limitations
/// This rate limiter uses a global atomic variable, which is appropriate for
/// single-user desktop applications. For multi-user scenarios, a per-user
/// rate limiter with a shared backend (e.g., Redis) would be required.
#[cfg(desktop)]
const MIN_TASK_CREATION_INTERVAL_MS: u64 = 500;

/// Global rate limiter for task creation (tracks last creation timestamp in ms
/// since epoch).
///
/// # Note
/// This is a simple global rate limiter suitable for single-user desktop apps.
/// In a multi-user environment, this would need to be replaced with per-user
/// rate limiting using a distributed cache or database.
#[cfg(desktop)]
static LAST_TASK_CREATION_TIME: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// Validation Functions
// =============================================================================

/// Validates a task prompt for security and sanity.
///
/// # Security
/// This prevents:
/// - Memory exhaustion from very large prompts
/// - Empty prompts that waste resources
/// - Prompts with null bytes or other dangerous characters
fn validate_prompt(prompt: &str) -> AppResult<()> {
    // Check minimum length
    if prompt.trim().len() < MIN_PROMPT_LENGTH {
        return Err(AppError::InvalidRequest(
            "Prompt cannot be empty".to_string(),
        ));
    }

    // Check maximum length
    if prompt.len() > MAX_PROMPT_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Prompt exceeds maximum length of {} characters (got {} characters)",
            MAX_PROMPT_LENGTH,
            prompt.len()
        )));
    }

    // Check for null bytes which could cause issues in string handling
    if prompt.contains('\0') {
        return Err(AppError::InvalidRequest(
            "Prompt cannot contain null bytes".to_string(),
        ));
    }

    Ok(())
}

/// Validates a task title.
fn validate_title(title: &str) -> AppResult<()> {
    if title.len() > MAX_TITLE_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Title exceeds maximum length of {} characters",
            MAX_TITLE_LENGTH
        )));
    }

    if title.contains('\0') {
        return Err(AppError::InvalidRequest(
            "Title cannot contain null bytes".to_string(),
        ));
    }

    Ok(())
}

/// Checks rate limiting for task creation.
///
/// # Security
/// This prevents:
/// - Resource exhaustion from rapid task creation
/// - Disk space exhaustion from too many worktrees
/// - CPU exhaustion from running too many agents
///
/// # Thread Safety
/// Uses `compare_exchange` to atomically check and update the timestamp,
/// preventing race conditions where concurrent requests could both pass
/// the rate limit check.
#[cfg(desktop)]
fn check_rate_limit() -> AppResult<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    loop {
        let last = LAST_TASK_CREATION_TIME.load(Ordering::SeqCst);

        if now.saturating_sub(last) < MIN_TASK_CREATION_INTERVAL_MS {
            return Err(AppError::RateLimitExceeded(format!(
                "Please wait at least {} ms between task creations",
                MIN_TASK_CREATION_INTERVAL_MS
            )));
        }

        // Atomic check-and-set to prevent race condition where multiple
        // concurrent requests pass the rate limit check before any updates
        // the timestamp.
        match LAST_TASK_CREATION_TIME.compare_exchange(
            last,
            now,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => return Ok(()),
            Err(_) => {
                // Another thread updated the timestamp, retry the check
                continue;
            }
        }
    }
}

/// Parameters for creating a unit task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUnitTaskParams {
    pub repository_group_id: String,
    pub prompt: String,
    pub title: Option<String>,
    pub branch_name: Option<String>,
    pub ai_agent_type: Option<String>,
    pub ai_agent_model: Option<String>,
}

/// Parameters for creating a composite task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompositeTaskParams {
    pub repository_group_id: String,
    pub prompt: String,
    pub title: Option<String>,
    pub execution_agent_type: Option<String>,
    pub planning_agent_type: Option<String>,
}

/// Parameters for listing tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTasksParams {
    pub repository_group_id: Option<String>,
    pub unit_status: Option<String>,
    pub composite_status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response for get_task command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_task: Option<UnitTask>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite_task: Option<CompositeTask>,
}

/// Response for list_tasks command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTasksResult {
    pub unit_tasks: Vec<UnitTask>,
    pub composite_tasks: Vec<CompositeTask>,
    pub total_count: i32,
}

/// A composite task node with its associated unit task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeTaskNodeWithUnitTask {
    pub node: CompositeTaskNode,
    pub unit_task: UnitTask,
}

/// Response for get_composite_task_nodes command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeTaskNodesResult {
    pub nodes: Vec<CompositeTaskNodeWithUnitTask>,
}

/// A normalized event entry with metadata.
#[cfg(desktop)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedEventEntry {
    pub id: i64,
    pub timestamp: String,
    pub event: NormalizedEvent,
}

/// A group of log events belonging to a single agent session.
#[cfg(desktop)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLogsGroup {
    pub session_id: String,
    /// Human-readable label, e.g. "Main Execution" or "Create PR".
    pub label: String,
    pub events: Vec<NormalizedEventEntry>,
    pub is_complete: bool,
    pub created_at: String,
}

/// Response for get_task_logs command.
#[cfg(desktop)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLogsResponse {
    pub events: Vec<NormalizedEventEntry>,
    pub is_complete: bool,
    pub last_event_id: Option<i64>,
    /// All sessions for this agent task, each with their own events.
    /// The first session is the main execution; subsequent sessions are
    /// subtasks.
    pub sessions: Vec<SessionLogsGroup>,
}

/// Response for get_task_logs command (mobile stub).
#[cfg(not(desktop))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLogsResponse {
    pub events: Vec<serde_json::Value>,
    pub is_complete: bool,
    pub last_event_id: Option<i64>,
    pub sessions: Vec<serde_json::Value>,
}

/// Parameters for responding to a TTY input request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespondTtyInputParams {
    pub request_id: String,
    pub response: String,
}

/// Creates a new unit task.
///
/// # Security
/// This command includes:
/// - Rate limiting to prevent resource exhaustion
/// - Prompt validation to prevent oversized inputs
/// - Title validation for sanity
#[cfg(desktop)]
#[tauri::command]
pub async fn create_unit_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateUnitTaskParams,
) -> AppResult<UnitTask> {
    // SECURITY: Check rate limit before processing
    check_rate_limit()?;

    // SECURITY: Validate prompt to prevent oversized inputs
    validate_prompt(&params.prompt)?;

    // SECURITY: Validate title if provided
    if let Some(ref title) = params.title {
        validate_title(title)?;
    }

    let state = state.read().await;

    // Validate input parameters
    validate_uuid_string(&params.repository_group_id, "repository group ID")?;
    validate_text(&params.prompt, "prompt")?;
    validate_optional_name(params.title.as_deref(), "title")?;
    validate_optional_name(params.branch_name.as_deref(), "branch name")?;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let agent_type = params
            .ai_agent_type
            .as_deref()
            .map(parse_agent_type)
            .transpose()?
            .map(entity_to_rpc_agent_type);

        let request = requests::CreateUnitTaskRequest {
            repository_group_id: params.repository_group_id,
            prompt: params.prompt,
            title: params.title,
            branch_name: params.branch_name,
            ai_agent_type: agent_type,
            ai_agent_model: params.ai_agent_model,
        };

        let response = client.create_unit_task(request).await?;
        let task = rpc_to_entity_unit_task(response.task)?;
        info!("Created unit task via remote: {}", task.id);
        return Ok(task);
    }

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let repo_group_id = Uuid::parse_str(&params.repository_group_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid repository group ID: {}", e)))?;

    let agent_type = params
        .ai_agent_type
        .as_deref()
        .map(parse_agent_type)
        .transpose()?
        .unwrap_or(AiAgentType::ClaudeCode);

    // Create an AgentTask first
    let mut agent_task = AgentTask::new();
    agent_task.ai_agent_type = Some(agent_type);
    agent_task.ai_agent_model.clone_from(&params.ai_agent_model);
    let agent_task = runtime
        .task_store_arc()
        .create_agent_task(agent_task)
        .await?;

    // Create the UnitTask
    let mut task = UnitTask::new(repo_group_id, agent_task.id, &params.prompt);
    if let Some(title) = params.title {
        task = task.with_title(title);
    }
    if let Some(branch_name) = params.branch_name {
        task = task.with_branch_name(branch_name);
    }

    let created = runtime.task_store_arc().create_unit_task(task).await?;
    info!("Created unit task: {}", created.id);

    // Trigger task execution if executor is initialized
    if let Some(executor) = runtime.executor().await {
        let task_id = created.id;
        tokio::spawn(async move {
            if let Err(e) = executor.execute_unit_task(task_id).await {
                tracing::error!("Failed to start task execution for {}: {}", task_id, e);
            }
        });
    } else {
        tracing::warn!(
            "Executor not initialized, task {} will not be executed",
            created.id
        );
    }

    Ok(created)
}

/// Creates a new unit task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn create_unit_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateUnitTaskParams,
) -> AppResult<UnitTask> {
    let state = state.read().await;

    // Validate input parameters
    validate_uuid_string(&params.repository_group_id, "repository group ID")?;
    validate_text(&params.prompt, "prompt")?;
    validate_optional_name(params.title.as_deref(), "title")?;
    validate_optional_name(params.branch_name.as_deref(), "branch name")?;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let agent_type = params
            .ai_agent_type
            .as_deref()
            .map(parse_agent_type)
            .transpose()?
            .map(entity_to_rpc_agent_type);

        let request = requests::CreateUnitTaskRequest {
            repository_group_id: params.repository_group_id,
            prompt: params.prompt,
            title: params.title,
            branch_name: params.branch_name,
            ai_agent_type: agent_type,
            ai_agent_model: params.ai_agent_model,
        };

        let response = client.create_unit_task(request).await?;
        let task = rpc_to_entity_unit_task(response.task)?;
        info!("Created unit task via remote: {}", task.id);
        return Ok(task);
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Creates a new composite task.
///
/// # Security
/// This command includes:
/// - Rate limiting to prevent resource exhaustion
/// - Prompt validation to prevent oversized inputs
/// - Title validation for sanity
#[cfg(desktop)]
#[tauri::command]
pub async fn create_composite_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateCompositeTaskParams,
) -> AppResult<CompositeTask> {
    // SECURITY: Check rate limit before processing
    check_rate_limit()?;

    // SECURITY: Validate prompt to prevent oversized inputs
    validate_prompt(&params.prompt)?;

    // SECURITY: Validate title if provided
    if let Some(ref title) = params.title {
        validate_title(title)?;
    }

    let state = state.read().await;

    // Validate input parameters
    validate_uuid_string(&params.repository_group_id, "repository group ID")?;
    validate_text(&params.prompt, "prompt")?;
    validate_optional_name(params.title.as_deref(), "title")?;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let execution_agent_type = params
            .execution_agent_type
            .as_deref()
            .map(parse_agent_type)
            .transpose()?
            .map(entity_to_rpc_agent_type);

        let planning_agent_type = params
            .planning_agent_type
            .as_deref()
            .map(parse_agent_type)
            .transpose()?
            .map(entity_to_rpc_agent_type);

        let request = requests::CreateCompositeTaskRequest {
            repository_group_id: params.repository_group_id,
            prompt: params.prompt,
            title: params.title,
            execution_agent_type,
            planning_agent_type,
        };

        let response = client.create_composite_task(request).await?;
        let task = rpc_to_entity_composite_task(response.task)?;
        info!("Created composite task via remote: {}", task.id);
        return Ok(task);
    }

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let repo_group_id = Uuid::parse_str(&params.repository_group_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid repository group ID: {}", e)))?;

    let execution_agent_type = params
        .execution_agent_type
        .as_deref()
        .map(parse_agent_type)
        .transpose()?;

    let planning_agent_type = params
        .planning_agent_type
        .as_deref()
        .map(parse_agent_type)
        .transpose()?
        .unwrap_or(AiAgentType::ClaudeCode);

    // Create a planning AgentTask
    let mut planning_task = AgentTask::new();
    planning_task.ai_agent_type = Some(planning_agent_type);
    let planning_task = runtime
        .task_store_arc()
        .create_agent_task(planning_task)
        .await?;

    // Create the CompositeTask
    let mut task = CompositeTask::new(repo_group_id, planning_task.id, &params.prompt);
    if let Some(title) = params.title {
        task = task.with_title(title);
    }
    if let Some(agent_type) = execution_agent_type {
        task = task.with_execution_agent_type(agent_type);
    }

    let created = runtime.task_store_arc().create_composite_task(task).await?;
    info!("Created composite task: {}", created.id);

    // Trigger planning task execution if executor is initialized
    if let Some(executor) = runtime.executor().await {
        let composite_task_id = created.id;
        tokio::spawn(async move {
            if let Err(e) = executor.execute_composite_task(composite_task_id).await {
                tracing::error!(
                    "Failed to start planning execution for composite task {}: {}",
                    composite_task_id,
                    e
                );
            }
        });
    } else {
        tracing::warn!(
            "Executor not initialized, composite task {} planning will not be executed",
            created.id
        );
    }

    Ok(created)
}

/// Creates a new composite task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn create_composite_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateCompositeTaskParams,
) -> AppResult<CompositeTask> {
    let state = state.read().await;

    // Validate input parameters
    validate_uuid_string(&params.repository_group_id, "repository group ID")?;
    validate_text(&params.prompt, "prompt")?;
    validate_optional_name(params.title.as_deref(), "title")?;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let execution_agent_type = params
            .execution_agent_type
            .as_deref()
            .map(parse_agent_type)
            .transpose()?
            .map(entity_to_rpc_agent_type);

        let planning_agent_type = params
            .planning_agent_type
            .as_deref()
            .map(parse_agent_type)
            .transpose()?
            .map(entity_to_rpc_agent_type);

        let request = requests::CreateCompositeTaskRequest {
            repository_group_id: params.repository_group_id,
            prompt: params.prompt,
            title: params.title,
            execution_agent_type,
            planning_agent_type,
        };

        let response = client.create_composite_task(request).await?;
        let task = rpc_to_entity_composite_task(response.task)?;
        info!("Created composite task via remote: {}", task.id);
        return Ok(task);
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Gets a task by ID.
#[cfg(desktop)]
#[tauri::command]
pub async fn get_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<TaskResponse> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::GetTaskRequest {
            task_id: task_id.clone(),
        };

        let response = client.get_task(request).await?;
        return match response {
            rpc_protocol::responses::GetTaskResponse::UnitTask { unit_task } => Ok(TaskResponse {
                unit_task: Some(rpc_to_entity_unit_task(unit_task)?),
                composite_task: None,
            }),
            rpc_protocol::responses::GetTaskResponse::CompositeTask { composite_task } => {
                Ok(TaskResponse {
                    unit_task: None,
                    composite_task: Some(rpc_to_entity_composite_task(composite_task)?),
                })
            }
        };
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Try to find as unit task first
    if let Some(unit_task) = runtime.task_store_arc().get_unit_task(id).await? {
        return Ok(TaskResponse {
            unit_task: Some(unit_task),
            composite_task: None,
        });
    }

    // Try as composite task
    if let Some(composite_task) = runtime.task_store_arc().get_composite_task(id).await? {
        return Ok(TaskResponse {
            unit_task: None,
            composite_task: Some(composite_task),
        });
    }

    Err(AppError::NotFound(format!("Task not found: {}", task_id)))
}

/// Gets a task by ID (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn get_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<TaskResponse> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::GetTaskRequest {
            task_id: task_id.clone(),
        };

        let response = client.get_task(request).await?;
        return match response {
            rpc_protocol::responses::GetTaskResponse::UnitTask { unit_task } => Ok(TaskResponse {
                unit_task: Some(rpc_to_entity_unit_task(unit_task)?),
                composite_task: None,
            }),
            rpc_protocol::responses::GetTaskResponse::CompositeTask { composite_task } => {
                Ok(TaskResponse {
                    unit_task: None,
                    composite_task: Some(rpc_to_entity_composite_task(composite_task)?),
                })
            }
        };
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Lists tasks with optional filters.
#[cfg(desktop)]
#[tauri::command]
pub async fn list_tasks(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListTasksParams,
) -> AppResult<ListTasksResult> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let unit_status = params
            .unit_status
            .as_deref()
            .and_then(|s| parse_unit_status(s).ok())
            .map(entity_to_rpc_unit_status);

        let composite_status = params
            .composite_status
            .as_deref()
            .and_then(|s| parse_composite_status(s).ok())
            .map(entity_to_rpc_composite_status);

        let request = requests::ListTasksRequest {
            repository_group_id: params.repository_group_id,
            unit_status,
            composite_status,
            limit: params.limit.unwrap_or(100),
            offset: params.offset.unwrap_or(0),
        };

        let response = client.list_tasks(request).await?;
        let unit_tasks: AppResult<Vec<_>> = response
            .unit_tasks
            .into_iter()
            .map(rpc_to_entity_unit_task)
            .collect();
        let composite_tasks: AppResult<Vec<_>> = response
            .composite_tasks
            .into_iter()
            .map(rpc_to_entity_composite_task)
            .collect();
        return Ok(ListTasksResult {
            unit_tasks: unit_tasks?,
            composite_tasks: composite_tasks?,
            total_count: response.total_count,
        });
    }

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let filter = TaskFilter {
        repository_group_id: params
            .repository_group_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok()),
        unit_status: params
            .unit_status
            .as_deref()
            .and_then(|s| parse_unit_status(s).ok()),
        composite_status: params
            .composite_status
            .as_deref()
            .and_then(|s| parse_composite_status(s).ok()),
        limit: params.limit.map(|l| l as u32),
        offset: params.offset.map(|o| o as u32),
    };

    let (unit_tasks, unit_count) = runtime
        .task_store_arc()
        .list_unit_tasks(filter.clone())
        .await?;
    let (composite_tasks, composite_count) = runtime
        .task_store_arc()
        .list_composite_tasks(filter)
        .await?;

    Ok(ListTasksResult {
        unit_tasks,
        composite_tasks,
        total_count: (unit_count + composite_count) as i32,
    })
}

/// Lists tasks with optional filters (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn list_tasks(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListTasksParams,
) -> AppResult<ListTasksResult> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let unit_status = params
            .unit_status
            .as_deref()
            .and_then(|s| parse_unit_status(s).ok())
            .map(entity_to_rpc_unit_status);

        let composite_status = params
            .composite_status
            .as_deref()
            .and_then(|s| parse_composite_status(s).ok())
            .map(entity_to_rpc_composite_status);

        let request = requests::ListTasksRequest {
            repository_group_id: params.repository_group_id,
            unit_status,
            composite_status,
            limit: params.limit.unwrap_or(100),
            offset: params.offset.unwrap_or(0),
        };

        let response = client.list_tasks(request).await?;
        let unit_tasks: AppResult<Vec<_>> = response
            .unit_tasks
            .into_iter()
            .map(rpc_to_entity_unit_task)
            .collect();
        let composite_tasks: AppResult<Vec<_>> = response
            .composite_tasks
            .into_iter()
            .map(rpc_to_entity_composite_task)
            .collect();
        return Ok(ListTasksResult {
            unit_tasks: unit_tasks?,
            composite_tasks: composite_tasks?,
            total_count: response.total_count,
        });
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Approves a task.
#[cfg(desktop)]
#[tauri::command]
pub async fn approve_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    app_handle: tauri::AppHandle,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::ApproveTaskRequest {
            task_id: task_id.clone(),
        };

        client.approve_task(request).await?;
        info!("Approved task via remote: {}", task_id);
        return Ok(());
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Try to find and update unit task
    if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
        let old_status = format!("{:?}", task.status).to_lowercase();
        task.status = UnitTaskStatus::Approved;
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_unit_task(task).await?;
        info!("Approved unit task: {}", id);

        // Emit task-status-changed so the frontend auto-updates
        let _ = app_handle.emit(
            event_names::TASK_STATUS_CHANGED,
            &TaskStatusChangedEvent {
                task_id: task_id.clone(),
                task_type: TaskType::UnitTask,
                old_status,
                new_status: "approved".to_string(),
            },
        );

        return Ok(());
    }

    // Try composite task
    if let Some(mut task) = runtime.task_store_arc().get_composite_task(id).await? {
        if task.status == CompositeTaskStatus::PendingApproval {
            task.status = CompositeTaskStatus::InProgress;
            task.updated_at = chrono::Utc::now();
            runtime.task_store_arc().update_composite_task(task).await?;
            info!("Approved composite task: {}", id);

            // Emit task-status-changed so the frontend auto-updates immediately
            let _ = app_handle.emit(
                event_names::TASK_STATUS_CHANGED,
                &TaskStatusChangedEvent {
                    task_id: task_id.clone(),
                    task_type: TaskType::CompositeTask,
                    old_status: "pending_approval".to_string(),
                    new_status: "in_progress".to_string(),
                },
            );

            // Trigger composite task graph execution: parse plan_yaml, create
            // nodes and unit tasks, and start executing root tasks.
            if let Some(executor) = runtime.executor().await {
                let composite_task_id = id;
                tokio::spawn(async move {
                    if let Err(e) = executor
                        .execute_composite_task_graph(composite_task_id)
                        .await
                    {
                        tracing::error!(
                            "Failed to start composite task graph execution for {}: {}",
                            composite_task_id,
                            e
                        );
                    }
                });
            } else {
                tracing::warn!(
                    "Executor not initialized, composite task {} graph will not be executed",
                    id
                );
            }
        } else {
            let old_status = serde_json::to_string(&task.status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            task.status = CompositeTaskStatus::Done;
            task.updated_at = chrono::Utc::now();
            runtime.task_store_arc().update_composite_task(task).await?;
            info!("Approved composite task (marked done): {}", id);

            // Emit task-status-changed so the frontend auto-updates
            let _ = app_handle.emit(
                event_names::TASK_STATUS_CHANGED,
                &TaskStatusChangedEvent {
                    task_id: task_id.clone(),
                    task_type: TaskType::CompositeTask,
                    old_status,
                    new_status: "done".to_string(),
                },
            );
        }
        return Ok(());
    }

    Err(AppError::NotFound(format!("Task not found: {}", task_id)))
}

/// Approves a task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn approve_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::ApproveTaskRequest {
            task_id: task_id.clone(),
        };

        client.approve_task(request).await?;
        info!("Approved task via remote: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Rejects a task.
///
/// Note: The `reason` parameter is accepted for API completeness but is not
/// currently persisted. This will be implemented when the entity schema
/// supports rejection reasons.
#[cfg(desktop)]
#[tauri::command]
pub async fn reject_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    reason: Option<String>,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::RejectTaskRequest {
            task_id: task_id.clone(),
            reason,
        };

        client.reject_task(request).await?;
        info!("Rejected task via remote: {}", task_id);
        return Ok(());
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Try to find and update unit task
    if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
        task.status = UnitTaskStatus::Rejected;
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_unit_task(task).await?;
        info!("Rejected unit task: {}", id);
        return Ok(());
    }

    // Try composite task
    if let Some(mut task) = runtime.task_store_arc().get_composite_task(id).await? {
        task.status = CompositeTaskStatus::Rejected;
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_composite_task(task).await?;
        info!("Rejected composite task: {}", id);
        return Ok(());
    }

    Err(AppError::NotFound(format!("Task not found: {}", task_id)))
}

/// Rejects a task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn reject_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    reason: Option<String>,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::RejectTaskRequest {
            task_id: task_id.clone(),
            reason,
        };

        client.reject_task(request).await?;
        info!("Rejected task via remote: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Updates the plan for a composite task by re-running the planning agent
/// with additional user feedback appended to the original prompt.
///
/// The composite task must be in `PendingApproval` or `Failed` status.
/// This resets the task to `Planning` status, creates a new planning agent
/// task and session, and triggers re-planning.
#[cfg(desktop)]
#[tauri::command]
pub async fn update_plan_with_prompt(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    prompt: String,
) -> AppResult<()> {
    // Validate input parameters
    validate_uuid_string(&task_id, "task ID")?;
    validate_prompt(&prompt)?;

    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::UpdatePlanRequest {
            task_id: task_id.clone(),
            prompt: prompt.clone(),
        };

        client.update_plan(request).await?;
        info!("Updated plan via remote for task: {}", task_id);
        return Ok(());
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Get the composite task
    let mut composite_task = runtime
        .task_store_arc()
        .get_composite_task(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Composite task not found: {}", task_id)))?;

    // Verify the task is in a state that allows re-planning
    let previous_status = composite_task.status;
    if previous_status != CompositeTaskStatus::PendingApproval
        && previous_status != CompositeTaskStatus::Failed
    {
        return Err(AppError::InvalidRequest(format!(
            "Cannot update plan: task is in {} status (must be PendingApproval or Failed)",
            previous_status
        )));
    }

    // Sanitize and validate the feedback prompt
    let sanitized_prompt = entities::sanitize_user_input(&prompt);
    if sanitized_prompt.len() > entities::MAX_FEEDBACK_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Feedback exceeds maximum length of {} characters",
            entities::MAX_FEEDBACK_LENGTH
        )));
    }

    // Save updated_at for optimistic concurrency check
    let expected_updated_at = composite_task.updated_at;

    // Re-fetch the task to check for concurrent modifications (optimistic lock)
    // This check is done BEFORE creating the agent task or mutating the
    // composite task to avoid side effects if a conflict is detected.
    let current_task = runtime
        .task_store_arc()
        .get_composite_task(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Composite task not found: {}", task_id)))?;
    if current_task.updated_at != expected_updated_at {
        return Err(AppError::InvalidRequest(
            "Task was modified concurrently. Please try again.".to_string(),
        ));
    }

    // Store the feedback for re-planning. The executor will use the existing
    // plan_yaml together with this feedback (instead of the original prompt)
    // to generate a new plan.
    composite_task.update_plan_feedback = Some(sanitized_prompt);

    // Create a new planning agent task
    let mut planning_task = AgentTask::new();
    planning_task.ai_agent_type = Some(AiAgentType::ClaudeCode);
    let planning_task = runtime
        .task_store_arc()
        .create_agent_task(planning_task)
        .await?;

    // Update the composite task with the new planning task and reset status
    composite_task.planning_task_id = planning_task.id;
    composite_task.status = CompositeTaskStatus::Planning;
    composite_task.updated_at = chrono::Utc::now();
    runtime
        .task_store_arc()
        .update_composite_task(composite_task)
        .await?;

    info!(
        task_id = %id,
        prompt_length = prompt.len(),
        previous_status = %previous_status,
        "Updated composite task for re-planning with new prompt"
    );

    // Trigger re-planning if executor is initialized
    if let Some(executor) = runtime.executor().await {
        let composite_task_id = id;
        tokio::spawn(async move {
            if let Err(e) = executor.execute_composite_task(composite_task_id).await {
                tracing::error!(
                    "Failed to start re-planning execution for composite task {}: {}",
                    composite_task_id,
                    e
                );
            }
        });
    } else {
        tracing::warn!(
            "Executor not initialized, composite task {} re-planning will not be executed",
            id
        );
    }

    Ok(())
}

/// Updates the plan for a composite task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn update_plan_with_prompt(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    prompt: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let request = requests::UpdatePlanRequest {
            task_id: task_id.clone(),
            prompt: prompt.clone(),
        };

        client.update_plan(request).await?;
        info!("Updated plan via remote for task: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Requests changes for a task.
///
/// In local mode, this creates a subtask that applies the requested changes
/// using the AI agent. The feedback (including any inline review comments)
/// is passed as the prompt. On completion the task returns to InReview.
#[cfg(desktop)]
#[tauri::command]
pub async fn request_changes(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    feedback: String,
) -> AppResult<()> {
    let state = state.read().await;

    // Validate input parameters
    validate_uuid_string(&task_id, "task ID")?;
    validate_text(&feedback, "feedback")?;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::RequestChangesRequest {
            task_id: task_id.clone(),
            feedback: feedback.clone(),
        };

        client.request_changes(request).await?;
        info!("Requested changes via remote for task: {}", task_id);
        return Ok(());
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Validate executor exists before transitioning state to avoid leaving
    // the task stuck in Approved if the executor is unavailable.
    let executor = runtime
        .executor()
        .await
        .ok_or_else(|| AppError::Internal("Executor not initialized".to_string()))?;

    // Verify the unit task exists and is in a reviewable state.
    // We need to first approve the task so execute_subtask can run
    // (it requires Approved status).
    if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
        if task.status != UnitTaskStatus::InReview {
            return Err(AppError::InvalidRequest(format!(
                "Task {} is not in InReview status (current: {})",
                task_id,
                format!("{:?}", task.status)
            )));
        }

        // Transition to Approved so execute_subtask can pick it up
        task.status = UnitTaskStatus::Approved;
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_unit_task(task).await?;
    } else {
        return Err(AppError::NotFound(format!("Task not found: {}", task_id)));
    }

    let prompt = format!(
        "The reviewer has requested changes to your work. Please apply the following feedback and \
         fix any issues mentioned. After applying all changes, make sure the code compiles and \
         works correctly.\n\n--- Requested Changes ---\n{}",
        feedback
    );

    if let Err(e) = executor
        .execute_subtask(id, prompt, UnitTaskStatus::InReview)
        .await
    {
        // Revert task status back to InReview on failure so the user can retry
        if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
            task.status = UnitTaskStatus::InReview;
            task.updated_at = chrono::Utc::now();
            runtime.task_store_arc().update_unit_task(task).await?;
        }
        return Err(AppError::Internal(format!(
            "Failed to start request-changes subtask: {}",
            e
        )));
    }

    info!(
        "Started request-changes subtask for unit task: {} (feedback length: {})",
        id,
        feedback.len()
    );
    Ok(())
}

/// Requests changes for a task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn request_changes(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    feedback: String,
) -> AppResult<()> {
    let state = state.read().await;

    // Validate input parameters
    validate_uuid_string(&task_id, "task ID")?;
    validate_text(&feedback, "feedback")?;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::RequestChangesRequest {
            task_id: task_id.clone(),
            feedback: feedback.clone(),
        };

        client.request_changes(request).await?;
        info!("Requested changes via remote for task: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Gets logs for an agent task.
///
/// Returns normalized events from the agent session output.
/// Only accepts agent task IDs directly.
#[cfg(desktop)]
#[tauri::command]
pub async fn get_task_logs(
    state: State<'_, Arc<RwLock<AppState>>>,
    agent_task_id: String,
    after_event_id: Option<i64>,
) -> AppResult<TaskLogsResponse> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // In remote mode on desktop, we currently return minimal data
        // Full log streaming support requires additional server-side work
        // For now, return an empty response indicating task is complete
        // TODO: Implement proper remote log streaming (agent_task_id and after_event_id
        // are unused here)
        return Ok(TaskLogsResponse {
            events: Vec::new(),
            is_complete: true,
            last_event_id: None,
            sessions: Vec::new(),
        });
    }

    let agent_task_id = Uuid::parse_str(&agent_task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid agent task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Verify the agent task exists
    if runtime
        .task_store_arc()
        .get_agent_task(agent_task_id)
        .await?
        .is_none()
    {
        return Err(AppError::NotFound(format!(
            "Agent task not found: {}",
            agent_task_id
        )));
    }

    // Get the sessions for the agent task
    let mut sessions = runtime
        .task_store_arc()
        .list_agent_sessions(agent_task_id)
        .await?;

    // Sort sessions by created_at to ensure chronological order
    sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // If no sessions, return empty (task hasn't started yet)
    if sessions.is_empty() {
        return Ok(TaskLogsResponse {
            events: Vec::new(),
            is_complete: false,
            last_event_id: None,
            sessions: Vec::new(),
        });
    }

    // Build grouped session logs for all sessions
    let mut session_groups: Vec<SessionLogsGroup> = Vec::new();
    for (session_idx, session) in sessions.iter().enumerate() {
        let mut session_events = Vec::new();

        if let Some(output_log) = &session.output_log {
            for (idx, line) in output_log.lines().enumerate() {
                let event_id = idx as i64;

                if let Ok(timestamped) = serde_json::from_str::<TimestampedEvent>(line) {
                    session_events.push(NormalizedEventEntry {
                        id: event_id,
                        timestamp: timestamped.timestamp.to_rfc3339(),
                        event: timestamped.event,
                    });
                } else if let Ok(event) = serde_json::from_str::<NormalizedEvent>(line) {
                    session_events.push(NormalizedEventEntry {
                        id: event_id,
                        timestamp: session.created_at.to_rfc3339(),
                        event,
                    });
                }
            }
        }

        // First session is "Main Execution", subsequent are subtask sessions
        let label = if session_idx == 0 {
            "Main Execution".to_string()
        } else {
            format!("Subtask {}", session_idx)
        };

        session_groups.push(SessionLogsGroup {
            session_id: session.id.to_string(),
            label,
            events: session_events,
            is_complete: session.completed_at.is_some(),
            created_at: session.created_at.to_rfc3339(),
        });
    }

    // For backward compatibility, the top-level `events` field still contains
    // the latest session's events (used by the real-time streaming path).
    let latest_session = sessions.last().unwrap();
    let is_complete = latest_session.completed_at.is_some();
    let mut events = Vec::new();
    let mut last_event_id: Option<i64> = None;

    if let Some(output_log) = &latest_session.output_log {
        for (idx, line) in output_log.lines().enumerate() {
            let event_id = idx as i64;

            if let Some(after_id) = after_event_id {
                if event_id <= after_id {
                    continue;
                }
            }

            if let Ok(timestamped) = serde_json::from_str::<TimestampedEvent>(line) {
                events.push(NormalizedEventEntry {
                    id: event_id,
                    timestamp: timestamped.timestamp.to_rfc3339(),
                    event: timestamped.event,
                });
                last_event_id = Some(event_id);
            } else if let Ok(event) = serde_json::from_str::<NormalizedEvent>(line) {
                events.push(NormalizedEventEntry {
                    id: event_id,
                    timestamp: latest_session.created_at.to_rfc3339(),
                    event,
                });
                last_event_id = Some(event_id);
            }
        }
    }

    Ok(TaskLogsResponse {
        events,
        is_complete,
        last_event_id,
        sessions: session_groups,
    })
}

/// Gets logs for an agent task (mobile - remote mode only).
///
/// Note: In remote mode, we fetch the session log from the server. The log
/// format may differ from local mode as we receive it as a single string
/// rather than parsed events.
#[cfg(not(desktop))]
#[tauri::command]
#[allow(unused_variables)]
pub async fn get_task_logs(
    state: State<'_, Arc<RwLock<AppState>>>,
    agent_task_id: String,
    after_event_id: Option<i64>,
) -> AppResult<TaskLogsResponse> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // In remote mode for mobile, we currently return minimal data
        // Full log streaming support requires additional server-side work
        // For now, return an empty response indicating task is complete
        // TODO: Implement proper remote log streaming
        return Ok(TaskLogsResponse {
            events: Vec::new(),
            is_complete: true,
            last_event_id: None,
            sessions: Vec::new(),
        });
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Cancels a running task.
#[cfg(desktop)]
#[tauri::command]
pub async fn cancel_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Get the executor and cancel the execution
    let executor = runtime
        .executor()
        .await
        .ok_or_else(|| AppError::Internal("Executor not initialized".to_string()))?;

    let was_cancelled = executor.cancel_execution(id).await;

    if was_cancelled {
        // Update task status to cancelled
        if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
            // Only update status if the task is still in progress
            // This prevents race conditions where the task completes just before
            // cancellation
            if task.status == UnitTaskStatus::InProgress {
                task.status = UnitTaskStatus::Cancelled;
                task.updated_at = chrono::Utc::now();
                if let Err(e) = runtime.task_store_arc().update_unit_task(task).await {
                    tracing::error!("Failed to update task status after cancellation: {}", e);
                    return Err(e.into());
                }
            } else {
                tracing::warn!(
                    "Task {} was cancelled but status was already {:?}, not updating",
                    id,
                    task.status
                );
            }
        } else {
            tracing::warn!("Task {} was cancelled but not found in store", id);
        }
        info!("Cancelled task: {}", id);
    } else {
        info!("Task {} was not running or already completed", id);
    }

    Ok(())
}

/// Cancels a running task (mobile stub - local mode not supported).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn cancel_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    _task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "Remote mode not yet implemented".to_string(),
        ));
    }

    Err(AppError::InvalidRequest(
        "Local mode is not supported on this platform".to_string(),
    ))
}

/// Responds to a TTY input request from an agent.
#[cfg(desktop)]
#[tauri::command]
pub async fn respond_tty_input(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: RespondTtyInputParams,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::SubmitTtyInputRequest {
            request_id: params.request_id.clone(),
            response: params.response,
        };

        client.submit_tty_input(request).await?;
        info!("Responded to TTY input via remote: {}", params.request_id);
        return Ok(());
    }

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let request_id = Uuid::parse_str(&params.request_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid request ID: {}", e)))?;

    // Get the TTY request manager
    let tty_manager = runtime
        .tty_request_manager()
        .await
        .ok_or_else(|| AppError::Internal("Executor not initialized".to_string()))?;

    // Respond to the request
    let delivered = tty_manager.respond(request_id, params.response).await;

    if !delivered {
        return Err(AppError::NotFound(format!(
            "TTY request not found or already responded: {}",
            params.request_id
        )));
    }

    info!("Responded to TTY input request: {}", params.request_id);
    Ok(())
}

/// Responds to a TTY input request from an agent (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn respond_tty_input(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: RespondTtyInputParams,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let client = state.get_remote_client()?;

        let request = requests::SubmitTtyInputRequest {
            request_id: params.request_id.clone(),
            response: params.response,
        };

        client.submit_tty_input(request).await?;
        info!("Responded to TTY input via remote: {}", params.request_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Gets all nodes for a composite task with their associated unit tasks.
///
/// # Note
/// Remote mode support for task graph visualization requires additional
/// server-side API endpoints. For now, in remote mode we return an empty
/// result. The frontend gracefully handles this.
#[cfg(desktop)]
#[tauri::command]
pub async fn get_composite_task_nodes(
    state: State<'_, Arc<RwLock<AppState>>>,
    composite_task_id: String,
) -> AppResult<CompositeTaskNodesResult> {
    let state = state.read().await;

    // In remote mode, return empty result for now as the server doesn't yet
    // have an endpoint for composite task nodes
    if state.mode == AppMode::Remote {
        // TODO: Implement remote API call when server supports composite task nodes
        // endpoint (composite_task_id is unused in remote mode until the API is
        // implemented)
        return Ok(CompositeTaskNodesResult { nodes: Vec::new() });
    }

    let id = Uuid::parse_str(&composite_task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid composite task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let nodes = runtime
        .task_store_arc()
        .list_composite_task_nodes(id)
        .await?;

    // TODO(performance): Consider adding a bulk fetch method
    // `get_unit_tasks_by_ids(Vec<Uuid>)` to the TaskStore trait to avoid N+1
    // queries for large graphs. For now, this is acceptable for typical graph
    // sizes (< 50 nodes).
    let mut result = Vec::with_capacity(nodes.len());
    for node in nodes {
        if let Some(unit_task) = runtime
            .task_store_arc()
            .get_unit_task(node.unit_task_id)
            .await?
        {
            result.push(CompositeTaskNodeWithUnitTask { node, unit_task });
        } else {
            tracing::warn!(
                "CompositeTaskNode {} references missing UnitTask {}",
                node.id,
                node.unit_task_id
            );
        }
    }

    Ok(CompositeTaskNodesResult { nodes: result })
}

/// Gets all nodes for a composite task (mobile - remote mode only).
///
/// Note: Remote mode support for task graph visualization requires additional
/// server-side API endpoints. For now, this returns an empty result.
#[cfg(not(desktop))]
#[tauri::command]
pub async fn get_composite_task_nodes(
    state: State<'_, Arc<RwLock<AppState>>>,
    _composite_task_id: String,
) -> AppResult<CompositeTaskNodesResult> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // TODO: Implement remote API call when server supports composite task nodes
        // endpoint For now, return an empty result
        return Ok(CompositeTaskNodesResult { nodes: Vec::new() });
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Dismisses approval for a task, moving it back to InReview.
#[cfg(desktop)]
#[tauri::command]
pub async fn dismiss_approval(
    state: State<'_, Arc<RwLock<AppState>>>,
    app_handle: tauri::AppHandle,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let request = requests::DismissApprovalRequest {
            task_id: task_id.clone(),
        };

        client.dismiss_approval(request).await?;
        info!("Dismissed approval via remote for task: {}", task_id);
        return Ok(());
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
        if task.status != UnitTaskStatus::Approved {
            return Err(AppError::InvalidRequest(format!(
                "Task {} is not in Approved status (current: {:?})",
                task_id, task.status
            )));
        }
        let old_status = "approved".to_string();
        task.status = UnitTaskStatus::InReview;
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_unit_task(task).await?;
        info!("Dismissed approval for unit task: {}", id);

        let _ = app_handle.emit(
            event_names::TASK_STATUS_CHANGED,
            &TaskStatusChangedEvent {
                task_id: task_id.clone(),
                task_type: TaskType::UnitTask,
                old_status,
                new_status: "in_review".to_string(),
            },
        );

        return Ok(());
    }

    Err(AppError::NotFound(format!("Task not found: {}", task_id)))
}

/// Dismisses approval for a task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn dismiss_approval(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let request = requests::DismissApprovalRequest {
            task_id: task_id.clone(),
        };

        client.dismiss_approval(request).await?;
        info!("Dismissed approval via remote for task: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Creates a pull request for an approved task.
#[cfg(desktop)]
#[tauri::command]
pub async fn create_pr(
    state: State<'_, Arc<RwLock<AppState>>>,
    _app_handle: tauri::AppHandle,
    task_id: String,
) -> AppResult<String> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let request = requests::CreatePrRequest {
            task_id: task_id.clone(),
        };

        let response = client.create_pr(request).await?;
        info!("Created PR via remote for task: {}", task_id);
        return Ok(response.pr_url);
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let executor = runtime
        .executor()
        .await
        .ok_or_else(|| AppError::Internal("Executor not initialized".to_string()))?;

    let prompt = "Create a pull request with the changes from this task. Push the current branch \
                  to the remote and create a PR using the available tools (e.g. `gh pr create`). \
                  Output the PR URL."
        .to_string();

    executor
        .execute_subtask(id, prompt, UnitTaskStatus::PrOpen)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to start PR creation subtask: {}", e)))?;

    info!("Started PR creation subtask for unit task: {}", id);
    Ok(String::new())
}

/// Creates a pull request for an approved task (mobile - remote mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn create_pr(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<String> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let request = requests::CreatePrRequest {
            task_id: task_id.clone(),
        };

        let response = client.create_pr(request).await?;
        info!("Created PR via remote for task: {}", task_id);
        return Ok(response.pr_url);
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Commits approved task changes to the local git repository.
#[cfg(desktop)]
#[tauri::command]
pub async fn commit_to_local(
    state: State<'_, Arc<RwLock<AppState>>>,
    _app_handle: tauri::AppHandle,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let request = requests::CommitToLocalRequest {
            task_id: task_id.clone(),
        };

        client.commit_to_local(request).await?;
        info!("Committed to local via remote for task: {}", task_id);
        return Ok(());
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    let executor = runtime
        .executor()
        .await
        .ok_or_else(|| AppError::Internal("Executor not initialized".to_string()))?;

    let prompt = "Commit the changes from this task to the local repository. Create a \
                  well-structured commit with an appropriate commit message that summarizes the \
                  changes made."
        .to_string();

    executor
        .execute_subtask(id, prompt, UnitTaskStatus::Done)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to start commit-to-local subtask: {}", e))
        })?;

    info!("Started commit-to-local subtask for unit task: {}", id);
    Ok(())
}

/// Commits approved task changes to the local git repository (mobile - remote
/// mode only).
#[cfg(not(desktop))]
#[tauri::command]
pub async fn commit_to_local(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let request = requests::CommitToLocalRequest {
            task_id: task_id.clone(),
        };

        client.commit_to_local(request).await?;
        info!("Committed to local via remote for task: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

// Helper functions

fn parse_agent_type(s: &str) -> AppResult<AiAgentType> {
    match s.to_lowercase().as_str() {
        "claude_code" | "claudecode" => Ok(AiAgentType::ClaudeCode),
        "open_code" | "opencode" => Ok(AiAgentType::OpenCode),
        "gemini_cli" | "geminicli" => Ok(AiAgentType::GeminiCli),
        "codex_cli" | "codexcli" => Ok(AiAgentType::CodexCli),
        "aider" => Ok(AiAgentType::Aider),
        "amp" => Ok(AiAgentType::Amp),
        _ => Err(AppError::InvalidRequest(format!(
            "Unknown agent type: {}",
            s
        ))),
    }
}

fn parse_unit_status(s: &str) -> AppResult<UnitTaskStatus> {
    match s.to_lowercase().as_str() {
        "in_progress" => Ok(UnitTaskStatus::InProgress),
        "in_review" => Ok(UnitTaskStatus::InReview),
        "approved" => Ok(UnitTaskStatus::Approved),
        "pr_open" => Ok(UnitTaskStatus::PrOpen),
        "done" => Ok(UnitTaskStatus::Done),
        "rejected" => Ok(UnitTaskStatus::Rejected),
        "failed" => Ok(UnitTaskStatus::Failed),
        "cancelled" => Ok(UnitTaskStatus::Cancelled),
        _ => Err(AppError::InvalidRequest(format!(
            "Unknown unit task status: {}",
            s
        ))),
    }
}

fn parse_composite_status(s: &str) -> AppResult<CompositeTaskStatus> {
    match s.to_lowercase().as_str() {
        "planning" => Ok(CompositeTaskStatus::Planning),
        "pending_approval" => Ok(CompositeTaskStatus::PendingApproval),
        "in_progress" => Ok(CompositeTaskStatus::InProgress),
        "done" => Ok(CompositeTaskStatus::Done),
        "rejected" => Ok(CompositeTaskStatus::Rejected),
        "failed" => Ok(CompositeTaskStatus::Failed),
        _ => Err(AppError::InvalidRequest(format!(
            "Unknown composite task status: {}",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Agent Type Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_agent_type_claude_code() {
        assert!(matches!(
            parse_agent_type("claude_code"),
            Ok(AiAgentType::ClaudeCode)
        ));
        assert!(matches!(
            parse_agent_type("ClaudeCode"),
            Ok(AiAgentType::ClaudeCode)
        ));
        assert!(matches!(
            parse_agent_type("CLAUDECODE"),
            Ok(AiAgentType::ClaudeCode)
        ));
    }

    #[test]
    fn test_parse_agent_type_other_agents() {
        assert!(matches!(
            parse_agent_type("open_code"),
            Ok(AiAgentType::OpenCode)
        ));
        assert!(matches!(
            parse_agent_type("gemini_cli"),
            Ok(AiAgentType::GeminiCli)
        ));
        assert!(matches!(
            parse_agent_type("codex_cli"),
            Ok(AiAgentType::CodexCli)
        ));
        assert!(matches!(parse_agent_type("aider"), Ok(AiAgentType::Aider)));
        assert!(matches!(parse_agent_type("amp"), Ok(AiAgentType::Amp)));
    }

    #[test]
    fn test_parse_agent_type_invalid() {
        let result = parse_agent_type("invalid_agent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown agent type"));
    }

    // =========================================================================
    // Unit Task Status Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_unit_status_all_variants() {
        assert!(matches!(
            parse_unit_status("in_progress"),
            Ok(UnitTaskStatus::InProgress)
        ));
        assert!(matches!(
            parse_unit_status("in_review"),
            Ok(UnitTaskStatus::InReview)
        ));
        assert!(matches!(
            parse_unit_status("approved"),
            Ok(UnitTaskStatus::Approved)
        ));
        assert!(matches!(
            parse_unit_status("pr_open"),
            Ok(UnitTaskStatus::PrOpen)
        ));
        assert!(matches!(
            parse_unit_status("done"),
            Ok(UnitTaskStatus::Done)
        ));
        assert!(matches!(
            parse_unit_status("rejected"),
            Ok(UnitTaskStatus::Rejected)
        ));
        assert!(matches!(
            parse_unit_status("failed"),
            Ok(UnitTaskStatus::Failed)
        ));
        assert!(matches!(
            parse_unit_status("cancelled"),
            Ok(UnitTaskStatus::Cancelled)
        ));
    }

    #[test]
    fn test_parse_unit_status_case_insensitive() {
        assert!(matches!(
            parse_unit_status("IN_PROGRESS"),
            Ok(UnitTaskStatus::InProgress)
        ));
        assert!(matches!(
            parse_unit_status("In_Review"),
            Ok(UnitTaskStatus::InReview)
        ));
    }

    #[test]
    fn test_parse_unit_status_invalid() {
        let result = parse_unit_status("invalid_status");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown unit task status"));
    }

    // =========================================================================
    // Composite Task Status Parsing Tests
    // =========================================================================

    #[test]
    fn test_parse_composite_status_all_variants() {
        assert!(matches!(
            parse_composite_status("planning"),
            Ok(CompositeTaskStatus::Planning)
        ));
        assert!(matches!(
            parse_composite_status("pending_approval"),
            Ok(CompositeTaskStatus::PendingApproval)
        ));
        assert!(matches!(
            parse_composite_status("in_progress"),
            Ok(CompositeTaskStatus::InProgress)
        ));
        assert!(matches!(
            parse_composite_status("done"),
            Ok(CompositeTaskStatus::Done)
        ));
        assert!(matches!(
            parse_composite_status("rejected"),
            Ok(CompositeTaskStatus::Rejected)
        ));
        assert!(matches!(
            parse_composite_status("failed"),
            Ok(CompositeTaskStatus::Failed)
        ));
    }

    #[test]
    fn test_parse_composite_status_case_insensitive() {
        assert!(matches!(
            parse_composite_status("PLANNING"),
            Ok(CompositeTaskStatus::Planning)
        ));
        assert!(matches!(
            parse_composite_status("Pending_Approval"),
            Ok(CompositeTaskStatus::PendingApproval)
        ));
    }

    #[test]
    fn test_parse_composite_status_invalid() {
        let result = parse_composite_status("invalid_status");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown composite task status"));
    }

    // =========================================================================
    // Prompt Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_prompt_valid() {
        assert!(validate_prompt("A simple prompt").is_ok());
        assert!(validate_prompt("Fix the bug in the login page").is_ok());
        assert!(validate_prompt("a").is_ok()); // Minimum valid prompt
    }

    #[test]
    fn test_validate_prompt_empty() {
        let result = validate_prompt("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_prompt_whitespace_only() {
        let result = validate_prompt("   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_prompt_too_long() {
        let long_prompt = "a".repeat(MAX_PROMPT_LENGTH + 1);
        let result = validate_prompt(&long_prompt);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum length"));
    }

    #[test]
    fn test_validate_prompt_max_length() {
        let max_prompt = "a".repeat(MAX_PROMPT_LENGTH);
        assert!(validate_prompt(&max_prompt).is_ok());
    }

    #[test]
    fn test_validate_prompt_null_byte() {
        let result = validate_prompt("Hello\0World");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null bytes"));
    }

    // =========================================================================
    // Title Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_title_valid() {
        assert!(validate_title("Fix login bug").is_ok());
        assert!(validate_title("").is_ok()); // Empty is allowed for title
        assert!(validate_title("Add new feature for user authentication").is_ok());
    }

    #[test]
    fn test_validate_title_too_long() {
        let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
        let result = validate_title(&long_title);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum length"));
    }

    #[test]
    fn test_validate_title_max_length() {
        let max_title = "a".repeat(MAX_TITLE_LENGTH);
        assert!(validate_title(&max_title).is_ok());
    }

    #[test]
    fn test_validate_title_null_byte() {
        let result = validate_title("Title\0with null");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null bytes"));
    }

    // =========================================================================
    // Rate Limiting Tests
    // =========================================================================

    // NOTE: These tests are combined into a single test because they share
    // global state (LAST_TASK_CREATION_TIME). Running them as separate #[test]
    // functions causes flaky failures due to parallel execution.
    #[cfg(desktop)]
    #[test]
    fn test_check_rate_limit() {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Test 1: First call should always succeed
        LAST_TASK_CREATION_TIME.store(0, Ordering::SeqCst);
        assert!(check_rate_limit().is_ok());

        // Test 2: Rapid subsequent call should be blocked
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        LAST_TASK_CREATION_TIME.store(now, Ordering::SeqCst);

        let result = check_rate_limit();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Please wait at least"));

        // Test 3: Call should succeed after interval has passed
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let past = now.saturating_sub(MIN_TASK_CREATION_INTERVAL_MS + 100);
        LAST_TASK_CREATION_TIME.store(past, Ordering::SeqCst);

        assert!(check_rate_limit().is_ok());
    }
}
