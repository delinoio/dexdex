//! Single-process mode implementation.
//!
//! In single-process mode, the Tauri app embeds both the server and worker
//! functionality, using direct function calls instead of network RPC.

mod executor;
mod runtime;
mod tty_handler;

pub use executor::LocalExecutor;
pub use runtime::SingleProcessRuntime;
pub use tty_handler::TtyInputRequestManager;
