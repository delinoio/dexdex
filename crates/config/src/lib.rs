//! Configuration management for DeliDev.
//!
//! This crate handles configuration loading and parsing for:
//! - Global settings (`~/.delidev/config.toml`)
//! - Repository settings (`.delidev/config.toml`)
//! - VCS credentials (`~/.delidev/credentials.toml`)

mod credentials;
mod error;
mod global;
mod repository;

pub use credentials::*;
pub use error::*;
pub use global::*;
pub use repository::*;

use std::path::{Path, PathBuf};

/// Returns the default DeliDev configuration directory path.
///
/// On Unix-like systems: `~/.delidev`
/// On Windows: `%USERPROFILE%\.delidev`
pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".delidev"))
}

/// Returns the path to the global configuration file.
///
/// `~/.delidev/config.toml`
pub fn global_config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.toml"))
}

/// Returns the path to the credentials file.
///
/// `~/.delidev/credentials.toml`
pub fn credentials_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("credentials.toml"))
}

/// Returns the path to the repository configuration file.
///
/// `.delidev/config.toml` relative to the repository root.
pub fn repository_config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".delidev").join("config.toml")
}

/// Merged configuration with precedence handling.
///
/// Repository settings take precedence over global settings,
/// which take precedence over built-in defaults.
#[derive(Debug, Clone)]
pub struct MergedConfig {
    /// Learning settings (merged).
    pub learning: LearningSettings,
    /// Hotkey settings (from global only).
    pub hotkey: HotkeySettings,
    /// Notification settings (from global only).
    pub notification: NotificationSettings,
    /// Agent configuration for planning (merged).
    pub agent_planning: AgentConfig,
    /// Agent configuration for execution (merged).
    pub agent_execution: AgentConfig,
    /// Agent configuration for chat (from global only).
    pub agent_chat: AgentConfig,
    /// Container settings (from global only).
    pub container: ContainerSettings,
    /// Composite task settings (merged).
    pub composite_task: CompositeTaskSettings,
    /// Concurrency settings (from global only).
    pub concurrency: ConcurrencySettings,
    /// Branch settings (from repository only).
    pub branch: BranchSettings,
    /// Automation settings (from repository only).
    pub automation: AutomationSettings,
}

impl MergedConfig {
    /// Creates a merged configuration from global and repository settings.
    ///
    /// Repository settings take precedence over global settings.
    pub fn merge(global: &GlobalConfig, repo: Option<&RepositoryConfig>) -> Self {
        let default_global = GlobalConfig::default();
        let default_repo = RepositoryConfig::default();

        let repo = repo.unwrap_or(&default_repo);

        // Learning: repo takes precedence if set
        let learning = LearningSettings {
            auto_learn_from_reviews: repo
                .learning
                .as_ref()
                .and_then(|l| l.auto_learn_from_reviews)
                .or(global.learning.as_ref().and_then(|l| l.auto_learn_from_reviews))
                .unwrap_or(default_global.learning.as_ref().map(|l| l.auto_learn_from_reviews.unwrap_or(false)).unwrap_or(false)),
        };

        // Composite task: repo takes precedence if set
        let composite_task = CompositeTaskSettings {
            auto_approve: repo
                .composite_task
                .as_ref()
                .and_then(|c| c.auto_approve)
                .or(global.composite_task.as_ref().and_then(|c| c.auto_approve))
                .unwrap_or(false),
        };

        Self {
            learning,
            hotkey: global.hotkey.clone().unwrap_or_default(),
            notification: global.notification.clone().unwrap_or_default(),
            agent_planning: global
                .agent
                .as_ref()
                .and_then(|a| a.planning.clone())
                .unwrap_or_default(),
            agent_execution: global
                .agent
                .as_ref()
                .and_then(|a| a.execution.clone())
                .unwrap_or_default(),
            agent_chat: global
                .agent
                .as_ref()
                .and_then(|a| a.chat.clone())
                .unwrap_or_default(),
            container: global.container.clone().unwrap_or_default(),
            composite_task,
            concurrency: global.concurrency.clone().unwrap_or_default(),
            branch: repo.branch.clone().unwrap_or_default(),
            automation: repo.automation.clone().unwrap_or_default(),
        }
    }
}

