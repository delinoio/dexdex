//! Workspace-related entity definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The kind of workspace, determining how it connects to a backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceKind {
    /// Local single-process mode (embedded server and worker).
    #[default]
    Local,
    /// Remote mode (connects to an external server).
    Remote,
}

impl std::fmt::Display for WorkspaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceKind::Local => write!(f, "local"),
            WorkspaceKind::Remote => write!(f, "remote"),
        }
    }
}

/// A workspace groups repositories and tasks together.
///
/// Each workspace has a `kind` that determines whether it runs locally
/// (embedded server) or connects to a remote server. This replaces the
/// previous global app mode concept.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    /// Unique identifier.
    pub id: Uuid,
    /// Workspace name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// The kind of workspace (local or remote).
    pub kind: WorkspaceKind,
    /// Remote server URL (only used when kind is Remote).
    pub server_url: Option<String>,
    /// Associated user ID (None in single-user mode).
    pub user_id: Option<Uuid>,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    /// Creates a new local workspace.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            kind: WorkspaceKind::Local,
            server_url: None,
            user_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new remote workspace.
    pub fn new_remote(name: impl Into<String>, server_url: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            kind: WorkspaceKind::Remote,
            server_url: Some(server_url.into()),
            user_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Sets the description for this workspace.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the user ID for this workspace.
    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Returns true if this workspace is local.
    pub fn is_local(&self) -> bool {
        self.kind == WorkspaceKind::Local
    }

    /// Returns true if this workspace is remote.
    pub fn is_remote(&self) -> bool {
        self.kind == WorkspaceKind::Remote
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let workspace =
            Workspace::new("My Workspace").with_description("A workspace for my projects");

        assert_eq!(workspace.name, "My Workspace");
        assert_eq!(
            workspace.description,
            Some("A workspace for my projects".to_string())
        );
        assert!(workspace.user_id.is_none());
        assert_eq!(workspace.kind, WorkspaceKind::Local);
        assert!(workspace.server_url.is_none());
        assert!(workspace.is_local());
        assert!(!workspace.is_remote());
    }

    #[test]
    fn test_workspace_remote() {
        let workspace = Workspace::new_remote("Remote Workspace", "https://example.com");

        assert_eq!(workspace.name, "Remote Workspace");
        assert_eq!(workspace.kind, WorkspaceKind::Remote);
        assert_eq!(
            workspace.server_url,
            Some("https://example.com".to_string())
        );
        assert!(workspace.is_remote());
        assert!(!workspace.is_local());
    }

    #[test]
    fn test_workspace_with_user() {
        let user_id = Uuid::new_v4();
        let workspace = Workspace::new("Team Workspace").with_user_id(user_id);

        assert_eq!(workspace.user_id, Some(user_id));
    }

    #[test]
    fn test_workspace_kind_serialization() {
        let kind = WorkspaceKind::Remote;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"remote\"");

        let kind = WorkspaceKind::Local;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"local\"");
    }

    #[test]
    fn test_workspace_kind_deserialization() {
        let kind: WorkspaceKind = serde_json::from_str("\"local\"").unwrap();
        assert_eq!(kind, WorkspaceKind::Local);

        let kind: WorkspaceKind = serde_json::from_str("\"remote\"").unwrap();
        assert_eq!(kind, WorkspaceKind::Remote);
    }
}
