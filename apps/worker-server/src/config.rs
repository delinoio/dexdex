//! Worker server configuration.

use entities::AiAgentType;

/// Worker server configuration.
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Main server URL to connect to.
    pub main_server_url: String,
    /// Worker name/identifier.
    pub worker_name: String,
    /// Log level.
    pub log_level: String,
    /// Poll interval in milliseconds.
    pub poll_interval_ms: u64,
    /// Default AI agent type.
    pub agent_type: AiAgentType,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            main_server_url: "http://localhost:54871".to_string(),
            worker_name: hostname(),
            log_level: "info".to_string(),
            poll_interval_ms: 2000,
            agent_type: AiAgentType::ClaudeCode,
        }
    }
}

impl WorkerConfig {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Self {
        let default = Self::default();

        let agent_type = std::env::var("DEXDEX_AGENT_TYPE")
            .ok()
            .and_then(|s| parse_agent_type(&s))
            .unwrap_or(default.agent_type);

        Self {
            main_server_url: std::env::var("DEXDEX_MAIN_SERVER_URL")
                .unwrap_or(default.main_server_url),
            worker_name: std::env::var("DEXDEX_WORKER_NAME").unwrap_or(default.worker_name),
            log_level: std::env::var("DEXDEX_LOG_LEVEL").unwrap_or(default.log_level),
            poll_interval_ms: std::env::var("DEXDEX_POLL_INTERVAL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.poll_interval_ms),
            agent_type,
        }
    }
}

/// Returns the hostname of the current machine.
fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "worker".to_string())
}

/// Parses an agent type string.
fn parse_agent_type(s: &str) -> Option<AiAgentType> {
    match s.to_lowercase().as_str() {
        "claude_code" | "claudecode" => Some(AiAgentType::ClaudeCode),
        "open_code" | "opencode" => Some(AiAgentType::OpenCode),
        "gemini_cli" | "geminicli" => Some(AiAgentType::GeminiCli),
        "codex_cli" | "codexcli" => Some(AiAgentType::CodexCli),
        "aider" => Some(AiAgentType::Aider),
        "amp" => Some(AiAgentType::Amp),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorkerConfig::default();
        assert_eq!(config.main_server_url, "http://localhost:54871");
        assert_eq!(config.poll_interval_ms, 2000);
        assert_eq!(config.agent_type, AiAgentType::ClaudeCode);
    }

    #[test]
    fn test_parse_agent_type() {
        assert_eq!(
            parse_agent_type("claude_code"),
            Some(AiAgentType::ClaudeCode)
        );
        assert_eq!(
            parse_agent_type("ClaudeCode"),
            Some(AiAgentType::ClaudeCode)
        );
        assert_eq!(parse_agent_type("aider"), Some(AiAgentType::Aider));
        assert_eq!(parse_agent_type("unknown"), None);
    }
}
