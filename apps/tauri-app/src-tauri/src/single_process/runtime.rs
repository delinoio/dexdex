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
use tracing::info;
use uuid::Uuid;

use crate::{config::data_dir, error::AppResult};

/// Single-process runtime that embeds server and worker functionality.
pub struct SingleProcessRuntime {
    /// Task store (using SQLite storage for persistence).
    task_store: Arc<SqliteTaskStore>,
    /// Default workspace ID for single-user mode.
    default_workspace_id: Uuid,
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
        })
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
