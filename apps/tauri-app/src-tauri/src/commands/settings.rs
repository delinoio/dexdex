//! Settings-related Tauri commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    config::{save_config, settings_to_config, GlobalSettings, RepositorySettings},
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
        state
            .set_mode(settings.mode, settings.server_url.clone())
            .await?;
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
/// returns an error indicating the feature is not yet implemented. Future
/// versions will save to `.delidev/config.toml` within each repository
/// directory.
///
/// See: https://github.com/delinoio/delidev/issues/52 for implementation tracking.
#[tauri::command]
pub async fn update_repository_settings(
    _state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    _settings: RepositorySettings,
) -> AppResult<RepositorySettings> {
    // Repository-specific settings are not yet implemented.
    // Return an explicit error to avoid silent failures.
    // Tracked in: https://github.com/delinoio/delidev/issues/52
    Err(crate::error::AppError::InvalidRequest(format!(
        "Repository-specific settings for '{}' are not yet implemented. \
         See https://github.com/delinoio/delidev/issues/52",
        repo_id
    )))
}
