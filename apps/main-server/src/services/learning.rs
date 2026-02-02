//! AI Document Learning service.
//!
//! This service implements the learning pipeline for capturing feedback
//! from PR reviews and updating project documentation (AGENTS.md, CLAUDE.md)
//! when feedback is deemed generalizable.

use std::sync::Arc;

use entities::UnitTask;
use task_store::TaskStore;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

/// Errors that can occur during learning operations.
#[derive(Debug, Error)]
pub enum LearningError {
    /// Task not found.
    #[error("task not found: {0}")]
    TaskNotFound(Uuid),

    /// Store error.
    #[error("store error: {0}")]
    StoreError(#[from] task_store::TaskStoreError),

    /// Learning is disabled.
    #[error("learning is disabled")]
    LearningDisabled,
}

/// Category of feedback for learning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackCategory {
    /// Bug fix or correction.
    BugFix,
    /// Code style or formatting.
    Style,
    /// Performance improvement.
    Performance,
    /// Security concern.
    Security,
    /// Architecture or design pattern.
    Architecture,
    /// Documentation or comments.
    Documentation,
    /// Testing related.
    Testing,
    /// Other/uncategorized.
    Other,
}

impl FeedbackCategory {
    /// Attempts to categorize feedback based on keywords.
    pub fn from_feedback(feedback: &str) -> Self {
        let lower = feedback.to_lowercase();

        if lower.contains("security")
            || lower.contains("vulnerability")
            || lower.contains("xss")
            || lower.contains("injection")
        {
            Self::Security
        } else if lower.contains("performance")
            || lower.contains("optimize")
            || lower.contains("slow")
            || lower.contains("memory")
        {
            Self::Performance
        } else if lower.contains("style")
            || lower.contains("format")
            || lower.contains("naming")
            || lower.contains("convention")
        {
            Self::Style
        } else if lower.contains("test")
            || lower.contains("coverage")
            || lower.contains("spec")
            || lower.contains("assertion")
        {
            Self::Testing
        } else if lower.contains("doc")
            || lower.contains("comment")
            || lower.contains("readme")
            || lower.contains("jsdoc")
        {
            Self::Documentation
        } else if lower.contains("architecture")
            || lower.contains("pattern")
            || lower.contains("design")
            || lower.contains("refactor")
        {
            Self::Architecture
        } else if lower.contains("bug")
            || lower.contains("fix")
            || lower.contains("error")
            || lower.contains("wrong")
        {
            Self::BugFix
        } else {
            Self::Other
        }
    }
}

/// A captured feedback item for potential learning.
#[derive(Debug, Clone)]
pub struct FeedbackItem {
    /// The feedback ID.
    pub id: Uuid,
    /// The associated UnitTask ID.
    pub unit_task_id: Uuid,
    /// The reviewer who provided feedback.
    pub reviewer: String,
    /// The feedback content.
    pub feedback: String,
    /// The file path related to the feedback.
    pub file_path: Option<String>,
    /// The category of feedback.
    pub category: FeedbackCategory,
    /// Whether this feedback has been processed.
    pub processed: bool,
    /// Whether this feedback is generalizable.
    pub is_generalizable: Option<bool>,
    /// The learning outcome (update to make).
    pub learning_outcome: Option<String>,
}

impl FeedbackItem {
    /// Creates a new feedback item.
    pub fn new(
        unit_task_id: Uuid,
        reviewer: impl Into<String>,
        feedback: impl Into<String>,
    ) -> Self {
        let feedback_str = feedback.into();
        let category = FeedbackCategory::from_feedback(&feedback_str);

        Self {
            id: Uuid::new_v4(),
            unit_task_id,
            reviewer: reviewer.into(),
            feedback: feedback_str,
            file_path: None,
            category,
            processed: false,
            is_generalizable: None,
            learning_outcome: None,
        }
    }

    /// Sets the file path for this feedback.
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }
}

