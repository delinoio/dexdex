//! AI coding agent implementations.
//!
//! This module provides implementations for various AI coding agents,
//! all implementing a common trait for unified handling.

mod aider;
mod amp;
mod claude_code;
mod codex_cli;
mod gemini_cli;
mod open_code;
mod traits;

pub use aider::AiderAgent;
pub use amp::AmpAgent;
pub use claude_code::ClaudeCodeAgent;
pub use codex_cli::CodexCliAgent;
pub use gemini_cli::GeminiCliAgent;
pub use open_code::OpenCodeAgent;
pub use traits::{Agent, AgentConfig, TtyInputHandler};
