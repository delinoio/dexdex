//! Claude Code agent implementation.
//!
//! Claude Code is Anthropic's terminal-based agentic coding tool.
//! It outputs JSON stream format when using --output-format stream-json.

use std::process::Stdio;

use async_trait::async_trait;
use entities::AiAgentType;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
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
                    // System message - can be init (session start) or other system events
                    let subtype = value.get("subtype").and_then(|v| v.as_str());

                    if subtype == Some("init") {
                        // Init event contains session metadata (cwd, session_id, tools, model,
                        // etc.)
                        events.push(NormalizedEvent::session_start(
                            "claude_code",
                            value
                                .get("model")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        ));
                    } else if let Some(msg) = value.get("message").and_then(|v| v.as_str()) {
                        // System event with a message field
                        events.push(NormalizedEvent::session_start(
                            "claude_code",
                            value
                                .get("model")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        ));
                        events.push(NormalizedEvent::text(msg, false));
                    } else {
                        // Other system events - include as raw for visibility
                        debug!("System event without recognized format: {}", line);
                        events.push(NormalizedEvent::raw(line));
                    }
                }
                "assistant" => {
                    // Assistant message with content - can be either:
                    // 1. {"type":"assistant","content":[...]} - older format
                    // 2. {"type":"assistant","message":{"content":[...]}} - newer format
                    let content = value
                        .get("content")
                        .or_else(|| value.get("message").and_then(|m| m.get("content")));

                    if let Some(content_arr) = content.and_then(|c| c.as_array()) {
                        for item in content_arr {
                            self.parse_content_item(item, &mut events);
                        }
                    } else {
                        // Assistant event without expected format, include as raw
                        debug!("Assistant event without content array: {}", line);
                        events.push(NormalizedEvent::raw(line));
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
                    } else {
                        // Thinking event without expected format, include as raw
                        debug!("Thinking event without thinking field: {}", line);
                        events.push(NormalizedEvent::raw(line));
                    }
                }
                "error" => {
                    // Error
                    if let Some(error) = value.get("error").and_then(|v| v.as_str()) {
                        events.push(NormalizedEvent::error(error));
                    } else {
                        // Error event without expected format, include as raw
                        debug!("Error event without error field: {}", line);
                        events.push(NormalizedEvent::raw(line));
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
                "user" => {
                    // User message - extract content from the message field
                    // Content can be:
                    // 1. A string: {"message":{"content":"text"}}
                    // 2. An array of text blocks:
                    //    {"message":{"content":[{"type":"text","text":"..."}]}}
                    // 3. An array of tool_result blocks:
                    //    {"message":{"content":[{"type":"tool_result","content":"..."}]}}
                    let mut found_content = false;
                    if let Some(message) = value.get("message") {
                        if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                            events.push(NormalizedEvent::user_response(content));
                            found_content = true;
                        } else if let Some(content_arr) =
                            message.get("content").and_then(|v| v.as_array())
                        {
                            // Content can be an array of content blocks
                            for item in content_arr {
                                let item_type = item.get("type").and_then(|v| v.as_str());
                                match item_type {
                                    Some("text") => {
                                        if let Some(text) =
                                            item.get("text").and_then(|v| v.as_str())
                                        {
                                            events.push(NormalizedEvent::user_response(text));
                                            found_content = true;
                                        }
                                    }
                                    Some("tool_result") => {
                                        // Tool result from user - this is the result of a tool call
                                        let tool_use_id = item
                                            .get("tool_use_id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown");
                                        let output = item
                                            .get("content")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null);
                                        let is_error = item
                                            .get("is_error")
                                            .and_then(|v| v.as_bool())
                                            .unwrap_or(false);
                                        // Use tool_use_id as a placeholder for tool_name since it's
                                        // not available The
                                        // tool result event will show the output
                                        events.push(NormalizedEvent::tool_result(
                                            tool_use_id,
                                            output,
                                            is_error,
                                        ));
                                        found_content = true;
                                    }
                                    _ => {
                                        // Other content types - try to extract as text
                                        if let Some(text) =
                                            item.get("text").and_then(|v| v.as_str())
                                        {
                                            events.push(NormalizedEvent::user_response(text));
                                            found_content = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !found_content {
                        // User event without expected format, include as raw
                        debug!("User event without extractable content: {}", line);
                        events.push(NormalizedEvent::raw(line));
                    }
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
            "--print".to_string(),   // Non-interactive mode (required for automation)
            "--verbose".to_string(), // Required for stream-json with --print
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--dangerously-skip-permissions".to_string(), // Skip permission prompts
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
        tracing::info!(
            "Running Claude Code agent: command='{}', working_dir='{}', args={:?}",
            self.command(),
            config.working_dir,
            args
        );

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
        tracing::info!("Spawning Claude Code process...");
        let mut child = cmd.spawn()?;
        tracing::info!(
            "Claude Code process spawned successfully, pid={:?}",
            child.id()
        );

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

        // Drop stdin immediately - Claude Code with --print mode doesn't need stdin
        // and will hang if stdin stays open. If we need TTY input later,
        // we'll need a different approach (like PTY).
        drop(stdin);
        tracing::info!("Dropped stdin to allow Claude Code to proceed");

        // Send session start event
        tracing::info!("Sending session_start event...");
        let _ = event_tx
            .send(NormalizedEvent::session_start(
                "claude_code",
                config.model.clone(),
            ))
            .await;
        tracing::info!("Session start event sent, now processing stdout/stderr");

        // Process stdout
        let event_tx_clone = event_tx.clone();
        // Note: TTY input via stdin is currently disabled because Claude Code
        // hangs when stdin is kept open. For now, we drop stdin and log warnings
        // if Claude requests user input via AskUserQuestion.
        // TODO: Implement proper TTY handling using PTY or alternative approach.
        let _tty_handler = tty_handler; // Store for potential future use

        let stdout_handle = tokio::spawn(async move {
            tracing::info!("Starting stdout reader task");
            let mut reader = BufReader::new(stdout).lines();
            let mut line_count = 0u64;
            while let Ok(Some(line)) = reader.next_line().await {
                line_count += 1;
                tracing::info!("Received stdout line {}: {} bytes", line_count, line.len());
                let events = ClaudeCodeAgent::new().parse_stream_json(&line);
                for event in events {
                    // Log warning if Claude requests user input (not supported yet)
                    if let NormalizedEvent::AskUserQuestion { ref question, .. } = event {
                        warn!(
                            "Agent requested user input but stdin is closed: {}",
                            question
                        );
                    }
                    if let Err(e) = event_tx_clone.send(event).await {
                        warn!("Failed to send event: {}", e);
                    }
                }
            }
            tracing::info!("Stdout reader task finished after {} lines", line_count);
        });

        // Process stderr
        let event_tx_stderr = event_tx.clone();
        let stderr_handle = tokio::spawn(async move {
            tracing::info!("Starting stderr reader task");
            let mut reader = BufReader::new(stderr).lines();
            let mut line_count = 0u64;
            while let Ok(Some(line)) = reader.next_line().await {
                line_count += 1;
                tracing::info!(
                    "Received stderr line {}: {}",
                    line_count,
                    &line[..line.len().min(100)]
                );
                if !line.trim().is_empty()
                    && let Err(e) = event_tx_stderr.send(NormalizedEvent::error(&line)).await
                {
                    warn!("Failed to send stderr event: {}", e);
                }
            }
            tracing::info!("Stderr reader task finished after {} lines", line_count);
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
        tracing::info!("Claude Code process completed with status: {:?}", status);

        // Wait for output processing to complete
        tracing::info!("Waiting for stdout/stderr handlers to complete...");
        let _ = stdout_handle.await;
        let _ = stderr_handle.await;
        tracing::info!("Output handlers completed");

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

    #[test]
    fn test_parse_user_event() {
        let agent = ClaudeCodeAgent::new();
        // User event with string content
        let line = r#"{"type":"user","message":{"role":"user","content":"What's the capital of France?"},"session_id":"default"}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::UserResponse { response }) if response == "What's the capital of France?"
        ));
    }

    #[test]
    fn test_parse_user_event_with_array_content() {
        let agent = ClaudeCodeAgent::new();
        // User event with array content
        let line = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"Hello from array"}]},"session_id":"default"}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::UserResponse { response }) if response == "Hello from array"
        ));
    }

    #[test]
    fn test_parse_system_init_event() {
        let agent = ClaudeCodeAgent::new();
        // System init event from Claude Code stream-json format
        let line = r#"{"type":"system","subtype":"init","cwd":"/workspace","session_id":"abc123","tools":["Bash","Read"],"model":"claude-opus-4-5-20251101"}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::SessionStart { agent_type, model }) if agent_type == "claude_code" && model == &Some("claude-opus-4-5-20251101".to_string())
        ));
    }

    #[test]
    fn test_parse_assistant_nested_message_content() {
        let agent = ClaudeCodeAgent::new();
        // Assistant event with nested message.content format
        let line = r#"{"type":"assistant","message":{"model":"claude-opus-4-5-20251101","content":[{"type":"text","text":"Hello from nested message"}]}}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::TextOutput { content, .. }) if content == "Hello from nested message"
        ));
    }

    #[test]
    fn test_parse_assistant_nested_tool_use() {
        let agent = ClaudeCodeAgent::new();
        // Assistant event with nested tool_use in message.content
        let line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"tool123","name":"Bash","input":{"command":"pwd"}}]}}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::ToolUse { tool_name, .. }) if tool_name == "Bash"
        ));
    }

    #[test]
    fn test_parse_user_tool_result() {
        let agent = ClaudeCodeAgent::new();
        // User event with tool_result content
        let line = r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"tool123","type":"tool_result","content":"/workspace","is_error":false}]}}"#;
        let events = agent.parse_output(line);

        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(NormalizedEvent::ToolResult { tool_name, is_error, .. }) if tool_name == "tool123" && !*is_error
        ));
    }
}
