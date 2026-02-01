//! Global configuration settings.
//!
//! Location: `~/.delidev/config.toml`

use crate::ConfigError;
use entities::AiAgentType;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Global configuration for DeliDev.
///
/// Stored at `~/.delidev/config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Learning settings.
    #[serde(default)]
    pub learning: Option<LearningSettingsOptional>,

    /// Hotkey configuration.
    #[serde(default)]
    pub hotkey: Option<HotkeySettings>,

    /// Notification preferences.
    #[serde(default)]
    pub notification: Option<NotificationSettings>,

    /// Agent configurations.
    #[serde(default)]
    pub agent: Option<AgentSettings>,

    /// Container settings.
    #[serde(default)]
    pub container: Option<ContainerSettings>,

    /// Composite task settings.
    #[serde(default)]
    pub composite_task: Option<CompositeTaskSettingsOptional>,

    /// Concurrency settings.
    #[serde(default)]
    pub concurrency: Option<ConcurrencySettings>,
}

impl GlobalConfig {
    /// Loads global configuration from a file.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadFile {
            path: path.to_path_buf(),
            source: e,
        })?;

        toml::from_str(&contents).map_err(|e| ConfigError::ParseToml {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Saves global configuration to a file.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::WriteFile {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents).map_err(|e| ConfigError::WriteFile {
            path: path.to_path_buf(),
            source: e,
        })?;

        Ok(())
    }
}

/// Learning settings (optional values for parsing).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningSettingsOptional {
    /// Whether to automatically learn from reviews.
    #[serde(default)]
    pub auto_learn_from_reviews: Option<bool>,
}

/// Learning settings (resolved values).
#[derive(Debug, Clone, Default)]
pub struct LearningSettings {
    /// Whether to automatically learn from reviews.
    pub auto_learn_from_reviews: bool,
}

/// Hotkey configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeySettings {
    /// Hotkey to open chat.
    /// Default: "Option+Z" (macOS) / "Alt+Z" (Windows/Linux)
    #[serde(default = "default_open_chat_hotkey")]
    pub open_chat: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            open_chat: default_open_chat_hotkey(),
        }
    }
}

fn default_open_chat_hotkey() -> String {
    "Option+Z".to_string()
}

/// Notification preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    /// Whether notifications are enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Notify on approval requests.
    #[serde(default = "default_true")]
    pub approval_request: bool,

    /// Notify on user questions (TTY input).
    #[serde(default = "default_true")]
    pub user_question: bool,

    /// Notify when review is ready.
    #[serde(default = "default_true")]
    pub review_ready: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            approval_request: true,
            user_question: true,
            review_ready: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Agent settings for different contexts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentSettings {
    /// Agent configuration for planning tasks.
    #[serde(default)]
    pub planning: Option<AgentConfig>,

    /// Agent configuration for execution tasks.
    #[serde(default)]
    pub execution: Option<AgentConfig>,

    /// Agent configuration for chat.
    #[serde(default)]
    pub chat: Option<AgentConfig>,
}

/// Configuration for an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentConfig {
    /// Type of AI agent.
    #[serde(rename = "type", default)]
    pub agent_type: AiAgentType,

    /// Model to use.
    #[serde(default = "default_model")]
    pub model: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_type: AiAgentType::ClaudeCode,
            model: default_model(),
        }
    }
}

fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

/// Container runtime settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContainerSettings {
    /// Container runtime to use.
    #[serde(default = "default_runtime")]
    pub runtime: ContainerRuntime,

    /// Whether to use containers for task execution.
    #[serde(default = "default_true")]
    pub use_container: bool,
}

impl Default for ContainerSettings {
    fn default() -> Self {
        Self {
            runtime: ContainerRuntime::Docker,
            use_container: true,
        }
    }
}

fn default_runtime() -> ContainerRuntime {
    ContainerRuntime::Docker
}

/// Container runtime type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerRuntime {
    /// Docker
    #[default]
    Docker,
    /// Podman
    Podman,
}

