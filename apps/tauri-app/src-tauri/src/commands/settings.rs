//! Settings-related Tauri commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    config::{GlobalSettings, RepositorySettings, save_config, settings_to_config},
    error::AppResult,
    state::AppState,
};

/// Gets global settings.
#[tauri::command]
pub async fn get_global_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
) -> AppResult<GlobalSettings> {
    let state = state.read().await;
    Ok(state.settings.clone())
}

/// Updates global settings.
#[tauri::command]
pub async fn update_global_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    settings: GlobalSettings,
) -> AppResult<GlobalSettings> {
    let mut state = state.write().await;

    // Update mode if changed
    if settings.mode != state.settings.mode || settings.server_url != state.settings.server_url {
        state.set_mode(settings.mode, settings.server_url.clone()).await?;
    }

    // Update settings
    state.settings = settings.clone();

    // Save to config file
    let config = settings_to_config(&state.settings);
    save_config(&config)?;

    info!("Updated global settings");
    Ok(settings)
}

/// Gets repository-specific settings.
#[tauri::command]
pub async fn get_repository_settings(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _repo_id: String,
) -> AppResult<RepositorySettings> {
    // TODO: Load from .delidev/config.toml in the repository
    // For now, return defaults
    Ok(RepositorySettings::default())
}

/// Updates repository-specific settings.
#[tauri::command]
pub async fn update_repository_settings(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _repo_id: String,
    settings: RepositorySettings,
) -> AppResult<RepositorySettings> {
    // TODO: Save to .delidev/config.toml in the repository
    info!("Updated repository settings");
    Ok(settings)
}
