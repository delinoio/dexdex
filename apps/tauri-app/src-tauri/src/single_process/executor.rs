//! Embedded task executor for single-process mode.
//!
//! This module executes AI coding agents directly without Docker,
//! suitable for local desktop execution.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use coding_agents::{AgentConfig, AgentResult, NormalizedEvent, TtyInputHandler};
use entities::{AgentSession, AiAgentType, UnitTaskStatus};
use task_store::{SqliteTaskStore, TaskFilter, TaskStore};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Default timeout for TTY input responses (5 minutes).
const TTY_RESPONSE_TIMEOUT_SECS: u64 = 300;

/// Maximum output size to store (10 MB).
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024;

/// Status of the embedded executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorStatus {
    /// Executor is idle and can accept tasks.
    Idle,
    /// Executor is busy running a task.
    Busy,
    /// Executor is shutting down.
    ShuttingDown,
}

/// State for a running task.
pub struct RunningTask {
    /// Task ID.
    pub task_id: Uuid,
    /// Session ID.
    pub session_id: Uuid,
    /// Output buffer.
    pub output: Arc<RwLock<String>>,
    /// Cancellation sender.
    pub cancel_tx: mpsc::Sender<()>,
}

/// Embedded task executor for single-process mode.
pub struct EmbeddedExecutor {
    /// Task store.
    task_store: Arc<SqliteTaskStore>,
    /// Current executor status.
    status: Arc<RwLock<ExecutorStatus>>,
    /// Currently running task (if any).
    current_task: Arc<RwLock<Option<RunningTask>>>,
    /// TTY response channels (request_id -> response sender).
    tty_responses: Arc<tokio::sync::Mutex<std::collections::HashMap<Uuid, mpsc::Sender<String>>>>,
    /// Working directory for cloned repositories.
    workdir: PathBuf,
}

impl EmbeddedExecutor {
    /// Creates a new embedded executor.
    pub fn new(task_store: Arc<SqliteTaskStore>, workdir: PathBuf) -> Self {
        Self {
            task_store,
            status: Arc::new(RwLock::new(ExecutorStatus::Idle)),
            current_task: Arc::new(RwLock::new(None)),
            tty_responses: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
            workdir,
        }
    }

    /// Gets the current executor status.
    pub async fn get_status(&self) -> ExecutorStatus {
        *self.status.read().await
    }

    /// Sets the executor status.
    pub async fn set_status(&self, status: ExecutorStatus) {
        *self.status.write().await = status;
    }

    /// Registers a TTY response channel for a pending request.
    pub async fn register_tty_response(&self, request_id: Uuid, tx: mpsc::Sender<String>) {
        self.tty_responses.lock().await.insert(request_id, tx);
    }

    /// Sends a TTY response to a pending request.
    pub async fn send_tty_response(&self, request_id: Uuid, response: String) -> AppResult<()> {
        let tx = self
            .tty_responses
            .lock()
            .await
            .remove(&request_id)
            .ok_or_else(|| AppError::NotFound("TTY response channel not found".to_string()))?;

        tx.send(response)
            .await
            .map_err(|_| AppError::Internal("Failed to send TTY response".to_string()))?;

        Ok(())
    }

    /// Clears all pending TTY response channels.
    pub async fn clear_tty_responses(&self) {
        self.tty_responses.lock().await.clear();
    }

    /// Cancels the currently running task.
    pub async fn cancel_current_task(&self) {
        let guard = self.current_task.read().await;
        if let Some(ref task) = *guard {
            let _ = task.cancel_tx.send(()).await;
        }
    }

    /// Polls for and executes the next available task.
    ///
    /// Returns `true` if a task was found and executed.
    pub async fn poll_and_execute(&self) -> AppResult<bool> {
        // Check if we're idle
        if self.get_status().await != ExecutorStatus::Idle {
            return Ok(false);
        }

        // Find a task with InProgress status that doesn't have a completed session
        let filter = TaskFilter {
            unit_status: Some(UnitTaskStatus::InProgress),
            ..Default::default()
        };

        let (tasks, _) = self.task_store.list_unit_tasks(filter).await?;

        for task in tasks {
            // Get the agent task to check sessions
            let agent_task = match self.task_store.get_agent_task(task.agent_task_id).await? {
                Some(t) => t,
                None => continue,
            };

            // Check if there's already a running or completed session
            let has_active_session = agent_task.agent_sessions.iter().any(|s| {
                s.started_at.is_some() && s.completed_at.is_none() // Running
                    || s.completed_at.is_some() // Completed
            });

            if has_active_session {
                continue;
            }

            // Get repository info
            let repo_group = match self
                .task_store
                .get_repository_group(task.repository_group_id)
                .await?
            {
                Some(g) => g,
                None => continue,
            };

            // Get first repository in the group
            let repository_id = match repo_group.repository_ids.first() {
                Some(id) => *id,
                None => continue,
            };

            let repository = match self.task_store.get_repository(repository_id).await? {
                Some(r) => r,
                None => continue,
            };

            // Determine agent type
            let agent_type = agent_task
                .ai_agent_type
                .unwrap_or(AiAgentType::ClaudeCode);

            // Execute the task
            info!(
                "Found task {} to execute with agent {:?}",
                task.id, agent_type
            );

            self.execute_task(
                task.id,
                task.agent_task_id,
                agent_type,
                agent_task.ai_agent_model.clone(),
                &repository.remote_url,
                &task.branch_name.unwrap_or_else(|| repository.default_branch.clone()),
                &task.prompt,
            )
            .await?;

            return Ok(true);
        }

        Ok(false)
    }

