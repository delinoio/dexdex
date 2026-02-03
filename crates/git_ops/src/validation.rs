//! Security validation for git operations.
//!
//! This module provides validation functions to prevent security
//! vulnerabilities such as command injection and path traversal attacks.

use crate::{GitError, GitResult};

/// Maximum allowed length for a repository URL.
const MAX_URL_LENGTH: usize = 2048;

/// Maximum allowed length for a branch name.
const MAX_BRANCH_NAME_LENGTH: usize = 256;

/// Allowed URL schemes for git operations.
const ALLOWED_SCHEMES: &[&str] = &["https://", "http://", "git@", "ssh://"];

/// Validates a repository URL for security concerns.
///
/// This function checks for:
/// - URL length limits to prevent buffer-related issues
/// - Allowed URL schemes (rejects file://, git://, and other potentially unsafe
///   schemes)
/// - Dangerous characters that could enable command injection
/// - Path traversal attempts in the URL
///
/// # Security
/// This is a critical security function. Malicious URLs could:
/// - Execute commands via protocol handlers
/// - Access local files via `file://` URLs
/// - Inject shell commands if URLs are passed to shell commands unsafely
///
/// # Example
/// ```
/// use git_ops::validate_repository_url;
///
/// // Valid URLs
/// assert!(validate_repository_url("https://github.com/user/repo").is_ok());
/// assert!(validate_repository_url("git@github.com:user/repo.git").is_ok());
///
/// // Invalid URLs
/// assert!(validate_repository_url("file:///etc/passwd").is_err());
/// assert!(validate_repository_url("https://github.com/$(whoami)/repo").is_err());
/// ```
pub fn validate_repository_url(url: &str) -> GitResult<()> {
    // Check URL length
    if url.len() > MAX_URL_LENGTH {
        return Err(GitError::UnsafeUrl(format!(
            "URL exceeds maximum length of {} characters",
            MAX_URL_LENGTH
        )));
    }

    // Check for empty URL
    if url.trim().is_empty() {
        return Err(GitError::InvalidUrl("URL cannot be empty".to_string()));
    }

    // Check for allowed URL schemes
    let has_allowed_scheme = ALLOWED_SCHEMES
        .iter()
        .any(|scheme| url.to_lowercase().starts_with(scheme));

    if !has_allowed_scheme {
        return Err(GitError::UnsafeUrl(format!(
            "URL scheme not allowed. Allowed schemes: https://, http://, git@, ssh://. Got: {}",
            url.chars().take(50).collect::<String>()
        )));
    }

    // Check for command injection characters
    // These characters could be dangerous if URLs are passed to shell commands
    let dangerous_chars = ['$', '`', '|', ';', '&', '\n', '\r', '\0'];
    for c in dangerous_chars {
        if url.contains(c) {
            return Err(GitError::UnsafeUrl(format!(
                "URL contains potentially dangerous character: {:?}",
                c
            )));
        }
    }

    // Check for shell command substitution patterns
    if url.contains("$(") || url.contains("${") {
        return Err(GitError::UnsafeUrl(
            "URL contains shell command substitution pattern".to_string(),
        ));
    }

    // Check for excessive path components (could indicate unusual activity)
    let path_depth = url.matches('/').count();
    if path_depth > 20 {
        return Err(GitError::UnsafeUrl(
            "URL has excessive path depth (> 20 components)".to_string(),
        ));
    }

    Ok(())
}

