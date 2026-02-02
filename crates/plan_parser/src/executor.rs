//! Plan execution orchestration module.
//!
//! This module provides functionality for orchestrating the execution of
//! tasks defined in a PLAN.yaml file, including parallel execution where
//! dependencies allow.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::{validate_plan, Plan, ValidationError};

/// Errors that can occur during plan execution.
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Plan validation failed.
    #[error("plan validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),

    /// Multiple validation errors.
    #[error("plan validation failed with {0} errors")]
    MultipleValidationErrors(usize),

    /// Task not found.
    #[error("task not found: {0}")]
    TaskNotFound(String),

    /// Task execution failed.
    #[error("task '{task_id}' execution failed: {message}")]
    TaskFailed { task_id: String, message: String },

    /// Execution was cancelled.
    #[error("execution cancelled")]
    Cancelled,
}

/// Status of a task during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskExecutionStatus {
    /// Task is waiting for dependencies.
    Pending,
    /// Task is ready to execute (dependencies satisfied).
    Ready,
    /// Task is currently executing.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed,
    /// Task was skipped (dependency failed).
    Skipped,
}

/// Information about a task's execution state.
#[derive(Debug, Clone)]
pub struct TaskExecutionState {
    /// The task ID from the plan.
    pub plan_task_id: String,
    /// The UnitTask ID (assigned when execution starts).
    pub unit_task_id: Option<Uuid>,
    /// Current execution status.
    pub status: TaskExecutionStatus,
    /// Error message if failed.
    pub error: Option<String>,
}

impl TaskExecutionState {
    /// Creates a new pending task state.
    pub fn new(plan_task_id: impl Into<String>) -> Self {
        Self {
            plan_task_id: plan_task_id.into(),
            unit_task_id: None,
            status: TaskExecutionStatus::Pending,
            error: None,
        }
    }

    /// Returns true if the task is complete (successfully or failed).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TaskExecutionStatus::Completed
                | TaskExecutionStatus::Failed
                | TaskExecutionStatus::Skipped
        )
    }
}

/// Orchestrates the execution of a plan.
#[derive(Debug)]
pub struct PlanExecutor {
    /// The plan being executed.
    plan: Arc<Plan>,
    /// Task states by plan task ID.
    states: HashMap<String, TaskExecutionState>,
    /// Mapping from plan task ID to its dependencies.
    dependencies: HashMap<String, Vec<String>>,
    /// Mapping from plan task ID to tasks that depend on it.
    dependents: HashMap<String, Vec<String>>,
}

impl PlanExecutor {
    /// Creates a new executor for the given plan.
    pub fn new(plan: Plan) -> Result<Self, ExecutionError> {
        // Validate the plan first
        let validation = validate_plan(&plan);
        if !validation.is_valid() {
            if validation.errors.len() == 1 {
                return Err(ExecutionError::ValidationFailed(
                    validation.errors.into_iter().next().unwrap(),
                ));
            } else {
                return Err(ExecutionError::MultipleValidationErrors(
                    validation.errors.len(),
                ));
            }
        }

        let mut states = HashMap::new();
        let mut dependencies = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for task in &plan.tasks {
            states.insert(task.id.clone(), TaskExecutionState::new(&task.id));
            dependencies.insert(task.id.clone(), task.depends_on.clone());
            dependents.entry(task.id.clone()).or_default();

            for dep_id in &task.depends_on {
                dependents
                    .entry(dep_id.clone())
                    .or_default()
                    .push(task.id.clone());
            }
        }

        Ok(Self {
            plan: Arc::new(plan),
            states,
            dependencies,
            dependents,
        })
    }

    /// Returns the underlying plan.
    pub fn plan(&self) -> &Plan {
        &self.plan
    }

    /// Returns all tasks that are ready to execute.
    pub fn get_ready_tasks(&self) -> Vec<&str> {
        self.states
            .iter()
            .filter(|(task_id, state)| {
                state.status == TaskExecutionStatus::Pending && self.all_deps_completed(task_id)
            })
            .map(|(task_id, _)| task_id.as_str())
            .collect()
    }

    /// Returns all currently running tasks.
    pub fn get_running_tasks(&self) -> Vec<&str> {
        self.states
            .iter()
            .filter(|(_, state)| state.status == TaskExecutionStatus::Running)
            .map(|(task_id, _)| task_id.as_str())
            .collect()
    }

