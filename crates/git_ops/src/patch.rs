//! Git patch generation utilities.
//!
//! This module provides functionality for generating git patches (unified
//! diffs) from worktrees. Patches are used to persist changes in the database
//! when the worker server doesn't have write access to the repository.

use std::path::Path;

use tracing::{debug, info, warn};

use crate::{GitError, GitResult};

/// Maximum allowed patch size in bytes (10 MB).
/// Patches exceeding this limit are discarded to prevent database bloat.
const MAX_PATCH_SIZE: usize = 10 * 1024 * 1024;

/// Generates a git patch (unified diff) for all changes in a worktree
/// relative to HEAD.
///
/// This captures both staged and unstaged changes, including untracked files,
/// producing a patch that can be applied with `git apply` to reproduce the
/// exact state of the worktree.
///
/// Uses the system `git` command because git2 is compiled without TLS support.
///
/// # Arguments
/// * `worktree_path` - Path to the git worktree
///
/// # Returns
/// The patch as a string in unified diff format, or `None` if there are no
/// changes.
pub fn generate_patch(worktree_path: &Path) -> GitResult<Option<String>> {
    use std::process::Command;

    debug!("Generating git patch for worktree at {:?}", worktree_path);

    // Stage all changes (including untracked files) so they appear in
    // the diff. We use `git add -A` to capture everything the agent produced.
    let add_output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("add")
        .arg("-A")
        .output()
        .map_err(|e| GitError::Other(format!("Failed to run git add: {}", e)))?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        warn!("git add failed: {}", stderr);
        return Err(GitError::Other(format!("git add failed: {}", stderr)));
    }

    // Generate the diff of staged changes against HEAD
    let diff_output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg("--cached")
        .arg("--binary")
        .output()
        .map_err(|e| GitError::Other(format!("Failed to run git diff: {}", e)))?;

    // Unstage changes to avoid mutating the worktree state. This ensures
    // repeated calls produce consistent results and the worktree remains
    // in its original state for inspection.
    let reset_output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("reset")
        .arg("HEAD")
        .output();
    if let Err(e) = reset_output {
        warn!("Failed to unstage changes after patch generation: {}", e);
    }

    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        warn!("git diff failed: {}", stderr);
        return Err(GitError::Other(format!("git diff failed: {}", stderr)));
    }

    let patch = String::from_utf8_lossy(&diff_output.stdout).to_string();

    if patch.trim().is_empty() {
        debug!("No changes detected in worktree");
        Ok(None)
    } else if patch.len() > MAX_PATCH_SIZE {
        warn!(
            "Generated patch is too large ({} bytes, limit {} bytes) for worktree at {:?}, \
             skipping",
            patch.len(),
            MAX_PATCH_SIZE,
            worktree_path
        );
        Ok(None)
    } else {
        info!(
            "Generated patch ({} bytes) for worktree at {:?}",
            patch.len(),
            worktree_path
        );
        Ok(Some(patch))
    }
}

