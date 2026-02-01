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
/// Returns default repository settings. Repository-specific settings are
/// loaded directly from `.delidev/config.toml` within each repository when
/// performing operations on that repository.
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

    // Return default settings - actual repository settings are loaded
    // from the repository's .delidev/config.toml when performing operations
    tracing::debug!(
        "Returning default repository settings for {}",
        repo_id
    );
    Ok(RepositorySettings::default())
}

/// Updates repository-specific settings.
///
/// Repository settings should be edited directly in the repository's
/// `.delidev/config.toml` file. This command validates the repository exists
/// but returns an error since the app does not manage repository settings.
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

    // Repository settings are managed directly in each repository's .delidev/config.toml file
    Err(AppError::InvalidRequest(format!(
        "Cannot save repository settings for '{}': \
         repository settings must be edited directly in the repository's .delidev/config.toml file.",
        repo_id
    )))
}
