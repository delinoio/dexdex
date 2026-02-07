//! Git patch generation utilities.
//!
//! This module provides functionality for generating git patches (unified
//! diffs) from worktrees. Patches are used to persist changes in the database
//! when the worker server doesn't have write access to the repository.

use std::path::Path;

use tracing::{debug, warn};

use crate::{GitError, GitResult};

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

    // First, stage all changes (including untracked files) so they appear in
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

    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        warn!("git diff failed: {}", stderr);
        return Err(GitError::Other(format!("git diff failed: {}", stderr)));
    }

    let patch = String::from_utf8_lossy(&diff_output.stdout).to_string();

    if patch.trim().is_empty() {
        debug!("No changes detected in worktree");
        Ok(None)
    } else {
        debug!(
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

    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        warn!("git diff failed: {}", stderr);
        return Err(GitError::Other(format!("git diff failed: {}", stderr)));
    }

    let patch = String::from_utf8_lossy(&diff_output.stdout).to_string();

    if patch.trim().is_empty() {
        debug!("No changes detected in worktree");
        Ok(None)
    } else {
        debug!(
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
        assert!(patch.contains("README.md"));
        assert!(patch.contains("new_file.txt"));
        assert!(patch.contains("Updated content"));
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
        assert!(patch.contains("README.md"));
        assert!(patch.contains("Updated"));
    }
}
