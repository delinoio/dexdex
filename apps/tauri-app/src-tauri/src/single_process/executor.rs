//! Local task executor for running AI agents in single-process mode.
//!
//! This module provides the `LocalExecutor` which handles the actual execution
//! of AI coding agents for unit tasks. It manages git worktrees, runs agents,
//! and streams output events.

use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use coding_agents::{AgentConfig, NormalizedEvent};
use entities::{AgentSession, AiAgentType, UnitTaskStatus};
use git_ops::RepositoryCache;
use task_store::{SqliteTaskStore, TaskStore};
use tauri::{AppHandle, Emitter};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use super::tty_handler::{LocalTtyHandler, TtyInputRequestManager};
use crate::{
    config::data_dir,
    events::{event_names, AgentOutputEvent, TaskStatusChangedEvent, TaskType},
};

/// Result of a task execution.
#[derive(Debug)]
pub enum ExecutionResult {
    /// Task completed successfully.
    Success,
    /// Task failed with an error.
    Failed(String),
    /// Task was cancelled.
    Cancelled,
}

/// Local task executor that runs AI agents in the same process.
pub struct LocalExecutor {
    /// Task store for reading/updating tasks.
    task_store: Arc<SqliteTaskStore>,
    /// Tauri app handle for emitting events.
    app_handle: AppHandle,
    /// TTY input request manager.
    tty_request_manager: Arc<TtyInputRequestManager>,
    /// Repository cache for managing git worktrees.
    repo_cache: RepositoryCache,
    /// Active execution handles keyed by task ID.
    execution_handles: Arc<RwLock<HashMap<Uuid, JoinHandle<ExecutionResult>>>>,
}

