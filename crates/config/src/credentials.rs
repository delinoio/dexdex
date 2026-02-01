//! VCS provider credentials.
//!
//! Location: `~/.delidev/credentials.toml`

use crate::{validate_config_path, ConfigError};
use entities::VcsProviderType;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// VCS provider credentials.
///
/// Stored at `~/.delidev/credentials.toml`.
/// This file should NOT be committed to version control.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VcsCredentials {
    /// GitHub credentials.
    #[serde(default)]
    pub github: Option<GithubCredentials>,

    /// GitLab credentials.
    #[serde(default)]
    pub gitlab: Option<GitlabCredentials>,

    /// Bitbucket credentials.
    #[serde(default)]
    pub bitbucket: Option<BitbucketCredentials>,
}

impl VcsCredentials {
    /// Loads VCS credentials from a file.
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

    /// Saves VCS credentials to a file.
    ///
    /// The path must be within the DeliDev configuration directory (`~/.delidev`)
    /// to prevent path traversal attacks.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        // Validate path is within the config directory
        validate_config_path(path)?;
        self.save_unchecked(path)
    }

    /// Saves VCS credentials to a file without path validation.
    ///
    /// # Safety
    /// This method bypasses path validation. Only use this in test code
    /// or when you have already validated the path.
    #[doc(hidden)]
    pub fn save_unchecked(&self, path: &Path) -> Result<(), ConfigError> {
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

    /// Returns the token for a specific VCS provider.
    pub fn get_token(&self, provider: VcsProviderType) -> Option<&str> {
        match provider {
            VcsProviderType::Github => self.github.as_ref().map(|c| c.token.as_str()),
            VcsProviderType::Gitlab => self.gitlab.as_ref().map(|c| c.token.as_str()),
            VcsProviderType::Bitbucket => {
                // Bitbucket uses app_password, not token
                self.bitbucket.as_ref().map(|c| c.app_password.as_str())
            }
        }
    }

    /// Returns whether credentials are configured for a specific provider.
    pub fn has_credentials(&self, provider: VcsProviderType) -> bool {
        match provider {
            VcsProviderType::Github => self
                .github
                .as_ref()
                .is_some_and(|c| !c.token.is_empty()),
            VcsProviderType::Gitlab => self
                .gitlab
                .as_ref()
                .is_some_and(|c| !c.token.is_empty()),
            VcsProviderType::Bitbucket => self.bitbucket.as_ref().is_some_and(|c| {
                !c.username.is_empty() && !c.app_password.is_empty()
            }),
        }
    }
}

/// GitHub credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubCredentials {
    /// Personal access token (PAT) or OAuth token.
    ///
    /// Should have appropriate scopes for repository access.
    /// Classic PAT: `repo` scope
    /// Fine-grained PAT: Repository access with Contents and Pull requests permissions
    pub token: String,
}

/// GitLab credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitlabCredentials {
    /// Personal access token.
    ///
    /// Should have `api` scope for full repository access.
    pub token: String,
}

