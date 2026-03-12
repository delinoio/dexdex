//! Agent session entity definitions.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AiAgentType;

/// Status of an AgentSession.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSessionStatus {
    /// Session is being initialized.
    Starting,
    /// Session is actively running.
    Running,
    /// Session is paused waiting for user input.
    WaitingForInput,
    /// Session completed successfully.
    Completed,
    /// Session failed with an error.
    Failed,
    /// Session was cancelled.
    Cancelled,
}

/// Kind of output produced by a session event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOutputKind {
    /// Plain text output from the agent.
    Text,
    /// An update to the agent's plan.
    PlanUpdate,
    /// The agent is calling a tool.
    ToolCall,
    /// The result of a tool call.
    ToolResult,
    /// A progress update from the agent.
    Progress,
    /// A warning message.
    Warning,
    /// An error message.
    Error,
}

/// Token usage and cost metrics for an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageMetrics {
    /// The AI provider name (e.g., "anthropic", "openai").
    pub provider: String,
    /// The model identifier used in this session.
    pub model: String,
    /// Number of input tokens consumed (excluding cache reads).
    pub input_tokens: u64,
    /// Number of output tokens generated.
    pub output_tokens: u64,
    /// Number of tokens read from the prompt cache.
    pub cache_read_tokens: u64,
    /// Number of tokens written to the prompt cache.
    pub cache_write_tokens: u64,
    /// Total tokens (input + output + cache read + cache write).
    pub total_tokens: u64,
    /// Total cost in USD for this session.
    pub total_cost_usd: f64,
}

/// A single AI agent session scoped to a SubTask.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    /// Unique identifier.
    pub id: Uuid,
    /// The parent subtask ID.
    pub sub_task_id: Uuid,
    /// Type of AI agent running in this session.
    pub agent_type: AiAgentType,
    /// Optional model override for this session.
    pub model: Option<String>,
    /// Current status of this session.
    pub status: AgentSessionStatus,
    /// Token usage metrics, populated when the session completes.
    pub token_usage: Option<TokenUsageMetrics>,
    /// When the agent session actually started running.
    pub started_at: Option<DateTime<Utc>>,
    /// When the agent session finished (success or failure).
    pub completed_at: Option<DateTime<Utc>>,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
}

impl AgentSession {
    /// Creates a new agent session.
    pub fn new(sub_task_id: Uuid, agent_type: AiAgentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            sub_task_id,
            agent_type,
            model: None,
            status: AgentSessionStatus::Starting,
            token_usage: None,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    /// Sets the model override for this session.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

/// A single output event emitted by an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionOutputEvent {
    /// Unique identifier.
    pub id: Uuid,
    /// The session that emitted this event.
    pub session_id: Uuid,
    /// Monotonically increasing sequence number within the session.
    pub sequence: u64,
    /// The kind of output this event represents.
    pub kind: SessionOutputKind,
    /// The human-readable message content.
    pub message: String,
    /// Additional structured attributes for this event.
    pub attributes: HashMap<String, String>,
    /// When this event was emitted by the agent.
    pub emitted_at: DateTime<Utc>,
}

impl SessionOutputEvent {
    /// Creates a new session output event.
    pub fn new(
        session_id: Uuid,
        sequence: u64,
        kind: SessionOutputKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            sequence,
            kind,
            message: message.into(),
            attributes: HashMap::new(),
            emitted_at: Utc::now(),
        }
    }

    /// Adds an attribute to this event.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_session_creation() {
        let sub_task_id = Uuid::new_v4();
        let session = AgentSession::new(sub_task_id, AiAgentType::ClaudeCode)
            .with_model("claude-sonnet-4-20250514");

        assert_eq!(session.sub_task_id, sub_task_id);
        assert_eq!(session.agent_type, AiAgentType::ClaudeCode);
        assert_eq!(session.model, Some("claude-sonnet-4-20250514".to_string()));
        assert_eq!(session.status, AgentSessionStatus::Starting);
    }

    #[test]
    fn test_session_output_event_creation() {
        let session_id = Uuid::new_v4();
        let event = SessionOutputEvent::new(
            session_id,
            1,
            SessionOutputKind::Text,
            "Agent started working",
        )
        .with_attribute("tool", "bash");

        assert_eq!(event.session_id, session_id);
        assert_eq!(event.sequence, 1);
        assert_eq!(event.kind, SessionOutputKind::Text);
        assert_eq!(event.message, "Agent started working");
        assert_eq!(event.attributes.get("tool"), Some(&"bash".to_string()));
    }
}
