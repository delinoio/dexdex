//! HTTP client for communicating with the main server's WorkerService
//! endpoints.
//!
//! Uses Connect RPC convention: POST `{base_url}/{ServiceName}/{MethodName}`
//! with JSON body.

use entities::{GeneratedCommit, SessionOutputEvent, SubTask, SubTaskStatus, UnitTask};
use rpc_protocol::{
    WorkerStatus,
    requests::{
        EmitSessionEventRequest, GetNextSubTaskRequest, HeartbeatRequest, RegisterWorkerRequest,
        ReportSubTaskStatusRequest, UnregisterWorkerRequest,
    },
    responses::{GetNextSubTaskResponse, RegisterWorkerResponse},
};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{WorkerError, WorkerResult};

/// HTTP client for the main server's WorkerService.
pub struct MainServerClient {
    base_url: String,
    client: reqwest::Client,
    worker_id: Uuid,
}

impl MainServerClient {
    /// Registers this worker with the main server and returns a connected
    /// client.
    pub async fn register(base_url: &str, name: &str) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();
        let url = format!("{}/WorkerService/Register", base_url);

        let body = RegisterWorkerRequest {
            name: name.to_string(),
            // The worker is a polling client, no endpoint URL needed.
            endpoint_url: String::new(),
        };

        debug!("Registering worker at {}", url);

        let response = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to reach main server: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Registration failed with status {}: {}", status, text);
        }

        let reg: RegisterWorkerResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse registration response: {}", e))?;

        info!("Registered with main server, worker_id={}", reg.worker_id);

        Ok(Self {
            base_url: base_url.to_string(),
            client,
            worker_id: reg.worker_id,
        })
    }

    /// Returns the worker ID assigned by the main server.
    pub fn worker_id(&self) -> Uuid {
        self.worker_id
    }

    /// Sends a heartbeat to the main server.
    pub async fn heartbeat(
        &self,
        status: WorkerStatus,
        current_sub_task_id: Option<Uuid>,
    ) -> WorkerResult<()> {
        let url = format!("{}/WorkerService/Heartbeat", self.base_url);

        let body = HeartbeatRequest {
            worker_id: self.worker_id,
            status,
            current_sub_task_id,
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(WorkerError::Http)?;

        if !response.status().is_success() {
            warn!(
                "Heartbeat failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        } else {
            debug!("Heartbeat sent successfully (worker_id={})", self.worker_id);
        }

        Ok(())
    }

    /// Polls the main server for the next available subtask.
    ///
    /// Returns `Some((sub_task, unit_task))` if a subtask is available, `None`
    /// otherwise.
    pub async fn get_next_sub_task(&self) -> WorkerResult<Option<(SubTask, UnitTask)>> {
        let url = format!("{}/WorkerService/GetNextSubTask", self.base_url);

        let body = GetNextSubTaskRequest {
            worker_id: self.worker_id,
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(WorkerError::Http)?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WorkerError::TaskExecution(format!(
                "GetNextSubTask failed with status {}: {}",
                status, text
            )));
        }

        let resp: GetNextSubTaskResponse = response.json().await.map_err(WorkerError::Http)?;

        match (resp.sub_task, resp.unit_task) {
            (Some(sub_task), Some(unit_task)) => {
                debug!("Received subtask sub_task_id={}", sub_task.id);
                Ok(Some((sub_task, unit_task)))
            }
            _ => Ok(None),
        }
    }

    /// Reports the completion status of a subtask to the main server.
    pub async fn report_sub_task_status(
        &self,
        sub_task_id: Uuid,
        status: SubTaskStatus,
        commits: Vec<GeneratedCommit>,
        error: Option<String>,
    ) -> WorkerResult<()> {
        let url = format!("{}/WorkerService/ReportSubTaskStatus", self.base_url);

        let body = ReportSubTaskStatusRequest {
            worker_id: self.worker_id,
            sub_task_id,
            status,
            generated_commits: commits,
            error,
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(WorkerError::Http)?;

        if !response.status().is_success() {
            let status_code = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WorkerError::TaskExecution(format!(
                "ReportSubTaskStatus failed with status {}: {}",
                status_code, text
            )));
        }

        debug!(
            "Reported subtask status sub_task_id={} status={:?}",
            sub_task_id, status
        );

        Ok(())
    }

    /// Emits a session output event to the main server.
    pub async fn emit_session_event(&self, event: SessionOutputEvent) -> WorkerResult<()> {
        let url = format!("{}/WorkerService/EmitSessionEvent", self.base_url);

        let body = EmitSessionEventRequest {
            worker_id: self.worker_id,
            event,
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(WorkerError::Http)?;

        if !response.status().is_success() {
            warn!(
                "EmitSessionEvent failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        Ok(())
    }

    /// Unregisters this worker from the main server.
    pub async fn unregister(&self) -> WorkerResult<()> {
        let url = format!("{}/WorkerService/Unregister", self.base_url);

        let body = UnregisterWorkerRequest {
            worker_id: self.worker_id,
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(WorkerError::Http)?;

        if !response.status().is_success() {
            warn!(
                "Unregister failed with status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        } else {
            info!(
                "Unregistered from main server (worker_id={})",
                self.worker_id
            );
        }

        Ok(())
    }
}
