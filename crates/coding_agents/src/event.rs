//! Normalized event types for AI agent output.
//!
//! All AI coding agents produce different output formats, but this module
//! normalizes them to a common event stream format.

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

/// Normalized event types from AI coding agents.
///
/// This enum represents all possible events that can be emitted by any
/// supported AI coding agent, normalized to a common format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NormalizedEvent {
    /// Text output from the agent.
    TextOutput {
        /// The text content.
        content: String,
        /// Whether this is streaming output (partial).
        stream: bool,
    },

    /// Error output from the agent.
    ErrorOutput {
        /// The error message.
        content: String,
    },

    /// Agent is using a tool.
    ToolUse {
        /// Name of the tool being used.
        tool_name: String,
        /// Input to the tool (JSON value).
        input: serde_json::Value,
    },

    /// Result from a tool use.
    ToolResult {
        /// Name of the tool that was used.
        tool_name: String,
        /// Output from the tool (JSON value).
        output: serde_json::Value,
        /// Whether the tool returned an error.
        is_error: bool,
    },

    /// File was changed by the agent.
    FileChange {
        /// Path to the file.
        path: String,
        /// Type of change.
        change_type: FileChangeType,
        /// New file content (if applicable).
        content: Option<String>,
    },

    /// Command was executed by the agent.
    CommandExecution {
        /// The command that was executed.
        command: String,
        /// Exit code of the command.
        exit_code: Option<i32>,
        /// Output from the command.
        output: Option<String>,
    },

    /// Agent is asking the user a question.
    AskUserQuestion {
        /// The question being asked.
        question: String,
        /// Available options (if multiple choice).
        options: Option<Vec<String>>,
    },

    /// User responded to a question.
    UserResponse {
        /// The user's response.
        response: String,
    },

    /// Agent session started.
    SessionStart {
        /// Type of agent.
        agent_type: String,
        /// Model being used (if applicable).
        model: Option<String>,
    },

    /// Agent session ended.
    SessionEnd {
        /// Whether the session completed successfully.
        success: bool,
        /// Error message if the session failed.
        error: Option<String>,
    },

    /// Agent is thinking/reasoning.
    Thinking {
        /// The thinking content.
        content: String,
    },

    /// Raw/unparsed output (fallback for unknown formats).
    Raw {
        /// Raw content from the agent.
        content: String,
    },
}

impl NormalizedEvent {
    /// Creates a new text output event.
    pub fn text(content: impl Into<String>, stream: bool) -> Self {
        Self::TextOutput {
            content: content.into(),
            stream,
        }
    }

    /// Creates a new error output event.
    pub fn error(content: impl Into<String>) -> Self {
        Self::ErrorOutput {
            content: content.into(),
        }
    }

    /// Creates a new session start event.
    pub fn session_start(agent_type: impl Into<String>, model: Option<String>) -> Self {
        Self::SessionStart {
            agent_type: agent_type.into(),
            model,
        }
    }

    /// Creates a new session end event.
    pub fn session_end(success: bool, error: Option<String>) -> Self {
        Self::SessionEnd { success, error }
    }

    /// Creates a new thinking event.
    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking {
            content: content.into(),
        }
    }

    /// Creates a new file change event.
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

    /// Creates a new command execution event.
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

    /// Creates a new tool use event.
    pub fn tool_use(tool_name: impl Into<String>, input: serde_json::Value) -> Self {
        Self::ToolUse {
            tool_name: tool_name.into(),
            input,
        }
    }

    /// Creates a new tool result event.
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

    /// Creates a new ask user question event.
    pub fn ask_user(question: impl Into<String>, options: Option<Vec<String>>) -> Self {
        Self::AskUserQuestion {
            question: question.into(),
            options,
        }
    }

    /// Creates a new user response event.
    pub fn user_response(response: impl Into<String>) -> Self {
        Self::UserResponse {
            response: response.into(),
        }
    }

    /// Creates a new raw output event.
    pub fn raw(content: impl Into<String>) -> Self {
        Self::Raw {
            content: content.into(),
        }
    }

    /// Returns true if this event is an ask-user question (TTY input required).
    pub fn is_tty_input_required(&self) -> bool {
        matches!(self, Self::AskUserQuestion { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_event_creation() {
        let text = NormalizedEvent::text("Hello, world!", false);
        assert!(matches!(
            text,
            NormalizedEvent::TextOutput {
                content,
                stream: false
            } if content == "Hello, world!"
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
            NormalizedEvent::FileChange {
                path,
                change_type: FileChangeType::Create,
                ..
            } if path == "test.rs"
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
            NormalizedEvent::FileChange {
                change_type: FileChangeType::Rename { from },
                ..
            } if from == "old.rs"
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
