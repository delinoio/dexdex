//! DeliDev Worker Server entry point.

use tracing_subscriber::EnvFilter;
use worker_server::config::WorkerConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = WorkerConfig::from_env();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .init();

    tracing::info!("Starting DeliDev Worker Server");
    tracing::info!("Main server: {}", config.main_server_url);
    tracing::info!("Worker name: {}", config.worker_name);
    tracing::info!("Worker port: {}", config.worker_port);

    // Run the server
    worker_server::run_server(config).await?;

    Ok(())
}
