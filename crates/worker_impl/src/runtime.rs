//! Local runtime implementation.
//!
//! This module provides the runtime that manages the task store and executor
//! lifecycle for single-process (local) mode.
//!
//! # Data Persistence
//!
//! Task data is persisted to a SQLite database at ~/.delidev/data/tasks.db.
//! Data is preserved across application restarts.

use std::{path::PathBuf, sync::Arc};

use coding_agents::executor::EventEmitter;
use entities::Workspace;
use task_store::{SqliteTaskStore, TaskStore, WorkspaceFilter};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::{
    TtyInputRequestManager,
    error::{WorkerError, WorkerResult},
    executor::LocalExecutor,
};

/// Local runtime that manages the task store and executor lifecycle.
///
/// This runtime is designed to be used in single-process (local) mode where
/// the application embeds both server and worker functionality.
pub struct LocalRuntime<E: EventEmitter> {
    /// Task store (using SQLite storage for persistence).
    task_store: Arc<SqliteTaskStore>,
    /// Default workspace ID for single-user mode.
    default_workspace_id: Uuid,
    /// Local executor for running AI agents (initialized lazily when emitter
    /// is available).
    executor: RwLock<Option<Arc<LocalExecutor<E>>>>,
    /// Data directory path.
    data_dir: PathBuf,
}

impl<E: EventEmitter + 'static> LocalRuntime<E> {
    /// Creates a new local runtime.
    ///
    /// This initializes the SQLite task store and creates a default workspace
    /// if one doesn't exist.
    pub async fn new() -> WorkerResult<Self> {
        info!("Initializing local runtime");

        // Determine the data directory
        let data_dir = get_data_dir()?;

        // Create SQLite task store for persistent storage
        let db_path = data_dir.join("tasks.db");
        let task_store = Arc::new(SqliteTaskStore::new(&db_path).await?);

        // Check if default workspace already exists, otherwise create it
        let (workspaces, _) = task_store
            .list_workspaces(WorkspaceFilter::default())
            .await?;

        let default_workspace_id = if let Some(workspace) = workspaces.first() {
            // Use existing workspace
            info!("Using existing workspace: {}", workspace.id);
            workspace.id
        } else {
            // Create default workspace for single-user mode
            let default_workspace_id = Uuid::new_v4();
            let default_workspace = Workspace {
                id: default_workspace_id,
                user_id: None, // No user in single-user mode
                name: "Default Workspace".to_string(),
                description: Some("Default workspace for local mode".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            task_store.create_workspace(default_workspace).await?;
            info!("Created new default workspace: {}", default_workspace_id);
            default_workspace_id
        };

        info!(
            "Local runtime initialized with workspace {}",
            default_workspace_id
        );

        Ok(Self {
            task_store,
            default_workspace_id,
            executor: RwLock::new(None),
            data_dir,
        })
    }

    /// Initializes the local executor with the provided event emitter.
    ///
    /// This must be called after the runtime is created to enable task
    /// execution. The emitter is platform-specific and handles event delivery.
    pub async fn init_executor(&self, emitter: Arc<E>) {
        let executor = LocalExecutor::new(self.task_store.clone(), self.data_dir.clone(), emitter);
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
    pub fn task_store_arc(&self) -> Arc<SqliteTaskStore> {
        self.task_store.clone()
    }

    /// Gets the default workspace ID.
    pub fn default_workspace_id(&self) -> Uuid {
        self.default_workspace_id
    }

    /// Gets the data directory path.
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}

/// Gets the data directory for the worker.
///
/// Returns an error if the directory cannot be determined.
fn get_data_dir() -> WorkerResult<PathBuf> {
    let config_dir = config::config_dir()
        .ok_or_else(|| WorkerError::Config("Cannot find home directory".to_string()))?;

    let data_dir = config_dir.join("data");
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| WorkerError::Config(format!("Failed to create data directory: {}", e)))?;

    Ok(data_dir)
}
