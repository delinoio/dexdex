//! Planning prompt builder for CompositeTask execution.
//!
//! This module provides the prompt template for the planning agent that
//! generates PLAN.yaml files. The planning prompt instructs the AI agent on:
//! - The PLAN.yaml format and schema
//! - How to break down the user's request into tasks
//! - Best practices for task dependencies

/// Builds the full planning prompt by combining the system instructions
/// with the user's request.
///
/// # Arguments
/// * `user_prompt` - The user's original request/prompt
///
/// # Returns
/// A complete prompt string that instructs the AI agent to generate PLAN.yaml
pub fn build_planning_prompt(user_prompt: &str) -> String {
    format!(
        r#"You are a planning agent for DeliDev, an AI coding orchestration tool. Your task is to analyze the user's request and generate a PLAN.yaml file that breaks down the work into smaller, executable tasks.

## Your Goal

Create a file named `PLAN-{{random}}.yaml` (where {{random}} is a short random string like `a1b2c3`) in the repository root that defines a task graph for the AI coding agents to execute.

## PLAN.yaml Format

The file must follow this exact YAML structure:

```yaml
tasks:
  - id: string          # Unique identifier for this task (required)
    title: string       # Human-readable task title (optional, defaults to id)
    prompt: string      # Task description for the AI agent (required)
    branchName: string  # Custom git branch name (optional)
    dependsOn: string[] # IDs of tasks that must complete first (optional)
```

## Field Specifications

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier within this plan. Use lowercase with hyphens (e.g., "setup-db", "auth-api"). |
| `title` | string | No | Human-readable title. Defaults to `id` if not specified. |
| `prompt` | string | Yes | Detailed description of what the AI agent should do. Be specific about requirements. |
| `branchName` | string | No | Custom git branch name. If not specified, the system generates one. |
| `dependsOn` | string[] | No | List of task IDs that must complete before this task starts. |

## Example PLAN.yaml

```yaml
tasks:
  - id: "setup-db"
    title: "Setup Database Schema"
    prompt: "Create database schema for user authentication including users, sessions, and password_reset_tokens tables with proper indexes and foreign keys."
    branchName: "feature/auth-database"

  - id: "auth-utils"
    title: "Implement Auth Utilities"
    prompt: "Implement authentication utilities including password hashing with bcrypt, JWT token generation and validation, and session management helpers."
    dependsOn: ["setup-db"]

  - id: "auth-api"
    title: "REST API Endpoints"
    prompt: "Implement REST API endpoints for login, signup, logout, and password reset. Include input validation and proper error responses."
    dependsOn: ["setup-db", "auth-utils"]

  - id: "auth-tests"
    title: "Authentication Tests"
    prompt: "Write unit and integration tests for the authentication system covering all API endpoints and utility functions."
    dependsOn: ["auth-api"]
```

## Guidelines for Creating Tasks

### Task Granularity
- **Good**: "Implement login API endpoint with input validation" (focused, reviewable)
- **Too broad**: "Implement entire authentication system" (hard to review)
- **Too narrow**: "Add import statement" (excessive overhead)

### Dependencies
- Only add dependencies when truly necessary (one task needs another's output)
- Independent tasks should NOT have dependencies so they can run in parallel
- Avoid unnecessary linear chains when tasks can be parallelized

### Prompts
- Be specific about requirements and acceptance criteria
- Reference existing patterns in the codebase when applicable
- Include technical details needed for implementation

## Validation Rules (Your PLAN.yaml must follow these)

1. **Unique IDs**: Each task must have a unique `id`
2. **Valid References**: All IDs in `dependsOn` must reference existing task IDs
3. **No Cycles**: The dependency graph must be acyclic (no circular dependencies)
4. **Non-empty Prompts**: Each task must have a non-empty `prompt`

## Your Task

Analyze the following user request and create an appropriate PLAN.yaml file:

---

{user_prompt}

---

Instructions:
1. First, explore the codebase to understand its structure and existing patterns
2. Break down the user's request into logical, focused tasks
3. Identify dependencies between tasks
4. Create the PLAN.yaml file in the repository root with a random suffix (e.g., PLAN-x7k9m2.yaml)
5. Ensure the plan follows best practices for task granularity and parallelization

Create the PLAN.yaml file now."#,
        user_prompt = user_prompt
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_planning_prompt_contains_user_prompt() {
        let user_prompt = "Add user authentication to the app";
        let full_prompt = build_planning_prompt(user_prompt);

        assert!(full_prompt.contains(user_prompt));
    }

    #[test]
    fn test_build_planning_prompt_contains_format_instructions() {
        let full_prompt = build_planning_prompt("test");

        // Check for key format instructions
        assert!(full_prompt.contains("PLAN-{random}.yaml"));
        assert!(full_prompt.contains("tasks:"));
        assert!(full_prompt.contains("id:"));
        assert!(full_prompt.contains("prompt:"));
        assert!(full_prompt.contains("dependsOn:"));
    }

    #[test]
    fn test_build_planning_prompt_contains_validation_rules() {
        let full_prompt = build_planning_prompt("test");

        assert!(full_prompt.contains("Unique IDs"));
        assert!(full_prompt.contains("Valid References"));
        assert!(full_prompt.contains("No Cycles"));
        assert!(full_prompt.contains("Non-empty Prompts"));
    }

    #[test]
    fn test_build_planning_prompt_contains_example() {
        let full_prompt = build_planning_prompt("test");

        // Check for example content
        assert!(full_prompt.contains("setup-db"));
        assert!(full_prompt.contains("auth-api"));
    }
}
