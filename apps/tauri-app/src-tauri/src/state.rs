//! Application state management.

use std::sync::Arc;

use secrets::{Keychain, NativeKeychain};
#[cfg(desktop)]
use task_store::TaskStore;
use tracing::info;

#[cfg(desktop)]
use crate::single_process::SingleProcessRuntime;
use crate::{
    config::{
        config_to_settings, load_config, save_config, settings_to_config, AppMode, GlobalSettings,
    },
    error::{AppError, AppResult},
    mobile::platform::supports_local_mode,
    remote_client::RemoteClient,
};

/// Shared application state.
pub struct AppState {
    /// Current application mode.
    pub mode: AppMode,
    /// Global settings.
    pub settings: GlobalSettings,
    /// Keychain for secrets.
    pub keychain: Box<dyn Keychain>,
    /// Single process runtime (only used in local mode on desktop).
    #[cfg(desktop)]
    pub local_runtime: Option<SingleProcessRuntime>,
    /// Remote server URL (only used in remote mode).
    pub remote_server_url: Option<String>,
    /// JWT authentication token for remote mode.
    /// This is obtained after successful OIDC authentication with the server.
    /// See `docs/design.md` for authentication flow details.
    pub auth_token: Option<String>,
    /// HTTP client for remote mode.
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Creates a new application state.
    pub async fn new() -> AppResult<Self> {
        // Load configuration
        let config = load_config()?;
        let mut settings = config_to_settings(&config);

        // On mobile, force remote mode as local mode is not supported
        if !supports_local_mode() && settings.mode == AppMode::Local {
            info!("Mobile device detected, forcing remote mode");
            settings.mode = AppMode::Remote;
            // Save the corrected config to avoid repeating this on next launch
            if let Err(e) = save_config(&settings_to_config(&settings)) {
                tracing::warn!("Failed to save corrected config: {}", e);
            }
        }

        // Create keychain
        let keychain: Box<dyn Keychain> = Box::new(NativeKeychain::new());

        // Create local runtime if in local mode (desktop only)
        #[cfg(desktop)]
        let local_runtime = if settings.mode == AppMode::Local && supports_local_mode() {
            Some(SingleProcessRuntime::new().await?)
        } else {
            None
        };

        let remote_server_url = settings.server_url.clone();

        Ok(Self {
            mode: settings.mode,
            settings,
            keychain,
            #[cfg(desktop)]
            local_runtime,
            remote_server_url,
            auth_token: None,
            http_client: reqwest::Client::new(),
        })
    }

    /// Gets the task store (for local mode, desktop only).
    #[cfg(desktop)]
    pub fn task_store(&self) -> AppResult<&dyn TaskStore> {
        self.local_runtime
            .as_ref()
            .map(|rt| rt.task_store())
            .ok_or_else(|| AppError::InvalidRequest("Not in local mode".to_string()))
    }

    /// Sets the application mode.
    pub async fn set_mode(&mut self, mode: AppMode, server_url: Option<String>) -> AppResult<()> {
        self.mode = mode;
        self.remote_server_url = server_url.clone();

        // Update settings
        self.settings.mode = mode;
        self.settings.server_url = server_url;

        // Create or destroy local runtime based on mode (desktop only)
        #[cfg(desktop)]
        match mode {
            AppMode::Local => {
                if self.local_runtime.is_none() {
                    self.local_runtime = Some(SingleProcessRuntime::new().await?);
                }
            }
            AppMode::Remote => {
                self.local_runtime = None;
            }
        }

        Ok(())
    }

    /// Gets the remote server URL for making API calls.
    #[allow(dead_code)]
    pub fn get_remote_url(&self, path: &str) -> AppResult<String> {
        let base_url = self
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config(ERR_REMOTE_URL_NOT_CONFIGURED.to_string()))?;
        Ok(format!("{}{}", base_url.trim_end_matches('/'), path))
    }

    /// Creates a RemoteClient configured with the current auth token.
    ///
    /// This is the recommended way to create a RemoteClient for making API calls
    /// in remote mode. It automatically handles authentication if a token is
    /// available.
    ///
    /// # Errors
    ///
    /// Returns an error if the remote server URL is not configured.
    pub fn get_remote_client(&self) -> AppResult<RemoteClient> {
        let base_url = self
            .remote_server_url
            .as_ref()
            .ok_or_else(|| AppError::Config(ERR_REMOTE_URL_NOT_CONFIGURED.to_string()))?;

        let mut client = RemoteClient::new(self.http_client.clone(), base_url.clone());
        if let Some(ref token) = self.auth_token {
            client = client.with_auth_token(token.clone());
        }
        Ok(client)
    }

    /// Sets the authentication token for remote mode.
    ///
    /// This should be called after successful OIDC authentication with the server.
    #[allow(dead_code)]
    pub fn set_auth_token(&mut self, token: Option<String>) {
        self.auth_token = token;
    }
}

/// Error message constant for remote server URL not configured.
pub const ERR_REMOTE_URL_NOT_CONFIGURED: &str = "Remote server URL not configured";

/// Error message constant for local mode not supported on platform.
pub const ERR_LOCAL_MODE_NOT_SUPPORTED: &str = "Local mode is not supported on this platform";

/// Type alias for shared state in Tauri commands.
pub type SharedState = Arc<tokio::sync::RwLock<AppState>>;
