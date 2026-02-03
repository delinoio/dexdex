//! AI coding agent abstraction and output normalization for DeliDev.
//!
//! This crate provides:
//! - A unified interface for running various AI coding agents
//! - Output normalization to a common event format
//! - TTY input detection for interactive prompts
//! - Task execution with platform-agnostic event emission

mod agents;
mod error;
mod event;
pub mod executor;
mod runner;

pub use agents::*;
pub use error::*;
pub use event::*;
pub use executor::*;
pub use runner::*;
