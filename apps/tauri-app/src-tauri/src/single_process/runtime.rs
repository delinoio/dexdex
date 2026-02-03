//! Single-process runtime implementation.
//!
//! This module provides an embedded server and worker that run in the same
//! process as the Tauri app, using direct function calls instead of network
//! communication.
//!
//! # Data Persistence
//!
//! Task data is persisted to a SQLite database at ~/.delidev/data/tasks.db.
//! Data is preserved across application restarts.

use std::sync::Arc;

use entities::Workspace;
use task_store::{SqliteTaskStore, TaskStore, WorkspaceFilter};
use tauri::AppHandle;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use super::{executor::LocalExecutor, tty_handler::TtyInputRequestManager};
use crate::{config::data_dir, error::AppResult};

/// Single-process runtime that embeds server and worker functionality.
pub struct SingleProcessRuntime {
    /// Task store (using SQLite storage for persistence).
    task_store: Arc<SqliteTaskStore>,
    /// Default workspace ID for single-user mode.
    default_workspace_id: Uuid,
    /// Local executor for running AI agents (initialized lazily when app handle
    /// is available).
    executor: RwLock<Option<Arc<LocalExecutor>>>,
}

impl SingleProcessRuntime {
    /// Creates a new single-process runtime.
    pub async fn new() -> AppResult<Self> {
        info!("Initializing single-process runtime");

        // Create SQLite task store for persistent storage
        let db_path = data_dir()?.join("tasks.db");
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
            "Single-process runtime initialized with workspace {}",
            default_workspace_id
        );

        Ok(Self {
            task_store,
            default_workspace_id,
            executor: RwLock::new(None),
        })
    }

    /// Initializes the local executor with the app handle.
    ///
    /// This must be called after the Tauri app is set up to enable task
    /// execution.
    pub async fn init_executor(&self, app_handle: AppHandle) {
        let executor = LocalExecutor::new(self.task_store.clone(), app_handle);
        let mut executor_lock = self.executor.write().await;
        *executor_lock = Some(Arc::new(executor));
        info!("Local executor initialized");
    }

    /// Gets the local executor if initialized.
    pub async fn executor(&self) -> Option<Arc<LocalExecutor>> {
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
}
