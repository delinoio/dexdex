//! Mobile-specific functionality for DeliDev.
//!
//! This module provides mobile platform detection, remote-only mode
//! enforcement, and mobile-specific features like biometric authentication and
//! push notifications.

pub mod biometric;
pub mod notifications;
pub mod platform;

pub use biometric::*;
pub use notifications::*;
pub use platform::*;
