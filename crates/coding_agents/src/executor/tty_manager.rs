//! TTY input request manager for handling interactive prompts.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::{RwLock, oneshot};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{EventEmitter, TtyInputRequestEvent};
use crate::{AgentError, AgentResult, TtyInputHandler};

/// Default timeout for TTY input requests (5 minutes).
const DEFAULT_TTY_TIMEOUT: Duration = Duration::from_secs(300);

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
    pending: RwLock<HashMap<Uuid, PendingRequest>>,
}

impl TtyInputRequestManager {
    /// Creates a new TTY input request manager.
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a new pending request and returns a receiver for the response.
    pub async fn register(&self, request_id: Uuid) -> oneshot::Receiver<String> {
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
        if let Some(request) = pending.remove(&request_id)
            && request.response_tx.send(response).is_ok()
        {
            info!("Delivered TTY response for request {}", request_id);
            return true;
        }
        false
    }

    /// Cancels a pending request (e.g., on timeout or task cancellation).
    pub async fn cancel(&self, request_id: Uuid) {
        let mut pending = self.pending.write().await;
        pending.remove(&request_id);
    }

    /// Returns the number of pending requests.
    pub async fn pending_count(&self) -> usize {
        self.pending.read().await.len()
    }
}

impl Default for TtyInputRequestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// TTY input handler that uses an event emitter to communicate with the
/// frontend.
///
/// This handler emits TTY input request events via the provided event emitter
/// and waits for responses to be delivered via the request manager.
pub struct EventEmitterTtyHandler<E: EventEmitter> {
    /// Event emitter for sending TTY input request events.
    emitter: Arc<E>,
    /// The task ID this handler is associated with.
    task_id: Uuid,
    /// The session ID this handler is associated with.
    session_id: Uuid,
    /// Shared request manager.
    request_manager: Arc<TtyInputRequestManager>,
}

impl<E: EventEmitter> EventEmitterTtyHandler<E> {
    /// Creates a new event emitter TTY handler.
    pub fn new(
        emitter: Arc<E>,
        task_id: Uuid,
        session_id: Uuid,
        request_manager: Arc<TtyInputRequestManager>,
    ) -> Self {
        Self {
            emitter,
            task_id,
            session_id,
            request_manager,
        }
    }
}

#[async_trait]
impl<E: EventEmitter + 'static> TtyInputHandler for EventEmitterTtyHandler<E> {
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

        // Emit the TTY input request event
        let event = TtyInputRequestEvent {
            request_id: request_id.to_string(),
            task_id: self.task_id.to_string(),
            session_id: self.session_id.to_string(),
            question: question.to_string(),
            options: options.map(|opts| opts.to_vec()),
        };

        self.emitter.emit_tty_input_request(event)?;

        debug!("Waiting for TTY response for request {}", request_id);

        // Wait for the response with timeout
        match tokio::time::timeout(DEFAULT_TTY_TIMEOUT, response_rx).await {
            Ok(Ok(response)) => {
                debug!("Received TTY response: {}", response);
                Ok(response)
            }
            Ok(Err(_)) => {
                // Channel was dropped (e.g., request was cancelled)
                Err(AgentError::TtyInputRequired(
                    "TTY input request was cancelled".to_string(),
                ))
            }
            Err(_) => {
                // Timeout elapsed - clean up the pending request
                warn!(
                    "TTY input request {} timed out after {:?}",
                    request_id, DEFAULT_TTY_TIMEOUT
                );
                self.request_manager.cancel(request_id).await;
                Err(AgentError::TtyInputRequired(format!(
                    "TTY input request timed out after {} seconds",
                    DEFAULT_TTY_TIMEOUT.as_secs()
                )))
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
        let manager = Arc::new(manager);
        let rm = manager.clone();
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

    #[tokio::test]
    async fn test_pending_count() {
        let manager = TtyInputRequestManager::new();
        assert_eq!(manager.pending_count().await, 0);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let _rx1 = manager.register(id1).await;
        assert_eq!(manager.pending_count().await, 1);

        let _rx2 = manager.register(id2).await;
        assert_eq!(manager.pending_count().await, 2);

        manager.cancel(id1).await;
        assert_eq!(manager.pending_count().await, 1);
    }
}
