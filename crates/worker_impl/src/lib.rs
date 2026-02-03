//! Local worker implementation for task execution.
//!
//! This crate provides the core execution logic for running AI coding agents
//! in single-process (local) mode. It's designed to be platform-agnostic and
//! can be used in different environments (desktop, CLI, etc.).
//!
//! # Architecture
//!
//! The crate provides:
//! - `LocalRuntime`: Manages the task store and executor lifecycle
//! - `LocalExecutor`: Executes unit tasks using the AI coding agents
//! - `EventEmitter` trait: Platform-specific event emission (implemented by
//!   consumers)
//!
//! The executor logic is reused across different platforms:
//! - Desktop (Tauri): Uses `TauriEventEmitter` for emitting events to the
//!   frontend
//! - CLI: Could use a simple console emitter
//! - Server (future): Could use WebSocket emitter

mod executor;
mod runtime;

pub mod error;

// Re-export types from coding_agents for convenience
pub use coding_agents::executor::{
    AgentOutputEvent, EventEmitter, ExecutionResult, TaskCompletedEvent, TaskExecutionConfig,
    TaskExecutor, TaskStatusChangedEvent, TaskType, TtyInputRequestEvent, TtyInputRequestManager,
};
pub use executor::LocalExecutor;
pub use runtime::LocalRuntime;
