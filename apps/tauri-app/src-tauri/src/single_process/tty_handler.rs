//! Local TTY input handler for agent interactions.
//!
//! This module provides a TTY input handler that works with the Tauri frontend
//! to handle interactive prompts from AI coding agents.

use std::sync::Arc;

use async_trait::async_trait;
use coding_agents::{AgentError, AgentResult, TtyInputHandler};
use tauri::{AppHandle, Emitter};
use tokio::sync::{RwLock, oneshot};
use tracing::{debug, info};
use uuid::Uuid;

use crate::events::{TtyInputRequestEvent, event_names};

/// A pending TTY input request waiting for a response.
struct PendingRequest {
    /// Channel to send the response back to the agent.
    response_tx: oneshot::Sender<String>,
}

/// Manager for pending TTY input requests.
///
/// This struct tracks all pending requests and allows responses to be
/// delivered to the correct agent.
pub struct TtyInputRequestManager {
    /// Pending requests keyed by request ID.
    pending: RwLock<std::collections::HashMap<Uuid, PendingRequest>>,
}

impl TtyInputRequestManager {
    /// Creates a new TTY input request manager.
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Registers a new pending request and returns a receiver for the response.
    async fn register(&self, request_id: Uuid) -> oneshot::Receiver<String> {
        let (tx, rx) = oneshot::channel();
        let mut pending = self.pending.write().await;
        pending.insert(request_id, PendingRequest { response_tx: tx });
        rx
    }

    /// Responds to a pending request.
    ///
    /// Returns `true` if the response was delivered, `false` if the request
    /// was not found.
    pub async fn respond(&self, request_id: Uuid, response: String) -> bool {
        let mut pending = self.pending.write().await;
        if let Some(request) = pending.remove(&request_id) {
            if request.response_tx.send(response).is_ok() {
                info!("Delivered TTY response for request {}", request_id);
                return true;
            }
        }
        false
    }

    /// Cancels a pending request (e.g., on timeout or task cancellation).
    pub async fn cancel(&self, request_id: Uuid) {
        let mut pending = self.pending.write().await;
        pending.remove(&request_id);
    }
}

impl Default for TtyInputRequestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Local TTY input handler that uses Tauri events to communicate with the
/// frontend.
pub struct LocalTtyHandler {
    /// Tauri app handle for emitting events.
    app_handle: AppHandle,
    /// The task ID this handler is associated with.
    task_id: Uuid,
    /// The session ID this handler is associated with.
    session_id: Uuid,
    /// Shared request manager.
    request_manager: Arc<TtyInputRequestManager>,
}

impl LocalTtyHandler {
    /// Creates a new local TTY handler.
    pub fn new(
        app_handle: AppHandle,
        task_id: Uuid,
        session_id: Uuid,
        request_manager: Arc<TtyInputRequestManager>,
    ) -> Self {
        Self {
            app_handle,
            task_id,
            session_id,
            request_manager,
        }
    }
}

#[async_trait]
impl TtyInputHandler for LocalTtyHandler {
    async fn handle_input(
        &self,
        question: &str,
        options: Option<&[String]>,
    ) -> AgentResult<String> {
        let request_id = Uuid::new_v4();

        info!(
            "TTY input requested for task {}, session {}: {}",
            self.task_id, self.session_id, question
        );

        // Register the pending request before emitting the event
        let response_rx = self.request_manager.register(request_id).await;

        // Emit the TTY input request event to the frontend
        let event = TtyInputRequestEvent {
            request_id: request_id.to_string(),
            task_id: self.task_id.to_string(),
            session_id: self.session_id.to_string(),
            question: question.to_string(),
            options: options.map(|opts| opts.to_vec()),
        };

        self.app_handle
            .emit(event_names::TTY_INPUT_REQUEST, &event)
            .map_err(|e| AgentError::TtyInputRequired(format!("Failed to emit event: {}", e)))?;

        debug!("Waiting for TTY response for request {}", request_id);

        // Wait for the response
        match response_rx.await {
            Ok(response) => {
                debug!("Received TTY response: {}", response);
                Ok(response)
            }
            Err(_) => {
                // Channel was dropped (e.g., request was cancelled)
                Err(AgentError::TtyInputRequired(
                    "TTY input request was cancelled".to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_request_manager_respond() {
        let manager = TtyInputRequestManager::new();
        let request_id = Uuid::new_v4();

        // Register a pending request
        let rx = manager.register(request_id).await;

        // Respond in a separate task
        let response_manager = Arc::new(manager);
        let rm = response_manager.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            rm.respond(request_id, "yes".to_string()).await
        });

        // Wait for the response
        let response = rx.await.unwrap();
        assert_eq!(response, "yes");

        let delivered = handle.await.unwrap();
        assert!(delivered);
    }

    #[tokio::test]
    async fn test_request_manager_cancel() {
        let manager = TtyInputRequestManager::new();
        let request_id = Uuid::new_v4();

        // Register a pending request
        let rx = manager.register(request_id).await;

        // Cancel it
        manager.cancel(request_id).await;

        // The receiver should return an error
        assert!(rx.await.is_err());
    }

    #[tokio::test]
    async fn test_respond_to_unknown_request() {
        let manager = TtyInputRequestManager::new();
        let unknown_id = Uuid::new_v4();

        // Responding to an unknown request should return false
        let result = manager.respond(unknown_id, "response".to_string()).await;
        assert!(!result);
    }
}
