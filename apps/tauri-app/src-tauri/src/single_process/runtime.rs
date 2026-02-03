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
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::executor::{EmbeddedExecutor, ExecutorStatus};
use crate::{config::data_dir, error::AppResult};

/// Default polling interval for task execution (5 seconds).
const POLL_INTERVAL_SECS: u64 = 5;

/// Single-process runtime that embeds server and worker functionality.
pub struct SingleProcessRuntime {
    /// Task store (using SQLite storage for persistence).
    task_store: Arc<SqliteTaskStore>,
    /// Default workspace ID for single-user mode.
    default_workspace_id: Uuid,
    /// Embedded task executor.
    executor: Arc<EmbeddedExecutor>,
    /// Shutdown signal sender.
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SingleProcessRuntime {
    /// Creates a new single-process runtime.
    pub async fn new() -> AppResult<Self> {
        info!("Initializing single-process runtime");

        // Create data directory
        let data_path = data_dir()?;
        tokio::fs::create_dir_all(&data_path).await?;

        // Create SQLite task store for persistent storage
        let db_path = data_path.join("tasks.db");
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

        // Create working directory for task execution
        let workdir = data_path.join("worktrees");
        tokio::fs::create_dir_all(&workdir).await?;

        // Create embedded executor
        let executor = Arc::new(EmbeddedExecutor::new(task_store.clone(), workdir));

        info!(
            "Single-process runtime initialized with workspace {}",
            default_workspace_id
        );

        Ok(Self {
            task_store,
            default_workspace_id,
            executor,
            shutdown_tx: None,
        })
    }

    /// Starts the background task polling loop.
    ///
    /// This must be called after the runtime is created to begin task execution.
    pub fn start_polling_loop(&mut self) {
        let executor = self.executor.clone();
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        info!("Starting task polling loop (interval: {}s)", POLL_INTERVAL_SECS);

        tokio::spawn(async move {
            let poll_interval = std::time::Duration::from_secs(POLL_INTERVAL_SECS);

            loop {
                tokio::select! {
                    _ = tokio::time::sleep(poll_interval) => {
                        // Check if we're shutting down
                        if executor.get_status().await == ExecutorStatus::ShuttingDown {
                            info!("Executor shutting down, stopping polling loop");
                            break;
                        }

                        // Try to execute next task
                        match executor.poll_and_execute().await {
                            Ok(true) => {
                                info!("Task execution completed");
                            }
                            Ok(false) => {
                                // No task available, continue polling
                            }
                            Err(e) => {
                                error!("Task execution failed: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Received shutdown signal, stopping polling loop");
                        break;
                    }
                }
            }

            info!("Task polling loop stopped");
        });
    }

    /// Stops the background task polling loop.
    pub async fn stop_polling_loop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            // Signal shutdown
            self.executor.set_status(ExecutorStatus::ShuttingDown).await;

            // Cancel any running task
            self.executor.cancel_current_task().await;

            // Send shutdown signal
            if let Err(e) = tx.send(()).await {
                warn!("Failed to send shutdown signal: {}", e);
            }
        }
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

    /// Gets a reference to the executor.
    pub fn executor(&self) -> &Arc<EmbeddedExecutor> {
        &self.executor
    }
}

impl Drop for SingleProcessRuntime {
    fn drop(&mut self) {
        // Best-effort cleanup - can't use async in drop
        if let Some(tx) = self.shutdown_tx.take() {
            // Try to send shutdown signal synchronously
            let _ = tx.try_send(());
        }
    }
}
