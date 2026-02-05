//! Local task executor for running AI agents in single-process mode.
//!
//! This module provides the `LocalExecutor` which wraps the core `TaskExecutor`
//! from the `coding_agents` crate with platform-specific event emission.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use plan_parser::{Plan, validate_plan};

use chrono::Utc;
pub use coding_agents::executor::ExecutionResult;
use coding_agents::{
    AgentResult, TimestampedEvent,
    executor::{
        AgentOutputEvent, EventEmitter, ExecutionResultWithWorktree, TaskCompletedEvent,
        TaskExecutionConfig, TaskExecutor, TaskStatusChangedEvent, TaskType, TtyInputRequestEvent,
    },
};
use entities::{
    AgentSession, AgentTask, AiAgentType, CompositeTaskNode, CompositeTaskStatus, UnitTask,
    UnitTaskStatus,
};
use task_store::{SqliteTaskStore, TaskStore};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    TtyInputRequestManager,
    error::WorkerError,
    planning_prompt::{build_planning_prompt, generate_plan_yaml_suffix, plan_yaml_filename},
};

/// Maximum number of tasks allowed in a single composite task plan.
/// This prevents resource exhaustion from excessively large plans.
const MAX_TASKS_PER_PLAN: usize = 100;

/// Default polling interval in seconds for the composite task graph monitor.
const DEFAULT_GRAPH_MONITOR_INTERVAL_SECS: u64 = 3;

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

        // Generate the plan YAML filename before creating the prompt, so the
        // agent is told exactly which file to create (instead of choosing its own
        // random suffix).
        let plan_suffix = generate_plan_yaml_suffix();
        let plan_filename = plan_yaml_filename(&plan_suffix);

        // Build the full planning prompt with PLAN.yaml format instructions
        let planning_prompt = build_planning_prompt(&composite_task.prompt, &plan_filename);

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
        let remote_url = repository.remote_url.clone();
        let branch_name_clone = branch_name.clone();
        let plan_filename_clone = plan_filename.clone();

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

        let plan = Plan::from_yaml(plan_yaml)
            .map_err(|e| format!("Failed to parse plan YAML: {}", e))?;

        // Validate the plan for cycles, invalid dependencies, duplicate IDs,
        // etc.
        let validation = validate_plan(&plan);
        if !validation.is_valid() {
            let err_msg = format!("Plan validation failed: {:?}", validation.errors);
            error!(
                composite_task_id = %composite_task_id,
                errors = ?validation.errors,
                "Plan validation failed"
            );
            // Mark composite task as failed
            if let Ok(Some(mut ct)) = self
                .task_store
                .get_composite_task(composite_task_id)
                .await
            {
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
            if let Ok(Some(mut ct)) = self
                .task_store
                .get_composite_task(composite_task_id)
                .await
            {
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
                if let Ok(Some(mut ct)) = self
                    .task_store
                    .get_composite_task(composite_task_id)
                    .await
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

        for unit_task_id in root_unit_task_ids {
            if let Err(e) = self.execute_unit_task(unit_task_id).await {
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
                let mut unit_task = UnitTask::new(
                    repository_group_id,
                    agent_task.id,
                    &plan_task.prompt,
                );
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
                    let dep_node_id =
                        plan_id_to_node_id.get(dep_plan_id).ok_or_else(|| {
                            format!("Dependency plan task not found: {}", dep_plan_id)
                        })?;
                    node.depends_on(dep_node_id.to_owned());
                }

                self.task_store
                    .update_composite_task_node(node)
                    .await
                    .map_err(|e| {
                        format!("Failed to update composite task node deps: {}", e)
                    })?;
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
    async fn monitor_composite_task_graph(
        executor: Arc<Self>,
        composite_task_id: Uuid,
        monitor_interval_secs: u64,
    ) {
        info!(
            "Starting graph monitor for composite task {}",
            composite_task_id
        );

        loop {
            // Wait before checking again (configurable interval)
            tokio::time::sleep(tokio::time::Duration::from_secs(monitor_interval_secs)).await;

            // Get all nodes for this composite task
            let nodes = match executor
                .task_store
                .list_composite_task_nodes(composite_task_id)
                .await
            {
                Ok(nodes) => nodes,
                Err(e) => {
                    error!(
                        "Graph monitor: Failed to list nodes for composite task {}: {}",
                        composite_task_id, e
                    );
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

            // A unit task is considered "successfully complete" when the AI
            // agent has finished its work (InReview or later positive states).
            let is_successfully_complete = |status: &UnitTaskStatus| -> bool {
                matches!(
                    status,
                    UnitTaskStatus::InReview
                        | UnitTaskStatus::Approved
                        | UnitTaskStatus::PrOpen
                        | UnitTaskStatus::Done
                )
            };

            // Find nodes whose dependencies are all complete but haven't started
            let mut newly_ready: Vec<Uuid> = Vec::new();
            for node in &nodes {
                let status = match node_statuses.get(&node.id) {
                    Some(s) => s,
                    None => continue,
                };

                // Only consider tasks that are InProgress (initial state) and not
                // yet being executed
                if *status != UnitTaskStatus::InProgress {
                    continue;
                }

                if executor.is_executing(node.unit_task_id).await {
                    continue;
                }

                // Check if all dependencies are successfully complete
                let all_deps_complete = node.depends_on_ids.iter().all(|dep_id| {
                    node_statuses
                        .get(dep_id)
                        .is_some_and(|s| is_successfully_complete(s))
                });

                if all_deps_complete && !node.depends_on_ids.is_empty() {
                    newly_ready.push(node.unit_task_id);
                }
            }

            // Start newly ready tasks
            for unit_task_id in &newly_ready {
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

            // Check if all tasks are in a terminal state
            let all_terminal = node_statuses.values().all(|status| {
                matches!(
                    status,
                    UnitTaskStatus::InReview
                        | UnitTaskStatus::Approved
                        | UnitTaskStatus::PrOpen
                        | UnitTaskStatus::Done
                        | UnitTaskStatus::Failed
                        | UnitTaskStatus::Rejected
                        | UnitTaskStatus::Cancelled
                )
            });

            if all_terminal && newly_ready.is_empty() {
                // All tasks reached terminal state - update composite task status
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
                    "Graph monitor: All tasks in composite task {} are terminal, \
                     setting status to {:?}",
                    composite_task_id, new_status
                );

                if let Ok(Some(mut ct)) = executor
                    .task_store
                    .get_composite_task(composite_task_id)
                    .await
                {
                    ct.status = new_status;
                    ct.updated_at = chrono::Utc::now();
                    if let Err(e) = executor.task_store.update_composite_task(ct).await {
                        error!(
                            "Graph monitor: Failed to update composite task {} status: {}",
                            composite_task_id, e
                        );
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
