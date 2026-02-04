//! Local task executor for running AI agents in single-process mode.
//!
//! This module provides the `LocalExecutor` which wraps the core `TaskExecutor`
//! from the `coding_agents` crate with platform-specific event emission.

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use chrono::Utc;
pub use coding_agents::executor::ExecutionResult;
use coding_agents::executor::{EventEmitter, TaskExecutionConfig, TaskExecutor};
use entities::{AgentSession, AiAgentType, TokenUsage, UnitTaskStatus};
use task_store::{SqliteTaskStore, TaskStore};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{TtyInputRequestManager, error::WorkerError};

/// Local task executor that runs AI agents in the same process.
///
/// This is a wrapper around the core `TaskExecutor` that integrates with
/// the task store and handles session management. It's generic over the
/// `EventEmitter` type to support different platforms.
pub struct LocalExecutor<E: EventEmitter> {
    /// Task store for reading/updating tasks.
    task_store: Arc<SqliteTaskStore>,
    /// Core task executor.
    executor: TaskExecutor<E>,
    /// Event emitter for platform-specific event delivery.
    emitter: Arc<E>,
    /// Data directory path (kept for potential future use).
    #[allow(dead_code)]
    data_dir: PathBuf,
    /// Active execution handles keyed by task ID.
    /// This stores handles for spawned tasks so they can be cancelled.
    execution_handles: Arc<RwLock<HashMap<Uuid, JoinHandle<()>>>>,
}