    /// Returns all completed tasks.
    pub fn get_completed_tasks(&self) -> Vec<&str> {
        self.states
            .iter()
            .filter(|(_, state)| state.status == TaskExecutionStatus::Completed)
            .map(|(task_id, _)| task_id.as_str())
            .collect()
    }

    /// Returns all failed tasks.
    pub fn get_failed_tasks(&self) -> Vec<&str> {
        self.states
            .iter()
            .filter(|(_, state)| state.status == TaskExecutionStatus::Failed)
            .map(|(task_id, _)| task_id.as_str())
            .collect()
    }

    /// Returns the state of a specific task.
    pub fn get_task_state(&self, task_id: &str) -> Option<&TaskExecutionState> {
        self.states.get(task_id)
    }

    /// Marks a task as started.
    pub fn start_task(
        &mut self,
        task_id: &str,
        unit_task_id: Uuid,
    ) -> Result<(), ExecutionError> {
        let state = self
            .states
            .get_mut(task_id)
            .ok_or_else(|| ExecutionError::TaskNotFound(task_id.to_string()))?;

        state.status = TaskExecutionStatus::Running;
        state.unit_task_id = Some(unit_task_id);
        Ok(())
    }

    /// Marks a task as completed successfully.
    pub fn complete_task(&mut self, task_id: &str) -> Result<Vec<&str>, ExecutionError> {
        let state = self
            .states
            .get_mut(task_id)
            .ok_or_else(|| ExecutionError::TaskNotFound(task_id.to_string()))?;

        state.status = TaskExecutionStatus::Completed;

        // Return tasks that are now ready to run
        Ok(self.get_ready_tasks())
    }

    /// Marks a task as failed.
    pub fn fail_task(&mut self, task_id: &str, error: &str) -> Result<(), ExecutionError> {
        let state = self
            .states
            .get_mut(task_id)
            .ok_or_else(|| ExecutionError::TaskNotFound(task_id.to_string()))?;

        state.status = TaskExecutionStatus::Failed;
        state.error = Some(error.to_string());

        // Skip all dependent tasks
        self.skip_dependents(task_id);

        Ok(())
    }

    /// Returns true if all tasks are in a terminal state.
    pub fn is_complete(&self) -> bool {
        self.states.values().all(|s| s.is_terminal())
    }

    /// Returns true if all tasks completed successfully.
    pub fn is_successful(&self) -> bool {
        self.states
            .values()
            .all(|s| s.status == TaskExecutionStatus::Completed)
    }

    /// Returns execution progress as (completed, total).
    pub fn progress(&self) -> (usize, usize) {
        let completed = self
            .states
            .values()
            .filter(|s| s.status == TaskExecutionStatus::Completed)
            .count();
        (completed, self.states.len())
    }

    /// Checks if all dependencies of a task are completed.
    fn all_deps_completed(&self, task_id: &str) -> bool {
        if let Some(deps) = self.dependencies.get(task_id) {
            deps.iter().all(|dep_id| {
                self.states
                    .get(dep_id)
                    .is_some_and(|s| s.status == TaskExecutionStatus::Completed)
            })
        } else {
            true
        }
    }

    /// Checks if any dependency of a task has failed.
    fn any_dep_failed(&self, task_id: &str) -> bool {
        if let Some(deps) = self.dependencies.get(task_id) {
            deps.iter().any(|dep_id| {
                self.states.get(dep_id).is_some_and(|s| {
                    s.status == TaskExecutionStatus::Failed
                        || s.status == TaskExecutionStatus::Skipped
                })
            })
        } else {
            false
        }
    }

    /// Recursively skips all tasks that depend on the given task.
    fn skip_dependents(&mut self, task_id: &str) {
        let mut to_skip: Vec<String> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: Vec<String> = vec![task_id.to_string()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(deps) = self.dependents.get(&current) {
                for dep_id in deps {
                    if !visited.contains(dep_id) {
                        to_skip.push(dep_id.clone());
                        queue.push(dep_id.clone());
                    }
                }
            }
        }

        for dep_id in to_skip {
            if let Some(state) = self.states.get_mut(&dep_id) {
                if state.status == TaskExecutionStatus::Pending {
                    state.status = TaskExecutionStatus::Skipped;
                    state.error = Some(format!("Skipped due to dependency failure: {}", task_id));
                }
            }
        }
    }
}

