//! Main server client for worker communication.

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    error::{WorkerError, WorkerResult},
    state::AppState,
};

/// Task assignment from main server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// Task ID.
    pub task_id: Uuid,
    /// Session ID.
    pub session_id: Uuid,
    /// Task prompt.
    pub prompt: String,
    /// Repository URL.
    pub repository_url: String,
    /// Branch name.
    pub branch_name: String,
    /// Agent type.
    pub agent_type: String,
    /// Optional agent model.
    pub agent_model: Option<String>,
}

/// Worker registration response.
#[derive(Debug, Deserialize)]
pub struct RegistrationResponse {
    /// Assigned worker ID.
    pub worker_id: Uuid,
}

/// Task status update.
#[derive(Debug, Serialize)]
pub struct TaskStatusUpdate {
    /// Task ID.
    pub task_id: Uuid,
    /// Session ID.
    pub session_id: Uuid,
    /// Status (running, completed, failed).
    pub status: String,
    /// Output log.
    pub output: Option<String>,
    /// Error message.
    pub error: Option<String>,
    /// End commit hash.
    pub end_commit: Option<String>,
    /// Git patch (unified diff) representing changes made by the AI agent.
    pub git_patch: Option<String>,
}

/// Secrets response from main server.
#[derive(Debug, Deserialize)]
pub struct SecretsResponse {
    /// Secrets as key-value pairs.
    pub secrets: HashMap<String, String>,
}

/// TTY input request to main server.
#[derive(Debug, Serialize)]
pub struct TtyInputRequest {
    /// Request ID.
    pub request_id: Uuid,
    /// Task ID.
    pub task_id: Uuid,
    /// Session ID.
    pub session_id: Uuid,
    /// Question being asked.
    pub question: String,
    /// Available options.
    pub options: Option<Vec<String>>,
}

/// Main server client.
pub struct MainServerClient {
    state: Arc<AppState>,
}

impl MainServerClient {
    /// Creates a new main server client.
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Registers this worker with the main server.
    pub async fn register(&self) -> WorkerResult<Uuid> {
        let url = format!("{}/api/worker/register", self.state.config.main_server_url);

        let body = serde_json::json!({
            "name": self.state.config.worker_name,
            "endpoint_url": self.state.config.callback_url(),
        });

        debug!("Registering worker at {}", url);

        let response = self
            .state
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| WorkerError::Registration(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WorkerError::Registration(format!(
                "Registration failed with status {}: {}",
                status, text
            )));
        }

        let reg_response: RegistrationResponse = response
            .json()
            .await
            .map_err(|e| WorkerError::Registration(e.to_string()))?;

        info!("Registered with worker ID: {}", reg_response.worker_id);
        Ok(reg_response.worker_id)
    }

    /// Sends a heartbeat to the main server.
    pub async fn heartbeat(&self) -> WorkerResult<()> {
        let worker_id = self
            .state
            .get_worker_id()
            .await
            .ok_or_else(|| WorkerError::Registration("Worker not registered".to_string()))?;

        let url = format!("{}/api/worker/heartbeat", self.state.config.main_server_url);

        let status = self.state.get_status().await;
        let current_task_id = self.state.get_current_task_id().await;

        let body = serde_json::json!({
            "worker_id": worker_id,
            "status": status.as_str(),
            "current_task_id": current_task_id,
        });

        let response = self.state.http_client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            warn!(
                "Heartbeat failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        } else {
            debug!("Heartbeat sent successfully");
        }

        Ok(())
    }

    /// Unregisters this worker from the main server.
    pub async fn unregister(&self) -> WorkerResult<()> {
        let worker_id = match self.state.get_worker_id().await {
            Some(id) => id,
            None => return Ok(()), // Not registered
        };

        let url = format!(
            "{}/api/worker/unregister",
            self.state.config.main_server_url
        );

        let body = serde_json::json!({
            "worker_id": worker_id,
        });

        let response = self.state.http_client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            warn!(
                "Unregister failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        } else {
            info!("Unregistered from main server");
        }

        Ok(())
    }

    /// Gets the next task from the main server.
    pub async fn get_task(&self) -> WorkerResult<Option<TaskAssignment>> {
        let worker_id = self
            .state
            .get_worker_id()
            .await
            .ok_or_else(|| WorkerError::Registration("Worker not registered".to_string()))?;

        let url = format!(
            "{}/api/worker/get-task?worker_id={}",
            self.state.config.main_server_url, worker_id
        );

        let response = self.state.http_client.get(&url).send().await?;

        if response.status().as_u16() == 204 {
            // No task available
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WorkerError::TaskExecution(format!(
                "Failed to get task: {} - {}",
                status, text
            )));
        }

        let task: TaskAssignment = response.json().await?;
        debug!("Received task: {:?}", task.task_id);
        Ok(Some(task))
    }

    /// Reports task status to the main server.
    pub async fn report_status(&self, update: TaskStatusUpdate) -> WorkerResult<()> {
        let worker_id = self
            .state
            .get_worker_id()
            .await
            .ok_or_else(|| WorkerError::Registration("Worker not registered".to_string()))?;

        let url = format!(
            "{}/api/worker/report-status",
            self.state.config.main_server_url
        );

        let body = serde_json::json!({
            "worker_id": worker_id,
            "task_id": update.task_id,
            "session_id": update.session_id,
            "status": update.status,
            "output": update.output,
            "error": update.error,
            "end_commit": update.end_commit,
            "git_patch": update.git_patch,
        });

        let response = self.state.http_client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            error!(
                "Failed to report status: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        } else {
            debug!("Status reported successfully for task {}", update.task_id);
        }

        Ok(())
    }

    /// Gets secrets for a task from the main server.
    pub async fn get_secrets(&self, task_id: Uuid) -> WorkerResult<HashMap<String, String>> {
        let worker_id = self
            .state
            .get_worker_id()
            .await
            .ok_or_else(|| WorkerError::Registration("Worker not registered".to_string()))?;

        let url = format!(
            "{}/api/worker/get-secrets?worker_id={}&task_id={}",
            self.state.config.main_server_url, worker_id, task_id
        );

        let response = self.state.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WorkerError::TaskExecution(format!(
                "Failed to get secrets: {} - {}",
                status, text
            )));
        }

        let secrets_response: SecretsResponse = response.json().await?;
        debug!("Received {} secrets", secrets_response.secrets.len());
        Ok(secrets_response.secrets)
    }

    /// Creates a TTY input request on the main server.
    pub async fn create_tty_input_request(&self, request: TtyInputRequest) -> WorkerResult<()> {
        let url = format!(
            "{}/api/session/tty-input-request",
            self.state.config.main_server_url
        );

        let response = self
            .state
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            error!(
                "Failed to create TTY input request: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        } else {
            debug!("TTY input request created: {}", request.request_id);
        }

        Ok(())
    }
}
