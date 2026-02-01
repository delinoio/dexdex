//! Secrets-related Tauri commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    config::AppMode,
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
pub async fn delete_secret(
    state: State<'_, Arc<RwLock<AppState>>>,
    key: String,
) -> AppResult<()> {
    let state = state.read().await;
    state.keychain.delete_by_name(&key).await?;
    info!("Deleted secret: {}", key);
    Ok(())
}

/// Lists all known secret keys that have values.
#[tauri::command]
pub async fn list_secrets(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<Vec<String>> {
    let state = state.read().await;
    let keys = state.keychain.list().await?;
    Ok(keys.into_iter().map(|k| k.key_name().to_string()).collect())
}

/// Sends secrets to the remote server for a task (remote mode only).
#[tauri::command]
pub async fn send_secrets(
    state: State<'_, Arc<RwLock<AppState>>>,
    task_id: String,
) -> AppResult<()> {
    let state = state.read().await;

    if state.mode == AppMode::Local {
        // In local mode, secrets are accessed directly from keychain
        return Ok(());
    }

    // Remote mode not yet implemented
    Err(AppError::InvalidRequest(format!(
        "Remote mode not yet implemented (task_id: {})",
        task_id
    )))
}
