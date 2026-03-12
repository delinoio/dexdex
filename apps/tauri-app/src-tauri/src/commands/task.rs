//! Task-related Tauri commands.

#[cfg(desktop)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use entities::{AiAgentType, UnitTask, UnitTaskStatus};
use serde::{Deserialize, Serialize};
#[cfg(desktop)]
use task_store::{TaskFilter, TaskStore};
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::AppMode,
    error::{AppError, AppResult},
    remote_client::{
        entity_to_rpc_agent_type, validate_optional_name, validate_text, validate_uuid_string,
    },
    state::{AppState, ERR_LOCAL_MODE_NOT_SUPPORTED},
};

// =============================================================================
// Security Constants
// =============================================================================

/// Maximum allowed prompt length in characters.
const MAX_PROMPT_LENGTH: usize = 100_000;

/// Minimum prompt length to be useful.
const MIN_PROMPT_LENGTH: usize = 1;

/// Maximum allowed title length in characters.
const MAX_TITLE_LENGTH: usize = 500;

/// Minimum time between task creations in milliseconds.
#[cfg(desktop)]
const MIN_TASK_CREATION_INTERVAL_MS: u64 = 500;

/// Global rate limiter for task creation.
#[cfg(desktop)]
static LAST_TASK_CREATION_TIME: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// Validation Functions
// =============================================================================

fn validate_prompt(prompt: &str) -> AppResult<()> {
    if prompt.trim().len() < MIN_PROMPT_LENGTH {
        return Err(AppError::InvalidRequest(
            "Prompt cannot be empty".to_string(),
        ));
    }

    if prompt.len() > MAX_PROMPT_LENGTH {
        return Err(AppError::InvalidRequest(format!(
            "Prompt exceeds maximum length of {} characters (got {} characters)",
            MAX_PROMPT_LENGTH,
            prompt.len()
        )));
    }

    if prompt.contains('\0') {
        return Err(AppError::InvalidRequest(
            "Prompt cannot contain null bytes".to_string(),
        ));
    }

    Ok(())
}

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

        match LAST_TASK_CREATION_TIME.compare_exchange(
            last,
            now,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => return Ok(()),
            Err(_) => continue,
        }
    }
}

// =============================================================================
// Request/Response Types
// =============================================================================

/// Parameters for creating a unit task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUnitTaskParams {
    pub workspace_id: Option<String>,
    pub repository_group_id: String,
    pub prompt: String,
    pub title: Option<String>,
    pub branch_name: Option<String>,
    pub ai_agent_type: Option<String>,
    pub ai_agent_model: Option<String>,
}

/// Parameters for creating a composite task (stub - returns error).
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
    pub workspace_id: Option<String>,
    pub repository_group_id: Option<String>,
    pub unit_status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Response for get_task command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_task: Option<UnitTask>,
}

/// Response for list_tasks command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTasksResult {
    pub unit_tasks: Vec<UnitTask>,
    pub total_count: i32,
}

/// Stub response for composite tasks (not supported in new architecture).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeTaskNodesResult {
    pub nodes: Vec<serde_json::Value>,
}

/// Response for get_task_logs command.
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