/// Learning service configuration.
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Whether automatic learning from reviews is enabled.
    pub auto_learn_from_reviews: bool,
    /// Categories of feedback to consider for learning.
    pub learnable_categories: Vec<FeedbackCategory>,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            auto_learn_from_reviews: false,
            learnable_categories: vec![
                FeedbackCategory::Security,
                FeedbackCategory::Architecture,
                FeedbackCategory::Style,
                FeedbackCategory::Testing,
            ],
        }
    }
}

/// Service for managing AI document learning.
pub struct LearningService<S: TaskStore> {
    #[allow(dead_code)] // Will be used when full learning implementation is complete
    store: Arc<S>,
    config: LearningConfig,
}

impl<S: TaskStore> LearningService<S> {
    /// Creates a new learning service.
    pub fn new(store: Arc<S>, config: LearningConfig) -> Self {
        Self { store, config }
    }

    /// Records feedback for potential learning.
    pub async fn record_feedback(&self, feedback: FeedbackItem) -> Result<Uuid, LearningError> {
        if !self.config.auto_learn_from_reviews {
            return Err(LearningError::LearningDisabled);
        }

        info!(
            feedback_id = %feedback.id,
            unit_task_id = %feedback.unit_task_id,
            category = ?feedback.category,
            "Recording feedback for learning"
        );

        // Check if the category is learnable
        if !self
            .config
            .learnable_categories
            .contains(&feedback.category)
        {
            info!(
                feedback_id = %feedback.id,
                category = ?feedback.category,
                "Feedback category not configured for learning"
            );
        }

        // TODO: Store the feedback item for later processing
        // self.store.create_feedback_item(&feedback).await?;

        Ok(feedback.id)
    }

    /// Processes a feedback item to determine if it's generalizable.
    ///
    /// This would typically involve:
    /// 1. Analyzing the feedback content
    /// 2. Checking if similar feedback has been given before
    /// 3. Using AI to determine generalizability
    /// 4. Generating the appropriate documentation update
    pub async fn process_feedback(
        &self,
        feedback_id: Uuid,
    ) -> Result<Option<String>, LearningError> {
        if !self.config.auto_learn_from_reviews {
            return Err(LearningError::LearningDisabled);
        }

        info!(feedback_id = %feedback_id, "Processing feedback for generalizability");

        // TODO: Implement actual processing
        // 1. Fetch the feedback item
        // 2. Use AI to analyze if it's generalizable
        // 3. If yes, generate documentation update
        // 4. Return the suggested update

        Ok(None)
    }

    /// Generates a learning prompt for the AI to determine generalizability.
    pub fn generate_learning_prompt(
        &self,
        unit_task: &UnitTask,
        feedback: &FeedbackItem,
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are analyzing review feedback to determine if it contains ");
        prompt.push_str("generalizable lessons that should be documented for future work.\n\n");

        prompt.push_str("## Original Task\n");
        prompt.push_str(&unit_task.prompt);
        prompt.push_str("\n\n");

        prompt.push_str("## Review Feedback\n");
        prompt.push_str(&format!("Reviewer: {}\n", feedback.reviewer));
        prompt.push_str(&format!("Category: {:?}\n", feedback.category));
        if let Some(path) = &feedback.file_path {
            prompt.push_str(&format!("File: {}\n", path));
        }
        prompt.push_str(&format!("\nFeedback:\n{}\n\n", feedback.feedback));

        prompt.push_str("## Questions to Answer\n");
        prompt
            .push_str("1. Is this feedback specific to this task, or is it a general principle?\n");
        prompt.push_str("2. Would this guidance apply to similar future tasks?\n");
        prompt.push_str("3. If generalizable, what document should be updated?\n");
        prompt.push_str("   - AGENTS.md for project-wide coding guidelines\n");
        prompt.push_str("   - CLAUDE.md for AI-specific instructions\n");
        prompt.push_str("4. What specific text should be added?\n\n");

        prompt.push_str("## Response Format\n");
        prompt.push_str("Respond with:\n");
        prompt.push_str("- is_generalizable: true/false\n");
        prompt.push_str("- target_document: AGENTS.md or CLAUDE.md (if generalizable)\n");
        prompt.push_str("- suggested_addition: The text to add (if generalizable)\n");

        prompt
    }

