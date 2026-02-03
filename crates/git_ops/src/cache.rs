//! Repository cache management.
//!
//! This module provides functionality for caching cloned repositories
//! and creating worktrees from them for task execution.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::{
    FetchOpts, GitCredentials, GitError, GitRepository, GitResult, WorktreeExt, WorktreeOptions,
};

/// Default cache directory name within the data directory.
const CACHE_DIR_NAME: &str = "repo-cache";

/// Default worktrees directory name within the data directory.
const WORKTREES_DIR_NAME: &str = "worktrees";

/// Repository cache manager.
///
/// Manages a cache of bare git repositories and creates worktrees from them
/// for task execution. This improves performance by avoiding repeated full
/// clones of repositories.
#[derive(Clone)]
pub struct RepositoryCache {
    /// Base directory for the cache.
    cache_dir: PathBuf,
    /// Base directory for worktrees.
    worktrees_dir: PathBuf,
}

impl RepositoryCache {
    /// Creates a new repository cache manager.
    ///
    /// # Arguments
    /// * `data_dir` - The base data directory (e.g., `~/.delidev`)
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref();
        Self {
            cache_dir: data_dir.join(CACHE_DIR_NAME),
            worktrees_dir: data_dir.join(WORKTREES_DIR_NAME),
        }
    }

    /// Creates a new repository cache manager with custom directories.
    pub fn with_dirs(cache_dir: impl AsRef<Path>, worktrees_dir: impl AsRef<Path>) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
            worktrees_dir: worktrees_dir.as_ref().to_path_buf(),
        }
    }

    /// Returns the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Returns the worktrees directory path.
    pub fn worktrees_dir(&self) -> &Path {
        &self.worktrees_dir
    }

    /// Converts a remote URL to a cache directory name.
    ///
    /// Uses SHA256 hash of the normalized URL to guarantee uniqueness
    /// and avoid collisions from different URLs mapping to the same name.
    /// The URL is normalized by stripping any userinfo (credentials) and
    /// the `.git` suffix.
    fn url_to_cache_name(remote_url: &str) -> String {
        // Normalize the URL by stripping credentials and .git suffix
        let normalized = Self::strip_userinfo_from_url(remote_url)
            .trim_end_matches(".git")
            .trim_end_matches('/')
            .to_string();

        // Use SHA256 hash for unique, collision-free directory names
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        let hash = hasher.finalize();

        // Use first 16 bytes (32 hex chars) for readable yet unique names
        format!("{:x}", hash)[..32].to_string()
    }

    /// Strips userinfo (credentials) from a URL.
    ///
    /// Examples:
    /// - `https://token@github.com/user/repo` -> `https://github.com/user/repo`
    /// - `git@github.com:user/repo` -> `git@github.com:user/repo` (no change
    ///   for SSH)
    fn strip_userinfo_from_url(url: &str) -> String {
        // Handle HTTPS/HTTP URLs with embedded credentials
        if let Some(rest) = url.strip_prefix("https://") {
            if let Some(at_pos) = rest.find('@') {
                // Check if '@' appears before the first '/'
                let slash_pos = rest.find('/').unwrap_or(rest.len());
                if at_pos < slash_pos {
                    return format!("https://{}", &rest[at_pos + 1..]);
                }
            }
            return url.to_string();
        }
        if let Some(rest) = url.strip_prefix("http://") {
            if let Some(at_pos) = rest.find('@') {
                let slash_pos = rest.find('/').unwrap_or(rest.len());
                if at_pos < slash_pos {
                    return format!("http://{}", &rest[at_pos + 1..]);
                }
            }
            return url.to_string();
        }
        // For git@ SSH URLs, keep as-is (the user part is not a credential)
        url.to_string()
    }

    /// Extracts a redacted form of the URL safe for logging.
    ///
    /// Returns host and path without any credentials.
    fn redact_url_for_logging(url: &str) -> String {
        Self::strip_userinfo_from_url(url)
    }

    /// Returns the path to the cached repository for a given URL.
    pub fn cached_repo_path(&self, remote_url: &str) -> PathBuf {
        self.cache_dir.join(Self::url_to_cache_name(remote_url))
    }

    /// Ensures a repository is cached (clones if not present, fetches if
    /// present).
    ///
    /// Uses a lockfile to prevent TOCTOU race conditions when multiple
    /// concurrent tasks try to clone the same repository simultaneously.
    ///
    /// For HTTPS URLs, uses the system `git` command because git2 is compiled
    /// without TLS support (to avoid OpenSSL dependency for iOS
    /// cross-compilation).
    ///
    /// Returns the path to the cached bare repository.
    pub fn ensure_cached(
        &self,
        remote_url: &str,
        credentials: Option<GitCredentials>,
    ) -> GitResult<PathBuf> {
        let cache_path = self.cached_repo_path(remote_url);
        let redacted_url = Self::redact_url_for_logging(remote_url);

        // Ensure cache directory exists for lockfile
        std::fs::create_dir_all(&self.cache_dir)?;

        // Use a lockfile to prevent concurrent clones/fetches to the same repo
        let lock_path = self
            .cache_dir
            .join(format!("{}.lock", Self::url_to_cache_name(remote_url)));
        let mut lock = fslock::LockFile::open(&lock_path).map_err(|e| {
            crate::GitError::Io(std::io::Error::other(format!(
                "Failed to open lockfile: {}",
                e
            )))
        })?;
        lock.lock().map_err(|e| {
            crate::GitError::Io(std::io::Error::other(format!(
                "Failed to acquire lock: {}",
                e
            )))
        })?;

        // Determine if we need to use system git (for HTTPS URLs)
        let use_system_git = Self::needs_system_git(remote_url);

        // Re-check existence after acquiring lock (another process may have cloned)
        if cache_path.exists() {
            // Repository is cached, fetch latest changes
            debug!("Fetching updates for cached repository: {}", redacted_url);

            if use_system_git {
                Self::fetch_with_system_git(&cache_path, credentials.as_ref())?;
            } else {
                let repo = GitRepository::open(&cache_path)?;
                repo.fetch(
                    "origin",
                    FetchOpts {
                        credentials,
                        prune: true,
                    },
                )?;
            }
            info!("Updated cached repository: {}", redacted_url);
        } else {
            // Clone as bare repository
            info!("Cloning repository to cache: {}", redacted_url);

            if use_system_git {
                Self::clone_with_system_git(remote_url, &cache_path, credentials.as_ref())?;
            } else {
                // For non-HTTPS URLs, try system git first as fallback
                if let Err(e) =
                    Self::clone_with_system_git(remote_url, &cache_path, credentials.as_ref())
                {
                    warn!(
                        "System git clone failed ({}), this may indicate git is not installed",
                        e
                    );
                    return Err(GitError::CloneFailed(format!(
                        "Clone failed: {}. Please ensure git is installed and accessible.",
                        e
                    )));
                }
            }
            info!("Cached repository: {}", redacted_url);
        }

        // Lock is automatically released when `lock` goes out of scope
        Ok(cache_path)
    }

    /// Checks if the URL requires system git (HTTPS URLs need TLS which git2
    /// doesn't have).
    fn needs_system_git(url: &str) -> bool {
        url.starts_with("https://") || url.starts_with("http://")
    }

    /// Clones a repository using the system git command (blocking).
    ///
    /// Note: This is a synchronous operation. Consider using
    /// `clone_with_system_git_async` for non-blocking behavior.
    ///
    /// # Security
    /// Credentials are passed via environment variables (GIT_ASKPASS) rather than
    /// being embedded in the URL to prevent exposure via process listings.
    fn clone_with_system_git(
        url: &str,
        path: &Path,
        credentials: Option<&GitCredentials>,
    ) -> GitResult<()> {
        use std::process::Command;

        let mut cmd = Command::new("git");
        cmd.arg("clone").arg("--bare");

        // Log redacted URL to avoid leaking credentials
        let redacted_url = Self::redact_url_for_logging(url);

        // SECURITY: Use environment-based authentication instead of embedding credentials in URL
        // This prevents credential exposure via process listings (ps command)
        Self::configure_git_auth(&mut cmd, credentials);

        cmd.arg(url).arg(path);

        debug!("Running: git clone --bare {} {:?}", redacted_url, path);

        let output = cmd
            .output()
            .map_err(|e| GitError::CloneFailed(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::CloneFailed(format!(
                "git clone failed: {}",
                stderr.trim()
            )));
        }

        Ok(())
    }

    /// Clones a repository using the system git command (async/non-blocking).
    ///
    /// # Security
    /// Credentials are passed via environment variables rather than
    /// being embedded in the URL to prevent exposure via process listings.
    pub async fn clone_with_system_git_async(
        url: &str,
        path: &Path,
        credentials: Option<&GitCredentials>,
    ) -> GitResult<()> {
        use tokio::process::Command;

        let mut cmd = Command::new("git");
        cmd.arg("clone").arg("--bare");

        // Log redacted URL to avoid leaking credentials
        let redacted_url = Self::redact_url_for_logging(url);

        // SECURITY: Use environment-based authentication instead of embedding credentials in URL
        Self::configure_git_auth_async(&mut cmd, credentials);

        cmd.arg(url).arg(path);

        debug!(
            "Running (async): git clone --bare {} {:?}",
            redacted_url, path
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| GitError::CloneFailed(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::CloneFailed(format!(
                "git clone failed: {}",
                stderr.trim()
            )));
        }

        Ok(())
    }

    /// Fetches from origin using the system git command (blocking).
    ///
    /// Note: This is a synchronous operation. Consider using
    /// `fetch_with_system_git_async` for non-blocking behavior.
    fn fetch_with_system_git(
        repo_path: &Path,
        credentials: Option<&GitCredentials>,
    ) -> GitResult<()> {
        use std::process::Command;

        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(repo_path)
            .arg("fetch")
            .arg("--prune")
            .arg("origin");

        // Note: For fetch, credentials in URL are already configured in the
        // repo's remote config from the initial clone. If we need to update
        // credentials, we'd need to modify the remote URL.

        // If credentials are provided, we might need to set them via
        // environment or credential helper. For now, we rely on the system's
        // git credential configuration.
        if credentials.is_some() {
            debug!("Note: Fetch credentials are handled by system git credential helpers");
        }

        debug!("Running: git -C {:?} fetch --prune origin", repo_path);

        let output = cmd
            .output()
            .map_err(|e| GitError::FetchFailed(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::FetchFailed(format!(
                "git fetch failed: {}",
                stderr.trim()
            )));
        }

        Ok(())
    }

    /// Fetches from origin using the system git command (async/non-blocking).
    pub async fn fetch_with_system_git_async(
        repo_path: &Path,
        credentials: Option<&GitCredentials>,
    ) -> GitResult<()> {
        use tokio::process::Command;

        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(repo_path)
            .arg("fetch")
            .arg("--prune")
            .arg("origin");

        if credentials.is_some() {
            debug!("Note: Fetch credentials are handled by system git credential helpers");
        }

        debug!(
            "Running (async): git -C {:?} fetch --prune origin",
            repo_path
        );

        let output = cmd
            .output()
            .await
            .map_err(|e| GitError::FetchFailed(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::FetchFailed(format!(
                "git fetch failed: {}",
                stderr.trim()
            )));
        }

        Ok(())
    }

    /// Finds the default branch (main or master) from the repository.
    ///
    /// For bare repositories (used in cache), branches are in refs/heads/*
    /// rather than refs/remotes/origin/*.
    fn find_default_branch(repo: &git2::Repository) -> GitResult<String> {
        // Try to find HEAD which points to the default branch
        if let Ok(head) = repo.find_reference("HEAD") {
            if let Ok(resolved) = head.resolve() {
                if let Some(name) = resolved.name() {
                    // For bare repos, HEAD points to refs/heads/main or similar
                    debug!("HEAD points to: {}", name);
                    return Ok(name.to_string());
                }
            }
        }

        // Fallback: try common default branch names (bare repo format)
        for branch in &["refs/heads/main", "refs/heads/master"] {
            if repo.find_reference(branch).is_ok() {
                debug!("Found default branch: {}", branch);
                return Ok(branch.to_string());
            }
        }

        // Also try the short names with revparse
        for branch in &["main", "master"] {
            if repo.revparse_single(branch).is_ok() {
                debug!("Found default branch via revparse: {}", branch);
                return Ok(format!("refs/heads/{}", branch));
            }
        }

        Err(GitError::Other(
            "Could not determine default branch. Neither main nor master found.".to_string(),
        ))
    }

    /// Configures git authentication via environment variables (blocking version).
    ///
    /// # Security
    /// This approach avoids embedding credentials in URLs which could be exposed
    /// via process listings (`ps` command) or error messages.
    fn configure_git_auth(cmd: &mut std::process::Command, credentials: Option<&GitCredentials>) {
        if let Some(GitCredentials::UserPass { username, password }) = credentials {
            // Use git's credential helper mechanism via environment variables
            // This is more secure than embedding credentials in the URL
            cmd.env(
                "GIT_CONFIG_COUNT",
                "2",
            );
            cmd.env(
                "GIT_CONFIG_KEY_0",
                "credential.helper",
            );
            cmd.env(
                "GIT_CONFIG_VALUE_0",
                "",
            );
            cmd.env(
                "GIT_CONFIG_KEY_1",
                "credential.helper",
            );
            // Use a shell command to echo credentials - this avoids URL embedding
            // The credentials are passed via environment variables, not visible in process list
            cmd.env("GIT_USERNAME", username);
            cmd.env("GIT_PASSWORD", password);
            cmd.env(
                "GIT_CONFIG_VALUE_1",
                "!f() { echo \"username=$GIT_USERNAME\"; echo \"password=$GIT_PASSWORD\"; }; f",
            );
        }
    }

    /// Configures git authentication via environment variables (async version).
    ///
    /// # Security
    /// This approach avoids embedding credentials in URLs which could be exposed
    /// via process listings (`ps` command) or error messages.
    fn configure_git_auth_async(
        cmd: &mut tokio::process::Command,
        credentials: Option<&GitCredentials>,
    ) {
        if let Some(GitCredentials::UserPass { username, password }) = credentials {
            // Use git's credential helper mechanism via environment variables
            cmd.env("GIT_CONFIG_COUNT", "2");
            cmd.env("GIT_CONFIG_KEY_0", "credential.helper");
            cmd.env("GIT_CONFIG_VALUE_0", "");
            cmd.env("GIT_CONFIG_KEY_1", "credential.helper");
            cmd.env("GIT_USERNAME", username);
            cmd.env("GIT_PASSWORD", password);
            cmd.env(
                "GIT_CONFIG_VALUE_1",
                "!f() { echo \"username=$GIT_USERNAME\"; echo \"password=$GIT_PASSWORD\"; }; f",
            );
        }
    }

    /// Sanitizes a task ID for use in file paths.
    ///
    /// This is critical for security - task_id could come from untrusted sources
    /// and must not contain path traversal characters like `..` or `/`.
    /// Only alphanumeric characters and hyphens are allowed.
    pub fn sanitize_task_id(task_id: &str) -> String {
        task_id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    }

    /// Creates a worktree from a cached repository for a specific task.
    ///
    /// # Arguments
    /// * `remote_url` - The remote repository URL
    /// * `branch_name` - The branch to checkout in the worktree
    /// * `task_id` - Unique identifier for the task (used in worktree name)
    /// * `credentials` - Optional git credentials for fetching
    ///
    /// # Returns
    /// The path to the created worktree.
    ///
    /// # Concurrency
    /// This function assumes task_id is unique per task. Concurrent calls with
    /// the same task_id will race and may cause issues.
    ///
    /// # Security
    /// The task_id is sanitized to prevent path traversal attacks.
    pub fn create_worktree_for_task(
        &self,
        remote_url: &str,
        branch_name: &str,
        task_id: &str,
        credentials: Option<GitCredentials>,
    ) -> GitResult<PathBuf> {
        // Ensure repository is cached and up-to-date
        let cache_path = self.ensure_cached(remote_url, credentials)?;

        // Open the cached repository
        let repo = GitRepository::open(&cache_path)?;
        let git_repo = repo.inner();

        // Create worktree directory
        std::fs::create_dir_all(&self.worktrees_dir)?;

        // Generate worktree name and path
        // SECURITY: Sanitize task_id to prevent path traversal attacks
        let sanitized_task_id = Self::sanitize_task_id(task_id);
        let worktree_name = format!(
            "{}-{}",
            sanitized_task_id,
            Self::sanitize_branch_name(branch_name)
        );
        let worktree_path = self.worktrees_dir.join(&worktree_name);

        // Remove existing worktree if it exists
        if worktree_path.exists() {
            debug!("Removing existing worktree at {:?}", worktree_path);
            // Try to remove the worktree properly first
            if let Err(e) = repo.remove_worktree(&worktree_name, true) {
                debug!("Could not remove worktree via git: {}", e);
            }
            // Then remove the directory
            std::fs::remove_dir_all(&worktree_path).map_err(|e| {
                crate::GitError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to remove worktree directory {:?}: {}",
                        worktree_path, e
                    ),
                ))
            })?;
        }

        // Check if the branch already exists locally
        let mut branch_exists = false;
        if let Ok(mut branch) = git_repo.find_branch(branch_name, git2::BranchType::Local) {
            // Try to delete the existing local branch to ensure we get fresh state
            if let Err(e) = branch.delete() {
                debug!(
                    "Failed to delete existing local branch {}: {}",
                    branch_name, e
                );
                // Branch deletion failed (might be checked out elsewhere),
                // but we'll try to proceed anyway
                branch_exists = true;
            }
            // If deletion succeeded, branch no longer exists
        }

        // Determine the base branch:
        // 1. If the branch already exists in the repo, use it (continuing work)
        // 2. Otherwise, use the default branch (creating a new feature branch)
        //
        // Note: In bare repos (used for cache), branches are in refs/heads/*
        // not refs/remotes/origin/*
        let branch_ref = format!("refs/heads/{}", branch_name);
        let base_branch = if git_repo.find_reference(&branch_ref).is_ok() {
            // Branch exists in the bare repo, use it as base
            debug!("Using existing branch {} as base", branch_ref);
            branch_ref
        } else {
            // Branch doesn't exist, find the default branch
            let default_branch = Self::find_default_branch(git_repo)?;
            debug!(
                "Branch {} not found, using default branch {} as base",
                branch_ref, default_branch
            );
            default_branch
        };

        // Create the worktree with the branch based on the determined base
        let options = WorktreeOptions {
            branch: Some(branch_name.to_string()),
            create_branch: !branch_exists,
            base: Some(base_branch),
        };

        repo.create_worktree(&worktree_name, &worktree_path, options)?;

        info!(
            "Created worktree for task {} at {:?}",
            task_id, worktree_path
        );

        Ok(worktree_path)
    }

    /// Removes a worktree for a task.
    ///
    /// # Security
    /// The task_id is sanitized to prevent path traversal attacks.
    pub fn remove_worktree_for_task(
        &self,
        remote_url: &str,
        task_id: &str,
        branch_name: &str,
    ) -> GitResult<()> {
        let cache_path = self.cached_repo_path(remote_url);
        // SECURITY: Sanitize task_id to prevent path traversal attacks
        let sanitized_task_id = Self::sanitize_task_id(task_id);
        let worktree_name = format!(
            "{}-{}",
            sanitized_task_id,
            Self::sanitize_branch_name(branch_name)
        );
        let worktree_path = self.worktrees_dir.join(&worktree_name);

        if cache_path.exists() {
            let repo = GitRepository::open(&cache_path)?;
            if let Err(e) = repo.remove_worktree(&worktree_name, true) {
                debug!("Could not remove worktree via git: {}", e);
            }
        }

        if worktree_path.exists() {
            std::fs::remove_dir_all(&worktree_path)?;
            info!("Removed worktree for task {}", task_id);
        }

        Ok(())
    }

    /// Lists all cached repositories.
    pub fn list_cached(&self) -> GitResult<Vec<String>> {
        if !self.cache_dir.exists() {
            return Ok(Vec::new());
        }

        let mut repos = Vec::new();
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    repos.push(name.to_string());
                }
            }
        }
        Ok(repos)
    }

    /// Removes a cached repository.
    pub fn remove_cached(&self, remote_url: &str) -> GitResult<()> {
        let cache_path = self.cached_repo_path(remote_url);
        let redacted_url = Self::redact_url_for_logging(remote_url);
        if cache_path.exists() {
            std::fs::remove_dir_all(&cache_path)?;
            info!("Removed cached repository: {}", redacted_url);
        }
        Ok(())
    }

    /// Clears all cached repositories.
    pub fn clear_cache(&self) -> GitResult<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            info!("Cleared repository cache");
        }
        Ok(())
    }

    /// Sanitizes a branch name for use in file paths.
    ///
    /// Replaces special characters with hyphens to ensure safe filesystem
    /// paths.
    pub fn sanitize_branch_name(branch: &str) -> String {
        branch
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '-'
                }
            })
            .collect()
    }
}