    /// Executes a task.
    async fn execute_task(
        &self,
        task_id: Uuid,
        agent_task_id: Uuid,
        agent_type: AiAgentType,
        agent_model: Option<String>,
        repository_url: &str,
        branch_name: &str,
        prompt: &str,
    ) -> AppResult<()> {
        info!("Starting execution of task {}", task_id);

        // Set status to busy
        self.set_status(ExecutorStatus::Busy).await;

        // Create agent session
        let mut session = AgentSession::new(agent_task_id, agent_type);
        if let Some(model) = agent_model.clone() {
            session = session.with_model(model);
        }
        session.started_at = Some(Utc::now());

        let session = self.task_store.create_agent_session(session).await?;
        let session_id = session.id;

        // Create cancellation channel
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

        // Create output buffer
        let output = Arc::new(RwLock::new(String::new()));

        // Create running task entry
        let running_task = RunningTask {
            task_id,
            session_id,
            output: output.clone(),
            cancel_tx,
        };
        *self.current_task.write().await = Some(running_task);

        // Execute the task with cancellation support
        let result = tokio::select! {
            result = self.execute_task_inner(
                task_id,
                session_id,
                agent_type,
                agent_model,
                repository_url,
                branch_name,
                prompt,
                output.clone(),
            ) => result,
            _ = cancel_rx.recv() => {
                warn!("Task {} was cancelled", task_id);
                Err(AppError::Cancelled)
            }
        };

        // Get final output
        let final_output = output.read().await.clone();

        // Update session with completion info
        let mut session = self
            .task_store
            .get_agent_session(session_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

        session.completed_at = Some(Utc::now());
        session.output_log = Some(final_output);
        self.task_store.update_agent_session(session).await?;

        // Update task status based on result
        let mut task = self
            .task_store
            .get_unit_task(task_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        match &result {
            Ok(end_commit) => {
                task.status = UnitTaskStatus::InReview;
                task.end_commit = end_commit.clone();
                info!("Task {} completed successfully", task_id);
            }
            Err(e) => {
                // Keep in InProgress status for retry
                error!("Task {} failed: {}", task_id, e);
            }
        }
        task.updated_at = Utc::now();
        self.task_store.update_unit_task(task).await?;

        // Clean up
        *self.current_task.write().await = None;
        self.clear_tty_responses().await;
        self.set_status(ExecutorStatus::Idle).await;

        info!("Completed execution of task {}", task_id);

        result.map(|_| ())
    }

    /// Inner task execution logic.
    async fn execute_task_inner(
        &self,
        task_id: Uuid,
        session_id: Uuid,
        agent_type: AiAgentType,
        agent_model: Option<String>,
        repository_url: &str,
        branch_name: &str,
        prompt: &str,
        output: Arc<RwLock<String>>,
    ) -> AppResult<Option<String>> {
        // 1. Create working directory
        let worktree_path = self.prepare_worktree(repository_url, branch_name).await?;
        info!("Prepared worktree at {:?}", worktree_path);

        // 2. Create TTY handler
        let tty_handler = Box::new(LocalTtyHandler {
            executor: self.task_store.clone(), // Note: TTY not fully supported in local mode yet
            task_id,
            session_id,
        });

        // 3. Build agent configuration
        let working_dir = worktree_path.to_string_lossy().into_owned();
        let mut config = AgentConfig::new(agent_type, working_dir, prompt);
        if let Some(model) = agent_model {
            config = config.with_model(model);
        }

        // 4. Run the agent
        info!(
            "Executing agent {:?} in {:?}",
            agent_type, worktree_path
        );

        let agent = coding_agents::create_agent(agent_type);
        let (event_tx, mut event_rx) = mpsc::channel(1024);

        // Spawn agent execution in background
        let agent_config = config.clone();
        let agent_handle = tokio::spawn(async move {
            agent.run(agent_config, event_tx, Some(tty_handler)).await
        });

        // Process events and accumulate output
        while let Some(event) = event_rx.recv().await {
            // Log event
            self.handle_agent_event(&event).await;

            // Serialize event to output
            let event_str = match serde_json::to_string(&event) {
                Ok(s) => format!("{}\n", s),
                Err(_) => continue,
            };

            // Append to output buffer (with size limit)
            {
                let mut out = output.write().await;
                if out.len() + event_str.len() <= MAX_OUTPUT_SIZE {
                    out.push_str(&event_str);
                } else {
                    let remaining = MAX_OUTPUT_SIZE.saturating_sub(out.len());
                    if remaining > 0 {
                        out.push_str(&event_str[..remaining]);
                    }
                    warn!("Output buffer limit reached ({} bytes), truncating", MAX_OUTPUT_SIZE);
                }
            }
        }

        // Wait for agent to complete
        let agent_result = agent_handle.await
            .map_err(|e| AppError::Internal(format!("Agent task panicked: {}", e)))?;

        if let Err(e) = agent_result {
            error!("Agent execution failed: {}", e);
            return Err(AppError::Agent(e.to_string()));
        }

        // 5. Get end commit
        let end_commit = self.get_current_commit(&worktree_path).await.ok();

        Ok(end_commit)
    }

    /// Prepares a worktree for the task.
    async fn prepare_worktree(
        &self,
        repository_url: &str,
        branch_name: &str,
    ) -> AppResult<PathBuf> {
        tokio::fs::create_dir_all(&self.workdir).await?;

        let repo_name = self.extract_repo_name(repository_url)?;
        let worktree_name = format!("{}-{}", repo_name, branch_name.replace('/', "-"));
        let worktree_path = self.workdir.join(&worktree_name);

        // Clone if doesn't exist
        if !worktree_path.exists() {
            info!("Cloning {} branch {} to {:?}", repository_url, branch_name, worktree_path);

            let output = tokio::process::Command::new("git")
                .args([
                    "clone",
                    "--branch",
                    branch_name,
                    "--single-branch",
                    repository_url,
                    &worktree_path.to_string_lossy(),
                ])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(AppError::Git(format!("Failed to clone: {}", stderr)));
            }
        } else {
            // Pull latest changes
            info!("Pulling latest changes in {:?}", worktree_path);

            let output = tokio::process::Command::new("git")
                .args(["pull", "--ff-only"])
                .current_dir(&worktree_path)
                .output()
                .await?;

            if !output.status.success() {
                warn!(
                    "Failed to pull latest changes: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(worktree_path)
    }

    /// Gets the current commit hash.
    async fn get_current_commit(&self, worktree_path: &PathBuf) -> AppResult<String> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(worktree_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(AppError::Git("Failed to get commit hash".to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Extracts repository name from URL.
    fn extract_repo_name(&self, url: &str) -> AppResult<String> {
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or("repo")
            .trim_end_matches(".git");

        if name.is_empty() {
            return Err(AppError::Validation("Repository name cannot be empty".to_string()));
        }

        Ok(name.to_string())
    }

    /// Handles an agent event.
    async fn handle_agent_event(&self, event: &NormalizedEvent) {
        match event {
            NormalizedEvent::TextOutput { content, stream } => {
                if !*stream {
                    debug!("Agent output: {}", content);
                }
            }
            NormalizedEvent::ErrorOutput { content } => {
                warn!("Agent error: {}", content);
            }
            NormalizedEvent::FileChange { path, change_type, .. } => {
                debug!("File changed: {} ({:?})", path, change_type);
            }
            NormalizedEvent::CommandExecution { command, exit_code, .. } => {
                debug!("Command executed: {} (exit: {:?})", command, exit_code);
            }
            NormalizedEvent::SessionStart { agent_type, model } => {
                info!("Agent session started: {} (model: {:?})", agent_type, model);
            }
            NormalizedEvent::SessionEnd { success, error } => {
                if *success {
                    info!("Agent session ended successfully");
                } else {
                    warn!("Agent session ended with error: {:?}", error);
                }
            }
            _ => {
                debug!("Agent event: {:?}", event);
            }
        }
    }
}

/// TTY input handler for local mode (placeholder for now).
struct LocalTtyHandler {
    #[allow(dead_code)]
    executor: Arc<SqliteTaskStore>,
    #[allow(dead_code)]
    task_id: Uuid,
    #[allow(dead_code)]
    session_id: Uuid,
}

#[async_trait]
impl TtyInputHandler for LocalTtyHandler {
    async fn handle_input(
        &self,
        question: &str,
        _options: Option<&[String]>,
    ) -> AgentResult<String> {
        // For now, log the question and return a default response
        // Full TTY support would require UI integration
        warn!(
            "TTY input requested but not fully supported in local mode: {}",
            question
        );

        Err(coding_agents::AgentError::TtyInputRequired(format!(
            "TTY input not supported in local mode. Question: {}",
            question
        )))
    }
}
