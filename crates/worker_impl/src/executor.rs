//! Local task executor for running AI agents in single-process mode.
//!
//! This module provides the `LocalExecutor` which wraps the core `TaskExecutor`
//! from the `coding_agents` crate with platform-specific event emission.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
pub use coding_agents::executor::ExecutionResult;
use coding_agents::{
    AgentResult, NormalizedEvent, TimestampedEvent,
    executor::{
        AgentOutputEvent, EventEmitter, ExecutionResultWithWorktree, TaskCompletedEvent,
        TaskExecutionConfig, TaskExecutor, TaskStatusChangedEvent, TaskType, TtyInputRequestEvent,
    },
};
use entities::{
    AgentSession, AgentTask, AiAgentType, CompositeTaskNode, CompositeTaskStatus, UnitTask,
    UnitTaskStatus,
};
use plan_parser::{Plan, validate_plan};
use task_store::{SqliteTaskStore, TaskStore};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    TtyInputRequestManager,
    error::WorkerError,
    planning_prompt::{
        build_planning_prompt, build_update_planning_prompt, generate_plan_yaml_suffix,
        plan_yaml_filename,
    },
};

/// Maximum number of tasks allowed in a single composite task plan.
/// This prevents resource exhaustion from excessively large plans.
const MAX_TASKS_PER_PLAN: usize = 100;

/// Default polling interval in seconds for the composite task graph monitor.
const DEFAULT_GRAPH_MONITOR_INTERVAL_SECS: u64 = 3;

/// Maximum number of consecutive failures in the graph monitor before giving
/// up. This prevents infinite error loops that consume resources when the
/// database becomes persistently unavailable.
const MAX_CONSECUTIVE_MONITOR_FAILURES: u32 = 10;

/// Default timeout in seconds for the planning phase of a composite task.
/// If the planning agent doesn't complete within this time, the composite task
/// is marked as Failed. Defaults to 30 minutes.
const DEFAULT_PLANNING_TIMEOUT_SECS: u64 = 30 * 60;

