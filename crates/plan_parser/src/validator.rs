//! PLAN.yaml validation module.
//!
//! This module provides validation rules for PLAN.yaml files:
//! - Unique task IDs
//! - Valid dependency references
//! - No cyclic dependencies
//! - Non-empty prompts

use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::Plan;

/// Errors that can occur during plan validation.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ValidationError {
    /// Two tasks have the same ID.
    #[error("duplicate task ID: {0}")]
    DuplicateTaskId(String),

    /// A dependency references a non-existent task.
    #[error("invalid dependency: task '{task_id}' depends on non-existent task '{dependency_id}'")]
    InvalidDependency {
        task_id: String,
        dependency_id: String,
    },

    /// Circular dependency detected.
    #[error("cyclic dependency detected involving task: {0}")]
    CyclicDependency(String),

    /// Task has empty prompt.
    #[error("task '{0}' has empty prompt")]
    EmptyPrompt(String),

    /// Task ID is empty.
    #[error("task has empty ID at index {0}")]
    EmptyTaskId(usize),
}

/// Result of plan validation.
#[derive(Debug)]
pub struct ValidationResult {
    /// List of validation errors (empty if valid).
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Returns true if the plan is valid.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the first error, if any.
    pub fn first_error(&self) -> Option<&ValidationError> {
        self.errors.first()
    }
}

/// Validates a plan and returns all validation errors.
pub fn validate_plan(plan: &Plan) -> ValidationResult {
    let mut errors = Vec::new();

    // Check for empty task IDs
    for (idx, task) in plan.tasks.iter().enumerate() {
        if task.id.trim().is_empty() {
            errors.push(ValidationError::EmptyTaskId(idx));
        }
    }

    // Check for duplicate task IDs
    let mut seen_ids: HashSet<&str> = HashSet::new();
    for task in &plan.tasks {
        if !task.id.trim().is_empty() && !seen_ids.insert(&task.id) {
            errors.push(ValidationError::DuplicateTaskId(task.id.clone()));
        }
    }

    // Check for empty prompts
    for task in &plan.tasks {
        if task.prompt.trim().is_empty() {
            errors.push(ValidationError::EmptyPrompt(task.id.clone()));
        }
    }

    // Check for invalid dependency references
    let valid_ids: HashSet<&str> = plan.tasks.iter().map(|t| t.id.as_str()).collect();
    for task in &plan.tasks {
        for dep_id in &task.depends_on {
            if !valid_ids.contains(dep_id.as_str()) {
                errors.push(ValidationError::InvalidDependency {
                    task_id: task.id.clone(),
                    dependency_id: dep_id.clone(),
                });
            }
        }
    }

    // Check for cyclic dependencies
    if let Some(cycle_task) = detect_cycle(plan) {
        errors.push(ValidationError::CyclicDependency(cycle_task));
    }

    ValidationResult { errors }
}

/// Detects cyclic dependencies in a plan using DFS.
/// Returns the ID of a task involved in a cycle, or None if no cycle exists.
fn detect_cycle(plan: &Plan) -> Option<String> {
    // Build adjacency list: task_id -> list of tasks that depend on it
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for task in &plan.tasks {
        dependents.entry(&task.id).or_default();
        for dep_id in &task.depends_on {
            dependents.entry(dep_id.as_str()).or_default();
        }
    }

    // State for cycle detection
    // 0 = unvisited, 1 = visiting (in current path), 2 = visited
    let mut state: HashMap<&str, u8> = HashMap::new();
    for task in &plan.tasks {
        state.insert(&task.id, 0);
    }

    // Build reverse mapping: task_id -> dependencies
    let task_deps: HashMap<&str, &Vec<String>> = plan
        .tasks
        .iter()
        .map(|t| (t.id.as_str(), &t.depends_on))
        .collect();

    // DFS from each unvisited node
    for task in &plan.tasks {
        if state.get(task.id.as_str()) == Some(&0) {
            if let Some(cycle_id) = dfs_cycle(&task.id, &task_deps, &mut state) {
                return Some(cycle_id);
            }
        }
    }

    None
}

/// DFS helper for cycle detection.
fn dfs_cycle<'a>(
    task_id: &'a str,
    task_deps: &'a HashMap<&'a str, &'a Vec<String>>,
    state: &mut HashMap<&'a str, u8>,
) -> Option<String> {
    state.insert(task_id, 1); // Mark as visiting

    if let Some(deps) = task_deps.get(task_id) {
        for dep_id in *deps {
            match state.get(dep_id.as_str()) {
                Some(&1) => {
                    // Found a back edge - cycle detected
                    return Some(dep_id.clone());
                }
                Some(&0) | None => {
                    // Unvisited - recurse
                    if let Some(cycle_id) = dfs_cycle(dep_id.as_str(), task_deps, state) {
                        return Some(cycle_id);
                    }
                }
                Some(&2) => {
                    // Already fully visited - no cycle through this path
                }
                _ => {}
            }
        }
    }

    state.insert(task_id, 2); // Mark as visited
    None
}