/// Bitbucket credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitbucketCredentials {
    /// Bitbucket username.
    pub username: String,

    /// App password.
    ///
    /// Create at: Bitbucket Settings > Personal settings > App passwords
    /// Required permissions: Repositories (Read, Write)
    pub app_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_vcs_credentials() {
        let creds = VcsCredentials::default();
        assert!(creds.github.is_none());
        assert!(creds.gitlab.is_none());
        assert!(creds.bitbucket.is_none());
    }

    #[test]
    fn test_parse_github_credentials() {
        let toml_content = r#"
[github]
token = "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
"#;

        let creds: VcsCredentials = toml::from_str(toml_content).unwrap();
        assert_eq!(
            creds.github.unwrap().token,
            "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        );
    }

    #[test]
    fn test_parse_gitlab_credentials() {
        let toml_content = r#"
[gitlab]
token = "glpat-xxxxxxxxxxxxxxxxxxxx"
"#;

        let creds: VcsCredentials = toml::from_str(toml_content).unwrap();
        assert_eq!(
            creds.gitlab.unwrap().token,
            "glpat-xxxxxxxxxxxxxxxxxxxx"
        );
    }

    #[test]
    fn test_parse_bitbucket_credentials() {
        let toml_content = r#"
[bitbucket]
username = "myuser"
app_password = "xxxxxxxxxxxxxxxxxxxx"
"#;

        let creds: VcsCredentials = toml::from_str(toml_content).unwrap();
        let bb = creds.bitbucket.unwrap();
        assert_eq!(bb.username, "myuser");
        assert_eq!(bb.app_password, "xxxxxxxxxxxxxxxxxxxx");
    }

    #[test]
    fn test_parse_all_credentials() {
        let toml_content = r#"
[github]
token = "ghp_github"

[gitlab]
token = "glpat-gitlab"

[bitbucket]
username = "bbuser"
app_password = "bbpass"
"#;

        let creds: VcsCredentials = toml::from_str(toml_content).unwrap();
        assert_eq!(creds.github.unwrap().token, "ghp_github");
        assert_eq!(creds.gitlab.unwrap().token, "glpat-gitlab");
        assert_eq!(creds.bitbucket.as_ref().unwrap().username, "bbuser");
        assert_eq!(creds.bitbucket.unwrap().app_password, "bbpass");
    }

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[github]
token = "ghp_test123"
"#
        )
        .unwrap();

        let creds = VcsCredentials::load(file.path()).unwrap();
        assert_eq!(creds.github.unwrap().token, "ghp_test123");
    }

    #[test]
    fn test_save_to_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        let mut creds = VcsCredentials::default();
        creds.github = Some(GithubCredentials {
            token: "ghp_saved".to_string(),
        });

        creds.save_unchecked(&path).unwrap();

        let loaded = VcsCredentials::load(&path).unwrap();
        assert_eq!(loaded.github.unwrap().token, "ghp_saved");
    }

    #[test]
    fn test_get_token() {
        let mut creds = VcsCredentials::default();
        creds.github = Some(GithubCredentials {
            token: "ghp_test".to_string(),
        });
        creds.gitlab = Some(GitlabCredentials {
            token: "glpat_test".to_string(),
        });
        creds.bitbucket = Some(BitbucketCredentials {
            username: "user".to_string(),
            app_password: "pass".to_string(),
        });

        assert_eq!(
            creds.get_token(VcsProviderType::Github),
            Some("ghp_test")
        );
        assert_eq!(
            creds.get_token(VcsProviderType::Gitlab),
            Some("glpat_test")
        );
        assert_eq!(
            creds.get_token(VcsProviderType::Bitbucket),
            Some("pass")
        );
    }

    #[test]
    fn test_get_token_missing() {
        let creds = VcsCredentials::default();
        assert!(creds.get_token(VcsProviderType::Github).is_none());
        assert!(creds.get_token(VcsProviderType::Gitlab).is_none());
        assert!(creds.get_token(VcsProviderType::Bitbucket).is_none());
    }

    #[test]
    fn test_has_credentials() {
        let mut creds = VcsCredentials::default();

        assert!(!creds.has_credentials(VcsProviderType::Github));

        creds.github = Some(GithubCredentials {
            token: "ghp_test".to_string(),
        });
        assert!(creds.has_credentials(VcsProviderType::Github));

        // Empty token should return false
        creds.github = Some(GithubCredentials {
            token: "".to_string(),
        });
        assert!(!creds.has_credentials(VcsProviderType::Github));
    }

    #[test]
    fn test_has_bitbucket_credentials() {
        let mut creds = VcsCredentials::default();

        assert!(!creds.has_credentials(VcsProviderType::Bitbucket));

        // Only username is not enough
        creds.bitbucket = Some(BitbucketCredentials {
            username: "user".to_string(),
            app_password: "".to_string(),
        });
        assert!(!creds.has_credentials(VcsProviderType::Bitbucket));

        // Both required
        creds.bitbucket = Some(BitbucketCredentials {
            username: "user".to_string(),
            app_password: "pass".to_string(),
        });
        assert!(creds.has_credentials(VcsProviderType::Bitbucket));
    }
}
