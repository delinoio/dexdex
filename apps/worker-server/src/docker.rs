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

use crate::{
    config::WorkerConfig,
    error::{WorkerError, WorkerResult},
};

/// Docker manager for container lifecycle management.
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

    /// Creates and starts a container for agent execution.
    pub async fn create_container(
        &self,
        image: &str,
        container_name: &str,
        worktree_path: &str,
        repo_name: &str,
        env_vars: HashMap<String, String>,
    ) -> WorkerResult<String> {
        info!(
            "Creating container {} with image {} for {}",
            container_name, image, repo_name
        );

        // Build environment variables
        let mut env: Vec<String> = vec![
            "HOME=/workspace".to_string(),
            "TERM=xterm-256color".to_string(),
        ];

        for (key, value) in env_vars {
            env.push(format!("{}={}", key, value));
        }

        // Build mounts
        let workspace_mount = format!("{}:/workspace/{}", worktree_path, repo_name);
        let binds = vec![workspace_mount];

        // Parse memory limit
        let memory = self.parse_memory_limit(&self.config.container_memory_limit);

        // Build host config
        let host_config = bollard::models::HostConfig {
            binds: Some(binds),
            network_mode: Some("host".to_string()),
            memory,
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
