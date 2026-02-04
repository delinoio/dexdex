//! Git operations for DeliDev.
//!
//! This crate provides:
//! - Repository cloning and fetching
//! - Worktree creation and management
//! - Branch operations
//! - Remote URL parsing
//! - Repository caching for improved performance
//! - Security validation for URLs and branch names

mod cache;
mod error;
mod remote;
mod repository;
mod validation;
mod worktree;

pub use cache::*;
pub use error::*;
pub use remote::*;
pub use repository::*;
pub use validation::*;
pub use worktree::*;
