//! Normalized event types for AI agent output.
//!
//! All AI coding agents produce different output formats, but this module
//! normalizes them to a common event stream format.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of file change made by an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeType {
    /// File was created.
    Create,
    /// File was modified.
    Modify,
    /// File was deleted.
    Delete,
    /// File was renamed.
    Rename {
        /// Original file path.
        from: String,
    },
}

/// Token usage statistics from an AI coding agent session.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub total_cost_usd: f64,
    pub duration_ms: u64,
    pub num_turns: u32,
}

/// Normalized event types from AI coding agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NormalizedEvent {
    TextOutput {
        content: String,
        stream: bool,
    },
    ErrorOutput {
        content: String,
    },
    ToolUse {
        tool_name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_name: String,
        output: serde_json::Value,
        is_error: bool,
    },
    FileChange {
        path: String,
        change_type: FileChangeType,
        content: Option<String>,
    },
    CommandExecution {
        command: String,
        exit_code: Option<i32>,
        output: Option<String>,
    },
    AskUserQuestion {
        question: String,
        options: Option<Vec<String>>,
    },
    UserResponse {
        response: String,
    },
    SessionStart {
        agent_type: String,
        model: Option<String>,
    },
    SessionEnd {
        success: bool,
        error: Option<String>,
        token_usage: Option<TokenUsage>,
    },
    Thinking {
        content: String,
    },
    Raw {
        content: String,
    },
}

impl NormalizedEvent {
    pub fn text(content: impl Into<String>, stream: bool) -> Self {
        Self::TextOutput {
            content: content.into(),
            stream,
        }
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self::ErrorOutput {
            content: content.into(),
        }
    }

    pub fn session_start(agent_type: impl Into<String>, model: Option<String>) -> Self {
        Self::SessionStart {
            agent_type: agent_type.into(),
            model,
        }
    }

    pub fn session_end(success: bool, error: Option<String>) -> Self {
        Self::SessionEnd {
            success,
            error,
            token_usage: None,
        }
    }

    pub fn session_end_with_usage(
        success: bool,
        error: Option<String>,
        token_usage: Option<TokenUsage>,
    ) -> Self {
        Self::SessionEnd {
            success,
            error,
            token_usage,
        }
    }

    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking {
            content: content.into(),
        }
    }

    pub fn file_change(
        path: impl Into<String>,
        change_type: FileChangeType,
        content: Option<String>,
    ) -> Self {
        Self::FileChange {
            path: path.into(),
            change_type,
            content,
        }
    }

    pub fn command(
        command: impl Into<String>,
        exit_code: Option<i32>,
        output: Option<String>,
    ) -> Self {
        Self::CommandExecution {
            command: command.into(),
            exit_code,
            output,
        }
    }

    pub fn tool_use(tool_name: impl Into<String>, input: serde_json::Value) -> Self {
        Self::ToolUse {
            tool_name: tool_name.into(),
            input,
        }
    }

    pub fn tool_result(
        tool_name: impl Into<String>,
        output: serde_json::Value,
        is_error: bool,
    ) -> Self {
        Self::ToolResult {
            tool_name: tool_name.into(),
            output,
            is_error,
        }
    }

    pub fn ask_user(question: impl Into<String>, options: Option<Vec<String>>) -> Self {
        Self::AskUserQuestion {
            question: question.into(),
            options,
        }
    }

    pub fn user_response(response: impl Into<String>) -> Self {
        Self::UserResponse {
            response: response.into(),
        }
    }

    pub fn raw(content: impl Into<String>) -> Self {
        Self::Raw {
            content: content.into(),
        }
    }

    pub fn is_tty_input_required(&self) -> bool {
        matches!(self, Self::AskUserQuestion { .. })
    }
}

/// A timestamped event for storage in logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEvent {
    pub timestamp: DateTime<Utc>,
    pub event: NormalizedEvent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_event_creation() {
        let text = NormalizedEvent::text("Hello, world!", false);
        assert!(matches!(
            text,
            NormalizedEvent::TextOutput { content, stream: false } if content == "Hello, world!"
        ));

        let error = NormalizedEvent::error("Something went wrong");
        assert!(matches!(
            error,
            NormalizedEvent::ErrorOutput { content } if content == "Something went wrong"
        ));
    }

    #[test]
    fn test_tty_input_detection() {
        let ask = NormalizedEvent::ask_user("Continue?", Some(vec!["Yes".into(), "No".into()]));
        assert!(ask.is_tty_input_required());

        let text = NormalizedEvent::text("Output", false);
        assert!(!text.is_tty_input_required());
    }

    #[test]
    fn test_file_change_types() {
        let create =
            NormalizedEvent::file_change("test.rs", FileChangeType::Create, Some("content".into()));
        assert!(matches!(
            create,
            NormalizedEvent::FileChange { path, change_type: FileChangeType::Create, .. } if path == "test.rs"
        ));

        let rename = NormalizedEvent::file_change(
            "new.rs",
            FileChangeType::Rename {
                from: "old.rs".into(),
            },
            None,
        );
        assert!(matches!(
            rename,
            NormalizedEvent::FileChange { change_type: FileChangeType::Rename { from }, .. } if from == "old.rs"
        ));
    }

    #[test]
    fn test_serialization() {
        let event = NormalizedEvent::tool_use("read_file", serde_json::json!({"path": "test.rs"}));
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("tool_use"));
        assert!(json.contains("read_file"));
    }
}
