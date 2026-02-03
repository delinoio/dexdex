//! Tauri-specific event emitter implementation.
//!
//! This module provides the `TauriEventEmitter` which implements the
//! `EventEmitter` trait from `worker_impl` to emit events via the Tauri app
//! handle.

use std::sync::Arc;

use coding_agents::AgentResult;
use tauri::{AppHandle, Emitter};
use worker_impl::{
    AgentOutputEvent as CoreAgentOutputEvent, EventEmitter,
    TaskCompletedEvent as CoreTaskCompletedEvent,
    TaskStatusChangedEvent as CoreTaskStatusChangedEvent, TaskType as CoreTaskType,
    TtyInputRequestEvent as CoreTtyInputRequestEvent,
};

use crate::events::{
    event_names, AgentOutputEvent, TaskCompletedEvent, TaskStatusChangedEvent, TaskType,
    TtyInputRequestEvent,
};

/// Tauri-specific event emitter that emits events via the Tauri app handle.
pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    /// Creates a new Tauri event emitter.
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Creates a new Tauri event emitter wrapped in an Arc.
    pub fn new_arc(app_handle: AppHandle) -> Arc<Self> {
        Arc::new(Self::new(app_handle))
    }
}

impl EventEmitter for TauriEventEmitter {
    fn emit_task_status_changed(&self, event: CoreTaskStatusChangedEvent) -> AgentResult<()> {
        let tauri_event = TaskStatusChangedEvent {
            task_id: event.task_id,
            task_type: match event.task_type {
                CoreTaskType::UnitTask => TaskType::UnitTask,
                CoreTaskType::CompositeTask => TaskType::CompositeTask,
            },
            old_status: event.old_status,
            new_status: event.new_status,
        };

        self.app_handle
            .emit(event_names::TASK_STATUS_CHANGED, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }

    fn emit_agent_output(&self, event: CoreAgentOutputEvent) -> AgentResult<()> {
        let tauri_event = AgentOutputEvent {
            task_id: event.task_id,
            session_id: event.session_id,
            event: event.event,
        };

        self.app_handle
            .emit(event_names::AGENT_OUTPUT, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }

    fn emit_tty_input_request(&self, event: CoreTtyInputRequestEvent) -> AgentResult<()> {
        let tauri_event = TtyInputRequestEvent {
            request_id: event.request_id,
            task_id: event.task_id,
            session_id: event.session_id,
            question: event.question,
            options: event.options,
        };

        self.app_handle
            .emit(event_names::TTY_INPUT_REQUEST, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }

    fn emit_task_completed(&self, event: CoreTaskCompletedEvent) -> AgentResult<()> {
        let tauri_event = TaskCompletedEvent {
            task_id: event.task_id,
            task_type: match event.task_type {
                CoreTaskType::UnitTask => TaskType::UnitTask,
                CoreTaskType::CompositeTask => TaskType::CompositeTask,
            },
            success: event.success,
            error: event.error,
        };

        self.app_handle
            .emit(event_names::TASK_COMPLETED, &tauri_event)
            .map_err(|e| coding_agents::AgentError::Other(format!("Failed to emit event: {}", e)))
    }
}
