//! SQLite task store implementation.
//!
//! This module provides a persistent task store backed by SQLite,
//! suitable for single-user desktop applications.

use std::path::Path;

use async_trait::async_trait;
use entities::{
    AgentSession, AgentTask, CompositeTask, CompositeTaskNode, Repository, RepositoryGroup,
    TodoItem, TtyInputRequest, UnitTask, User, VcsProviderType, VcsType, Workspace,
};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use tracing::info;
use uuid::Uuid;

use crate::{
    RepositoryFilter, RepositoryGroupFilter, TaskFilter, TaskStore, TaskStoreError,
    TaskStoreResult, TodoFilter, TtyInputFilter, WorkspaceFilter,
};

/// SQLite-backed task store for persistent storage.
pub struct SqliteTaskStore {
    pool: SqlitePool,
}

impl SqliteTaskStore {
    /// Creates a new SQLite task store at the given path.
    pub async fn new(db_path: &Path) -> TaskStoreResult<Self> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        info!("Opening SQLite database at {}", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let store = Self { pool };
        store.initialize_schema().await?;

        info!("SQLite task store initialized");
        Ok(store)
    }

    /// Initializes the database schema.
    async fn initialize_schema(&self) -> TaskStoreResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                name TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                user_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS repositories (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                name TEXT NOT NULL,
                remote_url TEXT NOT NULL,
                default_branch TEXT NOT NULL,
                vcs_type TEXT NOT NULL,
                vcs_provider_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS repository_groups (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                name TEXT,
                repository_ids TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agent_tasks (
                id TEXT PRIMARY KEY,
                base_remotes TEXT NOT NULL,
                agent_sessions TEXT NOT NULL,
                ai_agent_type TEXT,
                ai_agent_model TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agent_sessions (
                id TEXT PRIMARY KEY,
                agent_task_id TEXT NOT NULL,
                ai_agent_type TEXT NOT NULL,
                ai_agent_model TEXT,
                started_at TEXT,
                completed_at TEXT,
                output_log TEXT,
                token_usage TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS unit_tasks (
                id TEXT PRIMARY KEY,
                repository_group_id TEXT NOT NULL,
                agent_task_id TEXT NOT NULL,
                prompt TEXT NOT NULL,
                title TEXT,
                branch_name TEXT,
                linked_pr_url TEXT,
                base_commit TEXT,
                end_commit TEXT,
                auto_fix_task_ids TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS composite_tasks (
                id TEXT PRIMARY KEY,
                repository_group_id TEXT NOT NULL,
                planning_task_id TEXT NOT NULL,
                prompt TEXT NOT NULL,
                title TEXT,
                node_ids TEXT NOT NULL,
                status TEXT NOT NULL,
                execution_agent_type TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS composite_task_nodes (
                id TEXT PRIMARY KEY,
                composite_task_id TEXT NOT NULL,
                unit_task_id TEXT NOT NULL,
                depends_on_ids TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS todo_items (
                id TEXT PRIMARY KEY,
                item_type TEXT NOT NULL,
                source TEXT NOT NULL,
                status TEXT NOT NULL,
                repository_id TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tty_input_requests (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                prompt TEXT NOT NULL,
                input_type TEXT NOT NULL,
                options TEXT,
                status TEXT NOT NULL,
                response TEXT,
                created_at TEXT NOT NULL,
                responded_at TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Helper to parse UUID from string.
    fn parse_uuid(s: &str) -> TaskStoreResult<Uuid> {
        Uuid::parse_str(s).map_err(|e| TaskStoreError::Other(format!("Invalid UUID: {}", e)))
    }

    /// Helper to parse optional UUID from string.
    fn parse_uuid_opt(s: Option<String>) -> TaskStoreResult<Option<Uuid>> {
        match s {
            Some(s) => Ok(Some(Self::parse_uuid(&s)?)),
            None => Ok(None),
        }
    }

    /// Helper to parse Vec<Uuid> from JSON string.
    fn parse_uuid_vec(s: &str) -> TaskStoreResult<Vec<Uuid>> {
        serde_json::from_str(s).map_err(TaskStoreError::Serialization)
    }

    /// Helper to serialize Vec<Uuid> to JSON string.
    fn serialize_uuid_vec(v: &[Uuid]) -> TaskStoreResult<String> {
        serde_json::to_string(v).map_err(TaskStoreError::Serialization)
    }

    /// Appends LIMIT and OFFSET clauses to a query string.
    ///
    /// # Safety
    ///
    /// This function uses format strings to append pagination values to SQL
    /// queries. This is safe because:
    /// 1. The values are typed as `u32`, which can only contain numeric digits
    /// 2. Rust's Display trait for numeric types produces digit-only output
    /// 3. There is no way for a u32 to contain SQL injection characters
    ///
    /// Additionally, this function enforces reasonable bounds (max 10,000 rows)
    /// to prevent accidental resource exhaustion.
    fn append_pagination(query: &mut String, limit: Option<u32>, offset: Option<u32>) {
        const MAX_LIMIT: u32 = 10_000;

        if let Some(limit) = limit {
            // Cap limit to prevent accidental resource exhaustion
            let safe_limit = limit.min(MAX_LIMIT);
            query.push_str(&format!(" LIMIT {}", safe_limit));
        }
        if let Some(offset) = offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }
    }
}

#[async_trait]
impl TaskStore for SqliteTaskStore {
    // =========================================================================
    // User operations
    // =========================================================================

    async fn create_user(&self, user: User) -> TaskStoreResult<User> {
        sqlx::query(
            "INSERT INTO users (id, email, name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.created_at.to_rfc3339())
        .bind(user.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("User", user.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(user)
    }

    async fn get_user(&self, id: Uuid) -> TaskStoreResult<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: Self::parse_uuid(row.get("id"))?,
                    email: row.get("email"),
                    name: row.get("name"),
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn get_user_by_email(&self, email: &str) -> TaskStoreResult<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: Self::parse_uuid(row.get("id"))?,
                    email: row.get("email"),
                    name: row.get("name"),
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    async fn update_user(&self, user: User) -> TaskStoreResult<User> {
        let result =
            sqlx::query("UPDATE users SET email = ?, name = ?, updated_at = ? WHERE id = ?")
                .bind(&user.email)
                .bind(&user.name)
                .bind(user.updated_at.to_rfc3339())
                .bind(user.id.to_string())
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("User", user.id.to_string()));
        }
        Ok(user)
    }

    async fn delete_user(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("User", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Workspace operations
    // =========================================================================

    async fn create_workspace(&self, workspace: Workspace) -> TaskStoreResult<Workspace> {
        sqlx::query(
            "INSERT INTO workspaces (id, name, description, user_id, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(workspace.id.to_string())
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(workspace.user_id.map(|id| id.to_string()))
        .bind(workspace.created_at.to_rfc3339())
        .bind(workspace.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("Workspace", workspace.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(workspace)
    }

    async fn get_workspace(&self, id: Uuid) -> TaskStoreResult<Option<Workspace>> {
        let row = sqlx::query("SELECT * FROM workspaces WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let workspace = Workspace {
                    id: Self::parse_uuid(row.get("id"))?,
                    name: row.get("name"),
                    description: row.get("description"),
                    user_id: Self::parse_uuid_opt(row.get("user_id"))?,
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(workspace))
            }
            None => Ok(None),
        }
    }

    async fn list_workspaces(
        &self,
        filter: WorkspaceFilter,
    ) -> TaskStoreResult<(Vec<Workspace>, u32)> {
        let mut query = String::from("SELECT * FROM workspaces");
        let mut conditions = Vec::new();

        if filter.user_id.is_some() {
            conditions.push("user_id = ?");
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Get total count first
        let count_query = query.replace("SELECT *", "SELECT COUNT(*) as count");
        let count_row = if let Some(user_id) = filter.user_id {
            sqlx::query(&count_query)
                .bind(user_id.to_string())
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query(&count_query).fetch_one(&self.pool).await?
        };
        let total: i64 = count_row.get("count");

        // Add pagination (uses safe numeric formatting - see append_pagination docs)
        Self::append_pagination(&mut query, filter.limit, filter.offset);

        let rows = if let Some(user_id) = filter.user_id {
            sqlx::query(&query)
                .bind(user_id.to_string())
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await?
        };

        let mut workspaces = Vec::new();
        for row in rows {
            workspaces.push(Workspace {
                id: Self::parse_uuid(row.get("id"))?,
                name: row.get("name"),
                description: row.get("description"),
                user_id: Self::parse_uuid_opt(row.get("user_id"))?,
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok((workspaces, total as u32))
    }

    async fn update_workspace(&self, workspace: Workspace) -> TaskStoreResult<Workspace> {
        let result = sqlx::query(
            "UPDATE workspaces SET name = ?, description = ?, user_id = ?, updated_at = ? WHERE \
             id = ?",
        )
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(workspace.user_id.map(|id| id.to_string()))
        .bind(workspace.updated_at.to_rfc3339())
        .bind(workspace.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "Workspace",
                workspace.id.to_string(),
            ));
        }
        Ok(workspace)
    }

    async fn delete_workspace(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("Workspace", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Repository operations
    // =========================================================================

    async fn create_repository(&self, repository: Repository) -> TaskStoreResult<Repository> {
        sqlx::query(
            "INSERT INTO repositories (id, workspace_id, name, remote_url, default_branch, \
             vcs_type, vcs_provider_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, \
             ?)",
        )
        .bind(repository.id.to_string())
        .bind(repository.workspace_id.to_string())
        .bind(&repository.name)
        .bind(&repository.remote_url)
        .bind(&repository.default_branch)
        .bind(serde_json::to_string(&repository.vcs_type).unwrap())
        .bind(serde_json::to_string(&repository.vcs_provider_type).unwrap())
        .bind(repository.created_at.to_rfc3339())
        .bind(repository.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("Repository", repository.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(repository)
    }

    async fn get_repository(&self, id: Uuid) -> TaskStoreResult<Option<Repository>> {
        let row = sqlx::query("SELECT * FROM repositories WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let vcs_type_str: String = row.get("vcs_type");
                let vcs_provider_str: String = row.get("vcs_provider_type");
                let repository = Repository {
                    id: Self::parse_uuid(row.get("id"))?,
                    workspace_id: Self::parse_uuid(row.get("workspace_id"))?,
                    name: row.get("name"),
                    remote_url: row.get("remote_url"),
                    default_branch: row.get("default_branch"),
                    vcs_type: serde_json::from_str(&vcs_type_str).unwrap_or(VcsType::Git),
                    vcs_provider_type: serde_json::from_str(&vcs_provider_str)
                        .unwrap_or(VcsProviderType::Github),
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(repository))
            }
            None => Ok(None),
        }
    }

    async fn list_repositories(
        &self,
        filter: RepositoryFilter,
    ) -> TaskStoreResult<(Vec<Repository>, u32)> {
        let mut query = String::from("SELECT * FROM repositories");
        let mut conditions = Vec::new();

        if filter.workspace_id.is_some() {
            conditions.push("workspace_id = ?");
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Get total count
        let count_query = query.replace("SELECT *", "SELECT COUNT(*) as count");
        let count_row = if let Some(ws_id) = filter.workspace_id {
            sqlx::query(&count_query)
                .bind(ws_id.to_string())
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query(&count_query).fetch_one(&self.pool).await?
        };
        let total: i64 = count_row.get("count");

        // Add pagination (uses safe numeric formatting - see append_pagination docs)
        Self::append_pagination(&mut query, filter.limit, filter.offset);

        let rows = if let Some(ws_id) = filter.workspace_id {
            sqlx::query(&query)
                .bind(ws_id.to_string())
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await?
        };

        let mut repositories = Vec::new();
        for row in rows {
            let vcs_type_str: String = row.get("vcs_type");
            let vcs_provider_str: String = row.get("vcs_provider_type");
            repositories.push(Repository {
                id: Self::parse_uuid(row.get("id"))?,
                workspace_id: Self::parse_uuid(row.get("workspace_id"))?,
                name: row.get("name"),
                remote_url: row.get("remote_url"),
                default_branch: row.get("default_branch"),
                vcs_type: serde_json::from_str(&vcs_type_str).unwrap_or(VcsType::Git),
                vcs_provider_type: serde_json::from_str(&vcs_provider_str)
                    .unwrap_or(VcsProviderType::Github),
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok((repositories, total as u32))
    }

    async fn update_repository(&self, repository: Repository) -> TaskStoreResult<Repository> {
        let result = sqlx::query(
            "UPDATE repositories SET workspace_id = ?, name = ?, remote_url = ?, default_branch = \
             ?, vcs_type = ?, vcs_provider_type = ?, updated_at = ? WHERE id = ?",
        )
        .bind(repository.workspace_id.to_string())
        .bind(&repository.name)
        .bind(&repository.remote_url)
        .bind(&repository.default_branch)
        .bind(serde_json::to_string(&repository.vcs_type).unwrap())
        .bind(serde_json::to_string(&repository.vcs_provider_type).unwrap())
        .bind(repository.updated_at.to_rfc3339())
        .bind(repository.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "Repository",
                repository.id.to_string(),
            ));
        }
        Ok(repository)
    }

    async fn delete_repository(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM repositories WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("Repository", id.to_string()));
        }

        // Remove the deleted repository from all repository groups
        let repo_id_str = id.to_string();
        let rows = sqlx::query("SELECT id, repository_ids FROM repository_groups")
            .fetch_all(&self.pool)
            .await?;

        for row in rows {
            let group_id: String = row.get("id");
            let repo_ids_str: String = row.get("repository_ids");
            let mut repo_ids: Vec<Uuid> = Self::parse_uuid_vec(&repo_ids_str)?;

            if repo_ids.contains(&id) {
                repo_ids.retain(|&r| r != id);
                let updated_repo_ids_json = Self::serialize_uuid_vec(&repo_ids)?;
                let updated_at = chrono::Utc::now().to_rfc3339();

                sqlx::query(
                    "UPDATE repository_groups SET repository_ids = ?, updated_at = ? WHERE id = ?",
                )
                .bind(&updated_repo_ids_json)
                .bind(&updated_at)
                .bind(&group_id)
                .execute(&self.pool)
                .await?;

                tracing::info!(
                    "Removed repository {} from repository group {}",
                    repo_id_str,
                    group_id
                );
            }
        }

        Ok(())
    }

    // =========================================================================
    // Repository Group operations
    // =========================================================================

    async fn create_repository_group(
        &self,
        group: RepositoryGroup,
    ) -> TaskStoreResult<RepositoryGroup> {
        let repo_ids_json = Self::serialize_uuid_vec(&group.repository_ids)?;
        sqlx::query(
            "INSERT INTO repository_groups (id, workspace_id, name, repository_ids, created_at, \
             updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(group.id.to_string())
        .bind(group.workspace_id.to_string())
        .bind(&group.name)
        .bind(&repo_ids_json)
        .bind(group.created_at.to_rfc3339())
        .bind(group.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("RepositoryGroup", group.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(group)
    }

    async fn get_repository_group(&self, id: Uuid) -> TaskStoreResult<Option<RepositoryGroup>> {
        let row = sqlx::query("SELECT * FROM repository_groups WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let repo_ids_str: String = row.get("repository_ids");
                let group = RepositoryGroup {
                    id: Self::parse_uuid(row.get("id"))?,
                    workspace_id: Self::parse_uuid(row.get("workspace_id"))?,
                    name: row.get("name"),
                    repository_ids: Self::parse_uuid_vec(&repo_ids_str)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    async fn list_repository_groups(
        &self,
        filter: RepositoryGroupFilter,
    ) -> TaskStoreResult<(Vec<RepositoryGroup>, u32)> {
        let mut query = String::from("SELECT * FROM repository_groups");
        let mut conditions = Vec::new();

        if filter.workspace_id.is_some() {
            conditions.push("workspace_id = ?");
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Get total count
        let count_query = query.replace("SELECT *", "SELECT COUNT(*) as count");
        let count_row = if let Some(ws_id) = filter.workspace_id {
            sqlx::query(&count_query)
                .bind(ws_id.to_string())
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query(&count_query).fetch_one(&self.pool).await?
        };
        let total: i64 = count_row.get("count");

        // Add pagination (uses safe numeric formatting - see append_pagination docs)
        Self::append_pagination(&mut query, filter.limit, filter.offset);

        let rows = if let Some(ws_id) = filter.workspace_id {
            sqlx::query(&query)
                .bind(ws_id.to_string())
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(&query).fetch_all(&self.pool).await?
        };

        let mut groups = Vec::new();
        for row in rows {
            let repo_ids_str: String = row.get("repository_ids");
            groups.push(RepositoryGroup {
                id: Self::parse_uuid(row.get("id"))?,
                workspace_id: Self::parse_uuid(row.get("workspace_id"))?,
                name: row.get("name"),
                repository_ids: Self::parse_uuid_vec(&repo_ids_str)?,
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok((groups, total as u32))
    }

    async fn update_repository_group(
        &self,
        group: RepositoryGroup,
    ) -> TaskStoreResult<RepositoryGroup> {
        let repo_ids_json = Self::serialize_uuid_vec(&group.repository_ids)?;
        let result = sqlx::query(
            "UPDATE repository_groups SET workspace_id = ?, name = ?, repository_ids = ?, \
             updated_at = ? WHERE id = ?",
        )
        .bind(group.workspace_id.to_string())
        .bind(&group.name)
        .bind(&repo_ids_json)
        .bind(group.updated_at.to_rfc3339())
        .bind(group.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "RepositoryGroup",
                group.id.to_string(),
            ));
        }
        Ok(group)
    }

    async fn delete_repository_group(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM repository_groups WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("RepositoryGroup", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Agent Task operations
    // =========================================================================

    async fn create_agent_task(&self, task: AgentTask) -> TaskStoreResult<AgentTask> {
        let base_remotes_json = serde_json::to_string(&task.base_remotes)?;
        let sessions_json = serde_json::to_string(&task.agent_sessions)?;
        sqlx::query(
            "INSERT INTO agent_tasks (id, base_remotes, agent_sessions, ai_agent_type, \
             ai_agent_model, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(task.id.to_string())
        .bind(&base_remotes_json)
        .bind(&sessions_json)
        .bind(
            task.ai_agent_type
                .map(|t| serde_json::to_string(&t))
                .transpose()?,
        )
        .bind(&task.ai_agent_model)
        .bind(task.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("AgentTask", task.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(task)
    }

    async fn get_agent_task(&self, id: Uuid) -> TaskStoreResult<Option<AgentTask>> {
        let row = sqlx::query("SELECT * FROM agent_tasks WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let base_remotes_str: String = row.get("base_remotes");
                let sessions_str: String = row.get("agent_sessions");
                let agent_type_str: Option<String> = row.get("ai_agent_type");
                let task = AgentTask {
                    id: Self::parse_uuid(row.get("id"))?,
                    base_remotes: serde_json::from_str(&base_remotes_str)?,
                    agent_sessions: serde_json::from_str(&sessions_str)?,
                    ai_agent_type: agent_type_str
                        .map(|s| serde_json::from_str(&s))
                        .transpose()?,
                    ai_agent_model: row.get("ai_agent_model"),
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    async fn update_agent_task(&self, task: AgentTask) -> TaskStoreResult<AgentTask> {
        let base_remotes_json = serde_json::to_string(&task.base_remotes)?;
        let sessions_json = serde_json::to_string(&task.agent_sessions)?;
        let result = sqlx::query(
            "UPDATE agent_tasks SET base_remotes = ?, agent_sessions = ?, ai_agent_type = ?, \
             ai_agent_model = ? WHERE id = ?",
        )
        .bind(&base_remotes_json)
        .bind(&sessions_json)
        .bind(
            task.ai_agent_type
                .map(|t| serde_json::to_string(&t))
                .transpose()?,
        )
        .bind(&task.ai_agent_model)
        .bind(task.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("AgentTask", task.id.to_string()));
        }
        Ok(task)
    }

    async fn delete_agent_task(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM agent_tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("AgentTask", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Agent Session operations
    // =========================================================================

    async fn create_agent_session(&self, session: AgentSession) -> TaskStoreResult<AgentSession> {
        let token_usage_json = session
            .token_usage
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()?;
        sqlx::query(
            "INSERT INTO agent_sessions (id, agent_task_id, ai_agent_type, ai_agent_model, \
             started_at, completed_at, output_log, token_usage, created_at) VALUES (?, ?, ?, ?, \
             ?, ?, ?, ?, ?)",
        )
        .bind(session.id.to_string())
        .bind(session.agent_task_id.to_string())
        .bind(serde_json::to_string(&session.ai_agent_type)?)
        .bind(&session.ai_agent_model)
        .bind(session.started_at.map(|t| t.to_rfc3339()))
        .bind(session.completed_at.map(|t| t.to_rfc3339()))
        .bind(&session.output_log)
        .bind(&token_usage_json)
        .bind(session.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("AgentSession", session.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(session)
    }

    async fn get_agent_session(&self, id: Uuid) -> TaskStoreResult<Option<AgentSession>> {
        let row = sqlx::query("SELECT * FROM agent_sessions WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let agent_type_str: String = row.get("ai_agent_type");
                let started_at_str: Option<String> = row.get("started_at");
                let completed_at_str: Option<String> = row.get("completed_at");
                let token_usage_str: Option<String> = row.get("token_usage");
                let session = AgentSession {
                    id: Self::parse_uuid(row.get("id"))?,
                    agent_task_id: Self::parse_uuid(row.get("agent_task_id"))?,
                    ai_agent_type: serde_json::from_str(&agent_type_str)?,
                    ai_agent_model: row.get("ai_agent_model"),
                    started_at: started_at_str
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                        })
                        .transpose()
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?,
                    completed_at: completed_at_str
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                        })
                        .transpose()
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?,
                    output_log: row.get("output_log"),
                    token_usage: token_usage_str
                        .map(|s| serde_json::from_str(&s))
                        .transpose()?,
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn list_agent_sessions(&self, agent_task_id: Uuid) -> TaskStoreResult<Vec<AgentSession>> {
        let rows = sqlx::query("SELECT * FROM agent_sessions WHERE agent_task_id = ?")
            .bind(agent_task_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut sessions = Vec::new();
        for row in rows {
            let agent_type_str: String = row.get("ai_agent_type");
            let started_at_str: Option<String> = row.get("started_at");
            let completed_at_str: Option<String> = row.get("completed_at");
            let token_usage_str: Option<String> = row.get("token_usage");
            sessions.push(AgentSession {
                id: Self::parse_uuid(row.get("id"))?,
                agent_task_id: Self::parse_uuid(row.get("agent_task_id"))?,
                ai_agent_type: serde_json::from_str(&agent_type_str)?,
                ai_agent_model: row.get("ai_agent_model"),
                started_at: started_at_str
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?,
                completed_at: completed_at_str
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?,
                output_log: row.get("output_log"),
                token_usage: token_usage_str
                    .map(|s| serde_json::from_str(&s))
                    .transpose()?,
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok(sessions)
    }

    async fn update_agent_session(&self, session: AgentSession) -> TaskStoreResult<AgentSession> {
        let token_usage_json = session
            .token_usage
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()?;
        let result = sqlx::query(
            "UPDATE agent_sessions SET agent_task_id = ?, ai_agent_type = ?, ai_agent_model = ?, \
             started_at = ?, completed_at = ?, output_log = ?, token_usage = ? WHERE id = ?",
        )
        .bind(session.agent_task_id.to_string())
        .bind(serde_json::to_string(&session.ai_agent_type)?)
        .bind(&session.ai_agent_model)
        .bind(session.started_at.map(|t| t.to_rfc3339()))
        .bind(session.completed_at.map(|t| t.to_rfc3339()))
        .bind(&session.output_log)
        .bind(&token_usage_json)
        .bind(session.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "AgentSession",
                session.id.to_string(),
            ));
        }
        Ok(session)
    }

    async fn delete_agent_session(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM agent_sessions WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("AgentSession", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Unit Task operations
    // =========================================================================

    async fn create_unit_task(&self, task: UnitTask) -> TaskStoreResult<UnitTask> {
        let auto_fix_ids_json = Self::serialize_uuid_vec(&task.auto_fix_task_ids)?;
        sqlx::query(
            "INSERT INTO unit_tasks (id, repository_group_id, agent_task_id, prompt, title, \
             branch_name, linked_pr_url, base_commit, end_commit, auto_fix_task_ids, status, \
             created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(task.id.to_string())
        .bind(task.repository_group_id.to_string())
        .bind(task.agent_task_id.to_string())
        .bind(&task.prompt)
        .bind(&task.title)
        .bind(&task.branch_name)
        .bind(&task.linked_pr_url)
        .bind(&task.base_commit)
        .bind(&task.end_commit)
        .bind(&auto_fix_ids_json)
        .bind(serde_json::to_string(&task.status).unwrap())
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("UnitTask", task.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(task)
    }

    async fn get_unit_task(&self, id: Uuid) -> TaskStoreResult<Option<UnitTask>> {
        let row = sqlx::query("SELECT * FROM unit_tasks WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let auto_fix_ids_str: String = row.get("auto_fix_task_ids");
                let status_str: String = row.get("status");
                let task = UnitTask {
                    id: Self::parse_uuid(row.get("id"))?,
                    repository_group_id: Self::parse_uuid(row.get("repository_group_id"))?,
                    agent_task_id: Self::parse_uuid(row.get("agent_task_id"))?,
                    prompt: row.get("prompt"),
                    title: row.get("title"),
                    branch_name: row.get("branch_name"),
                    linked_pr_url: row.get("linked_pr_url"),
                    base_commit: row.get("base_commit"),
                    end_commit: row.get("end_commit"),
                    auto_fix_task_ids: Self::parse_uuid_vec(&auto_fix_ids_str)?,
                    status: serde_json::from_str(&status_str)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    async fn list_unit_tasks(&self, filter: TaskFilter) -> TaskStoreResult<(Vec<UnitTask>, u32)> {
        let mut query = String::from("SELECT * FROM unit_tasks");
        let mut conditions = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(group_id) = filter.repository_group_id {
            conditions.push("repository_group_id = ?");
            binds.push(group_id.to_string());
        }
        if let Some(status) = filter.unit_status {
            conditions.push("status = ?");
            binds.push(serde_json::to_string(&status).unwrap());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Get total count
        let count_query = query.replace("SELECT *", "SELECT COUNT(*) as count");
        let mut count_q = sqlx::query(&count_query);
        for bind in &binds {
            count_q = count_q.bind(bind);
        }
        let count_row = count_q.fetch_one(&self.pool).await?;
        let total: i64 = count_row.get("count");

        // Add pagination (uses safe numeric formatting - see append_pagination docs)
        Self::append_pagination(&mut query, filter.limit, filter.offset);

        let mut q = sqlx::query(&query);
        for bind in &binds {
            q = q.bind(bind);
        }
        let rows = q.fetch_all(&self.pool).await?;

        let mut tasks = Vec::new();
        for row in rows {
            let auto_fix_ids_str: String = row.get("auto_fix_task_ids");
            let status_str: String = row.get("status");
            tasks.push(UnitTask {
                id: Self::parse_uuid(row.get("id"))?,
                repository_group_id: Self::parse_uuid(row.get("repository_group_id"))?,
                agent_task_id: Self::parse_uuid(row.get("agent_task_id"))?,
                prompt: row.get("prompt"),
                title: row.get("title"),
                branch_name: row.get("branch_name"),
                linked_pr_url: row.get("linked_pr_url"),
                base_commit: row.get("base_commit"),
                end_commit: row.get("end_commit"),
                auto_fix_task_ids: Self::parse_uuid_vec(&auto_fix_ids_str)?,
                status: serde_json::from_str(&status_str)?,
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok((tasks, total as u32))
    }

    async fn update_unit_task(&self, task: UnitTask) -> TaskStoreResult<UnitTask> {
        let auto_fix_ids_json = Self::serialize_uuid_vec(&task.auto_fix_task_ids)?;
        let result = sqlx::query(
            "UPDATE unit_tasks SET repository_group_id = ?, agent_task_id = ?, prompt = ?, title \
             = ?, branch_name = ?, linked_pr_url = ?, base_commit = ?, end_commit = ?, \
             auto_fix_task_ids = ?, status = ?, updated_at = ? WHERE id = ?",
        )
        .bind(task.repository_group_id.to_string())
        .bind(task.agent_task_id.to_string())
        .bind(&task.prompt)
        .bind(&task.title)
        .bind(&task.branch_name)
        .bind(&task.linked_pr_url)
        .bind(&task.base_commit)
        .bind(&task.end_commit)
        .bind(&auto_fix_ids_json)
        .bind(serde_json::to_string(&task.status).unwrap())
        .bind(task.updated_at.to_rfc3339())
        .bind(task.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("UnitTask", task.id.to_string()));
        }
        Ok(task)
    }

    async fn delete_unit_task(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM unit_tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("UnitTask", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Composite Task operations
    // =========================================================================

    async fn create_composite_task(&self, task: CompositeTask) -> TaskStoreResult<CompositeTask> {
        let node_ids_json = Self::serialize_uuid_vec(&task.node_ids)?;
        sqlx::query(
            "INSERT INTO composite_tasks (id, repository_group_id, planning_task_id, prompt, \
             title, node_ids, status, execution_agent_type, created_at, updated_at) VALUES (?, ?, \
             ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(task.id.to_string())
        .bind(task.repository_group_id.to_string())
        .bind(task.planning_task_id.to_string())
        .bind(&task.prompt)
        .bind(&task.title)
        .bind(&node_ids_json)
        .bind(serde_json::to_string(&task.status).unwrap())
        .bind(
            task.execution_agent_type
                .map(|t| serde_json::to_string(&t).unwrap()),
        )
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("CompositeTask", task.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(task)
    }

    async fn get_composite_task(&self, id: Uuid) -> TaskStoreResult<Option<CompositeTask>> {
        let row = sqlx::query("SELECT * FROM composite_tasks WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let node_ids_str: String = row.get("node_ids");
                let status_str: String = row.get("status");
                let agent_type_str: Option<String> = row.get("execution_agent_type");
                let task = CompositeTask {
                    id: Self::parse_uuid(row.get("id"))?,
                    repository_group_id: Self::parse_uuid(row.get("repository_group_id"))?,
                    planning_task_id: Self::parse_uuid(row.get("planning_task_id"))?,
                    prompt: row.get("prompt"),
                    title: row.get("title"),
                    node_ids: Self::parse_uuid_vec(&node_ids_str)?,
                    status: serde_json::from_str(&status_str)?,
                    execution_agent_type: agent_type_str
                        .map(|s| serde_json::from_str(&s))
                        .transpose()?,
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    async fn list_composite_tasks(
        &self,
        filter: TaskFilter,
    ) -> TaskStoreResult<(Vec<CompositeTask>, u32)> {
        let mut query = String::from("SELECT * FROM composite_tasks");
        let mut conditions = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(group_id) = filter.repository_group_id {
            conditions.push("repository_group_id = ?");
            binds.push(group_id.to_string());
        }
        if let Some(status) = filter.composite_status {
            conditions.push("status = ?");
            binds.push(serde_json::to_string(&status).unwrap());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Get total count
        let count_query = query.replace("SELECT *", "SELECT COUNT(*) as count");
        let mut count_q = sqlx::query(&count_query);
        for bind in &binds {
            count_q = count_q.bind(bind);
        }
        let count_row = count_q.fetch_one(&self.pool).await?;
        let total: i64 = count_row.get("count");

        // Add pagination (uses safe numeric formatting - see append_pagination docs)
        Self::append_pagination(&mut query, filter.limit, filter.offset);

        let mut q = sqlx::query(&query);
        for bind in &binds {
            q = q.bind(bind);
        }
        let rows = q.fetch_all(&self.pool).await?;

        let mut tasks = Vec::new();
        for row in rows {
            let node_ids_str: String = row.get("node_ids");
            let status_str: String = row.get("status");
            let agent_type_str: Option<String> = row.get("execution_agent_type");
            tasks.push(CompositeTask {
                id: Self::parse_uuid(row.get("id"))?,
                repository_group_id: Self::parse_uuid(row.get("repository_group_id"))?,
                planning_task_id: Self::parse_uuid(row.get("planning_task_id"))?,
                prompt: row.get("prompt"),
                title: row.get("title"),
                node_ids: Self::parse_uuid_vec(&node_ids_str)?,
                status: serde_json::from_str(&status_str)?,
                execution_agent_type: agent_type_str
                    .map(|s| serde_json::from_str(&s))
                    .transpose()?,
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok((tasks, total as u32))
    }

    async fn update_composite_task(&self, task: CompositeTask) -> TaskStoreResult<CompositeTask> {
        let node_ids_json = Self::serialize_uuid_vec(&task.node_ids)?;
        let result = sqlx::query(
            "UPDATE composite_tasks SET repository_group_id = ?, planning_task_id = ?, prompt = \
             ?, title = ?, node_ids = ?, status = ?, execution_agent_type = ?, updated_at = ? \
             WHERE id = ?",
        )
        .bind(task.repository_group_id.to_string())
        .bind(task.planning_task_id.to_string())
        .bind(&task.prompt)
        .bind(&task.title)
        .bind(&node_ids_json)
        .bind(serde_json::to_string(&task.status).unwrap())
        .bind(
            task.execution_agent_type
                .map(|t| serde_json::to_string(&t).unwrap()),
        )
        .bind(task.updated_at.to_rfc3339())
        .bind(task.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "CompositeTask",
                task.id.to_string(),
            ));
        }
        Ok(task)
    }

    async fn delete_composite_task(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM composite_tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("CompositeTask", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // Composite Task Node operations
    // =========================================================================

    async fn create_composite_task_node(
        &self,
        node: CompositeTaskNode,
    ) -> TaskStoreResult<CompositeTaskNode> {
        let depends_on_json = Self::serialize_uuid_vec(&node.depends_on_ids)?;
        sqlx::query(
            "INSERT INTO composite_task_nodes (id, composite_task_id, unit_task_id, \
             depends_on_ids, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(node.id.to_string())
        .bind(node.composite_task_id.to_string())
        .bind(node.unit_task_id.to_string())
        .bind(&depends_on_json)
        .bind(node.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("CompositeTaskNode", node.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(node)
    }

    async fn get_composite_task_node(
        &self,
        id: Uuid,
    ) -> TaskStoreResult<Option<CompositeTaskNode>> {
        let row = sqlx::query("SELECT * FROM composite_task_nodes WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let depends_on_str: String = row.get("depends_on_ids");
                let node = CompositeTaskNode {
                    id: Self::parse_uuid(row.get("id"))?,
                    composite_task_id: Self::parse_uuid(row.get("composite_task_id"))?,
                    unit_task_id: Self::parse_uuid(row.get("unit_task_id"))?,
                    depends_on_ids: Self::parse_uuid_vec(&depends_on_str)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    async fn list_composite_task_nodes(
        &self,
        composite_task_id: Uuid,
    ) -> TaskStoreResult<Vec<CompositeTaskNode>> {
        let rows = sqlx::query("SELECT * FROM composite_task_nodes WHERE composite_task_id = ?")
            .bind(composite_task_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut nodes = Vec::new();
        for row in rows {
            let depends_on_str: String = row.get("depends_on_ids");
            nodes.push(CompositeTaskNode {
                id: Self::parse_uuid(row.get("id"))?,
                composite_task_id: Self::parse_uuid(row.get("composite_task_id"))?,
                unit_task_id: Self::parse_uuid(row.get("unit_task_id"))?,
                depends_on_ids: Self::parse_uuid_vec(&depends_on_str)?,
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok(nodes)
    }

    async fn update_composite_task_node(
        &self,
        node: CompositeTaskNode,
    ) -> TaskStoreResult<CompositeTaskNode> {
        let depends_on_json = Self::serialize_uuid_vec(&node.depends_on_ids)?;
        let result = sqlx::query(
            "UPDATE composite_task_nodes SET composite_task_id = ?, unit_task_id = ?, \
             depends_on_ids = ? WHERE id = ?",
        )
        .bind(node.composite_task_id.to_string())
        .bind(node.unit_task_id.to_string())
        .bind(&depends_on_json)
        .bind(node.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "CompositeTaskNode",
                node.id.to_string(),
            ));
        }
        Ok(node)
    }

    async fn delete_composite_task_node(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM composite_task_nodes WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "CompositeTaskNode",
                id.to_string(),
            ));
        }
        Ok(())
    }

    // =========================================================================
    // Todo Item operations
    // =========================================================================

    async fn create_todo_item(&self, item: TodoItem) -> TaskStoreResult<TodoItem> {
        let data_json = serde_json::to_string(&item.data)?;
        sqlx::query(
            "INSERT INTO todo_items (id, item_type, source, status, repository_id, data, \
             created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(item.id.to_string())
        .bind(serde_json::to_string(&item.item_type).unwrap())
        .bind(serde_json::to_string(&item.source).unwrap())
        .bind(serde_json::to_string(&item.status).unwrap())
        .bind(item.repository_id.to_string())
        .bind(&data_json)
        .bind(item.created_at.to_rfc3339())
        .bind(item.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("TodoItem", item.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(item)
    }

    async fn get_todo_item(&self, id: Uuid) -> TaskStoreResult<Option<TodoItem>> {
        let row = sqlx::query("SELECT * FROM todo_items WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let item_type_str: String = row.get("item_type");
                let source_str: String = row.get("source");
                let status_str: String = row.get("status");
                let data_str: String = row.get("data");
                let item = TodoItem {
                    id: Self::parse_uuid(row.get("id"))?,
                    item_type: serde_json::from_str(&item_type_str)?,
                    source: serde_json::from_str(&source_str)?,
                    status: serde_json::from_str(&status_str)?,
                    repository_id: Self::parse_uuid(row.get("repository_id"))?,
                    data: serde_json::from_str(&data_str)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                };
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    async fn list_todo_items(&self, filter: TodoFilter) -> TaskStoreResult<(Vec<TodoItem>, u32)> {
        let mut query = String::from("SELECT * FROM todo_items");
        let mut conditions = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(repo_id) = filter.repository_id {
            conditions.push("repository_id = ?");
            binds.push(repo_id.to_string());
        }
        if let Some(status) = filter.status {
            conditions.push("status = ?");
            binds.push(serde_json::to_string(&status).unwrap());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Get total count
        let count_query = query.replace("SELECT *", "SELECT COUNT(*) as count");
        let mut count_q = sqlx::query(&count_query);
        for bind in &binds {
            count_q = count_q.bind(bind);
        }
        let count_row = count_q.fetch_one(&self.pool).await?;
        let total: i64 = count_row.get("count");

        // Add pagination (uses safe numeric formatting - see append_pagination docs)
        Self::append_pagination(&mut query, filter.limit, filter.offset);

        let mut q = sqlx::query(&query);
        for bind in &binds {
            q = q.bind(bind);
        }
        let rows = q.fetch_all(&self.pool).await?;

        let mut items = Vec::new();
        for row in rows {
            let item_type_str: String = row.get("item_type");
            let source_str: String = row.get("source");
            let status_str: String = row.get("status");
            let data_str: String = row.get("data");
            items.push(TodoItem {
                id: Self::parse_uuid(row.get("id"))?,
                item_type: serde_json::from_str(&item_type_str)?,
                source: serde_json::from_str(&source_str)?,
                status: serde_json::from_str(&status_str)?,
                repository_id: Self::parse_uuid(row.get("repository_id"))?,
                data: serde_json::from_str(&data_str)?,
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok((items, total as u32))
    }

    async fn update_todo_item(&self, item: TodoItem) -> TaskStoreResult<TodoItem> {
        let data_json = serde_json::to_string(&item.data)?;
        let result = sqlx::query(
            "UPDATE todo_items SET item_type = ?, source = ?, status = ?, repository_id = ?, data \
             = ?, updated_at = ? WHERE id = ?",
        )
        .bind(serde_json::to_string(&item.item_type).unwrap())
        .bind(serde_json::to_string(&item.source).unwrap())
        .bind(serde_json::to_string(&item.status).unwrap())
        .bind(item.repository_id.to_string())
        .bind(&data_json)
        .bind(item.updated_at.to_rfc3339())
        .bind(item.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("TodoItem", item.id.to_string()));
        }
        Ok(item)
    }

    async fn delete_todo_item(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM todo_items WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("TodoItem", id.to_string()));
        }
        Ok(())
    }

    // =========================================================================
    // TTY Input Request operations
    // =========================================================================

    async fn create_tty_input_request(
        &self,
        request: TtyInputRequest,
    ) -> TaskStoreResult<TtyInputRequest> {
        let options_json = request
            .options
            .as_ref()
            .map(|o| serde_json::to_string(o).unwrap());
        sqlx::query(
            "INSERT INTO tty_input_requests (id, task_id, session_id, prompt, input_type, \
             options, status, response, created_at, responded_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, \
             ?, ?)",
        )
        .bind(request.id.to_string())
        .bind(request.task_id.to_string())
        .bind(request.session_id.to_string())
        .bind(&request.prompt)
        .bind(serde_json::to_string(&request.input_type).unwrap())
        .bind(&options_json)
        .bind(serde_json::to_string(&request.status).unwrap())
        .bind(&request.response)
        .bind(request.created_at.to_rfc3339())
        .bind(request.responded_at.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                TaskStoreError::already_exists("TtyInputRequest", request.id.to_string())
            } else {
                TaskStoreError::Database(e)
            }
        })?;
        Ok(request)
    }

    async fn get_tty_input_request(&self, id: Uuid) -> TaskStoreResult<Option<TtyInputRequest>> {
        let row = sqlx::query("SELECT * FROM tty_input_requests WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let input_type_str: String = row.get("input_type");
                let options_str: Option<String> = row.get("options");
                let status_str: String = row.get("status");
                let responded_at_str: Option<String> = row.get("responded_at");
                let request = TtyInputRequest {
                    id: Self::parse_uuid(row.get("id"))?,
                    task_id: Self::parse_uuid(row.get("task_id"))?,
                    session_id: Self::parse_uuid(row.get("session_id"))?,
                    prompt: row.get("prompt"),
                    input_type: serde_json::from_str(&input_type_str)?,
                    options: options_str.map(|s| serde_json::from_str(&s)).transpose()?,
                    status: serde_json::from_str(&status_str)?,
                    response: row.get("response"),
                    created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    responded_at: responded_at_str
                        .map(|s| {
                            chrono::DateTime::parse_from_rfc3339(&s)
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                        })
                        .transpose()
                        .map_err(|e| TaskStoreError::Other(e.to_string()))?,
                };
                Ok(Some(request))
            }
            None => Ok(None),
        }
    }

    async fn list_tty_input_requests(
        &self,
        filter: TtyInputFilter,
    ) -> TaskStoreResult<Vec<TtyInputRequest>> {
        let mut query = String::from("SELECT * FROM tty_input_requests");
        let mut conditions = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(task_id) = filter.task_id {
            conditions.push("task_id = ?");
            binds.push(task_id.to_string());
        }
        if let Some(session_id) = filter.session_id {
            conditions.push("session_id = ?");
            binds.push(session_id.to_string());
        }
        if let Some(status) = filter.status {
            conditions.push("status = ?");
            binds.push(serde_json::to_string(&status).unwrap());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Add pagination (uses safe numeric formatting - see append_pagination docs)
        Self::append_pagination(&mut query, filter.limit, filter.offset);

        let mut q = sqlx::query(&query);
        for bind in &binds {
            q = q.bind(bind);
        }
        let rows = q.fetch_all(&self.pool).await?;

        let mut requests = Vec::new();
        for row in rows {
            let input_type_str: String = row.get("input_type");
            let options_str: Option<String> = row.get("options");
            let status_str: String = row.get("status");
            let responded_at_str: Option<String> = row.get("responded_at");
            requests.push(TtyInputRequest {
                id: Self::parse_uuid(row.get("id"))?,
                task_id: Self::parse_uuid(row.get("task_id"))?,
                session_id: Self::parse_uuid(row.get("session_id"))?,
                prompt: row.get("prompt"),
                input_type: serde_json::from_str(&input_type_str)?,
                options: options_str.map(|s| serde_json::from_str(&s)).transpose()?,
                status: serde_json::from_str(&status_str)?,
                response: row.get("response"),
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?
                    .with_timezone(&chrono::Utc),
                responded_at: responded_at_str
                    .map(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
                    .transpose()
                    .map_err(|e| TaskStoreError::Other(e.to_string()))?,
            });
        }

        Ok(requests)
    }

    async fn update_tty_input_request(
        &self,
        request: TtyInputRequest,
    ) -> TaskStoreResult<TtyInputRequest> {
        let options_json = request
            .options
            .as_ref()
            .map(|o| serde_json::to_string(o).unwrap());
        let result = sqlx::query(
            "UPDATE tty_input_requests SET task_id = ?, session_id = ?, prompt = ?, input_type = \
             ?, options = ?, status = ?, response = ?, responded_at = ? WHERE id = ?",
        )
        .bind(request.task_id.to_string())
        .bind(request.session_id.to_string())
        .bind(&request.prompt)
        .bind(serde_json::to_string(&request.input_type).unwrap())
        .bind(&options_json)
        .bind(serde_json::to_string(&request.status).unwrap())
        .bind(&request.response)
        .bind(request.responded_at.map(|t| t.to_rfc3339()))
        .bind(request.id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found(
                "TtyInputRequest",
                request.id.to_string(),
            ));
        }
        Ok(request)
    }

    async fn delete_tty_input_request(&self, id: Uuid) -> TaskStoreResult<()> {
        let result = sqlx::query("DELETE FROM tty_input_requests WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TaskStoreError::not_found("TtyInputRequest", id.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    struct TestStore {
        store: SqliteTaskStore,
        #[allow(dead_code)]
        _dir: TempDir, // Keep the temp dir alive
    }

    async fn create_test_store() -> TestStore {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteTaskStore::new(&db_path).await.unwrap();
        TestStore { store, _dir: dir }
    }

    #[tokio::test]
    async fn test_workspace_crud() {
        let test = create_test_store().await;
        let store = &test.store;

        // Create
        let workspace = Workspace::new("Test Workspace");
        let created = store.create_workspace(workspace.clone()).await.unwrap();
        assert_eq!(created.name, "Test Workspace");

        // Get
        let fetched = store.get_workspace(created.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "Test Workspace");

        // List
        let (workspaces, count) = store
            .list_workspaces(WorkspaceFilter::default())
            .await
            .unwrap();
        assert_eq!(count, 1);
        assert_eq!(workspaces.len(), 1);

        // Delete
        store.delete_workspace(created.id).await.unwrap();
        assert!(store.get_workspace(created.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_unit_task_crud() {
        let test = create_test_store().await;
        let store = &test.store;

        // Setup
        let workspace = Workspace::new("Test Workspace");
        let workspace = store.create_workspace(workspace).await.unwrap();
        let group = RepositoryGroup::new(workspace.id);
        let group = store.create_repository_group(group).await.unwrap();
        let agent_task = AgentTask::new();
        let agent_task = store.create_agent_task(agent_task).await.unwrap();

        // Create unit task
        let task = UnitTask::new(group.id, agent_task.id, "Fix the bug");
        let created = store.create_unit_task(task).await.unwrap();
        assert_eq!(created.prompt, "Fix the bug");

        // Get
        let fetched = store.get_unit_task(created.id).await.unwrap().unwrap();
        assert_eq!(fetched.prompt, "Fix the bug");

        // List
        let (tasks, count) = store.list_unit_tasks(TaskFilter::default()).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(tasks.len(), 1);

        // Delete
        store.delete_unit_task(created.id).await.unwrap();
        assert!(store.get_unit_task(created.id).await.unwrap().is_none());
    }

    // =========================================================================
    // Pagination Helper Tests
    // =========================================================================

    #[test]
    fn test_append_pagination_with_limit() {
        let mut query = String::from("SELECT * FROM tasks");
        SqliteTaskStore::append_pagination(&mut query, Some(10), None);
        assert_eq!(query, "SELECT * FROM tasks LIMIT 10");
    }

    #[test]
    fn test_append_pagination_with_offset() {
        let mut query = String::from("SELECT * FROM tasks");
        SqliteTaskStore::append_pagination(&mut query, None, Some(20));
        assert_eq!(query, "SELECT * FROM tasks OFFSET 20");
    }

    #[test]
    fn test_append_pagination_with_both() {
        let mut query = String::from("SELECT * FROM tasks");
        SqliteTaskStore::append_pagination(&mut query, Some(10), Some(20));
        assert_eq!(query, "SELECT * FROM tasks LIMIT 10 OFFSET 20");
    }

    #[test]
    fn test_append_pagination_with_none() {
        let mut query = String::from("SELECT * FROM tasks");
        SqliteTaskStore::append_pagination(&mut query, None, None);
        assert_eq!(query, "SELECT * FROM tasks");
    }

    #[test]
    fn test_append_pagination_caps_limit() {
        let mut query = String::from("SELECT * FROM tasks");
        // Request limit higher than MAX_LIMIT (10,000)
        SqliteTaskStore::append_pagination(&mut query, Some(20_000), None);
        assert_eq!(query, "SELECT * FROM tasks LIMIT 10000");
    }

    #[test]
    fn test_append_pagination_allows_reasonable_limit() {
        let mut query = String::from("SELECT * FROM tasks");
        SqliteTaskStore::append_pagination(&mut query, Some(100), None);
        assert_eq!(query, "SELECT * FROM tasks LIMIT 100");
    }

    #[tokio::test]
    async fn test_delete_repository_removes_from_groups() {
        let test = create_test_store().await;
        let store = &test.store;

        // Create workspace
        let workspace = Workspace::new("Test Workspace");
        let workspace = store.create_workspace(workspace).await.unwrap();

        // Create repositories
        let repo1 = Repository::new(
            workspace.id,
            "repo1",
            "https://github.com/test/repo1",
            VcsProviderType::Github,
        );
        let repo1 = store.create_repository(repo1).await.unwrap();

        let repo2 = Repository::new(
            workspace.id,
            "repo2",
            "https://github.com/test/repo2",
            VcsProviderType::Github,
        );
        let repo2 = store.create_repository(repo2).await.unwrap();

        // Create a repository group containing both repos
        let mut group = RepositoryGroup::new(workspace.id);
        group.add_repository(repo1.id);
        group.add_repository(repo2.id);
        let group = store.create_repository_group(group).await.unwrap();

        // Verify both repos are in the group
        let fetched_group = store.get_repository_group(group.id).await.unwrap().unwrap();
        assert_eq!(fetched_group.repository_ids.len(), 2);
        assert!(fetched_group.repository_ids.contains(&repo1.id));
        assert!(fetched_group.repository_ids.contains(&repo2.id));

        // Delete repo1
        store.delete_repository(repo1.id).await.unwrap();

        // Verify repo1 was removed from the group
        let fetched_group = store.get_repository_group(group.id).await.unwrap().unwrap();
        assert_eq!(fetched_group.repository_ids.len(), 1);
        assert!(!fetched_group.repository_ids.contains(&repo1.id));
        assert!(fetched_group.repository_ids.contains(&repo2.id));
    }
}