/// Composite task settings (optional values for parsing).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CompositeTaskSettingsOptional {
    /// Whether to auto-approve composite task plans.
    #[serde(default)]
    pub auto_approve: Option<bool>,
}

/// Composite task settings (resolved values).
#[derive(Debug, Clone, Default)]
pub struct CompositeTaskSettings {
    /// Whether to auto-approve composite task plans.
    pub auto_approve: bool,
}

/// Concurrency settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConcurrencySettings {
    /// Maximum number of concurrent agent sessions.
    #[serde(default)]
    pub max_concurrent_sessions: Option<u32>,
}

impl Default for ConcurrencySettings {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: None, // No limit by default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_global_config() {
        let config = GlobalConfig::default();
        assert!(config.learning.is_none());
        assert!(config.hotkey.is_none());
        assert!(config.notification.is_none());
    }

    #[test]
    fn test_parse_full_config() {
        let toml_content = r#"
[learning]
autoLearnFromReviews = true

[hotkey]
openChat = "Alt+X"

[notification]
enabled = true
approvalRequest = true
userQuestion = true
reviewReady = false

[agent.planning]
type = "claude_code"
model = "claude-sonnet-4-20250514"

[agent.execution]
type = "open_code"
model = "gpt-4"

[agent.chat]
type = "claude_code"
model = "claude-sonnet-4-20250514"

[container]
runtime = "docker"
use_container = true

[composite_task]
auto_approve = true

[concurrency]
max_concurrent_sessions = 5
"#;

        let config: GlobalConfig = toml::from_str(toml_content).unwrap();

        assert!(config.learning.unwrap().auto_learn_from_reviews.unwrap());
        assert_eq!(config.hotkey.unwrap().open_chat, "Alt+X");

        let notification = config.notification.unwrap();
        assert!(notification.enabled);
        assert!(!notification.review_ready);

        let agent = config.agent.unwrap();
        assert_eq!(
            agent.planning.unwrap().agent_type,
            AiAgentType::ClaudeCode
        );
        assert_eq!(agent.execution.unwrap().agent_type, AiAgentType::OpenCode);

        assert!(config.composite_task.unwrap().auto_approve.unwrap());
        assert_eq!(config.concurrency.unwrap().max_concurrent_sessions, Some(5));
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml_content = r#"
[hotkey]
openChat = "Ctrl+Space"
"#;

        let config: GlobalConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.hotkey.unwrap().open_chat, "Ctrl+Space");
        assert!(config.learning.is_none());
    }

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[hotkey]
openChat = "Meta+Z"
"#
        )
        .unwrap();

        let config = GlobalConfig::load(file.path()).unwrap();
        assert_eq!(config.hotkey.unwrap().open_chat, "Meta+Z");
    }

    #[test]
    fn test_save_to_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        let mut config = GlobalConfig::default();
        config.hotkey = Some(HotkeySettings {
            open_chat: "Ctrl+K".to_string(),
        });

        config.save(&path).unwrap();

        let loaded = GlobalConfig::load(&path).unwrap();
        assert_eq!(loaded.hotkey.unwrap().open_chat, "Ctrl+K");
    }

    #[test]
    fn test_default_hotkey_settings() {
        let settings = HotkeySettings::default();
        assert_eq!(settings.open_chat, "Option+Z");
    }

    #[test]
    fn test_default_notification_settings() {
        let settings = NotificationSettings::default();
        assert!(settings.enabled);
        assert!(settings.approval_request);
        assert!(settings.user_question);
        assert!(settings.review_ready);
    }

    #[test]
    fn test_default_agent_config() {
        let config = AgentConfig::default();
        assert_eq!(config.agent_type, AiAgentType::ClaudeCode);
        assert_eq!(config.model, "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_default_container_settings() {
        let settings = ContainerSettings::default();
        assert_eq!(settings.runtime, ContainerRuntime::Docker);
        assert!(settings.use_container);
    }
}
