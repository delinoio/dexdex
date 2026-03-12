//! Local runtime implementation.
//!
//! This module provides the runtime that manages the task store and executor
//! lifecycle for single-process (local) mode.
//!
//! NOTE: This is a minimal stub implementation. In the new architecture,
//! workers are separate processes. This runtime uses an in-memory task store.

use std::sync::Arc;

use coding_agents::executor::EventEmitter;
use entities::Workspace;
use task_store::{MemoryTaskStore, TaskStore};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{TtyInputRequestManager, error::WorkerResult, executor::LocalExecutor};

/// Local runtime that manages the task store and executor lifecycle.
///
/// This runtime is designed to be used in single-process (local) mode where
/// the application embeds both server and worker functionality.
///
/// NOTE: Uses in-memory storage (data is not persisted across restarts).
pub struct LocalRuntime<E: EventEmitter> {
    /// Task store (using in-memory storage).
    task_store: Arc<MemoryTaskStore>,
    /// Default workspace ID for single-user mode.
    default_workspace_id: Uuid,
    /// Local executor for running AI agents.
    executor: RwLock<Option<Arc<LocalExecutor<E>>>>,
}

impl<E: EventEmitter + 'static> LocalRuntime<E> {
    /// Creates a new local runtime.
    ///
    /// This initializes the in-memory task store and creates a default
    /// workspace.
    pub async fn new() -> WorkerResult<Self> {
        info!("Initializing local runtime (in-memory mode)");

        // Create in-memory task store
        let task_store = Arc::new(MemoryTaskStore::new());

        // Create default workspace for single-user mode
        let default_workspace = Workspace::new("Default Workspace")
            .with_description("Default workspace for local mode");
        let default_workspace_id = default_workspace.id;
        task_store.create_workspace(default_workspace).await?;
        info!("Created default workspace: {}", default_workspace_id);

        info!(
            "Local runtime initialized with workspace {}",
            default_workspace_id
        );

        Ok(Self {
            task_store,
            default_workspace_id,
            executor: RwLock::new(None),
        })
    }

    /// Initializes the local executor with the provided event emitter.
    pub async fn init_executor(&self, emitter: Arc<E>) {
        // Use a temporary data dir (not used for storage in stub mode)
        let data_dir = std::path::PathBuf::from("/tmp/dexdex");
        let executor = LocalExecutor::new(self.task_store.clone(), data_dir, emitter);
        let mut executor_lock = self.executor.write().await;
        *executor_lock = Some(Arc::new(executor));
        info!("Local executor initialized");
    }

    /// Gets the local executor if initialized.
    pub async fn executor(&self) -> Option<Arc<LocalExecutor<E>>> {
        let executor_lock = self.executor.read().await;
        executor_lock.clone()
    }

    /// Gets the TTY request manager for responding to input requests.
    pub async fn tty_request_manager(&self) -> Option<Arc<TtyInputRequestManager>> {
        let executor_lock = self.executor.read().await;
        executor_lock.as_ref().map(|e| e.tty_request_manager())
    }

    /// Gets a reference to the task store.
    pub fn task_store(&self) -> &dyn TaskStore {
        self.task_store.as_ref()
    }

    /// Gets the task store as an Arc for cloning.
    pub fn task_store_arc(&self) -> Arc<MemoryTaskStore> {
        self.task_store.clone()
    }

    /// Gets the default workspace ID.
    pub fn default_workspace_id(&self) -> Uuid {
        self.default_workspace_id
    }
}