    /// Applies a learning outcome to the appropriate document.
    pub async fn apply_learning(
        &self,
        _feedback_id: Uuid,
        target_document: &str,
        addition: &str,
    ) -> Result<(), LearningError> {
        info!(
            document = %target_document,
            "Applying learning to document"
        );

        // TODO: Implement actual document update
        // This would:
        // 1. Read the current document
        // 2. Find the appropriate section to add the learning
        // 3. Add the new content
        // 4. Create a commit with the changes
        // 5. Optionally create a PR for review

        info!(
            document = %target_document,
            addition_length = addition.len(),
            "Learning would be applied (not implemented)"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_category_detection() {
        assert_eq!(
            FeedbackCategory::from_feedback("This has a security vulnerability"),
            FeedbackCategory::Security
        );
        assert_eq!(
            FeedbackCategory::from_feedback("Please optimize this for better performance"),
            FeedbackCategory::Performance
        );
        assert_eq!(
            FeedbackCategory::from_feedback("Add tests for this function"),
            FeedbackCategory::Testing
        );
        assert_eq!(
            FeedbackCategory::from_feedback("The naming convention is wrong"),
            FeedbackCategory::Style
        );
        assert_eq!(
            FeedbackCategory::from_feedback("Consider refactoring this using the factory pattern"),
            FeedbackCategory::Architecture
        );
        assert_eq!(
            FeedbackCategory::from_feedback("Please update the docs"),
            FeedbackCategory::Documentation
        );
        assert_eq!(
            FeedbackCategory::from_feedback("This is a bug"),
            FeedbackCategory::BugFix
        );
        assert_eq!(
            FeedbackCategory::from_feedback("Something something"),
            FeedbackCategory::Other
        );
    }

    #[test]
    fn test_feedback_item_creation() {
        let task_id = Uuid::new_v4();
        let feedback =
            FeedbackItem::new(task_id, "alice", "Please add input validation for security")
                .with_file_path("src/api.rs");

        assert_eq!(feedback.unit_task_id, task_id);
        assert_eq!(feedback.reviewer, "alice");
        assert_eq!(feedback.category, FeedbackCategory::Security);
        assert_eq!(feedback.file_path, Some("src/api.rs".to_string()));
        assert!(!feedback.processed);
        assert!(feedback.is_generalizable.is_none());
    }

    #[test]
    fn test_learning_config_default() {
        let config = LearningConfig::default();
        assert!(!config.auto_learn_from_reviews);
        assert!(
            config
                .learnable_categories
                .contains(&FeedbackCategory::Security)
        );
        assert!(
            config
                .learnable_categories
                .contains(&FeedbackCategory::Architecture)
        );
    }

    #[test]
    fn test_generate_learning_prompt() {
        use task_store::MemoryTaskStore;

        let store = Arc::new(MemoryTaskStore::new());
        let service = LearningService::new(store, LearningConfig::default());

        let unit_task = UnitTask::new(Uuid::new_v4(), Uuid::new_v4(), "Implement user API");
        let feedback = FeedbackItem::new(
            unit_task.id,
            "reviewer",
            "Always validate user input to prevent injection attacks",
        )
        .with_file_path("src/api.rs");

        let prompt = service.generate_learning_prompt(&unit_task, &feedback);

        assert!(prompt.contains("generalizable"));
        assert!(prompt.contains("Implement user API"));
        assert!(prompt.contains("injection attacks"));
        assert!(prompt.contains("AGENTS.md"));
        assert!(prompt.contains("CLAUDE.md"));
    }
}
