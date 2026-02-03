//! Single-process mode implementation.
//!
//! In single-process mode, the Tauri app embeds both the server and worker
//! functionality, using direct function calls instead of network RPC.

mod executor;
mod runtime;

pub use executor::{EmbeddedExecutor, ExecutorStatus};
pub use runtime::SingleProcessRuntime;
