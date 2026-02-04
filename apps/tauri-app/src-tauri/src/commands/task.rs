//! Task-related Tauri commands.

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
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::AppMode,
    error::{AppError, AppResult},
    remote_client::{
        entity_to_rpc_agent_type, entity_to_rpc_composite_status, entity_to_rpc_unit_status,
        rpc_to_entity_composite_task, rpc_to_entity_unit_task, validate_optional_name,
        validate_text, validate_uuid_string, RemoteClient,
    },
    state::AppState,
};

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

/// Response for get_task_logs command.
#[cfg(desktop)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLogsResponse {
    pub events: Vec<NormalizedEventEntry>,
    pub is_complete: bool,
    pub last_event_id: Option<i64>,
}

/// Response for get_task_logs command (mobile stub).
#[cfg(not(desktop))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskLogsResponse {
    pub events: Vec<serde_json::Value>,
    pub is_complete: bool,
    pub last_event_id: Option<i64>,
}

/// Parameters for responding to a TTY input request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespondTtyInputParams {
    pub request_id: String,
    pub response: String,
}

/// Creates a new unit task.
#[cfg(desktop)]
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        "Local mode is not supported on this platform".to_string(),
    ))
}

/// Creates a new composite task.
#[cfg(desktop)]
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        "Local mode is not supported on this platform".to_string(),
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        "Local mode is not supported on this platform".to_string(),
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        "Local mode is not supported on this platform".to_string(),
    ))
}

/// Approves a task.
#[cfg(desktop)]
#[tauri::command]
pub async fn approve_task(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // Remote mode: make API call to main server
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        task.status = UnitTaskStatus::Approved;
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_unit_task(task).await?;
        info!("Approved unit task: {}", id);
        return Ok(());
    }

    // Try composite task
    if let Some(mut task) = runtime.task_store_arc().get_composite_task(id).await? {
        if task.status == CompositeTaskStatus::PendingApproval {
            task.status = CompositeTaskStatus::InProgress;
        } else {
            task.status = CompositeTaskStatus::Done;
        }
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_composite_task(task).await?;
        info!("Approved composite task: {}", id);
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

        let request = requests::ApproveTaskRequest {
            task_id: task_id.clone(),
        };

        client.approve_task(request).await?;
        info!("Approved task via remote: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        "Local mode is not supported on this platform".to_string(),
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

        let request = requests::RejectTaskRequest {
            task_id: task_id.clone(),
            reason,
        };

        client.reject_task(request).await?;
        info!("Rejected task via remote: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        "Local mode is not supported on this platform".to_string(),
    ))
}

/// Requests changes for a task.
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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

    // Try to find and update unit task
    if let Some(mut task) = runtime.task_store_arc().get_unit_task(id).await? {
        task.status = UnitTaskStatus::InProgress;
        task.updated_at = chrono::Utc::now();
        runtime.task_store_arc().update_unit_task(task).await?;
        info!(
            "Requested changes for unit task: {} (feedback: {})",
            id, feedback
        );
        return Ok(());
    }

    Err(AppError::NotFound(format!("Task not found: {}", task_id)))
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

        let request = requests::RequestChangesRequest {
            task_id: task_id.clone(),
            feedback: feedback.clone(),
        };

        client.request_changes(request).await?;
        info!("Requested changes via remote for task: {}", task_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        "Local mode is not supported on this platform".to_string(),
    ))
}

