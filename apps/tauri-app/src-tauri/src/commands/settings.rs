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
///
/// # Note
///
/// Repository-specific settings are not yet persisted. This function currently
/// returns default settings. Future versions will load settings from
/// `.delidev/config.toml` within each repository directory.
///
/// See: https://github.com/delinoio/delidev/issues/52 for implementation tracking.
#[tauri::command]
pub async fn get_repository_settings(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _repo_id: String,
) -> AppResult<RepositorySettings> {
    // Note: Repository-specific settings are not yet implemented.
    // This returns defaults until repository config loading is added.
    // Tracked in: https://github.com/delinoio/delidev/issues/52
    tracing::debug!(
        "Repository settings requested for {}; returning defaults (not yet implemented)",
        _repo_id
    );
    Ok(RepositorySettings::default())
}

/// Updates repository-specific settings.
///
/// # Note
///
/// Repository-specific settings are not yet persisted. This function currently
/// accepts settings but does not save them to disk. Future versions will save
/// to `.delidev/config.toml` within each repository directory.
///
/// See: https://github.com/delinoio/delidev/issues/52 for implementation tracking.
#[tauri::command]
pub async fn update_repository_settings(
    _state: State<'_, Arc<RwLock<AppState>>>,
    _repo_id: String,
    settings: RepositorySettings,
) -> AppResult<RepositorySettings> {
    // Note: Repository-specific settings are not yet persisted.
    // Settings are accepted but not saved until repository config is implemented.
    // Tracked in: https://github.com/delinoio/delidev/issues/52
    tracing::warn!(
        "Repository settings update for {} not persisted (not yet implemented)",
        _repo_id
    );
    info!("Updated repository settings (in-memory only)");
    Ok(settings)
}
