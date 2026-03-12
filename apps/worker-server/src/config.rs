//! Worker server configuration.

use serde::{Deserialize, Serialize};

/// Worker server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Main server URL to connect to.
    pub main_server_url: String,
    /// Worker name/identifier.
    pub worker_name: String,
    /// Worker port for callbacks.
    pub worker_port: u16,
    /// Docker socket path.
    pub docker_socket: String,
    /// Container memory limit.
    pub container_memory_limit: String,
    /// Container CPU limit (empty string means no limit).
    pub container_cpu_limit: String,
    /// Log level.
    pub log_level: String,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval_secs: u64,
    /// Default Docker image to use.
    pub default_docker_image: String,
    /// Working directory for worktrees.
    pub workdir: String,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            main_server_url: "http://localhost:54871".to_string(),
            worker_name: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "worker".to_string()),
            worker_port: 54872,
            docker_socket: "/var/run/docker.sock".to_string(),
            container_memory_limit: "8g".to_string(),
            container_cpu_limit: String::new(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            default_docker_image: "node:20-slim".to_string(),
            workdir: std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
                + "/.dexdex/worktrees",
        }
    }
}

impl WorkerConfig {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            main_server_url: std::env::var("DEXDEX_MAIN_SERVER_URL")
                .unwrap_or(default.main_server_url),
            worker_name: std::env::var("DEXDEX_WORKER_NAME").unwrap_or(default.worker_name),
            worker_port: std::env::var("DEXDEX_WORKER_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.worker_port),
            docker_socket: std::env::var("DEXDEX_DOCKER_SOCKET").unwrap_or(default.docker_socket),
            container_memory_limit: std::env::var("DEXDEX_CONTAINER_MEMORY_LIMIT")
                .unwrap_or(default.container_memory_limit),
            container_cpu_limit: std::env::var("DEXDEX_CONTAINER_CPU_LIMIT")
                .unwrap_or(default.container_cpu_limit),
            log_level: std::env::var("DEXDEX_LOG_LEVEL").unwrap_or(default.log_level),
            heartbeat_interval_secs: std::env::var("DEXDEX_HEARTBEAT_INTERVAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.heartbeat_interval_secs),
            default_docker_image: std::env::var("DEXDEX_DEFAULT_DOCKER_IMAGE")
                .unwrap_or(default.default_docker_image),
            workdir: std::env::var("DEXDEX_WORKDIR").unwrap_or(default.workdir),
        }
    }

    /// Returns the worker's callback endpoint URL.
    pub fn callback_url(&self) -> String {
        format!("http://{}:{}", self.worker_name, self.worker_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorkerConfig::default();
        assert_eq!(config.main_server_url, "http://localhost:54871");
        assert_eq!(config.worker_port, 54872);
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.default_docker_image, "node:20-slim");
    }
}
