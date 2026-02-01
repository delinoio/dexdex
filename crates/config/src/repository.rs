//! Repository-specific configuration settings.
//!
//! Location: `.delidev/config.toml` (committed to git)

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{CompositeTaskSettingsOptional, ConfigError, LearningSettingsOptional};

/// Repository-specific configuration.
///
/// Stored at `.delidev/config.toml` in the repository root.
/// These settings are committed to git and shared with the team.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Branch naming configuration.
    #[serde(default)]
    pub branch: Option<BranchSettings>,

    /// Automation settings.
    #[serde(default)]
    pub automation: Option<AutomationSettings>,

    /// Learning settings (overrides global).
    #[serde(default)]
    pub learning: Option<LearningSettingsOptional>,

    /// Composite task settings (overrides global).
    #[serde(default)]
    pub composite_task: Option<CompositeTaskSettingsOptional>,
}

impl RepositoryConfig {
    /// Loads repository configuration from a file.
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

    /// Saves repository configuration to a file.
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

/// Branch naming configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchSettings {
    /// Template for branch names.
    ///
    /// Available variables:
    /// - `${taskId}` - The task ID
    /// - `${slug}` - A URL-safe slug derived from the task title
    ///
    /// Example: `feature/${taskId}-${slug}`
    #[serde(default = "default_branch_template")]
    pub template: String,
}

impl Default for BranchSettings {
    fn default() -> Self {
        Self {
            template: default_branch_template(),
        }
    }
}

fn default_branch_template() -> String {
    "feature/${taskId}-${slug}".to_string()
}

impl BranchSettings {
    /// Generates a branch name from the template.
    ///
    /// # Arguments
    ///
    /// * `task_id` - The task ID
    /// * `title` - The task title (will be converted to a slug)
    pub fn generate_branch_name(&self, task_id: &str, title: &str) -> String {
        let slug = Self::slugify(title);
        self.template
            .replace("${taskId}", task_id)
            .replace("${slug}", &slug)
    }

    /// Converts a string to a URL-safe slug.
    fn slugify(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    '-'
                } else {
                    // Skip other characters
                    '\0'
                }
            })
            .filter(|c| *c != '\0')
            .collect::<String>()
            // Remove consecutive dashes
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

/// Automation settings for PR management.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationSettings {
    /// Whether to automatically fix review comments.
    #[serde(default)]
    pub auto_fix_review_comments: bool,

    /// Filter for which review comments to auto-fix.
    #[serde(default)]
    pub auto_fix_review_comments_filter: ReviewCommentFilter,

    /// Whether to automatically fix CI failures.
    #[serde(default)]
    pub auto_fix_ci_failures: bool,

    /// Maximum number of auto-fix attempts.
    #[serde(default = "default_max_auto_fix_attempts")]
    pub max_auto_fix_attempts: u32,
}

impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            auto_fix_review_comments: false,
            auto_fix_review_comments_filter: ReviewCommentFilter::WriteAccessOnly,
            auto_fix_ci_failures: false,
            max_auto_fix_attempts: default_max_auto_fix_attempts(),
        }
    }
}

fn default_max_auto_fix_attempts() -> u32 {
    3
}

