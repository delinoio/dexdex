//! Aider agent implementation.
//!
//! Aider is an open-source CLI for multi-file changes that outputs text format.

use std::process::Stdio;

use async_trait::async_trait;
use entities::AiAgentType;
use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::mpsc,
};
use tracing::{debug, error, warn};

use crate::{
    Agent, AgentConfig, AgentError, AgentResult, FileChangeType, NormalizedEvent, TtyInputHandler,
};

/// Aider agent.
#[derive(Debug)]
pub struct AiderAgent {
    /// Regex for detecting file operations.
    file_regex: Regex,
    /// Regex for detecting git operations.
    git_regex: Regex,
    /// Regex for detecting questions.
    question_regex: Regex,
}

impl Default for AiderAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl AiderAgent {
    /// Creates a new Aider agent.
    pub fn new() -> Self {
        Self {
            file_regex: Regex::new(
                r"(?:Wrote|Applied\s+\w+\s+to|Modified|Edited|Created)\s+[`']?([^`'\s]+)[`']?",
            )
            .unwrap(),
            git_regex: Regex::new(r"(?:Commit|commit)\s+([a-f0-9]{7,40})").unwrap(),
            question_regex: Regex::new(
                r"(?:\?\s*$|(?:Do you want|Would you like|Should I|Continue\?|y/n|Y/n))",
            )
            .unwrap(),
        }
    }

    /// Parses Aider's text output format.
    fn parse_text_output(&self, line: &str) -> Vec<NormalizedEvent> {
        let mut events = Vec::new();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return events;
        }

        // Check for file operations
        if let Some(caps) = self.file_regex.captures(trimmed)
            && let Some(path) = caps.get(1)
        {
            let change_type = if trimmed.contains("Created") {
                FileChangeType::Create
            } else {
                FileChangeType::Modify
            };
            events.push(NormalizedEvent::file_change(
                path.as_str(),
                change_type,
                None,
            ));
        }

        // Check for git operations (Aider auto-commits)
        if self.git_regex.is_match(trimmed) {
            events.push(NormalizedEvent::command(
                format!("git commit: {}", trimmed),
                None,
                None,
            ));
        }

        // Check for questions (TTY input)
        if self.question_regex.is_match(trimmed) {
            events.push(NormalizedEvent::ask_user(trimmed, None));
        }

        // Always add the text output
        events.push(NormalizedEvent::text(line, false));

        events
    }
}

#[async_trait]
impl Agent for AiderAgent {
    fn agent_type(&self) -> AiAgentType {
        AiAgentType::Aider
    }

    fn command(&self) -> &str {
        "aider"
    }

    fn args(&self, config: &AgentConfig) -> Vec<String> {
        let mut args = vec!["--yes".to_string()];

        if let Some(ref model) = config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        // Add message as argument
        args.push("--message".to_string());
        args.push(config.prompt.clone());

        args
    }

    fn parse_output(&self, line: &str) -> Vec<NormalizedEvent> {
        self.parse_text_output(line)
    }

