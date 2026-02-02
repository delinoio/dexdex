//! PLAN.yaml parsing and validation for DeliDev.
//!
//! This crate provides functionality to parse, validate, and execute
//! PLAN.yaml files that define task graphs for CompositeTask execution.
//!
//! # Example
//!
//! ```yaml
//! tasks:
//!   - id: "setup-db"
//!     title: "Setup Database Schema"
//!     prompt: "Create database schema for user authentication"
//!     branchName: "feature/auth-database"
//!
//!   - id: "auth-api"
//!     prompt: "Implement auth API endpoints"
//!     dependsOn: ["setup-db"]
//! ```

mod parser;
mod validator;
mod executor;

pub use parser::*;
pub use validator::*;
pub use executor::*;
