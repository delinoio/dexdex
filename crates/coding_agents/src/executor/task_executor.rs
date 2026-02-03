//! Task executor for running AI agents.
//!
//! This module provides the core execution logic for running AI coding agents.
//! It manages git worktrees, runs agents, and streams output events through
//! an event emitter.

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use entities::AiAgentType;
use git_ops::RepositoryCache;
use tokio::{
    sync::{RwLock, mpsc},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{
    AgentOutputEvent, EventEmitter, EventEmitterTtyHandler, TaskCompletedEvent,
    TaskStatusChangedEvent, TaskType, TtyInputRequestManager,
};
use crate::{AgentConfig, NormalizedEvent, create_agent};

/// Result of a task execution.
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Task completed successfully.
    Success {
        /// Collected log entries (JSON-serialized NormalizedEvents).
        logs: Vec<String>,
    },
    /// Task failed with an error.
    Failed {
        /// Error message.
        error: String,
        /// Collected log entries (JSON-serialized NormalizedEvents).
        logs: Vec<String>,
    },
    /// Task was cancelled.
    Cancelled,
}

impl ExecutionResult {
    /// Returns the collected logs, if any.
    pub fn logs(&self) -> &[String] {
        match self {
            ExecutionResult::Success { logs } => logs,
            ExecutionResult::Failed { logs, .. } => logs,
            ExecutionResult::Cancelled => &[],
        }
    }

    /// Returns true if the execution was successful.
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Success { .. })
    }
}

/// Configuration for executing a task.
#[derive(Debug, Clone)]
pub struct TaskExecutionConfig {
    /// The task ID.
    pub task_id: Uuid,
    /// The session ID.
    pub session_id: Uuid,
    /// The remote repository URL.
    pub remote_url: String,
    /// The branch name to use.
    pub branch_name: String,
    /// The AI agent type.
    pub agent_type: AiAgentType,
    /// The AI agent model (optional).
    pub agent_model: Option<String>,
    /// The prompt/task to execute.
    pub prompt: String,
}

/// Task executor that runs AI agents with platform-agnostic event emission.
///
/// This struct provides the core execution logic for running AI coding agents.
/// It manages git worktrees via `RepositoryCache` and emits events through
/// the provided `EventEmitter`.
pub struct TaskExecutor<E: EventEmitter> {
    /// Repository cache for managing git worktrees.
    repo_cache: RepositoryCache,
    /// Event emitter for sending events.
    emitter: Arc<E>,
    /// TTY input request manager.
    tty_request_manager: Arc<TtyInputRequestManager>,
    /// Active execution handles keyed by task ID.
    execution_handles: Arc<RwLock<HashMap<Uuid, JoinHandle<ExecutionResult>>>>,
}

