//! OpenCode agent implementation.
//!
//! OpenCode is an open-source Claude Code alternative that outputs JSON format.

use std::process::Stdio;

use async_trait::async_trait;
use entities::AiAgentType;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::mpsc,
};
use tracing::{debug, error, warn};

use crate::{
    Agent, AgentConfig, AgentError, AgentResult, FileChangeType, NormalizedEvent, TtyInputHandler,
};

/// OpenCode agent.
#[derive(Debug, Default)]
pub struct OpenCodeAgent;

impl OpenCodeAgent {
    /// Creates a new OpenCode agent.
    pub fn new() -> Self {
        Self
    }

    /// Parses OpenCode's JSON output format.
    fn parse_json_output(&self, line: &str) -> Vec<NormalizedEvent> {
        let mut events = Vec::new();

        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            if !line.trim().is_empty() {
                events.push(NormalizedEvent::raw(line));
            }
            return events;
        };

        if let Some(event_type) = value.get("type").and_then(|v| v.as_str()) {
            match event_type {
                "message" | "text" => {
                    if let Some(content) = value.get("content").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::text(content, false));
                    }
                }
                "tool_call" | "tool_use" => {
                    let name = value
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let input = value
                        .get("arguments")
                        .or(value.get("input"))
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    events.push(NormalizedEvent::tool_use(name, input.clone()));
                    self.handle_tool_use(name, &input, &mut events);
                }
                "tool_result" => {
                    let name = value
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let output = value
                        .get("output")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    let is_error = value
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    events.push(NormalizedEvent::tool_result(name, output, is_error));
                }
                "error" => {
                    if let Some(msg) = value.get("message").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::error(msg));
                    }
                }
                "question" | "ask" => {
                    if let Some(question) = value.get("question").and_then(|v| v.as_str()) {
                        let options = value.get("options").and_then(|v| v.as_array()).map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        });
                        events.push(NormalizedEvent::ask_user(question, options));
                    }
                }
                "thinking" => {
                    if let Some(content) = value.get("content").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::thinking(content));
                    }
                }
                "done" | "complete" => {
                    let success = value
                        .get("success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    let error = value
                        .get("error")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    events.push(NormalizedEvent::session_end(success, error));
                }
                _ => {
                    debug!("Unknown OpenCode event type: {}", event_type);
                    events.push(NormalizedEvent::raw(line));
                }
            }
        } else {
            events.push(NormalizedEvent::raw(line));
        }

        events
    }

    /// Handles specific tool uses and creates appropriate events.
    fn handle_tool_use(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
        events: &mut Vec<NormalizedEvent>,
    ) {
        match tool_name {
            "write_file" | "edit_file" => {
                if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                    let content = input
                        .get("content")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let change_type = if tool_name == "write_file" {
                        FileChangeType::Create
                    } else {
                        FileChangeType::Modify
                    };
                    events.push(NormalizedEvent::file_change(path, change_type, content));
                }
            }
            "run_command" | "bash" => {
                if let Some(command) = input.get("command").and_then(|v| v.as_str()) {
                    events.push(NormalizedEvent::command(command, None, None));
                }
            }
            "ask_user" => {
                if let Some(question) = input.get("question").and_then(|v| v.as_str()) {
                    let options = input.get("options").and_then(|v| v.as_array()).map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    });
                    events.push(NormalizedEvent::ask_user(question, options));
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl Agent for OpenCodeAgent {
    fn agent_type(&self) -> AiAgentType {
        AiAgentType::OpenCode
    }

    fn command(&self) -> &str {
        "opencode"
    }

    fn args(&self, config: &AgentConfig) -> Vec<String> {
        let mut args = vec!["--output-format".to_string(), "json".to_string()];

        if let Some(ref model) = config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        args.push(config.prompt.clone());
        args
    }

    fn parse_output(&self, line: &str) -> Vec<NormalizedEvent> {
        self.parse_json_output(line)
    }

    async fn run(
        &self,
        config: AgentConfig,
        event_tx: mpsc::Sender<NormalizedEvent>,
        tty_handler: Option<Box<dyn TtyInputHandler>>,
    ) -> AgentResult<()> {
        let args = self.args(&config);
        debug!("Running OpenCode with args: {:?}", args);

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
                "open_code",
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
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let events = OpenCodeAgent::new().parse_json_output(&line);
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
                                        error!("Failed to write newline to stdin: {}", e);
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
    fn test_parse_text_event() {
        let agent = OpenCodeAgent::new();
        let line = r#"{"type":"message","content":"Hello from OpenCode"}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::TextOutput { content, .. }) if content == "Hello from OpenCode"
        ));
    }

    #[test]
    fn test_parse_question_event() {
        let agent = OpenCodeAgent::new();
        let line = r#"{"type":"question","question":"Proceed with changes?"}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(events.first().unwrap().is_tty_input_required());
    }
}
