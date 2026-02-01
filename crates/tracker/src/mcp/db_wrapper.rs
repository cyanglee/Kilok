//! Database wrapper for MCP server
//!
//! Wraps the main database module and provides simplified access for MCP tools

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use std::path::PathBuf;

// Billable hours calculation constants
const BILLABLE_MULTIPLIER: f64 = 1.2;
const BILLABLE_ROUND_UNIT: f64 = 0.5; // hours

/// Calculate billable seconds from raw seconds
/// Formula: raw_hours × 1.2, rounded up to nearest 0.5h
pub fn calculate_billable_seconds(raw_seconds: i64) -> i64 {
    if raw_seconds <= 0 {
        return 0;
    }
    let raw_hours = raw_seconds as f64 / 3600.0;
    let multiplied = raw_hours * BILLABLE_MULTIPLIER;
    let rounded = (multiplied / BILLABLE_ROUND_UNIT).ceil() * BILLABLE_ROUND_UNIT;
    (rounded * 3600.0) as i64
}

// Import from the main crate
// Note: Since this is a separate binary, we need to duplicate some code
// or use the library approach. For now, we'll use direct database access.

use libsql::{params, Builder, Connection};

/// Work item from database
#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: i64,
    pub project_id: i64,
    pub identifier: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub time_adjustment_seconds: i64,
    pub completed_date: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Session from database
#[derive(Debug, Clone)]
pub struct Session {
    pub id: i64,
    pub project_id: i64,
    pub project_name: Option<String>,
    pub branch: String,
    pub work_item: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub active_seconds: Option<i64>,
    pub client_slug: Option<String>,
    pub client_name: Option<String>,
}

/// Commit from database
#[derive(Debug, Clone)]
pub struct Commit {
    pub id: i64,
    pub session_id: i64,
    pub hash: String,
    pub message: Option<String>,
    pub committed_at: Option<DateTime<Utc>>,
    /// Actual time spent working on this commit (calculated from heartbeats)
    pub active_seconds: Option<i64>,
}

/// Work item detail with sessions and commits
#[derive(Debug)]
pub struct WorkItemDetail {
    pub work_item: WorkItem,
    pub total_seconds: i64,
    pub sessions: Vec<(Session, Vec<Commit>)>,
}

/// Database wrapper for MCP operations
#[derive(Clone)]
pub struct DbWrapper {
    conn: Connection,
}

impl DbWrapper {
    pub async fn new() -> Result<Self> {
        // Try to load config to get Turso credentials
        let config = load_config()?;

        let conn = if let (Some(url), Some(token)) = (config.turso_url, config.turso_auth_token) {
            // Use remote mode for MCP server (safer for concurrent access)
            let db = Builder::new_remote(url, token)
                .build()
                .await
                .context("Failed to connect to Turso")?;
            db.connect().context("Failed to connect to database")?
        } else {
            // Fall back to local database
            let db_path = get_local_db_path()?;

            // Ensure parent directory exists
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let db = Builder::new_local(&db_path)
                .build()
                .await
                .context("Failed to open local database")?;
            db.connect().context("Failed to connect to database")?
        };

        let wrapper = Self { conn };
        wrapper.initialize_schema().await?;
        Ok(wrapper)
    }

