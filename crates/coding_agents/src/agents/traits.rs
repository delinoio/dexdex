//! Core traits for AI coding agents.

use async_trait::async_trait;
use entities::AiAgentType;
use tokio::sync::mpsc;

use crate::{AgentResult, NormalizedEvent};

/// Handler for TTY input requests from agents.
///
/// When an agent requires user input (e.g., asking a question),
/// this handler is called to obtain the response.
#[async_trait]
pub trait TtyInputHandler: Send + Sync {
    /// Handle a TTY input request.
    ///
    /// # Arguments
    /// * `question` - The question being asked
    /// * `options` - Available options (if multiple choice)
    ///
    /// # Returns
    /// The user's response as a string.
    async fn handle_input(
        &self,
        question: &str,
        options: Option<&[String]>,
    ) -> AgentResult<String>;
}

/// Configuration for running an AI coding agent.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Type of agent to run.
    pub agent_type: AiAgentType,
    /// Working directory for the agent.
    pub working_dir: String,
    /// The prompt/task to execute.
    pub prompt: String,
    /// Optional model override.
    pub model: Option<String>,
    /// Environment variables to set.
    pub env_vars: Vec<(String, String)>,
    /// Maximum execution time in seconds (None = no limit).
    pub timeout_secs: Option<u64>,
}

impl AgentConfig {
    /// Creates a new agent configuration.
    pub fn new(
        agent_type: AiAgentType,
        working_dir: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            agent_type,
            working_dir: working_dir.into(),
            prompt: prompt.into(),
            model: None,
            env_vars: Vec::new(),
            timeout_secs: None,
        }
    }

    /// Sets the model override.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Adds an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Sets the timeout in seconds.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }
}

/// Trait for AI coding agents.
///
/// Each supported AI coding agent implements this trait to provide
/// a unified interface for execution and output handling.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Returns the type of this agent.
    fn agent_type(&self) -> AiAgentType;

    /// Returns the command to invoke this agent.
    fn command(&self) -> &str;

    /// Returns the arguments for the agent command.
    fn args(&self, config: &AgentConfig) -> Vec<String>;

    /// Runs the agent with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Configuration for the agent run
    /// * `event_tx` - Channel to send normalized events
    /// * `tty_handler` - Optional handler for TTY input requests
    ///
    /// # Returns
    /// Result indicating success or failure.
    async fn run(
        &self,
        config: AgentConfig,
        event_tx: mpsc::Sender<NormalizedEvent>,
        tty_handler: Option<Box<dyn TtyInputHandler>>,
    ) -> AgentResult<()>;

    /// Parses a line of output from the agent.
    ///
    /// # Arguments
    /// * `line` - A line of output from the agent
    ///
    /// # Returns
    /// Parsed normalized events (may be empty if line is not parseable).
    fn parse_output(&self, line: &str) -> Vec<NormalizedEvent>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_builder() {
        let config = AgentConfig::new(AiAgentType::ClaudeCode, "/workspace/repo", "Fix the bug")
            .with_model("claude-sonnet-4-20250514")
            .with_env("ANTHROPIC_API_KEY", "test-key")
            .with_timeout(300);

        assert_eq!(config.agent_type, AiAgentType::ClaudeCode);
        assert_eq!(config.working_dir, "/workspace/repo");
        assert_eq!(config.prompt, "Fix the bug");
        assert_eq!(config.model, Some("claude-sonnet-4-20250514".to_string()));
        assert_eq!(config.env_vars.len(), 1);
        assert_eq!(config.timeout_secs, Some(300));
    }
}
