//! Claude Code agent implementation.
//!
//! Claude Code is Anthropic's terminal-based agentic coding tool.
//! It outputs JSON stream format when using --output-format stream-json.

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

/// Claude Code agent.
#[derive(Debug, Default)]
pub struct ClaudeCodeAgent;

impl ClaudeCodeAgent {
    /// Creates a new Claude Code agent.
    pub fn new() -> Self {
        Self
    }

    /// Parses Claude Code's stream-json output format.
    fn parse_stream_json(&self, line: &str) -> Vec<NormalizedEvent> {
        let mut events = Vec::new();

        // Try to parse as JSON
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            // Not JSON, treat as raw output
            if !line.trim().is_empty() {
                events.push(NormalizedEvent::raw(line));
            }
            return events;
        };

        // Parse based on event type
        if let Some(event_type) = value.get("type").and_then(|v| v.as_str()) {
            match event_type {
                "system" => {
                    // System message - session start
                    if let Some(msg) = value.get("message").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::session_start(
                            "claude_code",
                            value
                                .get("model")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        ));
                        events.push(NormalizedEvent::text(msg, false));
                    }
                }
                "assistant" => {
                    // Assistant message with content
                    if let Some(content) = value.get("content")
                        && let Some(content_arr) = content.as_array()
                    {
                        for item in content_arr {
                            self.parse_content_item(item, &mut events);
                        }
                    }
                }
                "tool_use" => {
                    // Tool use
                    let tool_name = value
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let input = value
                        .get("input")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    events.push(NormalizedEvent::tool_use(tool_name, input.clone()));

                    // Check for specific tool types
                    self.handle_tool_use(tool_name, &input, &mut events);
                }
                "tool_result" => {
                    // Tool result
                    let tool_name = value
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
                    events.push(NormalizedEvent::tool_result(tool_name, output, is_error));
                }
                "thinking" => {
                    // Extended thinking
                    if let Some(thinking) = value.get("thinking").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::thinking(thinking));
                    }
                }
                "error" => {
                    // Error
                    if let Some(error) = value.get("error").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::error(error));
                    }
                }
                "result" => {
                    // Final result
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
                    // Unknown event type, include as raw
                    debug!("Unknown Claude Code event type: {}", event_type);
                    events.push(NormalizedEvent::raw(line));
                }
            }
        } else {
            // No type field, try to extract content
            if let Some(text) = value.get("text").and_then(|v| v.as_str()) {
                events.push(NormalizedEvent::text(text, true));
            } else {
                events.push(NormalizedEvent::raw(line));
            }
        }

        events
    }

    /// Parses a content item from Claude Code output.
    fn parse_content_item(&self, item: &serde_json::Value, events: &mut Vec<NormalizedEvent>) {
        if let Some(item_type) = item.get("type").and_then(|v| v.as_str()) {
            match item_type {
                "text" => {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::text(text, false));
                    }
                }
                "tool_use" => {
                    let tool_name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let input = item
                        .get("input")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    events.push(NormalizedEvent::tool_use(tool_name, input.clone()));
                    self.handle_tool_use(tool_name, &input, events);
                }
                _ => {}
            }
        }
    }

    /// Handles specific tool uses and creates appropriate events.
    fn handle_tool_use(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
        events: &mut Vec<NormalizedEvent>,
    ) {
        match tool_name {
            "Write" | "Edit" => {
                // File modification
                if let Some(path) = input.get("file_path").and_then(|v| v.as_str()) {
                    let content = input
                        .get("content")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let change_type = if tool_name == "Write" {
                        FileChangeType::Create
                    } else {
                        FileChangeType::Modify
                    };
                    events.push(NormalizedEvent::file_change(path, change_type, content));
                }
            }
            "Bash" => {
                // Command execution
                if let Some(command) = input.get("command").and_then(|v| v.as_str()) {
                    events.push(NormalizedEvent::command(command, None, None));
                }
            }
            "AskUserQuestion" => {
                // TTY input request
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
impl Agent for ClaudeCodeAgent {
    fn agent_type(&self) -> AiAgentType {
        AiAgentType::ClaudeCode
    }

    fn command(&self) -> &str {
        "claude"
    }

    fn args(&self, config: &AgentConfig) -> Vec<String> {
        let mut args = vec![
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ];

        // Add model if specified
        if let Some(ref model) = config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        // Add the prompt as the positional argument
        args.push(config.prompt.clone());

        args
    }

    fn parse_output(&self, line: &str) -> Vec<NormalizedEvent> {
        self.parse_stream_json(line)
    }

    async fn run(
        &self,
        config: AgentConfig,
        event_tx: mpsc::Sender<NormalizedEvent>,
        tty_handler: Option<Box<dyn TtyInputHandler>>,
    ) -> AgentResult<()> {
        let args = self.args(&config);
        debug!("Running Claude Code with args: {:?}", args);

        let mut cmd = Command::new(self.command());
        cmd.args(&args)
            .current_dir(&config.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        // Spawn the process
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

        // Send session start event
        let _ = event_tx
            .send(NormalizedEvent::session_start(
                "claude_code",
                config.model.clone(),
            ))
            .await;

        // Process stdout
        let event_tx_clone = event_tx.clone();
        let tty_handler_arc = tty_handler.map(std::sync::Arc::new);
        let stdin = std::sync::Arc::new(tokio::sync::Mutex::new(stdin));

        let stdout_handle = tokio::spawn({
            let stdin = stdin.clone();
            let tty_handler = tty_handler_arc.clone();
            async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let events = ClaudeCodeAgent::new().parse_stream_json(&line);
                    for event in events {
                        // Check for TTY input request
                        if let NormalizedEvent::AskUserQuestion {
                            ref question,
                            ref options,
                        } = event
                            && let Some(ref handler) = tty_handler
                        {
                            match handler.handle_input(question, options.as_deref()).await {
                                Ok(response) => {
                                    // Send response to stdin
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
                                    // Send user response event
                                    if let Err(e) = event_tx_clone
                                        .send(NormalizedEvent::user_response(&response))
                                        .await
                                    {
                                        warn!("Failed to send user response event: {}", e);
                                    }
                                }
                                Err(e) => {
                                    warn!("TTY handler failed: {}", e);
                                }
                            }
                        } else if let NormalizedEvent::AskUserQuestion { .. } = event {
                            // No TTY handler but agent requested input - log warning
                            warn!("Agent requested TTY input but no handler is configured");
                        }
                        if let Err(e) = event_tx_clone.send(event).await {
                            warn!("Failed to send event: {}", e);
                        }
                    }
                }
            }
        });

        // Process stderr
        let event_tx_stderr = event_tx.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if !line.trim().is_empty()
                    && let Err(e) = event_tx_stderr.send(NormalizedEvent::error(&line)).await
                {
                    warn!("Failed to send stderr event: {}", e);
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
                    // Timeout occurred - kill the process
                    warn!("Agent timed out after {} seconds, killing process", secs);
                    if let Err(e) = child.kill().await {
                        error!("Failed to kill timed out process: {}", e);
                    }
                    // Wait for cleanup
                    let _ = child.wait().await;
                    // Abort the output handlers
                    stdout_handle.abort();
                    stderr_handle.abort();
                    // Send timeout event
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

        // Wait for output processing to complete
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;

        // Send session end event
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
        let agent = ClaudeCodeAgent::new();
        let line = r#"{"type":"assistant","content":[{"type":"text","text":"Hello, world!"}]}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::TextOutput { content, stream: false }) if content == "Hello, world!"
        ));
    }

    #[test]
    fn test_parse_tool_use_event() {
        let agent = ClaudeCodeAgent::new();
        let line = r#"{"type":"tool_use","name":"Bash","input":{"command":"ls -la"}}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::ToolUse { tool_name, .. }) if tool_name == "Bash"
        ));
    }

    #[test]
    fn test_parse_ask_user_event() {
        let agent = ClaudeCodeAgent::new();
        let line = r#"{"type":"tool_use","name":"AskUserQuestion","input":{"question":"Continue?","options":["Yes","No"]}}"#;
        let events = agent.parse_output(line);

        // Should have both tool_use and ask_user events
        let has_ask_user = events.iter().any(|e| e.is_tty_input_required());
        assert!(has_ask_user);
    }

    #[test]
    fn test_args_generation() {
        let agent = ClaudeCodeAgent::new();
        let config = AgentConfig::new(AiAgentType::ClaudeCode, "/workspace", "Fix the bug")
            .with_model("claude-sonnet-4-20250514");

        let args = agent.args(&config);

        assert!(args.contains(&"--output-format".to_string()));
        assert!(args.contains(&"stream-json".to_string()));
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"claude-sonnet-4-20250514".to_string()));
        assert!(args.contains(&"Fix the bug".to_string()));
    }
}
