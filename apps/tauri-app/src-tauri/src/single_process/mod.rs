//! Single-process mode implementation.
//!
//! In single-process mode, the Tauri app embeds both the server and worker
//! functionality, using direct function calls instead of network RPC.
//!
//! This module provides Tauri-specific implementations that delegate to the
//! platform-agnostic `worker_impl` crate.

mod emitter;

pub use emitter::TauriEventEmitter;
// Re-export types from worker_impl for convenience
pub use worker_impl::{LocalExecutor, LocalRuntime, TtyInputRequestManager};

/// Type alias for the Tauri-specific local runtime.
pub type SingleProcessRuntime = LocalRuntime<TauriEventEmitter>;
