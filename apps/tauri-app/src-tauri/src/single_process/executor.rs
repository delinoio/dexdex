//! Local task executor for running AI agents in single-process mode.
//!
//! This module provides the `LocalExecutor` which wraps the core `TaskExecutor`
//! from the `coding_agents` crate with Tauri-specific event emission.

use std::sync::Arc;

use chrono::Utc;
pub use coding_agents::executor::ExecutionResult;
use coding_agents::{
    executor::{
        AgentOutputEvent as CoreAgentOutputEvent, EventEmitter,
        TaskCompletedEvent as CoreTaskCompletedEvent, TaskExecutionConfig, TaskExecutor,
        TaskStatusChangedEvent as CoreTaskStatusChangedEvent, TaskType as CoreTaskType,
        TtyInputRequestEvent as CoreTtyInputRequestEvent, TtyInputRequestManager,
    },
    AgentResult,
};
use entities::{AgentSession, AiAgentType, UnitTaskStatus};
use task_store::{SqliteTaskStore, TaskStore};
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    config::data_dir,
    events::{
        event_names, AgentOutputEvent, TaskCompletedEvent, TaskStatusChangedEvent, TaskType,
        TtyInputRequestEvent,
    },
};

/// Tauri-specific event emitter that emits events via the Tauri app handle.
pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    /// Creates a new Tauri event emitter.
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl EventEmitter for TauriEventEmitter {
    fn emit_task_status_changed(&self, event: CoreTaskStatusChangedEvent) -> AgentResult<()> {
        let tauri_event = TaskStatusChangedEvent {
            task_id: event.task_id,
            task_type: match event.task_type {
                CoreTaskType::UnitTask => TaskType::UnitTask,
                CoreTaskType::CompositeTask => TaskType::CompositeTask,
            },
            old_status: event.old_status,
            new_status: event.new_status,
        };

        self.app_handle
            .emit(event_names::TASK_STATUS_CHANGED, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }

    fn emit_agent_output(&self, event: CoreAgentOutputEvent) -> AgentResult<()> {
        let tauri_event = AgentOutputEvent {
            task_id: event.task_id,
            session_id: event.session_id,
            event: event.event,
        };

        self.app_handle
            .emit(event_names::AGENT_OUTPUT, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }

    fn emit_tty_input_request(&self, event: CoreTtyInputRequestEvent) -> AgentResult<()> {
        let tauri_event = TtyInputRequestEvent {
            request_id: event.request_id,
            task_id: event.task_id,
            session_id: event.session_id,
            question: event.question,
            options: event.options,
        };

        self.app_handle
            .emit(event_names::TTY_INPUT_REQUEST, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }

    fn emit_task_completed(&self, event: CoreTaskCompletedEvent) -> AgentResult<()> {
        let tauri_event = TaskCompletedEvent {
            task_id: event.task_id,
            task_type: match event.task_type {
                CoreTaskType::UnitTask => TaskType::UnitTask,
                CoreTaskType::CompositeTask => TaskType::CompositeTask,
            },
            success: event.success,
            error: event.error,
        };

        self.app_handle
            .emit(event_names::TASK_COMPLETED, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }
}

/// Local task executor that runs AI agents in the same process.
///
/// This is a wrapper around the core `TaskExecutor` that integrates with
/// the Tauri task store and handles session management.
pub struct LocalExecutor {
    /// Task store for reading/updating tasks.
    task_store: Arc<SqliteTaskStore>,
    /// Tauri app handle for emitting events.
    app_handle: AppHandle,
    /// Core task executor.
    executor: TaskExecutor<TauriEventEmitter>,
}

impl LocalExecutor {
    /// Creates a new local executor.
    pub fn new(task_store: Arc<SqliteTaskStore>, app_handle: AppHandle) -> Self {
        // Initialize the repository cache using the data directory
        let data_dir = data_dir().unwrap_or_else(|_| {
            let home = dirs::home_dir().expect("Could not find home directory");
            home.join(".delidev")
        });

        let emitter = Arc::new(TauriEventEmitter::new(app_handle.clone()));
        let executor = TaskExecutor::new(data_dir, emitter);

        Self {
            task_store,
            app_handle,
            executor,
        }
    }

    /// Returns the TTY request manager for responding to input requests.
    pub fn tty_request_manager(&self) -> Arc<TtyInputRequestManager> {
        self.executor.tty_request_manager()
    }

    /// Executes a unit task asynchronously.
    ///
    /// This spawns a background task that:
    /// 1. Creates a git worktree for the task
    /// 2. Runs the AI agent with the task prompt
    /// 3. Streams output events to the frontend
    /// 4. Updates the task status on completion
    pub async fn execute_unit_task(&self, task_id: Uuid) -> Result<(), String> {
        info!("Starting execution of unit task: {}", task_id);

        // Get the task from the store
        let task = self
            .task_store
            .get_unit_task(task_id)
            .await
            .map_err(|e| format!("Failed to get task: {}", e))?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        // Get the repository group to find repositories
        let repo_group = self
            .task_store
            .get_repository_group(task.repository_group_id)
            .await
            .map_err(|e| format!("Failed to get repository group: {}", e))?
            .ok_or_else(|| format!("Repository group not found: {}", task.repository_group_id))?;

        // Get the first repository (for now, we only support single-repo tasks)
        let repo_id = repo_group
            .repository_ids
            .first()
            .ok_or_else(|| "Repository group has no repositories".to_string())?;

        let repository = self
            .task_store
            .get_repository(*repo_id)
            .await
            .map_err(|e| format!("Failed to get repository: {}", e))?
            .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

        // Get the agent task to find agent type
        let agent_task = self
            .task_store
            .get_agent_task(task.agent_task_id)
            .await
            .map_err(|e| format!("Failed to get agent task: {}", e))?
            .ok_or_else(|| format!("Agent task not found: {}", task.agent_task_id))?;

        let agent_type = agent_task.ai_agent_type.unwrap_or(AiAgentType::ClaudeCode);
        let agent_model = agent_task.ai_agent_model.clone();

        // Create an agent session
        let mut session = AgentSession::new(task.agent_task_id, agent_type);
        if let Some(model) = &agent_model {
            session = session.with_model(model.clone());
        }
        session.started_at = Some(Utc::now());

        let session = self
            .task_store
            .create_agent_session(session)
            .await
            .map_err(|e| format!("Failed to create agent session: {}", e))?;

        let session_id = session.id;

        // Determine branch name
        let branch_name = task
            .branch_name
            .clone()
            .unwrap_or_else(|| format!("delidev/{}", task_id));

        // Create the execution config
        let config = TaskExecutionConfig {
            task_id,
            session_id,
            remote_url: repository.remote_url.clone(),
            branch_name: branch_name.clone(),
            agent_type,
            agent_model,
            prompt: task.prompt.clone(),
        };

        // Clone values needed for the spawned task
        let task_store = self.task_store.clone();
        let app_handle = self.app_handle.clone();
        // Reuse the existing repo_cache to avoid redundant clones
        let repo_cache = self.executor.repo_cache().clone();
        let emitter = Arc::new(TauriEventEmitter::new(app_handle.clone()));
        let executor = TaskExecutor::with_repo_cache(repo_cache, emitter);

        // Spawn the execution task
        tokio::spawn(async move {
            let result = executor.execute_and_wait(config).await;

            // Update task status based on result
            match &result {
                ExecutionResult::Success => {
                    info!("Task {} completed successfully", task_id);
                    if let Err(e) = Self::update_task_status(
                        &task_store,
                        &app_handle,
                        task_id,
                        session_id,
                        UnitTaskStatus::InReview,
                    )
                    .await
                    {
                        error!("Failed to update task status: {}", e);
                    }
                }
                ExecutionResult::Failed(error) => {
                    error!("Task {} failed: {}", task_id, error);
                    // Update task status to Failed
                    if let Err(e) = Self::update_task_status(
                        &task_store,
                        &app_handle,
                        task_id,
                        session_id,
                        UnitTaskStatus::Failed,
                    )
                    .await
                    {
                        error!("Failed to update task status to Failed: {}", e);
                    }
                }
                ExecutionResult::Cancelled => {
                    info!("Task {} was cancelled", task_id);
                }
            }
        });

        Ok(())
    }

    /// Updates a task's status and emits an event.
    async fn update_task_status(
        task_store: &Arc<SqliteTaskStore>,
        app_handle: &AppHandle,
        task_id: Uuid,
        session_id: Uuid,
        new_status: UnitTaskStatus,
    ) -> Result<(), String> {
        let mut task = task_store
            .get_unit_task(task_id)
            .await
            .map_err(|e| format!("Failed to get task: {}", e))?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        let old_status = task.status;
        task.status = new_status;
        task.updated_at = Utc::now();

        task_store
            .update_unit_task(task)
            .await
            .map_err(|e| format!("Failed to update task: {}", e))?;

        // Update session completed_at
        if let Ok(Some(mut session)) = task_store.get_agent_session(session_id).await {
            session.completed_at = Some(Utc::now());
            if let Err(e) = task_store.update_agent_session(session).await {
                warn!("Failed to update agent session: {}", e);
            }
        }

        // Emit status changed event
        let event = TaskStatusChangedEvent {
            task_id: task_id.to_string(),
            task_type: TaskType::UnitTask,
            old_status: format!("{:?}", old_status).to_lowercase(),
            new_status: format!("{:?}", new_status).to_lowercase(),
        };

        if let Err(e) = app_handle.emit(event_names::TASK_STATUS_CHANGED, &event) {
            warn!("Failed to emit task status changed event: {}", e);
        }

        Ok(())
    }

    /// Checks if a task is currently being executed.
    pub async fn is_executing(&self, task_id: Uuid) -> bool {
        self.executor.is_executing(task_id).await
    }

    /// Cancels execution of a task.
    pub async fn cancel_execution(&self, task_id: Uuid) -> bool {
        self.executor.cancel_execution(task_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_debug() {
        let success = ExecutionResult::Success;
        assert!(format!("{:?}", success).contains("Success"));

        let failed = ExecutionResult::Failed("test error".to_string());
        assert!(format!("{:?}", failed).contains("Failed"));
        assert!(format!("{:?}", failed).contains("test error"));

        let cancelled = ExecutionResult::Cancelled;
        assert!(format!("{:?}", cancelled).contains("Cancelled"));
    }
}