// =============================================================================
// Helper Functions
// =============================================================================

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
        "queued" => Ok(UnitTaskStatus::Queued),
        "in_progress" | "inprogress" => Ok(UnitTaskStatus::InProgress),
        "action_required" | "actionrequired" => Ok(UnitTaskStatus::ActionRequired),
        "blocked" => Ok(UnitTaskStatus::Blocked),
        "completed" | "done" => Ok(UnitTaskStatus::Completed),
        "failed" => Ok(UnitTaskStatus::Failed),
        "cancelled" => Ok(UnitTaskStatus::Cancelled),
        _ => Err(AppError::InvalidRequest(format!(
            "Unknown unit task status: {}",
            s
        ))),
    }
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Creates a new unit task.
#[tauri::command]
pub async fn create_unit_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: CreateUnitTaskParams,
) -> AppResult<UnitTask> {
    // SECURITY: Check rate limit before processing
    #[cfg(desktop)]
    check_rate_limit()?;

    validate_prompt(&params.prompt)?;

    if let Some(ref title) = params.title {
        validate_title(title)?;
    }

    let state = state.read().await;

    validate_uuid_string(&params.repository_group_id, "repository group ID")?;
    validate_text(&params.prompt, "prompt")?;
    validate_optional_name(params.title.as_deref(), "title")?;
    validate_optional_name(params.branch_name.as_deref(), "branch name")?;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let agent_type = params
            .ai_agent_type
            .as_deref()
            .map(parse_agent_type)
            .transpose()?
            .map(entity_to_rpc_agent_type);

        let workspace_id = params
            .workspace_id
            .as_deref()
            .map(|s| s.parse::<Uuid>())
            .transpose()
            .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?
            .unwrap_or_else(Uuid::new_v4);

        let repo_group_id = params
            .repository_group_id
            .parse::<Uuid>()
            .map_err(|e| AppError::InvalidRequest(format!("Invalid repository group ID: {}", e)))?;

        let request = rpc_protocol::requests::CreateTaskRequest {
            workspace_id,
            repository_group_id: repo_group_id,
            title: params.title.unwrap_or_default(),
            prompt: params.prompt,
            branch_name: params.branch_name,
            agent_type,
            model: params.ai_agent_model,
            plan_mode_enabled: false,
        };

        let response = client.create_task(request).await?;
        info!("Created unit task via remote: {}", response.task.id);
        return Ok(response.task);
    }

    #[cfg(desktop)]
    {
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let repo_group_id = Uuid::parse_str(&params.repository_group_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid repository group ID: {}", e)))?;

        let workspace_id = runtime.default_workspace_id();

        let title = params
            .title
            .unwrap_or_else(|| params.prompt.chars().take(80).collect::<String>());

        let mut task = UnitTask::new(workspace_id, repo_group_id, &title, &params.prompt);
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

        return Ok(created);
    }

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Creates a new composite task (not supported in new architecture).
#[tauri::command]
pub async fn create_composite_task(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _params: CreateCompositeTaskParams,
) -> AppResult<serde_json::Value> {
    Err(AppError::InvalidRequest(
        "Composite tasks are not supported in the new architecture. Use unit tasks instead."
            .to_string(),
    ))
}

/// Gets a task by ID.
#[tauri::command]
pub async fn get_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<TaskResponse> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

        let request = rpc_protocol::requests::GetTaskRequest { task_id: id };
        let response = client.get_task(request).await?;
        return Ok(TaskResponse {
            unit_task: Some(response.task),
        });
    }

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        if let Some(unit_task) = runtime.task_store_arc().get_unit_task(id).await? {
            return Ok(TaskResponse {
                unit_task: Some(unit_task),
            });
        }

        return Err(AppError::NotFound(format!("Task not found: {}", task_id)));
    }

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Lists tasks with optional filters.
#[tauri::command]
pub async fn list_tasks(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: ListTasksParams,
) -> AppResult<ListTasksResult> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;

        let workspace_id = params
            .workspace_id
            .as_deref()
            .map(|s| s.parse::<Uuid>())
            .transpose()
            .map_err(|e| AppError::InvalidRequest(format!("Invalid workspace ID: {}", e)))?;

        let repo_group_id = params
            .repository_group_id
            .as_deref()
            .map(|s| s.parse::<Uuid>())
            .transpose()
            .map_err(|e| AppError::InvalidRequest(format!("Invalid repository group ID: {}", e)))?;

        let status = params
            .unit_status
            .as_deref()
            .and_then(|s| parse_unit_status(s).ok());

        let request = rpc_protocol::requests::ListTasksRequest {
            workspace_id,
            repository_group_id: repo_group_id,
            status,
            limit: params.limit.unwrap_or(100),
            offset: params.offset.unwrap_or(0),
        };

        let response = client.list_tasks(request).await?;
        return Ok(ListTasksResult {
            unit_tasks: response.tasks,
            total_count: response.total_count,
        });
    }

    #[cfg(desktop)]
    {
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let filter = TaskFilter {
            workspace_id: params
                .workspace_id
                .as_deref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            repository_group_id: params
                .repository_group_id
                .as_deref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            status: params
                .unit_status
                .as_deref()
                .and_then(|s| parse_unit_status(s).ok()),
            limit: params.limit.map(|l| l as u32),
            offset: params.offset.map(|o| o as u32),
        };

        let (unit_tasks, count) = runtime.task_store_arc().list_unit_tasks(filter).await?;

        return Ok(ListTasksResult {
            unit_tasks,
            total_count: count as i32,
        });
    }

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Approves a task (marks as ActionRequired resolved or transitions status).
/// In new architecture, this is a stub that marks the task as Completed.
#[tauri::command]
pub async fn approve_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "approve_task not supported in remote mode in new architecture".to_string(),
        ));
    }

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
            task.status = UnitTaskStatus::Completed;
            task.updated_at = chrono::Utc::now();
            runtime.task_store_arc().update_unit_task(task).await?;
            info!("Approved (completed) unit task: {}", id);
            return Ok(());
        }

        return Err(AppError::NotFound(format!("Task not found: {}", task_id)));
    }

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Rejects a task (cancels it in the new architecture).
#[tauri::command]
pub async fn reject_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    reason: Option<String>,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "reject_task not supported in remote mode in new architecture".to_string(),
        ));
    }

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
            task.status = UnitTaskStatus::Cancelled;
            task.updated_at = chrono::Utc::now();
            runtime.task_store_arc().update_unit_task(task).await?;
            info!(
                "Rejected (cancelled) unit task: {} (reason: {:?})",
                id, reason
            );
            return Ok(());
        }

        return Err(AppError::NotFound(format!("Task not found: {}", task_id)));
    }

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Updates the plan for a composite task (not supported in new architecture).
#[tauri::command]
pub async fn update_plan_with_prompt(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _task_id: String,
    _prompt: String,
) -> AppResult<()> {
    Err(AppError::InvalidRequest(
        "update_plan_with_prompt not supported in new architecture".to_string(),
    ))
}

