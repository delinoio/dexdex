//! Single-process runtime implementation.
//!
//! This module provides an embedded server and worker that run in the same process
//! as the Tauri app, using direct function calls instead of network communication.

use std::sync::Arc;

use entities::Workspace;
use task_store::{MemoryTaskStore, TaskStore};
use tracing::info;
use uuid::Uuid;

use crate::error::AppResult;

/// Single-process runtime that embeds server and worker functionality.
pub struct SingleProcessRuntime {
    /// Task store (using in-memory or SQLite storage).
    task_store: Arc<MemoryTaskStore>,
    /// Default workspace ID for single-user mode.
    default_workspace_id: Uuid,
}

impl SingleProcessRuntime {
    /// Creates a new single-process runtime.
    pub async fn new() -> AppResult<Self> {
        info!("Initializing single-process runtime");

        // Create task store (using memory store for now, can be switched to SQLite)
        let task_store = Arc::new(MemoryTaskStore::new());

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
    pub fn task_store_arc(&self) -> Arc<MemoryTaskStore> {
        self.task_store.clone()
    }

    /// Gets the default workspace ID.
    pub fn default_workspace_id(&self) -> Uuid {
        self.default_workspace_id
    }
}