/// Configuration loader that handles loading and merging configurations.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    /// Global configuration.
    global: GlobalConfig,
    /// VCS credentials.
    credentials: VcsCredentials,
}

impl ConfigLoader {
    /// Creates a new configuration loader by loading global config and credentials.
    pub fn load() -> Result<Self, ConfigError> {
        let global = if let Some(path) = global_config_path() {
            if path.exists() {
                GlobalConfig::load(&path)?
            } else {
                GlobalConfig::default()
            }
        } else {
            GlobalConfig::default()
        };

        let credentials = if let Some(path) = credentials_path() {
            if path.exists() {
                VcsCredentials::load(&path)?
            } else {
                VcsCredentials::default()
            }
        } else {
            VcsCredentials::default()
        };

        Ok(Self { global, credentials })
    }

    /// Creates a new configuration loader with the given global config and credentials.
    pub fn new(global: GlobalConfig, credentials: VcsCredentials) -> Self {
        Self { global, credentials }
    }

    /// Returns the global configuration.
    pub fn global(&self) -> &GlobalConfig {
        &self.global
    }

    /// Returns the VCS credentials.
    pub fn credentials(&self) -> &VcsCredentials {
        &self.credentials
    }

    /// Loads and merges configuration for a specific repository.
    pub fn for_repository(&self, repo_root: &Path) -> Result<MergedConfig, ConfigError> {
        let repo_config_path = repository_config_path(repo_root);
        let repo_config = if repo_config_path.exists() {
            Some(RepositoryConfig::load(&repo_config_path)?)
        } else {
            None
        };

        Ok(MergedConfig::merge(&self.global, repo_config.as_ref()))
    }

    /// Returns a merged configuration using only global settings (no repository).
    pub fn global_only(&self) -> MergedConfig {
        MergedConfig::merge(&self.global, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_config_dir() {
        let dir = config_dir();
        assert!(dir.is_some());
        assert!(dir.unwrap().ends_with(".delidev"));
    }

    #[test]
    fn test_global_config_path() {
        let path = global_config_path();
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("config.toml"));
    }

    #[test]
    fn test_credentials_path() {
        let path = credentials_path();
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("credentials.toml"));
    }

    #[test]
    fn test_repository_config_path() {
        let repo_root = Path::new("/home/user/myrepo");
        let path = repository_config_path(repo_root);
        assert_eq!(path, PathBuf::from("/home/user/myrepo/.delidev/config.toml"));
    }

    #[test]
    fn test_merged_config_defaults() {
        let global = GlobalConfig::default();
        let merged = MergedConfig::merge(&global, None);

        assert!(!merged.learning.auto_learn_from_reviews);
        assert!(!merged.composite_task.auto_approve);
        assert_eq!(merged.hotkey.open_chat, "Option+Z");
    }

    #[test]
    fn test_merged_config_repo_precedence() {
        let global = GlobalConfig::default();
        let mut repo = RepositoryConfig::default();
        repo.composite_task = Some(CompositeTaskSettingsOptional {
            auto_approve: Some(true),
        });

        let merged = MergedConfig::merge(&global, Some(&repo));

        assert!(merged.composite_task.auto_approve);
    }

    #[test]
    fn test_config_loader_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let creds_path = temp_dir.path().join("credentials.toml");

        // Write test config
        let mut config_file = std::fs::File::create(&config_path).unwrap();
        writeln!(
            config_file,
            r#"
[hotkey]
openChat = "Alt+X"

[notification]
enabled = false
"#
        )
        .unwrap();

        // Write test credentials
        let mut creds_file = std::fs::File::create(&creds_path).unwrap();
        writeln!(
            creds_file,
            r#"
[github]
token = "ghp_test123"
"#
        )
        .unwrap();

        let global = GlobalConfig::load(&config_path).unwrap();
        let creds = VcsCredentials::load(&creds_path).unwrap();
        let loader = ConfigLoader::new(global, creds);

        assert_eq!(
            loader.global().hotkey.as_ref().unwrap().open_chat,
            "Alt+X"
        );
        assert_eq!(
            loader.credentials().github.as_ref().unwrap().token,
            "ghp_test123"
        );
    }
}
