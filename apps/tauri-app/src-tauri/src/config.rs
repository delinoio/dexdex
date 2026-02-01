//! Configuration management for the Tauri app.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Application mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    /// Local single-process mode (embedded server and worker).
    #[default]
    Local,
    /// Remote mode (connects to external server).
    Remote,
}

/// Global application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSettings {
    /// Current application mode.
    pub mode: AppMode,
    /// Remote server URL (only used in remote mode).
    pub server_url: Option<String>,
    /// Global hotkey for opening the app.
    pub hotkey: String,
    /// Whether notifications are enabled.
    pub notifications_enabled: bool,
    /// Default AI agent type.
    pub default_agent_type: String,
    /// Default AI agent model.
    pub default_agent_model: Option<String>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            mode: AppMode::Local,
            server_url: None,
            hotkey: if cfg!(target_os = "macos") {
                "Option+Z".to_string()
            } else {
                "Alt+Z".to_string()
            },
            notifications_enabled: true,
            default_agent_type: "claude_code".to_string(),
            default_agent_model: None,
        }
    }
}

/// Repository-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettings {
    /// Branch name template.
    pub branch_template: Option<String>,
    /// Whether auto-fix for review comments is enabled.
    pub auto_fix_review_comments: bool,
    /// Whether auto-fix for CI failures is enabled.
    pub auto_fix_ci_failures: bool,
    /// Maximum retry attempts for auto-fix.
    pub max_auto_fix_retries: u32,
}

/// Configuration file structure for ~/.delidev/config.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    /// Mode configuration.
    pub mode: Option<ModeConfig>,
    /// Hotkey configuration.
    pub hotkey: Option<HotkeyConfig>,
    /// Notification configuration.
    pub notifications: Option<NotificationsConfig>,
    /// Agent configuration.
    pub agent: Option<AgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub mode: Option<AppMode>,
    pub server_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub open_chat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub default_type: Option<String>,
    pub default_model: Option<String>,
}

/// Gets the DeliDev configuration directory.
pub fn config_dir() -> AppResult<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| AppError::Config("Cannot find home directory".to_string()))?;
    Ok(home.join(".delidev"))
}

/// Gets the path to the global configuration file.
pub fn config_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// Gets the path to the data directory (for SQLite database, etc.).
pub fn data_dir() -> AppResult<PathBuf> {
    let dir = config_dir()?.join("data");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Loads the configuration file.
pub fn load_config() -> AppResult<ConfigFile> {
    let path = config_file_path()?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        toml::from_str(&content).map_err(|e| AppError::Config(format!("Failed to parse config: {}", e)))
    } else {
        Ok(ConfigFile::default())
    }
}

/// Saves the configuration file.
pub fn save_config(config: &ConfigFile) -> AppResult<()> {
    let path = config_file_path()?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)
        .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Converts a ConfigFile to GlobalSettings.
pub fn config_to_settings(config: &ConfigFile) -> GlobalSettings {
    let mut settings = GlobalSettings::default();

    if let Some(mode_config) = &config.mode {
        if let Some(mode) = mode_config.mode {
            settings.mode = mode;
        }
        settings.server_url.clone_from(&mode_config.server_url);
    }

    if let Some(hotkey_config) = &config.hotkey {
        if let Some(hotkey) = &hotkey_config.open_chat {
            settings.hotkey = hotkey.clone();
        }
    }

    if let Some(notif_config) = &config.notifications {
        if let Some(enabled) = notif_config.enabled {
            settings.notifications_enabled = enabled;
        }
    }

    if let Some(agent_config) = &config.agent {
        if let Some(agent_type) = &agent_config.default_type {
            settings.default_agent_type = agent_type.clone();
        }
        settings.default_agent_model.clone_from(&agent_config.default_model);
    }

    settings
}

/// Converts GlobalSettings to a ConfigFile.
pub fn settings_to_config(settings: &GlobalSettings) -> ConfigFile {
    ConfigFile {
        mode: Some(ModeConfig {
            mode: Some(settings.mode),
            server_url: settings.server_url.clone(),
        }),
        hotkey: Some(HotkeyConfig {
            open_chat: Some(settings.hotkey.clone()),
        }),
        notifications: Some(NotificationsConfig {
            enabled: Some(settings.notifications_enabled),
        }),
        agent: Some(AgentConfig {
            default_type: Some(settings.default_agent_type.clone()),
            default_model: settings.default_agent_model.clone(),
        }),
    }
}
