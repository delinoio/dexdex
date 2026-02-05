//! Local task executor for running AI agents in single-process mode.
//!
//! This module provides the `LocalExecutor` which wraps the core `TaskExecutor`
//! from the `coding_agents` crate with platform-specific event emission.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
pub use coding_agents::executor::ExecutionResult;
use coding_agents::{
    AgentResult, TimestampedEvent,
    executor::{
        AgentOutputEvent, EventEmitter, ExecutionResultWithWorktree, TaskCompletedEvent,
        TaskExecutionConfig, TaskExecutor, TaskStatusChangedEvent, TtyInputRequestEvent,
    },
};
use entities::{AgentSession, AiAgentType, CompositeTaskStatus, UnitTaskStatus};
use task_store::{SqliteTaskStore, TaskStore};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{TtyInputRequestManager, error::WorkerError, planning_prompt::build_planning_prompt};

/// A wrapper emitter that both emits events to an inner emitter AND accumulates
/// them for final persistence to the database when the task completes.
///
/// Events are delivered in real-time via the inner emitter (e.g., Tauri events)
/// and accumulated in memory. On task completion, all events are persisted to
/// the database so they can be loaded on page reload.
struct PersistingEventEmitter<E: EventEmitter> {
    /// The inner emitter for real-time event delivery.
    inner: Arc<E>,
    /// The task store for persisting logs.
    task_store: Arc<SqliteTaskStore>,
    /// The session ID to persist logs for.
    session_id: Uuid,
    /// Accumulated log lines (protected by mutex for thread-safe access).
    logs: Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl<E: EventEmitter> PersistingEventEmitter<E> {
    /// Creates a new persisting event emitter.
    fn new(inner: Arc<E>, task_store: Arc<SqliteTaskStore>, session_id: Uuid) -> Self {
        Self {
            inner,
            task_store,
            session_id,
            logs: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Persists the accumulated logs to the database.
    async fn persist_logs(&self) {
        let logs = self.logs.lock().await;
        if logs.is_empty() {
            return;
        }

        let output_log = logs.join("\n");

        if let Ok(Some(mut session)) = self.task_store.get_agent_session(self.session_id).await {
            session.output_log = Some(output_log.clone());
            if let Err(e) = self.task_store.update_agent_session(session).await {
                warn!(
                    "Failed to persist final logs for session {}: {}",
                    self.session_id, e
                );
            } else {
                debug!(
                    "Persisted {} bytes of final logs for session {}",
                    output_log.len(),
                    self.session_id
                );
            }
        }
    }
}

impl<E: EventEmitter> EventEmitter for PersistingEventEmitter<E> {
    fn emit_task_status_changed(&self, event: TaskStatusChangedEvent) -> AgentResult<()> {
        self.inner.emit_task_status_changed(event)
    }

    fn emit_agent_output(&self, event: AgentOutputEvent) -> AgentResult<()> {
        // Accumulate the event in the log buffer for final persistence
        let timestamped = TimestampedEvent {
            timestamp: Utc::now(),
            event: event.event.clone(),
        };
        match serde_json::to_string(&timestamped) {
            Ok(json) => {
                // Use try_lock to avoid blocking if possible.
                // If lock is contended, we use blocking_lock since this is called from
                // a sync context within an async runtime. The lock should be held briefly.
                let mut logs = match self.logs.try_lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        // Lock is contended, use blocking approach
                        // This is safe because we're in an async context and the lock
                        // holder should release quickly
                        self.logs.blocking_lock()
                    }
                };
                logs.push(json);
            }
            Err(e) => {
                warn!(
                    "Failed to serialize event for session {}: {}",
                    self.session_id, e
                );
            }
        }

        // Emit to the inner emitter for real-time delivery
        self.inner.emit_agent_output(event)
    }

    fn emit_tty_input_request(&self, event: TtyInputRequestEvent) -> AgentResult<()> {
        self.inner.emit_tty_input_request(event)
    }

    fn emit_task_completed(&self, event: TaskCompletedEvent) -> AgentResult<()> {
        self.inner.emit_task_completed(event)
    }
}

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

        // Create a persisting emitter that both emits events AND persists them
        // incrementally. This ensures events are available via polling even if
        // the frontend misses real-time events.
        let persisting_emitter = Arc::new(PersistingEventEmitter::new(
            emitter,
            task_store.clone(),
            session_id,
        ));
        let executor = TaskExecutor::with_repo_cache(repo_cache, persisting_emitter.clone());
        let execution_handles = self.execution_handles.clone();

