//! Task executor for running AI agents with platform-agnostic event emission.
//!
//! This module provides the core execution logic for running AI coding agents.
//! It uses traits to abstract platform-specific concerns like event emission,
//! allowing the executor to be used in different environments (Tauri, CLI,
//! server, etc.).

mod emitter;
mod task_executor;
mod tty_manager;

pub use emitter::*;
pub use task_executor::*;
pub use tty_manager::*;
