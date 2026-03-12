//! DexDex Worker Server entry point.
//!
//! Loads configuration, registers with the main server, and starts the
//! polling executor loop.

use std::sync::Arc;

use tracing_subscriber::EnvFilter;
use worker_server::{
    client::MainServerClient, config::WorkerConfig, executor::Executor, state::WorkerState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present.
    dotenvy::dotenv().ok();

    // Load configuration from environment.
    let config = WorkerConfig::from_env();

    // Initialize logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .init();

    tracing::info!("Starting DexDex Worker Server");
    tracing::info!("Main server URL: {}", config.main_server_url);
    tracing::info!("Worker name: {}", config.worker_name);
    tracing::info!("Agent type: {:?}", config.agent_type);
    tracing::info!("Poll interval: {}ms", config.poll_interval_ms);

    // Register with the main server.
    let client =
        Arc::new(MainServerClient::register(&config.main_server_url, &config.worker_name).await?);

    tracing::info!(
        "Registered with main server, worker_id={}",
        client.worker_id()
    );

    // Build shared state.
    let _state = WorkerState::new(config.clone(), client.clone());

    // Build executor and run the main loop.
    let executor = Executor::new(config, client.clone());

    // Handle Ctrl+C gracefully.
    let client_shutdown = client.clone();
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            tracing::info!("Received shutdown signal, unregistering from main server...");
            if let Err(e) = client_shutdown.unregister().await {
                tracing::warn!("Failed to unregister on shutdown: {}", e);
            }
            std::process::exit(0);
        }
    });

    executor.run().await;

    Ok(())
}