impl<E: EventEmitter + 'static> LocalExecutor<E> {
    /// Creates a new local executor.
    pub fn new(task_store: Arc<SqliteTaskStore>, data_dir: PathBuf, emitter: Arc<E>) -> Self {
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
        let emitter = self.emitter.clone();
        // Reuse the existing repo_cache to avoid redundant clones
        let repo_cache = self.executor.repo_cache().clone();
        let executor = TaskExecutor::with_repo_cache(repo_cache, emitter);
        let execution_handles = self.execution_handles.clone();

        // Spawn the execution task and store the handle for cancellation support
        let handle = tokio::spawn(async move {
            let result = executor.execute_and_wait(config).await;

            // Persist logs to the database before updating task status
            let logs = result.logs();
            if !logs.is_empty() {
                let output_log = logs.join("\n");
                if let Err(e) =
                    Self::persist_session_logs(&task_store, session_id, &output_log).await
                {
                    error!("Failed to persist session logs: {}", e);
                }
            }

            // Update task status based on result
            match &result {
                ExecutionResult::Success { .. } => {
                    info!("Task {} completed successfully", task_id);
                    if let Err(e) = Self::update_task_status(
                        &task_store,
                        task_id,
                        session_id,
                        UnitTaskStatus::InReview,
                    )
                    .await
                    {
                        error!("Failed to update task status: {}", e);
                    }
                }
                ExecutionResult::Failed { error, .. } => {
                    error!("Task {} failed: {}", task_id, error);
                    // Update task status to Failed
                    if let Err(e) = Self::update_task_status(
                        &task_store,
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
                    // Update task status to Cancelled
                    if let Err(e) = Self::update_task_status(
                        &task_store,
                        task_id,
                        session_id,
                        UnitTaskStatus::Cancelled,
                    )
                    .await
                    {
                        error!("Failed to update task status to Cancelled: {}", e);
                    }
                }
            }

            // Remove handle from the map after completion
            execution_handles.write().await.remove(&task_id);
        });

        // Store the handle so it can be cancelled later
        self.execution_handles.write().await.insert(task_id, handle);

        Ok(())
    }

    /// Updates a task's status.
    async fn update_task_status(
        task_store: &Arc<SqliteTaskStore>,
        task_id: Uuid,
        session_id: Uuid,
        new_status: UnitTaskStatus,
    ) -> Result<(), WorkerError> {
        let mut task = task_store
            .get_unit_task(task_id)
            .await?
            .ok_or_else(|| WorkerError::TaskNotFound(task_id.to_string()))?;

        task.status = new_status;
        task.updated_at = Utc::now();

        task_store.update_unit_task(task).await?;

        // Update session completed_at
        if let Ok(Some(mut session)) = task_store.get_agent_session(session_id).await {
            session.completed_at = Some(Utc::now());
            if let Err(e) = task_store.update_agent_session(session).await {
                warn!("Failed to update agent session: {}", e);
            }
        }

        Ok(())
    }

    /// Persists the output logs and token usage to the agent session in the
    /// database.
    async fn persist_session_logs(
        task_store: &Arc<SqliteTaskStore>,
        session_id: Uuid,
        output_log: &str,
    ) -> Result<(), String> {
        if let Ok(Some(mut session)) = task_store.get_agent_session(session_id).await {
            session.output_log = Some(output_log.to_string());

            // Extract token usage from the logs
            if let Some(token_usage) = Self::extract_token_usage_from_logs(output_log) {
                debug!(
                    "Extracted token usage for session {}: input={}, output={}, cache_read={}, \
                     cache_write={}",
                    session_id,
                    token_usage.input_tokens,
                    token_usage.output_tokens,
                    token_usage.cache_read_tokens,
                    token_usage.cache_write_tokens
                );
                session.token_usage = Some(token_usage);
            }

            task_store
                .update_agent_session(session)
                .await
                .map_err(|e| format!("Failed to update agent session with logs: {}", e))?;
            info!(
                "Persisted {} bytes of logs for session {}",
                output_log.len(),
                session_id
            );
        } else {
            warn!("Could not find session {} to persist logs", session_id);
        }
        Ok(())
    }

    /// Extracts token usage from the log entries.
    ///
    /// This function parses the JSON log entries looking for usage_report
    /// events and aggregates the token usage data.
    fn extract_token_usage_from_logs(output_log: &str) -> Option<TokenUsage> {
        let mut total_usage = TokenUsage::new();
        let mut found_usage = false;

        for line in output_log.lines() {
            // Try to parse the line as a timestamped event
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                // Check if this is a timestamped event with a usage_report
                if let Some(event) = value.get("event") {
                    if let Some(event_type) = event.get("type").and_then(|v| v.as_str()) {
                        if event_type == "usage_report" {
                            let input_tokens = event
                                .get("input_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let output_tokens = event
                                .get("output_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let cache_read_tokens = event
                                .get("cache_read_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let cache_write_tokens = event
                                .get("cache_write_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);

                            let usage = TokenUsage {
                                input_tokens,
                                output_tokens,
                                cache_read_tokens,
                                cache_write_tokens,
                            };
                            total_usage.add(&usage);
                            found_usage = true;
                        }
                    }
                }
            }
        }

        if found_usage { Some(total_usage) } else { None }
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

    /// Cancels execution of a task.
    ///
    /// Returns `true` if the task was found and aborted, `false` if it wasn't
    /// running.
    pub async fn cancel_execution(&self, task_id: Uuid) -> bool {
        let mut handles = self.execution_handles.write().await;
        if let Some(handle) = handles.remove(&task_id) {
            info!("Aborting execution of task {}", task_id);
            handle.abort();
            true
        } else {
            warn!(
                "Task {} not found in execution handles, may have already completed",
                task_id
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_result_debug() {
        let success = ExecutionResult::Success { logs: vec![] };
        assert!(format!("{:?}", success).contains("Success"));

        let failed = ExecutionResult::Failed {
            error: "test error".to_string(),
            logs: vec![],
        };
        assert!(format!("{:?}", failed).contains("Failed"));
        assert!(format!("{:?}", failed).contains("test error"));

        let cancelled = ExecutionResult::Cancelled;
        assert!(format!("{:?}", cancelled).contains("Cancelled"));
    }
}
