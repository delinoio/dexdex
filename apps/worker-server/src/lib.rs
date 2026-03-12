//! DexDex Worker Server.
//!
//! The Worker Server executes AI coding agents in isolated Docker containers
//! and reports progress to the Main Server.

pub mod api;
pub mod client;
pub mod config;
pub mod docker;
pub mod error;
pub mod executor;
pub mod state;

use std::sync::Arc;

use axum::Router;
use client::MainServerClient;
use config::WorkerConfig;
use error::WorkerResult;
use executor::TaskExecutor;
use state::{AppState, WorkerStatus};
use tokio::sync::mpsc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};

/// Runs the heartbeat loop.
async fn run_heartbeat_loop(client: Arc<MainServerClient>, interval_secs: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        if let Err(e) = client.heartbeat().await {
            warn!("Heartbeat failed: {}", e);
        }
    }
}

/// Runs the task polling loop.
async fn run_task_loop(
    state: Arc<AppState>,
    client: Arc<MainServerClient>,
    executor: Arc<TaskExecutor>,
    mut shutdown_rx: mpsc::Receiver<()>,
) {
    let poll_interval = std::time::Duration::from_secs(5);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {
                // Check if we're idle and can accept a task
                if state.get_status().await != WorkerStatus::Idle {
                    continue;
                }

                // Try to get a task
                match client.get_task().await {
                    Ok(Some(task)) => {
                        info!("Received task: {}", task.task_id);

                        // Execute the task
                        if let Err(e) = executor.execute(task).await {
                            error!("Task execution failed: {}", e);
                        }
                    }
                    Ok(None) => {
                        // No task available, continue polling
                    }
                    Err(e) => {
                        warn!("Failed to get task: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                info!("Shutting down task loop");
                break;
            }
        }
    }
}

/// Creates the worker server.
pub async fn create_server(config: WorkerConfig) -> WorkerResult<(Router, Arc<AppState>)> {
    let state = AppState::new(config).await?;

    let router = Router::new()
        .nest("/api", api::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    Ok((router, state))
}

/// Runs the worker server.
pub async fn run_server(config: WorkerConfig) -> WorkerResult<()> {
    // Create application state
    let state = AppState::new(config.clone()).await?;

    // Create main server client
    let client = Arc::new(MainServerClient::new(state.clone()));

    // Register with main server
    let worker_id = client.register().await?;
    state.set_worker_id(worker_id).await;

    // Create task executor
    let executor = Arc::new(TaskExecutor::new(state.clone(), client.clone()));

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

    // Spawn heartbeat loop
    let heartbeat_client = client.clone();
    let heartbeat_interval = config.heartbeat_interval_secs;
    tokio::spawn(async move {
        run_heartbeat_loop(heartbeat_client, heartbeat_interval).await;
    });

    // Spawn task polling loop
    let task_state = state.clone();
    let task_client = client.clone();
    let task_executor = executor.clone();
    tokio::spawn(async move {
        run_task_loop(task_state, task_client, task_executor, shutdown_rx).await;
    });

    // Build router
    let router = Router::new()
        .nest("/api", api::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    // Bind server
    let addr = format!("0.0.0.0:{}", config.worker_port);
    info!("Worker server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Handle shutdown signal
    let shutdown_client = client.clone();
    let shutdown_state = state.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Received shutdown signal");

        // Set status to shutting down
        shutdown_state.set_status(WorkerStatus::ShuttingDown).await;

        // Cancel current task if any
        shutdown_state.cancel_current_task().await;

        // Unregister from main server
        if let Err(e) = shutdown_client.unregister().await {
            warn!("Failed to unregister: {}", e);
        }

        // Send shutdown signal to task loop
        let _ = shutdown_tx.send(()).await;
    });

    // Run server
    axum::serve(listener, router).await?;

    Ok(())
}