/// A wrapper emitter that both emits events to an inner emitter AND persists
/// them to the database incrementally.
///
/// This ensures that events are available for initial log loading when the
/// frontend opens a task view after events have already been emitted.
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
        // Persist the event to the log buffer
        let timestamped = TimestampedEvent {
            timestamp: Utc::now(),
            event: event.event.clone(),
        };
        match serde_json::to_string(&timestamped) {
            Ok(json) => {
                let task_store = self.task_store.clone();
                let session_id = self.session_id;

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
                let logs_count = logs.len();

                // Spawn incremental persistence every 10 events
                if logs_count % 10 == 0 {
                    let output_log = logs.join("\n");
                    drop(logs); // Release lock before spawning
                    tokio::spawn(async move {
                        if let Ok(Some(mut session)) =
                            task_store.get_agent_session(session_id).await
                        {
                            session.output_log = Some(output_log);
                            if let Err(e) = task_store.update_agent_session(session).await {
                                warn!(
                                    "Failed to persist incremental logs for session {}: {}",
                                    session_id, e
                                );
                            } else {
                                debug!(
                                    "Incrementally persisted logs for session {} ({} events)",
                                    session_id, logs_count
                                );
                            }
                        }
                    });
                }
            }
            Err(e) => {
                warn!(
                    "Failed to serialize event for session {}: {}",
                    self.session_id, e
                );
            }
        }

        // Always emit to the inner emitter for real-time delivery
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
    /// Polling interval in seconds for the composite task graph monitor.
    graph_monitor_interval_secs: u64,
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
            graph_monitor_interval_secs: DEFAULT_GRAPH_MONITOR_INTERVAL_SECS,
        }
    }

    /// Returns the TTY request manager for responding to input requests.
    pub fn tty_request_manager(&self) -> Arc<TtyInputRequestManager> {
        self.executor.tty_request_manager()
    }

    /// Returns the event emitter for emitting task lifecycle events.
    pub fn emitter(&self) -> &Arc<E> {
        &self.emitter
    }

    /// Sets the polling interval in seconds for the composite task graph
    /// monitor.
    pub fn set_graph_monitor_interval_secs(&mut self, secs: u64) {
        self.graph_monitor_interval_secs = secs;
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
        let remote_url = repository.remote_url.clone();
        let branch_name_clone = branch_name.clone();

        // Create a persisting emitter that both emits events AND persists them
        // incrementally. This ensures events are available for initial log
        // loading when the frontend opens a task view.
        let persisting_emitter = Arc::new(PersistingEventEmitter::new(
            emitter,
            task_store.clone(),
            session_id,
        ));
        let executor = TaskExecutor::with_repo_cache(repo_cache, persisting_emitter.clone());
        let execution_handles = self.execution_handles.clone();

        // Spawn the execution task and store the handle for cancellation support
        let handle = tokio::spawn(async move {
            // Execute without cleanup so we can generate a git patch from the
            // worktree before deciding whether to clean it up.
            let exec_result = executor.execute_and_wait_without_cleanup(config).await;
            let result = exec_result.result;
            let worktree_path = exec_result.worktree_path;

            // Final persist of all logs
            // Note: We use PersistingEventEmitter's logs as the source of truth.
            // result.logs() from TaskExecutor contains the same events, but
            // PersistingEventEmitter has already been handling incremental persistence.
            persisting_emitter.persist_logs().await;

            // Generate git patch from the worktree if the task succeeded.
            // This captures all changes made by the AI agent so they can be
            // persisted in the database without needing write access to the
            // repository.
            let git_patch = if result.is_success() {
                if let Some(ref wt_path) = worktree_path {
                    match git_ops::generate_patch_async(wt_path).await {
                        Ok(patch) => {
                            if patch.is_some() {
                                info!(
                                    "Generated git patch for task {} ({} bytes)",
                                    task_id,
                                    patch.as_ref().map_or(0, |p| p.len())
                                );
                            } else {
                                debug!("No changes detected for task {}", task_id);
                            }
                            patch
                        }
                        Err(e) => {
                            warn!("Failed to generate git patch for task {}: {}", task_id, e);
                            None
                        }
                    }
                } else {
                    debug!(
                        "No worktree path available for task {}, skipping patch generation",
                        task_id
                    );
                    None
                }
            } else {
                None
            };

            // Update task status based on result and emit events so the
            // frontend updates without requiring a manual refresh.
            let (new_status, success, error_msg) = match &result {
                ExecutionResult::Success { .. } => {
                    info!("Task {} completed successfully", task_id);
                    (UnitTaskStatus::InReview, true, None)
                }
                ExecutionResult::Failed { error, .. } => {
                    error!("Task {} failed: {}", task_id, error);
                    (UnitTaskStatus::Failed, false, Some(error.clone()))
                }
                ExecutionResult::Cancelled => {
                    info!("Task {} was cancelled", task_id);
                    (UnitTaskStatus::Cancelled, false, None)
                }
            };

            // Retrieve the actual old status from the database before
            // updating, matching the pattern used by composite tasks. This
            // avoids hardcoding "in_progress" which would be wrong if the
            // task was re-executed from a different state (e.g. cancelled).
            let old_status = match task_store.get_unit_task(task_id).await {
                Ok(Some(t)) => serde_json::to_string(&t.status)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string(),
                _ => {
                    warn!(
                        "Could not fetch current status for unit task {}, defaulting to \
                         in_progress",
                        task_id
                    );
                    "in_progress".to_string()
                }
            };

            if let Err(e) = Self::update_task_status_with_patch(
                &task_store,
                task_id,
                session_id,
                new_status,
                git_patch,
            )
            .await
            {
                error!("Failed to update task status: {}", e);
            }

            // In local mode, preserve the worktree while the task is in
            // review so the user can inspect changes directly. Only clean
            // up if the task failed or was cancelled.
            let should_preserve_worktree = result.is_success();
            if !should_preserve_worktree {
                if let Err(e) = executor.repo_cache().remove_worktree_for_task(
                    &remote_url,
                    &task_id.to_string(),
                    &branch_name_clone,
                ) {
                    warn!(
                        "Failed to cleanup worktree for task {}: {}. Manual cleanup may be \
                         required.",
                        task_id, e
                    );
                } else {
                    info!("Cleaned up worktree for failed/cancelled task {}", task_id);
                }
            } else {
                info!(
                    "Preserving worktree for task {} (in review) for local inspection",
                    task_id
                );
            }

            // Always emit events regardless of whether the DB update
            // succeeded. This ensures the frontend is notified that the
            // session has ended even if the persistence layer had an error.
            let new_status_str = serde_json::to_string(&new_status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            if let Err(e) = persisting_emitter.emit_task_status_changed(TaskStatusChangedEvent {
                task_id: task_id.to_string(),
                task_type: TaskType::UnitTask,
                old_status,
                new_status: new_status_str,
            }) {
                warn!(
                    "Failed to emit status changed event for unit task {}: {}",
                    task_id, e
                );
            }

            // Emit task-completed so the frontend knows the session has
            // ended.
            if let Err(e) = persisting_emitter.emit_task_completed(TaskCompletedEvent {
                task_id: task_id.to_string(),
                task_type: TaskType::UnitTask,
                success,
                error: error_msg,
            }) {
                warn!(
                    "Failed to emit task completed event for unit task {}: {}",
                    task_id, e
                );
            }

            // Remove handle from the map after completion
            execution_handles.write().await.remove(&task_id);
        });

        // Store the handle so it can be cancelled later
        self.execution_handles.write().await.insert(task_id, handle);

        Ok(())
    }

    /// Updates a task's status and optionally stores a git patch.
    async fn update_task_status_with_patch(
        task_store: &Arc<SqliteTaskStore>,
        task_id: Uuid,
        session_id: Uuid,
        new_status: UnitTaskStatus,
        git_patch: Option<String>,
    ) -> Result<(), WorkerError> {
        let mut task = task_store
            .get_unit_task(task_id)
            .await?
            .ok_or_else(|| WorkerError::TaskNotFound(task_id.to_string()))?;

        task.status = new_status;
        task.updated_at = Utc::now();

        if git_patch.is_some() {
            task.git_patch = git_patch;
        }

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

    /// Returns `true` if the unit task status indicates the AI agent has
    /// successfully finished its work (InReview or later positive states).
    ///
    /// This is the single source of truth for "successful completion" checks,
    /// used by both the graph monitor and terminal-state logic to prevent
    /// drift.
    fn is_successfully_complete(status: &UnitTaskStatus) -> bool {
        matches!(
            status,
            UnitTaskStatus::InReview
                | UnitTaskStatus::Approved
                | UnitTaskStatus::PrOpen
                | UnitTaskStatus::Done
        )
    }

    /// Returns `true` if the unit task status is terminal (no further state
    /// transitions expected). Includes both successful and failed states.
    ///
    /// This is the single source of truth for "terminal state" checks, used
    /// by the graph monitor to determine when the composite task is complete.
    fn is_terminal_status(status: &UnitTaskStatus) -> bool {
        Self::is_successfully_complete(status)
            || matches!(
                status,
                UnitTaskStatus::Failed | UnitTaskStatus::Rejected | UnitTaskStatus::Cancelled
            )
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

        // Generate the plan YAML filename before creating the prompt, so the
        // agent is told exactly which file to create (instead of choosing its own
        // random suffix).
        let plan_suffix = generate_plan_yaml_suffix();
        let plan_filename = plan_yaml_filename(&plan_suffix);

        // Build the full planning prompt with PLAN.yaml format instructions.
        // If the task has update_plan_feedback, use the existing plan + feedback
        // instead of the original prompt.
        let planning_prompt = if let (Some(feedback), Some(existing_plan)) = (
            &composite_task.update_plan_feedback,
            &composite_task.plan_yaml,
        ) {
            if feedback.trim().is_empty() || existing_plan.trim().is_empty() {
                warn!(
                    "Empty feedback or plan for composite task {}, falling back to initial \
                     planning prompt",
                    composite_task_id
                );
                build_planning_prompt(&composite_task.prompt, &plan_filename)
            } else {
                info!(
                    "Using update planning prompt for composite task {}",
                    composite_task_id
                );
                build_update_planning_prompt(existing_plan, feedback, &plan_filename)
            }
        } else {
            info!(
                "Using initial planning prompt for composite task {}",
                composite_task_id
            );
            build_planning_prompt(&composite_task.prompt, &plan_filename)
        };

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

        // Store remote_url, branch_name, and plan_filename for the spawned task
        let remote_url = repository.remote_url;
        let branch_name_clone = branch_name;
        let plan_filename_clone = plan_filename;

        // Spawn the planning execution task with a timeout to prevent
        // indefinite hangs during the planning phase
        let planning_timeout = tokio::time::Duration::from_secs(DEFAULT_PLANNING_TIMEOUT_SECS);
        let handle = tokio::spawn(async move {
            // Execute without cleanup so we can read PLAN.yaml from the worktree.
            // Wrap in a timeout so a hung planning agent doesn't block forever.
            let timed_result = tokio::time::timeout(
                planning_timeout,
                executor.execute_and_wait_without_cleanup(config),
            )
            .await;

            let (result, worktree_path) = match timed_result {
                Ok(ExecutionResultWithWorktree {
                    result,
                    worktree_path,
                }) => (result, worktree_path),
                Err(_elapsed) => {
                    error!(
                        "Planning for composite task {} timed out after {} seconds",
                        composite_task_id, DEFAULT_PLANNING_TIMEOUT_SECS
                    );
                    (
                        ExecutionResult::Failed {
                            error: format!(
                                "Planning timed out after {} seconds",
                                DEFAULT_PLANNING_TIMEOUT_SECS
                            ),
                            logs: vec![],
                        },
                        None,
                    )
                }
            };

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
                        Self::read_plan_yaml_from_worktree(worktree, &plan_filename_clone).await
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
                            let old_status = serde_json::to_string(&composite_task.status)
                                .unwrap_or_default()
                                .trim_matches('"')
                                .to_string();
                            composite_task.status = CompositeTaskStatus::PendingApproval;
                            composite_task.plan_yaml = plan_yaml_content;
                            // Clear the update feedback now that re-planning is done
                            composite_task.update_plan_feedback = None;
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

                                // Emit task-status-changed event so the frontend updates
                                // automatically
                                if let Err(e) = persisting_emitter.emit_task_status_changed(
                                    TaskStatusChangedEvent {
                                        task_id: composite_task_id.to_string(),
                                        task_type: TaskType::CompositeTask,
                                        old_status,
                                        new_status: "pending_approval".to_string(),
                                    },
                                ) {
                                    warn!(
                                        "Failed to emit status changed event for composite task \
                                         {}: {}",
                                        composite_task_id, e
                                    );
                                }

                                // Emit task-completed event for the planning phase
                                if let Err(e) =
                                    persisting_emitter.emit_task_completed(TaskCompletedEvent {
                                        task_id: composite_task_id.to_string(),
                                        task_type: TaskType::CompositeTask,
                                        success: true,
                                        error: None,
                                    })
                                {
                                    warn!(
                                        "Failed to emit task completed event for composite task \
                                         {}: {}",
                                        composite_task_id, e
                                    );
                                }
                            }
                        } else {
                            // PLAN.yaml not found - the planning agent didn't complete properly
                            error!(
                                "PLAN.yaml not found for composite task {} - marking as failed",
                                composite_task_id
                            );
                            let old_status = serde_json::to_string(&composite_task.status)
                                .unwrap_or_default()
                                .trim_matches('"')
                                .to_string();
                            composite_task.status = CompositeTaskStatus::Failed;
                            // Clear the update feedback on failure too
                            composite_task.update_plan_feedback = None;
                            composite_task.updated_at = Utc::now();
                            if let Err(e) = task_store.update_composite_task(composite_task).await {
                                error!("Failed to update composite task status to Failed: {}", e);
                            } else {
                                // Emit status changed and task completed events
                                if let Err(e) = persisting_emitter.emit_task_status_changed(
                                    TaskStatusChangedEvent {
                                        task_id: composite_task_id.to_string(),
                                        task_type: TaskType::CompositeTask,
                                        old_status,
                                        new_status: "failed".to_string(),
                                    },
                                ) {
                                    warn!(
                                        "Failed to emit status changed event for composite task \
                                         {}: {}",
                                        composite_task_id, e
                                    );
                                }
                                if let Err(e) =
                                    persisting_emitter.emit_task_completed(TaskCompletedEvent {
                                        task_id: composite_task_id.to_string(),
                                        task_type: TaskType::CompositeTask,
                                        success: false,
                                        error: Some(
                                            "PLAN.yaml not found after planning completed"
                                                .to_string(),
                                        ),
                                    })
                                {
                                    warn!(
                                        "Failed to emit task completed event for composite task \
                                         {}: {}",
                                        composite_task_id, e
                                    );
                                }
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
                        let old_status = serde_json::to_string(&composite_task.status)
                            .unwrap_or_default()
                            .trim_matches('"')
                            .to_string();
                        composite_task.status = CompositeTaskStatus::Failed;
                        // Clear the update feedback on failure
                        composite_task.update_plan_feedback = None;
                        composite_task.updated_at = Utc::now();
                        if let Err(e) = task_store.update_composite_task(composite_task).await {
                            error!("Failed to update composite task status to Failed: {}", e);
                        } else {
                            // Emit status changed and task completed events
                            if let Err(e) = persisting_emitter.emit_task_status_changed(
                                TaskStatusChangedEvent {
                                    task_id: composite_task_id.to_string(),
                                    task_type: TaskType::CompositeTask,
                                    old_status,
                                    new_status: "failed".to_string(),
                                },
                            ) {
                                warn!(
                                    "Failed to emit status changed event for composite task {}: {}",
                                    composite_task_id, e
                                );
                            }
                            if let Err(e) =
                                persisting_emitter.emit_task_completed(TaskCompletedEvent {
                                    task_id: composite_task_id.to_string(),
                                    task_type: TaskType::CompositeTask,
                                    success: false,
                                    error: Some(error.clone()),
                                })
                            {
                                warn!(
                                    "Failed to emit task completed event for composite task {}: {}",
                                    composite_task_id, e
                                );
                            }
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
    /// The executor determines the exact filename (e.g., `PLAN-a1b2c3.yaml`)
    /// before creating the planning prompt, so we know which file to read.
    /// Falls back to glob matching if the exact file is not found, in case the
    /// agent created the file with a different name.
    async fn read_plan_yaml_from_worktree(
        worktree_path: &Path,
        plan_filename: &str,
    ) -> Option<String> {
        // First, try to read the exact file we told the agent to create
        let exact_path = worktree_path.join(plan_filename);
        debug!("Looking for plan YAML file: {:?}", exact_path);

        match tokio::fs::read_to_string(&exact_path).await {
            Ok(content) => {
                info!(
                    "Read PLAN.yaml content ({} bytes) from {:?}",
                    content.len(),
                    exact_path
                );
                return Some(content);
            }
            Err(e) => {
                warn!(
                    "Expected plan file {:?} not found ({}), falling back to glob search",
                    exact_path, e
                );
            }
        }

        // Fallback: glob for PLAN-*.yaml in case the agent used a different name
        let pattern = worktree_path.join("PLAN-*.yaml");
        let pattern_str = pattern.to_string_lossy();

        debug!(
            "Falling back to glob search for PLAN.yaml files: {}",
            pattern_str
        );

        match glob::glob(&pattern_str) {
            Ok(paths) => {
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

    /// Executes the task graph for a composite task after approval.
    ///
    /// This method:
    /// 1. Parses the `plan_yaml` from the composite task
    /// 2. Creates `UnitTask` and `CompositeTaskNode` records for each plan task
    /// 3. Starts executing tasks that have no dependencies (root tasks)
    /// 4. Spawns a monitoring task that checks for newly ready tasks as
    ///    dependencies complete
    pub async fn execute_composite_task_graph(
        self: &Arc<Self>,
        composite_task_id: Uuid,
    ) -> Result<(), String> {
        info!(
            "Starting composite task graph execution for: {}",
            composite_task_id
        );

        // Get the composite task
        let composite_task = self
            .task_store
            .get_composite_task(composite_task_id)
            .await
            .map_err(|e| format!("Failed to get composite task: {}", e))?
            .ok_or_else(|| format!("Composite task not found: {}", composite_task_id))?;

        // Parse plan_yaml
        let plan_yaml = composite_task
            .plan_yaml
            .as_ref()
            .ok_or_else(|| "Composite task has no plan_yaml".to_string())?;

        let plan =
            Plan::from_yaml(plan_yaml).map_err(|e| format!("Failed to parse plan YAML: {}", e))?;

        // Validate the plan for cycles, invalid dependencies, duplicate IDs,
        // etc.
        //
        // NOTE: This validation is intentionally duplicated here and in the
        // server's `approve_task` endpoint (`validate_composite_task_plan`).
        // The server validates on the API boundary for immediate user
        // feedback, while this validation guards the executor for the desktop
        // (Tauri) code path where approval bypasses the server.
        let validation = validate_plan(&plan);
        if !validation.is_valid() {
            let err_msg = format!("Plan validation failed: {:?}", validation.errors);
            error!(
                composite_task_id = %composite_task_id,
                errors = ?validation.errors,
                "Plan validation failed"
            );
            // Mark composite task as failed
            if let Ok(Some(mut ct)) = self.task_store.get_composite_task(composite_task_id).await {
                ct.status = CompositeTaskStatus::Failed;
                ct.updated_at = chrono::Utc::now();
                let _ = self.task_store.update_composite_task(ct).await;
            }
            return Err(err_msg);
        }

        // Check resource limits
        if plan.tasks.len() > MAX_TASKS_PER_PLAN {
            let err_msg = format!(
                "Plan has {} tasks, exceeding the maximum of {}",
                plan.tasks.len(),
                MAX_TASKS_PER_PLAN
            );
            error!(
                composite_task_id = %composite_task_id,
                task_count = plan.tasks.len(),
                max = MAX_TASKS_PER_PLAN,
                "Plan exceeds maximum task limit"
            );
            if let Ok(Some(mut ct)) = self.task_store.get_composite_task(composite_task_id).await {
                ct.status = CompositeTaskStatus::Failed;
                ct.updated_at = chrono::Utc::now();
                let _ = self.task_store.update_composite_task(ct).await;
            }
            return Err(err_msg);
        }

        info!(
            "Parsed plan with {} tasks for composite task {}",
            plan.tasks.len(),
            composite_task_id
        );

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

        let repo_id = repo_group
            .repository_ids
            .first()
            .ok_or_else(|| "Repository group has no repositories".to_string())?;

        // Verify repository exists
        self.task_store
            .get_repository(*repo_id)
            .await
            .map_err(|e| format!("Failed to get repository: {}", e))?
            .ok_or_else(|| format!("Repository not found: {}", repo_id))?;

        let agent_type = composite_task
            .execution_agent_type
            .unwrap_or(AiAgentType::ClaudeCode);

        // Create all nodes with cleanup on error. If any node creation fails,
        // we delete all previously created nodes and unit tasks to avoid orphaned
        // records.
        let create_result = self
            .create_task_graph_nodes(
                composite_task_id,
                composite_task.repository_group_id,
                agent_type,
                &plan,
            )
            .await;

        let (node_ids, plan_id_to_unit_task_id) = match create_result {
            Ok(result) => result,
            Err(e) => {
                error!(
                    "Failed to create task graph nodes for composite task {}: {}",
                    composite_task_id, e
                );
                // Mark composite task as failed
                if let Ok(Some(mut ct)) =
                    self.task_store.get_composite_task(composite_task_id).await
                {
                    ct.status = CompositeTaskStatus::Failed;
                    ct.updated_at = chrono::Utc::now();
                    let _ = self.task_store.update_composite_task(ct).await;
                }
                return Err(e);
            }
        };

        // Update composite task with node_ids
        let mut composite_task = self
            .task_store
            .get_composite_task(composite_task_id)
            .await
            .map_err(|e| format!("Failed to get composite task: {}", e))?
            .ok_or_else(|| format!("Composite task not found: {}", composite_task_id))?;

        composite_task.node_ids = node_ids;
        composite_task.updated_at = chrono::Utc::now();
        self.task_store
            .update_composite_task(composite_task)
            .await
            .map_err(|e| format!("Failed to update composite task node_ids: {}", e))?;

        // Emit task-status-changed so the frontend refreshes the composite task
        // detail view and picks up the newly created nodes and graph data.
        if let Err(e) = self
            .emitter
            .emit_task_status_changed(TaskStatusChangedEvent {
                task_id: composite_task_id.to_string(),
                task_type: TaskType::CompositeTask,
                old_status: "in_progress".to_string(),
                new_status: "in_progress".to_string(),
            })
        {
            warn!(
                "Failed to emit status changed event after node creation for composite task {}: {}",
                composite_task_id, e
            );
        }

        // Find root tasks (no dependencies) and start them
        let root_unit_task_ids: Vec<Uuid> = plan
            .tasks
            .iter()
            .filter(|t| t.depends_on.is_empty())
            .filter_map(|t| plan_id_to_unit_task_id.get(&t.id).copied())
            .collect();

        info!(
            "Starting {} root tasks for composite task {}",
            root_unit_task_ids.len(),
            composite_task_id
        );

        for unit_task_id in &root_unit_task_ids {
            // Guard against duplicate execution (idempotency). This is
            // defensive: root tasks are freshly created so they shouldn't be
            // running yet, but the check prevents issues if
            // execute_composite_task_graph is called twice for the same task.
            if self.is_executing(*unit_task_id).await {
                warn!(
                    "Root unit task {} is already executing, skipping",
                    unit_task_id
                );
                continue;
            }
            if let Err(e) = self.execute_unit_task(*unit_task_id).await {
                error!(
                    "Failed to start unit task {} for composite task {}: {}",
                    unit_task_id, composite_task_id, e
                );
            }
        }

        // Spawn a monitoring task that periodically checks for newly ready
        // tasks and starts them. This runs until all tasks are complete.
        let executor = Arc::clone(self);
        let monitor_interval_secs = self.graph_monitor_interval_secs;
        tokio::spawn(async move {
            Self::monitor_composite_task_graph(executor, composite_task_id, monitor_interval_secs)
                .await;
        });

        Ok(())
    }

    /// Creates AgentTask + UnitTask + CompositeTaskNode records for each plan
    /// task. If any creation fails, cleans up all previously created records
    /// to avoid orphaned data.
    ///
    /// Returns `(node_ids, plan_id_to_unit_task_id)` on success.
    async fn create_task_graph_nodes(
        &self,
        composite_task_id: Uuid,
        repository_group_id: Uuid,
        agent_type: AiAgentType,
        plan: &Plan,
    ) -> Result<(Vec<Uuid>, HashMap<String, Uuid>), String> {
        let mut plan_id_to_node_id: HashMap<String, Uuid> = HashMap::new();
        let mut node_ids: Vec<Uuid> = Vec::new();
        let mut plan_id_to_unit_task_id: HashMap<String, Uuid> = HashMap::new();
        // Track created records for cleanup on error
        let mut created_node_ids: Vec<Uuid> = Vec::new();
        let mut created_unit_task_ids: Vec<Uuid> = Vec::new();
        let mut created_agent_task_ids: Vec<Uuid> = Vec::new();

        // First pass: Create AgentTask + UnitTask + CompositeTaskNode for each
        // plan task (without dependencies set yet)
        let first_pass_result: Result<(), String> = async {
            for plan_task in &plan.tasks {
                // Create agent task
                let mut agent_task = AgentTask::new();
                agent_task.ai_agent_type = Some(agent_type);
                let agent_task = self
                    .task_store
                    .create_agent_task(agent_task)
                    .await
                    .map_err(|e| format!("Failed to create agent task: {}", e))?;
                created_agent_task_ids.push(agent_task.id);

                // Create unit task
                let mut unit_task =
                    UnitTask::new(repository_group_id, agent_task.id, &plan_task.prompt);
                if let Some(title) = &plan_task.title {
                    unit_task = unit_task.with_title(title);
                }
                if let Some(branch_name) = &plan_task.branch_name {
                    unit_task = unit_task.with_branch_name(branch_name);
                }

                let unit_task = self
                    .task_store
                    .create_unit_task(unit_task)
                    .await
                    .map_err(|e| format!("Failed to create unit task: {}", e))?;
                created_unit_task_ids.push(unit_task.id);

                // Create composite task node
                let node = CompositeTaskNode::new(composite_task_id, unit_task.id);
                let node = self
                    .task_store
                    .create_composite_task_node(node)
                    .await
                    .map_err(|e| format!("Failed to create composite task node: {}", e))?;
                created_node_ids.push(node.id);

                info!(
                    "Created node {} (unit task {}) for plan task '{}'",
                    node.id, unit_task.id, plan_task.id
                );

                plan_id_to_node_id.insert(plan_task.id.clone(), node.id);
                plan_id_to_unit_task_id.insert(plan_task.id.clone(), unit_task.id);
                node_ids.push(node.id);
            }
            Ok(())
        }
        .await;

        if let Err(e) = first_pass_result {
            self.cleanup_created_records(
                &created_node_ids,
                &created_unit_task_ids,
                &created_agent_task_ids,
            )
            .await;
            return Err(e);
        }

        // Second pass: Set dependencies on nodes
        let second_pass_result: Result<(), String> = async {
            for plan_task in &plan.tasks {
                if plan_task.depends_on.is_empty() {
                    continue;
                }

                let node_id = plan_id_to_node_id[&plan_task.id];
                let mut node = self
                    .task_store
                    .get_composite_task_node(node_id)
                    .await
                    .map_err(|e| format!("Failed to get composite task node: {}", e))?
                    .ok_or_else(|| format!("Composite task node not found: {}", node_id))?;

                for dep_plan_id in &plan_task.depends_on {
                    let dep_node_id = plan_id_to_node_id.get(dep_plan_id).ok_or_else(|| {
                        format!("Dependency plan task not found: {}", dep_plan_id)
                    })?;
                    node.depends_on(dep_node_id.to_owned());
                }

                self.task_store
                    .update_composite_task_node(node)
                    .await
                    .map_err(|e| format!("Failed to update composite task node deps: {}", e))?;
            }
            Ok(())
        }
        .await;

        if let Err(e) = second_pass_result {
            self.cleanup_created_records(
                &created_node_ids,
                &created_unit_task_ids,
                &created_agent_task_ids,
            )
            .await;
            return Err(e);
        }

        Ok((node_ids, plan_id_to_unit_task_id))
    }

    /// Best-effort cleanup of records created during a failed node creation.
    async fn cleanup_created_records(
        &self,
        node_ids: &[Uuid],
        unit_task_ids: &[Uuid],
        agent_task_ids: &[Uuid],
    ) {
        warn!(
            "Cleaning up {} nodes, {} unit tasks, {} agent tasks after failed node creation",
            node_ids.len(),
            unit_task_ids.len(),
            agent_task_ids.len()
        );

        for id in node_ids {
            if let Err(e) = self.task_store.delete_composite_task_node(*id).await {
                warn!("Failed to cleanup composite task node {}: {}", id, e);
            }
        }
        for id in unit_task_ids {
            if let Err(e) = self.task_store.delete_unit_task(*id).await {
                warn!("Failed to cleanup unit task {}: {}", id, e);
            }
        }
        for id in agent_task_ids {
            if let Err(e) = self.task_store.delete_agent_task(*id).await {
                warn!("Failed to cleanup agent task {}: {}", id, e);
            }
        }
    }

    /// Monitors a composite task graph and starts dependent tasks as their
    /// dependencies complete. Runs until all tasks reach a terminal state.
    ///
    /// Uses a `started_tasks` set to prevent the race condition where multiple
    /// monitor iterations could start the same task before it registers in
    /// `execution_handles`. The set is checked atomically with the start call
    /// to guarantee each task is started at most once.
    async fn monitor_composite_task_graph(
        executor: Arc<Self>,
        composite_task_id: Uuid,
        monitor_interval_secs: u64,
    ) {
        info!(
            "Starting graph monitor for composite task {}",
            composite_task_id
        );

        // Track which tasks we have already started to prevent duplicate
        // executions across monitor iterations (solves the race between
        // checking `is_executing()` and actually calling `execute_unit_task`).
        let mut started_tasks: HashSet<Uuid> = HashSet::new();

        // Count consecutive failures to bail out if the database becomes
        // persistently unavailable.
        let mut consecutive_failures: u32 = 0;

        loop {
            // Wait before checking again (configurable interval)
            tokio::time::sleep(tokio::time::Duration::from_secs(monitor_interval_secs)).await;

            // Get all nodes for this composite task
            let nodes = match executor
                .task_store
                .list_composite_task_nodes(composite_task_id)
                .await
            {
                Ok(nodes) => {
                    // Reset failure counter on success
                    consecutive_failures = 0;
                    nodes
                }
                Err(e) => {
                    consecutive_failures += 1;
                    error!(
                        "Graph monitor: Failed to list nodes for composite task {} (consecutive \
                         failure {}/{}): {}",
                        composite_task_id,
                        consecutive_failures,
                        MAX_CONSECUTIVE_MONITOR_FAILURES,
                        e
                    );
                    if consecutive_failures >= MAX_CONSECUTIVE_MONITOR_FAILURES {
                        error!(
                            "Graph monitor: Giving up after {} consecutive failures for composite \
                             task {}. Marking as failed.",
                            MAX_CONSECUTIVE_MONITOR_FAILURES, composite_task_id
                        );
                        if let Ok(Some(mut ct)) = executor
                            .task_store
                            .get_composite_task(composite_task_id)
                            .await
                        {
                            let old_status = serde_json::to_string(&ct.status)
                                .unwrap_or_default()
                                .trim_matches('"')
                                .to_string();
                            ct.status = CompositeTaskStatus::Failed;
                            ct.updated_at = chrono::Utc::now();
                            if executor.task_store.update_composite_task(ct).await.is_ok() {
                                // Emit task-status-changed so the frontend updates
                                if let Err(e) = executor.emitter.emit_task_status_changed(
                                    TaskStatusChangedEvent {
                                        task_id: composite_task_id.to_string(),
                                        task_type: TaskType::CompositeTask,
                                        old_status,
                                        new_status: "failed".to_string(),
                                    },
                                ) {
                                    warn!(
                                        "Graph monitor: Failed to emit status changed event for \
                                         composite task {}: {}",
                                        composite_task_id, e
                                    );
                                }
                            }
                        }
                        break;
                    }
                    continue;
                }
            };

            if nodes.is_empty() {
                debug!(
                    "Graph monitor: No nodes found for composite task {}",
                    composite_task_id
                );
                break;
            }

            // Build a mapping from node_id to its unit_task status
            let mut node_statuses: HashMap<Uuid, UnitTaskStatus> = HashMap::new();
            for node in &nodes {
                match executor.task_store.get_unit_task(node.unit_task_id).await {
                    Ok(Some(ut)) => {
                        node_statuses.insert(node.id, ut.status);
                    }
                    Ok(None) => {
                        warn!(
                            "Graph monitor: Unit task {} not found for node {}",
                            node.unit_task_id, node.id
                        );
                    }
                    Err(e) => {
                        error!(
                            "Graph monitor: Failed to get unit task {} for node {}: {}",
                            node.unit_task_id, node.id, e
                        );
                    }
                }
            }

            // Find dependent nodes whose dependencies are all complete but
            // haven't started yet.
            //
            // NOTE: Root tasks (nodes with empty depends_on_ids) are excluded
            // here because they are started separately during graph
            // initialization in `execute_composite_task_graph`. Only tasks
            // with dependencies need to be monitored for readiness.
            let mut newly_ready: Vec<Uuid> = Vec::new();
            for node in &nodes {
                let status = match node_statuses.get(&node.id) {
                    Some(s) => s,
                    None => continue,
                };

                // Only consider tasks that are InProgress (initial state) and
                // not yet being executed
                if *status != UnitTaskStatus::InProgress {
                    continue;
                }

                // Skip root tasks — they are started during initialization,
                // not by the monitor.
                if node.depends_on_ids.is_empty() {
                    continue;
                }

                // Use the started_tasks set as the primary guard against
                // duplicate starts. This is checked before is_executing() to
                // avoid the TOCTOU race between checking the execution handles
                // and actually spawning the task.
                if started_tasks.contains(&node.unit_task_id) {
                    continue;
                }

                // Also check is_executing as a secondary guard (e.g., if a
                // task was started outside this monitor)
                if executor.is_executing(node.unit_task_id).await {
                    started_tasks.insert(node.unit_task_id);
                    continue;
                }

                // Check if all dependencies are successfully complete
                let all_deps_complete = node.depends_on_ids.iter().all(|dep_id| {
                    node_statuses
                        .get(dep_id)
                        .is_some_and(|s| Self::is_successfully_complete(s))
                });

                if all_deps_complete {
                    newly_ready.push(node.unit_task_id);
                }
            }

            // Start newly ready tasks, marking them in the started set
            // *before* calling execute to prevent the next iteration from
            // starting the same task.
            for unit_task_id in &newly_ready {
                started_tasks.insert(*unit_task_id);
                info!(
                    "Graph monitor: Starting dependent task {} for composite task {}",
                    unit_task_id, composite_task_id
                );
                if let Err(e) = executor.execute_unit_task(*unit_task_id).await {
                    error!(
                        "Graph monitor: Failed to start unit task {} for composite task {}: {}",
                        unit_task_id, composite_task_id, e
                    );
                }
            }

            // Check if all tasks are in a terminal state using the shared
            // helper to stay in sync with `is_successfully_complete`.
            let all_terminal = node_statuses
                .values()
                .all(|status| Self::is_terminal_status(status));

            if all_terminal && newly_ready.is_empty() {
                // All tasks reached terminal state - update composite task
                // status
                let any_failed = node_statuses.values().any(|status| {
                    matches!(
                        status,
                        UnitTaskStatus::Failed
                            | UnitTaskStatus::Rejected
                            | UnitTaskStatus::Cancelled
                    )
                });

                let new_status = if any_failed {
                    CompositeTaskStatus::Failed
                } else {
                    CompositeTaskStatus::Done
                };

                info!(
                    "Graph monitor: All tasks in composite task {} are terminal, setting status \
                     to {:?}",
                    composite_task_id, new_status
                );

                if let Ok(Some(mut ct)) = executor
                    .task_store
                    .get_composite_task(composite_task_id)
                    .await
                {
                    let old_status = serde_json::to_string(&ct.status)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string();
                    ct.status = new_status;
                    ct.updated_at = chrono::Utc::now();
                    if let Err(e) = executor.task_store.update_composite_task(ct).await {
                        error!(
                            "Graph monitor: Failed to update composite task {} status: {}",
                            composite_task_id, e
                        );
                    } else {
                        // Emit task-status-changed so the frontend updates
                        let new_status_str = serde_json::to_string(&new_status)
                            .unwrap_or_default()
                            .trim_matches('"')
                            .to_string();
                        if let Err(e) =
                            executor
                                .emitter
                                .emit_task_status_changed(TaskStatusChangedEvent {
                                    task_id: composite_task_id.to_string(),
                                    task_type: TaskType::CompositeTask,
                                    old_status,
                                    new_status: new_status_str,
                                })
                        {
                            warn!(
                                "Graph monitor: Failed to emit status changed event for composite \
                                 task {}: {}",
                                composite_task_id, e
                            );
                        }
                    }
                }

                info!(
                    "Graph monitor: Composite task {} completed with status {:?}",
                    composite_task_id, new_status
                );
                break;
            }
        }
    }

    /// Executes a subtask for an existing unit task.
    ///
    /// A subtask is an agent session that runs within the existing worktree of
    /// a parent unit task. It is used for operations like "Create a PR" or
    /// "Commit to local" where the AI agent needs to perform an action on the
    /// already-completed work.
    ///
    /// Subtasks:
    /// - Belong to the parent unit task's AgentTask
    /// - Run in the existing worktree (no new worktree is created)
    /// - Are not shown in the dashboard task list
    /// - Update the parent task status on completion
    ///
    /// Returns the result of the subtask execution.
    pub async fn execute_subtask(
        &self,
        task_id: Uuid,
        prompt: String,
        target_status: UnitTaskStatus,
    ) -> Result<(), String> {
        info!(
            "Starting subtask execution for unit task: {} (target: {:?})",
            task_id, target_status
        );

        // Get the task from the store
        let task = self
            .task_store
            .get_unit_task(task_id)
            .await
            .map_err(|e| format!("Failed to get task: {}", e))?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        if task.status != UnitTaskStatus::Approved {
            return Err(format!(
                "Task {} is not in Approved status (current: {:?})",
                task_id, task.status
            ));
        }

        // Get the repository group to find repositories
        let repo_group = self
            .task_store
            .get_repository_group(task.repository_group_id)
            .await
            .map_err(|e| format!("Failed to get repository group: {}", e))?
            .ok_or_else(|| format!("Repository group not found: {}", task.repository_group_id))?;

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

        // Get the agent task
        let agent_task = self
            .task_store
            .get_agent_task(task.agent_task_id)
            .await
            .map_err(|e| format!("Failed to get agent task: {}", e))?
            .ok_or_else(|| format!("Agent task not found: {}", task.agent_task_id))?;

        let agent_type = agent_task.ai_agent_type.unwrap_or(AiAgentType::ClaudeCode);
        let agent_model = agent_task.ai_agent_model.clone();

        // Create a new agent session under the same agent task
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

        // Determine branch name (same as the parent task)
        let branch_name = task
            .branch_name
            .clone()
            .unwrap_or_else(|| format!("delidev/{}", task_id));

        // Find the existing worktree path
        let worktree_path = git_ops::worktree_path_for_task_with_cache(
            self.executor.repo_cache().worktrees_dir(),
            &task_id.to_string(),
            &branch_name,
        );

        if !worktree_path.exists() {
            return Err(format!(
                "Worktree not found for task {}. The worktree may have been cleaned up.",
                task_id
            ));
        }

        info!("Subtask will run in existing worktree: {:?}", worktree_path);

        // Transition task status to InProgress while the subtask runs.
        // Re-use the task fetched above to avoid a redundant database query and
        // reduce the race window between read and write.
        let old_status = "approved".to_string();
        {
            let mut task = task.clone();
            task.status = UnitTaskStatus::InProgress;
            task.updated_at = Utc::now();
            self.task_store
                .update_unit_task(task)
                .await
                .map_err(|e| format!("Failed to update task status: {}", e))?;
        }

        // Emit status change to InProgress
        if let Err(e) = self
            .emitter
            .emit_task_status_changed(TaskStatusChangedEvent {
                task_id: task_id.to_string(),
                task_type: TaskType::UnitTask,
                old_status: old_status.clone(),
                new_status: "in_progress".to_string(),
            })
        {
            warn!(
                "Failed to emit status changed event for subtask {}: {}",
                task_id, e
            );
        }

        // Create the execution config
        let config = TaskExecutionConfig {
            task_id,
            session_id,
            remote_url: repository.remote_url.clone(),
            branch_name: branch_name.clone(),
            agent_type,
            agent_model,
            prompt,
        };

        // Clone values needed for the spawned task
        let task_store = self.task_store.clone();
        let emitter = self.emitter.clone();
        let tty_manager = self.executor.tty_request_manager();
        let execution_handles = self.execution_handles.clone();

        // Create a persisting emitter for the subtask session
        let persisting_emitter = Arc::new(PersistingEventEmitter::new(
            emitter.clone(),
            task_store.clone(),
            session_id,
        ));

        // Spawn the subtask execution
        let handle = tokio::spawn(async move {
            let result = TaskExecutor::<PersistingEventEmitter<E>>::run_agent_in_worktree(
                config,
                persisting_emitter.clone(),
                tty_manager,
                worktree_path,
            )
            .await;

            // Final persist of all logs
            persisting_emitter.persist_logs().await;

            // Update session completed_at
            if let Ok(Some(mut session)) = task_store.get_agent_session(session_id).await {
                session.completed_at = Some(Utc::now());
                if let Err(e) = task_store.update_agent_session(session).await {
                    warn!("Failed to update subtask agent session: {}", e);
                }
            }

            // Update task status based on result
            let (new_status, success, error_msg) = match &result {
                ExecutionResult::Success { .. } => {
                    info!("Subtask for task {} completed successfully", task_id);
                    (target_status, true, None)
                }
                ExecutionResult::Failed { error, .. } => {
                    error!("Subtask for task {} failed: {}", task_id, error);
                    // On failure, revert to Approved so the user can retry
                    (UnitTaskStatus::Approved, false, Some(error.clone()))
                }
                ExecutionResult::Cancelled => {
                    info!("Subtask for task {} was cancelled", task_id);
                    (UnitTaskStatus::Approved, false, None)
                }
            };

            // Update task status and extract PR URL if applicable
            if let Ok(Some(mut task)) = task_store.get_unit_task(task_id).await {
                task.status = new_status;
                task.updated_at = Utc::now();

                // For PR creation subtasks, extract the PR URL from agent output
                if success && target_status == UnitTaskStatus::PrOpen {
                    if let Some(pr_url) = extract_pr_url_from_logs(result.logs()) {
                        info!("Extracted PR URL for task {}: {}", task_id, pr_url);
                        task.linked_pr_url = Some(pr_url);
                    } else {
                        warn!(
                            "PR creation subtask for task {} succeeded but no PR URL found in logs",
                            task_id
                        );
                    }
                }

                if let Err(e) = task_store.update_unit_task(task).await {
                    error!("Failed to update task status after subtask: {}", e);
                }
            }

            // Emit status change events
            let new_status_str = serde_json::to_string(&new_status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            if let Err(e) = persisting_emitter.emit_task_status_changed(TaskStatusChangedEvent {
                task_id: task_id.to_string(),
                task_type: TaskType::UnitTask,
                old_status: "in_progress".to_string(),
                new_status: new_status_str,
            }) {
                warn!(
                    "Failed to emit status changed event for subtask {}: {}",
                    task_id, e
                );
            }

            if let Err(e) = persisting_emitter.emit_task_completed(TaskCompletedEvent {
                task_id: task_id.to_string(),
                task_type: TaskType::UnitTask,
                success,
                error: error_msg,
            }) {
                warn!(
                    "Failed to emit task completed event for subtask {}: {}",
                    task_id, e
                );
            }

            // Remove handle from the map after completion
            execution_handles.write().await.remove(&task_id);
        });

        // Store the handle so it can be cancelled later
        self.execution_handles.write().await.insert(task_id, handle);

        Ok(())
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

/// Extracts a PR URL from agent output logs.
///
/// Scans the JSON-serialized log lines for `TextOutput` events containing
/// a GitHub/GitLab/Bitbucket PR URL. Returns the first PR URL found, or
/// `None` if no URL is present.
fn extract_pr_url_from_logs(logs: &[String]) -> Option<String> {
    for log_line in logs {
        // Each log line is a JSON-serialized TimestampedEvent
        if let Ok(timestamped) = serde_json::from_str::<TimestampedEvent>(log_line) {
            let content = match &timestamped.event {
                NormalizedEvent::TextOutput { content, .. } => Some(content.as_str()),
                NormalizedEvent::ToolResult { output, .. } => output.as_str(),
                _ => None,
            };

            if let Some(url) = content.and_then(find_pr_url_in_text) {
                return Some(url);
            }
        }
    }
    None
}

/// Searches text for a pull request / merge request URL.
///
/// Supported patterns:
/// - GitHub: `https://github.com/{owner}/{repo}/pull/{number}`
/// - GitLab: `https://gitlab.com/{owner}/{repo}/-/merge_requests/{number}`
/// - Bitbucket: `https://bitbucket.org/{owner}/{repo}/pull-requests/{number}`
fn find_pr_url_in_text(text: &str) -> Option<String> {
    // Split by whitespace and common delimiters to find URL tokens
    for token in text.split(|c: char| c.is_whitespace() || ['"', '\'', '<', '>'].contains(&c)) {
        let token = token.trim_end_matches(['.', ',', ')', ']']);
        if token.starts_with("https://github.com/") && token.contains("/pull/") {
            // Validate it looks like a proper GitHub PR URL
            let parts: Vec<&str> = token.splitn(7, '/').collect();
            // https: / / github.com / owner / repo / pull / number
            let pr_num = parts
                .get(6)
                .and_then(|s| s.split('/').next())
                .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
            if let Some(pr_num) = pr_num {
                // Reconstruct the canonical URL up to the PR number
                return Some(format!(
                    "https://github.com/{}/{}/pull/{}",
                    parts[3], parts[4], pr_num
                ));
            }
        } else if (token.starts_with("https://gitlab.com/") && token.contains("/-/merge_requests/"))
            || (token.starts_with("https://bitbucket.org/") && token.contains("/pull-requests/"))
        {
            return Some(token.to_string());
        }
    }
    None
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

    /// Verifies that UnitTaskStatus variants serialize to the snake_case
    /// strings expected by the frontend event handlers.
    #[test]
    fn test_unit_task_status_serialization_for_events() {
        // Helper that mirrors the serialization pattern used in
        // execute_unit_task when building TaskStatusChangedEvent payloads.
        fn status_to_event_string(status: UnitTaskStatus) -> String {
            serde_json::to_string(&status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string()
        }

        assert_eq!(
            status_to_event_string(UnitTaskStatus::InProgress),
            "in_progress"
        );
        assert_eq!(
            status_to_event_string(UnitTaskStatus::InReview),
            "in_review"
        );
        assert_eq!(status_to_event_string(UnitTaskStatus::Approved), "approved");
        assert_eq!(status_to_event_string(UnitTaskStatus::PrOpen), "pr_open");
        assert_eq!(status_to_event_string(UnitTaskStatus::Done), "done");
        assert_eq!(status_to_event_string(UnitTaskStatus::Rejected), "rejected");
        assert_eq!(status_to_event_string(UnitTaskStatus::Failed), "failed");
        assert_eq!(
            status_to_event_string(UnitTaskStatus::Cancelled),
            "cancelled"
        );
    }

    /// Verifies is_terminal_status correctly identifies all terminal states.
    #[test]
    fn test_is_terminal_status() {
        // Terminal states
        assert!(LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::InReview
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::Approved
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::PrOpen
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::Done
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::Failed
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::Rejected
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::Cancelled
        ));

        // Non-terminal states
        assert!(!LocalExecutor::<DummyEmitter>::is_terminal_status(
            &UnitTaskStatus::InProgress
        ));
    }

    /// Verifies is_successfully_complete only includes positive completion
    /// states.
    #[test]
    fn test_is_successfully_complete() {
        // Successful completions
        assert!(LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::InReview
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::Approved
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::PrOpen
        ));
        assert!(LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::Done
        ));

        // Not successful completions
        assert!(!LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::InProgress
        ));
        assert!(!LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::Failed
        ));
        assert!(!LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::Rejected
        ));
        assert!(!LocalExecutor::<DummyEmitter>::is_successfully_complete(
            &UnitTaskStatus::Cancelled
        ));
    }

    /// A minimal EventEmitter implementation for testing static methods on
    /// LocalExecutor that don't actually emit events.
    struct DummyEmitter;

    impl EventEmitter for DummyEmitter {
        fn emit_task_status_changed(&self, _event: TaskStatusChangedEvent) -> AgentResult<()> {
            Ok(())
        }

        fn emit_agent_output(&self, _event: AgentOutputEvent) -> AgentResult<()> {
            Ok(())
        }

        fn emit_tty_input_request(&self, _event: TtyInputRequestEvent) -> AgentResult<()> {
            Ok(())
        }

        fn emit_task_completed(&self, _event: TaskCompletedEvent) -> AgentResult<()> {
            Ok(())
        }
    }

    // =========================================================================
    // PR URL Extraction Tests
    // =========================================================================

    /// Helper to create a JSON-serialized TextOutput log line.
    fn make_text_log(content: &str) -> String {
        let event = TimestampedEvent {
            timestamp: Utc::now(),
            event: NormalizedEvent::TextOutput {
                content: content.to_string(),
                stream: false,
            },
        };
        serde_json::to_string(&event).unwrap()
    }

    /// Helper to create a JSON-serialized ToolResult log line.
    fn make_tool_result_log(output: &str) -> String {
        let event = TimestampedEvent {
            timestamp: Utc::now(),
            event: NormalizedEvent::ToolResult {
                tool_name: "bash".to_string(),
                output: serde_json::Value::String(output.to_string()),
                is_error: false,
            },
        };
        serde_json::to_string(&event).unwrap()
    }

    #[test]
    fn test_find_pr_url_github() {
        let url = find_pr_url_in_text("Created PR: https://github.com/delinoio/delidev/pull/123");
        assert_eq!(
            url,
            Some("https://github.com/delinoio/delidev/pull/123".to_string())
        );
    }

    #[test]
    fn test_find_pr_url_github_in_sentence() {
        let url = find_pr_url_in_text(
            "I've created the pull request at https://github.com/owner/repo/pull/42. Please \
             review it.",
        );
        assert_eq!(
            url,
            Some("https://github.com/owner/repo/pull/42".to_string())
        );
    }

    #[test]
    fn test_find_pr_url_no_url() {
        assert_eq!(find_pr_url_in_text("No URL here"), None);
        assert_eq!(find_pr_url_in_text("https://github.com/owner/repo"), None);
        assert_eq!(find_pr_url_in_text(""), None);
    }

    #[test]
    fn test_find_pr_url_invalid_pr_number() {
        // /pull/ without a number should not match
        assert_eq!(
            find_pr_url_in_text("https://github.com/owner/repo/pull/abc"),
            None
        );
    }

    #[test]
    fn test_find_pr_url_gitlab() {
        let url = find_pr_url_in_text("MR: https://gitlab.com/owner/repo/-/merge_requests/456");
        assert_eq!(
            url,
            Some("https://gitlab.com/owner/repo/-/merge_requests/456".to_string())
        );
    }

    #[test]
    fn test_find_pr_url_bitbucket() {
        let url = find_pr_url_in_text("PR: https://bitbucket.org/owner/repo/pull-requests/789");
        assert_eq!(
            url,
            Some("https://bitbucket.org/owner/repo/pull-requests/789".to_string())
        );
    }

    #[test]
    fn test_extract_pr_url_from_logs_text_output() {
        let logs = vec![
            make_text_log("Starting PR creation..."),
            make_text_log("Pushing branch to remote..."),
            make_text_log("Created PR: https://github.com/delinoio/delidev/pull/217"),
        ];
        assert_eq!(
            extract_pr_url_from_logs(&logs),
            Some("https://github.com/delinoio/delidev/pull/217".to_string())
        );
    }

    #[test]
    fn test_extract_pr_url_from_logs_tool_result() {
        let logs = vec![
            make_text_log("Creating PR..."),
            make_tool_result_log("https://github.com/owner/repo/pull/99\n"),
        ];
        assert_eq!(
            extract_pr_url_from_logs(&logs),
            Some("https://github.com/owner/repo/pull/99".to_string())
        );
    }

    #[test]
    fn test_extract_pr_url_from_logs_no_url() {
        let logs = vec![make_text_log("Starting agent..."), make_text_log("Done.")];
        assert_eq!(extract_pr_url_from_logs(&logs), None);
    }

    #[test]
    fn test_extract_pr_url_from_logs_empty() {
        assert_eq!(extract_pr_url_from_logs(&[]), None);
    }

    #[test]
    fn test_extract_pr_url_from_logs_returns_first_match() {
        let logs = vec![
            make_text_log("First PR: https://github.com/owner/repo/pull/1"),
            make_text_log("Second PR: https://github.com/owner/repo/pull/2"),
        ];
        assert_eq!(
            extract_pr_url_from_logs(&logs),
            Some("https://github.com/owner/repo/pull/1".to_string())
        );
    }

    #[test]
    fn test_extract_pr_url_from_logs_skips_invalid_json() {
        let logs = vec![
            "not valid json".to_string(),
            make_text_log("PR: https://github.com/owner/repo/pull/42"),
        ];
        assert_eq!(
            extract_pr_url_from_logs(&logs),
            Some("https://github.com/owner/repo/pull/42".to_string())
        );
    }
}
