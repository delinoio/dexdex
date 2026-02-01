//! Error types for configuration operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during configuration operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read configuration file.
    #[error("failed to read configuration file at {path}: {source}")]
    ReadFile {
        /// Path to the file that failed to read.
        path: PathBuf,
        /// The underlying IO error.
        source: std::io::Error,
    },

    /// Failed to parse TOML configuration.
    #[error("failed to parse TOML configuration at {path}: {source}")]
    ParseToml {
        /// Path to the file that failed to parse.
        path: PathBuf,
        /// The underlying TOML error.
        source: toml::de::Error,
    },

    /// Failed to serialize configuration to TOML.
    #[error("failed to serialize configuration to TOML: {0}")]
    SerializeToml(#[from] toml::ser::Error),

    /// Failed to write configuration file.
    #[error("failed to write configuration file at {path}: {source}")]
    WriteFile {
        /// Path to the file that failed to write.
        path: PathBuf,
        /// The underlying IO error.
        source: std::io::Error,
    },

    /// Configuration directory not found.
    #[error("configuration directory not found (HOME directory is not set)")]
    ConfigDirNotFound,

    /// Invalid configuration value.
    #[error("invalid configuration value for {field}: {message}")]
    InvalidValue {
        /// The field with the invalid value.
        field: String,
        /// Description of why the value is invalid.
        message: String,
    },

    /// Path is outside the allowed directory.
    #[error("path {path} is outside the allowed directory {allowed_dir}")]
    PathTraversal {
        /// The path that is outside the allowed directory.
        path: PathBuf,
        /// The allowed directory.
        allowed_dir: PathBuf,
    },
}
