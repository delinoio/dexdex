//! Server configuration.

use std::env;

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server host address.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// Whether running in single-user mode.
    pub single_user_mode: bool,
    /// Log level.
    pub log_level: String,
}

impl Config {
    /// Loads configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let single_user_mode = env::var("DEXDEX_SINGLE_USER_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        Ok(Self {
            host: env::var("DEXDEX_SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("DEXDEX_SERVER_PORT")
                .unwrap_or_else(|_| "54871".to_string())
                .parse()
                .unwrap_or(54871),
            single_user_mode,
            log_level: env::var("DEXDEX_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }

    /// Returns the server address.
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_single_user_mode() {
        unsafe {
            env::remove_var("DEXDEX_SINGLE_USER_MODE");
        }

        let config = Config::from_env().unwrap();
        assert!(config.single_user_mode);
    }
}
