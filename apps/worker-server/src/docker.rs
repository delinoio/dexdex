//! Docker container management for AI agent execution.

use std::{collections::HashMap, path::Path};

use bollard::{
    Docker,
    container::{
        Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
        StopContainerOptions,
    },
    image::BuildImageOptions,
};
use futures::StreamExt;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    config::WorkerConfig,
    error::{WorkerError, WorkerResult},
};

/// Path inside the container where secrets are mounted.
pub const SECRETS_MOUNT_PATH: &str = "/run/secrets";

/// Docker manager for container lifecycle management.
#[derive(Clone)]
pub struct DockerManager {
    client: Docker,
    config: WorkerConfig,
}

impl DockerManager {
    /// Creates a new Docker manager.
    pub async fn new(config: &WorkerConfig) -> WorkerResult<Self> {
        let client = if config.docker_socket.starts_with("unix://") {
            Docker::connect_with_socket(&config.docker_socket, 120, bollard::API_DEFAULT_VERSION)?
        } else if config.docker_socket.starts_with('/') {
            Docker::connect_with_socket(
                &format!("unix://{}", config.docker_socket),
                120,
                bollard::API_DEFAULT_VERSION,
            )?
        } else {
            Docker::connect_with_socket_defaults()?
        };

        // Verify connection
        client.ping().await?;
        info!("Connected to Docker daemon");

        Ok(Self {
            client,
            config: config.clone(),
        })
    }

    /// Checks if a custom Dockerfile exists in the repository.
    pub fn has_custom_dockerfile(repo_path: &Path) -> bool {
        repo_path.join(".delidev/setup/Dockerfile").exists()
    }

    /// Builds a Docker image from the repository's custom Dockerfile.
    pub async fn build_custom_image(&self, repo_path: &Path, tag: &str) -> WorkerResult<String> {
        let dockerfile_path = repo_path.join(".delidev/setup/Dockerfile");
        if !dockerfile_path.exists() {
            return Err(WorkerError::Config(format!(
                "Custom Dockerfile not found at {:?}",
                dockerfile_path
            )));
        }

        info!("Building custom Docker image from {:?}", dockerfile_path);

        // Create build context from the .delidev/setup directory
        let context_path = repo_path.join(".delidev/setup");
        let tar = self.create_build_context(&context_path).await?;

        let options = BuildImageOptions {
            t: tag,
            dockerfile: "Dockerfile",
            rm: true,
            ..Default::default()
        };

        let mut stream = self.client.build_image(options, None, Some(tar.into()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    if let Some(stream) = output.stream {
                        debug!("Build: {}", stream.trim());
                    }
                    if let Some(error) = output.error {
                        error!("Build error: {}", error);
                        return Err(WorkerError::Docker(
                            bollard::errors::Error::DockerResponseServerError {
                                status_code: 500,
                                message: error,
                            },
                        ));
                    }
                }
                Err(e) => {
                    error!("Build stream error: {}", e);
                    return Err(WorkerError::Docker(e));
                }
            }
        }

