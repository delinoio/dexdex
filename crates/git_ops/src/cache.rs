//! Repository cache management.
//!
//! This module provides functionality for caching cloned repositories
//! and creating worktrees from them for task execution.

use std::path::{Path, PathBuf};

use tracing::{debug, info};

use crate::{
    CloneOptions, FetchOpts, GitCredentials, GitError, GitRepository, GitResult, WorktreeExt,
    WorktreeOptions,
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
    /// This creates a deterministic directory name from the URL by:
    /// 1. Extracting the host and path
    /// 2. Replacing special characters with underscores
    /// 3. Removing the `.git` suffix if present
    fn url_to_cache_name(remote_url: &str) -> String {
        let url = remote_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("git@")
            .replace(':', "/")
            .trim_end_matches(".git")
            .to_string();

        // Replace special characters with underscores
        url.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Returns the path to the cached repository for a given URL.
    pub fn cached_repo_path(&self, remote_url: &str) -> PathBuf {
        self.cache_dir.join(Self::url_to_cache_name(remote_url))
    }

    /// Ensures a repository is cached (clones if not present, fetches if
    /// present).
    ///
    /// Returns the path to the cached bare repository.
    pub fn ensure_cached(
        &self,
        remote_url: &str,
        credentials: Option<GitCredentials>,
    ) -> GitResult<PathBuf> {
        let cache_path = self.cached_repo_path(remote_url);

        if cache_path.exists() {
            // Repository is cached, fetch latest changes
            debug!("Fetching updates for cached repository: {}", remote_url);
            let repo = GitRepository::open(&cache_path)?;
            repo.fetch(
                "origin",
                FetchOpts {
                    credentials,
                    prune: true,
                },
            )?;
            info!("Updated cached repository: {}", remote_url);
        } else {
            // Clone as bare repository
            info!("Cloning repository to cache: {}", remote_url);
            std::fs::create_dir_all(&self.cache_dir)?;

            GitRepository::clone(
                remote_url,
                &cache_path,
                CloneOptions {
                    bare: true,
                    credentials,
                    ..Default::default()
                },
            )?;
            info!("Cached repository: {}", remote_url);
        }

        Ok(cache_path)
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

        // Create worktree directory
        std::fs::create_dir_all(&self.worktrees_dir)?;

        // Generate worktree name and path
        let worktree_name = format!("{}-{}", task_id, Self::sanitize_branch_name(branch_name));
        let worktree_path = self.worktrees_dir.join(&worktree_name);

        // Remove existing worktree if it exists
        if worktree_path.exists() {
            debug!("Removing existing worktree at {:?}", worktree_path);
            // Try to remove the worktree properly first
            if let Err(e) = repo.remove_worktree(&worktree_name, true) {
                debug!("Could not remove worktree via git: {}", e);
            }
            // Then remove the directory
            std::fs::remove_dir_all(&worktree_path)?;
        }

        // Determine if we need to create the branch
        let remote_branch = format!("origin/{}", branch_name);
        let branch_exists = repo
            .inner()
            .find_branch(branch_name, git2::BranchType::Local)
            .is_ok();

        // Create the worktree
        let options = WorktreeOptions {
            branch: Some(branch_name.to_string()),
            create_branch: !branch_exists,
            base: if branch_exists {
                None
            } else {
                Some(remote_branch)
            },
        };

        repo.create_worktree(&worktree_name, &worktree_path, options)?;

        info!(
            "Created worktree for task {} at {:?}",
            task_id, worktree_path
        );

        Ok(worktree_path)
    }

    /// Removes a worktree for a task.
    pub fn remove_worktree_for_task(
        &self,
        remote_url: &str,
        task_id: &str,
        branch_name: &str,
    ) -> GitResult<()> {
        let cache_path = self.cached_repo_path(remote_url);
        let worktree_name = format!("{}-{}", task_id, Self::sanitize_branch_name(branch_name));
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
        if cache_path.exists() {
            std::fs::remove_dir_all(&cache_path)?;
            info!("Removed cached repository: {}", remote_url);
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
    fn sanitize_branch_name(branch: &str) -> String {
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
pub fn worktree_path_for_task_with_cache(
    worktrees_dir: impl AsRef<Path>,
    task_id: &str,
    branch_name: &str,
) -> PathBuf {
    let sanitized_branch = branch_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();

    worktrees_dir
        .as_ref()
        .join(format!("{}-{}", task_id, sanitized_branch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_to_cache_name() {
        assert_eq!(
            RepositoryCache::url_to_cache_name("https://github.com/user/repo.git"),
            "github_com_user_repo"
        );
        assert_eq!(
            RepositoryCache::url_to_cache_name("git@github.com:user/repo.git"),
            "github_com_user_repo"
        );
        assert_eq!(
            RepositoryCache::url_to_cache_name("https://gitlab.example.com/group/subgroup/repo"),
            "gitlab_example_com_group_subgroup_repo"
        );
    }

    #[test]
    fn test_cached_repo_path() {
        let cache = RepositoryCache::new("/home/user/.delidev");
        let path = cache.cached_repo_path("https://github.com/user/repo.git");
        assert_eq!(
            path,
            PathBuf::from("/home/user/.delidev/repo-cache/github_com_user_repo")
        );
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
}