/// Computes the topological order of tasks (for execution scheduling).
/// Returns None if the graph has a cycle.
pub fn topological_sort(plan: &Plan) -> Option<Vec<&str>> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    // Initialize
    for task in &plan.tasks {
        in_degree.insert(&task.id, 0);
        dependents.entry(&task.id).or_default();
    }

    // Count in-degrees and build dependents map
    for task in &plan.tasks {
        for dep_id in &task.depends_on {
            if let Some(count) = in_degree.get_mut(task.id.as_str()) {
                *count += 1;
            }
            if let Some(deps) = dependents.get_mut(dep_id.as_str()) {
                deps.push(&task.id);
            }
        }
    }

    // Start with nodes that have no dependencies
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut result = Vec::new();

    while let Some(task_id) = queue.pop() {
        result.push(task_id);

        if let Some(deps) = dependents.get(task_id) {
            for dep_id in deps {
                if let Some(count) = in_degree.get_mut(dep_id) {
                    *count -= 1;
                    if *count == 0 {
                        queue.push(dep_id);
                    }
                }
            }
        }
    }

    if result.len() == plan.tasks.len() {
        Some(result)
    } else {
        None // Cycle detected
    }
}

/// Returns tasks that have no dependencies (can start immediately).
pub fn get_root_tasks(plan: &Plan) -> Vec<&str> {
    plan.tasks
        .iter()
        .filter(|t| t.depends_on.is_empty())
        .map(|t| t.id.as_str())
        .collect()
}

/// Returns tasks that depend on the given task.
pub fn get_dependent_tasks<'a>(plan: &'a Plan, task_id: &str) -> Vec<&'a str> {
    plan.tasks
        .iter()
        .filter(|t| t.depends_on.iter().any(|d| d == task_id))
        .map(|t| t.id.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PlanTask;

    fn create_plan_with_tasks(tasks: Vec<PlanTask>) -> Plan {
        Plan { tasks }
    }

    #[test]
    fn test_valid_plan() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
            PlanTask::new("c", "Task C").with_depends_on(vec!["a".to_string(), "b".to_string()]),
        ]);

        let result = validate_plan(&plan);
        assert!(result.is_valid());
    }

    #[test]
    fn test_duplicate_task_id() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("same-id", "Task 1"),
            PlanTask::new("same-id", "Task 2"),
        ]);

        let result = validate_plan(&plan);
        assert!(!result.is_valid());
        assert!(matches!(
            result.first_error(),
            Some(ValidationError::DuplicateTaskId(id)) if id == "same-id"
        ));
    }

    #[test]
    fn test_invalid_dependency() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A").with_depends_on(vec!["non-existent".to_string()])
        ]);

        let result = validate_plan(&plan);
        assert!(!result.is_valid());
        assert!(matches!(
            result.first_error(),
            Some(ValidationError::InvalidDependency { task_id, dependency_id })
            if task_id == "a" && dependency_id == "non-existent"
        ));
    }

    #[test]
    fn test_cyclic_dependency_simple() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A").with_depends_on(vec!["b".to_string()]),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
        ]);

        let result = validate_plan(&plan);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicDependency(_))));
    }

    #[test]
    fn test_cyclic_dependency_chain() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A").with_depends_on(vec!["c".to_string()]),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
            PlanTask::new("c", "Task C").with_depends_on(vec!["b".to_string()]),
        ]);

        let result = validate_plan(&plan);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicDependency(_))));
    }

    #[test]
    fn test_self_dependency() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A").with_depends_on(vec!["a".to_string()])
        ]);

        let result = validate_plan(&plan);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CyclicDependency(_))));
    }

    #[test]
    fn test_empty_prompt() {
        let plan = create_plan_with_tasks(vec![PlanTask::new("a", "")]);

        let result = validate_plan(&plan);
        assert!(!result.is_valid());
        assert!(matches!(
            result.first_error(),
            Some(ValidationError::EmptyPrompt(id)) if id == "a"
        ));
    }

    #[test]
    fn test_topological_sort() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("c", "Task C").with_depends_on(vec!["a".to_string(), "b".to_string()]),
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
        ]);

        let sorted = topological_sort(&plan).unwrap();

        // a must come before b and c
        // b must come before c
        let pos_a = sorted.iter().position(|&x| x == "a").unwrap();
        let pos_b = sorted.iter().position(|&x| x == "b").unwrap();
        let pos_c = sorted.iter().position(|&x| x == "c").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_topological_sort_cycle() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A").with_depends_on(vec!["b".to_string()]),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
        ]);

        let sorted = topological_sort(&plan);
        assert!(sorted.is_none());
    }

    #[test]
    fn test_get_root_tasks() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B"),
            PlanTask::new("c", "Task C").with_depends_on(vec!["a".to_string()]),
        ]);

        let roots = get_root_tasks(&plan);
        assert!(roots.contains(&"a"));
        assert!(roots.contains(&"b"));
        assert!(!roots.contains(&"c"));
    }

    #[test]
    fn test_get_dependent_tasks() {
        let plan = create_plan_with_tasks(vec![
            PlanTask::new("a", "Task A"),
            PlanTask::new("b", "Task B").with_depends_on(vec!["a".to_string()]),
            PlanTask::new("c", "Task C").with_depends_on(vec!["a".to_string()]),
            PlanTask::new("d", "Task D").with_depends_on(vec!["b".to_string()]),
        ]);

        let dependents = get_dependent_tasks(&plan, "a");
        assert!(dependents.contains(&"b"));
        assert!(dependents.contains(&"c"));
        assert!(!dependents.contains(&"d"));
    }
}
