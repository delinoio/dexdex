//! RPC type definitions.
//!
//! Re-exports all entity types and defines protocol-specific types.

use chrono::{DateTime, Utc};
// Re-export everything from entities so consumers only need rpc_protocol.
pub use entities::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A server-sent event wrapper for streaming updates to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamEvent {
    /// The type of event being streamed.
    pub event_type: StreamEventType,
    /// The workspace this event is scoped to.
    pub workspace_id: String,
    /// The event payload as a JSON value.
    pub payload: serde_json::Value,
}

/// Status of a registered worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    /// Worker is available to accept tasks.
    Idle,
    /// Worker is currently executing a task.
    Busy,
    /// Worker is not responding to health checks.
    Unhealthy,
}

/// A worker instance registered with the main server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Worker {
    /// Unique identifier.
    pub id: Uuid,
    /// Human-readable worker name.
    pub name: String,
    /// The endpoint URL where this worker can be reached.
    pub endpoint_url: String,
    /// Current status of this worker.
    pub status: WorkerStatus,
    /// When this worker last sent a heartbeat.
    pub last_heartbeat: DateTime<Utc>,
    /// The subtask this worker is currently executing, if any.
    pub current_sub_task_id: Option<Uuid>,
    /// When this worker registered with the server.
    pub registered_at: DateTime<Utc>,
}

/// A key-value secret pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    /// The secret key name.
    pub key: String,
    /// The secret value.
    pub value: String,
}
