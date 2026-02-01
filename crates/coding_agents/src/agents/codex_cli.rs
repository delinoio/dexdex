//! Codex CLI agent implementation.
//!
//! Codex CLI is OpenAI's terminal-based coding assistant that outputs text
//! format.

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

/// Codex CLI agent.
#[derive(Debug)]
pub struct CodexCliAgent {
    /// Regex for detecting file operations.
    file_regex: Regex,
    /// Regex for detecting command execution.
    command_regex: Regex,
    /// Regex for detecting questions.
    question_regex: Regex,
}

impl Default for CodexCliAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexCliAgent {
    /// Creates a new Codex CLI agent.
    pub fn new() -> Self {
        Self {
            file_regex: Regex::new(
                r"(?:Writing|Creating|Modifying|Updating)\s+[`']?([^`'\s]+)[`']?",
            )
            .unwrap(),
            command_regex: Regex::new(r"(?:Running|Executing|>\s*|[$#]\s*)\s*[`']?([^`'\n]+)[`']?")
                .unwrap(),
            question_regex: Regex::new(
                r"(?:\?\s*$|(?:Do you want|Would you like|Should I|Continue\?|Proceed\?|y/n))",
            )
            .unwrap(),
        }
    }

    /// Parses Codex CLI's text output format.
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
            let change_type = if trimmed.contains("Creating") || trimmed.contains("Writing") {
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

        // Check for command execution
        if let Some(caps) = self.command_regex.captures(trimmed)
            && let Some(cmd) = caps.get(1)
        {
            let cmd_str = cmd.as_str().trim();
            if !cmd_str.is_empty() && !cmd_str.starts_with("Writing") {
                events.push(NormalizedEvent::command(cmd_str, None, None));
            }
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
impl Agent for CodexCliAgent {
    fn agent_type(&self) -> AiAgentType {
        AiAgentType::CodexCli
    }

    fn command(&self) -> &str {
        "codex"
    }

    fn args(&self, config: &AgentConfig) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(ref model) = config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

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
        debug!("Running Codex CLI with args: {:?}", args);

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
                "codex_cli",
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
                let agent = CodexCliAgent::new();
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
    fn test_parse_file_creation() {
        let agent = CodexCliAgent::new();
        let events = agent.parse_output("Creating `src/lib.rs`");

        let has_file_change = events.iter().any(|e| {
            matches!(e, NormalizedEvent::FileChange { path, change_type: FileChangeType::Create, .. } if path == "src/lib.rs")
        });
        assert!(has_file_change);
    }

    #[test]
    fn test_parse_question() {
        let agent = CodexCliAgent::new();
        let events = agent.parse_output("Would you like to continue? (y/n)");

        assert!(events.iter().any(|e| e.is_tty_input_required()));
    }
}