impl LocalExecutor {
    /// Creates a new local executor.
    pub fn new(task_store: Arc<SqliteTaskStore>, app_handle: AppHandle) -> Self {
        // Initialize the repository cache using the data directory
        let data_dir = data_dir().unwrap_or_else(|_| {
            let home = dirs::home_dir().expect("Could not find home directory");
            home.join(".delidev")
        });
        let repo_cache = RepositoryCache::new(&data_dir);

        Self {
            task_store,
            app_handle,
            tty_request_manager: Arc::new(TtyInputRequestManager::new()),
            repo_cache,
            execution_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the TTY request manager for responding to input requests.
    pub fn tty_request_manager(&self) -> Arc<TtyInputRequestManager> {
        self.tty_request_manager.clone()
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

        // Clone values needed for the spawned task
        let task_store = self.task_store.clone();
        let app_handle = self.app_handle.clone();
        let tty_manager = self.tty_request_manager.clone();
        let repo_cache = RepositoryCache::new(
            data_dir().unwrap_or_else(|_| dirs::home_dir().unwrap().join(".delidev")),
        );
        let remote_url = repository.remote_url.clone();
        let prompt = task.prompt.clone();

        // Spawn the execution task
        let handle = tokio::spawn(async move {
            let result = Self::run_agent_task(
                task_id,
                session_id,
                &remote_url,
                &branch_name,
                agent_type,
                agent_model,
                &prompt,
                task_store.clone(),
                app_handle.clone(),
                tty_manager,
                repo_cache,
            )
            .await;

            // Update task status based on result
            match &result {
                ExecutionResult::Success => {
                    info!("Task {} completed successfully", task_id);
                    if let Err(e) = Self::update_task_status(
                        &task_store,
                        &app_handle,
                        task_id,
                        UnitTaskStatus::InReview,
                    )
                    .await
                    {
                        error!("Failed to update task status: {}", e);
                    }
                }
                ExecutionResult::Failed(error) => {
                    error!("Task {} failed: {}", task_id, error);
                    // Keep task in InProgress status but log the error
                    // Future: Add a separate failed status or error field
                }
                ExecutionResult::Cancelled => {
                    info!("Task {} was cancelled", task_id);
                }
            }

            result
        });

        // Store the handle
        let mut handles = self.execution_handles.write().await;
        handles.insert(task_id, handle);

        Ok(())
    }

    /// Runs the agent task (internal implementation).
    async fn run_agent_task(
        task_id: Uuid,
        session_id: Uuid,
        remote_url: &str,
        branch_name: &str,
        agent_type: AiAgentType,
        agent_model: Option<String>,
        prompt: &str,
        task_store: Arc<SqliteTaskStore>,
        app_handle: AppHandle,
        tty_manager: Arc<TtyInputRequestManager>,
        repo_cache: RepositoryCache,
    ) -> ExecutionResult {
        // Create a worktree for the task
        info!(
            "Creating worktree for task {} at branch {}",
            task_id, branch_name
        );

        let worktree_path = match repo_cache.create_worktree_for_task(
            remote_url,
            branch_name,
            &task_id.to_string(),
            None, // Use default credentials for now
        ) {
            Ok(path) => {
                info!("Created worktree at {:?}", path);
                path
            }
            Err(e) => {
                error!("Failed to create worktree: {}", e);
                return ExecutionResult::Failed(format!("Failed to create worktree: {}", e));
            }
        };

        // Create the agent configuration
        let mut config = AgentConfig::new(agent_type, worktree_path.to_string_lossy(), prompt);

        if let Some(model) = agent_model {
            config = config.with_model(model);
        }

        // Create the TTY handler
        let tty_handler =
            LocalTtyHandler::new(app_handle.clone(), task_id, session_id, tty_manager);

        // Create an event channel
        let (event_tx, mut event_rx) = mpsc::channel::<NormalizedEvent>(1024);

        // Collect output log (used for fallback if event_handler fails)
        let output_log = Vec::new();

        // Run the agent
        let agent = coding_agents::create_agent(agent_type);

        // Spawn a task to handle events
        let app_handle_clone = app_handle.clone();
        let event_handler = tokio::spawn(async move {
            let mut logs = Vec::new();
            while let Some(event) = event_rx.recv().await {
                // Serialize the event for the output log
                if let Ok(json) = serde_json::to_string(&event) {
                    logs.push(json);
                }

                // Emit the event to the frontend
                let output_event = AgentOutputEvent {
                    task_id: task_id.to_string(),
                    session_id: session_id.to_string(),
                    event: event.clone(),
                };

                if let Err(e) = app_handle_clone.emit(event_names::AGENT_OUTPUT, &output_event) {
                    warn!("Failed to emit agent output event: {}", e);
                }
            }
            logs
        });

        // Run the agent
        info!(
            "Starting agent execution for task {}, agent_type={:?}",
            task_id, agent_type
        );
        let run_result = agent
            .run(config, event_tx, Some(Box::new(tty_handler)))
            .await;
        info!(
            "Agent execution completed for task {}, result={:?}",
            task_id,
            run_result.as_ref().map(|_| "Ok").unwrap_or_else(|e| "Err")
        );

        // Wait for event handler to finish and collect logs
        let logs = match event_handler.await {
            Ok(logs) => logs,
            Err(e) => {
                warn!("Event handler task failed: {}", e);
                output_log
            }
        };

        // Update the session with the output log
        let output_log_str = logs.join("\n");
        if let Ok(Some(mut session)) = task_store.get_agent_session(session_id).await {
            session.output_log = Some(output_log_str);
            session.completed_at = Some(Utc::now());
            if let Err(e) = task_store.update_agent_session(session).await {
                error!("Failed to update agent session: {}", e);
            }
        }

        // Clean up worktree (optional - might want to keep for review)
        // repo_cache.remove_worktree_for_task(remote_url, &task_id.to_string(),
        // branch_name);

        match run_result {
            Ok(()) => ExecutionResult::Success,
            Err(e) => ExecutionResult::Failed(e.to_string()),
        }
    }

    /// Updates a task's status and emits an event.
    async fn update_task_status(
        task_store: &Arc<SqliteTaskStore>,
        app_handle: &AppHandle,
        task_id: Uuid,
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
        let handles = self.execution_handles.read().await;
        if let Some(handle) = handles.get(&task_id) {
            !handle.is_finished()
        } else {
            false
        }
    }

    /// Cancels execution of a task (future enhancement).
    pub async fn cancel_execution(&self, task_id: Uuid) -> bool {
        let mut handles = self.execution_handles.write().await;
        if let Some(handle) = handles.remove(&task_id) {
            handle.abort();
            true
        } else {
            false
        }
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
