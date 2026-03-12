//! Main execution logic for the worker server.
//!
//! This module implements a stub executor that simulates agent work by emitting
//! progress events and reporting completion. Actual AI agent integration is
//! out of scope for the initial rewrite.

use std::sync::Arc;

use entities::{SessionOutputEvent, SessionOutputKind, SubTask, SubTaskStatus, UnitTask};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{client::MainServerClient, config::WorkerConfig};

/// Main executor that polls for subtasks and executes them.
pub struct Executor {
    config: WorkerConfig,
    client: Arc<MainServerClient>,
}

impl Executor {
    /// Creates a new executor.
    pub fn new(config: WorkerConfig, client: Arc<MainServerClient>) -> Self {
        Self { config, client }
    }

    /// Runs the main polling loop until shutdown.
    ///
    /// 1. Polls `GetNextSubTask` from the main server.
    /// 2. If a subtask is available, executes it.
    /// 3. Sleeps for `poll_interval_ms` before polling again.
    pub async fn run(&self) {
        let poll_interval = std::time::Duration::from_millis(self.config.poll_interval_ms);

        info!(
            "Executor started, polling every {}ms",
            self.config.poll_interval_ms
        );

        // Heartbeat interval: every 30 seconds.
        let heartbeat_interval = std::time::Duration::from_secs(30);
        let mut last_heartbeat = std::time::Instant::now();

        loop {
            // Send heartbeat if due.
            if last_heartbeat.elapsed() >= heartbeat_interval {
                if let Err(e) = self
                    .client
                    .heartbeat(rpc_protocol::WorkerStatus::Idle, None)
                    .await
                {
                    warn!("Heartbeat failed: {}", e);
                }
                last_heartbeat = std::time::Instant::now();
            }

            // Poll for the next subtask.
            match self.client.get_next_sub_task().await {
                Ok(Some((sub_task, unit_task))) => {
                    info!(
                        "Picked up subtask sub_task_id={} unit_task_id={} type={:?}",
                        sub_task.id, sub_task.unit_task_id, sub_task.task_type
                    );

                    // Send heartbeat to indicate we're busy.
                    if let Err(e) = self
                        .client
                        .heartbeat(rpc_protocol::WorkerStatus::Busy, Some(sub_task.id))
                        .await
                    {
                        warn!("Heartbeat (busy) failed: {}", e);
                    }

                    self.execute_sub_task(sub_task, unit_task).await;
                }
                Ok(None) => {
                    // No work available; wait before polling again.
                }
                Err(e) => {
                    error!("Failed to poll for subtask: {}", e);
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Executes a single subtask.
    ///
    /// Stub implementation:
    /// 1. Creates an agent session (emits events).
    /// 2. Emits a few progress `SessionOutputEvent`s.
    /// 3. Waits briefly to simulate work.
    /// 4. Reports `Completed` status to the main server.
    async fn execute_sub_task(&self, sub_task: SubTask, _unit_task: UnitTask) {
        let session_id = Uuid::new_v4();
        let sub_task_id = sub_task.id;

        info!(
            "Starting stub execution: sub_task_id={} session_id={}",
            sub_task_id, session_id
        );

        // Emit session start event.
        self.emit_event(
            session_id,
            1,
            SessionOutputKind::Progress,
            format!("Starting {:?} subtask (stub executor)", sub_task.task_type),
        )
        .await;

        // Simulate work with a short delay.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Emit progress event.
        self.emit_event(
            session_id,
            2,
            SessionOutputKind::Progress,
            "Processing subtask...".to_string(),
        )
        .await;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Emit completion event.
        self.emit_event(
            session_id,
            3,
            SessionOutputKind::Text,
            "Subtask completed successfully (stub executor)".to_string(),
        )
        .await;

        // Report completed status.
        if let Err(e) = self
            .client
            .report_sub_task_status(sub_task_id, SubTaskStatus::Completed, vec![], None)
            .await
        {
            error!(
                "Failed to report completed status for sub_task_id={}: {}",
                sub_task_id, e
            );
        } else {
            info!("Reported Completed for sub_task_id={}", sub_task_id);
        }
    }

    /// Emits a single session output event to the main server.
    async fn emit_event(
        &self,
        session_id: Uuid,
        sequence: u64,
        kind: SessionOutputKind,
        message: String,
    ) {
        let event = SessionOutputEvent::new(session_id, sequence, kind, message);
        if let Err(e) = self.client.emit_session_event(event).await {
            warn!("Failed to emit session event: {}", e);
        }
    }
}
