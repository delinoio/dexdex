//! Local task executor for running AI agents in single-process mode.
//!
//! This module provides the `LocalExecutor` which wraps the core `TaskExecutor`
//! from the `coding_agents` crate with platform-specific event emission.
//!
//! NOTE: This is a minimal stub implementation. In the new architecture,
//! workers run as separate processes. This stub exists to satisfy the
//! Tauri desktop app's single-process mode compilation requirements.

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use coding_agents::executor::{
    EventEmitter, TaskCompletedEvent, TaskExecutionConfig, TaskExecutor, TaskStatusChangedEvent,
    TaskType,
};
use entities::{AiAgentType, UnitTaskStatus};
use task_store::{MemoryTaskStore, TaskStore};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{TtyInputRequestManager, error::WorkerError};

/// Local task executor that runs AI agents in the same process.
///
/// This is a stub wrapper around the core `TaskExecutor` that integrates
/// with the task store and handles session management. In the new architecture,
/// this is kept minimal as workers are separate processes.
pub struct LocalExecutor<E: EventEmitter> {
    /// Task store for reading/updating tasks.
    task_store: Arc<MemoryTaskStore>,
    /// Core task executor.
    executor: TaskExecutor<E>,
    /// Event emitter for platform-specific event delivery.
    emitter: Arc<E>,
    /// Data directory path.
    #[allow(dead_code)]
    data_dir: PathBuf,
    /// Active execution handles keyed by task ID.
    execution_handles: Arc<RwLock<HashMap<Uuid, JoinHandle<()>>>>,
}

impl<E: EventEmitter + 'static> LocalExecutor<E> {
    /// Creates a new local executor.
    pub fn new(task_store: Arc<MemoryTaskStore>, data_dir: PathBuf, emitter: Arc<E>) -> Self {
        let executor = TaskExecutor::new(data_dir.clone(), emitter.clone());

        Self {
            task_store,
            executor,
            emitter,
            data_dir,
            execution_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the TTY request manager for responding to input requests.
    pub fn tty_request_manager(&self) -> Arc<TtyInputRequestManager> {
        self.executor.tty_request_manager()
    }

    /// Returns the event emitter.
    pub fn emitter(&self) -> &Arc<E> {
        &self.emitter
    }

    /// Executes a unit task asynchronously.
    ///
    /// NOTE: This is a stub implementation. In the new architecture, tasks
    /// are executed by separate worker processes.
    pub async fn execute_unit_task(&self, task_id: Uuid) -> Result<(), String> {
        info!("execute_unit_task called for task {} (stub)", task_id);

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

        // Get the first repository
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

        let agent_type = AiAgentType::ClaudeCode;
        let agent_model: Option<String> = None;

        // Create an agent session
        let session = entities::AgentSession::new(task_id, agent_type);
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
            .unwrap_or_else(|| format!("dexdex/{}", task_id));

        // Create the execution config
        let config = TaskExecutionConfig {
            task_id,
            session_id,
            remote_url: repository.remote_url.clone(),
            branch_name,
            agent_type,
            agent_model,
            prompt: task.prompt.clone(),
        };

        let task_store = self.task_store.clone();
        let emitter = self.emitter.clone();
        let execution_handles = self.execution_handles.clone();
        let executor = TaskExecutor::new(self.data_dir.clone(), emitter.clone());

        let handle = tokio::spawn(async move {
            let exec_result = executor.execute_and_wait_without_cleanup(config).await;
            let result = exec_result.result;

            // Update task status based on result
            let new_status = match &result {
                coding_agents::executor::ExecutionResult::Success { .. } => {
                    info!("Task {} completed successfully", task_id);
                    UnitTaskStatus::Completed
                }
                coding_agents::executor::ExecutionResult::Failed { error, .. } => {
                    error!("Task {} failed: {}", task_id, error);
                    UnitTaskStatus::Failed
                }
                coding_agents::executor::ExecutionResult::Cancelled => {
                    info!("Task {} was cancelled", task_id);
                    UnitTaskStatus::Cancelled
                }
            };

            let success = result.is_success();
            let error_msg = match &result {
                coding_agents::executor::ExecutionResult::Failed { error, .. } => {
                    Some(error.clone())
                }
                _ => None,
            };

            // Update the task status
            if let Ok(Some(mut t)) = task_store.get_unit_task(task_id).await {
                let old_status_str = serde_json::to_string(&t.status)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string();

                t.status = new_status;
                t.updated_at = chrono::Utc::now();
                if let Err(e) = task_store.update_unit_task(t).await {
                    error!("Failed to update task status: {}", e);
                }

                let new_status_str = serde_json::to_string(&new_status)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string();

                if let Err(e) = emitter.emit_task_status_changed(TaskStatusChangedEvent {
                    task_id: task_id.to_string(),
                    task_type: TaskType::UnitTask,
                    old_status: old_status_str,
                    new_status: new_status_str,
                }) {
                    warn!("Failed to emit status changed event: {}", e);
                }
            }

            // Update session completed_at
            if let Ok(Some(mut s)) = task_store.get_agent_session(session_id).await {
                s.completed_at = Some(chrono::Utc::now());
                if let Err(e) = task_store.update_agent_session(s).await {
                    warn!("Failed to update agent session: {}", e);
                }
            }

            // Emit task-completed event
            if let Err(e) = emitter.emit_task_completed(TaskCompletedEvent {
                task_id: task_id.to_string(),
                task_type: TaskType::UnitTask,
                success,
                error: error_msg,
            }) {
                warn!("Failed to emit task completed event: {}", e);
            }

            execution_handles.write().await.remove(&task_id);
        });

        self.execution_handles.write().await.insert(task_id, handle);

        Ok(())
    }

    /// Checks if a task is currently being executed.
    pub async fn is_executing(&self, task_id: Uuid) -> bool {
        let handles = self.execution_handles.read().await;
        if let Some(handle) = handles.get(&task_id) {
            !handle.is_finished()
        } else {
            false
        }
    }

    /// Cancels a running task.
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<(), WorkerError> {
        let mut handles = self.execution_handles.write().await;
        if let Some(handle) = handles.remove(&task_id) {
            handle.abort();
            info!("Cancelled task {}", task_id);
        }
        Ok(())
    }
}
