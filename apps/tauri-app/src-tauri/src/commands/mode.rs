//! Mode-related Tauri commands (deprecated - now workspace-based).
//!
//! These commands are kept for backwards compatibility. The concept of a
//! global "mode" has been replaced by per-workspace `kind`. Each workspace
//! can be either `local` or `remote`.

use std::sync::Arc;

use entities::WorkspaceKind;
use task_store::TaskStore;
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    error::{AppError, AppResult},
    mobile::platform::supports_local_mode,
    state::AppState,
};

/// Gets the current workspace kind for the default workspace.
///
/// Returns "local" or "remote" based on the default workspace's kind.
/// If no workspaces exist yet, returns "local" on desktop and "remote"
/// on mobile.
#[tauri::command]
pub async fn get_mode(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<String> {
    let state = state.read().await;

    #[cfg(desktop)]
    {
        if let Some(ref runtime) = state.local_runtime {
            let workspace = runtime
                .task_store()
                .get_workspace(runtime.default_workspace_id())
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

            if let Some(ws) = workspace {
                return Ok(ws.kind.to_string());
            }
        }
    }

    // On mobile or if no workspace exists, default to remote
    if supports_local_mode() {
        Ok("local".to_string())
    } else {
        Ok("remote".to_string())
    }
}

/// Sets the workspace kind for the default workspace.
///
/// This creates the default workspace if it doesn't exist, and updates
/// its kind and server_url.
#[tauri::command]
pub async fn set_mode(
    state: State<'_, Arc<RwLock<AppState>>>,
    mode: String,
    server_url: Option<String>,
) -> AppResult<()> {
    let kind = match mode.as_str() {
        "local" => {
            if !supports_local_mode() {
                return Err(AppError::PlatformError(
                    "Local mode is not supported on mobile devices. Please use remote mode."
                        .to_string(),
                ));
            }
            WorkspaceKind::Local
        }
        "remote" => WorkspaceKind::Remote,
        _ => {
            return Err(AppError::InvalidRequest(format!(
                "Invalid mode: {}. Must be 'local' or 'remote'",
                mode
            )));
        }
    };

    info!("Setting default workspace kind to {:?}", kind);

    let state = state.read().await;

    #[cfg(desktop)]
    {
        let runtime = state
            .local_runtime
            .as_ref()
            .ok_or_else(|| AppError::Internal("Local runtime not initialized".to_string()))?;

        let workspace_id = runtime.default_workspace_id();
        let existing = runtime
            .task_store()
            .get_workspace(workspace_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(mut ws) = existing {
            ws.kind = kind;
            ws.server_url = server_url;
            ws.updated_at = chrono::Utc::now();
            runtime
                .task_store_arc()
                .update_workspace(ws)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

    // Suppress unused variable warnings on non-desktop platforms
    #[cfg(not(desktop))]
    {
        let _ = (&kind, &server_url);
    }

    Ok(())
}