/// Returns the path to a worktree for a task.
///
/// This is a convenience function that computes the worktree path
/// without requiring a RepositoryCache instance.
///
/// # Security
/// The task_id is sanitized to prevent path traversal attacks.
pub fn worktree_path_for_task_with_cache(
    worktrees_dir: impl AsRef<Path>,
    task_id: &str,
    branch_name: &str,
) -> PathBuf {
    // SECURITY: Sanitize task_id to prevent path traversal attacks
    let sanitized_task_id = RepositoryCache::sanitize_task_id(task_id);
    let sanitized_branch = RepositoryCache::sanitize_branch_name(branch_name);

    worktrees_dir
        .as_ref()
        .join(format!("{}-{}", sanitized_task_id, sanitized_branch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_to_cache_name_is_deterministic() {
        // Same URL should always produce the same hash
        let hash1 = RepositoryCache::url_to_cache_name("https://github.com/user/repo.git");
        let hash2 = RepositoryCache::url_to_cache_name("https://github.com/user/repo.git");
        assert_eq!(hash1, hash2);
        // Hash should be 32 characters (first 16 bytes of SHA256 in hex)
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_url_to_cache_name_normalizes_git_suffix() {
        // With and without .git suffix should produce same hash
        let with_git = RepositoryCache::url_to_cache_name("https://github.com/user/repo.git");
        let without_git = RepositoryCache::url_to_cache_name("https://github.com/user/repo");
        assert_eq!(with_git, without_git);
    }

    #[test]
    fn test_url_to_cache_name_different_urls_different_hashes() {
        // Different URLs should produce different hashes
        let hash1 = RepositoryCache::url_to_cache_name("https://github.com/user/repo1");
        let hash2 = RepositoryCache::url_to_cache_name("https://github.com/user/repo2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_url_to_cache_name_strips_credentials() {
        // URL with credentials should produce same hash as without
        let with_creds = RepositoryCache::url_to_cache_name("https://token@github.com/user/repo");
        let without_creds = RepositoryCache::url_to_cache_name("https://github.com/user/repo");
        assert_eq!(with_creds, without_creds);
    }

    #[test]
    fn test_strip_userinfo_from_url() {
        assert_eq!(
            RepositoryCache::strip_userinfo_from_url("https://token@github.com/user/repo"),
            "https://github.com/user/repo"
        );
        assert_eq!(
            RepositoryCache::strip_userinfo_from_url("https://user:pass@github.com/user/repo"),
            "https://github.com/user/repo"
        );
        assert_eq!(
            RepositoryCache::strip_userinfo_from_url("https://github.com/user/repo"),
            "https://github.com/user/repo"
        );
        assert_eq!(
            RepositoryCache::strip_userinfo_from_url("git@github.com:user/repo"),
            "git@github.com:user/repo"
        );
    }

    #[test]
    fn test_cached_repo_path() {
        let cache = RepositoryCache::new("/home/user/.delidev");
        let path = cache.cached_repo_path("https://github.com/user/repo.git");
        // Path should be in cache dir with a hash name
        assert!(path.starts_with("/home/user/.delidev/repo-cache/"));
        // The hash component should be 32 characters
        let hash = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sanitize_branch_name() {
        assert_eq!(
            RepositoryCache::sanitize_branch_name("feature/add-login"),
            "feature-add-login"
        );
        assert_eq!(RepositoryCache::sanitize_branch_name("main"), "main");
        assert_eq!(
            RepositoryCache::sanitize_branch_name("fix/bug#123"),
            "fix-bug-123"
        );
    }

    #[test]
    fn test_sanitize_task_id() {
        // Normal UUIDs should be unchanged
        assert_eq!(
            RepositoryCache::sanitize_task_id("abc123-def456"),
            "abc123-def456"
        );
        // Path traversal attempts should be sanitized
        // "../../../etc/passwd" has 9 special chars (./): ..|/.|./.|./e... -> "---------etc-passwd"
        assert_eq!(
            RepositoryCache::sanitize_task_id("../../../etc/passwd"),
            "---------etc-passwd"
        );
        // "task/../../secret" has 7 special chars (/, ., ., /, ., ., /): -> "task-------secret"
        assert_eq!(
            RepositoryCache::sanitize_task_id("task/../../secret"),
            "task-------secret"
        );
        // Slashes should be converted to hyphens
        assert_eq!(
            RepositoryCache::sanitize_task_id("task/with/slashes"),
            "task-with-slashes"
        );
        // Underscores should be converted to hyphens (only alphanumeric and hyphens allowed)
        assert_eq!(
            RepositoryCache::sanitize_task_id("task_with_underscores"),
            "task-with-underscores"
        );
    }
}