/// Gets logs for a task.
///
/// Returns normalized events from the agent session output.
#[cfg(desktop)]
#[tauri::command]
pub async fn get_task_logs(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    after_event_id: Option<i64>,
) -> AppResult<TaskLogsResponse> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // In remote mode on desktop, we currently return minimal data
        // Full log streaming support requires additional server-side work
        // For now, return an empty response indicating task is complete
        // TODO: Implement proper remote log streaming
        let _ = (task_id, after_event_id);
        return Ok(TaskLogsResponse {
            events: Vec::new(),
            is_complete: true,
            last_event_id: None,
        });
    }

    let id = Uuid::parse_str(&task_id)
        .map_err(|e| AppError::InvalidRequest(format!("Invalid task ID: {}", e)))?;

    let runtime = state
        .local_runtime
        .as_ref()
        .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

    // Get the unit task
    let task = runtime
        .task_store_arc()
        .get_unit_task(id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task not found: {}", task_id)))?;

    // Get the agent task (verifies it exists)
    let _agent_task = runtime
        .task_store_arc()
        .get_agent_task(task.agent_task_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Agent task not found: {}", task.agent_task_id))
        })?;

    // Get the sessions
    let mut sessions = runtime
        .task_store_arc()
        .list_agent_sessions(task.agent_task_id)
        .await?;

    // Sort sessions by created_at to ensure we get the latest one
    sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // If no sessions, return empty
    if sessions.is_empty() {
        let is_complete = task.status != UnitTaskStatus::InProgress;
        return Ok(TaskLogsResponse {
            events: Vec::new(),
            is_complete,
            last_event_id: None,
        });
    }

    // Get the latest session's output log (safe because we checked is_empty()
    // above)
    let session = sessions
        .last()
        .ok_or_else(|| AppError::Internal("Sessions list became empty unexpectedly".to_string()))?;

    // Determine completion based on the latest agent session when available,
    // falling back to the unit task status otherwise.
    let is_complete = if session.completed_at.is_some() {
        true
    } else {
        task.status != UnitTaskStatus::InProgress
    };
    let mut events = Vec::new();
    let mut last_event_id: Option<i64> = None;

    if let Some(output_log) = &session.output_log {
        // Parse the output log (each line is a JSON timestamped event)
        for (idx, line) in output_log.lines().enumerate() {
            let event_id = idx as i64;

            // Skip events before after_event_id
            if let Some(after_id) = after_event_id {
                if event_id <= after_id {
                    continue;
                }
            }

            // Try to parse as TimestampedEvent first (new format with timestamps)
            if let Ok(timestamped) = serde_json::from_str::<TimestampedEvent>(line) {
                events.push(NormalizedEventEntry {
                    id: event_id,
                    timestamp: timestamped.timestamp.to_rfc3339(),
                    event: timestamped.event,
                });
                last_event_id = Some(event_id);
            } else if let Ok(event) = serde_json::from_str::<NormalizedEvent>(line) {
                // Fallback: parse as NormalizedEvent for backwards compatibility
                // with logs created before timestamps were added
                events.push(NormalizedEventEntry {
                    id: event_id,
                    timestamp: session.created_at.to_rfc3339(),
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
    })
}

/// Gets logs for a task (mobile - remote mode only).
///
/// Note: In remote mode, we fetch the session log from the server. The log
/// format may differ from local mode as we receive it as a single string
/// rather than parsed events.
#[cfg(not(desktop))]
#[tauri::command]
pub async fn get_task_logs(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
    _after_event_id: Option<i64>,
) -> AppResult<TaskLogsResponse> {
    let state = state.read().await;

    if state.mode == AppMode::Remote {
        // In remote mode for mobile, we currently return minimal data
        // Full log streaming support requires additional server-side work
        // For now, return an empty response indicating task is complete
        // TODO: Implement proper remote log streaming
        let _ = task_id;
        return Ok(TaskLogsResponse {
            events: Vec::new(),
            is_complete: true,
            last_event_id: None,
        });
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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

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
        let base_url = state
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config("Remote server URL not configured".to_string()))?;
        let client = RemoteClient::new(state.http_client.clone(), base_url.clone());

        let request = requests::SubmitTtyInputRequest {
            request_id: params.request_id.clone(),
            response: params.response,
        };

        client.submit_tty_input(request).await?;
        info!("Responded to TTY input via remote: {}", params.request_id);
        return Ok(());
    }

    Err(AppError::InvalidRequest(
        "Local mode is not supported on this platform".to_string(),
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
        // TODO: Implement remote API call when server supports composite task nodes endpoint
        let _ = composite_task_id;
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
        // TODO: Implement remote API call when server supports composite task nodes endpoint
        // For now, return an empty result
        return Ok(CompositeTaskNodesResult { nodes: Vec::new() });
    }

    Err(AppError::InvalidRequest(
        "Local mode is not supported on this platform".to_string(),
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
}
