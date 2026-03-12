//! Task storage abstraction for DexDex.
//!
//! This crate provides a trait-based abstraction for task storage,
//! with implementations for in-memory (testing) and persistent backends.

mod error;
mod memory;
mod traits;

pub use error::*;
pub use memory::MemoryTaskStore;
pub use traits::*;
