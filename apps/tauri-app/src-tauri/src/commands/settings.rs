//! Settings-related Tauri commands.

use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    config::{
        load_repository_settings, save_config, save_repository_settings, settings_to_config,
        GlobalSettings, RepositorySettings,
    },
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
#[tauri::command]
pub async fn get_repository_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
) -> AppResult<RepositorySettings> {
    let state = state.read().await;

    // Parse the repo_id as UUID
    let repo_uuid = Uuid::parse_str(&repo_id)
        .map_err(|_| AppError::InvalidRequest(format!("Invalid repository ID: {}", repo_id)))?;

    // Get the repository from the task store to find its path
    let task_store = state.task_store()?;
    let repo = task_store
        .get_repository(repo_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;

    // Load repository settings from the repo path
    // The remote_url is used to find the local clone path
    // For now, we try to find a local directory that matches
    // TODO: Store local_path in repository entity or use git_ops to locate
    let repo_path = std::path::Path::new(&repo.remote_url);
    if repo_path.exists() {
        load_repository_settings(repo_path)
    } else {
        // If we can't find the local path, return defaults
        tracing::debug!(
            "Repository local path not found for {}, returning default settings",
            repo_id
        );
        Ok(RepositorySettings::default())
    }
}

/// Updates repository-specific settings.
///
/// Saves settings to `.delidev/config.toml` within the repository directory.
#[tauri::command]
pub async fn update_repository_settings(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    settings: RepositorySettings,
) -> AppResult<RepositorySettings> {
    let state = state.read().await;

    // Parse the repo_id as UUID
    let repo_uuid = Uuid::parse_str(&repo_id)
        .map_err(|_| AppError::InvalidRequest(format!("Invalid repository ID: {}", repo_id)))?;

    // Get the repository from the task store to find its path
    let task_store = state.task_store()?;
    let repo = task_store
        .get_repository(repo_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;

    // Save repository settings to the repo path
    let repo_path = std::path::Path::new(&repo.remote_url);
    if repo_path.exists() {
        save_repository_settings(repo_path, &settings)?;
        info!("Updated repository settings for {}", repo_id);
        Ok(settings)
    } else {
        Err(AppError::InvalidRequest(format!(
            "Repository local path not found for '{}'. Cannot save settings.",
            repo_id
        )))
    }
}