    /// Initialize database schema including work_items table
    async fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS clients (
                id INTEGER PRIMARY KEY,
                slug TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                git_remote TEXT,
                display_name TEXT,
                work_item_pattern TEXT,
                work_item_source TEXT,
                client_id INTEGER REFERENCES clients(id),
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id),
                branch TEXT NOT NULL,
                work_item TEXT,
                start_commit TEXT,
                end_commit TEXT,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                active_seconds INTEGER,
                status TEXT NOT NULL DEFAULT 'active'
            );

            CREATE TABLE IF NOT EXISTS heartbeats (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL REFERENCES sessions(id),
                timestamp TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS commits (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL REFERENCES sessions(id),
                hash TEXT NOT NULL,
                message TEXT,
                committed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS work_items (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id),
                identifier TEXT NOT NULL,
                title TEXT,
                description TEXT,
                time_adjustment_seconds INTEGER DEFAULT 0,
                completed_date TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT,
                UNIQUE(project_id, identifier)
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions(project_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
            CREATE INDEX IF NOT EXISTS idx_heartbeats_session_id ON heartbeats(session_id);
            CREATE INDEX IF NOT EXISTS idx_commits_session_id ON commits(session_id);
            CREATE INDEX IF NOT EXISTS idx_work_items_project_id ON work_items(project_id);
            "#,
        )
        .await
        .context("Failed to initialize database schema")?;

        // Run migration: add work_item_id to sessions if not exists
        let session_columns: Vec<String> = {
            let mut rows = self.conn.query("PRAGMA table_info(sessions)", ()).await?;
            let mut cols = Vec::new();
            while let Some(row) = rows.next().await? {
                cols.push(row.get::<String>(1)?);
            }
            cols
        };

        if !session_columns.contains(&"work_item_id".to_string()) {
            self.conn.execute(
                "ALTER TABLE sessions ADD COLUMN work_item_id INTEGER REFERENCES work_items(id)",
                (),
            ).await.context("Failed to add work_item_id column")?;

            self.conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_sessions_work_item_id ON sessions(work_item_id)",
                (),
            ).await.context("Failed to create work_item_id index")?;
        }

        // Run migration: add work_item_source to projects if not exists
        let project_columns: Vec<String> = {
            let mut rows = self.conn.query("PRAGMA table_info(projects)", ()).await?;
            let mut cols = Vec::new();
            while let Some(row) = rows.next().await? {
                cols.push(row.get::<String>(1)?);
            }
            cols
        };

        if !project_columns.contains(&"work_item_source".to_string()) {
            self.conn.execute(
                "ALTER TABLE projects ADD COLUMN work_item_source TEXT",
                (),
            ).await.context("Failed to add work_item_source column")?;
        }

        // Run migration: add ignored column to projects if not exists
        if !project_columns.contains(&"ignored".to_string()) {
            self.conn.execute(
                "ALTER TABLE projects ADD COLUMN ignored INTEGER DEFAULT 0",
                (),
            ).await.context("Failed to add ignored column")?;
        }

        Ok(())
    }

    /// List sessions with optional project and month filter
    pub async fn list_sessions(
        &self,
        project: Option<&str>,
        month: Option<&str>,
    ) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();

        // Parse month filter
        let (start, end) = if let Some(m) = month {
            parse_month_range(m)?
        } else {
            (None, None)
        };

        // Build query based on filters
        // All queries JOIN with projects to get display_name (with fallback to path basename)
        // LEFT JOIN clients to get client info
        // COALESCE + REPLACE extracts the last path component when display_name is NULL
        // Use LOWER() for case-insensitive matching
        let query = match (project, start.as_ref(), end.as_ref()) {
            (Some(_), Some(_), Some(_)) => {
                "SELECT s.id, s.project_id,
                        COALESCE(p.display_name, REPLACE(p.path, RTRIM(p.path, REPLACE(p.path, '/', '')), '')) as project_name,
                        s.branch, s.work_item, s.started_at, s.ended_at, s.active_seconds,
                        c.slug as client_slug, c.name as client_name
                 FROM sessions s
                 JOIN projects p ON s.project_id = p.id
                 LEFT JOIN clients c ON p.client_id = c.id
                 WHERE (LOWER(p.path) LIKE ?1 OR LOWER(p.display_name) LIKE ?1)
                   AND s.started_at >= ?2 AND s.started_at < ?3
                   AND s.status = 'completed'
                 ORDER BY s.started_at"
            }
            (Some(_), None, None) => {
                "SELECT s.id, s.project_id,
                        COALESCE(p.display_name, REPLACE(p.path, RTRIM(p.path, REPLACE(p.path, '/', '')), '')) as project_name,
                        s.branch, s.work_item, s.started_at, s.ended_at, s.active_seconds,
                        c.slug as client_slug, c.name as client_name
                 FROM sessions s
                 JOIN projects p ON s.project_id = p.id
                 LEFT JOIN clients c ON p.client_id = c.id
                 WHERE (LOWER(p.path) LIKE ?1 OR LOWER(p.display_name) LIKE ?1)
                   AND s.status = 'completed'
                 ORDER BY s.started_at"
            }
            (None, Some(_), Some(_)) => {
                "SELECT s.id, s.project_id,
                        COALESCE(p.display_name, REPLACE(p.path, RTRIM(p.path, REPLACE(p.path, '/', '')), '')) as project_name,
                        s.branch, s.work_item, s.started_at, s.ended_at, s.active_seconds,
                        c.slug as client_slug, c.name as client_name
                 FROM sessions s
                 JOIN projects p ON s.project_id = p.id
                 LEFT JOIN clients c ON p.client_id = c.id
                 WHERE s.started_at >= ?1 AND s.started_at < ?2
                   AND s.status = 'completed'
                 ORDER BY s.started_at"
            }
            _ => {
                "SELECT s.id, s.project_id,
                        COALESCE(p.display_name, REPLACE(p.path, RTRIM(p.path, REPLACE(p.path, '/', '')), '')) as project_name,
                        s.branch, s.work_item, s.started_at, s.ended_at, s.active_seconds,
                        c.slug as client_slug, c.name as client_name
                 FROM sessions s
                 JOIN projects p ON s.project_id = p.id
                 LEFT JOIN clients c ON p.client_id = c.id
                 WHERE s.status = 'completed'
                 ORDER BY s.started_at"
            }
        };

        let mut rows = match (project, start.as_ref(), end.as_ref()) {
            (Some(p), Some(s), Some(e)) => {
                let pattern = format!("%{}%", p.to_lowercase());
                self.conn
                    .query(query, params![pattern, s.to_rfc3339(), e.to_rfc3339()])
                    .await?
            }
            (Some(p), None, None) => {
                let pattern = format!("%{}%", p.to_lowercase());
                self.conn.query(query, params![pattern]).await?
            }
            (None, Some(s), Some(e)) => {
                self.conn
                    .query(query, params![s.to_rfc3339(), e.to_rfc3339()])
                    .await?
            }
            _ => self.conn.query(query, ()).await?,
        };

        while let Some(row) = rows.next().await? {
            sessions.push(Session {
                id: row.get::<i64>(0)?,
                project_id: row.get::<i64>(1)?,
                project_name: row.get::<Option<String>>(2)?,
                branch: row.get::<String>(3)?,
                work_item: row.get::<Option<String>>(4)?,
                started_at: parse_datetime(row.get::<String>(5)?),
                ended_at: row.get::<Option<String>>(6)?.map(parse_datetime),
                active_seconds: row.get::<Option<i64>>(7)?,
                client_slug: row.get::<Option<String>>(8)?,
                client_name: row.get::<Option<String>>(9)?,
            });
        }

        Ok(sessions)
    }

    /// Get commits for a session (public method for MCP)
    pub async fn get_session_commits_public(&self, session_id: i64) -> Result<Vec<Commit>> {
        self.get_session_commits(session_id).await
    }

    /// List work items with optional project and month filter
    pub async fn list_work_items(
        &self,
        project: Option<&str>,
        month: Option<&str>,
    ) -> Result<Vec<WorkItem>> {
        let mut work_items = Vec::new();

        // Parse month filter
        let (start, end) = if let Some(m) = month {
            parse_month_range(m)?
        } else {
            (None, None)
        };

        // Build query based on filters
        // Use LOWER() for case-insensitive matching
        let query = match (project, start.as_ref(), end.as_ref()) {
            (Some(_), Some(_), Some(_)) => {
                "SELECT w.id, w.project_id, w.identifier, w.title, w.description,
                        w.time_adjustment_seconds, w.completed_date, w.created_at, w.updated_at
                 FROM work_items w
                 JOIN projects p ON w.project_id = p.id
                 WHERE (LOWER(p.path) LIKE ?1 OR LOWER(p.display_name) LIKE ?1)
                   AND w.created_at >= ?2 AND w.created_at < ?3
                 ORDER BY w.created_at DESC"
            }
            (Some(_), None, None) => {
                "SELECT w.id, w.project_id, w.identifier, w.title, w.description,
                        w.time_adjustment_seconds, w.completed_date, w.created_at, w.updated_at
                 FROM work_items w
                 JOIN projects p ON w.project_id = p.id
                 WHERE LOWER(p.path) LIKE ?1 OR LOWER(p.display_name) LIKE ?1
                 ORDER BY w.created_at DESC"
            }
            (None, Some(_), Some(_)) => {
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items
                 WHERE created_at >= ?1 AND created_at < ?2
                 ORDER BY created_at DESC"
            }
            _ => {
                "SELECT id, project_id, identifier, title, description,
                        time_adjustment_seconds, completed_date, created_at, updated_at
                 FROM work_items ORDER BY created_at DESC"
            }
        };

        let mut rows = match (project, start.as_ref(), end.as_ref()) {
            (Some(p), Some(s), Some(e)) => {
                let pattern = format!("%{}%", p.to_lowercase());
                self.conn
                    .query(query, params![pattern, s.to_rfc3339(), e.to_rfc3339()])
                    .await?
            }
            (Some(p), None, None) => {
                let pattern = format!("%{}%", p.to_lowercase());
                self.conn.query(query, params![pattern]).await?
            }
            (None, Some(s), Some(e)) => {
                self.conn
                    .query(query, params![s.to_rfc3339(), e.to_rfc3339()])
                    .await?
            }
            _ => self.conn.query(query, ()).await?,
        };

        while let Some(row) = rows.next().await? {
            work_items.push(row_to_work_item(&row)?);
        }

        Ok(work_items)
    }

    /// Get commits for a work item (via its sessions)
    pub async fn get_work_item_commits(&self, work_item_id: i64) -> Result<Vec<Commit>> {
        let mut commits = Vec::new();

        let mut rows = self
            .conn
            .query(
                "SELECT c.id, c.session_id, c.hash, c.message, c.committed_at, c.active_seconds
             FROM commits c
             JOIN sessions s ON c.session_id = s.id
             WHERE s.work_item_id = ?1
             ORDER BY c.committed_at",
                params![work_item_id],
            )
            .await?;

        while let Some(row) = rows.next().await? {
            commits.push(Commit {
                id: row.get::<i64>(0)?,
                session_id: row.get::<i64>(1)?,
                hash: row.get::<String>(2)?,
                message: row.get::<Option<String>>(3)?,
                committed_at: row.get::<Option<String>>(4)?.map(|s| parse_datetime(s)),
                active_seconds: row.get::<Option<i64>>(5)?,
            });
        }

        Ok(commits)
    }

    /// Calculate total seconds for a work item
    pub async fn calculate_work_item_total_seconds(&self, work_item_id: i64) -> Result<i64> {
        let mut rows = self
            .conn
            .query(
                "SELECT
                COALESCE(SUM(s.active_seconds), 0) + COALESCE(w.time_adjustment_seconds, 0)
             FROM work_items w
             LEFT JOIN sessions s ON s.work_item_id = w.id
             WHERE w.id = ?1
             GROUP BY w.id",
                params![work_item_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get::<i64>(0)?)
        } else {
            Ok(0)
        }
    }

    /// Get work item detail by ID or identifier
    pub async fn get_work_item_detail(
        &self,
        work_item_id: Option<i64>,
        identifier: Option<&str>,
        project: Option<&str>,
    ) -> Result<WorkItemDetail> {
        // Find work item
        let work_item = if let Some(id) = work_item_id {
            self.get_work_item_by_id(id).await?
        } else if let (Some(ident), Some(proj)) = (identifier, project) {
            self.get_work_item_by_identifier(ident, proj).await?
        } else {
            anyhow::bail!("Either work_item_id or (identifier + project) must be provided")
        };

        // Get sessions for this work item
        let mut sessions_with_commits = Vec::new();
        let mut session_rows = self
            .conn
            .query(
                "SELECT s.id, s.project_id,
                        COALESCE(p.display_name, REPLACE(p.path, RTRIM(p.path, REPLACE(p.path, '/', '')), '')) as project_name,
                        s.branch, s.work_item, s.started_at, s.ended_at, s.active_seconds
                 FROM sessions s
                 JOIN projects p ON s.project_id = p.id
                 WHERE s.work_item_id = ?1
                 ORDER BY s.started_at",
                params![work_item.id],
            )
            .await?;

        while let Some(row) = session_rows.next().await? {
            let session = Session {
                id: row.get::<i64>(0)?,
                project_id: row.get::<i64>(1)?,
                project_name: row.get::<Option<String>>(2)?,
                branch: row.get::<String>(3)?,
                work_item: row.get::<Option<String>>(4)?,
                started_at: parse_datetime(row.get::<String>(5)?),
                ended_at: row.get::<Option<String>>(6)?.map(parse_datetime),
                active_seconds: row.get::<Option<i64>>(7)?,
                // These are not needed for work item detail, so set to None
                client_slug: None,
                client_name: None,
            };

            // Get commits for this session
            let commits = self.get_session_commits(session.id).await?;
            sessions_with_commits.push((session, commits));
        }

        // Calculate total seconds
        let session_seconds: i64 = sessions_with_commits
            .iter()
            .filter_map(|(s, _)| s.active_seconds)
            .sum();
        let total_seconds = session_seconds + work_item.time_adjustment_seconds;

        Ok(WorkItemDetail {
            work_item,
            total_seconds,
            sessions: sessions_with_commits,
        })
    }

    async fn get_work_item_by_id(&self, id: i64) -> Result<WorkItem> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, project_id, identifier, title, description,
                    time_adjustment_seconds, completed_date, created_at, updated_at
             FROM work_items WHERE id = ?1",
                params![id],
            )
            .await?;

        let row = rows.next().await?.context("Work item not found")?;
        row_to_work_item(&row)
    }

    async fn get_work_item_by_identifier(&self, identifier: &str, project: &str) -> Result<WorkItem> {
        let pattern = format!("%{}%", project);
        let mut rows = self
            .conn
            .query(
                "SELECT w.id, w.project_id, w.identifier, w.title, w.description,
                    w.time_adjustment_seconds, w.completed_date, w.created_at, w.updated_at
             FROM work_items w
             JOIN projects p ON w.project_id = p.id
             WHERE w.identifier = ?1 AND (p.path LIKE ?2 OR p.display_name LIKE ?2)",
                params![identifier, pattern],
            )
            .await?;

        let row = rows.next().await?.context("Work item not found")?;
        row_to_work_item(&row)
    }

    async fn get_session_commits(&self, session_id: i64) -> Result<Vec<Commit>> {
        let mut commits = Vec::new();
        let mut rows = self
            .conn
            .query(
                "SELECT id, session_id, hash, message, committed_at, active_seconds FROM commits
             WHERE session_id = ?1 ORDER BY committed_at",
                params![session_id],
            )
            .await?;

        while let Some(row) = rows.next().await? {
            commits.push(Commit {
                id: row.get::<i64>(0)?,
                session_id: row.get::<i64>(1)?,
                hash: row.get::<String>(2)?,
                message: row.get::<Option<String>>(3)?,
                committed_at: row.get::<Option<String>>(4)?.map(|s| parse_datetime(s)),
                active_seconds: row.get::<Option<i64>>(5)?,
            });
        }

        Ok(commits)
    }

    /// Get project's work_item_source
    pub async fn get_project_source(&self, project_id: i64) -> Result<Option<String>> {
        let mut rows = self
            .conn
            .query(
                "SELECT work_item_source FROM projects WHERE id = ?1",
                params![project_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get::<Option<String>>(0)?)
        } else {
            Ok(None)
        }
    }

    /// Get or create a work item by identifier
    pub async fn get_or_create_work_item(
        &self,
        identifier: &str,
        project: &str,
    ) -> Result<WorkItem> {
        // Build case-insensitive pattern for LIKE search
        let pattern = format!("%{}%", project.to_lowercase());

        // First try to find existing work item
        let mut rows = self
            .conn
            .query(
                "SELECT w.id, w.project_id, w.identifier, w.title, w.description,
                    w.time_adjustment_seconds, w.completed_date, w.created_at, w.updated_at
                 FROM work_items w
                 JOIN projects p ON w.project_id = p.id
                 WHERE w.identifier = ?1 AND (LOWER(p.path) LIKE ?2 OR LOWER(p.display_name) LIKE ?2)",
                params![identifier, pattern.clone()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            return row_to_work_item(&row);
        }

        // Find project ID - use case-insensitive search on path and display_name
        let mut project_rows = self
            .conn
            .query(
                "SELECT id FROM projects WHERE LOWER(path) LIKE ?1 OR LOWER(display_name) LIKE ?1",
                params![pattern],
            )
            .await?;

        let project_row = project_rows.next().await?.context(format!("Project not found: {}", project))?;
        let project_id: i64 = project_row.get(0)?;

        // Create new work item
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO work_items (project_id, identifier, created_at) VALUES (?1, ?2, ?3)",
            params![project_id, identifier, now.to_rfc3339()],
        ).await?;

        // Fetch the created work item
        let new_id = self.conn.last_insert_rowid();
        self.get_work_item_by_id(new_id).await
    }

    /// Update work item
    pub async fn update_work_item(
        &self,
        work_item_id: i64,
        title: Option<&str>,
        description: Option<&str>,
        time_adjustment_seconds: Option<i64>,
        completed_date: Option<&str>,
    ) -> Result<WorkItem> {
        let now = Utc::now();

        // Build dynamic update query
        let mut updates = vec!["updated_at = ?1".to_string()];
        let mut params: Vec<libsql::Value> = vec![now.to_rfc3339().into()];
        let mut param_idx = 2;

        if let Some(t) = title {
            updates.push(format!("title = ?{}", param_idx));
            params.push(t.into());
            param_idx += 1;
        }

        if let Some(d) = description {
            updates.push(format!("description = ?{}", param_idx));
            params.push(d.into());
            param_idx += 1;
        }

        if let Some(adj) = time_adjustment_seconds {
            updates.push(format!("time_adjustment_seconds = ?{}", param_idx));
            params.push(adj.into());
            param_idx += 1;
        }

        if let Some(cd) = completed_date {
            updates.push(format!("completed_date = ?{}", param_idx));
            params.push(cd.into());
            param_idx += 1;
        }

        params.push(work_item_id.into());

        let query = format!(
            "UPDATE work_items SET {} WHERE id = ?{}",
            updates.join(", "),
            param_idx
        );

        self.conn.execute(&query, params).await?;
        self.get_work_item_by_id(work_item_id).await
    }

    /// List all projects with optional filter
    pub async fn list_projects(&self, filter: Option<&str>) -> Result<Vec<Project>> {
        let mut projects = Vec::new();

        let (query, params): (&str, Vec<libsql::Value>) = if let Some(f) = filter {
            let pattern = format!("%{}%", f.to_lowercase());
            (
                "SELECT p.id, p.path, p.display_name,
                        (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count
                 FROM projects p
                 WHERE LOWER(p.path) LIKE ?1 OR LOWER(p.display_name) LIKE ?1
                 ORDER BY session_count DESC",
                vec![pattern.into()],
            )
        } else {
            (
                "SELECT p.id, p.path, p.display_name,
                        (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count
                 FROM projects p
                 ORDER BY session_count DESC",
                vec![],
            )
        };

        let mut rows = self.conn.query(query, params).await?;

        while let Some(row) = rows.next().await? {
            projects.push(Project {
                id: row.get::<i64>(0)?,
                path: row.get::<String>(1)?,
                display_name: row.get::<Option<String>>(2)?,
                session_count: row.get::<i64>(3)?,
            });
        }

        Ok(projects)
    }

    /// Update a project's display name
    pub async fn update_project_display_name(&self, project_id: i64, display_name: &str) -> Result<Project> {
        self.conn
            .execute(
                "UPDATE projects SET display_name = ?1 WHERE id = ?2",
                params![display_name, project_id],
            )
            .await?;

        // Fetch updated project
        let mut rows = self
            .conn
            .query(
                "SELECT p.id, p.path, p.display_name,
                        (SELECT COUNT(*) FROM sessions WHERE project_id = p.id) as session_count
                 FROM projects p WHERE p.id = ?1",
                params![project_id],
            )
            .await?;

        let row = rows.next().await?.context("Project not found")?;
        Ok(Project {
            id: row.get::<i64>(0)?,
            path: row.get::<String>(1)?,
            display_name: row.get::<Option<String>>(2)?,
            session_count: row.get::<i64>(3)?,
        })
    }
}

/// Project from database
#[derive(Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub path: String,
    pub display_name: Option<String>,
    pub session_count: i64,
}

