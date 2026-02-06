//! Application state management.

use std::sync::Arc;

use entities::WorkspaceKind;
use secrets::{Keychain, NativeKeychain};
#[cfg(desktop)]
use task_store::TaskStore;

#[cfg(desktop)]
use crate::single_process::SingleProcessRuntime;
use crate::{
    config::{config_to_settings, load_config, GlobalSettings},
    error::{AppError, AppResult},
    remote_client::RemoteClient,
};

/// Shared application state.
///
/// The app no longer has a single global mode. Instead, each workspace
/// carries its own `kind` (local or remote). The local runtime is always
/// initialised on desktop to serve local workspaces, and remote workspaces
/// store their own server URL.
pub struct AppState {
    /// Global settings (hotkey, notifications, agents, etc.).
    pub settings: GlobalSettings,
    /// Keychain for secrets.
    pub keychain: Box<dyn Keychain>,
    /// Single process runtime (always created on desktop; serves local
    /// workspaces and stores all workspace metadata).
    #[cfg(desktop)]
    pub local_runtime: Option<SingleProcessRuntime>,
    /// JWT authentication token for remote workspaces.
    /// This is obtained after successful OIDC authentication with the server.
    /// See `docs/design.md` for authentication flow details.
    pub auth_token: Option<String>,
    /// HTTP client for remote requests.
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Creates a new application state.
    pub async fn new() -> AppResult<Self> {
        // Load configuration
        let config = load_config()?;
        let settings = config_to_settings(&config);

        // Create keychain
        let keychain: Box<dyn Keychain> = Box::new(NativeKeychain::new());

        // Always create local runtime on desktop. It holds the SQLite store
        // that persists workspace metadata (including remote workspaces).
        #[cfg(desktop)]
        let local_runtime = {
            match SingleProcessRuntime::new().await {
                Ok(rt) => Some(rt),
                Err(e) => {
                    tracing::error!("Failed to initialize local runtime: {}", e);
                    None
                }
            }
        };

        Ok(Self {
            settings,
            keychain,
            #[cfg(desktop)]
            local_runtime,
            auth_token: None,
            http_client: reqwest::Client::new(),
        })
    }

    /// Gets the task store (desktop only).
    #[cfg(desktop)]
    pub fn task_store(&self) -> AppResult<&dyn TaskStore> {
        self.local_runtime
            .as_ref()
            .map(|rt| rt.task_store())
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))
    }

    /// Creates a RemoteClient for a specific server URL.
    ///
    /// This is used when operating on remote workspaces. The server URL
    /// comes from the workspace itself.
    pub fn get_remote_client_for_url(&self, server_url: &str) -> AppResult<RemoteClient> {
        let mut client = RemoteClient::new(self.http_client.clone(), server_url.to_string());
        if let Some(ref token) = self.auth_token {
            client = client.with_auth_token(token.clone());
        }
        Ok(client)
    }

    /// Returns whether the given workspace kind is remote.
    ///
    /// Helper used by Tauri commands to decide routing.
    pub fn is_workspace_remote(kind: &WorkspaceKind) -> bool {
        *kind == WorkspaceKind::Remote
    }

    /// Sets the authentication token for remote workspaces.
    ///
    /// This should be called after successful OIDC authentication with the
    /// server.
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
