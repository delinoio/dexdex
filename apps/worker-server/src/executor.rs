//! Task execution pipeline.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use coding_agents::{AgentConfig, AgentResult, NormalizedEvent, TtyInputHandler};
use entities::AiAgentType;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    client::{MainServerClient, TaskAssignment, TaskStatusUpdate, TtyInputRequest},
    error::{WorkerError, WorkerResult},
    state::{AppState, RunningTask, WorkerStatus},
};

/// Default timeout for TTY input responses (5 minutes).
const TTY_RESPONSE_TIMEOUT_SECS: u64 = 300;

/// TTY input handler that forwards requests to the main server.
pub struct RemoteTtyHandler {
    state: Arc<AppState>,
    client: Arc<MainServerClient>,
    task_id: Uuid,
    session_id: Uuid,
}

impl RemoteTtyHandler {
    /// Creates a new remote TTY handler.
    pub fn new(
        state: Arc<AppState>,
        client: Arc<MainServerClient>,
        task_id: Uuid,
        session_id: Uuid,
    ) -> Self {
        Self {
            state,
            client,
            task_id,
            session_id,
        }
    }
}

#[async_trait]
impl TtyInputHandler for RemoteTtyHandler {
    async fn handle_input(
        &self,
        question: &str,
        options: Option<&[String]>,
    ) -> AgentResult<String> {
        let request_id = Uuid::new_v4();

        // Create a channel for the response
        let (tx, mut rx) = mpsc::channel::<String>(1);

        // Register the response channel
        self.state.register_tty_response(request_id, tx).await;

        // Ensure cleanup on all exit paths
        let state_cleanup = self.state.clone();
        scopeguard::defer! {
            // Remove the response channel if it wasn't consumed
            // This is a sync cleanup, so we use try_lock
            if let Ok(mut responses) = state_cleanup.tty_responses.try_lock() {
                responses.remove(&request_id);
            }
        }

        // Send request to main server
        let request = TtyInputRequest {
            request_id,
            task_id: self.task_id,
            session_id: self.session_id,
            question: question.to_string(),
            options: options.map(|o| o.to_vec()),
        };

        if let Err(e) = self.client.create_tty_input_request(request).await {
            error!("Failed to create TTY input request: {}", e);
            return Err(coding_agents::AgentError::TtyInputRequired(
                "Failed to forward TTY request".to_string(),
            ));
        }

        info!(
            "Waiting for TTY input response for request {} (timeout: {}s)",
            request_id, TTY_RESPONSE_TIMEOUT_SECS
        );

        // Wait for response with timeout to prevent memory leaks
        let timeout_duration = std::time::Duration::from_secs(TTY_RESPONSE_TIMEOUT_SECS);
        match tokio::time::timeout(timeout_duration, rx.recv()).await {
            Ok(Some(response)) => {
                debug!("Received TTY response: {}", response);
                Ok(response)
            }
            Ok(None) => {
                error!(
                    "TTY response channel closed unexpectedly for request {}",
                    request_id
                );
                Err(coding_agents::AgentError::TtyInputRequired(
                    "TTY response channel closed".to_string(),
                ))
            }
            Err(_) => {
                error!(
                    "TTY input request {} timed out after {} seconds",
                    request_id, TTY_RESPONSE_TIMEOUT_SECS
                );
                Err(coding_agents::AgentError::TtyInputRequired(format!(
                    "TTY input request timed out after {} seconds",
                    TTY_RESPONSE_TIMEOUT_SECS
                )))
            }
        }
    }
}

/// Task executor.
pub struct TaskExecutor {
    state: Arc<AppState>,
    client: Arc<MainServerClient>,
}

impl TaskExecutor {
    /// Creates a new task executor.
    pub fn new(state: Arc<AppState>, client: Arc<MainServerClient>) -> Self {
        Self { state, client }
    }

