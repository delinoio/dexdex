//! DexDex Main Server
//!
//! The Main Server is the central hub of DexDex's distributed architecture.
//! It maintains the task list, coordinates workers, and provides the RPC
//! interface for clients.

pub mod api;
pub mod broker;
pub mod config;
pub mod error;
pub mod state;

use std::sync::Arc;

use axum::Router;
use task_store::TaskStore;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    config::Config,
    state::{AppState, SharedState},
};

/// Creates the application router with all routes configured.
pub fn create_app(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    api::create_router()
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Creates the application state with the given configuration and store.
pub fn create_state<S: TaskStore + 'static>(config: Config, store: S) -> SharedState {
    Arc::new(AppState::new(config, Arc::new(store)))
}

/// Initializes tracing with the given log level.
pub fn init_tracing(log_level: &str) {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}
