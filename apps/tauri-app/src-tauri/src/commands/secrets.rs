//! Secrets-related Tauri commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    error::{AppError, AppResult},
    state::AppState,
};

/// Gets a secret from the keychain.
#[tauri::command]
pub async fn get_secret(
    state: State<'_, Arc<RwLock<AppState>>>,
    key: String,
) -> AppResult<Option<String>> {
    let state = state.read().await;
    let value = state.keychain.get_by_name(&key).await?;
    Ok(value)
}

/// Sets a secret in the keychain.
#[tauri::command]
pub async fn set_secret(
    state: State<'_, Arc<RwLock<AppState>>>,
    key: String,
    value: String,
) -> AppResult<()> {
    let state = state.read().await;
    state.keychain.set_by_name(&key, &value).await?;
    info!("Set secret: {}", key);
    Ok(())
}

/// Deletes a secret from the keychain.
#[tauri::command]
pub async fn delete_secret(state: State<'_, Arc<RwLock<AppState>>>, key: String) -> AppResult<()> {
    let state = state.read().await;
    state.keychain.delete_by_name(&key).await?;
    info!("Deleted secret: {}", key);
    Ok(())
}

/// Lists all known secret keys that have values.
#[tauri::command]
pub async fn list_secrets(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<Vec<String>> {
    let state = state.read().await;
    let keys = state.keychain.list().await?;
    Ok(keys.into_iter().map(|k| k.key_name().to_string()).collect())
}

/// Sends secrets to the remote server for a task (remote workspaces only).
///
/// For local workspaces, secrets are accessed directly from the keychain,
/// so this is a no-op.
#[tauri::command]
pub async fn send_secrets(
    _state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    // TODO: Look up the task's workspace to determine if this is a remote
    // workspace. For now, always succeed since local workspaces don't need
    // secret forwarding.
    info!(
        "send_secrets called for task_id={} (no-op for local workspaces)",
        task_id
    );
    Ok(())
}
