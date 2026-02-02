//! Auto-fix service for handling review comments and CI failures.
//!
//! This service creates AgentTasks to automatically fix issues
//! detected from PR review comments or CI failures.

use std::sync::Arc;

use entities::{AgentTask, UnitTask, UnitTaskStatus};
use task_store::TaskStore;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

/// Errors that can occur during auto-fix operations.
#[derive(Debug, Error)]
pub enum AutoFixError {
    /// Task not found.
    #[error("task not found: {0}")]
    TaskNotFound(Uuid),

    /// Task is not in a fixable state.
    #[error("task {0} is not in a fixable state (current: {1:?})")]
    TaskNotFixable(Uuid, UnitTaskStatus),

    /// Max auto-fix attempts exceeded.
    #[error("max auto-fix attempts ({max}) exceeded for task {task_id}")]
    MaxAttemptsExceeded { task_id: Uuid, max: u32 },

    /// Store error.
    #[error("store error: {0}")]
    StoreError(#[from] task_store::TaskStoreError),
}

/// Context for creating an auto-fix task from a review comment.
#[derive(Debug, Clone)]
pub struct ReviewCommentContext {
    /// The reviewer's username.
    pub reviewer: String,
    /// The file path being commented on.
    pub file_path: String,
    /// The line number (if applicable).
    pub line_number: Option<i32>,
    /// The comment body.
    pub comment: String,
    /// URL to the comment.
    pub comment_url: String,
}

/// Context for creating an auto-fix task from a CI failure.
#[derive(Debug, Clone)]
pub struct CiFailureContext {
    /// Name of the check/job that failed.
    pub check_name: String,
    /// Failure summary (if available).
    pub summary: Option<String>,
    /// Detailed log output (if available).
    pub log_output: Option<String>,
    /// URL to the check run.
    pub check_url: String,
}

/// Auto-fix service configuration.
#[derive(Debug, Clone)]
pub struct AutoFixConfig {
    /// Whether auto-fix for review comments is enabled.
    pub auto_fix_review_comments: bool,
    /// Whether auto-fix for CI failures is enabled.
    pub auto_fix_ci_failures: bool,
    /// Maximum number of auto-fix attempts.
    pub max_attempts: u32,
}

impl Default for AutoFixConfig {
    fn default() -> Self {
        Self {
            auto_fix_review_comments: false,
            auto_fix_ci_failures: false,
            max_attempts: 3,
        }
    }
}

/// Service for managing auto-fix tasks.
pub struct AutoFixService<S: TaskStore> {
    store: Arc<S>,
    config: AutoFixConfig,
}

impl<S: TaskStore> AutoFixService<S> {
    /// Creates a new auto-fix service.
    pub fn new(store: Arc<S>, config: AutoFixConfig) -> Self {
        Self { store, config }
    }

    /// Creates an auto-fix task for a review comment.
    pub async fn create_review_comment_fix(
        &self,
        unit_task_id: Uuid,
        context: ReviewCommentContext,
    ) -> Result<AgentTask, AutoFixError> {
        if !self.config.auto_fix_review_comments {
            warn!("Auto-fix for review comments is disabled");
            return Err(AutoFixError::TaskNotFixable(
                unit_task_id,
                UnitTaskStatus::InReview,
            ));
        }

        // Get the unit task
        let unit_task = self
            .store
            .get_unit_task(unit_task_id)
            .await?
            .ok_or(AutoFixError::TaskNotFound(unit_task_id))?;

        // Check if task is in a fixable state
        if !matches!(
            unit_task.status,
            UnitTaskStatus::InReview | UnitTaskStatus::PrOpen
        ) {
            return Err(AutoFixError::TaskNotFixable(unit_task_id, unit_task.status));
        }

        // Check max attempts
        let current_attempts = unit_task.auto_fix_task_ids.len() as u32;
        if current_attempts >= self.config.max_attempts {
            return Err(AutoFixError::MaxAttemptsExceeded {
                task_id: unit_task_id,
                max: self.config.max_attempts,
            });
        }

        // Generate the auto-fix prompt (for reference/logging)
        let _prompt = generate_review_fix_prompt(&unit_task, &context);

        info!(
            unit_task_id = %unit_task_id,
            reviewer = %context.reviewer,
            file = %context.file_path,
            attempt = current_attempts + 1,
            "Creating auto-fix task for review comment"
        );

        // Create the agent task
        // Note: The prompt is stored at UnitTask level, not AgentTask level.
        // AgentTask is a collection of sessions for retrying.
        let agent_task = AgentTask::new();

        // Store the agent task
        self.store.create_agent_task(agent_task.clone()).await?;

        // Update the unit task with the new auto-fix task ID
        let mut updated_unit_task = unit_task;
        updated_unit_task.auto_fix_task_ids.push(agent_task.id);
        self.store.update_unit_task(updated_unit_task).await?;

        Ok(agent_task)
    }

