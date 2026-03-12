//! Application state.

use std::{collections::HashMap, sync::Arc};

use rpc_protocol::Secret;
use task_store::TaskStore;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{broker::EventBroker, config::Config};

/// Secrets cache for tasks (in-memory storage).
#[derive(Debug, Default)]
pub struct SecretsCache {
    secrets: HashMap<Uuid, Vec<Secret>>,
}

impl SecretsCache {
    /// Creates a new secrets cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores secrets for a subtask.
    pub fn store(&mut self, sub_task_id: Uuid, secrets: Vec<Secret>) {
        self.secrets.insert(sub_task_id, secrets);
    }

    /// Gets secrets for a subtask.
    pub fn get(&self, sub_task_id: &Uuid) -> Option<&Vec<Secret>> {
        self.secrets.get(sub_task_id)
    }

    /// Clears secrets for a subtask.
    pub fn clear(&mut self, sub_task_id: &Uuid) {
        self.secrets.remove(sub_task_id);
    }
}

/// Worker registry entry.
#[derive(Debug, Clone)]
pub struct WorkerEntry {
    /// Worker ID.
    pub id: Uuid,
    /// Worker name.
    pub name: String,
    /// Worker endpoint URL.
    pub endpoint_url: String,
    /// Current subtask being executed.
    pub current_sub_task_id: Option<Uuid>,
    /// Last heartbeat time.
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Worker registry.
#[derive(Debug, Default)]
pub struct WorkerRegistry {
    workers: HashMap<Uuid, WorkerEntry>,
}

impl WorkerRegistry {
    /// Creates a new worker registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new worker and returns its ID.
    pub fn register(&mut self, name: String, endpoint_url: String) -> Uuid {
        let id = Uuid::new_v4();
        let entry = WorkerEntry {
            id,
            name,
            endpoint_url,
            current_sub_task_id: None,
            last_heartbeat: chrono::Utc::now(),
        };
        self.workers.insert(id, entry);
        id
    }

    /// Updates a worker's heartbeat.
    pub fn heartbeat(&mut self, worker_id: Uuid, current_sub_task_id: Option<Uuid>) -> bool {
        if let Some(worker) = self.workers.get_mut(&worker_id) {
            worker.last_heartbeat = chrono::Utc::now();
            worker.current_sub_task_id = current_sub_task_id;
            true
        } else {
            false
        }
    }

    /// Unregisters a worker.
    pub fn unregister(&mut self, worker_id: Uuid) -> bool {
        self.workers.remove(&worker_id).is_some()
    }

    /// Gets a worker by ID.
    pub fn get(&self, worker_id: &Uuid) -> Option<&WorkerEntry> {
        self.workers.get(worker_id)
    }
}

/// Shared application state.
pub struct AppState {
    /// Server configuration.
    pub config: Config,
    /// Task store.
    pub store: Arc<dyn TaskStore>,
    /// Event broker for SSE.
    pub broker: EventBroker,
    /// Secrets cache.
    pub secrets_cache: RwLock<SecretsCache>,
    /// Worker registry.
    pub worker_registry: RwLock<WorkerRegistry>,
}

impl AppState {
    /// Creates new application state.
    pub fn new(config: Config, store: Arc<dyn TaskStore>) -> Self {
        Self {
            config,
            store,
            broker: EventBroker::new(),
            secrets_cache: RwLock::new(SecretsCache::new()),
            worker_registry: RwLock::new(WorkerRegistry::new()),
        }
    }
}

/// Type alias for shared state.
pub type SharedState = Arc<AppState>;