    /// Executes a task assignment.
    pub async fn execute(&self, task: TaskAssignment) -> WorkerResult<()> {
        info!("Starting execution of task {}", task.task_id);

        // Set worker status to busy
        self.state.set_status(WorkerStatus::Busy).await;

        // Create cancellation channel
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

        // Create output buffer
        let output = Arc::new(RwLock::new(String::new()));

        // Create running task entry
        let running_task = RunningTask {
            task_id: task.task_id,
            session_id: task.session_id,
            container_id: None,
            worktree_path: String::new(),
            output: output.clone(),
            cancel_tx,
        };
        self.state.set_current_task(Some(running_task)).await;

        // Report running status
        self.client
            .report_status(TaskStatusUpdate {
                task_id: task.task_id,
                session_id: task.session_id,
                status: "running".to_string(),
                output: None,
                error: None,
                end_commit: None,
                git_patch: None,
            })
            .await?;

        // Execute the task with cancellation support
        let result = tokio::select! {
            result = self.execute_task_inner(&task, output.clone()) => result,
            _ = cancel_rx.recv() => {
                warn!("Task {} was cancelled", task.task_id);
                Err(WorkerError::Cancelled)
            }
        };

        // Get final output
        let final_output = output.read().await.clone();

        // Report final status
        let (status, error, end_commit, git_patch) = match &result {
            Ok((commit, patch)) => ("completed".to_string(), None, commit.clone(), patch.clone()),
            Err(e) => ("failed".to_string(), Some(e.to_string()), None, None),
        };

        self.client
            .report_status(TaskStatusUpdate {
                task_id: task.task_id,
                session_id: task.session_id,
                status,
                output: Some(final_output),
                error,
                end_commit,
                git_patch,
            })
            .await?;

        // Clean up
        self.state.set_current_task(None).await;
        self.state.clear_tty_responses().await; // Clean up any orphaned TTY channels
        self.state.set_status(WorkerStatus::Idle).await;

        info!("Completed execution of task {}", task.task_id);

        result.map(|_| ())
    }

    /// Inner task execution logic.
    ///
    /// Returns `(end_commit, git_patch)` on success.
    async fn execute_task_inner(
        &self,
        task: &TaskAssignment,
        output: Arc<RwLock<String>>,
    ) -> WorkerResult<(Option<String>, Option<String>)> {
        // 1. Validate repository URL and branch name (security: prevent command
        //    injection)
        Self::validate_repository_url(&task.repository_url)?;
        Self::validate_branch_name(&task.branch_name)?;

        // 2. Create worktree
        let worktree_path = self.create_worktree(task).await?;
        info!("Created worktree at {:?}", worktree_path);

        // Update running task with worktree path (avoid holding lock across await)
        {
            let mut guard = self.state.current_task.write().await;
            if let Some(ref mut running) = *guard {
                running.worktree_path = worktree_path.to_string_lossy().to_string();
            }
        }

        // 3. Get secrets
        let secrets = self.client.get_secrets(task.task_id).await?;
        debug!("Retrieved {} secrets", secrets.len());

        // 4. Create secrets directory (security: use file mount instead of env vars)
        let secrets_dir = if !secrets.is_empty() {
            Some(self.state.docker.create_secrets_dir(&secrets).await?)
        } else {
            None
        };

        // Ensure cleanup happens on all exit paths
        let cleanup_secrets_dir = secrets_dir.clone();
        let cleanup_docker = self.state.docker.clone();
        scopeguard::defer! {
            if let Some(ref dir) = cleanup_secrets_dir {
                // Best-effort cleanup in sync context
                let _ = std::fs::remove_dir_all(dir);
            }
        }

        // 5. Build or use default Docker image
        let image = self.get_or_build_image(&worktree_path).await?;
        info!("Using Docker image: {}", image);

        // 6. Create and start container
        let container_name = format!("delidev-{}", task.session_id);
        let repo_name = self.extract_repo_name(&task.repository_url)?;

        let container_id = self
            .state
            .docker
            .create_container(
                &image,
                &container_name,
                &worktree_path.to_string_lossy(),
                &repo_name,
                secrets_dir.as_deref(),
            )
            .await?;

        // Update running task with container ID (avoid holding lock across await)
        {
            let mut guard = self.state.current_task.write().await;
            if let Some(ref mut running) = *guard {
                running.container_id = Some(container_id.clone());
            }
        }

        info!("Started container: {}", container_id);

        // 7. Execute agent in container
        let agent_result = self
            .execute_agent_in_container(&container_id, task, output.clone())
            .await;

        // 8. Get end commit (if successful)
        let end_commit = if agent_result.is_ok() {
            self.get_current_commit(&worktree_path).await.ok()
        } else {
            None
        };

        // 9. Generate git patch from the worktree (if successful).
        // This captures all changes made by the AI agent so they can be
        // persisted in the database without needing write access to the
        // repository.
        let git_patch = if agent_result.is_ok() {
            match git_ops::generate_patch_async(&worktree_path).await {
                Ok(patch) => {
                    if patch.is_some() {
                        info!(
                            "Generated git patch for task {} ({} bytes)",
                            task.task_id,
                            patch.as_ref().map_or(0, |p| p.len())
                        );
                    }
                    patch
                }
                Err(e) => {
                    warn!(
                        "Failed to generate git patch for task {}: {}",
                        task.task_id, e
                    );
                    None
                }
            }
        } else {
            None
        };

        // 10. Clean up container with retry logic
        self.cleanup_container_with_retry(&container_id).await;

        // 11. Clean up secrets directory (explicit async cleanup, defer is backup)
        if let Some(ref dir) = secrets_dir
            && let Err(e) = cleanup_docker.cleanup_secrets_dir(dir).await
        {
            warn!("Failed to cleanup secrets directory: {}", e);
        }

        agent_result?;
        Ok((end_commit, git_patch))
    }