impl<E: EventEmitter + 'static> TaskExecutor<E> {
    /// Creates a new task executor.
    ///
    /// # Arguments
    /// * `data_dir` - The data directory for repository cache and worktrees
    /// * `emitter` - The event emitter for sending events
    pub fn new(data_dir: impl Into<PathBuf>, emitter: Arc<E>) -> Self {
        let data_dir = data_dir.into();
        Self {
            repo_cache: RepositoryCache::new(&data_dir),
            emitter,
            tty_request_manager: Arc::new(TtyInputRequestManager::new()),
            execution_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new task executor with an existing repository cache.
    pub fn with_repo_cache(repo_cache: RepositoryCache, emitter: Arc<E>) -> Self {
        Self {
            repo_cache,
            emitter,
            tty_request_manager: Arc::new(TtyInputRequestManager::new()),
            execution_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the TTY request manager for responding to input requests.
    pub fn tty_request_manager(&self) -> Arc<TtyInputRequestManager> {
        self.tty_request_manager.clone()
    }

    /// Returns the repository cache.
    pub fn repo_cache(&self) -> &RepositoryCache {
        &self.repo_cache
    }

    /// Executes a task asynchronously.
    ///
    /// This spawns a background task that:
    /// 1. Creates a git worktree for the task
    /// 2. Runs the AI agent with the task prompt
    /// 3. Streams output events to the emitter
    /// 4. Returns the execution result
    pub async fn execute(&self, config: TaskExecutionConfig) -> Result<(), String> {
        info!("Starting execution of task: {}", config.task_id);

        // Clone values needed for the spawned task
        let emitter = self.emitter.clone();
        let tty_manager = self.tty_request_manager.clone();
        let cache_parent = self
            .repo_cache
            .cache_dir()
            .parent()
            .ok_or_else(|| "Invalid cache directory path: no parent directory".to_string())?;
        let repo_cache = RepositoryCache::new(cache_parent);
        let task_id = config.task_id;

        // Spawn the execution task
        let handle = tokio::spawn(async move {
            Self::run_agent_task(config, emitter, tty_manager, repo_cache).await
        });

        // Store the handle and clean up finished ones
        let mut handles = self.execution_handles.write().await;
        // Clean up finished handles to prevent memory leaks
        handles.retain(|_id, h| !h.is_finished());
        handles.insert(task_id, handle);

        Ok(())
    }

    /// Executes a task and waits for the result.
    ///
    /// Unlike `execute`, this method waits for the task to complete and returns
    /// the result.
    pub async fn execute_and_wait(&self, config: TaskExecutionConfig) -> ExecutionResult {
        let emitter = self.emitter.clone();
        let tty_manager = self.tty_request_manager.clone();
        let cache_parent = match self.repo_cache.cache_dir().parent() {
            Some(parent) => parent,
            None => {
                return ExecutionResult::Failed {
                    error: "Invalid cache directory path: no parent directory".to_string(),
                    logs: Vec::new(),
                };
            }
        };
        let repo_cache = RepositoryCache::new(cache_parent);

        Self::run_agent_task(config, emitter, tty_manager, repo_cache).await
    }

    /// Cleans up finished execution handles to prevent memory leaks.
    pub async fn cleanup_finished_handles(&self) {
        let mut handles = self.execution_handles.write().await;
        handles.retain(|_task_id, handle| !handle.is_finished());
    }

    /// Runs the agent task (internal implementation).
    async fn run_agent_task(
        config: TaskExecutionConfig,
        emitter: Arc<E>,
        tty_manager: Arc<TtyInputRequestManager>,
        repo_cache: RepositoryCache,
    ) -> ExecutionResult {
        let task_id = config.task_id;
        let session_id = config.session_id;

        // Create a worktree for the task
        info!(
            "Creating worktree for task {} at branch {}",
            task_id, config.branch_name
        );

        let worktree_path = match repo_cache.create_worktree_for_task(
            &config.remote_url,
            &config.branch_name,
            &task_id.to_string(),
            None, // Use default credentials for now
        ) {
            Ok(path) => {
                info!("Created worktree at {:?}", path);
                path
            }
            Err(e) => {
                error!("Failed to create worktree: {}", e);
                return ExecutionResult::Failed {
                    error: format!("Failed to create worktree: {}", e),
                    logs: Vec::new(),
                };
            }
        };

        // Create the agent configuration
        let mut agent_config = AgentConfig::new(
            config.agent_type,
            worktree_path.to_string_lossy(),
            &config.prompt,
        );

        if let Some(model) = config.agent_model {
            agent_config = agent_config.with_model(model);
        }

        // Create the TTY handler
        let tty_handler =
            EventEmitterTtyHandler::new(emitter.clone(), task_id, session_id, tty_manager);

        // Create an event channel
        let (event_tx, mut event_rx) = mpsc::channel::<NormalizedEvent>(1024);

        // Run the agent
        let agent = create_agent(config.agent_type);

        // Spawn a task to handle events
        let emitter_clone = emitter.clone();
        let event_handler = tokio::spawn(async move {
            let mut logs = Vec::new();
            while let Some(event) = event_rx.recv().await {
                // Serialize the event for the output log
                if let Ok(json) = serde_json::to_string(&event) {
                    logs.push(json);
                }

                // Emit the event
                let output_event = AgentOutputEvent {
                    task_id: task_id.to_string(),
                    session_id: session_id.to_string(),
                    event: event.clone(),
                };

                if let Err(e) = emitter_clone.emit_agent_output(output_event) {
                    warn!("Failed to emit agent output event: {}", e);
                }
            }
            logs
        });

        // Run the agent
        info!(
            "Starting agent execution for task {}, agent_type={:?}",
            task_id, config.agent_type
        );
        let run_result = agent
            .run(agent_config, event_tx, Some(Box::new(tty_handler)))
            .await;
        info!(
            "Agent execution completed for task {}, result={:?}",
            task_id,
            run_result.as_ref().map(|_| "Ok").unwrap_or("Err")
        );

        // Wait for event handler to finish and collect logs
        let logs = match event_handler.await {
            Ok(logs) => logs,
            Err(e) => {
                error!("Event handler task failed: {}", e);
                Vec::new()
            }
        };

        debug!("Collected {} log entries for task {}", logs.len(), task_id);

        match run_result {
            Ok(()) => ExecutionResult::Success { logs },
            Err(e) => ExecutionResult::Failed {
                error: e.to_string(),
                logs,
            },
        }
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
    pub async fn cancel_execution(&self, task_id: Uuid) -> bool {
        let mut handles = self.execution_handles.write().await;
        if let Some(handle) = handles.remove(&task_id) {
            handle.abort();
            true
        } else {
            false
        }
    }

    /// Emits a task status changed event.
    pub fn emit_status_changed(
        &self,
        task_id: Uuid,
        task_type: TaskType,
        old_status: &str,
        new_status: &str,
    ) -> Result<(), String> {
        let event = TaskStatusChangedEvent {
            task_id: task_id.to_string(),
            task_type,
            old_status: old_status.to_string(),
            new_status: new_status.to_string(),
        };

        self.emitter
            .emit_task_status_changed(event)
            .map_err(|e| e.to_string())
    }

    /// Emits a task completed event.
    pub fn emit_completed(
        &self,
        task_id: Uuid,
        task_type: TaskType,
        success: bool,
        error: Option<String>,
    ) -> Result<(), String> {
        let event = TaskCompletedEvent {
            task_id: task_id.to_string(),
            task_type,
            success,
            error,
        };

        self.emitter
            .emit_task_completed(event)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::executor::NoOpEventEmitter;

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

    #[test]
    fn test_execution_result_logs() {
        let logs = vec!["log1".to_string(), "log2".to_string()];
        let success = ExecutionResult::Success { logs: logs.clone() };
        assert_eq!(success.logs(), &logs);
        assert!(success.is_success());

        let failed = ExecutionResult::Failed {
            error: "error".to_string(),
            logs: logs.clone(),
        };
        assert_eq!(failed.logs(), &logs);
        assert!(!failed.is_success());

        let cancelled = ExecutionResult::Cancelled;
        assert!(cancelled.logs().is_empty());
        assert!(!cancelled.is_success());
    }

    #[test]
    fn test_task_execution_config() {
        let config = TaskExecutionConfig {
            task_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            remote_url: "https://github.com/user/repo".to_string(),
            branch_name: "feature/test".to_string(),
            agent_type: AiAgentType::ClaudeCode,
            agent_model: Some("claude-sonnet-4-20250514".to_string()),
            prompt: "Fix the bug".to_string(),
        };

        assert_eq!(config.remote_url, "https://github.com/user/repo");
        assert_eq!(config.branch_name, "feature/test");
        assert_eq!(config.agent_type, AiAgentType::ClaudeCode);
    }

    #[tokio::test]
    async fn test_task_executor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let emitter = Arc::new(NoOpEventEmitter::new());
        let executor = TaskExecutor::new(temp_dir.path(), emitter);

        // Should have no executing tasks initially
        let task_id = Uuid::new_v4();
        assert!(!executor.is_executing(task_id).await);
    }

    #[tokio::test]
    async fn test_cancel_non_existent_task() {
        let temp_dir = TempDir::new().unwrap();
        let emitter = Arc::new(NoOpEventEmitter::new());
        let executor = TaskExecutor::new(temp_dir.path(), emitter);

        let task_id = Uuid::new_v4();
        assert!(!executor.cancel_execution(task_id).await);
    }

    #[tokio::test]
    async fn test_tty_request_manager() {
        let temp_dir = TempDir::new().unwrap();
        let emitter = Arc::new(NoOpEventEmitter::new());
        let executor = TaskExecutor::new(temp_dir.path(), emitter);

        let manager = executor.tty_request_manager();
        assert_eq!(manager.pending_count().await, 0);
    }
}
