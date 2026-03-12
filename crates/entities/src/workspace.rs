//! Workspace-related entity definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A workspace groups repositories and tasks together.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    /// Unique identifier.
    pub id: Uuid,
    /// Workspace name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Associated user ID (None in single-user mode).
    pub user_id: Option<Uuid>,
    /// Optional custom endpoint URL for connecting to a self-hosted server.
    pub endpoint_url: Option<String>,
    /// Optional auth profile ID for authentication configuration.
    pub auth_profile_id: Option<Uuid>,
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    /// Creates a new workspace.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            user_id: None,
            endpoint_url: None,
            auth_profile_id: None,
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

    /// Sets the endpoint URL for this workspace.
    pub fn with_endpoint_url(mut self, endpoint_url: impl Into<String>) -> Self {
        self.endpoint_url = Some(endpoint_url.into());
        self
    }

    /// Sets the auth profile ID for this workspace.
    pub fn with_auth_profile_id(mut self, auth_profile_id: Uuid) -> Self {
        self.auth_profile_id = Some(auth_profile_id);
        self
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
        assert!(workspace.endpoint_url.is_none());
        assert!(workspace.auth_profile_id.is_none());
    }

    #[test]
    fn test_workspace_with_user() {
        let user_id = Uuid::new_v4();
        let workspace = Workspace::new("Team Workspace").with_user_id(user_id);

        assert_eq!(workspace.user_id, Some(user_id));
    }

    #[test]
    fn test_workspace_with_endpoint() {
        let workspace =
            Workspace::new("Cloud Workspace").with_endpoint_url("https://api.example.com");

        assert_eq!(
            workspace.endpoint_url,
            Some("https://api.example.com".to_string())
        );
    }
}
