//! Agent runner for executing AI coding agents.

use entities::AiAgentType;
use tokio::sync::mpsc;

use crate::{
    Agent, AgentConfig, AgentResult, AiderAgent, AmpAgent, ClaudeCodeAgent, CodexCliAgent,
    GeminiCliAgent, NormalizedEvent, OpenCodeAgent, TtyInputHandler,
};

/// Creates an agent instance based on the agent type.
pub fn create_agent(agent_type: AiAgentType) -> Box<dyn Agent> {
    match agent_type {
        AiAgentType::ClaudeCode => Box::new(ClaudeCodeAgent::new()),
        AiAgentType::OpenCode => Box::new(OpenCodeAgent::new()),
        AiAgentType::GeminiCli => Box::new(GeminiCliAgent::new()),
        AiAgentType::CodexCli => Box::new(CodexCliAgent::new()),
        AiAgentType::Aider => Box::new(AiderAgent::new()),
        AiAgentType::Amp => Box::new(AmpAgent::new()),
    }
}

/// Runs an AI coding agent with the given configuration.
///
/// # Arguments
/// * `config` - Configuration for the agent run
/// * `tty_handler` - Optional handler for TTY input requests
///
/// # Returns
/// A tuple of (event receiver, result future).
pub async fn run_agent(
    config: AgentConfig,
    tty_handler: Option<Box<dyn TtyInputHandler>>,
) -> (mpsc::Receiver<NormalizedEvent>, AgentResult<()>) {
    let (tx, rx) = mpsc::channel(1024);
    let agent = create_agent(config.agent_type);

    let result = agent.run(config, tx, tty_handler).await;
    (rx, result)
}

/// Agent runner that manages the lifecycle of an agent execution.
pub struct AgentRunner {
    agent: Box<dyn Agent>,
    event_tx: Option<mpsc::Sender<NormalizedEvent>>,
}

impl AgentRunner {
    /// Creates a new agent runner for the specified agent type.
    pub fn new(agent_type: AiAgentType) -> Self {
        Self {
            agent: create_agent(agent_type),
            event_tx: None,
        }
    }

    /// Creates a new agent runner with a custom event channel.
    pub fn with_event_channel(agent_type: AiAgentType, tx: mpsc::Sender<NormalizedEvent>) -> Self {
        Self {
            agent: create_agent(agent_type),
            event_tx: Some(tx),
        }
    }

    /// Runs the agent with the given configuration.
    pub async fn run(
        self,
        config: AgentConfig,
        tty_handler: Option<Box<dyn TtyInputHandler>>,
    ) -> (Option<mpsc::Receiver<NormalizedEvent>>, AgentResult<()>) {
        let (tx, rx) = if let Some(tx) = self.event_tx {
            (tx, None)
        } else {
            let (tx, rx) = mpsc::channel(1024);
            (tx, Some(rx))
        };

        let result = self.agent.run(config, tx, tty_handler).await;
        (rx, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_agent() {
        let agent = create_agent(AiAgentType::ClaudeCode);
        assert_eq!(agent.agent_type(), AiAgentType::ClaudeCode);

        let agent = create_agent(AiAgentType::Aider);
        assert_eq!(agent.agent_type(), AiAgentType::Aider);
    }

    #[test]
    fn test_agent_runner_creation() {
        let runner = AgentRunner::new(AiAgentType::OpenCode);
        assert!(runner.event_tx.is_none());

        let (tx, _rx) = mpsc::channel(100);
        let runner = AgentRunner::with_event_channel(AiAgentType::GeminiCli, tx);
        assert!(runner.event_tx.is_some());
    }
}