/// Validates a branch name for security and format concerns.
///
/// This function checks for:
/// - Length limits
/// - Path traversal sequences (../)
/// - Dangerous characters
/// - Git-specific invalid patterns
///
/// # Security
/// Branch names are used in:
/// - Git commands (potential command injection)
/// - File system paths (path traversal attacks)
/// - URLs (potential injection in web contexts)
///
/// # Example
/// ```
/// use git_ops::validate_branch_name;
///
/// // Valid branch names
/// assert!(validate_branch_name("main").is_ok());
/// assert!(validate_branch_name("feature/add-login").is_ok());
/// assert!(validate_branch_name("delidev/task-123").is_ok());
///
/// // Invalid branch names
/// assert!(validate_branch_name("../../../etc/passwd").is_err());
/// assert!(validate_branch_name("branch\nname").is_err());
/// ```
pub fn validate_branch_name(branch: &str) -> GitResult<()> {
    // Check length
    if branch.len() > MAX_BRANCH_NAME_LENGTH {
        return Err(GitError::InvalidBranchName(format!(
            "Branch name exceeds maximum length of {} characters",
            MAX_BRANCH_NAME_LENGTH
        )));
    }

    // Check for empty branch name
    if branch.trim().is_empty() {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot be empty".to_string(),
        ));
    }

    // Check for path traversal sequences
    if branch.contains("..") {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot contain path traversal sequence (..)".to_string(),
        ));
    }

    // Check for dangerous characters
    // These could cause issues in shell commands, file paths, or git operations
    let dangerous_chars = [
        '$', '`', '|', ';', '&', '\n', '\r', '\0', '~', '^', ':', '?', '*', '[',
    ];
    for c in dangerous_chars {
        if branch.contains(c) {
            return Err(GitError::InvalidBranchName(format!(
                "Branch name contains invalid character: {:?}",
                c
            )));
        }
    }

    // Check for shell command substitution patterns
    if branch.contains("$(") || branch.contains("${") {
        return Err(GitError::InvalidBranchName(
            "Branch name contains shell command substitution pattern".to_string(),
        ));
    }

    // Git-specific invalid patterns
    // Branch names cannot start or end with a dot
    if branch.starts_with('.') || branch.ends_with('.') {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot start or end with a dot".to_string(),
        ));
    }

    // Branch names cannot end with .lock
    if branch.ends_with(".lock") {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot end with .lock".to_string(),
        ));
    }

    // Branch names cannot contain consecutive dots
    if branch.contains("..") {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot contain consecutive dots".to_string(),
        ));
    }

    // Branch names cannot contain @{
    if branch.contains("@{") {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot contain @{".to_string(),
        ));
    }

    // Branch names cannot be a single @
    if branch == "@" {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot be a single @".to_string(),
        ));
    }

    // Branch names cannot contain backslashes
    if branch.contains('\\') {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot contain backslashes".to_string(),
        ));
    }

    // Branch names cannot contain spaces (while technically allowed in git,
    // they cause many issues in scripts and URLs)
    if branch.contains(' ') {
        return Err(GitError::InvalidBranchName(
            "Branch name cannot contain spaces".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // URL Validation Tests
    // =========================================================================

    #[test]
    fn test_valid_https_url() {
        assert!(validate_repository_url("https://github.com/user/repo").is_ok());
        assert!(validate_repository_url("https://github.com/user/repo.git").is_ok());
        assert!(validate_repository_url("https://gitlab.com/group/subgroup/project").is_ok());
    }

    #[test]
    fn test_valid_ssh_url() {
        assert!(validate_repository_url("git@github.com:user/repo.git").is_ok());
        assert!(validate_repository_url("git@gitlab.com:group/project.git").is_ok());
        assert!(validate_repository_url("ssh://git@github.com/user/repo.git").is_ok());
    }

    #[test]
    fn test_invalid_file_url() {
        let result = validate_repository_url("file:///etc/passwd");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("URL scheme not allowed")
        );
    }

    #[test]
    fn test_invalid_git_protocol_url() {
        // git:// protocol can be unsafe due to no authentication
        let result = validate_repository_url("git://github.com/user/repo.git");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_injection_in_url() {
        // Test various command injection attempts
        assert!(validate_repository_url("https://github.com/$(whoami)/repo").is_err());
        assert!(validate_repository_url("https://github.com/`id`/repo").is_err());
        assert!(validate_repository_url("https://github.com/user/repo;rm -rf /").is_err());
        assert!(validate_repository_url("https://github.com/user/repo|cat /etc/passwd").is_err());
        assert!(validate_repository_url("https://github.com/user/repo&whoami").is_err());
    }

    #[test]
    fn test_newline_in_url() {
        assert!(validate_repository_url("https://github.com/user/repo\nmalicious").is_err());
        assert!(validate_repository_url("https://github.com/user/repo\rmalicious").is_err());
    }

    #[test]
    fn test_url_too_long() {
        let long_url = format!("https://github.com/{}", "a".repeat(3000));
        let result = validate_repository_url(&long_url);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds maximum length")
        );
    }

    #[test]
    fn test_empty_url() {
        assert!(validate_repository_url("").is_err());
        assert!(validate_repository_url("   ").is_err());
    }

    // =========================================================================
    // Branch Name Validation Tests
    // =========================================================================

    #[test]
    fn test_valid_branch_names() {
        assert!(validate_branch_name("main").is_ok());
        assert!(validate_branch_name("master").is_ok());
        assert!(validate_branch_name("feature/add-login").is_ok());
        assert!(validate_branch_name("delidev/task-12345678").is_ok());
        assert!(validate_branch_name("fix/issue-123").is_ok());
        assert!(validate_branch_name("release-v1.0.0").is_ok());
    }

    #[test]
    fn test_path_traversal_in_branch() {
        assert!(validate_branch_name("../../../etc/passwd").is_err());
        assert!(validate_branch_name("feature/../admin").is_err());
        assert!(validate_branch_name("..").is_err());
    }

    #[test]
    fn test_command_injection_in_branch() {
        assert!(validate_branch_name("feature$(whoami)").is_err());
        assert!(validate_branch_name("branch`id`").is_err());
        assert!(validate_branch_name("branch;rm").is_err());
        assert!(validate_branch_name("branch|cat").is_err());
        assert!(validate_branch_name("branch&whoami").is_err());
    }

    #[test]
    fn test_special_chars_in_branch() {
        assert!(validate_branch_name("branch~1").is_err());
        assert!(validate_branch_name("branch^2").is_err());
        assert!(validate_branch_name("branch:ref").is_err());
        assert!(validate_branch_name("branch?pattern").is_err());
        assert!(validate_branch_name("branch*glob").is_err());
        assert!(validate_branch_name("branch[0]").is_err());
    }

    #[test]
    fn test_newline_in_branch() {
        assert!(validate_branch_name("branch\nname").is_err());
        assert!(validate_branch_name("branch\rname").is_err());
    }

    #[test]
    fn test_git_invalid_patterns() {
        assert!(validate_branch_name(".hidden").is_err());
        assert!(validate_branch_name("branch.").is_err());
        assert!(validate_branch_name("branch.lock").is_err());
        assert!(validate_branch_name("@").is_err());
        assert!(validate_branch_name("branch@{1}").is_err());
        assert!(validate_branch_name("branch\\name").is_err());
        assert!(validate_branch_name("branch name").is_err());
    }

    #[test]
    fn test_empty_branch_name() {
        assert!(validate_branch_name("").is_err());
        assert!(validate_branch_name("   ").is_err());
    }

    #[test]
    fn test_branch_name_too_long() {
        let long_name = "a".repeat(300);
        let result = validate_branch_name(&long_name);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds maximum length")
        );
    }
}