    async fn run(
        &self,
        config: AgentConfig,
        event_tx: mpsc::Sender<NormalizedEvent>,
        tty_handler: Option<Box<dyn TtyInputHandler>>,
    ) -> AgentResult<()> {
        let args = self.args(&config);
        debug!("Running Aider with args: {:?}", args);

        let mut cmd = Command::new(self.command());
        cmd.args(&args)
            .current_dir(&config.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::Config("Failed to capture stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AgentError::Config("Failed to capture stderr".into()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AgentError::Config("Failed to capture stdin".into()))?;

        let _ = event_tx
            .send(NormalizedEvent::session_start(
                "aider",
                config.model.clone(),
            ))
            .await;

        let event_tx_clone = event_tx.clone();
        let tty_handler_arc = tty_handler.map(std::sync::Arc::new);
        let stdin = std::sync::Arc::new(tokio::sync::Mutex::new(stdin));

        let stdout_handle = tokio::spawn({
            let stdin = stdin.clone();
            let tty_handler = tty_handler_arc.clone();
            async move {
                let agent = AiderAgent::new();
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let events = agent.parse_text_output(&line);
                    for event in events {
                        if let NormalizedEvent::AskUserQuestion {
                            ref question,
                            ref options,
                        } = event
                            && let Some(ref handler) = tty_handler
                        {
                            match handler.handle_input(question, options.as_deref()).await {
                                Ok(response) => {
                                    let mut stdin_guard = stdin.lock().await;
                                    if let Err(e) = stdin_guard.write_all(response.as_bytes()).await
                                    {
                                        error!("Failed to write to stdin: {}", e);
                                    }
                                    if let Err(e) = stdin_guard.write_all(b"\n").await {
                                        error!("Failed to write newline: {}", e);
                                    }
                                    if let Err(e) = stdin_guard.flush().await {
                                        error!("Failed to flush stdin: {}", e);
                                    }
                                    let _ = event_tx_clone
                                        .send(NormalizedEvent::user_response(&response))
                                        .await;
                                }
                                Err(e) => {
                                    warn!("TTY handler failed: {}", e);
                                }
                            }
                        }
                        let _ = event_tx_clone.send(event).await;
                    }
                }
            }
        });

        let event_tx_stderr = event_tx.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if !line.trim().is_empty() {
                    let _ = event_tx_stderr.send(NormalizedEvent::error(&line)).await;
                }
            }
        });

        // Wait for process with optional timeout
        let timeout_secs = config.timeout_secs;
        let wait_result = if let Some(secs) = timeout_secs {
            let timeout_duration = std::time::Duration::from_secs(secs);
            match tokio::time::timeout(timeout_duration, child.wait()).await {
                Ok(result) => result,
                Err(_) => {
                    warn!("Agent timed out after {} seconds, killing process", secs);
                    if let Err(e) = child.kill().await {
                        error!("Failed to kill timed out process: {}", e);
                    }
                    let _ = child.wait().await;
                    stdout_handle.abort();
                    stderr_handle.abort();
                    let _ = event_tx
                        .send(NormalizedEvent::session_end(
                            false,
                            Some(format!("Agent timed out after {} seconds", secs)),
                        ))
                        .await;
                    return Err(AgentError::Timeout(secs));
                }
            }
        } else {
            child.wait().await
        };

        let status = wait_result?;

        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        let success = status.success();
        let error = if success {
            None
        } else {
            Some(format!("Process exited with code {:?}", status.code()))
        };
        let _ = event_tx
            .send(NormalizedEvent::session_end(success, error.clone()))
            .await;

        if success {
            Ok(())
        } else {
            Err(AgentError::ProcessExit {
                code: status.code().unwrap_or(-1),
                message: error.unwrap_or_default(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_modification() {
        let agent = AiderAgent::new();
        let events = agent.parse_output("Applied changes to `src/main.rs`");

        let has_file_change = events.iter().any(|e| {
            matches!(e, NormalizedEvent::FileChange { path, change_type: FileChangeType::Modify, .. } if path == "src/main.rs")
        });
        assert!(has_file_change);
    }

    #[test]
    fn test_parse_commit() {
        let agent = AiderAgent::new();
        let events = agent.parse_output("Commit abc1234 - Fix bug");

        let has_command = events.iter().any(|e| {
            matches!(e, NormalizedEvent::CommandExecution { command, .. } if command.contains("git commit"))
        });
        assert!(has_command);
    }

    #[test]
    fn test_args_generation() {
        let agent = AiderAgent::new();
        let config =
            AgentConfig::new(AiAgentType::Aider, "/workspace", "Fix the bug").with_model("gpt-4");

        let args = agent.args(&config);

        assert!(args.contains(&"--yes".to_string()));
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"gpt-4".to_string()));
        assert!(args.contains(&"--message".to_string()));
    }
}