    /// Creates an auto-fix task for a CI failure.
    pub async fn create_ci_failure_fix(
        &self,
        unit_task_id: Uuid,
        context: CiFailureContext,
    ) -> Result<AgentTask, AutoFixError> {
        if !self.config.auto_fix_ci_failures {
            warn!("Auto-fix for CI failures is disabled");
            return Err(AutoFixError::TaskNotFixable(
                unit_task_id,
                UnitTaskStatus::InProgress,
            ));
        }

        // Get the unit task
        let unit_task = self
            .store
            .get_unit_task(unit_task_id)
            .await?
            .ok_or(AutoFixError::TaskNotFound(unit_task_id))?;

        // Check max attempts
        let current_attempts = unit_task.auto_fix_task_ids.len() as u32;
        if current_attempts >= self.config.max_attempts {
            return Err(AutoFixError::MaxAttemptsExceeded {
                task_id: unit_task_id,
                max: self.config.max_attempts,
            });
        }

        // Generate the auto-fix prompt (for reference/logging)
        let _prompt = generate_ci_fix_prompt(&unit_task, &context);

        info!(
            unit_task_id = %unit_task_id,
            check_name = %context.check_name,
            attempt = current_attempts + 1,
            "Creating auto-fix task for CI failure"
        );

        // Create the agent task
        // Note: The prompt is stored at UnitTask level, not AgentTask level.
        // AgentTask is a collection of sessions for retrying.
        let agent_task = AgentTask::new();

        // Store the agent task
        self.store.create_agent_task(agent_task.clone()).await?;

        Ok(agent_task)
    }

