//! Agent-related entity definitions.

use serde::{Deserialize, Serialize};

/// Type of AI coding agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AiAgentType {
    /// Claude Code - Anthropic's terminal-based agentic coding tool.
    #[default]
    ClaudeCode,
    /// OpenCode - Open-source Claude Code alternative.
    OpenCode,
    /// Gemini CLI - Google's open-source AI agent.
    GeminiCli,
    /// Codex CLI - OpenAI's terminal-based coding assistant.
    CodexCli,
    /// Aider - Open-source CLI for multi-file changes.
    Aider,
    /// Amp - Sourcegraph's agentic coding CLI.
    Amp,
}

impl AiAgentType {
    /// Returns the command used to invoke this agent.
    pub fn command(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude",
            Self::OpenCode => "opencode",
            Self::GeminiCli => "gemini",
            Self::CodexCli => "codex",
            Self::Aider => "aider",
            Self::Amp => "amp",
        }
    }

    /// Returns the agent type as a string (snake_case format).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude_code",
            Self::OpenCode => "open_code",
            Self::GeminiCli => "gemini_cli",
            Self::CodexCli => "codex_cli",
            Self::Aider => "aider",
            Self::Amp => "amp",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_agent_type_command() {
        assert_eq!(AiAgentType::ClaudeCode.command(), "claude");
        assert_eq!(AiAgentType::OpenCode.command(), "opencode");
        assert_eq!(AiAgentType::GeminiCli.command(), "gemini");
        assert_eq!(AiAgentType::CodexCli.command(), "codex");
        assert_eq!(AiAgentType::Aider.command(), "aider");
        assert_eq!(AiAgentType::Amp.command(), "amp");
    }

    #[test]
    fn test_ai_agent_type_as_str() {
        assert_eq!(AiAgentType::ClaudeCode.as_str(), "claude_code");
        assert_eq!(AiAgentType::GeminiCli.as_str(), "gemini_cli");
    }
}
