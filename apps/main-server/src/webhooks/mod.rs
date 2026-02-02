//! GitHub webhook handlers for auto-fix functionality.
//!
//! This module handles incoming GitHub webhooks for:
//! - Pull request review comments (for auto-fix review comments)
//! - Check run events (for auto-fix CI failures)

mod github;
mod types;

pub use github::*;
pub use types::*;
