//! DeliDev Tauri App - Desktop and Mobile Application
//!
//! This crate provides the Tauri backend for the DeliDev desktop and mobile
//! applications. It supports both local (single-process) and remote modes.

pub mod commands;
pub mod config;
pub mod error;
pub mod events;
pub mod mobile;
pub mod notifications;
pub mod single_process;
pub mod state;

use std::sync::Arc;

use state::AppState;
use tauri::Manager;
use tracing::info;

/// Initializes the Tauri application.
fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_tracing();

    info!("DeliDev Tauri app starting...");

    // Create application state
    let rt = tokio::runtime::Runtime::new()?;
    let state = rt.block_on(async { AppState::new().await })?;
    app.manage(Arc::new(tokio::sync::RwLock::new(state)));

    // Store the runtime handle
    app.manage(rt);

    info!("DeliDev Tauri app initialized");

    Ok(())
}

/// Initialize tracing for logging.
fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tauri=warn"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init());

    // Global shortcut plugin is only available on desktop
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .setup(|app| {
            setup_app(app)?;

            // Set up global hotkey
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                use tauri_plugin_notification::NotificationExt;

                let app_handle = app.handle().clone();
                let shortcut = if cfg!(target_os = "macos") {
                    "Option+Z"
                } else {
                    "Alt+Z"
                };

                if let Err(e) =
                    app.global_shortcut()
                        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
                            info!("Global hotkey pressed");
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        })
                {
                    tracing::warn!("Failed to register global hotkey: {}", e);

                    // Notify user about the hotkey registration failure
                    let error_msg = format!(
                        "Failed to register global hotkey ({}). You can still use the app, but \
                         the {} shortcut won't work. Error: {}",
                        shortcut, shortcut, e
                    );
                    tracing::error!("{}", error_msg);

                    // Show a notification to the user
                    if let Err(notif_err) = app
                        .notification()
                        .builder()
                        .title("DeliDev Hotkey Error")
                        .body(format!(
                            "Could not register {} hotkey. The app will still work, but the \
                             global shortcut is unavailable.",
                            shortcut
                        ))
                        .show()
                    {
                        tracing::warn!("Failed to show hotkey error notification: {}", notif_err);
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Mode commands
            commands::mode::get_mode,
            commands::mode::set_mode,
            // Task commands
            commands::task::create_unit_task,
            commands::task::create_composite_task,
            commands::task::get_task,
            commands::task::list_tasks,
            commands::task::approve_task,
            commands::task::reject_task,
            commands::task::request_changes,
            // Repository commands
            commands::repository::add_repository,
            commands::repository::list_repositories,
            commands::repository::remove_repository,
            // Workspace commands
            commands::workspace::create_workspace,
            commands::workspace::list_workspaces,
            commands::workspace::get_workspace,
            commands::workspace::update_workspace,
            commands::workspace::delete_workspace,
            commands::workspace::get_default_workspace_id,
            // Settings commands
            commands::settings::get_global_settings,
            commands::settings::update_global_settings,
            commands::settings::get_repository_settings,
            commands::settings::update_repository_settings,
            // Secrets commands
            commands::secrets::get_secret,
            commands::secrets::set_secret,
            commands::secrets::delete_secret,
            commands::secrets::list_secrets,
            commands::secrets::send_secrets,
            // Mobile commands
            commands::mobile::get_platform_info,
            commands::mobile::is_mobile,
            commands::mobile::supports_local_mode,
            commands::mobile::check_biometric_availability,
            commands::mobile::authenticate_biometric,
            commands::mobile::request_push_permission,
            commands::mobile::register_push_notifications,
            commands::mobile::unregister_push_notifications,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