/// Generates a git patch (unified diff) for all changes in a worktree
/// relative to HEAD (async version).
///
/// See [`generate_patch`] for details.
pub async fn generate_patch_async(worktree_path: &Path) -> GitResult<Option<String>> {
    use tokio::process::Command;

    debug!(
        "Generating git patch (async) for worktree at {:?}",
        worktree_path
    );

    // Stage all changes including untracked files
    let add_output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("add")
        .arg("-A")
        .output()
        .await
        .map_err(|e| GitError::Other(format!("Failed to run git add: {}", e)))?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        warn!("git add failed: {}", stderr);
        return Err(GitError::Other(format!("git add failed: {}", stderr)));
    }

    // Generate the diff of staged changes against HEAD
    let diff_output = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("diff")
        .arg("--cached")
        .arg("--binary")
        .output()
        .await
        .map_err(|e| GitError::Other(format!("Failed to run git diff: {}", e)))?;

    // Unstage changes to avoid mutating the worktree state. This ensures
    // repeated calls produce consistent results and the worktree remains
    // in its original state for inspection.
    let reset_result = Command::new("git")
        .arg("-C")
        .arg(worktree_path)
        .arg("reset")
        .arg("HEAD")
        .output()
        .await;
    if let Err(e) = reset_result {
        warn!("Failed to unstage changes after patch generation: {}", e);
    }

    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        warn!("git diff failed: {}", stderr);
        return Err(GitError::Other(format!("git diff failed: {}", stderr)));
    }

    let patch = String::from_utf8_lossy(&diff_output.stdout).to_string();

    if patch.trim().is_empty() {
        debug!("No changes detected in worktree");
        Ok(None)
    } else if patch.len() > MAX_PATCH_SIZE {
        warn!(
            "Generated patch is too large ({} bytes, limit {} bytes) for worktree at {:?}, \
             skipping",
            patch.len(),
            MAX_PATCH_SIZE,
            worktree_path
        );
        Ok(None)
    } else {
        info!(
            "Generated patch ({} bytes) for worktree at {:?}",
            patch.len(),
            worktree_path
        );
        Ok(Some(patch))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_generate_patch_no_changes() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Init a git repo
        std::process::Command::new("git")
            .arg("init")
            .arg(path)
            .output()
            .unwrap();

        // Configure git user for the test repo
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.email")
            .arg("test@test.com")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.name")
            .arg("Test")
            .output()
            .unwrap();

        // Create initial commit
        fs::write(path.join("README.md"), "# Test").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("add")
            .arg("-A")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("commit")
            .arg("-m")
            .arg("initial")
            .output()
            .unwrap();

        // No changes => no patch
        let patch = generate_patch(path).unwrap();
        assert!(patch.is_none());
    }

    #[test]
    fn test_generate_patch_with_changes() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Init a git repo
        std::process::Command::new("git")
            .arg("init")
            .arg(path)
            .output()
            .unwrap();

        // Configure git user for the test repo
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.email")
            .arg("test@test.com")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.name")
            .arg("Test")
            .output()
            .unwrap();

        // Create initial commit
        fs::write(path.join("README.md"), "# Test").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("add")
            .arg("-A")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("commit")
            .arg("-m")
            .arg("initial")
            .output()
            .unwrap();

        // Make changes
        fs::write(path.join("README.md"), "# Test\n\nUpdated content").unwrap();
        fs::write(path.join("new_file.txt"), "New file content").unwrap();

        // Generate patch
        let patch = generate_patch(path).unwrap();
        assert!(patch.is_some());
        let patch = patch.unwrap();

        // Verify unified diff format
        assert!(
            patch.contains("diff --git"),
            "patch should contain unified diff headers"
        );
        assert!(patch.contains("README.md"));
        assert!(patch.contains("new_file.txt"));

        // Verify actual diff content
        assert!(
            patch.contains("+Updated content"),
            "patch should contain added lines"
        );
    }

    #[test]
    fn test_generate_patch_does_not_mutate_worktree() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Init a git repo
        std::process::Command::new("git")
            .arg("init")
            .arg(path)
            .output()
            .unwrap();

        // Configure git user for the test repo
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.email")
            .arg("test@test.com")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.name")
            .arg("Test")
            .output()
            .unwrap();

        // Create initial commit
        fs::write(path.join("README.md"), "# Test").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("add")
            .arg("-A")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("commit")
            .arg("-m")
            .arg("initial")
            .output()
            .unwrap();

        // Make changes
        fs::write(path.join("README.md"), "# Updated").unwrap();

        // Generate patch
        let patch = generate_patch(path).unwrap();
        assert!(patch.is_some());

        // Verify that the worktree has no staged changes (git reset was called)
        let status_output = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("diff")
            .arg("--cached")
            .arg("--name-only")
            .output()
            .unwrap();
        let staged = String::from_utf8_lossy(&status_output.stdout);
        assert!(
            staged.trim().is_empty(),
            "worktree should have no staged changes after generate_patch, but found: {}",
            staged
        );

        // Verify that calling generate_patch again produces the same result
        let patch2 = generate_patch(path).unwrap();
        assert!(patch2.is_some(), "second call should still produce a patch");
        assert_eq!(
            patch.unwrap(),
            patch2.unwrap(),
            "repeated calls should produce identical patches"
        );
    }

    #[tokio::test]
    async fn test_generate_patch_async_with_changes() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Init a git repo
        std::process::Command::new("git")
            .arg("init")
            .arg(path)
            .output()
            .unwrap();

        // Configure git user for the test repo
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.email")
            .arg("test@test.com")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("user.name")
            .arg("Test")
            .output()
            .unwrap();

        // Create initial commit
        fs::write(path.join("README.md"), "# Test").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("add")
            .arg("-A")
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("commit")
            .arg("-m")
            .arg("initial")
            .output()
            .unwrap();

        // Make changes
        fs::write(path.join("README.md"), "# Updated\n\nNew content").unwrap();

        // Generate patch async
        let patch = generate_patch_async(path).await.unwrap();
        assert!(patch.is_some());
        let patch = patch.unwrap();

        // Verify unified diff format and content
        assert!(
            patch.contains("diff --git"),
            "patch should contain unified diff headers"
        );
        assert!(patch.contains("README.md"));
        assert!(
            patch.contains("+# Updated"),
            "patch should contain added lines"
        );

        // Verify worktree is not left with staged changes
        let status_output = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("diff")
            .arg("--cached")
            .arg("--name-only")
            .output()
            .unwrap();
        let staged = String::from_utf8_lossy(&status_output.stdout);
        assert!(
            staged.trim().is_empty(),
            "worktree should have no staged changes after generate_patch_async"
        );
    }
}
