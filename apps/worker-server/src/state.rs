//! Worker server application state.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock, mpsc};
use uuid::Uuid;

use crate::{config::WorkerConfig, docker::DockerManager};

/// Worker status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerStatus {
    /// Worker is idle and ready to accept tasks.
    Idle,
    /// Worker is currently executing a task.
    Busy,
    /// Worker is shutting down.
    ShuttingDown,
}

impl WorkerStatus {
    /// Returns the status as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerStatus::Idle => "idle",
            WorkerStatus::Busy => "busy",
            WorkerStatus::ShuttingDown => "shutting_down",
        }
    }
}

/// Information about a running task.
#[derive(Debug, Clone)]
pub struct RunningTask {
    /// Task ID.
    pub task_id: Uuid,
    /// Session ID.
    pub session_id: Uuid,
    /// Container ID.
    pub container_id: Option<String>,
    /// Worktree path.
    pub worktree_path: String,
    /// Output log buffer.
    pub output: Arc<RwLock<String>>,
    /// Cancellation sender.
    pub cancel_tx: mpsc::Sender<()>,
}

/// Worker server application state.
pub struct AppState {
    /// Worker configuration.
    pub config: WorkerConfig,
    /// Docker manager.
    pub docker: DockerManager,
    /// Worker ID (assigned by main server).
    pub worker_id: RwLock<Option<Uuid>>,
    /// Current worker status.
    pub status: RwLock<WorkerStatus>,
    /// Currently running task.
    pub current_task: RwLock<Option<RunningTask>>,
    /// Pending TTY input responses.
    pub tty_responses: Mutex<HashMap<Uuid, mpsc::Sender<String>>>,
    /// HTTP client for main server communication.
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Creates a new application state.
    pub async fn new(config: WorkerConfig) -> Result<Arc<Self>, crate::error::WorkerError> {
        let docker = DockerManager::new(&config).await?;

        Ok(Arc::new(Self {
            config,
            docker,
            worker_id: RwLock::new(None),
            status: RwLock::new(WorkerStatus::Idle),
            current_task: RwLock::new(None),
            tty_responses: Mutex::new(HashMap::new()),
            http_client: reqwest::Client::new(),
        }))
    }

    /// Sets the worker ID.
    pub async fn set_worker_id(&self, id: Uuid) {
        let mut worker_id = self.worker_id.write().await;
        *worker_id = Some(id);
    }

    /// Gets the worker ID.
    pub async fn get_worker_id(&self) -> Option<Uuid> {
        *self.worker_id.read().await
    }

    /// Sets the worker status.
    pub async fn set_status(&self, status: WorkerStatus) {
        let mut current = self.status.write().await;
        *current = status;
    }

    /// Gets the current worker status.
    pub async fn get_status(&self) -> WorkerStatus {
        *self.status.read().await
    }

    /// Sets the current running task.
    pub async fn set_current_task(&self, task: Option<RunningTask>) {
        let mut current = self.current_task.write().await;
        *current = task;
    }

    /// Gets the current running task ID.
    pub async fn get_current_task_id(&self) -> Option<Uuid> {
        self.current_task.read().await.as_ref().map(|t| t.task_id)
    }

    /// Adds output to the current task log.
    pub async fn append_task_output(&self, output: &str) {
        if let Some(task) = self.current_task.read().await.as_ref() {
            let mut log = task.output.write().await;
            log.push_str(output);
            log.push('\n');
        }
    }

    /// Gets the current task output log.
    pub async fn get_task_output(&self) -> Option<String> {
        if let Some(task) = self.current_task.read().await.as_ref() {
            Some(task.output.read().await.clone())
        } else {
            None
        }
    }

    /// Registers a TTY response channel.
    pub async fn register_tty_response(&self, request_id: Uuid, tx: mpsc::Sender<String>) {
        let mut responses = self.tty_responses.lock().await;
        responses.insert(request_id, tx);
    }

    /// Submits a TTY response.
    pub async fn submit_tty_response(&self, request_id: Uuid, response: String) -> bool {
        let mut responses = self.tty_responses.lock().await;
        if let Some(tx) = responses.remove(&request_id) {
            tx.send(response).await.is_ok()
        } else {
            false
        }
    }

    /// Cancels the current task.
    pub async fn cancel_current_task(&self) -> bool {
        if let Some(task) = self.current_task.read().await.as_ref() {
            task.cancel_tx.send(()).await.is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_status() {
        assert_eq!(WorkerStatus::Idle.as_str(), "idle");
        assert_eq!(WorkerStatus::Busy.as_str(), "busy");
        assert_eq!(WorkerStatus::ShuttingDown.as_str(), "shutting_down");
    }
}