        info!("Successfully built Docker image: {}", tag);
        Ok(tag.to_string())
    }

    /// Creates a tar archive from a directory for Docker build context.
    async fn create_build_context(&self, path: &Path) -> WorkerResult<Vec<u8>> {
        use tar::Builder;

        let mut archive = Vec::new();
        {
            let mut builder = Builder::new(&mut archive);

            // Add all files from the context directory
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                let name = entry.file_name();

                if path.is_file() {
                    let mut file = std::fs::File::open(&path)?;
                    builder.append_file(name, &mut file)?;
                } else if path.is_dir() {
                    builder.append_dir_all(&name, &path)?;
                }
            }

            builder.finish()?;
        }

        Ok(archive)
    }

    /// Creates a temporary directory with secrets written as files.
    ///
    /// Secrets are written to individual files for secure mounting into
    /// containers. This avoids exposing secrets through environment
    /// variables.
    ///
    /// Returns the path to the secrets directory.
    pub async fn create_secrets_dir(
        &self,
        secrets: &HashMap<String, String>,
    ) -> WorkerResult<std::path::PathBuf> {
        let secrets_dir = std::path::PathBuf::from(&self.config.workdir)
            .join(format!(".secrets-{}", Uuid::new_v4()));

        tokio::fs::create_dir_all(&secrets_dir).await?;

        // Write each secret to a separate file
        for (key, value) in secrets {
            let secret_path = secrets_dir.join(key);
            tokio::fs::write(&secret_path, value).await?;
            // Set restrictive permissions (owner read only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o400);
                tokio::fs::set_permissions(&secret_path, perms).await?;
            }
        }

        debug!(
            "Created secrets directory at {:?} with {} secrets",
            secrets_dir,
            secrets.len()
        );
        Ok(secrets_dir)
    }

    /// Removes a secrets directory and all its contents.
    pub async fn cleanup_secrets_dir(&self, secrets_dir: &std::path::Path) -> WorkerResult<()> {
        if secrets_dir.exists() {
            tokio::fs::remove_dir_all(secrets_dir).await?;
            debug!("Cleaned up secrets directory at {:?}", secrets_dir);
        }
        Ok(())
    }

    /// Creates and starts a container for agent execution.
    ///
    /// Secrets are mounted from files in a dedicated directory rather than
    /// passed as environment variables to prevent leakage through process
    /// listing.
    pub async fn create_container(
        &self,
        image: &str,
        container_name: &str,
        worktree_path: &str,
        repo_name: &str,
        secrets_dir: Option<&std::path::Path>,
    ) -> WorkerResult<String> {
        info!(
            "Creating container {} with image {} for {}",
            container_name, image, repo_name
        );

        // Build environment variables (non-sensitive only)
        let env: Vec<String> = vec![
            "HOME=/workspace".to_string(),
            "TERM=xterm-256color".to_string(),
            // Tell applications where to find secrets
            format!("SECRETS_DIR={}", SECRETS_MOUNT_PATH),
        ];

        // Build mounts
        let workspace_mount = format!("{}:/workspace/{}", worktree_path, repo_name);
        let mut binds = vec![workspace_mount];

        // Mount secrets directory as read-only if provided
        if let Some(secrets_path) = secrets_dir {
            let secrets_mount = format!(
                "{}:{}:ro",
                secrets_path.to_string_lossy(),
                SECRETS_MOUNT_PATH
            );
            binds.push(secrets_mount);
        }

        // Parse memory limit
        let memory = self.parse_memory_limit(&self.config.container_memory_limit);

        // Parse CPU limit (in CPUs, e.g., "0.5" or "2") into nano_cpus for Docker
        let nano_cpus = self.parse_cpu_limit(&self.config.container_cpu_limit);

        // Build host config - use bridge network instead of host for isolation
        let host_config = bollard::models::HostConfig {
            binds: Some(binds),
            // Use default bridge network for container isolation instead of host mode
            network_mode: None,
            memory,
            nano_cpus,
            ..Default::default()
        };

        let config = Config {
            image: Some(image.to_string()),
            env: Some(env),
            working_dir: Some(format!("/workspace/{}", repo_name)),
            host_config: Some(host_config),
            tty: Some(true),
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            open_stdin: Some(true),
            // Ensure the container stays alive regardless of the image's default command
            cmd: Some(vec![
                "/bin/sh".into(),
                "-c".into(),
                "tail -f /dev/null".into(),
            ]),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: container_name,
            platform: None,
        };

        let response = self.client.create_container(Some(options), config).await?;
        let container_id = response.id;

        info!("Created container: {}", container_id);

        // Start the container
        self.client
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await?;

        info!("Started container: {}", container_id);

        Ok(container_id)
    }

    /// Parses a CPU limit string (e.g., "0.5", "2") to nano_cpus.
    fn parse_cpu_limit(&self, limit: &str) -> Option<i64> {
        let limit = limit.trim();
        if limit.is_empty() {
            return None;
        }

        match limit.parse::<f64>() {
            Ok(cpus) if cpus > 0.0 => {
                // Docker expects CPU quota as nano_cpus (1 CPU = 1e9)
                Some((cpus * 1_000_000_000.0) as i64)
            }
            Ok(_) => {
                warn!(
                    "Configured container_cpu_limit must be positive, got {}",
                    limit
                );
                None
            }
            Err(e) => {
                warn!(
                    "Failed to parse container_cpu_limit '{}': {}. Ignoring CPU limit.",
                    limit, e
                );
                None
            }
        }
    }

    /// Stops and removes a container.
    pub async fn remove_container(&self, container_id: &str) -> WorkerResult<()> {
        info!("Stopping container: {}", container_id);

        // Try to stop the container first
        if let Err(e) = self
            .client
            .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
            .await
        {
            warn!("Failed to stop container {}: {}", container_id, e);
        }

        // Remove the container
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        self.client
            .remove_container(container_id, Some(options))
            .await?;

        info!("Removed container: {}", container_id);
        Ok(())
    }

    /// Executes a command in a running container.
    pub async fn exec_in_container(
        &self,
        container_id: &str,
        cmd: Vec<&str>,
        env: Option<Vec<String>>,
    ) -> WorkerResult<String> {
        use bollard::exec::{CreateExecOptions, StartExecResults};

        let env_refs: Option<Vec<&str>> =
            env.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());

        let options = CreateExecOptions {
            cmd: Some(cmd.clone()),
            env: env_refs,
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self.client.create_exec(container_id, options).await?;

        let output = match self.client.start_exec(&exec.id, None).await? {
            StartExecResults::Attached { mut output, .. } => {
                let mut result = String::new();
                while let Some(chunk) = output.next().await {
                    match chunk {
                        Ok(bollard::container::LogOutput::StdOut { message }) => {
                            result.push_str(&String::from_utf8_lossy(&message));
                        }
                        Ok(bollard::container::LogOutput::StdErr { message }) => {
                            result.push_str(&String::from_utf8_lossy(&message));
                        }
                        Ok(_) => {}
                        Err(e) => {
                            error!("Exec output error: {}", e);
                        }
                    }
                }
                result
            }
            StartExecResults::Detached => String::new(),
        };

        Ok(output)
    }

    /// Checks if a container is running.
    pub async fn is_container_running(&self, container_id: &str) -> bool {
        match self.client.inspect_container(container_id, None).await {
            Ok(info) => info.state.and_then(|s| s.running).unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Parses a memory limit string (e.g., "8g", "512m") to bytes.
    fn parse_memory_limit(&self, limit: &str) -> Option<i64> {
        let limit = limit.trim().to_lowercase();
        if limit.is_empty() {
            return None;
        }

        let (number, unit) = if limit.ends_with('g') {
            (limit.trim_end_matches('g'), 1024 * 1024 * 1024_i64)
        } else if limit.ends_with('m') {
            (limit.trim_end_matches('m'), 1024 * 1024_i64)
        } else if limit.ends_with('k') {
            (limit.trim_end_matches('k'), 1024_i64)
        } else {
            (limit.as_str(), 1_i64)
        };

        number.parse::<i64>().ok().map(|n| n * unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory_limit() {
        let config = WorkerConfig::default();
        let manager = DockerManager {
            client: Docker::connect_with_local_defaults().unwrap(),
            config,
        };

        assert_eq!(
            manager.parse_memory_limit("8g"),
            Some(8 * 1024 * 1024 * 1024)
        );
        assert_eq!(manager.parse_memory_limit("512m"), Some(512 * 1024 * 1024));
        assert_eq!(manager.parse_memory_limit("1024k"), Some(1024 * 1024));
        assert_eq!(manager.parse_memory_limit(""), None);
    }
}
