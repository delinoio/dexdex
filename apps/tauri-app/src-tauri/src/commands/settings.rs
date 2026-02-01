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
    let repo_uuid = Uuid::parse_str(&repo_id).map_err(|e| {
        AppError::InvalidRequest(format!("Invalid repository ID '{}': {}", repo_id, e))
    })?;

    // Get the repository from the task store to find its path
    let task_store = state.task_store()?;
    let repo = task_store
        .get_repository(repo_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;

    // Load repository settings from the local path
    match &repo.local_path {
        Some(local_path) => {
            let repo_path = std::path::Path::new(local_path);
            if repo_path.exists() {
                load_repository_settings(repo_path)
            } else {
                tracing::debug!(
                    "Repository local path '{}' does not exist for {}, returning default settings",
                    local_path,
                    repo_id
                );
                Ok(RepositorySettings::default())
            }
        }
        None => {
            tracing::debug!(
                "Repository local path not configured for {}, returning default settings",
                repo_id
            );
            Ok(RepositorySettings::default())
        }
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
    let repo_uuid = Uuid::parse_str(&repo_id).map_err(|e| {
        AppError::InvalidRequest(format!("Invalid repository ID '{}': {}", repo_id, e))
    })?;

    // Get the repository from the task store to find its path
    let task_store = state.task_store()?;
    let repo = task_store
        .get_repository(repo_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Repository not found: {}", repo_id)))?;

    // Save repository settings to the local path
    match &repo.local_path {
        Some(local_path) => {
            let repo_path = std::path::Path::new(local_path);
            if repo_path.exists() {
                save_repository_settings(repo_path, &settings)?;
                info!("Updated repository settings for {}", repo_id);
                Ok(settings)
            } else {
                Err(AppError::InvalidRequest(format!(
                    "Repository local path '{}' does not exist. Cannot save settings.",
                    local_path
                )))
            }
        }
        None => Err(AppError::InvalidRequest(format!(
            "Repository local path not configured for '{}'. Cannot save settings.",
            repo_id
        ))),
    }
}