    /// Validates a repository URL to prevent command injection.
    ///
    /// Only allows:
    /// - HTTPS URLs matching https://hostname/path
    /// - SSH URLs matching git@hostname:path
    fn validate_repository_url(url: &str) -> WorkerResult<()> {
        // Pattern for HTTPS URLs
        let https_pattern = regex::Regex::new(
            r"^https://[a-zA-Z0-9][-a-zA-Z0-9.]*[a-zA-Z0-9](/[-a-zA-Z0-9_.~%/]+)*(\.git)?$",
        )
        .expect("valid regex");

        // Pattern for SSH URLs
        let ssh_pattern = regex::Regex::new(
            r"^git@[a-zA-Z0-9][-a-zA-Z0-9.]*[a-zA-Z0-9]:[-a-zA-Z0-9_.~/]+(\.git)?$",
        )
        .expect("valid regex");

        if https_pattern.is_match(url) || ssh_pattern.is_match(url) {
            Ok(())
        } else {
            error!("Invalid repository URL format: {}", url);
            Err(WorkerError::Validation(format!(
                "Invalid repository URL format. Must be HTTPS or SSH URL: {}",
                url
            )))
        }
    }

    /// Validates a branch name to prevent command injection.
    ///
    /// Rejects branch names containing:
    /// - Shell metacharacters
    /// - Path traversal sequences
    /// - Null bytes
    fn validate_branch_name(branch: &str) -> WorkerResult<()> {
        // Git branch name rules with additional security restrictions
        // Allow: alphanumeric, hyphen, underscore, forward slash, dot
        // Disallow: sequences like .., leading/trailing dots, special chars
        let valid_pattern =
            regex::Regex::new(r"^[a-zA-Z0-9][-a-zA-Z0-9_./]*[a-zA-Z0-9]$|^[a-zA-Z0-9]$")
                .expect("valid regex");

        // Check for path traversal
        if branch.contains("..") {
            error!("Branch name contains path traversal: {}", branch);
            return Err(WorkerError::Validation(
                "Branch name cannot contain '..'".to_string(),
            ));
        }

        // Check for null bytes
        if branch.contains('\0') {
            error!("Branch name contains null byte");
            return Err(WorkerError::Validation(
                "Branch name cannot contain null bytes".to_string(),
            ));
        }

        // Check for shell metacharacters
        let dangerous_chars = [
            '$', '`', '|', ';', '&', '>', '<', '!', '\\', '"', '\'', '\n', '\r',
        ];
        for ch in dangerous_chars {
            if branch.contains(ch) {
                error!("Branch name contains dangerous character: {:?}", ch);
                return Err(WorkerError::Validation(format!(
                    "Branch name contains invalid character: {:?}",
                    ch
                )));
            }
        }

        if !valid_pattern.is_match(branch) {
            error!("Invalid branch name format: {}", branch);
            return Err(WorkerError::Validation(format!(
                "Invalid branch name format: {}",
                branch
            )));
        }

        Ok(())
    }

