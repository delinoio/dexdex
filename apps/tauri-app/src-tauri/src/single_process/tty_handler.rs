//! Local TTY input handler for agent interactions.
//!
//! This module re-exports the TTY handler types from the `coding_agents` crate
//! for backward compatibility with existing code.

// Re-export TTY types from coding_agents for backward compatibility
pub use coding_agents::executor::TtyInputRequestManager;