    /// Checks if more auto-fix attempts are allowed for a task.
    pub async fn can_auto_fix(&self, unit_task_id: Uuid) -> Result<bool, AutoFixError> {
        let unit_task = self
            .store
            .get_unit_task(unit_task_id)
            .await?
            .ok_or(AutoFixError::TaskNotFound(unit_task_id))?;

        let current_attempts = unit_task.auto_fix_task_ids.len() as u32;
        Ok(current_attempts < self.config.max_attempts)
    }
}

/// Generates a prompt for fixing a review comment.
fn generate_review_fix_prompt(unit_task: &UnitTask, context: &ReviewCommentContext) -> String {
    let mut prompt = format!(
        "A reviewer ({}) has requested changes on the code.\n\n",
        context.reviewer
    );

    prompt.push_str(&format!("## Original Task\n{}\n\n", unit_task.prompt));

    prompt.push_str("## Review Feedback\n");
    prompt.push_str(&format!("File: `{}`\n", context.file_path));
    if let Some(line) = context.line_number {
        prompt.push_str(&format!("Line: {}\n", line));
    }
    prompt.push_str(&format!("\nComment:\n{}\n\n", context.comment));

    prompt.push_str("## Instructions\n");
    prompt.push_str("Please address the reviewer's feedback by making the necessary changes. ");
    prompt.push_str("Focus on the specific issue mentioned in the comment. ");
    prompt.push_str("Commit your changes with a clear message explaining the fix.");

    prompt
}

/// Generates a prompt for fixing a CI failure.
fn generate_ci_fix_prompt(unit_task: &UnitTask, context: &CiFailureContext) -> String {
    let mut prompt = format!("The CI check '{}' has failed.\n\n", context.check_name);

    prompt.push_str(&format!("## Original Task\n{}\n\n", unit_task.prompt));

    prompt.push_str("## CI Failure Details\n");
    if let Some(summary) = &context.summary {
        prompt.push_str(&format!("Summary: {}\n\n", summary));
    }
    if let Some(log) = &context.log_output {
        prompt.push_str("Log output:\n```\n");
        // Truncate log if too long, using char_indices for UTF-8 safety
        if log.chars().count() > 5000 {
            // Get the first 2500 characters safely
            let start: String = log.chars().take(2500).collect();
            prompt.push_str(&start);
            prompt.push_str("\n...[truncated]...\n");
            // Get the last 2500 characters safely
            let char_count = log.chars().count();
            let end: String = log.chars().skip(char_count.saturating_sub(2500)).collect();
            prompt.push_str(&end);
        } else {
            prompt.push_str(log);
        }
        prompt.push_str("\n```\n\n");
    }

    prompt.push_str("## Instructions\n");
    prompt.push_str("Please analyze the CI failure and fix the underlying issue. ");
    prompt.push_str("Common causes include:\n");
    prompt.push_str("- Test failures\n");
    prompt.push_str("- Linting errors\n");
    prompt.push_str("- Type check failures\n");
    prompt.push_str("- Build errors\n\n");
    prompt.push_str("Make the necessary fixes and commit with a clear message.");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_fix_prompt_generation() {
        let unit_task = UnitTask::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Implement user authentication",
        );

        let context = ReviewCommentContext {
            reviewer: "alice".to_string(),
            file_path: "src/auth.rs".to_string(),
            line_number: Some(42),
            comment: "Please add input validation here".to_string(),
            comment_url: "https://github.com/...".to_string(),
        };

        let prompt = generate_review_fix_prompt(&unit_task, &context);

        assert!(prompt.contains("alice"));
        assert!(prompt.contains("src/auth.rs"));
        assert!(prompt.contains("Line: 42"));
        assert!(prompt.contains("input validation"));
        assert!(prompt.contains("Implement user authentication"));
    }

    #[test]
    fn test_ci_fix_prompt_generation() {
        let unit_task = UnitTask::new(Uuid::new_v4(), Uuid::new_v4(), "Add new feature");

        let context = CiFailureContext {
            check_name: "test".to_string(),
            summary: Some("3 tests failed".to_string()),
            log_output: Some("FAIL src/feature.test.ts\n  ✗ should work".to_string()),
            check_url: "https://github.com/...".to_string(),
        };

        let prompt = generate_ci_fix_prompt(&unit_task, &context);

        assert!(prompt.contains("test"));
        assert!(prompt.contains("3 tests failed"));
        assert!(prompt.contains("FAIL src/feature.test.ts"));
        assert!(prompt.contains("Add new feature"));
    }

    #[test]
    fn test_ci_fix_prompt_truncates_long_logs() {
        let unit_task = UnitTask::new(Uuid::new_v4(), Uuid::new_v4(), "Task");

        let long_log = "x".repeat(10000);
        let context = CiFailureContext {
            check_name: "build".to_string(),
            summary: None,
            log_output: Some(long_log),
            check_url: "https://github.com/...".to_string(),
        };

        let prompt = generate_ci_fix_prompt(&unit_task, &context);

        assert!(prompt.contains("[truncated]"));
        assert!(prompt.len() < 15000); // Should be significantly smaller than 10000 log
    }

    #[test]
    fn test_auto_fix_config_default() {
        let config = AutoFixConfig::default();
        assert!(!config.auto_fix_review_comments);
        assert!(!config.auto_fix_ci_failures);
        assert_eq!(config.max_attempts, 3);
    }
}