// Helper functions

fn row_to_work_item(row: &libsql::Row) -> Result<WorkItem> {
    Ok(WorkItem {
        id: row.get::<i64>(0)?,
        project_id: row.get::<i64>(1)?,
        identifier: row.get::<String>(2)?,
        title: row.get::<Option<String>>(3)?,
        description: row.get::<Option<String>>(4)?,
        time_adjustment_seconds: row.get::<i64>(5)?,
        completed_date: row.get::<Option<String>>(6)?,
        created_at: parse_datetime(row.get::<String>(7)?),
        updated_at: row.get::<Option<String>>(8)?.map(parse_datetime),
    })
}

fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_month_range(month: &str) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>)> {
    // Parse "YYYY-MM" format
    let parts: Vec<&str> = month.split('-').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid month format. Expected YYYY-MM");
    }

    let year: i32 = parts[0].parse().context("Invalid year")?;
    let month_num: u32 = parts[1].parse().context("Invalid month")?;

    let start_date = NaiveDate::from_ymd_opt(year, month_num, 1)
        .context("Invalid date")?;
    let start = Utc.from_utc_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap());

    // Calculate end of month
    let (next_year, next_month) = if month_num == 12 {
        (year + 1, 1)
    } else {
        (year, month_num + 1)
    };

    let end_date = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .context("Invalid end date")?;
    let end = Utc.from_utc_datetime(&end_date.and_hms_opt(0, 0, 0).unwrap());

    Ok((Some(start), Some(end)))
}

fn get_local_db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .context("Could not find local data directory")?
        .join("claude-time-tracker");
    Ok(data_dir.join("data.db"))
}

#[derive(Default)]
struct Config {
    turso_url: Option<String>,
    turso_auth_token: Option<String>,
}

fn load_config() -> Result<Config> {
    // Try to load from config file (use ~/.config/ for consistency across platforms)
    let config_path = dirs::home_dir()
        .map(|d| d.join(".config").join("claude-time-tracker").join("config.toml"));

    if let Some(path) = config_path {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let parsed: toml::Value = content.parse()?;

            return Ok(Config {
                turso_url: parsed
                    .get("turso")
                    .and_then(|t| t.get("url"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                turso_auth_token: parsed
                    .get("turso")
                    .and_then(|t| t.get("auth_token"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
            });
        }
    }

    // Also check environment variables
    Ok(Config {
        turso_url: std::env::var("TURSO_DATABASE_URL").ok(),
        turso_auth_token: std::env::var("TURSO_AUTH_TOKEN").ok(),
    })
}
