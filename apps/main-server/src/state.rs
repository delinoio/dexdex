//! Application state.

use std::{collections::HashMap, sync::Arc};

use auth::JwtManager;
use task_store::TaskStore;
use tokio::sync::{RwLock, oneshot};
use uuid::Uuid;

use crate::{config::Config, services::worker_registry::WorkerRegistry};

/// Secrets cache for tasks (in-memory storage).
#[derive(Debug, Default)]
pub struct SecretsCache {
    /// Map of task ID to secrets.
    secrets: HashMap<Uuid, Vec<rpc_protocol::Secret>>,
}

impl SecretsCache {
    /// Creates a new secrets cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores secrets for a task.
    pub fn store(&mut self, task_id: Uuid, secrets: Vec<rpc_protocol::Secret>) {
        self.secrets.insert(task_id, secrets);
    }

    /// Gets secrets for a task.
    pub fn get(&self, task_id: &Uuid) -> Option<&Vec<rpc_protocol::Secret>> {
        self.secrets.get(task_id)
    }

    /// Clears secrets for a task.
    pub fn clear(&mut self, task_id: &Uuid) {
        self.secrets.remove(task_id);
    }
}

/// TTY response relay for delivering user responses to workers.
#[derive(Default)]
pub struct TtyResponseRelay {
    /// Map of request ID to response channel.
    /// Workers wait on these channels for user responses.
    pending: HashMap<Uuid, oneshot::Sender<String>>,
    /// Map of request ID to responses that arrived before the worker polled.
    /// This handles the case where the user responds before the worker checks.
    early_responses: HashMap<Uuid, String>,
}

impl TtyResponseRelay {
    /// Creates a new TTY response relay.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a pending TTY input request.
    /// Returns a receiver that the worker can use to wait for the response.
    pub fn register(&mut self, request_id: Uuid) -> oneshot::Receiver<String> {
        // Check if there's already an early response
        if let Some(response) = self.early_responses.remove(&request_id) {
            let (tx, rx) = oneshot::channel();
            // Immediately send the early response
            let _ = tx.send(response);
            return rx;
        }

        let (tx, rx) = oneshot::channel();
        self.pending.insert(request_id, tx);
        rx
    }

    /// Delivers a response to a pending TTY input request.
    /// Returns true if the response was delivered successfully.
    pub fn deliver(&mut self, request_id: Uuid, response: String) -> bool {
        if let Some(tx) = self.pending.remove(&request_id) {
            tx.send(response).is_ok()
        } else {
            // Worker hasn't polled yet, store as early response
            self.early_responses.insert(request_id, response);
            true
        }
    }

    /// Cancels a pending TTY input request.
    pub fn cancel(&mut self, request_id: &Uuid) {
        self.pending.remove(request_id);
        self.early_responses.remove(request_id);
    }

    /// Checks if there's a pending request.
    pub fn is_pending(&self, request_id: &Uuid) -> bool {
        self.pending.contains_key(request_id) || self.early_responses.contains_key(request_id)
    }

    /// Gets the number of pending requests.
    pub fn pending_count(&self) -> usize {
        self.pending.len() + self.early_responses.len()
    }
}

/// Timeout for HTTP requests to workers (e.g., cancellation signals).
const WORKER_HTTP_TIMEOUT_SECS: u64 = 10;

/// Maximum idle connections per host for the HTTP client pool.
/// This prevents resource exhaustion from malicious or misconfigured workers
/// that could cause the server to open many connections.
const HTTP_POOL_MAX_IDLE_PER_HOST: usize = 5;

/// Idle connection timeout in seconds.
/// Connections idle longer than this are closed to free resources.
const HTTP_POOL_IDLE_TIMEOUT_SECS: u64 = 30;

/// Shared application state.
pub struct AppState<S: TaskStore> {
    /// Server configuration.
    pub config: Config,
    /// Task store.
    pub store: S,
    /// JWT manager (optional, only used in multi-user mode).
    pub jwt_manager: Option<JwtManager>,
    /// Worker registry.
    pub worker_registry: RwLock<WorkerRegistry>,
    /// Secrets cache.
    pub secrets_cache: RwLock<SecretsCache>,
    /// TTY response relay.
    pub tty_response_relay: RwLock<TtyResponseRelay>,
    /// Shared HTTP client for worker communication (reused across requests).
    pub http_client: reqwest::Client,
}

impl<S: TaskStore> AppState<S> {
    /// Creates new application state.
    pub fn new(config: Config, store: S, jwt_manager: Option<JwtManager>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(WORKER_HTTP_TIMEOUT_SECS))
            .pool_max_idle_per_host(HTTP_POOL_MAX_IDLE_PER_HOST)
            .pool_idle_timeout(std::time::Duration::from_secs(HTTP_POOL_IDLE_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config,
            store,
            jwt_manager,
            worker_registry: RwLock::new(WorkerRegistry::new()),
            secrets_cache: RwLock::new(SecretsCache::new()),
            tty_response_relay: RwLock::new(TtyResponseRelay::new()),
            http_client,
        }
    }

    /// Returns true if authentication is enabled.
    pub fn auth_enabled(&self) -> bool {
        self.config.auth_enabled()
    }
}

/// Type alias for shared state.
pub type SharedState<S> = Arc<AppState<S>>;

/// Creates shared state from config and store.
pub fn create_shared_state<S: TaskStore>(
    config: Config,
    store: S,
    jwt_manager: Option<JwtManager>,
) -> SharedState<S> {
    Arc::new(AppState::new(config, store, jwt_manager))
}
