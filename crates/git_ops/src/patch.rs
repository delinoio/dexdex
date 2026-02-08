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

/// Applies a git patch to a target repository directory and commits the
/// changes.
///
/// This function:
/// 1. Validates the target path is a git repository
/// 2. Applies the patch using `git apply`
/// 3. Stages all applied changes
/// 4. Commits with the provided message
///
/// Uses the system `git` command for all operations.
///
/// # Arguments
/// * `target_path` - Path to the target git repository (must contain `.git`)
/// * `patch` - The unified diff patch string to apply
/// * `commit_message` - The commit message for the resulting commit
///
/// # Errors
/// Returns an error if:
/// - The target path doesn't exist or isn't a directory
/// - The target path isn't a git repository
/// - The patch fails to apply
/// - The commit fails
pub async fn apply_patch_and_commit(
    target_path: &Path,
    patch: &str,
    commit_message: &str,
) -> GitResult<()> {
    use tokio::process::Command;

    // Validate target path exists and is a directory
    if !target_path.exists() {
        return Err(GitError::Other(format!(
            "Target path does not exist: {:?}",
            target_path
        )));
    }
    if !target_path.is_dir() {
        return Err(GitError::Other(format!(
            "Target path is not a directory: {:?}",
            target_path
        )));
    }

    // Validate it's a git repository
    let git_dir = target_path.join(".git");
    if !git_dir.exists() {
        return Err(GitError::Other(format!(
            "Target path is not a git repository (no .git directory): {:?}",
            target_path
        )));
    }

    // Verify the working tree is clean before applying the patch.
    // Applying a patch to a dirty working tree can cause confusing merge
    // conflicts and makes rollback unreliable.
    let status_output = tokio::process::Command::new("git")
        .arg("-C")
        .arg(target_path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .await
        .map_err(|e| GitError::Other(format!("Failed to check git status: {}", e)))?;

    if !status_output.status.success() {
        let stderr = String::from_utf8_lossy(&status_output.stderr);
        return Err(GitError::Other(format!(
            "Failed to check repository status: {}",
            stderr
        )));
    }

    let status_text = String::from_utf8_lossy(&status_output.stdout);
    if !status_text.trim().is_empty() {
        return Err(GitError::Other(format!(
            "Target repository at {:?} has uncommitted changes. Please commit or stash them \
             before applying a patch.",
            target_path
        )));
    }

    info!(
        "Applying patch ({} bytes) to repository at {:?}",
        patch.len(),
        target_path
    );

    // Apply the patch using `git apply`
    let apply_output = Command::new("git")
        .arg("-C")
        .arg(target_path)
        .arg("apply")
        .arg("--verbose")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| GitError::Other(format!("Failed to spawn git apply: {}", e)))?;

    // Write the patch content to stdin
    use tokio::io::AsyncWriteExt;
    let mut child = apply_output;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(patch.as_bytes()).await.map_err(|e| {
            GitError::Other(format!("Failed to write patch to git apply stdin: {}", e))
        })?;
        // Drop stdin to signal EOF
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| GitError::Other(format!("Failed to wait for git apply: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("git apply failed: {}", stderr);
        return Err(GitError::Other(format!("git apply failed: {}", stderr)));
    }

    debug!("Patch applied successfully");

    // Check that git user.name and user.email are configured before attempting
    // to commit. Without these, `git commit` fails with a cryptic error.
    for config_key in &["user.name", "user.email"] {
        let config_check = Command::new("git")
            .arg("-C")
            .arg(target_path)
            .arg("config")
            .arg(config_key)
            .output()
            .await
            .map_err(|e| {
                GitError::Other(format!("Failed to check git config {}: {}", config_key, e))
            })?;

        if !config_check.status.success() {
            // Rollback the applied patch before returning the error
            warn!(
                "Git {} not configured in {:?}, rolling back applied patch",
                config_key, target_path
            );
            if let Err(rollback_err) = rollback_applied_changes(target_path).await {
                warn!("Rollback also failed: {}", rollback_err);
            }
            return Err(GitError::Other(format!(
                "Git {} is not configured in the target repository. Please run: git config {} \
                 \"<value>\"",
                config_key, config_key
            )));
        }

        // Validate that the config value is not empty or whitespace-only
        let config_value = String::from_utf8_lossy(&config_check.stdout);
        if config_value.trim().is_empty() {
            warn!(
                "Git {} is empty in {:?}, rolling back applied patch",
                config_key, target_path
            );
            if let Err(rollback_err) = rollback_applied_changes(target_path).await {
                warn!("Rollback also failed: {}", rollback_err);
            }
            return Err(GitError::Other(format!(
                "Git {} is set but empty in the target repository. Please run: git config {} \
                 \"<value>\"",
                config_key, config_key
            )));
        }
    }

    // Stage all applied changes
    let add_output = Command::new("git")
        .arg("-C")
        .arg(target_path)
        .arg("add")
        .arg("-A")
        .output()
        .await
        .map_err(|e| GitError::Other(format!("Failed to run git add: {}", e)))?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        warn!("git add failed after applying patch: {}", stderr);
        // Rollback the applied patch on failure
        if let Err(rollback_err) = rollback_applied_changes(target_path).await {
            warn!("Rollback also failed: {}", rollback_err);
        }
        return Err(GitError::Other(format!(
            "git add failed after applying patch: {}",
            stderr
        )));
    }

    // Commit the changes
    let commit_output = Command::new("git")
        .arg("-C")
        .arg(target_path)
        .arg("commit")
        .arg("-m")
        .arg(commit_message)
        .output()
        .await
        .map_err(|e| GitError::Other(format!("Failed to run git commit: {}", e)))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        warn!("git commit failed: {}", stderr);
        // Rollback the applied and staged changes on failure
        if let Err(rollback_err) = rollback_applied_changes(target_path).await {
            warn!("Rollback also failed: {}", rollback_err);
        }
        return Err(GitError::Other(format!("git commit failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&commit_output.stdout);
    info!(
        "Successfully committed changes to {:?}: {}",
        target_path,
        stdout.trim()
    );

    Ok(())
}

/// Rolls back uncommitted changes in the target repository.
///
/// This is used to clean up when `git add` or `git commit` fails after
/// `git apply` has already modified the working tree. Uses `git reset --hard
/// HEAD` as the primary mechanism since it atomically resets both the index
/// and working tree for tracked files, then `git clean -fd` to remove any
/// untracked files added by the patch.
///
/// Returns `Ok(())` if rollback succeeded, or `Err` if rollback failed
/// (indicating the repository may be in an inconsistent state).
async fn rollback_applied_changes(target_path: &Path) -> GitResult<()> {
    use tokio::process::Command;

    warn!(
        "Rolling back applied changes in {:?} due to post-apply failure",
        target_path
    );

    // Use `git reset --hard HEAD` which atomically resets both the index and
    // working tree for tracked files in a single operation.
    let reset_output = Command::new("git")
        .arg("-C")
        .arg(target_path)
        .arg("reset")
        .arg("--hard")
        .arg("HEAD")
        .output()
        .await
        .map_err(|e| {
            GitError::Other(format!(
                "Failed to run git reset --hard HEAD during rollback: {}",
                e
            ))
        })?;

    if !reset_output.status.success() {
        let stderr = String::from_utf8_lossy(&reset_output.stderr);
        warn!("git reset --hard HEAD failed during rollback: {}", stderr);
        return Err(GitError::Other(format!(
            "Rollback failed: git reset --hard HEAD failed: {}. Repository at {:?} may be in an \
             inconsistent state.",
            stderr, target_path
        )));
    }

    debug!("git reset --hard HEAD succeeded for {:?}", target_path);

    // Remove any untracked files that were added by the patch.
    // This is needed because `git reset --hard` only affects tracked files.
    let clean_output = Command::new("git")
        .arg("-C")
        .arg(target_path)
        .arg("clean")
        .arg("-fd")
        .output()
        .await
        .map_err(|e| {
            GitError::Other(format!(
                "Failed to run git clean -fd during rollback: {}",
                e
            ))
        })?;

    if !clean_output.status.success() {
        let stderr = String::from_utf8_lossy(&clean_output.stderr);
        warn!("git clean -fd failed during rollback: {}", stderr);
        return Err(GitError::Other(format!(
            "Rollback partially failed: git reset --hard succeeded but git clean -fd failed: {}. \
             Untracked files may remain in {:?}.",
            stderr, target_path
        )));
    }

    info!(
        "Rollback completed successfully for {:?}. Working tree is clean.",
        target_path
    );
    Ok(())
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

    /// Helper to initialize a test git repo with an initial commit.
    fn init_test_repo(path: &Path) {
        std::process::Command::new("git")
            .arg("init")
            .arg(path)
            .output()
            .unwrap();

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
    }

    #[tokio::test]
    async fn test_apply_patch_and_commit() {
        // Create source repo with changes to generate a patch
        let source_dir = tempdir().unwrap();
        let source_path = source_dir.path();
        init_test_repo(source_path);

        // Make changes in source
        fs::write(source_path.join("README.md"), "# Updated\n\nNew content").unwrap();
        fs::write(source_path.join("new_file.txt"), "New file content").unwrap();

        // Generate patch from source
        let patch = generate_patch(source_path).unwrap().unwrap();

        // Create target repo (same initial state as source)
        let target_dir = tempdir().unwrap();
        let target_path = target_dir.path();
        init_test_repo(target_path);

        // Apply the patch to target
        apply_patch_and_commit(target_path, &patch, "Apply changes from task")
            .await
            .unwrap();

        // Verify the changes were applied
        let readme_content = fs::read_to_string(target_path.join("README.md")).unwrap();
        assert_eq!(readme_content, "# Updated\n\nNew content");

        let new_file_content = fs::read_to_string(target_path.join("new_file.txt")).unwrap();
        assert_eq!(new_file_content, "New file content");

        // Verify a commit was created
        let log_output = std::process::Command::new("git")
            .arg("-C")
            .arg(target_path)
            .arg("log")
            .arg("--oneline")
            .output()
            .unwrap();
        let log = String::from_utf8_lossy(&log_output.stdout);
        assert!(
            log.contains("Apply changes from task"),
            "commit message should be in git log"
        );

        // Verify working tree is clean
        let status_output = std::process::Command::new("git")
            .arg("-C")
            .arg(target_path)
            .arg("status")
            .arg("--porcelain")
            .output()
            .unwrap();
        let status = String::from_utf8_lossy(&status_output.stdout);
        assert!(
            status.trim().is_empty(),
            "working tree should be clean after apply_patch_and_commit"
        );
    }

    #[tokio::test]
    async fn test_apply_patch_and_commit_non_git_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        // Not a git repo, just a regular directory

        let result = apply_patch_and_commit(path, "some patch", "commit msg").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not a git repository"),
            "error should mention not a git repository, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_apply_patch_and_commit_nonexistent_path() {
        let path = Path::new("/tmp/nonexistent_test_path_12345");

        let result = apply_patch_and_commit(path, "some patch", "commit msg").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("does not exist"),
            "error should mention path does not exist, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_apply_patch_and_commit_invalid_patch() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        init_test_repo(path);

        // Attempt to apply a malformed patch that `git apply` cannot parse
        let invalid_patch =
            "this is not a valid patch format at all\nrandom garbage data\n+++ b/file.txt\n";

        let result = apply_patch_and_commit(path, invalid_patch, "should fail").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("git apply failed"),
            "error should mention git apply failed, got: {}",
            err
        );

        // Verify working tree is still clean (no leftover changes)
        let status_output = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("status")
            .arg("--porcelain")
            .output()
            .unwrap();
        let status = String::from_utf8_lossy(&status_output.stdout);
        assert!(
            status.trim().is_empty(),
            "working tree should be clean after failed apply, got: {}",
            status
        );
    }

    #[tokio::test]
    async fn test_apply_patch_and_commit_conflicting_patch() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        init_test_repo(path);

        // Overwrite the README.md with different content so the patch won't
        // apply cleanly (it expects the original "# Test" content).
        fs::write(path.join("README.md"), "Completely different content").unwrap();
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
            .arg("diverge")
            .output()
            .unwrap();

        // Build a valid-looking patch that expects old content "# Test"
        let conflicting_patch = "\
diff --git a/README.md b/README.md
index 8ae0569..c738acd 100644
--- a/README.md
+++ b/README.md
@@ -1 +1,3 @@
 # Test
+
+Added line that conflicts with current content
";

        let result = apply_patch_and_commit(path, conflicting_patch, "should conflict").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("git apply failed"),
            "error should mention git apply failed, got: {}",
            err
        );

        // Verify working tree is still clean after failed apply
        let status_output = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("status")
            .arg("--porcelain")
            .output()
            .unwrap();
        let status = String::from_utf8_lossy(&status_output.stdout);
        assert!(
            status.trim().is_empty(),
            "working tree should be clean after conflicting apply, got: {}",
            status
        );
    }

    #[tokio::test]
    async fn test_apply_patch_and_commit_dirty_working_tree() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        init_test_repo(path);

        // Make uncommitted changes to dirty the working tree
        fs::write(path.join("README.md"), "# Dirty changes").unwrap();

        let valid_patch = "\
diff --git a/new_file.txt b/new_file.txt
new file mode 100644
index 0000000..3b18e51
--- /dev/null
+++ b/new_file.txt
@@ -0,0 +1 @@
+hello world
";

        let result =
            apply_patch_and_commit(path, valid_patch, "should fail due to dirty tree").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("uncommitted changes"),
            "error should mention uncommitted changes, got: {}",
            err
        );

        // Verify that the original dirty state is preserved (not wiped)
        let readme_content = fs::read_to_string(path.join("README.md")).unwrap();
        assert_eq!(
            readme_content, "# Dirty changes",
            "original uncommitted changes should be preserved"
        );
    }

    #[tokio::test]
    async fn test_apply_patch_and_commit_rollback_on_missing_git_config() {
        let dir = tempdir().unwrap();
        let path = dir.path();

        // Initialize a git repo WITHOUT user.name and user.email
        std::process::Command::new("git")
            .arg("init")
            .arg(path)
            .output()
            .unwrap();

        // Create initial commit using explicit author flags (bypasses config)
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
            .arg("-c")
            .arg("user.name=Temp")
            .arg("-c")
            .arg("user.email=temp@test.com")
            .arg("commit")
            .arg("-m")
            .arg("initial")
            .output()
            .unwrap();

        // Explicitly unset user.name and user.email
        let _ = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("--unset")
            .arg("user.name")
            .output();
        let _ = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("config")
            .arg("--unset")
            .arg("user.email")
            .output();

        // Create a valid patch
        let valid_patch = "\
diff --git a/new_file.txt b/new_file.txt
new file mode 100644
index 0000000..3b18e51
--- /dev/null
+++ b/new_file.txt
@@ -0,0 +1 @@
+hello world
";

        let result =
            apply_patch_and_commit(path, valid_patch, "should fail due to missing config").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("user.name") || err.contains("user.email"),
            "error should mention missing git config, got: {}",
            err
        );

        // Verify the working tree was rolled back (no leftover changes)
        let status_output = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("status")
            .arg("--porcelain")
            .output()
            .unwrap();
        let status = String::from_utf8_lossy(&status_output.stdout);
        assert!(
            status.trim().is_empty(),
            "working tree should be clean after rollback, got: {}",
            status
        );
    }
}
