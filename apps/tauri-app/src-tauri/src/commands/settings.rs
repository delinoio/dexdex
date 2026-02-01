//! Settings-related Tauri commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::{save_config, settings_to_config, GlobalSettings, RepositorySettings},
    error::{AppError, AppResult},
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
/// Loads settings from `.delidev/config.toml` within the repository directory.
///
/// Note: This currently returns default settings as the repository local path
/// is not stored. In the future, this could be enhanced to discover the local
/// clone path through other means (e.g., git remote matching).
#[tauri::command]
pub async fn get_repository_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
) -> AppResult<RepositorySettings> {
    let state = state.read().await;

    // Parse the repo_id as UUID
    let repo_uuid = Uuid::parse_str(&repo_id).map_err(|e| {
        AppError::InvalidRequest(format!("Invalid repository ID '{}': {}", repo_id, e))
    })?;

    // Get the repository from the task store to verify it exists
    let task_store = state.task_store()?;
    let _repo = task_store
        .get_repository(repo_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;

    // Return default settings since we don't have a local path stored
    tracing::debug!(
        "Repository local path not available for {}, returning default settings",
        repo_id
    );
    Ok(RepositorySettings::default())
}

/// Updates repository-specific settings.
///
/// Saves settings to `.delidev/config.toml` within the repository directory.
///
/// Note: This currently returns an error as the repository local path is not
/// stored. In the future, this could be enhanced to discover the local clone
/// path through other means (e.g., git remote matching).
#[tauri::command]
pub async fn update_repository_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    _settings: RepositorySettings,
) -> AppResult<RepositorySettings> {
    let state = state.read().await;

    // Parse the repo_id as UUID
    let repo_uuid = Uuid::parse_str(&repo_id).map_err(|e| {
        AppError::InvalidRequest(format!("Invalid repository ID '{}': {}", repo_id, e))
    })?;

    // Get the repository from the task store to verify it exists
    let task_store = state.task_store()?;
    let _repo = task_store
        .get_repository(repo_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;

    // Cannot save settings without a local path
    Err(AppError::InvalidRequest(format!(
        "Cannot save repository settings for '{}': local path not available. \
         Repository settings must be edited directly in the repository's .delidev/config.toml file.",
        repo_id
    )))
}