/// Determines which tasks can run in parallel at the current state.
pub fn get_parallel_batch(executor: &PlanExecutor) -> Vec<&str> {
    executor.get_ready_tasks()
}

/// Configuration for parallel execution.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent tasks.
    pub max_concurrent: usize,
    /// Whether to stop on first failure.
    pub fail_fast: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            fail_fast: false,
        }
    }
}

impl ExecutionConfig {
    /// Creates a new configuration with the given max concurrent tasks.
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Sets fail-fast mode.
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::PlanTask;

    use super::*;

    fn create_plan_with_tasks(tasks: Vec<PlanTask>) -> Plan {
        Plan { tasks }
    }

    #[test]
    fn test_executor_creation() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
        ]);

        let executor = PlanExecutor::new(plan).unwrap();
        assert_eq!(executor.progress(), (0, 2));
    }

    #[test]
    fn test_executor_invalid_plan() {
        let plan = create_plan_with_tasks(vec![PlanTask::new("a", "Task A")
            .with_depends_on(vec!["non-existent".to_string()])]);

        let result = PlanExecutor::new(plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_ready_tasks_initial() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B"),
            PlanTask::new("c", "Task C").with_depends_on(vec!["a".to_string()]),
        ]);

        let executor = PlanExecutor::new(plan).unwrap();
        let ready = executor.get_ready_tasks();

        assert!(ready.contains(&"a"));
        assert!(ready.contains(&"b"));
        assert!(!ready.contains(&"c"));
    }

    #[test]
    fn test_task_lifecycle() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
        ]);

        let mut executor = PlanExecutor::new(plan).unwrap();

        // Initially only "a" is ready
        assert_eq!(executor.get_ready_tasks(), vec!["a"]);

        // Start task "a"
        let unit_id = Uuid::new_v4();
        executor.start_task("a", unit_id).unwrap();
        assert!(executor.get_ready_tasks().is_empty());
        assert_eq!(executor.get_running_tasks(), vec!["a"]);

        // Complete task "a"
        let ready = executor.complete_task("a").unwrap();
        assert!(ready.contains(&"b"));
        assert_eq!(executor.get_completed_tasks(), vec!["a"]);

        // Start and complete task "b"
        executor.start_task("b", Uuid::new_v4()).unwrap();
        executor.complete_task("b").unwrap();

        assert!(executor.is_complete());
        assert!(executor.is_successful());
        assert_eq!(executor.progress(), (2, 2));
    }

    #[test]
    fn test_task_failure_skips_dependents() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
            PlanTask::new("c", "Task C").with_depends_on(vec!["b".to_string()]),
        ]);

        let mut executor = PlanExecutor::new(plan).unwrap();

        // Start and fail task "a"
        executor.start_task("a", Uuid::new_v4()).unwrap();
        executor.fail_task("a", "Something went wrong").unwrap();

        // Both "b" and "c" should be skipped
        let state_b = executor.get_task_state("b").unwrap();
        let state_c = executor.get_task_state("c").unwrap();

        assert_eq!(state_b.status, TaskExecutionStatus::Skipped);
        assert_eq!(state_c.status, TaskExecutionStatus::Skipped);

        assert!(executor.is_complete());
        assert!(!executor.is_successful());
    }

    #[test]
    fn test_parallel_execution_possible() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B"),
            PlanTask::new("c", "Task C"),
            PlanTask::new("d", "Task D")
                .with_depends_on(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        ]);

        let executor = PlanExecutor::new(plan).unwrap();
        let batch = get_parallel_batch(&executor);

        assert_eq!(batch.len(), 3);
        assert!(batch.contains(&"a"));
        assert!(batch.contains(&"b"));
        assert!(batch.contains(&"c"));
    }

    #[test]
    fn test_execution_config() {
        let config = ExecutionConfig::default()
            .with_max_concurrent(8)
            .with_fail_fast(true);

        assert_eq!(config.max_concurrent, 8);
        assert!(config.fail_fast);
    }

    #[test]
    fn test_task_state_is_terminal() {
        let mut state = TaskExecutionState::new("test");
        assert!(!state.is_terminal());

        state.status = TaskExecutionStatus::Running;
        assert!(!state.is_terminal());

        state.status = TaskExecutionStatus::Completed;
        assert!(state.is_terminal());

        state.status = TaskExecutionStatus::Failed;
        assert!(state.is_terminal());

        state.status = TaskExecutionStatus::Skipped;
        assert!(state.is_terminal());
    }
}