    /// Cleans up a container with retry logic.
    async fn cleanup_container_with_retry(&self, container_id: &str) {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 1000;

        for attempt in 1..=MAX_RETRIES {
            match self.state.docker.remove_container(container_id).await {
                Ok(()) => {
                    info!("Successfully removed container: {}", container_id);
                    return;
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        warn!(
                            "Failed to remove container {} (attempt {}/{}): {}. Retrying...",
                            container_id, attempt, MAX_RETRIES, e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS))
                            .await;
                    } else {
                        error!(
                            "Failed to remove container {} after {} attempts: {}. Manual cleanup \
                             may be required.",
                            container_id, MAX_RETRIES, e
                        );
                    }
                }
            }
        }
    }

    /// Creates a git worktree for the task.
    async fn create_worktree(&self, task: &TaskAssignment) -> WorkerResult<PathBuf> {
        let workdir = PathBuf::from(&self.state.config.workdir);
        tokio::fs::create_dir_all(&workdir).await?;

        let worktree_name = format!("{}-{}", task.task_id, task.branch_name);
        let worktree_path = workdir.join(&worktree_name);

        // Clone the repository if it doesn't exist
        if !worktree_path.exists() {
            let output = tokio::process::Command::new("git")
                .args([
                    "clone",
                    "--branch",
                    &task.branch_name,
                    "--single-branch",
                    "--depth",
                    "1",
                    &task.repository_url,
                    &worktree_path.to_string_lossy(),
                ])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(WorkerError::Git(format!("Failed to clone: {}", stderr)));
            }
        }

        Ok(worktree_path)
    }

    /// Gets or builds the Docker image for the task.
    async fn get_or_build_image(&self, worktree_path: &Path) -> WorkerResult<String> {
        use crate::docker::DockerManager;

        if DockerManager::has_custom_dockerfile(worktree_path) {
            let tag = format!("delidev-custom:{}", Uuid::new_v4());
            self.state
                .docker
                .build_custom_image(worktree_path, &tag)
                .await?;
            Ok(tag)
        } else {
            Ok(self.state.config.default_docker_image.clone())
        }
    }

    /// Executes the AI agent inside the container.
    ///
    /// Note: Secrets are already mounted into the container via file system.
    /// Applications should read secrets from /run/secrets/<KEY_NAME>.
    async fn execute_agent_in_container(
        &self,
        container_id: &str,
        task: &TaskAssignment,
        output: Arc<RwLock<String>>,
    ) -> WorkerResult<()> {
        let agent_type = self.parse_agent_type(&task.agent_type)?;
        let repo_name = self.extract_repo_name(&task.repository_url)?;

        // Build agent command
        let agent = coding_agents::create_agent(agent_type);
        let config = AgentConfig::new(
            agent_type,
            format!("/workspace/{}", repo_name),
            &task.prompt,
        );

        let args = agent.args(&config);
        let mut cmd = vec![agent.command()];
        cmd.extend(args.iter().map(|s| s.as_str()));

        // No secrets in environment variables - they're mounted as files
        // The SECRETS_DIR env var is set in create_container to point to /run/secrets

        // Execute command in container
        info!("Executing agent: {:?}", cmd);

        let exec_output = self
            .state
            .docker
            .exec_in_container(container_id, cmd, None)
            .await?;

        // Store output (with size limit to prevent memory exhaustion)
        const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10 MB limit
        {
            let mut out = output.write().await;
            if out.len() + exec_output.len() <= MAX_OUTPUT_SIZE {
                out.push_str(&exec_output);
            } else {
                let remaining = MAX_OUTPUT_SIZE.saturating_sub(out.len());
                if remaining > 0 {
                    out.push_str(&exec_output[..remaining]);
                }
                warn!(
                    "Output buffer limit reached ({} bytes), truncating",
                    MAX_OUTPUT_SIZE
                );
            }
        }

        // Parse agent output for events
        for line in exec_output.lines() {
            let events = agent.parse_output(line);
            for event in events {
                self.handle_agent_event(&event).await;
            }
        }

        Ok(())
    }

    /// Handles an agent event.
    async fn handle_agent_event(&self, event: &NormalizedEvent) {
        match event {
            NormalizedEvent::AskUserQuestion { question, options } => {
                info!("Agent asking: {}", question);
                // This would be handled by the TTY handler if we were streaming
                debug!("Options: {:?}", options);
            }
            NormalizedEvent::FileChange {
                path, change_type, ..
            } => {
                debug!("File changed: {} ({:?})", path, change_type);
            }
            NormalizedEvent::CommandExecution {
                command, exit_code, ..
            } => {
                debug!("Command executed: {} (exit: {:?})", command, exit_code);
            }
            NormalizedEvent::ErrorOutput { content } => {
                warn!("Agent error: {}", content);
            }
            _ => {
                debug!("Agent event: {:?}", event);
            }
        }
    }

    /// Gets the current commit hash from the worktree.
    async fn get_current_commit(&self, worktree_path: &PathBuf) -> WorkerResult<String> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(worktree_path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(WorkerError::Git("Failed to get commit hash".to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Parses the agent type string.
    fn parse_agent_type(&self, agent_type: &str) -> WorkerResult<AiAgentType> {
        match agent_type.to_lowercase().as_str() {
            "claude_code" | "claudecode" => Ok(AiAgentType::ClaudeCode),
            "open_code" | "opencode" => Ok(AiAgentType::OpenCode),
            "gemini_cli" | "geminicli" => Ok(AiAgentType::GeminiCli),
            "codex_cli" | "codexcli" => Ok(AiAgentType::CodexCli),
            "aider" => Ok(AiAgentType::Aider),
            "amp" => Ok(AiAgentType::Amp),
            _ => Err(WorkerError::Config(format!(
                "Unknown agent type: {}",
                agent_type
            ))),
        }
    }

    /// Extracts the repository name from a URL.
    ///
    /// Validates that the extracted name is safe for use as a directory name.
    fn extract_repo_name(&self, url: &str) -> WorkerResult<String> {
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or("repo")
            .trim_end_matches(".git");

        // Validate the repo name doesn't contain path traversal or dangerous characters
        if name.is_empty() {
            return Err(WorkerError::Validation(
                "Repository name cannot be empty".to_string(),
            ));
        }

        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(WorkerError::Validation(format!(
                "Invalid repository name: {}",
                name
            )));
        }

        Ok(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    /// Tests repository name extraction without needing a full executor.
    fn extract_repo_name(url: &str) -> String {
        url.rsplit('/')
            .next()
            .unwrap_or("repo")
            .trim_end_matches(".git")
            .to_string()
    }

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git"),
            "repo"
        );
        assert_eq!(extract_repo_name("git@github.com:user/repo.git"), "repo");
        assert_eq!(extract_repo_name("https://github.com/user/repo"), "repo");
    }
}
