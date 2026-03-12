//! RPC protocol definitions for DexDex.
//!
//! This crate contains the request/response types for DexDex's RPC API.
//! The API is designed to be compatible with Connect RPC protocol.

mod error;
pub mod requests;
pub mod responses;
pub mod types;

pub use error::*;
pub use types::*;

/// RPC error codes used in DexDex.
pub mod error_codes {
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const AUTHENTICATION_REQUIRED: i32 = -32001;
    pub const PERMISSION_DENIED: i32 = -32002;
    pub const RESOURCE_NOT_FOUND: i32 = -32003;
    pub const WORKER_UNAVAILABLE: i32 = -32004;
    pub const TASK_EXECUTION_FAILED: i32 = -32005;
}
