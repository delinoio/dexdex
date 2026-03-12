//! DexDex Worker Server.
//!
//! A polling client that picks up SubTask work items from the main server,
//! executes AI agents, and reports results back.

pub mod client;
pub mod config;
pub mod error;
pub mod executor;
pub mod state;