        // Spawn the execution task and store the handle for cancellation support
        let handle = tokio::spawn(async move {
            let result = executor.execute_and_wait(config).await;

            // Final persist of all logs
            // Note: We use PersistingEventEmitter's logs as the source of truth.
            // result.logs() from TaskExecutor contains the same events, but
            // PersistingEventEmitter has already been handling incremental persistence.
            persisting_emitter.persist_logs().await;

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

    /// Checks if a task is currently being executed.
    pub async fn is_executing(&self, task_id: Uuid) -> bool {
        let handles = self.execution_handles.read().await;
        if let Some(handle) = handles.get(&task_id) {
            !handle.is_finished()
        } else {
            false
        }
    }

    /// Executes the planning phase of a composite task asynchronously.
    ///
    /// This spawns a background task that:
    /// 1. Creates a git worktree for the planning task
    /// 2. Runs the planning AI agent with the composite task prompt
    /// 3. Streams output events to the frontend
    /// 4. The planning agent will generate task graph nodes
    pub async fn execute_composite_task(&self, composite_task_id: Uuid) -> Result<(), String> {
        info!(
            "Starting planning execution for composite task: {}",
            composite_task_id
        );

        // Get the composite task from the store
        let composite_task = self
            .task_store
            .get_composite_task(composite_task_id)
            .await
            .map_err(|e| format!("Failed to get composite task: {}", e))?
            .ok_or_else(|| format!("Composite task not found: {}", composite_task_id))?;

        // Get the repository group to find repositories
        let repo_group = self
            .task_store
            .get_repository_group(composite_task.repository_group_id)
            .await
            .map_err(|e| format!("Failed to get repository group: {}", e))?
            .ok_or_else(|| {
                format!(
                    "Repository group not found: {}",
                    composite_task.repository_group_id
                )
            })?;

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

        // Get the planning agent task
        let agent_task = self
            .task_store
            .get_agent_task(composite_task.planning_task_id)
            .await
            .map_err(|e| format!("Failed to get planning agent task: {}", e))?
            .ok_or_else(|| {
                format!(
                    "Planning agent task not found: {}",
                    composite_task.planning_task_id
                )
            })?;

        let agent_type = agent_task.ai_agent_type.unwrap_or(AiAgentType::ClaudeCode);
        let agent_model = agent_task.ai_agent_model.clone();

        // Create an agent session for the planning task
        let mut session = AgentSession::new(composite_task.planning_task_id, agent_type);
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

        // Use a branch name for the composite planning
        let branch_name = format!("delidev/composite/{}", composite_task_id);

        // Build the full planning prompt with PLAN.yaml format instructions
        let planning_prompt = build_planning_prompt(&composite_task.prompt);

        // Create the execution config using the planning prompt
        let config = TaskExecutionConfig {
            task_id: composite_task_id,
            session_id,
            remote_url: repository.remote_url.clone(),
            branch_name: branch_name.clone(),
            agent_type,
            agent_model,
            prompt: planning_prompt,
        };

        // Clone values needed for the spawned task
        let task_store = self.task_store.clone();
        let emitter = self.emitter.clone();
        let repo_cache = self.executor.repo_cache().clone();

        // Create a persisting emitter for the planning session
        let persisting_emitter = Arc::new(PersistingEventEmitter::new(
            emitter,
            task_store.clone(),
            session_id,
        ));
        let executor = TaskExecutor::with_repo_cache(repo_cache, persisting_emitter.clone());
        let execution_handles = self.execution_handles.clone();

        // Store remote_url and branch_name for worktree cleanup
        let remote_url = repository.remote_url.clone();
        let branch_name_clone = branch_name.clone();

        // Spawn the planning execution task
        let handle = tokio::spawn(async move {
            // Execute without cleanup so we can read PLAN.yaml from the worktree
            let ExecutionResultWithWorktree {
                result,
                worktree_path,
            } = executor.execute_and_wait_without_cleanup(config).await;

            // Final persist of all logs
            persisting_emitter.persist_logs().await;

            // Update session completed_at
            if let Ok(Some(mut session)) = task_store.get_agent_session(session_id).await {
                session.completed_at = Some(Utc::now());
                if let Err(e) = task_store.update_agent_session(session).await {
                    warn!("Failed to update planning agent session: {}", e);
                }
            }

            // Update composite task status based on planning result
            match &result {
                ExecutionResult::Success { .. } => {
                    info!(
                        "Planning for composite task {} completed successfully",
                        composite_task_id
                    );

                    // Read PLAN.yaml from the worktree before cleanup
                    let plan_yaml_content = if let Some(ref worktree) = worktree_path {
                        Self::read_plan_yaml_from_worktree(worktree).await
                    } else {
                        warn!(
                            "No worktree path available for composite task {}",
                            composite_task_id
                        );
                        None
                    };

                    // Update composite task with plan_yaml and status
                    // If PLAN.yaml is missing, fail the task since planning didn't complete
                    // properly
                    if let Ok(Some(mut composite_task)) =
                        task_store.get_composite_task(composite_task_id).await
                    {
                        if plan_yaml_content.is_some() {
                            composite_task.status = CompositeTaskStatus::PendingApproval;
                            composite_task.plan_yaml = plan_yaml_content;
                            composite_task.updated_at = Utc::now();
                            if let Err(e) = task_store.update_composite_task(composite_task).await {
                                error!(
                                    "Failed to update composite task status to PendingApproval: {}",
                                    e
                                );
                            } else {
                                info!(
                                    "Persisted PLAN.yaml for composite task {}",
                                    composite_task_id
                                );
                            }
                        } else {
                            // PLAN.yaml not found - the planning agent didn't complete properly
                            error!(
                                "PLAN.yaml not found for composite task {} - marking as failed",
                                composite_task_id
                            );
                            composite_task.status = CompositeTaskStatus::Failed;
                            composite_task.updated_at = Utc::now();
                            if let Err(e) = task_store.update_composite_task(composite_task).await {
                                error!("Failed to update composite task status to Failed: {}", e);
                            }
                        }
                    }
                }
                ExecutionResult::Failed { error, .. } => {
                    error!(
                        "Planning for composite task {} failed: {}",
                        composite_task_id, error
                    );
                    // Update composite task status to Failed
                    if let Ok(Some(mut composite_task)) =
                        task_store.get_composite_task(composite_task_id).await
                    {
                        composite_task.status = CompositeTaskStatus::Failed;
                        composite_task.updated_at = Utc::now();
                        if let Err(e) = task_store.update_composite_task(composite_task).await {
                            error!("Failed to update composite task status to Failed: {}", e);
                        }
                    }
                }
                ExecutionResult::Cancelled => {
                    info!(
                        "Planning for composite task {} was cancelled",
                        composite_task_id
                    );
                    // Keep the status as Planning for potential retry
                }
            }

            // Clean up the planning worktree now that we've persisted the PLAN.yaml
            // Reuse executor's repo_cache directly instead of creating a new one
            if let Err(e) = executor.repo_cache().remove_worktree_for_task(
                &remote_url,
                &composite_task_id.to_string(),
                &branch_name_clone,
            ) {
                warn!(
                    "Failed to cleanup planning worktree for composite task {}: {}",
                    composite_task_id, e
                );
            } else {
                info!(
                    "Cleaned up planning worktree for composite task {}",
                    composite_task_id
                );
            }

            // Remove handle from the map after completion
            execution_handles.write().await.remove(&composite_task_id);
        });

        // Store the handle so it can be cancelled later
        self.execution_handles
            .write()
            .await
            .insert(composite_task_id, handle);

        Ok(())
    }

    /// Reads the PLAN.yaml file from a worktree directory.
    ///
    /// The planning agent generates a file named `PLAN-{random}.yaml` in the
    /// worktree root. This method finds and reads the first matching file.
    /// If multiple files match, they are sorted alphabetically for
    /// deterministic behavior.
    async fn read_plan_yaml_from_worktree(worktree_path: &Path) -> Option<String> {
        // Look for PLAN-*.yaml files in the worktree root
        let pattern = worktree_path.join("PLAN-*.yaml");
        let pattern_str = pattern.to_string_lossy();

        debug!("Looking for PLAN.yaml files matching: {}", pattern_str);

        // Use glob to find matching files
        match glob::glob(&pattern_str) {
            Ok(paths) => {
                // Collect all valid paths and sort them for deterministic behavior
                let mut valid_paths: Vec<PathBuf> = paths.filter_map(|entry| entry.ok()).collect();
                valid_paths.sort();

                if valid_paths.len() > 1 {
                    warn!(
                        "Multiple PLAN.yaml files found ({} files), using first (alphabetically): \
                         {:?}",
                        valid_paths.len(),
                        valid_paths.first()
                    );
                }

                for path in valid_paths {
                    debug!("Found PLAN.yaml file: {:?}", path);
                    match tokio::fs::read_to_string(&path).await {
                        Ok(content) => {
                            info!(
                                "Read PLAN.yaml content ({} bytes) from {:?}",
                                content.len(),
                                path
                            );
                            return Some(content);
                        }
                        Err(e) => {
                            warn!("Failed to read PLAN.yaml file {:?}: {}", path, e);
                        }
                    }
                }
                warn!("No PLAN.yaml files found matching pattern: {}", pattern_str);
                None
            }
            Err(e) => {
                warn!("Failed to glob for PLAN.yaml files: {}", e);
                None
            }
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