/// Filter for which review comments should trigger auto-fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewCommentFilter {
    /// Only auto-fix comments from users with write access.
    #[default]
    WriteAccessOnly,
    /// Auto-fix comments from any user.
    All,
    /// Only auto-fix comments from repository maintainers.
    MaintainersOnly,
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_default_repository_config() {
        let config = RepositoryConfig::default();
        assert!(config.branch.is_none());
        assert!(config.automation.is_none());
    }

    #[test]
    fn test_parse_full_repository_config() {
        let toml_content = r#"
[branch]
template = "feature/${taskId}-${slug}"

[automation]
autoFixReviewComments = true
autoFixReviewCommentsFilter = "write_access_only"
autoFixCiFailures = true
maxAutoFixAttempts = 5

[learning]
autoLearnFromReviews = true

[composite_task]
auto_approve = true
"#;

        let config: RepositoryConfig = toml::from_str(toml_content).unwrap();

        assert_eq!(config.branch.unwrap().template, "feature/${taskId}-${slug}");

        let automation = config.automation.unwrap();
        assert!(automation.auto_fix_review_comments);
        assert_eq!(
            automation.auto_fix_review_comments_filter,
            ReviewCommentFilter::WriteAccessOnly
        );
        assert!(automation.auto_fix_ci_failures);
        assert_eq!(automation.max_auto_fix_attempts, 5);

        assert!(config.learning.unwrap().auto_learn_from_reviews.unwrap());
        assert!(config.composite_task.unwrap().auto_approve.unwrap());
    }

    #[test]
    fn test_parse_minimal_repository_config() {
        let toml_content = r#"
[branch]
template = "fix/${slug}"
"#;

        let config: RepositoryConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.branch.unwrap().template, "fix/${slug}");
        assert!(config.automation.is_none());
    }

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[automation]
autoFixCiFailures = true
"#
        )
        .unwrap();

        let config = RepositoryConfig::load(file.path()).unwrap();
        assert!(config.automation.unwrap().auto_fix_ci_failures);
    }

    #[test]
    fn test_save_to_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        let config = RepositoryConfig {
            branch: Some(BranchSettings {
                template: "custom/${taskId}".to_string(),
            }),
            ..Default::default()
        };

        config.save(&path).unwrap();

        let loaded = RepositoryConfig::load(&path).unwrap();
        assert_eq!(loaded.branch.unwrap().template, "custom/${taskId}");
    }

    #[test]
    fn test_default_branch_settings() {
        let settings = BranchSettings::default();
        assert_eq!(settings.template, "feature/${taskId}-${slug}");
    }

    #[test]
    fn test_branch_name_generation() {
        let settings = BranchSettings::default();
        let branch = settings.generate_branch_name("123", "Add user authentication");
        assert_eq!(branch, "feature/123-add-user-authentication");
    }

    #[test]
    fn test_branch_name_generation_with_special_chars() {
        let settings = BranchSettings::default();
        let branch = settings.generate_branch_name("456", "Fix bug: handle 404 errors!");
        assert_eq!(branch, "feature/456-fix-bug-handle-404-errors");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(BranchSettings::slugify("Hello World"), "hello-world");
        assert_eq!(BranchSettings::slugify("Fix Bug #123"), "fix-bug-123");
        assert_eq!(
            BranchSettings::slugify("  Multiple   Spaces  "),
            "multiple-spaces"
        );
        assert_eq!(
            BranchSettings::slugify("Special!@#$%^&*()Chars"),
            "specialchars"
        );
        assert_eq!(
            BranchSettings::slugify("Already-kebab-case"),
            "already-kebab-case"
        );
    }

    #[test]
    fn test_custom_branch_template() {
        let settings = BranchSettings {
            template: "delidev/${slug}/${taskId}".to_string(),
        };
        let branch = settings.generate_branch_name("abc", "My Feature");
        assert_eq!(branch, "delidev/my-feature/abc");
    }

    #[test]
    fn test_default_automation_settings() {
        let settings = AutomationSettings::default();
        assert!(!settings.auto_fix_review_comments);
        assert!(!settings.auto_fix_ci_failures);
        assert_eq!(
            settings.auto_fix_review_comments_filter,
            ReviewCommentFilter::WriteAccessOnly
        );
        assert_eq!(settings.max_auto_fix_attempts, 3);
    }

    #[test]
    fn test_review_comment_filter_parsing() {
        let config: RepositoryConfig = toml::from_str(
            r#"
[automation]
autoFixReviewCommentsFilter = "all"
"#,
        )
        .unwrap();
        assert_eq!(
            config.automation.unwrap().auto_fix_review_comments_filter,
            ReviewCommentFilter::All
        );

        let config: RepositoryConfig = toml::from_str(
            r#"
[automation]
autoFixReviewCommentsFilter = "maintainers_only"
"#,
        )
        .unwrap();
        assert_eq!(
            config.automation.unwrap().auto_fix_review_comments_filter,
            ReviewCommentFilter::MaintainersOnly
        );
    }
}
