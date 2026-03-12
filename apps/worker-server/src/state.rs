//! Worker server application state.

use std::sync::Arc;

use crate::{client::MainServerClient, config::WorkerConfig};

/// Worker server application state.
pub struct WorkerState {
    /// Worker configuration.
    pub config: WorkerConfig,
    /// Main server client.
    pub client: Arc<MainServerClient>,
}

impl WorkerState {
    /// Creates a new worker state.
    pub fn new(config: WorkerConfig, client: Arc<MainServerClient>) -> Arc<Self> {
        Arc::new(Self { config, client })
    }
}
