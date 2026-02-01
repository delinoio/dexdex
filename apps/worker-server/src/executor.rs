//! Task execution pipeline.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use coding_agents::{AgentConfig, AgentResult, NormalizedEvent, TtyInputHandler};
use entities::AiAgentType;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::client::{MainServerClient, TaskAssignment, TaskStatusUpdate, TtyInputRequest};
use crate::error::{WorkerError, WorkerResult};
use crate::state::{AppState, RunningTask, WorkerStatus};

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
            "Waiting for TTY input response for request {}",
            request_id
        );

        // Wait for response
        match rx.recv().await {
            Some(response) => {
                debug!("Received TTY response: {}", response);
                Ok(response)
            }
            None => Err(coding_agents::AgentError::TtyInputRequired(
                "TTY response channel closed".to_string(),
            )),
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
        let (status, error, end_commit) = match &result {
            Ok(commit) => ("completed".to_string(), None, commit.clone()),
            Err(e) => ("failed".to_string(), Some(e.to_string()), None),
        };

        self.client
            .report_status(TaskStatusUpdate {
                task_id: task.task_id,
                session_id: task.session_id,
                status,
                output: Some(final_output),
                error,
                end_commit,
            })
            .await?;

        // Clean up
        self.state.set_current_task(None).await;
        self.state.set_status(WorkerStatus::Idle).await;

        info!("Completed execution of task {}", task.task_id);

        result.map(|_| ())
    }

    /// Inner task execution logic.
    async fn execute_task_inner(
        &self,
        task: &TaskAssignment,
        output: Arc<RwLock<String>>,
    ) -> WorkerResult<Option<String>> {
        // 1. Create worktree
        let worktree_path = self.create_worktree(task).await?;
        info!("Created worktree at {:?}", worktree_path);

        // Update running task with worktree path
        if let Some(mut running) = self.state.current_task.write().await.take() {
            running.worktree_path = worktree_path.to_string_lossy().to_string();
            self.state.set_current_task(Some(running)).await;
        }

        // 2. Get secrets
        let secrets = self.client.get_secrets(task.task_id).await?;
        debug!("Retrieved {} secrets", secrets.len());

        // 3. Build or use default Docker image
        let image = self.get_or_build_image(&worktree_path).await?;
        info!("Using Docker image: {}", image);

        // 4. Create and start container
        let container_name = format!("delidev-{}", task.session_id);
        let repo_name = self.extract_repo_name(&task.repository_url);

        let container_id = self
            .state
            .docker
            .create_container(
                &image,
                &container_name,
                &worktree_path.to_string_lossy(),
                &repo_name,
                secrets.clone(),
            )
            .await?;

        // Update running task with container ID
        if let Some(mut running) = self.state.current_task.write().await.take() {
            running.container_id = Some(container_id.clone());
            self.state.set_current_task(Some(running)).await;
        }

        info!("Started container: {}", container_id);

        // 5. Execute agent in container
        let agent_result = self
            .execute_agent_in_container(
                &container_id,
                task,
                secrets,
                output.clone(),
            )
            .await;

        // 6. Get end commit (if successful)
        let end_commit = if agent_result.is_ok() {
            self.get_current_commit(&worktree_path).await.ok()
        } else {
            None
        };

        // 7. Clean up container (but keep worktree for review)
        if let Err(e) = self.state.docker.remove_container(&container_id).await {
            warn!("Failed to remove container: {}", e);
        }

        agent_result?;
        Ok(end_commit)
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
    async fn get_or_build_image(&self, worktree_path: &PathBuf) -> WorkerResult<String> {
        use crate::docker::DockerManager;

        if DockerManager::has_custom_dockerfile(worktree_path) {
            let tag = format!("delidev-custom:{}", Uuid::new_v4());
            self.state.docker.build_custom_image(worktree_path, &tag).await?;
            Ok(tag)
        } else {
            Ok(self.state.config.default_docker_image.clone())
        }
    }

    /// Executes the AI agent inside the container.
    async fn execute_agent_in_container(
        &self,
        container_id: &str,
        task: &TaskAssignment,
        secrets: HashMap<String, String>,
        output: Arc<RwLock<String>>,
    ) -> WorkerResult<()> {
        let agent_type = self.parse_agent_type(&task.agent_type)?;
        let repo_name = self.extract_repo_name(&task.repository_url);

        // Build agent command
        let agent = coding_agents::create_agent(agent_type);
        let config = AgentConfig::new(agent_type, format!("/workspace/{}", repo_name), &task.prompt);

        let args = agent.args(&config);
        let mut cmd = vec![agent.command()];
        cmd.extend(args.iter().map(|s| s.as_str()));

        // Build environment variables
        let mut env = Vec::new();
        for (key, value) in &secrets {
            env.push(format!("{}={}", key, value));
        }

        // Execute command in container
        info!("Executing agent: {:?}", cmd);

        let exec_output = self
            .state
            .docker
            .exec_in_container(container_id, cmd, Some(env))
            .await?;

        // Store output
        {
            let mut out = output.write().await;
            out.push_str(&exec_output);
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
            NormalizedEvent::FileChange { path, change_type, .. } => {
                debug!("File changed: {} ({:?})", path, change_type);
            }
            NormalizedEvent::CommandExecution { command, exit_code, .. } => {
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
    fn extract_repo_name(&self, url: &str) -> String {
        url.rsplit('/')
            .next()
            .unwrap_or("repo")
            .trim_end_matches(".git")
            .to_string()
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
        assert_eq!(
            extract_repo_name("git@github.com:user/repo.git"),
            "repo"
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/repo"),
            "repo"
        );
    }
}
