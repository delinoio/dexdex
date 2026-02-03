//! Git operations for DeliDev.
//!
//! This crate provides:
//! - Repository cloning and fetching
//! - Worktree creation and management
//! - Branch operations
//! - Remote URL parsing
//! - Repository caching for improved performance

mod cache;
mod error;
mod remote;
mod repository;
mod worktree;

pub use cache::*;
pub use error::*;
pub use remote::*;
pub use repository::*;
pub use worktree::*;