/// Requests changes for a task (not supported in new architecture).
#[tauri::command]
pub async fn request_changes(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _task_id: String,
    _feedback: String,
) -> AppResult<()> {
    Err(AppError::InvalidRequest(
        "request_changes not supported in new architecture".to_string(),
    ))
}

/// Gets logs for an agent task.
#[tauri::command]
pub async fn get_task_logs(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _agent_task_id: String,
    _after_event_id: Option<i64>,
) -> AppResult<TaskLogsResponse> {
    // In the new architecture, logs are accessed through SessionOutputEvent
    // records. This is a stub that returns empty results.
    Ok(TaskLogsResponse {
        events: Vec::new(),
        is_complete: true,
        last_event_id: None,
        sessions: Vec::new(),
    })
}

/// Cancels a running task.
#[tauri::command]
pub async fn cancel_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;
        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;
        client
            .cancel_task(rpc_protocol::requests::CancelTaskRequest { task_id: id })
            .await?;
        return Ok(());
    }

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        if let Some(executor) = runtime.executor().await {
            executor.cancel_task(id).await.map_err(AppError::Worker)?;
        }

        if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
            if task.status == UnitTaskStatus::InProgress {
                task.status = UnitTaskStatus::Cancelled;
                task.updated_at = chrono::Utc::now();
                runtime.task_store_arc().update_unit_task(task).await?;
            }
        }

        info!("Cancelled task: {}", id);
        return Ok(());
    }

    #[cfg(not(desktop))]
    let _ = &task_id;

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Deletes a task.
#[tauri::command]
pub async fn delete_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        let client = state.get_remote_client()?;
        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;
        client
            .delete_task(rpc_protocol::requests::DeleteTaskRequest { task_id: id })
            .await?;
        info!("Deleted task via remote: {}", task_id);
        return Ok(());
    }

    #[cfg(desktop)]
    {
        let id = Uuid::parse_str(&task_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let store = runtime.task_store_arc();

        if let Some(unit_task) = store.get_unit_task(id).await? {
            if unit_task.status == UnitTaskStatus::InProgress {
                if let Some(executor) = runtime.executor().await {
                    let _ = executor.cancel_task(id).await;
                    info!("Cancelled running execution before deleting task: {}", id);
                }
            }

            store.delete_unit_task_cascade(id).await?;
            info!("Deleted unit task with cascade: {}", id);
            return Ok(());
        }

        return Err(AppError::NotFound(format!("Task not found: {}", task_id)));
    }

    #[cfg(not(desktop))]
    let _ = &task_id;

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Dismisses approval for a task (stub - not supported in new architecture).
#[tauri::command]
pub async fn dismiss_approval(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _app_handle: tauri::AppHandle,
    _task_id: String,
) -> AppResult<()> {
    Err(AppError::InvalidRequest(
        "dismiss_approval not supported in new architecture".to_string(),
    ))
}

/// Creates a pull request for an approved task (stub).
#[tauri::command]
pub async fn create_pr(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _app_handle: tauri::AppHandle,
    _task_id: String,
) -> AppResult<String> {
    Err(AppError::InvalidRequest(
        "create_pr not supported in new architecture".to_string(),
    ))
}

/// Commits approved task changes to the local git repository (stub).
#[tauri::command]
pub async fn commit_to_local(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _app_handle: tauri::AppHandle,
    _task_id: String,
    _local_path: String,
) -> AppResult<()> {
    Err(AppError::InvalidRequest(
        "commit_to_local not supported in new architecture".to_string(),
    ))
}

/// Responds to a TTY input request from an agent.
#[tauri::command]
pub async fn respond_tty_input(
    state: State<'_, Arc<RwLock<AppState>>>,
    params: RespondTtyInputParams,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        return Err(AppError::InvalidRequest(
            "respond_tty_input not supported in remote mode".to_string(),
        ));
    }

    #[cfg(desktop)]
    {
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let request_id = Uuid::parse_str(&params.request_id)
            .map_err(|e| AppError::InvalidRequest(format!("Invalid request ID: {}", e)))?;

        let tty_manager = runtime
            .tty_request_manager()
            .await
            .ok_or_else(|| AppError::Internal("Executor not initialized".to_string()))?;

        let delivered = tty_manager.respond(request_id, params.response).await;

        if !delivered {
            return Err(AppError::NotFound(format!(
                "TTY request not found or already responded: {}",
                params.request_id
            )));
        }

        info!("Responded to TTY input request: {}", params.request_id);
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(AppError::InvalidRequest(
        ERR_LOCAL_MODE_NOT_SUPPORTED.to_string(),
    ))
}

/// Gets all nodes for a composite task (not supported in new architecture).
#[tauri::command]
pub async fn get_composite_task_nodes(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _composite_task_id: String,
) -> AppResult<CompositeTaskNodesResult> {
    Ok(CompositeTaskNodesResult { nodes: Vec::new() })
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_parse_unit_status_all_variants() {
        assert!(matches!(
            parse_unit_status("queued"),
            Ok(UnitTaskStatus::Queued)
        ));
        assert!(matches!(
            parse_unit_status("in_progress"),
            Ok(UnitTaskStatus::InProgress)
        ));
        assert!(matches!(
            parse_unit_status("completed"),
            Ok(UnitTaskStatus::Completed)
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
    fn test_parse_unit_status_invalid() {
        let result = parse_unit_status("invalid_status");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown unit task status"));
    }

    #[test]
    fn test_validate_prompt_valid() {
        assert!(validate_prompt("A simple prompt").is_ok());
        assert!(validate_prompt("a").is_ok());
    }

    #[test]
    fn test_validate_prompt_empty() {
        let result = validate_prompt("");
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
    fn test_validate_prompt_null_byte() {
        let result = validate_prompt("Hello\0World");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null bytes"));
    }

    #[test]
    fn test_validate_title_valid() {
        assert!(validate_title("Fix login bug").is_ok());
        assert!(validate_title("").is_ok());
    }

    #[test]
    fn test_validate_title_too_long() {
        let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
        let result = validate_title(&long_title);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("maximum length"));
    }

    #[test]
    fn test_validate_title_null_byte() {
        let result = validate_title("Title\0with null");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null bytes"));
    }

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
        let past = now.saturating_sub(MIN_TASK_CREATION_INTERVAL_MS + 100);
        LAST_TASK_CREATION_TIME.store(past, Ordering::SeqCst);

        assert!(check_rate_limit().is_ok());
    }
}
