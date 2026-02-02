//! Agent session API endpoints.

use std::sync::Arc;

use axum::{Json, extract::State};
use rpc_protocol::{requests::*, responses::*};
use task_store::TaskStore;
use uuid::Uuid;

use crate::{
    error::{ServerError, ServerResult},
    state::AppState,
};

/// Gets the output log for an agent session.
pub async fn get_log<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<GetLogRequest>,
) -> ServerResult<Json<GetLogResponse>> {
    let session_id: Uuid = request
        .session_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid session_id".to_string()))?;

    let session = state
        .store
        .get_agent_session(session_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Session not found".to_string()))?;

    Ok(Json(GetLogResponse {
        log: session.output_log.unwrap_or_default(),
    }))
}

/// Stops a running agent session.
pub async fn stop_session<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<StopSessionRequest>,
) -> ServerResult<Json<StopSessionResponse>> {
    let session_id: Uuid = request
        .session_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid session_id".to_string()))?;

    let mut session = state
        .store
        .get_agent_session(session_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("Session not found".to_string()))?;

    // Mark session as completed
    session.completed_at = Some(chrono::Utc::now());
    state.store.update_agent_session(session).await?;

    tracing::info!(session_id = %session_id, "Session stopped");

    // TODO: Actually stop the running agent process via worker

    Ok(Json(StopSessionResponse {}))
}

/// Submits a response to a TTY input request.
pub async fn submit_tty_input<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<SubmitTtyInputRequest>,
) -> ServerResult<Json<SubmitTtyInputResponse>> {
    let request_id: Uuid = request
        .request_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid request_id".to_string()))?;

    let mut tty_request = state
        .store
        .get_tty_input_request(request_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("TTY input request not found".to_string()))?;

    // Update the request with the response
    let response_text = request.response.clone();
    tty_request.response = Some(request.response);
    tty_request.status = entities::TtyInputStatus::Responded;
    tty_request.responded_at = Some(chrono::Utc::now());
    state.store.update_tty_input_request(tty_request).await?;

    // Relay response to worker via the TTY response relay
    {
        let mut relay = state.tty_response_relay.write().await;
        let delivered = relay.deliver(request_id, response_text);
        tracing::info!(
            request_id = %request_id,
            delivered = delivered,
            "TTY input submitted and relayed"
        );
    }

    Ok(Json(SubmitTtyInputResponse {}))
}

/// Waits for a TTY input response (called by worker).
/// This endpoint allows workers to poll for TTY responses.
pub async fn wait_tty_response<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<WaitTtyResponseRequest>,
) -> ServerResult<Json<WaitTtyResponseResponse>> {
    let request_id: Uuid = request
        .request_id
        .parse()
        .map_err(|_| ServerError::InvalidRequest("Invalid request_id".to_string()))?;

    // Register for the response
    let receiver = {
        let mut relay = state.tty_response_relay.write().await;
        relay.register(request_id)
    };

    // Wait for the response with timeout
    let timeout_ms = request.timeout_ms.unwrap_or(300_000); // Default 5 minutes
    let timeout_duration = std::time::Duration::from_millis(timeout_ms);

    match tokio::time::timeout(timeout_duration, receiver).await {
        Ok(Ok(response)) => {
            tracing::info!(request_id = %request_id, "TTY response received by worker");
            Ok(Json(WaitTtyResponseResponse {
                response: Some(response),
                timed_out: false,
            }))
        }
        Ok(Err(_)) => {
            // Channel closed (response relay cancelled)
            tracing::warn!(request_id = %request_id, "TTY response channel closed");
            Ok(Json(WaitTtyResponseResponse {
                response: None,
                timed_out: false,
            }))
        }
        Err(_) => {
            // Timeout
            tracing::info!(request_id = %request_id, "TTY response wait timed out");

            // Update the TTY request status to timeout
            if let Ok(Some(mut tty_request)) = state.store.get_tty_input_request(request_id).await {
                tty_request.status = entities::TtyInputStatus::Timeout;
                let _ = state.store.update_tty_input_request(tty_request).await;
            }

            // Clean up the relay
            {
                let mut relay = state.tty_response_relay.write().await;
                relay.cancel(&request_id);
            }

            Ok(Json(WaitTtyResponseResponse {
                response: None,
                timed_out: true,
            }))
        }
    }
}

/// Request to wait for a TTY response.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WaitTtyResponseRequest {
    /// The TTY input request ID.
    pub request_id: String,
    /// Timeout in milliseconds (default: 5 minutes).
    pub timeout_ms: Option<u64>,
}

/// Response to a TTY response wait.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WaitTtyResponseResponse {
    /// The user's response, if received.
    pub response: Option<String>,
    /// Whether the wait timed out.
    pub timed_out: bool,
}
