//! Mode-related Tauri commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    config::{save_config, settings_to_config, AppMode},
    error::AppResult,
    state::AppState,
};

/// Gets the current application mode.
#[tauri::command]
pub async fn get_mode(state: State<'_, Arc<RwLock<AppState>>>) -> AppResult<String> {
    let state = state.read().await;
    let mode = match state.mode {
        AppMode::Local => "local",
        AppMode::Remote => "remote",
    };
    Ok(mode.to_string())
}

/// Sets the application mode.
#[tauri::command]
pub async fn set_mode(
    state: State<'_, Arc<RwLock<AppState>>>,
    mode: String,
    server_url: Option<String>,
) -> AppResult<()> {
    let mut state = state.write().await;

    let app_mode = match mode.as_str() {
        "local" => AppMode::Local,
        "remote" => AppMode::Remote,
        _ => {
            return Err(crate::error::AppError::InvalidRequest(format!(
                "Invalid mode: {}. Must be 'local' or 'remote'",
                mode
            )))
        }
    };

    info!("Setting mode to {:?}", app_mode);

    state.set_mode(app_mode, server_url).await?;

    // Save settings to config file
    let config = settings_to_config(&state.settings);
    save_config(&config)?;

    Ok(())
}
