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
use once_cell::sync::Lazy;
use regex::Regex;
use task_store::{SqliteTaskStore, TaskFilter, TaskStore};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Default timeout for TTY input responses (5 minutes).
const TTY_RESPONSE_TIMEOUT_SECS: u64 = 300;

/// Maximum output size to store (10 MB).
const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of retry attempts for a single task.
const MAX_RETRY_ATTEMPTS: usize = 3;

/// Regex for validating branch names (Git reference name rules).
/// Allows alphanumeric, slashes, hyphens, underscores, and dots.
/// Must not start or end with slash, dot, or contain consecutive slashes/dots.
static BRANCH_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9/_.-]*[a-zA-Z0-9]$|^[a-zA-Z0-9]$").unwrap()
});

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

    /// Validates a branch name for security.
    ///
    /// Branch names must follow Git reference naming rules and not contain
    /// shell metacharacters that could be used for injection.
    fn validate_branch_name(branch_name: &str) -> AppResult<()> {
        // Check for empty branch name
        if branch_name.is_empty() {
            return Err(AppError::Validation("Branch name cannot be empty".to_string()));
        }

        // Check length (Git has practical limits)
        if branch_name.len() > 255 {
            return Err(AppError::Validation("Branch name too long".to_string()));
        }

        // Check for dangerous characters that could be used for command injection
        let dangerous_chars = ['$', '`', '!', '|', '&', ';', '<', '>', '(', ')', '{', '}', '[', ']', '\'', '"', '\\', '\n', '\r', '\0'];
        if branch_name.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(AppError::Validation(
                "Branch name contains invalid characters".to_string(),
            ));
        }

        // Check against regex for valid Git branch name format
        if !BRANCH_NAME_REGEX.is_match(branch_name) {
            return Err(AppError::Validation(
                "Invalid branch name format".to_string(),
            ));
        }

        // Check for path traversal attempts
        if branch_name.contains("..") {
            return Err(AppError::Validation(
                "Branch name cannot contain '..'".to_string(),
            ));
        }

        Ok(())
    }

    /// Validates a repository URL for security.
    fn validate_repository_url(url: &str) -> AppResult<()> {
        // Check for empty URL
        if url.is_empty() {
            return Err(AppError::Validation("Repository URL cannot be empty".to_string()));
        }

        // Must be a valid Git URL format (HTTPS or SSH)
        let is_https = url.starts_with("https://");
        let is_ssh = url.starts_with("git@") || url.starts_with("ssh://");

        if !is_https && !is_ssh {
            return Err(AppError::Validation(
                "Repository URL must use HTTPS or SSH".to_string(),
            ));
        }

        // Check for dangerous characters
        let dangerous_chars = ['|', '&', ';', '<', '>', '`', '$', '(', ')', '{', '}', '\'', '"', '\\', '\n', '\r', '\0'];
        if url.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(AppError::Validation(
                "Repository URL contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Polls for and executes the next available task.
    ///
    /// Returns `true` if a task was found and executed.
    ///
    /// This method uses atomic status checking to prevent race conditions
    /// where multiple poll calls could select the same task.
    pub async fn poll_and_execute(&self) -> AppResult<bool> {
        // Atomically check and set status to prevent race conditions
        // This is the fix for the TOCTOU vulnerability
        {
            let mut status = self.status.write().await;
            if *status != ExecutorStatus::Idle {
                return Ok(false);
            }
            // Set to Busy immediately while still holding the lock
            *status = ExecutorStatus::Busy;
        }

        // If we fail to find a task, we need to reset to Idle
        let result = self.try_select_and_execute_task().await;

        // If no task was selected or an error occurred during selection,
        // reset status back to Idle
        if result.as_ref().map(|found| !found).unwrap_or(true) {
            // Only reset if not currently running a task
            let current_task = self.current_task.read().await;
            if current_task.is_none() {
                self.set_status(ExecutorStatus::Idle).await;
            }
        }

        result
    }

    /// Attempts to select and execute a task.
    ///
    /// Called after status is already set to Busy.
    async fn try_select_and_execute_task(&self) -> AppResult<bool> {
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

            // Check retry count - if we've exceeded max retries, mark as failed
            let failed_sessions = agent_task
                .agent_sessions
                .iter()
                .filter(|s| s.completed_at.is_some())
                .count();

            if failed_sessions >= MAX_RETRY_ATTEMPTS {
                warn!(
                    "Task {} has exceeded max retry attempts ({}), marking as rejected",
                    task.id, MAX_RETRY_ATTEMPTS
                );
                // Mark task as rejected
                let mut task_to_update = task.clone();
                task_to_update.status = UnitTaskStatus::Rejected;
                task_to_update.updated_at = Utc::now();
                self.task_store.update_unit_task(task_to_update).await?;
                continue;
            }

            // Check if there's already a running session
            let has_running_session = agent_task.agent_sessions.iter().any(|s| {
                s.started_at.is_some() && s.completed_at.is_none()
            });

            if has_running_session {
                continue;
            }

            // Check if there's a successfully completed session (no error)
            let has_successful_session = agent_task.agent_sessions.iter().any(|s| {
                s.completed_at.is_some()
                    && s.output_log
                        .as_ref()
                        .map(|log| !log.contains("\"error\":"))
                        .unwrap_or(true)
            });

            if has_successful_session {
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

            // Validate inputs before execution
            let branch_name = task.branch_name.clone().unwrap_or_else(|| repository.default_branch.clone());
            if let Err(e) = Self::validate_branch_name(&branch_name) {
                warn!("Invalid branch name for task {}: {}", task.id, e);
                continue;
            }

            if let Err(e) = Self::validate_repository_url(&repository.remote_url) {
                warn!("Invalid repository URL for task {}: {}", task.id, e);
                continue;
            }

            // Determine agent type
            let agent_type = agent_task
                .ai_agent_type
                .unwrap_or(AiAgentType::ClaudeCode);

            // Log retry information if this is a retry
            if failed_sessions > 0 {
                info!(
                    "Retrying task {} (attempt {}/{})",
                    task.id,
                    failed_sessions + 1,
                    MAX_RETRY_ATTEMPTS
                );
            } else {
                info!(
                    "Found task {} to execute with agent {:?}",
                    task.id, agent_type
                );
            }

            self.execute_task(
                task.id,
                task.agent_task_id,
                agent_type,
                agent_task.ai_agent_model.clone(),
                &repository.remote_url,
                &branch_name,
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

        // Status is already set to Busy by poll_and_execute

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

        // Batch buffer for output writes to reduce lock contention
        let mut event_batch = Vec::with_capacity(64);
        let batch_size = 64;

        // Process events and accumulate output
        while let Some(event) = event_rx.recv().await {
            // Log event
            self.handle_agent_event(&event).await;

            // Serialize event to output
            let event_str = match serde_json::to_string(&event) {
                Ok(s) => format!("{}\n", s),
                Err(_) => continue,
            };

            event_batch.push(event_str);

            // Flush batch when full or channel is empty
            if event_batch.len() >= batch_size || event_rx.is_empty() {
                let mut out = output.write().await;
                for event_str in event_batch.drain(..) {
                    if out.len() + event_str.len() <= MAX_OUTPUT_SIZE {
                        out.push_str(&event_str);
                    } else {
                        let remaining = MAX_OUTPUT_SIZE.saturating_sub(out.len());
                        if remaining > 0 {
                            out.push_str(&event_str[..remaining]);
                        }
                        warn!("Output buffer limit reached ({} bytes), truncating", MAX_OUTPUT_SIZE);
                        break;
                    }
                }
            }
        }

        // Flush any remaining events
        if !event_batch.is_empty() {
            let mut out = output.write().await;
            for event_str in event_batch.drain(..) {
                if out.len() + event_str.len() <= MAX_OUTPUT_SIZE {
                    out.push_str(&event_str);
                }
            }
        }

        // Wait for agent to complete
        let agent_result = agent_handle.await
            .map_err(|e| AppError::Internal(format!("Agent task panicked: {}", e)))?;

        // 5. Get end commit before potential cleanup
        let end_commit = self.get_current_commit(&worktree_path).await.ok();

        // 6. Cleanup worktree after task completion
        if let Err(e) = self.cleanup_worktree(&worktree_path).await {
            warn!("Failed to cleanup worktree {:?}: {}", worktree_path, e);
            // Don't fail the task for cleanup errors
        }

        if let Err(e) = agent_result {
            error!("Agent execution failed: {}", e);
            return Err(AppError::Agent(e.to_string()));
        }

        Ok(end_commit)
    }

    /// Cleans up a worktree directory after task completion.
    async fn cleanup_worktree(&self, worktree_path: &PathBuf) -> AppResult<()> {
        if !worktree_path.exists() {
            return Ok(());
        }

        info!("Cleaning up worktree at {:?}", worktree_path);

        // Remove the worktree directory
        tokio::fs::remove_dir_all(worktree_path).await.map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to remove worktree: {}", e),
            ))
        })?;

        info!("Worktree cleanup completed: {:?}", worktree_path);
        Ok(())
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
    task_id: Uuid,
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
            "TTY input requested for task {} (session {}) but not fully supported in local mode: {}",
            self.task_id, self.session_id, question
        );

        Err(coding_agents::AgentError::TtyInputRequired(format!(
            "TTY input not supported in local mode. Question: {}",
            question
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_branch_name_valid() {
        // Valid branch names
        assert!(EmbeddedExecutor::validate_branch_name("main").is_ok());
        assert!(EmbeddedExecutor::validate_branch_name("feature/new-feature").is_ok());
        assert!(EmbeddedExecutor::validate_branch_name("fix/bug-123").is_ok());
        assert!(EmbeddedExecutor::validate_branch_name("release/v1.0.0").is_ok());
        assert!(EmbeddedExecutor::validate_branch_name("user/john_doe/feature").is_ok());
        assert!(EmbeddedExecutor::validate_branch_name("a").is_ok());
        assert!(EmbeddedExecutor::validate_branch_name("ab").is_ok());
    }

    #[test]
    fn test_validate_branch_name_empty() {
        let result = EmbeddedExecutor::validate_branch_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_branch_name_too_long() {
        let long_name = "a".repeat(256);
        let result = EmbeddedExecutor::validate_branch_name(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_validate_branch_name_dangerous_chars() {
        // Test command injection characters
        let dangerous_branches = [
            "branch; rm -rf /",
            "branch && malicious",
            "branch | cat /etc/passwd",
            "branch$(whoami)",
            "branch`id`",
            "branch\necho pwned",
            "branch'malicious'",
            "branch\"malicious\"",
            "branch\\escape",
        ];

        for branch in dangerous_branches {
            let result = EmbeddedExecutor::validate_branch_name(branch);
            assert!(
                result.is_err(),
                "Branch '{}' should be rejected but was accepted",
                branch
            );
        }
    }

    #[test]
    fn test_validate_branch_name_path_traversal() {
        let result = EmbeddedExecutor::validate_branch_name("feature/../../../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".."));
    }

    #[test]
    fn test_validate_repository_url_valid() {
        // Valid HTTPS URLs
        assert!(EmbeddedExecutor::validate_repository_url("https://github.com/user/repo.git").is_ok());
        assert!(EmbeddedExecutor::validate_repository_url("https://gitlab.com/user/repo").is_ok());

        // Valid SSH URLs
        assert!(EmbeddedExecutor::validate_repository_url("git@github.com:user/repo.git").is_ok());
        assert!(EmbeddedExecutor::validate_repository_url("ssh://git@github.com/user/repo.git").is_ok());
    }

    #[test]
    fn test_validate_repository_url_empty() {
        let result = EmbeddedExecutor::validate_repository_url("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_repository_url_invalid_protocol() {
        // File protocol should be rejected
        let result = EmbeddedExecutor::validate_repository_url("file:///path/to/repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTPS or SSH"));

        // FTP should be rejected
        let result = EmbeddedExecutor::validate_repository_url("ftp://server/repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_repository_url_dangerous_chars() {
        let dangerous_urls = [
            "https://github.com/user/repo; rm -rf /",
            "https://github.com/user/repo | cat /etc/passwd",
            "https://github.com/user/repo && malicious",
            "https://github.com/user/repo$(whoami)",
            "https://github.com/user/repo`id`",
        ];

        for url in dangerous_urls {
            let result = EmbeddedExecutor::validate_repository_url(url);
            assert!(
                result.is_err(),
                "URL '{}' should be rejected but was accepted",
                url
            );
        }
    }

    #[test]
    fn test_executor_status_default() {
        // Just test the enum variants
        let idle = ExecutorStatus::Idle;
        let busy = ExecutorStatus::Busy;
        let shutting_down = ExecutorStatus::ShuttingDown;

        assert_eq!(idle, ExecutorStatus::Idle);
        assert_eq!(busy, ExecutorStatus::Busy);
        assert_eq!(shutting_down, ExecutorStatus::ShuttingDown);
        assert_ne!(idle, busy);
    }

    #[test]
    fn test_max_retry_attempts_constant() {
        // Verify the retry constant is reasonable
        assert!(MAX_RETRY_ATTEMPTS > 0);
        assert!(MAX_RETRY_ATTEMPTS <= 10);
    }

    #[test]
    fn test_max_output_size_constant() {
        // Verify output size is reasonable (10 MB)
        assert_eq!(MAX_OUTPUT_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_branch_name_regex_patterns() {
        // Valid patterns
        assert!(BRANCH_NAME_REGEX.is_match("main"));
        assert!(BRANCH_NAME_REGEX.is_match("feature/test"));
        assert!(BRANCH_NAME_REGEX.is_match("fix-123"));
        assert!(BRANCH_NAME_REGEX.is_match("v1.0.0"));
        assert!(BRANCH_NAME_REGEX.is_match("a"));

        // Invalid patterns (edge cases)
        assert!(!BRANCH_NAME_REGEX.is_match("")); // Empty
        assert!(!BRANCH_NAME_REGEX.is_match(".hidden")); // Starts with dot
        assert!(!BRANCH_NAME_REGEX.is_match("/invalid")); // Starts with slash
        assert!(!BRANCH_NAME_REGEX.is_match("invalid/")); // Ends with slash
    }
}
