//! PLAN.yaml parsing module.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during PLAN.yaml parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// File read error.
    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// YAML parsing error.
    #[error("failed to parse YAML: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    /// Empty plan error.
    #[error("plan file contains no tasks")]
    EmptyPlan,
}

/// A single task definition in a PLAN.yaml file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanTask {
    /// Unique identifier for this task within the plan.
    pub id: String,

    /// Human-readable task title (optional).
    #[serde(default)]
    pub title: Option<String>,

    /// Task description for the AI agent.
    pub prompt: String,

    /// Custom git branch name (optional).
    #[serde(default)]
    pub branch_name: Option<String>,

    /// List of task IDs that must complete before this task.
    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl PlanTask {
    /// Creates a new plan task with the given ID and prompt.
    pub fn new(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: None,
            prompt: prompt.into(),
            branch_name: None,
            depends_on: Vec::new(),
        }
    }

    /// Sets the title for this task.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the branch name for this task.
    pub fn with_branch_name(mut self, branch_name: impl Into<String>) -> Self {
        self.branch_name = Some(branch_name.into());
        self
    }

    /// Adds dependencies to this task.
    pub fn with_depends_on(mut self, deps: Vec<String>) -> Self {
        self.depends_on = deps;
        self
    }

    /// Returns the display title (title if set, otherwise id).
    pub fn display_title(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.id)
    }
}

/// A parsed PLAN.yaml structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// List of tasks in the plan.
    pub tasks: Vec<PlanTask>,
}

impl Plan {
    /// Creates a new empty plan.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Adds a task to the plan.
    pub fn add_task(&mut self, task: PlanTask) {
        self.tasks.push(task);
    }

    /// Parses a PLAN.yaml file from a path.
    pub fn from_file(path: &Path) -> Result<Self, ParseError> {
        let contents = std::fs::read_to_string(path).map_err(|e| ParseError::ReadFile {
            path: path.to_path_buf(),
            source: e,
        })?;

        Self::from_yaml(&contents)
    }

    /// Parses a PLAN.yaml from a string.
    pub fn from_yaml(yaml: &str) -> Result<Self, ParseError> {
        let plan: Plan = serde_yaml::from_str(yaml)?;

        if plan.tasks.is_empty() {
            return Err(ParseError::EmptyPlan);
        }

        Ok(plan)
    }

    /// Serializes the plan to YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Saves the plan to a file.
    pub fn save(&self, path: &Path) -> Result<(), ParseError> {
        let yaml = self.to_yaml()?;
        std::fs::write(path, yaml).map_err(|e| ParseError::ReadFile {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Returns the number of tasks in the plan.
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Finds a task by ID.
    pub fn get_task(&self, id: &str) -> Option<&PlanTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Returns all task IDs in the plan.
    pub fn task_ids(&self) -> Vec<&str> {
        self.tasks.iter().map(|t| t.id.as_str()).collect()
    }
}

impl Default for Plan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_plan() {
        let yaml = r#"
tasks:
  - id: "task-1"
    prompt: "Do something"
"#;

        let plan = Plan::from_yaml(yaml).unwrap();
        assert_eq!(plan.task_count(), 1);
        assert_eq!(plan.tasks[0].id, "task-1");
        assert_eq!(plan.tasks[0].prompt, "Do something");
    }

    #[test]
    fn test_parse_full_plan() {
        let yaml = r#"
tasks:
  - id: "setup-db"
    title: "Setup Database"
    prompt: "Create database schema"
    branchName: "feature/db"

  - id: "auth-api"
    title: "Auth API"
    prompt: "Implement auth endpoints"
    dependsOn:
      - "setup-db"
"#;

        let plan = Plan::from_yaml(yaml).unwrap();
        assert_eq!(plan.task_count(), 2);

        let task1 = plan.get_task("setup-db").unwrap();
        assert_eq!(task1.title, Some("Setup Database".to_string()));
        assert_eq!(task1.branch_name, Some("feature/db".to_string()));
        assert!(task1.depends_on.is_empty());

        let task2 = plan.get_task("auth-api").unwrap();
        assert_eq!(task2.depends_on, vec!["setup-db"]);
    }

    #[test]
    fn test_parse_empty_plan_fails() {
        let yaml = "tasks: []";
        let result = Plan::from_yaml(yaml);
        assert!(matches!(result, Err(ParseError::EmptyPlan)));
    }

    #[test]
    fn test_plan_task_builder() {
        let task = PlanTask::new("my-task", "Do something")
            .with_title("My Task")
            .with_branch_name("feature/my-task")
            .with_depends_on(vec!["other-task".to_string()]);

        assert_eq!(task.id, "my-task");
        assert_eq!(task.title, Some("My Task".to_string()));
        assert_eq!(task.branch_name, Some("feature/my-task".to_string()));
        assert_eq!(task.depends_on, vec!["other-task"]);
        assert_eq!(task.display_title(), "My Task");
    }

    #[test]
    fn test_display_title_fallback() {
        let task = PlanTask::new("task-id", "prompt");
        assert_eq!(task.display_title(), "task-id");
    }

    #[test]
    fn test_plan_to_yaml() {
        let mut plan = Plan::new();
        plan.add_task(PlanTask::new("task-1", "First task"));
        plan.add_task(
            PlanTask::new("task-2", "Second task").with_depends_on(vec!["task-1".to_string()]),
        );

        let yaml = plan.to_yaml().unwrap();
        assert!(yaml.contains("task-1"));
        assert!(yaml.contains("task-2"));
        assert!(yaml.contains("dependsOn"));
    }

    #[test]
    fn test_task_ids() {
        let yaml = r#"
tasks:
  - id: "a"
    prompt: "A"
  - id: "b"
    prompt: "B"
  - id: "c"
    prompt: "C"
"#;

        let plan = Plan::from_yaml(yaml).unwrap();
        let ids = plan.task_ids();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }
}
